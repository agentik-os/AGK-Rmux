//! Append-only diagnostic log for the TUI's session-list / preview machinery.
//!
//! The TUI owns the terminal, so its stderr is invisible and `let _ =` error
//! swallowing made "the interface lost my session" reports undiagnosable
//! after the fact. Every load-bearing view event (list diff, preview capture
//! stall, rename, daemon reconnect) lands here with a timestamp instead:
//! `~/.omega/logs/tui.log`.

use std::io::Write;

/// Best-effort append — never panics and never blocks the UI on I/O errors.
pub fn log(line: impl AsRef<str>) {
    let path = crate::config::omega_dir().join("logs").join("tui.log");
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    // Cap unbounded growth: keep one rotated generation past ~5 MB.
    if let Ok(meta) = std::fs::metadata(&path) {
        if meta.len() > 5_000_000 {
            let _ = std::fs::rename(&path, path.with_extension("log.old"));
        }
    }
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let _ = writeln!(f, "[{ts}] {}", line.as_ref());
    }
}
