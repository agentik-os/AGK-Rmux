//! Hermes Home sync — doctrine, skills, and stream-ready config.
//!
//! Hermes is not a dispatched writer. Home panes (`omega new --agent hermes`)
//! still need the same Laws plus native skills. This module is the single
//! writer for `~/.hermes/` so `omega sync` and `install.sh` cannot drift.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Marker pair around the OmegaOS paragraph in `SOUL.md`. Never wrap the
/// operator's identity — only this block is ours.
pub const SOUL_BEGIN: &str = "<!-- OMEGAOS-KERNEL:START -->";
pub const SOUL_END: &str = "<!-- OMEGAOS-KERNEL:END -->";

/// Marker pair around the `skills.external_dirs` snippet in `config.yaml`.
pub const CONFIG_BEGIN: &str = "# OMEGAOS-SKILLS:START";
pub const CONFIG_END: &str = "# OMEGAOS-SKILLS:END";

/// Curated skills always linked under `~/.hermes/skills/omegaos/`.
/// Hermes indexes these locally even if `external_dirs` is ignored.
pub const CORE_SKILLS: &[&str] = &[
    "agentic-engineering-lab",
    "planner",
    "new-project",
    "acceptance",
    "monitor",
    "cleanup",
    "brand-identity",
    "vision",
    "prd",
    "product-development-system",
];

const SOUL_BODY: &str = "You run under OmegaOS — Home TUI (`omega new --agent hermes`) and the \
messaging gateway (`hermes gateway`) share this soul. Follow `~/.omega/AGENTS.md` \
(Laws L0–L6 + named rules). Durable state is `omega progress` / `omega done` \
(`omega` is on PATH). Use Hermes native tools — do not invent Claude TaskCreate, \
`/goal`, or Codex `update_plan`. You are not the Omega Telegram Atlas bot; \
never reuse its BotFather token.";

const BUNDLE_YAML: &str = "name: omegaos\n\
description: OmegaOS Home loop — plan, build, verify, report.\n\
skills:\n\
  - agentic-engineering-lab\n\
  - planner\n\
  - acceptance\n\
  - monitor\n\
instruction: |\n\
  You run under OmegaOS. Durable state is omega progress / omega done.\n\
  Use THIS CLI's native plan/todo tool. Never invent Claude TaskCreate,\n\
  /goal, or Codex update_plan.\n";

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct HermesSyncReport {
    pub home: PathBuf,
    pub soul: bool,
    pub agents_md: bool,
    pub config: bool,
    pub bundle: bool,
    pub skills_linked: usize,
}

pub const SCOPE_BEGIN: &str = "# OMEGAOS-SCOPE:START";
pub const SCOPE_END: &str = "# OMEGAOS-SCOPE:END";

