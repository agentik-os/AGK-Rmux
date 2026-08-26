//! OmegaUsage — Claude token-budget monitor + Telegram threshold alerts.
//!
//! Reinterpreted from the legacy AISB `usage-monitor.sh` (bash) into a
//! native OmegaOS Rust subcommand. Same intent, OmegaOS-aligned plumbing:
//!
//! - Reads the OAuth usage endpoint (`/api/oauth/usage`) for the active
//!   account's 5-hour + 7-day utilization. This is the authoritative
//!   source when Anthropic serves it.
//! - The alert metric is `max(session_5h%, week_7d%)` — whichever cap is
//!   closest to being hit, exactly like the old monitor.
//! - Proactive alerts at 80% / 85% / 90% / 95% — yellow under 90, red at/above.
//!   Only the HIGHEST threshold crossed fires on a given tick, and each
//!   threshold has its own 30-minute cooldown file under the resolved OmegaOS
//!   state root (`$OMEGA_DIR/state`, consolidated install, or legacy fallback)
//!   so we don't spam.
//! - Alerts go out through the SAME Telegram bot the bridge uses
//!   (the strict `OmegaTelegramConfig` authority), formatted with a usage bar.
//!
//! Scheduled by cron (`omega usage --check`, every 10 min) — set up by
//! install.sh so a fresh clone gets the alert automatically.
//!
//! NOTE: account auto-switching (the old 90% shadow-rotate) is NOT ported
//! here — OmegaOS handles multi-account differently. This module is
//! strictly the ALERT half the user asked for.

use anyhow::{Context, Result};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const OAUTH_USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const COOLDOWN_SECS: u64 = 1800; // 30 minutes per threshold
static ATOMIC_WRITE_SERIAL: AtomicU64 = AtomicU64::new(0);

#[cfg(unix)]
struct AlertDeliveryLock {
    _file: std::fs::File,
}

#[cfg(unix)]
impl AlertDeliveryLock {
    fn try_acquire(path: &Path) -> Result<Option<Self>> {
        let parent = path
            .parent()
            .with_context(|| format!("alert lock {} has no parent", path.display()))?;
        ensure_real_directory(parent)?;
        let mut options = std::fs::OpenOptions::new();
        options.read(true).write(true).create(true);
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
        #[cfg(target_os = "linux")]
        options.custom_flags(0o400000); // O_NOFOLLOW
        let file = options
            .open(path)
            .with_context(|| format!("opening usage alert lock {}", path.display()))?;
        let metadata = file
            .metadata()
            .with_context(|| format!("inspecting usage alert lock {}", path.display()))?;
        if !metadata.file_type().is_file() {
            anyhow::bail!("refusing non-regular usage alert lock {}", path.display());
        }
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 {
            anyhow::bail!("refusing hard-linked usage alert lock {}", path.display());
        }
        use std::os::fd::AsRawFd;
        const LOCK_EX: std::os::raw::c_int = 2;
        const LOCK_NB: std::os::raw::c_int = 4;
        unsafe extern "C" {
            fn flock(
                fd: std::os::raw::c_int,
                operation: std::os::raw::c_int,
            ) -> std::os::raw::c_int;
        }
        // SAFETY: `file` owns a valid open descriptor for the duration of the
        // call, and flock does not retain the pointer or access Rust memory.
        if unsafe { flock(file.as_raw_fd(), LOCK_EX | LOCK_NB) } == 0 {
            Ok(Some(Self { _file: file }))
        } else {
            let error = std::io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(11) | Some(35)) {
                Ok(None)
            } else {
                Err(error).with_context(|| format!("locking usage alert state {}", path.display()))
            }
        }
    }
}

#[cfg(unix)]
impl Drop for AlertDeliveryLock {
    fn drop(&mut self) {
        use std::os::fd::AsRawFd;
        const LOCK_UN: std::os::raw::c_int = 8;
        unsafe extern "C" {
            fn flock(
                fd: std::os::raw::c_int,
                operation: std::os::raw::c_int,
            ) -> std::os::raw::c_int;
        }
        // SAFETY: `_file` still owns a valid descriptor while Drop runs. An
        // unlock failure cannot be recovered here; closing the file directly
        // afterwards is the kernel's second release path.
        let _ = unsafe { flock(self._file.as_raw_fd(), LOCK_UN) };
    }
}

/// Utilization snapshot for the active account.
#[derive(Debug, Clone, Default)]
pub struct UsageSnapshot {
    pub session_pct: u32, // 5-hour window
    pub week_pct: u32,    // 7-day window
}

impl UsageSnapshot {
    /// The metric the alert fires on — closest cap to being hit.
    pub fn alert_pct(&self) -> u32 {
        self.session_pct.max(self.week_pct)
    }
}

