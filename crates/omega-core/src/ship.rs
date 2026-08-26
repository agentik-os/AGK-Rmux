//! Ship pipeline — full 12-step build→commit→push→deploy→verify (R-14).
//!
//! Mirrors the live system's oracle-ship.sh as typed Rust with
//! async steps, gitleaks scan, file whitelisting, lock serialization,
//! freeze-on-failure, and deploy verification.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipConfig {
    pub build_command: String,
    pub deploy_command: Option<String>,
    pub deploy_timeout_secs: u64,
    pub auto_rollback: bool,
    pub freeze_on_fail: bool,
    pub verify_url: Option<String>,
}

impl Default for ShipConfig {
    fn default() -> Self {
        Self {
            build_command: "cargo build --release".to_string(),
            deploy_command: None,
            deploy_timeout_secs: 600,
            auto_rollback: false,
            freeze_on_fail: true,
            verify_url: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipResult {
    pub result: ShipOutcome,
    pub commit: Option<String>,
    pub deploy_url: Option<String>,
    pub steps_completed: Vec<ShipStep>,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShipOutcome {
    Ok,
    Failed,
    Skipped,
    Frozen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipStep {
    pub name: String,
    pub passed: bool,
    pub output: Option<String>,
    pub duration_ms: u64,
}

pub struct ShipPipeline {
    project_dir: PathBuf,
    state_dir: PathBuf,
    config: ShipConfig,
}

impl ShipPipeline {
    pub fn new(project_dir: PathBuf, state_dir: PathBuf, config: ShipConfig) -> Self {
        Self {
            project_dir,
            state_dir,
            config,
        }
    }

    pub fn is_frozen(&self, project: &str) -> bool {
        self.freeze_path(project).exists()
    }

    fn freeze_path(&self, project: &str) -> PathBuf {
        self.state_dir.join(format!("ship-{}.frozen", project))
    }

    fn lock_path(&self, project: &str) -> PathBuf {
        self.state_dir.join(format!("ship-{}.lock", project))
    }

    /// Execute the full 12-step ship pipeline.
    ///
    /// Steps: frozen-check → build → stage (whitelisted) → gitleaks →
    /// whitespace → commit → lock → frozen-recheck → rebase → push →
    /// deploy → verify-deploy.
    pub async fn execute(
        &self,
        project: &str,
        commit_msg: &str,
        files_owned: &[String],
    ) -> ShipResult {
        let started_at = Utc::now();
        let mut steps = Vec::new();

        // Step 1: Frozen check (fast-path before taking the lock; re-checked under lock below)
        if self.is_frozen(project) {
            return ShipResult {
                result: ShipOutcome::Frozen,
                commit: None,
                deploy_url: None,
                steps_completed: steps,
                error: Some("Ship pipeline frozen — resolve before retrying".to_string()),
                started_at,
                finished_at: Utc::now(),
            };
        }

        // Step 2: Acquire ship lock BEFORE any build/commit so the entire
        // critical section (build→commit→push→deploy) is serialized across
        // oracles. Acquiring after the commit would leave an orphaned unpushed
        // commit if the lock is contended, and let two oracles commit to the
        // same branch before either serialized (racy rebase baseline).
        if let Err(e) = self.acquire_lock(project) {
            return self.fail(steps, &format!("Lock error: {}", e), started_at);
        }

        // From here on every early-return MUST release the lock first.

        // Step 3: Build
        match self.run_step("build", &self.config.build_command).await {
            Ok(step) => {
                let passed = step.passed;
                steps.push(step);
                if !passed {
                    self.release_lock(project);
                    return self.fail(steps, "Build failed", started_at);
                }
            }
            Err(e) => {
                self.release_lock(project);
                return self.fail(steps, &format!("Build error: {}", e), started_at);
            }
        }

        // Step 4: Stage files (whitelisted via files_owned, or all)
        let stage_cmd = if files_owned.is_empty() {
            "git add -A".to_string()
        } else {
            let escaped: Vec<String> = files_owned
                .iter()
                .map(|f| format!("'{}'", f.replace('\'', "'\\''")))
                .collect();
            format!("git add {}", escaped.join(" "))
        };
        match self.run_step("stage", &stage_cmd).await {
            Ok(step) => steps.push(step),
            Err(e) => {
                self.release_lock(project);
                return self.fail(steps, &format!("Stage error: {}", e), started_at);
            }
        }

        // Step 5: Secret scan (gitleaks)
        let gitleaks_cmd = "if command -v gitleaks >/dev/null 2>&1; then gitleaks detect --staged --no-banner; else echo 'gitleaks not installed, skipping'; fi";
        match self.run_step("gitleaks", gitleaks_cmd).await {
            Ok(step) => {
                let passed = step.passed;
                steps.push(step);
                if !passed {
                    self.release_lock(project);
                    return self.fail(steps, "Secrets detected in staged files", started_at);
                }
            }
            Err(e) => {
                self.release_lock(project);
                return self.fail(steps, &format!("Gitleaks error: {}", e), started_at);
            }
        }

        // Step 6: Whitespace sanity
        match self
            .run_step("whitespace", "git diff --cached --check")
            .await
        {
            Ok(step) => {
                let passed = step.passed;
                steps.push(step);
                if !passed {
                    self.release_lock(project);
                    return self.fail(steps, "Whitespace issues in staged files", started_at);
                }
            }
            Err(e) => {
                self.release_lock(project);
                return self.fail(steps, &format!("Whitespace check error: {}", e), started_at);
            }
        }

        // Step 7: Commit
        let commit_cmd = format!("git commit -m '{}'", commit_msg.replace('\'', "'\\''"));
        let commit_hash = match self.run_step("commit", &commit_cmd).await {
            Ok(step) => {
                let passed = step.passed;
                steps.push(step);
                if passed {
                    self.get_commit_hash().await.ok()
                } else {
                    None
                }
            }
            Err(e) => {
                self.release_lock(project);
                return self.fail(steps, &format!("Commit error: {}", e), started_at);
            }
        };

        // Step 8: Re-check frozen (under lock — another oracle may have frozen)
        if self.is_frozen(project) {
            self.release_lock(project);
            return ShipResult {
                result: ShipOutcome::Frozen,
                commit: commit_hash,
                deploy_url: None,
                steps_completed: steps,
                error: Some("Ship frozen after lock acquisition".to_string()),
                started_at,
                finished_at: Utc::now(),
            };
        }

        // Step 9: Rebase
        match self.run_step("rebase", "git pull --rebase").await {
            Ok(step) => {
                let passed = step.passed;
                steps.push(step);
                if !passed {
                    self.release_lock(project);
                    return self.fail(steps, "Rebase failed — resolve conflicts", started_at);
                }
            }
            Err(e) => {
                self.release_lock(project);
                return self.fail(steps, &format!("Rebase error: {}", e), started_at);
            }
        }

        // Step 10: Push
        match self.run_step("push", "git push").await {
            Ok(step) => {
                let passed = step.passed;
                steps.push(step);
                if !passed {
                    self.release_lock(project);
                    return self.fail(steps, "Push failed", started_at);
                }
            }
            Err(e) => {
                self.release_lock(project);
                return self.fail(steps, &format!("Push error: {}", e), started_at);
            }
        }

        // Step 11: Deploy (optional)
        let deploy_url = if let Some(ref deploy_cmd) = self.config.deploy_command {
            match self.run_step("deploy", deploy_cmd).await {
                Ok(step) => {
                    let passed = step.passed;
                    let output = step.output.clone();
                    steps.push(step);
                    if !passed {
                        if self.config.freeze_on_fail {
                            let _ = self.freeze(project);
                        }
                        self.release_lock(project);
                        return self.fail(steps, "Deploy failed", started_at);
                    }
                    output
                }
                Err(e) => {
                    if self.config.freeze_on_fail {
                        let _ = self.freeze(project);
                    }
                    self.release_lock(project);
                    return self.fail(steps, &format!("Deploy error: {}", e), started_at);
                }
            }
        } else {
            None
        };

        // Step 12: Verify deploy (HTTP 200 check, optional)
        if let Some(ref url) = self.config.verify_url {
            let verify_cmd = format!(
                "curl -sf -o /dev/null -w '%{{http_code}}' '{}'",
                url.replace('\'', "'\\''")
            );
            match self.run_step("verify-deploy", &verify_cmd).await {
                Ok(step) => {
                    let passed = step.passed;
                    steps.push(step);
                    if !passed {
                        if self.config.freeze_on_fail {
                            let _ = self.freeze(project);
                        }
                        self.release_lock(project);
                        return self.fail(
                            steps,
                            "Deploy verification failed — non-200 response",
                            started_at,
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("Deploy verification error (non-fatal): {}", e);
                }
            }
        }

        self.release_lock(project);

        ShipResult {
            result: ShipOutcome::Ok,
            commit: commit_hash,
            deploy_url,
            steps_completed: steps,
            error: None,
            started_at,
            finished_at: Utc::now(),
        }
    }

    async fn run_step(&self, name: &str, command: &str) -> Result<ShipStep> {
        let start = std::time::Instant::now();
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&self.project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context(format!("Failed to execute ship step '{}'", name))?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = if stderr.is_empty() {
            stdout.to_string()
        } else {
            format!("{}\n{}", stdout, stderr)
        };

        Ok(ShipStep {
            name: name.to_string(),
            passed: output.status.success(),
            output: if combined.trim().is_empty() {
                None
            } else {
                Some(combined.chars().take(2000).collect())
            },
            duration_ms,
        })
    }

    async fn get_commit_hash(&self) -> Result<String> {
        let output = tokio::process::Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .current_dir(&self.project_dir)
            .output()
            .await?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn freeze(&self, project: &str) -> Result<()> {
        let path = self.freeze_path(project);
        std::fs::write(
            &path,
            format!("Frozen at {}\n", Utc::now().format("%Y-%m-%dT%H:%M:%SZ")),
        )?;
        tracing::warn!(project = %project, "Ship pipeline FROZEN due to deploy failure");
        Ok(())
    }

    pub fn unfreeze(&self, project: &str) -> Result<()> {
        let path = self.freeze_path(project);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    fn acquire_lock(&self, project: &str) -> Result<()> {
        let path = self.lock_path(project);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                anyhow::bail!("Ship lock held by another oracle for {}", project)
            }
            Err(e) => Err(e.into()),
        }
    }

    fn release_lock(&self, project: &str) {
        let _ = std::fs::remove_file(self.lock_path(project));
    }

    pub fn write_result(&self, project: &str, result: &ShipResult) -> Result<()> {
        let path = self.state_dir.join(format!("ship-{}.result.json", project));
        let content = serde_json::to_string_pretty(result)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    fn fail(&self, steps: Vec<ShipStep>, error: &str, started_at: DateTime<Utc>) -> ShipResult {
        ShipResult {
            result: ShipOutcome::Failed,
            commit: None,
            deploy_url: None,
            steps_completed: steps,
            error: Some(error.to_string()),
            started_at,
            finished_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = ShipConfig::default();
        assert_eq!(cfg.build_command, "cargo build --release");
        assert!(cfg.deploy_command.is_none());
        assert!(cfg.freeze_on_fail);
        assert!(!cfg.auto_rollback);
        assert!(cfg.verify_url.is_none());
    }

    #[test]
    fn frozen_detection() {
        let dir = tempfile::tempdir().unwrap();
        let pipeline = ShipPipeline::new(
            PathBuf::from("."),
            dir.path().to_path_buf(),
            ShipConfig::default(),
        );
        assert!(!pipeline.is_frozen("test-project"));

        std::fs::write(dir.path().join("ship-test-project.frozen"), "frozen\n").unwrap();
        assert!(pipeline.is_frozen("test-project"));

        pipeline.unfreeze("test-project").unwrap();
        assert!(!pipeline.is_frozen("test-project"));
    }

    #[test]
    fn lock_acquire_release() {
        let dir = tempfile::tempdir().unwrap();
        let pipeline = ShipPipeline::new(
            PathBuf::from("."),
            dir.path().to_path_buf(),
            ShipConfig::default(),
        );

        pipeline.acquire_lock("proj").unwrap();
        assert!(dir.path().join("ship-proj.lock").exists());

        // Second acquire should fail
        let err = pipeline.acquire_lock("proj");
        assert!(err.is_err());

        pipeline.release_lock("proj");
        assert!(!dir.path().join("ship-proj.lock").exists());

        // After release, acquire works again
        pipeline.acquire_lock("proj").unwrap();
        pipeline.release_lock("proj");
    }

    #[test]
    fn result_serialization() {
        let dir = tempfile::tempdir().unwrap();
        let pipeline = ShipPipeline::new(
            PathBuf::from("."),
            dir.path().to_path_buf(),
            ShipConfig::default(),
        );
        let result = ShipResult {
            result: ShipOutcome::Ok,
            commit: Some("abc1234".into()),
            deploy_url: None,
            steps_completed: vec![ShipStep {
                name: "build".into(),
                passed: true,
                output: None,
                duration_ms: 500,
            }],
            error: None,
            started_at: Utc::now(),
            finished_at: Utc::now(),
        };
        pipeline.write_result("test", &result).unwrap();
        assert!(dir.path().join("ship-test.result.json").exists());
    }
}
