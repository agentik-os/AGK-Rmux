//! Project auto-discovery — walks the WHOLE home for real projects, scores
//! them, and returns them best-first. No hard-coded layout assumptions: the
//! old version only looked inside directories whose NAME matched a container
//! allowlist ("projects", "code", …) holding ≥2 git repos — on most machines
//! (including the reference VPS, whose root is `~/Station`) that found ZERO
//! projects and the Telegram "Add a project" menu came up empty.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredProject {
    pub name: String,
    pub path: PathBuf,
    /// Parent directory relative to $HOME ("Station/SideBusiness") — purely
    /// informational, shown next to the name so same-named projects differ.
    pub container: String,
    pub stack: Vec<String>,
    /// Relevance score (marker richness + recent activity − depth). Higher =
    /// shown first. Serde-defaulted so older JSON consumers keep parsing.
    #[serde(default)]
    pub score: i32,
    /// Days since the last observed activity (git HEAD/FETCH_HEAD or dir
    /// mtime), when resolvable.
    #[serde(default)]
    pub last_active_days: Option<u64>,
}

/// Directories that are never project containers — caches, dependency trees,
/// OS media folders. Pruned by EXACT name match at any depth (hidden dirs are
/// pruned separately).
const PRUNE: &[&str] = &[
    "node_modules",
    "target",
    "vendor",
    "venv",
    "__pycache__",
    "dist",
    "build",
    "out",
    "coverage",
    "Library",
    "Applications",
    "Movies",
    "Music",
    "Pictures",
    "Photos",
    "Public",
    "Templates",
    "snap",
];

/// How deep below $HOME the walk goes. 5 reaches e.g.
/// `~/Documents/dev/clients/acme/app` while keeping the walk fast.
const MAX_DEPTH: usize = 5;

/// Soft cap on results — discovery feeds button menus, not databases.
const MAX_RESULTS: usize = 200;

/// Look at the file structure to identify what kind of project this is.
fn detect_stack(path: &Path) -> Vec<String> {
    let mut stack = Vec::new();
    if path.join("package.json").exists() {
        if path.join("next.config.js").exists()
            || path.join("next.config.ts").exists()
            || path.join("next.config.mjs").exists()
        {
            stack.push("Next.js".to_string());
        } else if path.join("vite.config.ts").exists() || path.join("vite.config.js").exists() {
            stack.push("Vite".to_string());
        } else {
            stack.push("Node".to_string());
        }
    }
    if path.join("Cargo.toml").exists() {
        stack.push("Rust".to_string());
    }
    if path.join("pyproject.toml").exists() || path.join("requirements.txt").exists() {
        stack.push("Python".to_string());
    }
    if path.join("go.mod").exists() {
        stack.push("Go".to_string());
    }
    if path.join("Gemfile").exists() {
        stack.push("Ruby".to_string());
    }
    if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
        stack.push("Java".to_string());
    }
    if path.join(".git").exists() && stack.is_empty() {
        stack.push("Git repo".to_string());
    }
    stack
}

/// A MANIFEST marks a buildable project even without git.
fn has_manifest(path: &Path) -> bool {
    path.join("package.json").exists()
        || path.join("Cargo.toml").exists()
        || path.join("pyproject.toml").exists()
        || path.join("go.mod").exists()
        || path.join("Gemfile").exists()
        || path.join("composer.json").exists()
        || path.join("pubspec.yaml").exists()
        || path.join("pom.xml").exists()
        || path.join("build.gradle").exists()
}

fn is_project_dir(path: &Path) -> bool {
    path.join(".git").exists() || has_manifest(path)
}

/// Days since the freshest of: git HEAD / FETCH_HEAD mtime (real work
/// activity, survives a `find`-touched tree), falling back to the dir mtime.
fn last_active_days(path: &Path) -> Option<u64> {
    let mut newest: Option<std::time::SystemTime> = None;
    for probe in [".git/HEAD", ".git/FETCH_HEAD", ""] {
        let p = if probe.is_empty() {
            path.to_path_buf()
        } else {
            path.join(probe)
        };
        if let Ok(meta) = std::fs::metadata(&p) {
            if let Ok(m) = meta.modified() {
                if newest.is_none_or(|n| m > n) {
                    newest = Some(m);
                }
            }
        }
    }
    newest
        .and_then(|t| t.elapsed().ok())
        .map(|e| e.as_secs() / 86_400)
}

