//! Intent verifier — compare intent JSON vs actual outcome (Wave-4).
//!
//! Runs verify_commands (60% weight) + delta assessment (40% weight).
//! Loops if composite score < 75% (R-14).

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Stdio;

/// Structured intent parsed from user text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentSpec {
    pub action: String,
    pub target: String,
    pub success_criteria: Vec<String>,
    pub verify_commands: Vec<String>,
}

/// Result of intent verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResult {
    pub command_score: f32,
    pub delta_score: f32,
    pub composite_score: f32,
    pub passed: bool,
    pub command_results: Vec<CommandResult>,
    pub delta_analysis: String,
    pub iteration: u32,
    pub verified_at: DateTime<Utc>,
}

/// Result of a single verify command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub passed: bool,
    pub duration_ms: u64,
}

const COMMAND_WEIGHT: f32 = 0.6;
const DELTA_WEIGHT: f32 = 0.4;
const DEFAULT_THRESHOLD: f32 = 75.0;
const DEFAULT_MAX_ITERATIONS: u32 = 3;
const OUTPUT_CAP: usize = 2000;

pub struct IntentVerifier {
    max_iterations: u32,
    threshold: f32,
}

impl Default for IntentVerifier {
    fn default() -> Self {
        Self {
            max_iterations: DEFAULT_MAX_ITERATIONS,
            threshold: DEFAULT_THRESHOLD,
        }
    }
}

impl IntentVerifier {
    pub fn new(max_iterations: u32, threshold: f32) -> Self {
        Self {
            max_iterations,
            threshold,
        }
    }

    /// Single verification pass: run commands + assess delta.
    pub async fn verify_once(
        &self,
        intent: &IntentSpec,
        working_dir: &Path,
        iteration: u32,
    ) -> Result<VerifyResult> {
        let command_results = self
            .run_commands(&intent.verify_commands, working_dir)
            .await?;
        let total = command_results.len().max(1);
        let passed_count = command_results.iter().filter(|c| c.passed).count();
        let command_score = (passed_count as f32 / total as f32) * 100.0;

        let delta_score = Self::assess_delta(intent, &command_results);
        let composite = command_score * COMMAND_WEIGHT + delta_score * DELTA_WEIGHT;

        Ok(VerifyResult {
            command_score,
            delta_score,
            composite_score: composite,
            passed: composite >= self.threshold,
            command_results,
            delta_analysis: Self::format_delta(intent, passed_count, total),
            iteration,
            verified_at: Utc::now(),
        })
    }

    /// Full verify loop: run verify_once, retry if below threshold.
    pub async fn verify_loop(
        &self,
        intent: &IntentSpec,
        working_dir: &Path,
    ) -> Result<VerifyResult> {
        let mut result = self.verify_once(intent, working_dir, 1).await?;
        for i in 2..=self.max_iterations {
            if result.passed {
                break;
            }
            tracing::warn!(
                iteration = i,
                score = result.composite_score,
                threshold = self.threshold,
                "Intent verification below threshold, retrying"
            );
            result = self.verify_once(intent, working_dir, i).await?;
        }
        Ok(result)
    }

    async fn run_commands(
        &self,
        commands: &[String],
        working_dir: &Path,
    ) -> Result<Vec<CommandResult>> {
        let mut results = Vec::with_capacity(commands.len());
        for cmd in commands {
            let start = std::time::Instant::now();
            let output = tokio::process::Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .current_dir(working_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                // Guardian wraps this future in a timeout: when it fires, the
                // future is DROPPED — without kill_on_drop the verify child
                // would keep running detached forever.
                .kill_on_drop(true)
                .output()
                .await?;

            results.push(CommandResult {
                command: cmd.clone(),
                exit_code: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout)
                    .chars()
                    .take(OUTPUT_CAP)
                    .collect(),
                stderr: String::from_utf8_lossy(&output.stderr)
                    .chars()
                    .take(OUTPUT_CAP)
                    .collect(),
                passed: output.status.success(),
                duration_ms: start.elapsed().as_millis() as u64,
            });
        }
        Ok(results)
    }

    fn assess_delta(intent: &IntentSpec, results: &[CommandResult]) -> f32 {
        if intent.success_criteria.is_empty() {
            return if results.is_empty() {
                50.0
            } else {
                let p = results.iter().filter(|r| r.passed).count();
                (p as f32 / results.len() as f32) * 100.0
            };
        }

        let mut satisfied = 0u32;
        for criterion in &intent.success_criteria {
            let lower_crit = criterion.to_lowercase();
            // Require explicit evidence per criterion: a passing command whose
            // output actually mentions the criterion. Command success alone
            // (all_pass) proves execution, not intent satisfaction.
            let matched = results
                .iter()
                .any(|r| r.passed && r.stdout.to_lowercase().contains(&lower_crit));
            if matched {
                satisfied += 1;
            }
        }
        (satisfied as f32 / intent.success_criteria.len() as f32) * 100.0
    }

