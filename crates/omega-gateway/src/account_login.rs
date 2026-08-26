//! Account login driver — the real `claude`/`codex` CLI processes that turn an
//! isolated credential slot ([`crate::accounts::AccountStore::slot_dir`]) from
//! empty into a logged-in account, or check/end that login.
//!
//! Three pure/composable pieces, same shape as `chat_driver.rs`:
//! - [`parse_auth_status`] / [`parse_login_url`] parse CLI text output (no I/O, unit-testable).
//! - [`status`] / [`logout`] run a short-lived provider CLI call and parse its output.
//! - [`begin_claude_login`] / [`begin_codex_login`] spawn the OAuth browser-relay flow;
//!   [`codex_login_with_api_key`] drives the headless API-key flow.
//!
//! VERIFIED CLI facts this module encodes: `CLAUDE_CONFIG_DIR=<slot> claude auth
//! login|logout|status` (an empty `CLAUDE_CONFIG_DIR` is a clean isolated
//! account); `CODEX_HOME=<slot> codex login|logout`, `codex login status`,
//! `codex login --with-api-key` (reads the key from stdin — headless).

use crate::protocol::{Account, AccountKind};
use anyhow::{bail, Result};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// How long [`begin_claude_login`] / [`begin_codex_login`] wait for the CLI
/// to emit an authorization URL on stdout before concluding it needs an
/// interactive terminal ([`LoginOutcome::NeedsBox`]).
const LOGIN_URL_WAIT: Duration = Duration::from_secs(5);

/// The `claude` binary: `$OMEGA_CLAUDE_BIN` when set, else `claude` (resolved via PATH).
pub fn claude_bin() -> String {
    std::env::var("OMEGA_CLAUDE_BIN").unwrap_or_else(|_| "claude".to_string())
}

/// The `codex` binary: `$OMEGA_CODEX_BIN` when set, else `codex` (resolved via PATH).
pub fn codex_bin() -> String {
    std::env::var("OMEGA_CODEX_BIN").unwrap_or_else(|_| "codex".to_string())
}

/// Whether a credential slot is logged in, as read from a provider CLI's own
/// status text. `Unknown` covers both an unparseable output and a failure to
/// even spawn the CLI (e.g. it is not installed) — either way this module
/// cannot assert LoggedIn or LoggedOut with confidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthStatus {
    LoggedIn,
    LoggedOut,
    Unknown,
}

/// The outcome of starting a browser-OAuth login flow.
pub enum LoginOutcome {
    /// The CLI printed an authorization URL; the caller relays it to the app
    /// and the flow completes when the user authorizes in a browser and the
    /// child process exits. The child is handed back so the caller can reap
    /// it later; completion is polled via [`poll_login_complete`], never by
    /// waiting on the child (browser auth can take arbitrarily long).
    Url(String, Child),
    /// No URL was emitted within [`LOGIN_URL_WAIT`] (the CLI needs an
    /// interactive terminal instead) — the route should fall back to the
    /// box-side login flow.
    NeedsBox,
}

/// Parses one line/chunk of `claude auth status` / `codex login status`
/// output into an [`AuthStatus`]. Pure, no I/O.
///
/// VERIFIED LIVE (2026-08-10): the real `claude auth status` emits JSON —
/// `{"loggedIn": false, ...}` / `{"loggedIn": true, ...}` — never the English
/// prose this parser originally assumed, so the JSON `"loggedIn"` key is
/// matched FIRST and wins outright; camelCase has no space between "logged"
/// and "In" so the prose patterns below never accidentally match it either
/// way. The prose patterns are kept as a fallback for `codex login status`
/// (unverified live) and any provider CLI version that still prints text.
/// "Not logged in" contains the substring "logged in", so the LoggedOut
/// patterns are matched FIRST — order here is load-bearing.
pub fn parse_auth_status(output: &str) -> AuthStatus {
    let lower = output.to_lowercase();
    if lower.contains("\"loggedin\": false") || lower.contains("\"loggedin\":false") {
        return AuthStatus::LoggedOut;
    }
    if lower.contains("\"loggedin\": true") || lower.contains("\"loggedin\":true") {
        return AuthStatus::LoggedIn;
    }
    if lower.contains("not logged in")
        || lower.contains("not authenticated")
        || lower.contains("please run")
        || lower.contains("please log in")
    {
        AuthStatus::LoggedOut
    } else if lower.contains("logged in") {
        AuthStatus::LoggedIn
    } else {
        AuthStatus::Unknown
    }
}

