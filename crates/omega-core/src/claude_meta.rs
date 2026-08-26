//! Read the live model + token consumption of a Claude Code session from its
//! transcript, so the TUI can show "opus-4.7 · 1.2M tok" on the right of the
//! work-session preview title (replacing the static key-hint text).
//!
//! Claude Code writes one transcript per session under
//! `~/.claude/projects/<munged-cwd>/<uuid>.jsonl`, where the cwd has every
//! `/` and `.` replaced by `-`. Each assistant line carries `message.model`
//! and `message.usage.{input_tokens, output_tokens, cache_*_input_tokens}`.
//!
//! Sync (scans the file). Callers run it from `spawn_blocking`, throttled, so
//! it never touches the UI hot path.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ClaudeMeta {
    /// Short model label, e.g. "opus-4.7".
    pub model: String,
    /// Cumulative NEW tokens this session (input + output + cache creation;
    /// excludes cache_read re-reads, which would massively over-count).
    pub tokens: u64,
}

/// Map a working directory to Claude's transcript dir for it.
fn transcript_dir(working_dir: &Path) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let munged: String = working_dir
        .to_string_lossy()
        .chars()
        .map(|c| if c == '/' || c == '.' { '-' } else { c })
        .collect();
    Some(home.join(".claude/projects").join(munged))
}

/// Newest `*.jsonl` in a dir = the active session's transcript.
fn newest_jsonl(dir: &Path) -> Option<PathBuf> {
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        let Ok(meta) = entry.metadata() else { continue };
        let Ok(m) = meta.modified() else { continue };
        if best.as_ref().is_none_or(|(bt, _)| m > *bt) {
            best = Some((m, p));
        }
    }
    best.map(|(_, p)| p)
}

/// "claude-opus-4-7-20260315" → "opus-4.7". Keeps family + up to two numeric
/// version parts, drops the date suffix.
pub fn short_model(full: &str) -> String {
    let s = full.strip_prefix("claude-").unwrap_or(full);
    let mut parts = s.split('-');
    let family = parts.next().unwrap_or("");
    let mut nums: Vec<&str> = Vec::new();
    for p in parts {
        if p.chars().all(|c| c.is_ascii_digit()) && !p.is_empty() && nums.len() < 2 {
            nums.push(p);
        } else if !nums.is_empty() {
            break;
        }
    }
    if nums.is_empty() {
        family.to_string()
    } else {
        format!("{}-{}", family, nums.join("."))
    }
}

/// Compact token count: 1234 → "1.2k", 2_400_000 → "2.4M".
pub fn fmt_tokens(t: u64) -> String {
    if t >= 1_000_000 {
        format!("{:.1}M", t as f64 / 1_000_000.0)
    } else if t >= 1_000 {
        format!("{:.1}k", t as f64 / 1_000.0)
    } else {
        t.to_string()
    }
}

/// Resolve a session's REAL current working directory from rmux
/// (`#{pane_current_path}`). This is the robust path — it works for any
/// session regardless of how it was spawned, unlike OmegaSession.working_dir
/// (which classify() leaves as None).
fn pane_cwd(session_name: &str) -> Option<PathBuf> {
    let out = std::process::Command::new("rmux")
        .args([
            "list-panes",
            "-t",
            session_name,
            "-F",
            "#{pane_current_path}",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

/// Convenience: resolve the session's cwd from rmux, then read its Claude
/// model + token meta. Sync — call from spawn_blocking.
pub fn read_meta_for_session(session_name: &str) -> Option<ClaudeMeta> {
    let cwd = pane_cwd(session_name)?;
    read_meta(&cwd)
}

/// mtime-gated variant: returns the meta + the transcript's mtime ONLY when
/// the transcript has changed since `prev` (a full JSONL re-scan can be tens
/// of MB; this skips it when nothing moved). Returns None when unchanged or
/// unavailable. The caller stores the returned mtime to gate the next call.
pub fn read_meta_for_session_if_changed(
    session_name: &str,
    prev: Option<std::time::SystemTime>,
) -> Option<(ClaudeMeta, std::time::SystemTime)> {
    let cwd = pane_cwd(session_name)?;
    let mtime = transcript_mtime(&cwd)?;
    if prev == Some(mtime) {
        return None; // unchanged → skip the expensive scan
    }
    let meta = read_meta(&cwd)?;
    Some((meta, mtime))
}

/// mtime of the active transcript — for cheap re-scan gating.
pub fn transcript_mtime(working_dir: &Path) -> Option<std::time::SystemTime> {
    let dir = transcript_dir(working_dir)?;
    let file = newest_jsonl(&dir)?;
    std::fs::metadata(&file).ok()?.modified().ok()
}

/// Read model + cumulative new-token count for the session whose cwd is
/// `working_dir`. None if no transcript exists (e.g. a non-Claude session).
pub fn read_meta(working_dir: &Path) -> Option<ClaudeMeta> {
    let dir = transcript_dir(working_dir)?;
    let file = newest_jsonl(&dir)?;
    let content = std::fs::read_to_string(&file).ok()?;
    let mut model = String::new();
    let mut tokens: u64 = 0;
    for line in content.lines() {
        // Cheap pre-filter: only assistant turns carry usage.
        if !line.contains("\"usage\"") {
            continue;
        }
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let msg = v.get("message").unwrap_or(&v);
        if let Some(m) = msg.get("model").and_then(|m| m.as_str()) {
            if !m.is_empty() && m != "<synthetic>" {
                model = m.to_string();
            }
        }
        if let Some(u) = msg.get("usage") {
            let g = |k: &str| u.get(k).and_then(|x| x.as_u64()).unwrap_or(0);
            tokens += g("input_tokens") + g("output_tokens") + g("cache_creation_input_tokens");
        }
    }
    if model.is_empty() && tokens == 0 {
        return None;
    }
    Some(ClaudeMeta {
        model: short_model(&model),
        tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shorten_model() {
        assert_eq!(short_model("claude-opus-4-7-20260315"), "opus-4.7");
        assert_eq!(short_model("claude-sonnet-4-6"), "sonnet-4.6");
        assert_eq!(short_model("claude-opus-4-8"), "opus-4.8");
        assert_eq!(short_model("claude-fable-5"), "fable-5");
        assert_eq!(short_model("opus"), "opus");
    }

    #[test]
    fn token_fmt() {
        assert_eq!(fmt_tokens(950), "950");
        assert_eq!(fmt_tokens(1_200), "1.2k");
        assert_eq!(fmt_tokens(2_400_000), "2.4M");
    }
}