fn usage_state_dir(omega_dir: &Path) -> PathBuf {
    omega_dir.join("state")
}

fn state_dir() -> PathBuf {
    usage_state_dir(&omega_core::config::omega_dir())
}

fn ensure_real_directory(path: &Path) -> Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {
            Ok(())
        }
        Ok(_) => anyhow::bail!(
            "refusing non-directory or symlink usage state path {}",
            path.display()
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => std::fs::create_dir_all(path)
            .with_context(|| format!("creating usage state directory {}", path.display())),
        Err(error) => Err(error)
            .with_context(|| format!("inspecting usage state directory {}", path.display())),
    }
}

/// Read the OAuth access token from Claude's credentials file.
fn oauth_token() -> Result<String> {
    let home = dirs::home_dir().context("no home directory for Claude credentials")?;
    let cred = home.join(".claude/.credentials.json");
    let content = std::fs::read_to_string(&cred)
        .with_context(|| format!("reading OAuth credentials at {}", cred.display()))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("parsing OAuth credentials at {}", cred.display()))?;
    // Token may be at .claudeAiOauth.accessToken or .accessToken
    let token = json
        .get("claudeAiOauth")
        .and_then(|v| v.get("accessToken"))
        .or_else(|| json.get("accessToken"))
        .and_then(|v| v.as_str())
        .filter(|token| !token.trim().is_empty())
        .context("OAuth credentials contain no non-empty access token")?;
    Ok(token.to_string())
}

fn utilization(json: &serde_json::Value, key: &str) -> Result<u32> {
    let value = match json.get(key) {
        Some(serde_json::Value::Object(object)) => object.get("utilization"),
        value => value,
    }
    .and_then(serde_json::Value::as_f64)
    .with_context(|| format!("usage response has no numeric {key}.utilization"))?;
    if !value.is_finite() || value < 0.0 || value > u32::MAX as f64 {
        anyhow::bail!("usage response has invalid {key}.utilization");
    }
    Ok(value as u32)
}

async fn fetch_usage_from(
    client: &reqwest::Client,
    url: &str,
    token: &str,
) -> Result<UsageSnapshot> {
    let resp = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("anthropic-beta", "oauth-2025-04-20")
        .send()
        .await
        .context("requesting authoritative OAuth usage snapshot")?;
    if !resp.status().is_success() {
        anyhow::bail!("OAuth usage endpoint returned HTTP {}", resp.status());
    }
    let json: serde_json::Value = resp
        .json()
        .await
        .context("parsing authoritative OAuth usage response")?;

    Ok(UsageSnapshot {
        session_pct: utilization(&json, "five_hour")?,
        week_pct: utilization(&json, "seven_day")?,
    })
}

/// Fetch utilization from the OAuth endpoint. Authentication, transport,
/// HTTP, and schema failures are errors: none is an authoritative zero.
pub async fn fetch_usage(client: &reqwest::Client) -> Result<UsageSnapshot> {
    let token = oauth_token()?;
    fetch_usage_from(client, OAUTH_USAGE_URL, &token).await
}

/// Read the validated Telegram authority from the resolved OmegaOS root.
fn telegram_creds() -> Result<(String, i64)> {
    let config = omega_core::monitor::OmegaTelegramConfig::try_read()?
        .context("Telegram is not configured")?;
    Ok(telegram_destination(&config))
}

fn telegram_destination(config: &omega_core::monitor::OmegaTelegramConfig) -> (String, i64) {
    (config.bot_token.clone(), config.chat_id)
}

fn cooldown_elapsed(flag: &str) -> Result<u64> {
    let path = state_dir().join(flag);
    let meta = match std::fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(u64::MAX),
        Err(error) => return Err(error).with_context(|| format!("inspecting {}", path.display())),
    };
    if !meta.file_type().is_file() {
        anyhow::bail!("refusing non-regular usage cooldown {}", path.display());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if meta.nlink() != 1 {
            anyhow::bail!("refusing hard-linked usage cooldown {}", path.display());
        }
    }
    let modified = meta
        .modified()
        .with_context(|| format!("reading cooldown timestamp at {}", path.display()))?;
    let modified_secs = modified
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Ok(now.saturating_sub(modified_secs))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("{} has no parent directory", path.display()))?;
    ensure_real_directory(parent)?;
    let serial = ATOMIC_WRITE_SERIAL.fetch_add(1, Ordering::Relaxed);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("usage");
    let staged = parent.join(format!(".{name}.tmp-{}-{serial}", std::process::id()));
    let result = (|| -> Result<()> {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&staged)
            .with_context(|| format!("creating staged usage state {}", staged.display()))?;
        file.write_all(bytes)
            .with_context(|| format!("writing staged usage state {}", staged.display()))?;
        file.sync_all()
            .with_context(|| format!("syncing staged usage state {}", staged.display()))?;
        std::fs::rename(&staged, path)
            .with_context(|| format!("publishing usage state {}", path.display()))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&staged);
    }
    result
}

