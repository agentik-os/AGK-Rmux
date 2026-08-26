//! Account management for OmegaOS — high-level API on top of the OAuth flow
//! and the credentials/profile layout under `~/.claude/`.
//!
//! ## Boundary vs `credentials::CredentialStore` (KNOWN DESIGN-DEBT — read this)
//! There are intentionally TWO account surfaces, with different jobs:
//!   - THIS module (`account.rs`): Claude-ONLY. It owns the live-login *billing /
//!     usage* view (`get_billing`, `email_from_claude_auth_status`) and the
//!     `/account` switch flow, keyed by `~/.claude/accounts/accounts-meta.json`.
//!   - `credentials::CredentialStore`: the MULTI-PROVIDER store (claude / codex /
//!     gemini / glm) under `~/.omega/credentials/`, used by the `/model` flow.
//!
//! They SHARE the active Claude credential file: `switch_account` here writes
//! through `oauth::credentials_path()`, which resolves to the same
//! `${OMEGA_DIR}/credentials/claude.json` the CredentialStore reads. So the
//! *active* account stays consistent. What does NOT reconcile is the two SAVED
//! lists: a Claude profile saved via `/model` (CredentialStore `accounts/claude-*`)
//! is not visible to `/account` (this module's `accounts-meta.json`) and vice
//! versa. Unifying the saved-profile storage is a real refactor (this module also
//! carries billing/OAuth-status logic CredentialStore has no equivalent for); it
//! is deliberately deferred rather than risk the working billing+switch flow.
//! If you touch account-switching, keep BOTH lists + the active file in sync.
//!
//! Layout we care about:
//!   ~/.claude/.credentials.json           — currently active credentials
//!   ~/.claude/.credentials.json.previous  — backup made on switch/logout
//!   ~/.claude/accounts/                   — saved per-account credential profiles
//!   ~/.claude/accounts/accounts-meta.json — { active, accounts: { name: {label, email, icon, credential_file} } }
//!
//! Public API:
//!   - `list_accounts` → all known profiles + active flag
//!   - `current_account` → email/label/expiry of the live credentials.json
//!   - `switch_account(name)` → swap credentials atomically; if refresh fails,
//!     restore the previous credentials and signal a reauth-needed result.
//!   - `logout()` → backup + remove `.credentials.json`.
//!   - `get_billing()` → re-export of `monitor::UsageSnapshot::read`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::monitor::UsageSnapshot;
use crate::oauth::{credentials_path, read_credentials, CredentialsInfo};

/// Per-account profile metadata as stored in accounts-meta.json.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountMetaEntry {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub credential_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountsMeta {
    #[serde(default)]
    pub accounts: BTreeMap<String, AccountMetaEntry>,
    #[serde(default)]
    pub active: Option<String>,
}

impl AccountsMeta {
    pub fn load() -> Self {
        let path = meta_path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Self>(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<()> {
        let path = meta_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json).with_context(|| format!("writing {}", path.display()))?;
        chmod_600(&path);
        Ok(())
    }
}

/// Secrets-at-rest: best-effort chmod 0600 on a credential/account file. These
/// files hold OAuth refresh tokens; on a multi-user host they must not be
/// group/world readable (the default umask 0002 leaves them 0664 otherwise).
#[cfg(unix)]
fn chmod_600(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}
#[cfg(not(unix))]
fn chmod_600(_path: &std::path::Path) {}

/// Copy a credential file to `dst` without ever exposing it group/world-readable.
/// `std::fs::copy` creates the destination under the process umask (typically
/// 0664) and only later chmods it, leaving a window where another local user can
/// read the OAuth refresh token. On Unix we create the destination up-front with
/// mode 0600 (O_CREAT|O_TRUNC) so the bytes are written into an already-locked
/// file; on other platforms we fall back to the plain copy.
#[cfg(unix)]
fn secure_copy(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut reader = std::fs::File::open(src)?;
    let mut writer = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(dst)?;
    std::io::copy(&mut reader, &mut writer)?;
    Ok(())
}
#[cfg(not(unix))]
fn secure_copy(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::copy(src, dst).map(|_| ())
}

/// High-level view of an account for the Telegram UI.
#[derive(Debug, Clone, Serialize)]
pub struct AccountSummary {
    pub name: String,
    pub label: String,
    pub email: String,
    pub icon: String,
    pub is_active: bool,
}

/// Live snapshot of the currently active account.
#[derive(Debug, Clone, Serialize)]
pub struct CurrentAccount {
    pub name: Option<String>,
    pub label: Option<String>,
    pub email: String,
    pub expires_min: i64,
    pub valid: bool,
    pub warning: bool,
    pub tier: String,
    pub subscription: String,
}

impl CurrentAccount {
    pub fn read() -> Self {
        let creds = read_credentials(&credentials_path()).unwrap_or_default();
        let active_name = detect_active_account(&creds);
        let meta = AccountsMeta::load();
        let label = active_name
            .as_ref()
            .and_then(|n| meta.accounts.get(n))
            .map(|a| a.label.clone());

        let expires_min = creds.expires_min();
        let valid = expires_min > 0;
        let warning = valid && expires_min < 30;

        // The email is NOT in credentials.json — it must come from
        // `claude auth status` (JSON with an "email" field).
        let email = creds
            .email
            .clone()
            .filter(|e| !e.is_empty() && e != "?")
            .unwrap_or_else(email_from_claude_auth_status);

        Self {
            name: active_name,
            label,
            email,
            expires_min,
            valid,
            warning,
            tier: creds
                .rate_limit_tier
                .clone()
                .unwrap_or_else(|| "?".to_string()),
            subscription: creds
                .subscription_type
                .clone()
                .unwrap_or_else(|| "?".to_string()),
        }
    }
}

/// Get the logged-in email by running `claude auth status` (JSON).
/// The email is NOT stored in credentials.json — only Claude knows it.
pub fn email_from_claude_auth_status() -> String {
    let output = std::process::Command::new("claude")
        .args(["auth", "status"])
        .output();
    if let Ok(out) = output {
        if let Ok(text) = String::from_utf8(out.stdout) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(email) = json.get("email").and_then(|e| e.as_str()) {
                    if !email.is_empty() {
                        return email.to_string();
                    }
                }
            }
        }
    }
    "?".to_string()
}

