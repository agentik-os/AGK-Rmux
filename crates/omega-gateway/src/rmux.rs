use anyhow::{bail, Result};
use std::path::PathBuf;
use std::process::Command;

pub fn rmux_bin() -> PathBuf {
    if let Ok(bin) = std::env::var("OMEGA_RMUX_BIN") {
        return PathBuf::from(bin);
    }
    dirs::home_dir()
        .expect("no home dir")
        .join(".local/bin/rmux")
}

fn run(args: &[&str]) -> Result<String> {
    let out = Command::new(rmux_bin()).args(args).output()?;
    if !out.status.success() {
        bail!("{}", String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn list_sessions() -> Result<Vec<String>> {
    let out = run(&["ls", "-F", "#S"])?;
    Ok(out
        .lines()
        .map(str::to_string)
        .filter(|l| !l.is_empty())
        .collect())
}

pub fn capture_pane(session: &str, lines: u32) -> Result<String> {
    let start = format!("-{lines}");
    run(&["capture-pane", "-p", "-t", session, "-S", &start])
}

/// Same as [`capture_pane`] but with `-e`, which emits ANSI escape sequences
/// (color/attributes) instead of plain text. Used by the stream route's
/// opt-in `?color=1` mode; `capture_pane` itself stays plain-text-only so the
/// default (no query param) behavior is unchanged.
pub fn capture_pane_ansi(session: &str, lines: u32) -> Result<String> {
    let start = format!("-{lines}");
    run(&["capture-pane", "-p", "-t", session, "-S", &start, "-e"])
}

/// Sends literal keystrokes to a live rmux session WITHOUT submitting them
/// (no Enter). Mirrors the real CLI shape verified against working scripts
/// elsewhere on this box: `rmux send-keys -t <session> -l -- <text>`.
///
/// The `--` separator is load-bearing, not decorative: rmux's own clap-based
/// argument parser reads a `text` that starts with `-` (e.g. a user literally
/// typing `-N`) as an unrecognized flag rather than the literal keystroke
/// value, silently failing the send. Reproduced against the real `rmux`
/// binary: `[send-keys, -t, s, -l, -N]` -> `"a value is required for '-N'"`,
/// while `[send-keys, -t, s, -l, --, -N]` parses correctly. This is the same
/// class of bug `routes_dispatch.rs` already guards against for `omega`'s own
/// argv — never drop this separator.
pub fn send_keys_literal(session: &str, text: &str) -> Result<()> {
    run(&["send-keys", "-t", session, "-l", "--", text])?;
    Ok(())
}

/// Submits (presses Enter in) a live rmux session. ALWAYS a separate,
/// second subprocess invocation from [`send_keys_literal`] — never
/// concatenate text+Enter into one call, per the real `rmux` CLI shape.
pub fn send_enter(session: &str) -> Result<()> {
    run(&["send-keys", "-t", session, "Enter"])?;
    Ok(())
}

/// Renames a live rmux session: `rmux rename-session -t <old> <new>`.
///
/// Argv shape CONFIRMED against a real call site in this monorepo (not
/// inferred/guessed): `omega_core::session::SessionManager::rename_session`
/// (`crates/omega-core/src/session.rs:454-455`) already drives the same
/// real `rmux` binary with exactly `.args(["rename-session", "-t",
/// old_name, &safe])` — `-t <session>` selects the target, the new name is
/// a bare positional, matching this module's other `-t`-first shapes
/// (`capture_pane`, `send_keys_literal`, `send_enter`).
///
/// The caller (`routes_sessions::rename`) is responsible for validating
/// `new` as a safe slug BEFORE calling this: unlike
/// `SessionManager::rename_session`'s own `sanitize_session_name` chokepoint,
/// this thin wrapper passes `new` straight through, so an unsafe value here
/// would hit rmux's own silent `:`/`.` → `_` rewrite instead of being
/// rejected, and the HTTP caller expects `new_name` back verbatim.
pub fn rename_session(old: &str, new: &str) -> Result<()> {
    run(&["rename-session", "-t", old, new])?;
    Ok(())
}