pub fn hermes_home(user_home: &Path) -> PathBuf {
    user_home.join(".hermes")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeTerminal {
    pub cwd: PathBuf,
    pub home_mode: Option<String>,
}

/// Full Hermes Home sync. Creates `~/.hermes` when missing so install.sh
/// can wire doctrine before the first `hermes chat`.
pub fn sync_hermes_home(
    user_home: &Path,
    omega_dir: &Path,
    agents_md: &Path,
) -> Result<HermesSyncReport> {
    let report = sync_hermes_tree(&hermes_home(user_home), omega_dir, Some(agents_md), None)?;
    let _ = crate::hermes_gateway::write_path_dropin(user_home);
    Ok(report)
}

/// Sync doctrine into any Hermes tree (default `~/.hermes` or a profile).
pub fn sync_hermes_tree(
    home: &Path,
    omega_dir: &Path,
    agents_md: Option<&Path>,
    scope_terminal: Option<&ScopeTerminal>,
) -> Result<HermesSyncReport> {
    std::fs::create_dir_all(home.join("skills").join("omegaos"))
        .with_context(|| format!("creating {}", home.display()))?;
    std::fs::create_dir_all(home.join("skill-bundles"))?;
    std::fs::create_dir_all(home.join("memories"))?;

    upsert_marked_file(&home.join("SOUL.md"), SOUL_BEGIN, SOUL_END, SOUL_BODY)?;

    let mut agents_ok = false;
    if let Some(agents_md) = agents_md {
        if agents_md.is_file() {
            link_if_needed(&home.join("AGENTS.md"), agents_md)?;
            agents_ok = true;
        }
    }

    let omega_skills = omega_dir.join("skills");
    let external = [
        omega_skills.to_string_lossy().into_owned(),
        omega_skills.join("audits").to_string_lossy().into_owned(),
    ];
    let config_path = home.join("config.yaml");
    let existing = std::fs::read_to_string(&config_path).unwrap_or_default();
    let mut updated = ensure_external_skill_dirs(&existing, &external);
    if let Some(terminal) = scope_terminal {
        updated = ensure_scope_terminal(
            &updated,
            &terminal.cwd.to_string_lossy(),
            terminal.home_mode.as_deref(),
        );
    }
    if updated != existing {
        std::fs::write(&config_path, updated)?;
    }

    std::fs::write(home.join("skill-bundles").join("omegaos.yaml"), BUNDLE_YAML)?;

    let linked = link_core_skills(home, &omega_skills)?;

    Ok(HermesSyncReport {
        home: home.to_path_buf(),
        soul: true,
        agents_md: agents_ok,
        config: true,
        bundle: true,
        skills_linked: linked,
    })
}

pub fn ensure_scope_terminal(existing: &str, cwd: &str, home_mode: Option<&str>) -> String {
    let mut inner = format!("terminal:\n  cwd: {cwd}");
    if let Some(mode) = home_mode {
        inner.push_str(&format!("\n  home_mode: {mode}"));
    }
    let block = format!("{SCOPE_BEGIN}\n{inner}\n{SCOPE_END}");
    match (existing.find(SCOPE_BEGIN), existing.find(SCOPE_END)) {
        (Some(start), Some(finish)) if finish > start => {
            let before = &existing[..start];
            let after = &existing[finish + SCOPE_END.len()..];
            format!("{before}{block}{after}")
        }
        _ => {
            let mut out = existing.to_string();
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&block);
            out.push('\n');
            out
        }
    }
}

fn external_dirs_inner(dirs: &[String]) -> String {
    format!(
        "  external_dirs:\n{}",
        dirs.iter()
            .map(|d| format!("    - {d}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn is_top_level_skills_key(line: &str) -> bool {
    let bare = line.trim_end();
    (bare == "skills:" || bare.starts_with("skills:"))
        && !bare.starts_with(' ')
        && !bare.starts_with('\t')
}

fn has_top_level_skills(yaml: &str) -> bool {
    yaml.lines().any(is_top_level_skills_key)
}

fn after_skills_line(yaml: &str) -> Option<usize> {
    let mut offset = 0usize;
    for line in yaml.split_inclusive('\n') {
        if is_top_level_skills_key(line.trim_end_matches(['\n', '\r'])) {
            return Some(offset + line.len());
        }
        offset += line.len();
    }
    None
}

pub fn ensure_external_skill_dirs(existing: &str, dirs: &[String]) -> String {
    let inner = external_dirs_inner(dirs);
    let nested = format!("{CONFIG_BEGIN}\n{inner}\n{CONFIG_END}");
    let standalone = format!("{CONFIG_BEGIN}\nskills:\n{inner}\n{CONFIG_END}");
    match (existing.find(CONFIG_BEGIN), existing.find(CONFIG_END)) {
        (Some(start), Some(finish)) if finish > start => {
            let before = &existing[..start];
            let after = &existing[finish + CONFIG_END.len()..];
            let use_nested = has_top_level_skills(before) || has_top_level_skills(after);
            let block = if use_nested { nested } else { standalone };
            format!("{before}{block}{after}")
        }
        _ => {
            if dirs.iter().all(|d| existing.contains(d)) && existing.contains("external_dirs") {
                return existing.to_string();
            }
            if let Some(idx) = after_skills_line(existing) {
                let mut out = String::with_capacity(existing.len() + nested.len() + 2);
                out.push_str(&existing[..idx]);
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str(&nested);
                out.push('\n');
                out.push_str(&existing[idx..]);
                return out;
            }
            let mut out = existing.to_string();
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&standalone);
            out.push('\n');
            out
        }
    }
}

pub fn upsert_marked_file(path: &Path, begin: &str, end: &str, body: &str) -> Result<()> {
    let block = format!("{begin}\n{body}\n{end}");
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let updated = match (existing.find(begin), existing.find(end)) {
        (Some(start), Some(finish)) if finish > start => {
            let mut out = String::with_capacity(existing.len() + block.len());
            out.push_str(&existing[..start]);
            out.push_str(&block);
            out.push_str(&existing[finish + end.len()..]);
            out
        }
        _ => {
            let mut out = existing;
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&block);
            out.push('\n');
            out
        }
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, updated)?;
    Ok(())
}

fn link_if_needed(dest: &Path, src: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let stale = std::fs::read_link(dest)
        .map(|target| target != src)
        .unwrap_or(false);
    if stale {
        let _ = std::fs::remove_file(dest);
    }
    if !dest.exists() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(src, dest)?;
        #[cfg(not(unix))]
        std::fs::copy(src, dest)?;
    }
    Ok(())
}

