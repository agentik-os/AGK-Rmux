//! Monitor — read-only view into Claude Code billing, accounts, and AISB bot
//! activity. This module NEVER modifies the AISB system; it only reads cache
//! files and credential files that the AISB system already maintains.
//!
//! Files we read (read-only):
//! - `$OMEGA_DIR/state/usage.json`     — live billing percentages (written by
//!   the native `omega usage --check` cron)
//! - `~/.claude/.credentials.json`     — current OAuth credentials
//! - `~/.claude/accounts/*.json`       — saved account profiles
//! - `~/.omega/telegram.toml`          — Omega's own Telegram bot config
//!
//! Files we WRITE (Omega-owned only):
//! - `~/.omega/telegram.toml`          — when user sets up Omega Telegram bot
//! - `~/.omega/monitor.last.json`      — cache of last-seen snapshot for diffs

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// (removed) legacy /tmp/aisb-usage.json — billing now reads the configured OmegaOS state.

/// Live billing snapshot read from the AISB usage cache.
/// Values mirror what the AISB bot's `/billing` command shows.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageSnapshot {
    #[serde(default)]
    pub session_pct: u32,
    #[serde(default)]
    pub week_pct: u32,
    #[serde(default)]
    pub sonnet_pct: u32,
    #[serde(default)]
    pub extra_pct: u32,
    #[serde(default)]
    pub pct_5h_precise: f32,
    #[serde(default)]
    pub pct_week_precise: f32,
    #[serde(default)]
    pub tokens_5h: u64,
    #[serde(default)]
    pub tokens_7d: u64,
    #[serde(default)]
    pub sessions_5h: u32,
    #[serde(default)]
    pub sessions_7d: u32,
    #[serde(default)]
    pub budget_5h: u64,
    #[serde(default)]
    pub budget_week: u64,
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub active_account: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub source: String,
    /// ISO-8601 timestamp from the cache file
    #[serde(default)]
    pub ts: String,
}

impl UsageSnapshot {
    /// Path to the ACCURATE usage cache written by `omega usage --check`
    /// (real OAuth utilization endpoint). Preferred over the legacy
    /// /tmp/aisb-usage.json which holds a local-token ESTIMATE that
    /// over-reports (showed 89% when the real 5h was 36%).
    fn omega_usage_path() -> std::path::PathBuf {
        crate::config::omega_dir().join("state/usage.json")
    }

    /// Read the usage snapshot from `$OMEGA_DIR/state/usage.json` (written natively
    /// by `omega usage --check` from the OAuth endpoint). Omega-owned, zero
    /// legacy dependency — returns None if absent (e.g. the cron hasn't run yet).
    pub fn read() -> Result<Option<Self>> {
        let omega = Self::omega_usage_path();
        let Some(content) = crate::config::read_private_optional(&omega)? else {
            return Ok(None);
        };
        let snap: Self = serde_json::from_slice(&content)
            .with_context(|| format!("parsing {}", omega.display()))?;
        Ok(Some(snap))
    }

    /// Age of the preferred cache file in seconds (None if missing).
    pub fn cache_age_secs() -> Option<u64> {
        let meta = std::fs::metadata(Self::omega_usage_path()).ok()?;
        let modified = meta.modified().ok()?;
        modified.elapsed().ok().map(|d| d.as_secs())
    }

    pub fn parsed_ts(&self) -> Option<DateTime<Utc>> {
        chrono::DateTime::parse_from_rfc3339(&self.ts)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    }

    pub fn precise_5h(&self) -> f32 {
        if self.pct_5h_precise > 0.0 {
            self.pct_5h_precise
        } else {
            self.session_pct as f32
        }
    }

    pub fn precise_week(&self) -> f32 {
        if self.pct_week_precise > 0.0 {
            self.pct_week_precise
        } else {
            self.week_pct as f32
        }
    }
}