/// Relevance: marker richness + recency − depth. The ordering is the
/// "intelligence" of the discovery — a freshly-touched git+manifest project
/// near $HOME outranks a years-old bare repo buried five levels down.
fn score(path: &Path, depth: usize, days: Option<u64>) -> i32 {
    let mut s = 0i32;
    if path.join(".git").exists() {
        s += 3;
    }
    if has_manifest(path) {
        s += 2;
    }
    // An agent-ready project (Claude instructions present) is exactly what
    // the operator wants OmegaOS to manage.
    if path.join("CLAUDE.md").exists() || path.join(".claude").exists() {
        s += 2;
    }
    if path.join("README.md").exists() {
        s += 1;
    }
    s += match days {
        Some(d) if d <= 7 => 5,
        Some(d) if d <= 30 => 3,
        Some(d) if d <= 90 => 1,
        _ => 0,
    };
    s - depth.saturating_sub(1) as i32
}

/// Discover projects anywhere under `home`, best-scored first. A directory
/// that IS a project is not descended into (monorepo members belong to their
/// repo); hidden dirs, dependency/cache trees and OS media folders are pruned.
pub fn discover(home: &Path) -> Vec<DiscoveredProject> {
    let mut found: Vec<(i32, DiscoveredProject)> = Vec::new();
    let mut visited = std::collections::HashSet::new();
    walk(home, home, 0, &mut found, &mut visited);

    found.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));
    found.truncate(MAX_RESULTS);
    found.into_iter().map(|(_, p)| p).collect()
}

fn walk(
    home: &Path,
    dir: &Path,
    depth: usize,
    out: &mut Vec<(i32, DiscoveredProject)>,
    visited: &mut std::collections::HashSet<PathBuf>,
) {
    if depth > MAX_DEPTH || out.len() >= MAX_RESULTS * 2 {
        return;
    }
    // Symlink-loop guard: canonicalize once per visited dir.
    let Ok(canon) = dir.canonicalize() else {
        return;
    };
    if !visited.insert(canon) {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        let Some(name) = p.file_name().map(|n| n.to_string_lossy().to_string()) else {
            continue;
        };
        if name.starts_with('.') || PRUNE.contains(&name.as_str()) {
            continue;
        }

        if is_project_dir(&p) {
            let days = last_active_days(&p);
            let container = p
                .parent()
                .and_then(|par| par.strip_prefix(home).ok())
                .map(|r| r.to_string_lossy().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "~".to_string());
            out.push((
                score(&p, depth, days),
                DiscoveredProject {
                    name,
                    container,
                    stack: detect_stack(&p),
                    score: score(&p, depth, days),
                    last_active_days: days,
                    path: p,
                },
            ));
            continue; // never descend into a project
        }
        walk(home, &p, depth + 1, out, visited);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(p: &Path) {
        std::fs::create_dir_all(p).unwrap();
    }

    #[test]
    fn empty_dir_returns_no_projects() {
        let tmp = std::env::temp_dir().join("omega-test-empty");
        let _ = std::fs::remove_dir_all(&tmp);
        mk(&tmp);
        assert!(discover(&tmp).is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn finds_projects_under_arbitrary_dirs_and_prunes_caches() {
        let tmp = std::env::temp_dir().join("omega-test-discover");
        let _ = std::fs::remove_dir_all(&tmp);
        // ~/Station/Side/app — git project under a NON-allowlisted container.
        let app = tmp.join("Station/Side/app");
        mk(&app.join(".git"));
        // ~/random/tool — manifest-only project (no git).
        let tool = tmp.join("random/tool");
        mk(&tool);
        std::fs::write(tool.join("Cargo.toml"), "[package]").unwrap();
        // node_modules junk must be pruned.
        mk(&tmp.join("Station/node_modules/fake-pkg/.git"));
        // A nested member of a project must NOT be listed separately.
        mk(&app.join("subcrate"));
        std::fs::write(app.join("subcrate").join("Cargo.toml"), "[package]").unwrap();

        let found = discover(&tmp);
        let names: Vec<&str> = found.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"app"), "git project found: {names:?}");
        assert!(names.contains(&"tool"), "manifest project found: {names:?}");
        assert!(
            !names.contains(&"fake-pkg"),
            "node_modules pruned: {names:?}"
        );
        assert!(
            !names.contains(&"subcrate"),
            "no descent into projects: {names:?}"
        );
        // Container is the home-relative parent.
        let app_p = found.iter().find(|p| p.name == "app").unwrap();
        assert_eq!(app_p.container, "Station/Side");
        // Fresh git project outranks the bare manifest dir.
        assert!(found[0].name == "app", "scored ordering: {names:?}");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