fn usage_bar(pct: u32) -> String {
    let filled = (pct / 5).min(20) as usize;
    let empty = 20usize.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// Highest usage threshold crossed (95 → 90 → 85 → 80), or None below 80.
/// The alert fires only on this one so a single tick never double-alerts.
pub fn threshold_for(pct: u32) -> Option<u32> {
    [95u32, 90, 85, 80].into_iter().find(|&t| pct >= t)
}

async fn send_alert(client: &reqwest::Client, snap: &UsageSnapshot, threshold: u32) -> Result<()> {
    let (token, chat_id) = telegram_creds()?;
    let pct = snap.alert_pct();
    let red = threshold >= 90;
    let icon = if red { "🔴" } else { "🟡" };
    let bar = usage_bar(pct);
    // Branded to match the Telegram bot's Ω card grammar (status / model views).
    let text = format!(
        "━━━━━━━━━━━━━━━━━━━\n\
         {icon} <b>USAGE {threshold}%</b> · <code>{pct}%</code>\n\
         ━━━━━━━━━━━━━━━━━━━\n\
         <code>{bar}</code>\n\n\
         <b>5h:</b> {sess}%   <b>Week:</b> {week}%\n\n\
         <i>Approche de la limite. /clean ou switch de compte si besoin.</i>",
        icon = icon,
        threshold = threshold,
        pct = pct,
        bar = bar,
        sess = snap.session_pct,
        week = snap.week_pct,
    );
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let body = serde_json::json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "HTML",
    });
    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|_| anyhow::anyhow!("Telegram alert request failed"))?;
    if !response.status().is_success() {
        anyhow::bail!("Telegram alert returned HTTP {}", response.status());
    }
    let response: serde_json::Value = response
        .json()
        .await
        .map_err(|_| anyhow::anyhow!("Telegram alert returned malformed JSON"))?;
    if response.get("ok").and_then(serde_json::Value::as_bool) != Some(true) {
        anyhow::bail!("Telegram alert API rejected the delivery");
    }
    Ok(())
}

fn record_alert_delivery(delivery: Result<()>, flag: &str) -> Result<()> {
    record_alert_delivery_at(delivery, &state_dir().join(flag))
}

fn record_alert_delivery_at(delivery: Result<()>, path: &Path) -> Result<()> {
    delivery?;
    atomic_write(path, b"")
}

