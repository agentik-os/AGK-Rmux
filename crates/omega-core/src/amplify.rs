//! Mission amplifier — turns a raw user message into a structured Oracle
//! brief BEFORE dispatch.
//!
//! This is the single missing wire the Vaults analysis identified: OmegaOS
//! piped raw user text straight to the brain, so an oracle received a
//! copy-paste instead of a structured mission. We mirror Vaults'
//! `enhance_prompt()`: a tool-disabled one-shot Claude pass that STRUCTURES
//! (never rewrites) the message into
//!   ## Mission / ## Context / ## Tasks / ## Success Criteria / ## Constraints
//!
//! Design constraints (from docs/plans/VAULTS-PROMPT-ANALYSIS.md §6 "what to AVOID"):
//!  - The Brain uses NO tools (deterministic, fast — not a mini-agent).
//!  - It STRUCTURES, never invents terminology or features.
//!  - Output is mission-only — rules/identity live in the oracle system
//!    prompt (Layer 2), never inlined here.
//!  - Skip-gates keep it cheap: trivial + already-structured messages pass
//!    through verbatim; results are cached per (project, hash).
//!
//! Sync by design (spawns a blocking subprocess). Async callers should wrap
//! it in `tokio::task::spawn_blocking`.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

/// Below this many chars (after trim) a message is too small to be worth a
/// Brain pass — it's a follow-up or a one-liner. Pass it through verbatim so
/// conversational turns stay natural.
const TRIVIAL_CHARS: usize = 40;
/// At/above this many chars, if the message is ALREADY structured (markdown
/// headings or lists), the user pre-wrote a brief — don't second-guess it.
const STRUCTURED_CHARS: usize = 400;
/// LRU cap for the (project, hash) → brief cache.
const CACHE_CAP: usize = 64;

type MissionCache = (Vec<u64>, HashMap<u64, String>);

fn cache() -> &'static Mutex<MissionCache> {
    use std::sync::OnceLock;
    static C: OnceLock<Mutex<MissionCache>> = OnceLock::new();
    C.get_or_init(|| Mutex::new((Vec::new(), HashMap::new())))
}

fn hash_key(project: &str, raw: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    project.hash(&mut h);
    raw.hash(&mut h);
    h.finish()
}

fn cache_get(key: u64) -> Option<String> {
    let guard = cache().lock().ok()?;
    guard.1.get(&key).cloned()
}

fn cache_put(key: u64, value: String) {
    if let Ok(mut guard) = cache().lock() {
        if guard.1.contains_key(&key) {
            return;
        }
        if guard.0.len() >= CACHE_CAP {
            if let Some(old) = guard.0.first().copied() {
                guard.0.remove(0);
                guard.1.remove(&old);
            }
        }
        guard.0.push(key);
        guard.1.insert(key, value);
    }
}

/// True when a message is already a structured brief the user pre-wrote.
fn is_already_structured(raw: &str) -> bool {
    raw.len() >= STRUCTURED_CHARS
        && (raw.contains("\n## ")
            || raw.starts_with("## ")
            || raw.contains("\n- ")
            || raw.contains("\n1. ")
            || raw.contains("\n```"))
}

/// Should this message skip the Brain pass and be sent verbatim?
fn should_skip(raw: &str) -> bool {
    let t = raw.trim();
    if t.chars().count() < TRIVIAL_CHARS {
        return true;
    }
    if is_already_structured(t) {
        return true;
    }
    false
}