    fn format_delta(intent: &IntentSpec, passed: usize, total: usize) -> String {
        format!(
            "Action: {} on {} | Commands: {}/{} passed | Criteria: {}",
            intent.action,
            intent.target,
            passed,
            total,
            intent.success_criteria.len()
        )
    }

    pub fn write_result(state_dir: &Path, oracle: &str, result: &VerifyResult) -> Result<()> {
        let path = state_dir.join(format!("{}.verify-result.json", oracle));
        std::fs::write(&path, serde_json::to_string_pretty(result)?)?;
        Ok(())
    }

    pub fn read_result(state_dir: &Path, oracle: &str) -> Result<Option<VerifyResult>> {
        let path = state_dir.join(format!("{}.verify-result.json", oracle));
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(serde_json::from_str(&std::fs::read_to_string(
            &path,
        )?)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_intent(criteria: Vec<&str>, commands: Vec<&str>) -> IntentSpec {
        IntentSpec {
            action: "fix".into(),
            target: "auth".into(),
            success_criteria: criteria.into_iter().map(String::from).collect(),
            verify_commands: commands.into_iter().map(String::from).collect(),
        }
    }

    fn cmd_result(passed: bool) -> CommandResult {
        CommandResult {
            command: "test".into(),
            exit_code: if passed { 0 } else { 1 },
            stdout: String::new(),
            stderr: String::new(),
            passed,
            duration_ms: 10,
        }
    }

    fn cmd_result_with_stdout(passed: bool, stdout: &str) -> CommandResult {
        CommandResult {
            command: "test".into(),
            exit_code: if passed { 0 } else { 1 },
            stdout: stdout.into(),
            stderr: String::new(),
            passed,
            duration_ms: 10,
        }
    }

    #[test]
    fn delta_all_pass_without_evidence_satisfies_no_criteria() {
        // Commands succeed but their output does not mention the criteria:
        // success proves execution, not intent satisfaction.
        let intent = make_intent(vec!["login works", "no errors"], vec![]);
        let results = vec![cmd_result(true)];
        let score = IntentVerifier::assess_delta(&intent, &results);
        assert!(score.abs() < 0.1);
    }

    #[test]
    fn delta_criterion_satisfied_only_with_matching_evidence() {
        let intent = make_intent(vec!["login works", "no errors"], vec![]);
        let results = vec![cmd_result_with_stdout(true, "login works fine")];
        let score = IntentVerifier::assess_delta(&intent, &results);
        // 1 of 2 criteria has evidence in output.
        assert!((score - 50.0).abs() < 0.1);
    }

    #[test]
    fn delta_no_criteria_uses_command_ratio() {
        let intent = make_intent(vec![], vec![]);
        let results = vec![cmd_result(true), cmd_result(false)];
        let score = IntentVerifier::assess_delta(&intent, &results);
        assert!((score - 50.0).abs() < 0.1);
    }

    #[test]
    fn delta_empty_results_returns_50() {
        let intent = make_intent(vec![], vec![]);
        let score = IntentVerifier::assess_delta(&intent, &[]);
        assert!((score - 50.0).abs() < 0.1);
    }

    #[test]
    fn composite_weight_check() {
        let composite = 100.0 * COMMAND_WEIGHT + 50.0 * DELTA_WEIGHT;
        assert!((composite - 80.0).abs() < 0.1);
    }

    #[test]
    fn format_delta_includes_counts() {
        let intent = make_intent(vec!["a", "b"], vec![]);
        let s = IntentVerifier::format_delta(&intent, 3, 5);
        assert!(s.contains("3/5"));
        assert!(s.contains("Criteria: 2"));
    }

    #[test]
    fn default_config() {
        let v = IntentVerifier::default();
        assert_eq!(v.max_iterations, 3);
        assert!((v.threshold - 75.0).abs() < 0.1);
    }

    #[test]
    fn custom_config() {
        let v = IntentVerifier::new(5, 80.0);
        assert_eq!(v.max_iterations, 5);
        assert!((v.threshold - 80.0).abs() < 0.1);
    }

    #[test]
    fn result_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let result = VerifyResult {
            command_score: 100.0,
            delta_score: 80.0,
            composite_score: 92.0,
            passed: true,
            command_results: vec![],
            delta_analysis: "all good".into(),
            iteration: 1,
            verified_at: Utc::now(),
        };
        IntentVerifier::write_result(dir.path(), "test-oracle", &result).unwrap();
        let loaded = IntentVerifier::read_result(dir.path(), "test-oracle")
            .unwrap()
            .unwrap();
        assert!((loaded.composite_score - 92.0).abs() < 0.1);
    }
}