/// A saved Claude account profile (one per file in ~/.claude/accounts/).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountProfile {
    pub label: String,
    pub email: Option<String>,
    pub is_active: bool,
}

/// The currently connected Claude account: email + subscription plan.
/// Read live from `claude auth status --json`. Returns None if not logged
/// in or the CLI isn't available.
#[derive(Debug, Clone)]
pub struct ConnectedAccount {
    pub email: String,
    /// e.g. "max", "pro", "free", "team".
    pub plan: String,
    pub auth_method: String,
}

pub fn connected_account() -> Option<ConnectedAccount> {
    let out = std::process::Command::new("claude")
        .args(["auth", "status"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    if !json
        .get("loggedIn")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return None;
    }
    Some(ConnectedAccount {
        email: json
            .get("email")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string(),
        plan: json
            .get("subscriptionType")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string(),
        auth_method: json
            .get("authMethod")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
            .to_string(),
    })
}

/// Discover all Claude account profiles + identify the active one.
pub fn list_accounts() -> Vec<AccountProfile> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Vec::new(),
    };
    let creds_path = home.join(".claude/.credentials.json");
    let accounts_dir = home.join(".claude/accounts");

    // Read current refreshToken to identify the active account
    let current_token: Option<String> = std::fs::read_to_string(&creds_path)
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| {
            v.get("claudeAiOauth")
                .and_then(|o| o.get("refreshToken"))
                .and_then(|t| t.as_str().map(String::from))
        });

    let mut profiles = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&accounts_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            let Some(name) = p.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let label = name.strip_prefix("account-").unwrap_or(name).to_string();

            let (email, account_token) = std::fs::read_to_string(&p)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .map(|v| {
                    let oauth = v.get("claudeAiOauth");
                    let email = oauth
                        .and_then(|o| o.get("email"))
                        .and_then(|e| e.as_str())
                        .map(String::from);
                    let token = oauth
                        .and_then(|o| o.get("refreshToken"))
                        .and_then(|t| t.as_str())
                        .map(String::from);
                    (email, token)
                })
                .unwrap_or((None, None));

            let is_active = match (&current_token, &account_token) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            };

            profiles.push(AccountProfile {
                label,
                email,
                is_active,
            });
        }
    }
    profiles.sort_by(|a, b| a.label.cmp(&b.label));
    profiles
}

/// Detect whether the AISB Telegram bot is running on this machine.
/// Looks at /tmp/aisb-usage.json freshness + any process matching aisb*python.
pub fn aisb_bot_status() -> AisbBotStatus {
    let cache_age = UsageSnapshot::cache_age_secs();
    let bot_alive = is_aisb_bot_running();

    let cache_status = match cache_age {
        None => CacheStatus::Missing,
        Some(s) if s < 900 => CacheStatus::Fresh(s),
        Some(s) => CacheStatus::Stale(s),
    };

    AisbBotStatus {
        bot_alive,
        cache_status,
    }
}

fn is_aisb_bot_running() -> bool {
    // Best-effort: pgrep -f "aisb.*python" or similar.
    let Ok(output) = std::process::Command::new("pgrep")
        .args(["-f", "aisb.app|telegram-bot|agentik-monitor"])
        .output()
    else {
        return false;
    };
    output.status.success() && !output.stdout.is_empty()
}

#[derive(Debug, Clone)]
pub struct AisbBotStatus {
    pub bot_alive: bool,
    pub cache_status: CacheStatus,
}

#[derive(Debug, Clone)]
pub enum CacheStatus {
    /// Cache file exists and is < 15 minutes old
    Fresh(u64),
    /// Cache file exists but is older than 15 minutes
    Stale(u64),
    /// Cache file does not exist
    Missing,
}

/// Trigger a Claude OAuth login by opening `claude /login` in a new pane.
/// Returns the command the user should run if we cannot spawn it ourselves.
pub fn login_command_hint() -> String {
    "claude /login".to_string()
}