fn link_core_skills(hermes_home: &Path, omega_skills: &Path) -> Result<usize> {
    let dest_root = hermes_home.join("skills").join("omegaos");
    std::fs::create_dir_all(&dest_root)?;
    let mut linked = 0usize;
    for name in CORE_SKILLS {
        let src = omega_skills.join(name);
        if !src.join("SKILL.md").is_file() {
            continue;
        }
        let dest = dest_root.join(name);
        let stale = std::fs::read_link(&dest)
            .map(|target| target != src)
            .unwrap_or(false);
        if stale {
            let _ = std::fs::remove_file(&dest);
        }
        if dest.exists() {
            linked += 1;
            continue;
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(&src, &dest)?;
        #[cfg(not(unix))]
        {
            std::fs::create_dir_all(&dest)?;
            std::fs::copy(src.join("SKILL.md"), dest.join("SKILL.md"))?;
        }
        linked += 1;
    }
    Ok(linked)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_merge_is_idempotent_and_keeps_operator_yaml() {
        let first = ensure_external_skill_dirs("", &["/tmp/skills".into()]);
        assert!(first.contains(CONFIG_BEGIN));
        assert!(first.contains("/tmp/skills"));
        let again = ensure_external_skill_dirs(&first, &["/tmp/skills".into()]);
        assert_eq!(first, again);

        let operator = "model: nous/hermes\n";
        let merged = ensure_external_skill_dirs(operator, &["/tmp/skills".into()]);
        assert!(merged.starts_with("model: nous/hermes"));
        assert!(merged.contains("external_dirs"));

        let existing_skills = "model: nous/hermes\nskills:\n  creation:\n    enabled: true\n";
        let nested = ensure_external_skill_dirs(existing_skills, &["/tmp/skills".into()]);
        assert!(nested.contains("creation:\n    enabled: true"), "{nested}");
        assert_eq!(nested.matches("skills:").count(), 1, "{nested}");
        let nested_again = ensure_external_skill_dirs(&nested, &["/tmp/skills".into()]);
        assert_eq!(nested, nested_again);
    }

    #[test]
    fn sync_writes_soul_bundle_and_agents_link() {
        let tmp = tempfile::TempDir::new().unwrap();
        let user = tmp.path();
        let omega = user.join(".omega");
        let agents = omega.join("AGENTS.md");
        std::fs::create_dir_all(omega.join("skills").join("planner")).unwrap();
        std::fs::write(
            omega.join("skills").join("planner").join("SKILL.md"),
            "# p\n",
        )
        .unwrap();
        std::fs::write(&agents, "# kernel\n").unwrap();

        let report = sync_hermes_home(user, &omega, &agents).unwrap();
        assert!(report.soul && report.bundle);
        assert!(report.skills_linked >= 1);
        let soul = std::fs::read_to_string(report.home.join("SOUL.md")).unwrap();
        assert!(soul.contains("omega progress"));
        assert!(!soul.contains("claude --resume"));
        let bundle =
            std::fs::read_to_string(report.home.join("skill-bundles").join("omegaos.yaml"))
                .unwrap();
        assert!(bundle.contains("agentic-engineering-lab"));
        let linked = std::fs::read_link(report.home.join("AGENTS.md")).unwrap();
        assert_eq!(linked, agents);
    }

    #[test]
    fn scope_terminal_block_is_idempotent() {
        let first = ensure_scope_terminal("", "/home/agentik/workspace/mission", Some("profile"));
        assert!(first.contains("cwd: /home/agentik/workspace/mission"));
        assert!(first.contains("home_mode: profile"));
        let again =
            ensure_scope_terminal(&first, "/home/agentik/workspace/mission", Some("profile"));
        assert_eq!(first, again);
        let moved = ensure_scope_terminal(&first, "/tmp/mission", Some("profile"));
        assert!(moved.contains("cwd: /tmp/mission"));
        assert_eq!(moved.matches(SCOPE_BEGIN).count(), 1);
    }
}