/// One monitor tick: fetch usage, evaluate thresholds, alert if crossed
/// (respecting cooldowns). Returns the snapshot for logging.
pub async fn check_and_alert() -> Result<Option<UsageSnapshot>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let snap = fetch_usage(&client).await?;

    let pct = snap.alert_pct();

    // Persist the authoritative snapshot even if alert delivery later fails.
    let dir = state_dir();
    let snapshot = serde_json::json!({
        "session_pct": snap.session_pct,
        "week_pct": snap.week_pct,
        "alert_pct": pct,
        "ts": chrono::Utc::now().to_rfc3339(),
    })
    .to_string();
    atomic_write(&dir.join("usage.json"), snapshot.as_bytes())?;

    // Fire only the HIGHEST threshold crossed (95/90/85/80), each with its own
    // 30-min cooldown so escalating usage still escalates the alert.
    if let Some(threshold) = threshold_for(pct) {
        let flag = format!("usage-alert-{}-sent", threshold);
        #[cfg(unix)]
        let Some(_delivery_lock) =
            AlertDeliveryLock::try_acquire(&dir.join(format!(".{flag}.lock")))?
        else {
            // A concurrent checker owns delivery. It will publish the cooldown
            // only after Telegram confirms success.
            return Ok(Some(snap));
        };
        if cooldown_elapsed(&flag)? >= COOLDOWN_SECS {
            // The cooldown is a delivery receipt, not an attempt receipt. A
            // failed HTTP/API delivery remains retryable on the next tick.
            let delivery = send_alert(&client, &snap, threshold).await;
            record_alert_delivery(delivery, &flag)?;
        }
    }

    Ok(Some(snap))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "omega-usage-{name}-{}-{}",
            std::process::id(),
            ATOMIC_WRITE_SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn alert_pct_is_max() {
        let s = UsageSnapshot {
            session_pct: 72,
            week_pct: 81,
        };
        assert_eq!(s.alert_pct(), 81);
    }

    #[test]
    fn threshold_picks_highest_crossed() {
        assert_eq!(threshold_for(72), None);
        assert_eq!(threshold_for(80), Some(80));
        assert_eq!(threshold_for(84), Some(80));
        assert_eq!(threshold_for(85), Some(85));
        assert_eq!(threshold_for(89), Some(85));
        assert_eq!(threshold_for(90), Some(90));
        assert_eq!(threshold_for(94), Some(90));
        assert_eq!(threshold_for(95), Some(95));
        assert_eq!(threshold_for(100), Some(95));
    }

    #[test]
    fn usage_bar_fills() {
        assert_eq!(usage_bar(50).chars().filter(|c| *c == '█').count(), 10);
        assert_eq!(usage_bar(100).chars().filter(|c| *c == '█').count(), 20);
        assert_eq!(usage_bar(0).chars().filter(|c| *c == '█').count(), 0);
    }

    #[test]
    fn usage_bar_clamps_over_100() {
        assert_eq!(usage_bar(150).chars().count(), 20);
    }

    #[test]
    fn malformed_usage_is_not_reported_as_authoritative_zero() {
        let missing = serde_json::json!({"five_hour": {"utilization": 12}});
        assert!(utilization(&missing, "seven_day").is_err());
        let invalid = serde_json::json!({"five_hour": {"utilization": -1}});
        assert!(utilization(&invalid, "five_hour").is_err());
    }

    #[test]
    fn usage_state_and_alert_destination_follow_canonical_authority() {
        assert_eq!(
            usage_state_dir(Path::new("/srv/omega")),
            PathBuf::from("/srv/omega/state")
        );
        let config = omega_core::monitor::OmegaTelegramConfig {
            bot_token: "123:test".to_string(),
            chat_id: 9001,
            allow_user_ids: vec![42],
            relay_session: "aisb-master".to_string(),
            label: String::new(),
            enabled: true,
        };
        assert_eq!(
            telegram_destination(&config),
            ("123:test".to_string(), 9001),
            "alerts target the configured chat_id, never a sender allowlist id"
        );
    }

    #[test]
    fn cooldown_is_recorded_only_after_delivery() {
        let temp = test_dir("cooldown");
        let cooldown = temp.join("usage-alert-80-sent");
        let failure = record_alert_delivery_at(
            Err(anyhow::anyhow!("simulated delivery failure")),
            &cooldown,
        );
        assert!(failure.is_err());
        assert!(!cooldown.exists());

        record_alert_delivery_at(Ok(()), &cooldown).unwrap();
        assert!(cooldown.is_file());
        std::fs::remove_dir_all(temp).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn atomic_state_publish_does_not_follow_symlink_or_mutate_hardlink_target() {
        let temp = test_dir("aliases");
        let target = temp.join("target");
        std::fs::write(&target, b"do not change").unwrap();

        let symlink = temp.join("usage.json");
        std::os::unix::fs::symlink(&target, &symlink).unwrap();
        atomic_write(&symlink, b"safe snapshot").unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"do not change");
        assert_eq!(std::fs::read(&symlink).unwrap(), b"safe snapshot");
        assert!(!std::fs::symlink_metadata(&symlink)
            .unwrap()
            .file_type()
            .is_symlink());

        let hardlink = temp.join("cooldown");
        std::fs::hard_link(&target, &hardlink).unwrap();
        atomic_write(&hardlink, b"receipt").unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"do not change");
        assert_eq!(std::fs::read(&hardlink).unwrap(), b"receipt");

        let real_state = temp.join("real-state");
        std::fs::create_dir(&real_state).unwrap();
        let linked_state = temp.join("linked-state");
        std::os::unix::fs::symlink(&real_state, &linked_state).unwrap();
        assert!(atomic_write(&linked_state.join("usage.json"), b"blocked").is_err());
        assert!(!real_state.join("usage.json").exists());
        std::fs::remove_dir_all(temp).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn alert_delivery_lock_serializes_checkers_and_rejects_aliases() {
        let temp = test_dir("lock");
        let lock_path = temp.join("alert.lock");
        let first = AlertDeliveryLock::try_acquire(&lock_path)
            .unwrap()
            .expect("first checker owns the lock");
        assert!(AlertDeliveryLock::try_acquire(&lock_path)
            .unwrap()
            .is_none());
        drop(first);
        assert!(AlertDeliveryLock::try_acquire(&lock_path)
            .unwrap()
            .is_some());

        let target = temp.join("target");
        std::fs::write(&target, b"authority").unwrap();
        let symlink = temp.join("symlink.lock");
        std::os::unix::fs::symlink(&target, &symlink).unwrap();
        assert!(AlertDeliveryLock::try_acquire(&symlink).is_err());
        let hardlink = temp.join("hardlink.lock");
        std::fs::hard_link(&target, &hardlink).unwrap();
        assert!(AlertDeliveryLock::try_acquire(&hardlink).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), b"authority");
        std::fs::remove_dir_all(temp).unwrap();
    }
}