/// Omega's own Telegram bot configuration (separate from AISB's).
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OmegaTelegramConfig {
    pub bot_token: String,
    /// The ONLY chat_id allowed to talk to the bot. Messages from any other
    /// chat are silently dropped.
    pub chat_id: i64,
    /// Required allow-list of Telegram user IDs (sender). The chat_id check
    /// ALWAYS applies and the sender MUST additionally appear here. Empty
    /// lists are rejected because this bot controls the host.
    #[serde(default)]
    pub allow_user_ids: Vec<i64>,
    /// Session to relay messages to (default: aisb-master)
    #[serde(default = "default_relay_session")]
    pub relay_session: String,
    /// Optional human label so the user can identify which profile is active.
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub enabled: bool,
}

impl std::fmt::Debug for OmegaTelegramConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OmegaTelegramConfig")
            .field("has_bot_token", &!self.bot_token.is_empty())
            .field("chat_id", &self.chat_id)
            .field("allow_user_ids", &self.allow_user_ids)
            .field("relay_session", &self.relay_session)
            .field("label", &self.label)
            .field("enabled", &self.enabled)
            .finish()
    }
}

fn default_relay_session() -> String {
    crate::aisb::MASTER_SESSION_NAME.to_string()
}

impl OmegaTelegramConfig {
    fn path() -> PathBuf {
        crate::config::omega_dir().join("telegram.toml")
    }

    pub fn try_read() -> Result<Option<Self>> {
        let path = Self::path();
        let Some(bytes) = crate::config::read_private_optional(&path)? else {
            return Ok(None);
        };
        let content = std::str::from_utf8(&bytes)
            .with_context(|| format!("Telegram config {} is not UTF-8", path.display()))?;
        let config: Self = toml::from_str(content).map_err(|error: toml::de::Error| {
            crate::config::private_toml_error("Telegram config", &path, error.span())
        })?;
        config.validate()?;
        Ok(Some(config))
    }

    /// Compatibility reader for UI/gateway callers that only support an
    /// optional state. Security-sensitive CLI paths use `try_read` so malformed
    /// authority is an error, not "not configured".
    pub fn read() -> Option<Self> {
        Self::try_read().ok().flatten()
    }

    fn validate(&self) -> Result<()> {
        if self.bot_token.trim().is_empty() {
            anyhow::bail!("Telegram bot token is empty");
        }
        if self.chat_id == 0 {
            anyhow::bail!("Telegram chat id cannot be zero");
        }
        if self.allow_user_ids.is_empty() {
            anyhow::bail!("Telegram sender allow-list is empty");
        }
        if self.relay_session.trim().is_empty() {
            anyhow::bail!("Telegram relay session is empty");
        }
        Ok(())
    }

    pub fn write(&self) -> Result<()> {
        self.validate()?;
        let path = Self::path();
        let lock_path = path.with_extension("toml.lock");
        let _guard = crate::config::acquire_private_lock_path(&lock_path)?;
        let content = toml::to_string_pretty(self)?;
        crate::config::atomic_write_private(&path, content.as_bytes())
    }

    pub fn update_enabled(enabled: bool) -> Result<Self> {
        let path = Self::path();
        let lock_path = path.with_extension("toml.lock");
        let _guard = crate::config::acquire_private_lock_path(&lock_path)?;
        let Some(bytes) = crate::config::read_private_optional(&path)? else {
            anyhow::bail!("Telegram is not configured");
        };
        let content = std::str::from_utf8(&bytes)
            .with_context(|| format!("Telegram config {} is not UTF-8", path.display()))?;
        let mut config: Self = toml::from_str(content).map_err(|error: toml::de::Error| {
            crate::config::private_toml_error("Telegram config", &path, error.span())
        })?;
        config.validate()?;
        config.enabled = enabled;
        crate::config::atomic_write_private(&path, toml::to_string_pretty(&config)?.as_bytes())?;
        Ok(config)
    }