/// List all saved account profiles with active flag.
pub fn list_accounts() -> Vec<AccountSummary> {
    let meta = AccountsMeta::load();
    let creds = read_credentials(&credentials_path()).unwrap_or_default();
    let active_name = detect_active_account(&creds);

    let mut out: Vec<AccountSummary> = meta
        .accounts
        .iter()
        .map(|(name, entry)| AccountSummary {
            name: name.clone(),
            label: if entry.label.is_empty() {
                name.clone()
            } else {
                entry.label.clone()
            },
            email: entry.email.clone(),
            icon: entry.icon.clone(),
            is_active: active_name.as_deref() == Some(name.as_str()),
        })
        .collect();

    // Fallback: scan accounts/ directory if accounts-meta.json is empty.
    if out.is_empty() {
        if let Ok(entries) = std::fs::read_dir(accounts_dir()) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                let Some(stem) = p.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                if stem == "accounts-meta" {
                    continue;
                }
                let info = read_credentials(&p).unwrap_or_default();
                let label = stem.strip_prefix("account-").unwrap_or(stem).to_string();
                let is_active = match (&creds.refresh_token, &info.refresh_token) {
                    (Some(a), Some(b)) => a == b,
                    _ => false,
                };
                out.push(AccountSummary {
                    name: label.clone(),
                    label,
                    email: info.email.unwrap_or_default(),
                    icon: String::new(),
                    is_active,
                });
            }
        }
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Compare current credentials' refreshToken against each saved profile to
/// determine which one (if any) is the live account.
fn detect_active_account(current: &CredentialsInfo) -> Option<String> {
    let Some(token) = current.refresh_token.as_deref() else {
        // No live refresh token means the credentials file is missing, empty, or
        // corrupt. Surface it instead of silently reporting "no active account"
        // — callers (list_accounts / CurrentAccount::read) otherwise show every
        // profile as inactive with no clue why a just-switched account vanished.
        tracing::warn!(
            "detect_active_account: active credentials have no refresh_token — \
             cannot match any saved profile (credentials file missing/empty/corrupt)"
        );
        return None;
    };
    let meta = AccountsMeta::load();
    for (name, entry) in &meta.accounts {
        if entry.credential_file.is_empty() {
            continue;
        }
        let profile_path = accounts_dir().join(&entry.credential_file);
        if let Ok(profile) = read_credentials(&profile_path) {
            if profile.refresh_token.as_deref() == Some(token) {
                return Some(name.clone());
            }
        }
    }
    None
}

/// Outcome of `switch_account`.
#[derive(Debug, Clone, Serialize)]
pub struct SwitchResult {
    pub ok: bool,
    pub method: String, // "refresh" | "needs_reauth" | "no_profile"
    pub label: String,
    pub email: String,
    pub expires_min: i64,
    pub error: Option<String>,
}