/// Neutralize untrusted text (git output, user message) so it cannot
/// masquerade as the Brain prompt's own structure. A commit message or user
/// line that starts with a markdown heading (`## `) or a fence/HR (`---`) could
/// otherwise be read as a new instruction section or break the `---` delimiter
/// around the raw message. We defang only the structural markers at line start
/// (prefixing a single leading space so a heading, HR, or fence no longer
/// begins the line), preserving the text's meaning.
fn defang_prompt_structure(s: &str) -> String {
    s.lines()
        .map(|line| {
            let t = line.trim_start();
            if t.starts_with("## ") || t == "---" || t.starts_with("---") || t.starts_with("```") {
                format!(" {}", line)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Gather a small, fault-tolerant snapshot of the project's real git state so
/// the Brain can write an accurate `## Context` section. Each piece degrades
/// independently — a missing repo or git error just omits that line.
fn gather_project_context(work_dir: &str) -> String {
    let git = |args: &[&str]| -> Option<String> {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(work_dir)
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    };
    let mut parts = Vec::new();
    if let Some(branch) = git(&["rev-parse", "--abbrev-ref", "HEAD"]) {
        parts.push(format!("branch: {}", branch));
    }
    if let Some(log) = git(&["log", "--oneline", "-5"]) {
        parts.push(format!("recent commits:\n{}", log));
    }
    if let Some(status) = git(&["status", "--short"]) {
        parts.push(format!("uncommitted changes:\n{}", status));
    }
    // Commit messages / filenames are user-controlled; defang any line that
    // could be read as prompt structure before it reaches the Brain.
    defang_prompt_structure(&parts.join("\n"))
}

/// The strict instruction template. The Brain receives the user's raw message
/// plus the project context and must emit ONLY the five-section brief.
fn build_brain_prompt(raw: &str, project: &str, ctx: &str) -> String {
    // The user message is untrusted: a line like `## Mission` or a bare `---`
    // could inject a fake section or break the `---{raw}---` delimiter and
    // smuggle directives into the Brain's instruction set. Defang structural
    // markers at line start before interpolation. (ctx is already defanged by
    // gather_project_context.)
    let raw = defang_prompt_structure(raw);
    let raw = raw.as_str();
    let ctx_block = if ctx.is_empty() {
        String::new()
    } else {
        format!("\n\nLive project state (for the ## Context section — do not invent beyond this):\n{}\n", ctx)
    };
    format!(
        "You are a dispatch brief writer for an autonomous engineering oracle working on the project \"{project}\".\n\
         Your ONLY job is to STRUCTURE the user's message below into a clear brief. Follow these rules exactly:\n\
         - Output EXACTLY these five markdown sections, in order, nothing else:\n\
           ## Mission\n  ## Context\n  ## Tasks\n  ## Success Criteria\n  ## Constraints\n\
         - STRUCTURE the message, do NOT rewrite its intent or invent features/terminology the user did not state.\n\
         - If the user is vague, keep the Mission faithfully vague — do not fabricate specifics.\n\
         - A Linear /project/ URL is a SPEC to read, not a ticket list. A Google Drive / web link is content to READ.\n\
         - Keep it under 350 words. No preamble, no sign-off, no code fences around the whole thing.\n\
         - Use NO tools. Just write the brief.{ctx_block}\n\
         User message to structure:\n---\n{raw}\n---"
    )
}

/// Deterministic fallback brief when the Brain pass is unavailable or fails.
/// Never blocks dispatch — a structured-ish brief beats raw copy-paste.
fn fallback_brief(raw: &str, project: &str, ctx: &str) -> String {
    let ctx_line = if ctx.is_empty() {
        format!("Project: {}.", project)
    } else {
        format!("Project: {}.\n{}", project, ctx)
    };
    format!(
        "## Mission\n{raw}\n\n\
         ## Context\n{ctx_line}\n\n\
         ## Tasks\n- Analyze the mission and decompose into worker-sized tasks.\n\n\
         ## Success Criteria\n- The mission is satisfied and verified with runtime evidence (build/tests/prod).\n\n\
         ## Constraints\n- Follow the Three Laws. Verify before reporting done."
    )
}

/// Amplify a raw user message into a structured Oracle brief. Skip-gated and
/// cached. Falls back to a deterministic skeleton if the Brain pass fails —
/// it NEVER returns an error and never blocks dispatch.
pub fn amplify_mission(raw: &str, project: &str, work_dir: &str) -> String {
    if should_skip(raw) {
        return raw.to_string();
    }
    let key = hash_key(project, raw);
    if let Some(cached) = cache_get(key) {
        return cached;
    }

    let ctx = gather_project_context(work_dir);
    let brief = run_brain(raw, project, &ctx).unwrap_or_else(|| fallback_brief(raw, project, &ctx));

    cache_put(key, brief.clone());
    brief
}

/// Run the tool-disabled one-shot Claude Brain pass. Returns None on any
/// failure (timeout, non-zero exit, empty output) so the caller falls back.
fn run_brain(raw: &str, project: &str, ctx: &str) -> Option<String> {
    let prompt = build_brain_prompt(raw, project, ctx);
    // Single turn, opus alias (= latest = 4.8). Strip the API key so it uses
    // the OAuth subscription, matching the rest of the system.
    //
    // Tooling: we DON'T pass `--allowed-tools` — empirically (with this CLI,
    // 2.1.156) an empty value makes the run burn its single turn with no
    // output, and the variadic flag greedily eats a positional prompt. The
    // combination of `--max-turns 1` + a strict "use NO tools, just write the
    // brief" instruction reliably yields a clean text-only brief. If the
    // Brain ever does try a tool, max-turns 1 ends it with no usable text →
    // our sanity check rejects it → fallback_brief. Robust either way.
    //
    // Prompt goes via STDIN (not a positional arg) so nothing can be misread
    // as a flag value.
    let mut cmd = std::process::Command::new("claude");
    cmd.args(["--print", "--model", "opus", "--max-turns", "1"]);
    cmd.env_remove("ANTHROPIC_API_KEY");
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::null());

    // Bound the wall-clock so a hung Brain never stalls dispatch.
    // A spawn failure (most commonly: the `claude` CLI is not installed or not
    // on PATH) is a hard dependency being unavailable — surface it instead of
    // failing silently, then fall back so dispatch is never blocked.
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                error = %e,
                "amplify: failed to spawn `claude` Brain (CLI missing or not executable?) — using deterministic fallback brief"
            );
            return None;
        }
    };
    // Write the prompt to stdin, then close it so claude starts processing.
    {
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(prompt.as_bytes());
            // dropping stdin here closes the pipe (EOF)
        }
    }
    let timeout = Duration::from_secs(60);
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return None;
                }
                break;
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    // kill() sends SIGKILL on Unix; wait() reaps the process so
                    // it can't accumulate as a zombie across repeated dispatch
                    // calls. Best-effort: a kill failure still returns None.
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(150));
            }
            Err(_) => return None,
        }
    }
    let out = child.wait_with_output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    // Sanity: a real brief has the Mission heading. Reject garbage.
    if text.len() < 20 || !text.contains("## Mission") {
        return None;
    }
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trivial_passes_through() {
        assert!(should_skip("yo"));
        assert!(should_skip("fix the build"));
        assert!(!should_skip(
            "Please redesign the entire dashboard to feel like Linear with a command palette and keyboard nav"
        ));
    }

    #[test]
    fn prewritten_brief_passes_through() {
        let brief = format!(
            "## Mission\n{}\n## Tasks\n- a\n- b\n## Success Criteria\n- done",
            "x".repeat(400)
        );
        assert!(is_already_structured(&brief));
        assert!(should_skip(&brief));
    }

    #[test]
    fn fallback_is_structured() {
        let f = fallback_brief("do the thing", "OmegaOS", "branch: main");
        assert!(f.contains("## Mission"));
        assert!(f.contains("## Success Criteria"));
        assert!(f.contains("do the thing"));
    }

    #[test]
    fn cache_roundtrip() {
        let k = hash_key("P", "some unique mission text here");
        cache_put(k, "brief-A".to_string());
        assert_eq!(cache_get(k), Some("brief-A".to_string()));
    }
}
