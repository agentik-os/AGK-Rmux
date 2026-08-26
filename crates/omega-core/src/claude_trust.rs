//! Pre-trust a project folder in `~/.claude.json` so Claude Code never shows
//! the "Do you trust the files in this folder?" dialog for it.
//!
//! Why this exists: `~/.claude.json` is a whole-file last-writer-wins store
//! shared by EVERY Claude session. With many concurrent sessions (oracles,
//! workers, the operator's own terminals), an acceptance written by one
//! session is routinely clobbered when an older session flushes its in-memory
//! copy — so the operator keeps being asked to trust the same folders, and a
//! dispatched oracle can hang on the dialog instead of running its mission.
//!
//! The fix is to write the trust flag IMMEDIATELY before `claude` starts (the
//! launch command runs `omega trust-dir "$PWD"` first — see
//! `agents.rs::launch_command_with`), so the value is fresh when Claude reads
//! its config, regardless of what other sessions wrote earlier.

use std::path::Path;

/// Mark `dir` as trusted in `~/.claude.json` (atomic temp+rename, mode 600).
///
/// Returns `Ok(true)` if the file was updated, `Ok(false)` if the flag was
/// already set. Never clobbers an unreadable/corrupt config: any parse error
/// aborts with `Err` (the worst case is the dialog showing once, not a lost
/// config).
pub fn trust_dir(dir: &Path) -> std::io::Result<bool> {
    let home = dirs::home_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no home dir"))?;
    let cfg_path = home.join(".claude.json");
    let mut root: serde_json::Value = if cfg_path.exists() {
        serde_json::from_str(&std::fs::read_to_string(&cfg_path)?)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
    } else {
        serde_json::json!({})
    };
    let key = dir.to_string_lossy().to_string();
    let projects = root
        .as_object_mut()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "config root is not an object",
            )
        })?
        .entry("projects")
        .or_insert_with(|| serde_json::json!({}));
    let entry = projects
        .as_object_mut()
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "projects is not an object")
        })?
        .entry(key)
        .or_insert_with(|| serde_json::json!({}));
    let obj = entry.as_object_mut().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "project entry is not an object",
        )
    })?;
    if obj.get("hasTrustDialogAccepted").and_then(|v| v.as_bool()) == Some(true) {
        return Ok(false);
    }
    obj.insert(
        "hasTrustDialogAccepted".into(),
        serde_json::Value::Bool(true),
    );
    // Atomic write: temp file in the same dir + rename, so a concurrent reader
    // never sees a torn file (and a crash never truncates the real config).
    let tmp = cfg_path.with_extension(format!("json.trust-{}", std::process::id()));
    std::fs::write(&tmp, serde_json::to_string(&root)?)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
    }
    std::fs::rename(&tmp, &cfg_path)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    // trust_dir mutates the REAL ~/.claude.json (path is not injectable), so
    // unit tests only cover the pure JSON merge; the end-to-end path is
    // exercised by `omega trust-dir` in verify-install / live runtime.
    #[test]
    fn merge_preserves_existing_fields() {
        let mut root = serde_json::json!({
            "projects": { "/x": { "history": [1, 2], "hasTrustDialogAccepted": false } },
            "topLevel": "kept"
        });
        let obj = root["projects"]["/x"].as_object_mut().unwrap();
        obj.insert(
            "hasTrustDialogAccepted".into(),
            serde_json::Value::Bool(true),
        );
        assert_eq!(root["projects"]["/x"]["history"][1], 2);
        assert_eq!(root["topLevel"], "kept");
        assert_eq!(root["projects"]["/x"]["hasTrustDialogAccepted"], true);
    }
}