/// Swap the active credentials to the named profile.
///
/// Steps:
///   1. Backup current credentials → .credentials.json.previous
///   2. Copy profile credential file → .credentials.json
///   3. Try `claude-oauth try-refresh` if available; otherwise just trust the
///      profile credentials.
///   4. On refresh failure → restore backup and return needs_reauth.
pub fn switch_account(name: &str) -> SwitchResult {
    let meta = AccountsMeta::load();
    let Some(entry) = meta.accounts.get(name).cloned() else {
        return SwitchResult {
            ok: false,
            method: "no_profile".to_string(),
            label: name.to_string(),
            email: String::new(),
            expires_min: 0,
            error: Some(format!("account '{}' not in accounts-meta", name)),
        };
    };

    let profile_path = accounts_dir().join(&entry.credential_file);
    if !profile_path.exists() {
        return SwitchResult {
            ok: false,
            method: "no_profile".to_string(),
            label: entry.label.clone(),
            email: entry.email.clone(),
            expires_min: 0,
            error: Some(format!(
                "credential file missing: {}",
                entry.credential_file
            )),
        };
    }

    let creds_path = credentials_path();
    if creds_path.exists() {
        let backup = creds_path.with_extension("json.previous");
        let _ = secure_copy(&creds_path, &backup);
        chmod_600(&backup);
    }
    if let Err(e) = secure_copy(&profile_path, &creds_path) {
        return SwitchResult {
            ok: false,
            method: "no_profile".to_string(),
            label: entry.label.clone(),
            email: entry.email.clone(),
            expires_min: 0,
            error: Some(format!("copy failed: {}", e)),
        };
    }
    chmod_600(&creds_path);

    // Best-effort refresh via the bundled helper script (if present). Lives
    // under the consolidated system root via config::omega_dir() (was a
    // hardcoded ~/.omega that missed the script after relocation); ~/.claude
    // below stays as-is — that is Claude's own dir, not OmegaOS state.
    let helper = crate::config::omega_dir()
        .join("bin")
        .join("claude-oauth.sh");
    let mut refresh_ok = true;
    if helper.exists() {
        let output = std::process::Command::new(&helper)
            .arg("try-refresh")
            .output();
        refresh_ok = match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(stdout.trim()) {
                    v.get("ok").and_then(|b| b.as_bool()).unwrap_or(false)
                } else {
                    false
                }
            }
            Err(_) => false,
        };
    }

    if !refresh_ok {
        // Restore previous credentials so we don't break the live session. If the
        // restore can't happen (no backup, or copy failed), creds_path now holds
        // the new profile's creds while we report needs_reauth — that mismatch is
        // a real system fault, not a normal expired-token case, so surface it.
        let backup = creds_path.with_extension("json.previous");
        let restore_err: Option<String> = if !backup.exists() {
            Some("no backup of previous credentials to restore".to_string())
        } else if let Err(e) = secure_copy(&backup, &creds_path) {
            Some(format!("restore copy failed: {}", e))
        } else {
            chmod_600(&creds_path);
            None
        };
        let error = match restore_err {
            Some(why) => {
                tracing::error!(
                    "switch_account: refresh failed AND restore failed ({}); \
                     active credentials may now hold the wrong account",
                    why
                );
                Some(format!(
                    "refresh token expired and previous credentials could not be \
                     restored ({}) — credentials may be in an inconsistent state, \
                     run /login to recover",
                    why
                ))
            }
            None => Some("refresh token expired — need full OAuth reauth".to_string()),
        };
        return SwitchResult {
            ok: false,
            method: "needs_reauth".to_string(),
            label: entry.label.clone(),
            email: entry.email.clone(),
            expires_min: 0,
            error,
        };
    }

    // Refresh succeeded — copy refreshed creds back to the profile and update
    // accounts-meta.active.
    let _ = secure_copy(&creds_path, &profile_path);
    chmod_600(&profile_path);
    let mut meta_mut = meta;
    meta_mut.active = Some(name.to_string());
    let _ = meta_mut.save();

    let creds = read_credentials(&creds_path).unwrap_or_default();
    SwitchResult {
        ok: true,
        method: "refresh".to_string(),
        label: entry.label.clone(),
        email: entry.email.clone(),
        expires_min: creds.expires_min(),
        error: None,
    }
}

/// Backup + remove the active credentials file. Active sessions keep their
/// in-memory token but new sessions will need to /login.
pub fn logout() -> Result<()> {
    let creds_path = credentials_path();
    if !creds_path.exists() {
        return Ok(());
    }
    let backup = creds_path.with_extension("json.previous");
    std::fs::copy(&creds_path, &backup)
        .with_context(|| format!("backing up {}", creds_path.display()))?;
    chmod_600(&backup);
    std::fs::remove_file(&creds_path)
        .with_context(|| format!("removing {}", creds_path.display()))?;
    Ok(())
}

/// Re-export of the AISB billing snapshot so callers don't have to know about
/// the monitor module.
pub fn get_billing() -> Option<UsageSnapshot> {
    UsageSnapshot::read().ok().flatten()
}

// ───────────────────────── path helpers ─────────────────────────

fn accounts_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".claude")
        .join("accounts")
}

fn meta_path() -> PathBuf {
    accounts_dir().join("accounts-meta.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_accounts_does_not_panic() {
        let _ = list_accounts();
    }

    #[test]
    fn current_account_does_not_panic() {
        let _ = CurrentAccount::read();
    }

    #[test]
    fn meta_load_handles_missing_file() {
        // Should produce a default empty meta.
        let m = AccountsMeta::load();
        // No assertions on contents — file may exist on the test host.
        let _ = m;
    }
}