    pub fn exists() -> bool {
        Self::path().exists()
    }

    /// Remove the active config file. Returns Ok(true) if a file was removed.
    pub fn disconnect() -> Result<bool> {
        let path = Self::path();
        let lock_path = path.with_extension("toml.lock");
        let _guard = crate::config::acquire_private_lock_path(&lock_path)?;
        let existed = crate::config::read_private_optional(&path)?.is_some();
        crate::scope::remove_private_file(&path)
            .with_context(|| format!("removing {}", path.display()))?;
        Ok(existed)
    }

    /// True if a Telegram message is allowed to interact with this bot.
    ///
    /// Security model (BOTH gates must pass — no bypass):
    ///   1. chat_id MUST equal the configured chat_id, AND
    ///   2. allow_user_ids is non-empty and the sender_id is present in it.
    ///
    /// This closes N17: previously a non-empty allow-list bypassed the chat_id
    /// check, so a listed sender in the WRONG chat was authorized and the
    /// configured chat with an un-listed sender was rejected. Now the chat gate
    /// is always enforced and the sender allow-list narrows within that chat.
    pub fn is_authorized(&self, chat_id: i64, sender_id: Option<i64>) -> bool {
        if chat_id != self.chat_id {
            return false;
        }
        !self.allow_user_ids.is_empty()
            && matches!(sender_id, Some(uid) if self.allow_user_ids.contains(&uid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_snapshot_handles_missing_file() {
        // /tmp/aisb-usage.json may or may not exist on the test host
        let result = UsageSnapshot::read();
        assert!(result.is_ok());
    }

    #[test]
    fn list_accounts_does_not_panic() {
        let _ = list_accounts();
    }

    fn auth_cfg(chat_id: i64, allow_user_ids: Vec<i64>) -> OmegaTelegramConfig {
        OmegaTelegramConfig {
            bot_token: String::new(),
            chat_id,
            allow_user_ids,
            relay_session: default_relay_session(),
            label: String::new(),
            enabled: true,
        }
    }

    #[test]
    fn telegram_config_is_strict_and_never_debugs_its_token() {
        let mut cfg = auth_cfg(-1001, vec![42]);
        cfg.bot_token = "secret-token-sentinel".to_string();
        cfg.relay_session = "AISB-master".to_string();
        cfg.validate().unwrap();
        let debug = format!("{cfg:?}");
        assert!(!debug.contains("secret-token-sentinel"));
        assert!(debug.contains("has_bot_token: true"));

        cfg.allow_user_ids.clear();
        assert!(cfg.validate().is_err());
        let unknown = r#"
            bot_token = "token"
            chat_id = -1001
            allow_user_ids = [42]
            relay_session = "AISB-master"
            enabled = true
            surprise = "not allowed"
        "#;
        assert!(toml::from_str::<OmegaTelegramConfig>(unknown).is_err());
    }

    #[test]
    fn is_authorized_requires_both_chat_and_sender() {
        // Right chat + right sender = allow
        let cfg = auth_cfg(100, vec![42]);
        assert!(cfg.is_authorized(100, Some(42)));

        // Right chat + wrong sender = reject (no chat bypass)
        assert!(!cfg.is_authorized(100, Some(999)));
        // Right chat + missing sender = reject when an allow-list is set
        assert!(!cfg.is_authorized(100, None));

        // Wrong chat + right sender = reject (closes N17 bypass)
        assert!(!cfg.is_authorized(200, Some(42)));
        // Wrong chat + wrong sender = reject
        assert!(!cfg.is_authorized(200, Some(999)));
    }

    #[test]
    fn is_authorized_rejects_an_empty_allowlist() {
        let cfg = auth_cfg(100, vec![]);
        assert!(!cfg.is_authorized(100, Some(7)));
        assert!(!cfg.is_authorized(100, None));
        assert!(!cfg.is_authorized(200, Some(7)));
        assert!(!cfg.is_authorized(200, None));
    }
}
