//! Documentation registry — the installed OmegaOS manual, discoverable at runtime.
//!
//! `install.sh` mirrors the repo's `docs/` tree plus the root canon files
//! (README, CLAUDE, RULES, GUIDE, …) into `~/.omega/docs/`, so a user who
//! installed from `npx` — with no git checkout anywhere — can still read the
//! whole manual. This module discovers that tree, parses each file's title and
//! one-line summary from its markdown, and hands it to the TUI's System tab
//! (and anything else that wants the manual).
//!
//! Discovery is deliberately shallow-cheap: it stats and reads only the first
//! few lines of each file for the index. Bodies are read on demand.

use std::path::{Path, PathBuf};

/// One discovered markdown document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocEntry {
    /// Human title — the first `# ` heading, else the humanized file stem.
    pub title: String,
    /// First meaningful prose line, truncated. Empty when the file has none.
    pub summary: String,
    /// Path relative to the docs root, e.g. `reference/09-command-reference.md`.
    pub rel_path: String,
    /// Display group — `Canon` for the root canon files, `Guides` for the
    /// docs/ top level, else the sub-directory name.
    pub group: String,
    /// Absolute path on disk.
    pub path: PathBuf,
    /// File size in bytes (shown so a 90-page spec is recognizable as one).
    pub bytes: u64,
}

/// Group rank — Canon first, then the top-level guides, then sub-folders
/// alphabetically. Keeps the index stable across machines.
fn group_rank(group: &str) -> u8 {
    match group {
        "Canon" => 0,
        "Guides" => 1,
        _ => 2,
    }
}

/// The installed docs root: `$OMEGA_DIR/docs`.
pub fn docs_dir() -> PathBuf {
    crate::config::omega_dir().join("docs")
}

/// Discover the installed manual. Falls back to a `docs/` directory in the
/// current working tree when nothing is installed yet, so a developer running
/// from a checkout sees the same index as a user running from `~/.omega`.
pub fn discover() -> Vec<DocEntry> {
    let installed = docs_dir();
    if installed.is_dir() {
        let found = discover_in(&installed);
        if !found.is_empty() {
            return found;
        }
    }
    let local = PathBuf::from("docs");
    if local.is_dir() {
        return discover_in(&local);
    }
    Vec::new()
}

/// Discover every `.md` under `root` (one level of sub-directories — the depth
/// the docs tree actually uses), sorted by group then title.
pub fn discover_in(root: &Path) -> Vec<DocEntry> {
    let mut out = Vec::new();
    collect(root, root, &mut out, 0);
    out.sort_by(|a, b| {
        group_rank(&a.group)
            .cmp(&group_rank(&b.group))
            .then_with(|| a.group.cmp(&b.group))
            .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
    });
    out
}

/// Recursive walk, bounded to 2 levels below the root — deeper nesting is
/// vendored material (e.g. `reference/oauth/`), not manual pages.
fn collect(root: &Path, dir: &Path, out: &mut Vec<DocEntry>, depth: usize) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if depth < 2 {
                collect(root, &path, out, depth + 1);
            }
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        if let Some(doc) = parse_entry(root, &path) {
            if is_operator_manual(&doc.rel_path) {
                out.push(doc);
            }
        }
    }
}

/// The System-tab manual is OmegaOS only. Drop internal plans, personal
/// machine paths, and side-project notes so a fresh install does not show
/// Gareth/Station/Instagram leftovers as if they were the product docs.
fn is_operator_manual(rel_path: &str) -> bool {
    let rel = rel_path.replace('\\', "/");
    let deny_prefix = [
        "superpowers/plans/",
        "superpowers/specs/",
        "plans/",
        "specs/",
        "migrations/",
    ];
    if deny_prefix.iter().any(|prefix| rel.starts_with(prefix)) {
        return false;
    }
    let file = rel.rsplit('/').next().unwrap_or(&rel);
    !matches!(
        file,
        "INSTAGRAM-PUBLIC-ACCESS.md"
            | "MENU-AUDIT.md"
            | "0RA-SIM-BRIDGE.md"
            | "marketing-mastery-alignment.md"
            | "ADR-lab-three-backends.md"
    )
}

fn parse_entry(root: &Path, path: &Path) -> Option<DocEntry> {
    let rel = path.strip_prefix(root).ok()?;
    let rel_path = rel.to_string_lossy().replace('\\', "/");
    let group = match rel.parent().and_then(|p| p.to_str()).unwrap_or("") {
        "" => "Guides".to_string(),
        "canon" => "Canon".to_string(),
        other => humanize(other),
    };
    let bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let content = std::fs::read_to_string(path).ok()?;
    let (title, summary) = title_and_summary(&content);
    let title = title.unwrap_or_else(|| {
        humanize(
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled"),
        )
    });

    Some(DocEntry {
        title,
        summary,
        rel_path,
        group,
        path: path.to_path_buf(),
        bytes,
    })
}