/// Extracts the first `https://` URL from CLI output. Pure, no I/O.
pub fn parse_login_url(s: &str) -> Option<String> {
    for token in s.split_whitespace() {
        if let Some(idx) = token.find("https://") {
            let url = token[idx..].trim_end_matches(['.', ',', ')', ']', '>', '"', '\'', ';']);
            if !url.is_empty() {
                return Some(url.to_string());
            }
        }
    }
    None
}

/// Checks whether `slot` is logged in for `account`, via the provider CLI's
/// own status subcommand. Free — never a paid call.
pub fn status(account: &Account, slot: &Path) -> AuthStatus {
    let output = match account.kind {
        AccountKind::Claude => Command::new(claude_bin())
            .arg("auth")
            .arg("status")
            .env("CLAUDE_CONFIG_DIR", slot)
            .output(),
        AccountKind::Codex => Command::new(codex_bin())
            .arg("login")
            .arg("status")
            .env("CODEX_HOME", slot)
            .output(),
    };
    let Ok(output) = output else {
        return AuthStatus::Unknown;
    };
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    parse_auth_status(&combined)
}

/// Polls whether a login begun with [`begin_claude_login`] /
/// [`begin_codex_login`] has completed, by re-checking `status()`. The
/// caller re-invokes this (e.g. on a short interval) rather than waiting on
/// the [`LoginOutcome::Url`] child, since a browser OAuth flow can take
/// arbitrarily long.
pub fn poll_login_complete(account: &Account, slot: &Path) -> bool {
    status(account, slot) == AuthStatus::LoggedIn
}

/// Spawns `cmd`, reads its stdout looking for an authorization URL within
/// [`LOGIN_URL_WAIT`], and returns the outcome. Shared by
/// [`begin_claude_login`] and [`begin_codex_login`] — the only difference
/// between the two providers is which command gets built.
fn begin_login(mut cmd: Command) -> Result<LoginOutcome> {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().expect("stdout was piped");

    // Read lines on a background thread and hand them to the caller via a
    // bounded-wait channel, so a CLI that never prints AND never exits
    // (genuinely waiting on a TTY) cannot hang this call forever.
    let (tx, rx) = mpsc::channel::<String>();
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(std::io::Result::ok) {
            if tx.send(line).is_err() {
                break; // receiver gave up (URL already found, or timed out)
            }
        }
        // EOF: tx drops here, closing the channel.
    });

    let deadline = Instant::now() + LOGIN_URL_WAIT;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        match rx.recv_timeout(remaining) {
            Ok(line) => {
                if let Some(url) = parse_login_url(&line) {
                    return Ok(LoginOutcome::Url(url, child));
                }
                // Not this line; keep waiting for one that carries the URL.
            }
            Err(mpsc::RecvTimeoutError::Timeout) => break,
            // EOF with no URL line ever sent: the CLI needs a TTY.
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    Ok(LoginOutcome::NeedsBox)
}

/// Starts the Claude browser-OAuth login flow for `slot`.
pub fn begin_claude_login(slot: &Path) -> Result<LoginOutcome> {
    let mut cmd = Command::new(claude_bin());
    cmd.arg("auth").arg("login").env("CLAUDE_CONFIG_DIR", slot);
    begin_login(cmd)
}

