use crate::agents::Agent;
use crate::session::SessionManager;
use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;

const VIEWER_HEADER: &str = "  Ω  AISB conversation viewer (read-only)\n\
             ─────────────────────────────────────────────────\n\
             Talk to AISB from Telegram. Exchanges stream here.\n";

/// Backward-compatible session name of the read-only AISB conversation viewer.
pub const MASTER_SESSION_NAME: &str = "aisb-master";

/// Ensures the AISB conversation viewer exists. Idempotent: creates only if missing.
///
/// The session is a PURE READ-ONLY VIEWER: it `tail -F`s the conversation log
/// the brain stream writes, so the user can WATCH the live Telegram exchange
/// in the TUI. It is NOT the brain and NOT interactive (the brain is the
/// Telegram bot's own headless subprocess — see
/// `omega-gateway/src/chat_driver.rs` and `telegram-bot/omega-tg-bot.ts`).
///
/// Returns true if a new session was created, false if it already existed.
pub async fn ensure_viewer(mgr: &SessionManager, working_dir: &str) -> Result<bool> {
    let sessions = mgr.list_sessions().await?;
    if sessions.iter().any(|s| s.name == MASTER_SESSION_NAME) {
        return Ok(false);
    }

    // NEW MODEL (2026-05-28): the Telegram bot owns its OWN persistent
    // Claude headless subprocess (gateway chat driver / Bun bot) with full VPS
    // access — that
    // is the brain. The aisb-master rmux session is now a LIVE VIEWER that
    // tails the conversation log the bridge writes, so the user can WATCH
    // the Telegram chat stream in the TUI. It is no longer an interactive
    // Claude (that caused the "talks in the pane but not Telegram" split).
    let log = crate::config::omega_dir().join("state/aisb-conversation.log");
    // Surface filesystem errors instead of swallowing them: if the log can't be
    // created the viewer pane would `tail -F` a path that never appears, with no
    // diagnostic. A failed first call must error so the caller can retry; on a
    // successful create the log is guaranteed to exist for the idempotent path.
    ensure_viewer_log(&log)?;
    // The master pane is a PURE READ-ONLY MIRROR of the brain's
    // conversation: it tails the same `aisb-conversation.log` the brain
    // stream appends to via `mirror_to_master_pane`. It is NOT interactive
    // — there is one brain (the Telegram bot's SDK subprocess) and the user
    // talks to it from Telegram; this pane only lets them WATCH the live
    // exchange in the TUI. `exec` so the pane dies cleanly if the tail does.
    let log_path = log.to_string_lossy();
    let cmd = format!("exec tail -n 200 -F {}", shell_quote(&log_path));

    mgr.create_session(MASTER_SESSION_NAME, Some(working_dir), Some(&cmd))
        .await?;

    Ok(true)
}

/// Backward-compatible API name retained for downstream callers. New code
/// should use `ensure_viewer`, which describes the process truthfully.
pub async fn ensure_master(
    mgr: &SessionManager,
    _legacy_agent: Agent,
    working_dir: &str,
) -> Result<bool> {
    ensure_viewer(mgr, working_dir).await
}

/// Returns true if a session is the backward-compatible AISB viewer session.
pub fn is_viewer(name: &str) -> bool {
    name == MASTER_SESSION_NAME
}

/// Backward-compatible role predicate.
pub fn is_master(name: &str) -> bool {
    is_viewer(name)
}

/// Single-quote a path for safe interpolation into a shell command
/// (the viewer pane's `tail -F <path>`). Wraps in single quotes and
/// escapes any embedded single quote, so spaces/specials in the home
/// path can't break or inject into the command.
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn ensure_viewer_log(path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("viewer log {} has no parent directory", path.display()))?;
    match std::fs::symlink_metadata(parent) {
        Ok(metadata) if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {}
        Ok(_) => anyhow::bail!(
            "refusing non-directory or symlink AISB viewer state path {}",
            parent.display()
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating AISB viewer state dir {}", parent.display()))?;
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("inspecting AISB viewer state dir {}", parent.display()))
        }
    }
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => validate_viewer_log(path, &metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            match options.open(path) {
                Ok(mut file) => {
                    file.write_all(VIEWER_HEADER.as_bytes()).with_context(|| {
                        format!("initializing AISB viewer log {}", path.display())
                    })?;
                    file.sync_all()
                        .with_context(|| format!("syncing AISB viewer log {}", path.display()))?;
                    Ok(())
                }
                // A concurrent initializer won the create_new race. Validate
                // the winner rather than truncating or following it.
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    let metadata = std::fs::symlink_metadata(path).with_context(|| {
                        format!("inspecting concurrent AISB viewer log {}", path.display())
                    })?;
                    validate_viewer_log(path, &metadata)
                }
                Err(error) => Err(error)
                    .with_context(|| format!("creating AISB viewer log {}", path.display())),
            }
        }
        Err(error) => {
            Err(error).with_context(|| format!("inspecting AISB viewer log {}", path.display()))
        }
    }
}

fn validate_viewer_log(path: &Path, metadata: &std::fs::Metadata) -> Result<()> {
    if !metadata.file_type().is_file() {
        anyhow::bail!(
            "refusing non-regular AISB viewer log {} (symlinks are not trusted)",
            path.display()
        );
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 {
            anyhow::bail!("refusing hard-linked AISB viewer log {}", path.display());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewer_log_is_created_once_without_truncation() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("state/aisb-conversation.log");
        ensure_viewer_log(&path).unwrap();
        std::fs::write(&path, "existing conversation\n").unwrap();
        ensure_viewer_log(&path).unwrap();
        assert_eq!(
            std::fs::read_to_string(path).unwrap(),
            "existing conversation\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn viewer_log_rejects_symlink_and_hardlink_aliases() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("target");
        std::fs::write(&target, "secret").unwrap();
        let symlink = temp.path().join("symlink.log");
        std::os::unix::fs::symlink(&target, &symlink).unwrap();
        assert!(ensure_viewer_log(&symlink).is_err());

        let hardlink = temp.path().join("hardlink.log");
        std::fs::hard_link(&target, &hardlink).unwrap();
        assert!(ensure_viewer_log(&hardlink).is_err());
        assert_eq!(std::fs::read_to_string(target).unwrap(), "secret");

        let real_state = temp.path().join("real-state");
        std::fs::create_dir(&real_state).unwrap();
        let linked_state = temp.path().join("linked-state");
        std::os::unix::fs::symlink(&real_state, &linked_state).unwrap();
        assert!(ensure_viewer_log(&linked_state.join("viewer.log")).is_err());
        assert!(!real_state.join("viewer.log").exists());
    }
}