/// First `# ` heading and first prose line. Skips YAML frontmatter, badge/image
/// lines and HTML, which is what the READMEs open with.
fn title_and_summary(content: &str) -> (Option<String>, String) {
    let mut title = None;
    let mut summary = String::new();
    let mut in_frontmatter = false;

    for (i, raw) in content.lines().take(80).enumerate() {
        let line = raw.trim();
        if i == 0 && line == "---" {
            in_frontmatter = true;
            continue;
        }
        if in_frontmatter {
            if line == "---" {
                in_frontmatter = false;
            }
            continue;
        }
        if line.is_empty() {
            continue;
        }
        if title.is_none() {
            if let Some(rest) = line.strip_prefix("# ") {
                title = Some(rest.trim().to_string());
            }
            continue;
        }
        if summary.is_empty() && is_prose(line) {
            summary = truncate(&strip_markdown(line), 120);
        }
        if title.is_some() && !summary.is_empty() {
            break;
        }
    }
    (title, summary)
}

/// A line worth quoting as the summary: not a heading, badge, image, table,
/// HTML block, code fence or list bullet.
fn is_prose(line: &str) -> bool {
    !(line.starts_with('#')
        || line.starts_with('!')
        || line.starts_with('<')
        || line.starts_with('|')
        || line.starts_with('>')
        || line.starts_with("```")
        || line.starts_with("- ")
        || line.starts_with("* ")
        || line.starts_with("[!"))
}

/// Drop the markdown decorations that read as noise in a one-line summary.
fn strip_markdown(line: &str) -> String {
    line.replace(['`', '*'], "")
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let cut: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{}…", cut.trim_end())
}

/// `getting-started` / `GETTING_STARTED` → `Getting Started`.
fn humanize(stem: &str) -> String {
    stem.replace(['-', '_'], " ")
        .split_whitespace()
        .map(|w| {
            // Keep already-uppercase acronyms (API, SEO) intact.
            if w.len() > 1 && w.chars().all(|c| c.is_uppercase() || !c.is_alphabetic()) {
                w.to_string()
            } else {
                let mut chars = w.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Read a document's full body. Returns the error text as content so the TUI
/// always has something to render instead of a silent blank panel.
pub fn read_body(path: &Path) -> String {
    match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => format!("(could not read {}: {})", path.display(), e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, rel: &str, body: &str) {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }

    #[test]
    fn discovers_and_groups_the_tree() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            root,
            "GETTING-STARTED.md",
            "# Getting Started\n\nInstall it.\n",
        );
        write(
            root,
            "canon/README.md",
            "# OmegaOS\n\n![badge](x)\nThe OS.\n",
        );
        write(
            root,
            "reference/09-cmd.md",
            "# Commands\n\nEvery command.\n",
        );
        // Not markdown — must be ignored.
        write(root, "notes.txt", "nope");

        let docs = discover_in(root);
        assert_eq!(docs.len(), 3, "3 md files, the .txt is skipped");
        // Canon sorts first, then Guides, then sub-folders.
        assert_eq!(docs[0].group, "Canon");
        assert_eq!(docs[0].title, "OmegaOS");
        assert_eq!(docs[0].summary, "The OS.", "badge line is skipped");
        assert_eq!(docs[1].group, "Guides");
        assert_eq!(docs[2].group, "Reference");
        assert_eq!(docs[2].rel_path, "reference/09-cmd.md");
    }

    #[test]
    fn falls_back_to_the_filename_when_there_is_no_heading() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "third-party-skills.md", "Just a body line.\n");
        let docs = discover_in(tmp.path());
        assert_eq!(docs[0].title, "Third Party Skills");
    }

    #[test]
    fn skips_yaml_frontmatter() {
        let tmp = tempfile::tempdir().unwrap();
        write(
            tmp.path(),
            "skill.md",
            "---\nname: x\n---\n\n# Real Title\n\nReal summary.\n",
        );
        let docs = discover_in(tmp.path());
        assert_eq!(docs[0].title, "Real Title");
        assert_eq!(docs[0].summary, "Real summary.");
    }

    #[test]
    fn missing_root_is_empty_not_a_panic() {
        assert!(discover_in(Path::new("/nonexistent/omega/docs")).is_empty());
    }

    #[test]
    fn operator_manual_drops_internal_and_personal_pages() {
        assert!(!is_operator_manual("superpowers/plans/foo.md"));
        assert!(!is_operator_manual("INSTAGRAM-PUBLIC-ACCESS.md"));
        assert!(!is_operator_manual("MENU-AUDIT.md"));
        assert!(is_operator_manual("GETTING-STARTED.md"));
        assert!(is_operator_manual("reference/09-cmd.md"));
        assert!(is_operator_manual("THEMES.md"));
    }
}