/// Starts the Codex browser-OAuth login flow for `slot`.
pub fn begin_codex_login(slot: &Path) -> Result<LoginOutcome> {
    let mut cmd = Command::new(codex_bin());
    cmd.arg("login").env("CODEX_HOME", slot);
    begin_login(cmd)
}

/// Headless Codex login: pipes `api_key` to `codex login --with-api-key` via
/// stdin. The key is never logged and never stored by the gateway itself —
/// codex persists it inside `slot` (opaque to this module, same contract as
/// every other credential slot).
pub fn codex_login_with_api_key(slot: &Path, api_key: &str) -> Result<()> {
    let mut child = Command::new(codex_bin())
        .arg("login")
        .arg("--with-api-key")
        .env("CODEX_HOME", slot)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;
    {
        // Scoped so `stdin` is dropped (closing the pipe / signaling EOF)
        // before we wait on the child — a child reading until EOF on stdin
        // would otherwise hang forever waiting for us to close it.
        let mut stdin = child.stdin.take().expect("stdin was piped");
        stdin.write_all(api_key.as_bytes())?;
        stdin.write_all(b"\n")?;
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        bail!(
            "codex login --with-api-key failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

/// Logs `slot` out for `account`.
pub fn logout(account: &Account, slot: &Path) -> Result<()> {
    let output = match account.kind {
        AccountKind::Claude => Command::new(claude_bin())
            .arg("auth")
            .arg("logout")
            .env("CLAUDE_CONFIG_DIR", slot)
            .output()?,
        AccountKind::Codex => Command::new(codex_bin())
            .arg("logout")
            .env("CODEX_HOME", slot)
            .output()?,
    };
    if !output.status.success() {
        bail!(
            "logout failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    // OMEGA_CLAUDE_BIN / OMEGA_CODEX_BIN / CAPTURE_FILE are process-global;
    // serialize every test that touches them (same pattern as
    // chat_driver.rs's OMEGA_CHAT_BIN lock and missions.rs's OMEGA_STATE_DIR
    // lock).
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Writes an executable shell script fixture under `dir` and returns its path.
    fn fake_bin(dir: &Path, name: &str, script: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, format!("#!/bin/sh\n{script}\n")).unwrap();
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms).unwrap();
        path
    }

    fn test_account(kind: AccountKind) -> Account {
        Account {
            slug: "acct-1".to_string(),
            label: "Test".to_string(),
            kind,
            created_at: "t".to_string(),
            is_default: true,
        }
    }

    // --- parse_auth_status (pure) ---

    #[test]
    fn parse_auth_status_logged_in() {
        assert_eq!(parse_auth_status("Logged in as x"), AuthStatus::LoggedIn);
    }

    #[test]
    fn parse_auth_status_not_logged_in() {
        assert_eq!(parse_auth_status("Not logged in"), AuthStatus::LoggedOut);
    }

    #[test]
    fn parse_auth_status_please_run() {
        assert_eq!(
            parse_auth_status("Please run `claude auth login` first"),
            AuthStatus::LoggedOut
        );
    }

    #[test]
    fn parse_auth_status_garbage_is_unknown() {
        assert_eq!(parse_auth_status("¯\\_(ツ)_/¯ 42"), AuthStatus::Unknown);
    }

    // Captured live 2026-08-10 from `CLAUDE_CONFIG_DIR=<fresh slot> claude
    // auth status` / a real logged-in `claude auth status` — the actual CLI
    // emits JSON, not the English prose the original parser assumed.

    #[test]
    fn parse_auth_status_real_claude_json_logged_out() {
        let out = "{\n  \"loggedIn\": false,\n  \"authMethod\": \"none\",\n  \"apiProvider\": \"firstParty\"\n}\n";
        assert_eq!(parse_auth_status(out), AuthStatus::LoggedOut);
    }

    #[test]
    fn parse_auth_status_real_claude_json_logged_in() {
        let out = "{\n  \"loggedIn\": true,\n  \"authMethod\": \"claude.ai\",\n  \"apiProvider\": \"firstParty\",\n  \"email\": \"x@example.com\"\n}\n";
        assert_eq!(parse_auth_status(out), AuthStatus::LoggedIn);
    }

    #[test]
    fn parse_auth_status_compact_json_form() {
        assert_eq!(
            parse_auth_status("{\"loggedIn\":false}"),
            AuthStatus::LoggedOut
        );
        assert_eq!(
            parse_auth_status("{\"loggedIn\":true}"),
            AuthStatus::LoggedIn
        );
    }

    // --- parse_login_url (pure) ---

    #[test]
    fn parse_login_url_extracts_url() {
        let out = parse_login_url("Visit https://claude.ai/oauth?x=1 to continue");
        assert_eq!(out, Some("https://claude.ai/oauth?x=1".to_string()));
    }

    #[test]
    fn parse_login_url_trims_trailing_punctuation() {
        let out = parse_login_url("See (https://claude.ai/oauth?x=1).");
        assert_eq!(out, Some("https://claude.ai/oauth?x=1".to_string()));
    }

    #[test]
    fn parse_login_url_none_when_absent() {
        assert_eq!(parse_login_url("no url here"), None);
    }

    // --- status() with a fake claude/codex bin ---

    #[test]
    fn status_parses_fake_claude_not_logged_in() {
        let _g = LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let bin = fake_bin(dir.path(), "fake-claude", "echo 'Not logged in'\nexit 1");
        std::env::set_var("OMEGA_CLAUDE_BIN", &bin);

        let account = test_account(AccountKind::Claude);
        let slot = tempfile::tempdir().unwrap();
        let result = status(&account, slot.path());

        std::env::remove_var("OMEGA_CLAUDE_BIN");
        assert_eq!(result, AuthStatus::LoggedOut);
    }

    #[test]
    fn status_parses_fake_claude_logged_in() {
        let _g = LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let bin = fake_bin(
            dir.path(),
            "fake-claude",
            "echo 'Logged in as test@example.com'",
        );
        std::env::set_var("OMEGA_CLAUDE_BIN", &bin);

        let account = test_account(AccountKind::Claude);
        let slot = tempfile::tempdir().unwrap();
        let result = status(&account, slot.path());

        std::env::remove_var("OMEGA_CLAUDE_BIN");
        assert_eq!(result, AuthStatus::LoggedIn);
    }

    #[test]
    fn status_parses_fake_codex_logged_out() {
        let _g = LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let bin = fake_bin(dir.path(), "fake-codex", "echo 'Not logged in'\nexit 1");
        std::env::set_var("OMEGA_CODEX_BIN", &bin);

        let account = test_account(AccountKind::Codex);
        let slot = tempfile::tempdir().unwrap();
        let result = status(&account, slot.path());

        std::env::remove_var("OMEGA_CODEX_BIN");
        assert_eq!(result, AuthStatus::LoggedOut);
    }

    #[test]
    fn status_missing_binary_is_unknown() {
        let _g = LOCK.lock().unwrap();
        std::env::set_var(
            "OMEGA_CLAUDE_BIN",
            "/nonexistent/definitely-not-a-binary-xyz",
        );

        let account = test_account(AccountKind::Claude);
        let slot = tempfile::tempdir().unwrap();
        let result = status(&account, slot.path());

        std::env::remove_var("OMEGA_CLAUDE_BIN");
        assert_eq!(result, AuthStatus::Unknown);
    }

    // --- begin_claude_login() with a fake bin ---

    #[test]
    fn begin_claude_login_extracts_url_from_fake_bin() {
        let _g = LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let bin = fake_bin(
            dir.path(),
            "fake-claude",
            "echo 'Visit https://claude.ai/oauth?x=1 to continue'",
        );
        std::env::set_var("OMEGA_CLAUDE_BIN", &bin);

        let slot = tempfile::tempdir().unwrap();
        let outcome = begin_claude_login(slot.path()).unwrap();

        std::env::remove_var("OMEGA_CLAUDE_BIN");
        match outcome {
            LoginOutcome::Url(url, mut child) => {
                assert_eq!(url, "https://claude.ai/oauth?x=1");
                let _ = child.wait();
            }
            LoginOutcome::NeedsBox => panic!("expected Url outcome"),
        }
    }

    #[test]
    fn begin_claude_login_needs_box_when_no_url_emitted() {
        let _g = LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let bin = fake_bin(dir.path(), "fake-claude", "exit 1");
        std::env::set_var("OMEGA_CLAUDE_BIN", &bin);

        let slot = tempfile::tempdir().unwrap();
        let outcome = begin_claude_login(slot.path()).unwrap();

        std::env::remove_var("OMEGA_CLAUDE_BIN");
        assert!(matches!(outcome, LoginOutcome::NeedsBox));
    }

    // --- codex_login_with_api_key() with a fake bin capturing stdin ---

    #[test]
    fn codex_login_with_api_key_pipes_key_via_stdin() {
        let _g = LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let capture = dir.path().join("captured-stdin.txt");
        let bin = fake_bin(dir.path(), "fake-codex", "cat > \"$CAPTURE_FILE\"");
        std::env::set_var("OMEGA_CODEX_BIN", &bin);
        std::env::set_var("CAPTURE_FILE", &capture);

        let slot = tempfile::tempdir().unwrap();
        let result = codex_login_with_api_key(slot.path(), "sk-test-123");

        std::env::remove_var("OMEGA_CODEX_BIN");
        std::env::remove_var("CAPTURE_FILE");

        assert!(result.is_ok());
        let captured = std::fs::read_to_string(&capture).unwrap();
        assert_eq!(captured.trim(), "sk-test-123");
    }

    #[test]
    fn codex_login_with_api_key_propagates_failure() {
        let _g = LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let bin = fake_bin(dir.path(), "fake-codex", "cat > /dev/null\nexit 1");
        std::env::set_var("OMEGA_CODEX_BIN", &bin);

        let slot = tempfile::tempdir().unwrap();
        let result = codex_login_with_api_key(slot.path(), "sk-test-123");

        std::env::remove_var("OMEGA_CODEX_BIN");
        assert!(result.is_err());
    }

    // --- logout() with a fake bin ---

    #[test]
    fn logout_claude_runs_and_succeeds() {
        let _g = LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let bin = fake_bin(dir.path(), "fake-claude", "exit 0");
        std::env::set_var("OMEGA_CLAUDE_BIN", &bin);

        let account = test_account(AccountKind::Claude);
        let slot = tempfile::tempdir().unwrap();
        let result = logout(&account, slot.path());

        std::env::remove_var("OMEGA_CLAUDE_BIN");
        assert!(result.is_ok());
    }

    #[test]
    fn logout_propagates_failure() {
        let _g = LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let bin = fake_bin(dir.path(), "fake-codex", "echo 'boom' >&2\nexit 1");
        std::env::set_var("OMEGA_CODEX_BIN", &bin);

        let account = test_account(AccountKind::Codex);
        let slot = tempfile::tempdir().unwrap();
        let result = logout(&account, slot.path());

        std::env::remove_var("OMEGA_CODEX_BIN");
        assert!(result.is_err());
    }

    // --- claude_bin() / codex_bin() defaults ---

    #[test]
    fn claude_bin_defaults_when_env_unset() {
        let _g = LOCK.lock().unwrap();
        std::env::remove_var("OMEGA_CLAUDE_BIN");
        assert_eq!(claude_bin(), "claude");
    }

    #[test]
    fn codex_bin_defaults_when_env_unset() {
        let _g = LOCK.lock().unwrap();
        std::env::remove_var("OMEGA_CODEX_BIN");
        assert_eq!(codex_bin(), "codex");
    }
}
