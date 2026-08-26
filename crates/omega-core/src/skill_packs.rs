//! OmegaSkillPacks — YAML-defined grouped skill bundles.
//!
//! A *skill pack* groups several Claude/OmegaOS skills under one name
//! with optional extra instructions. Loading a pack = importing all
//! its skills + appending the extra instructions to the dispatched
//! agent's prompt.
//!
//! Format (`~/.omega/skill-packs/<name>.yaml`):
//!
//! ```yaml
//! name: web-research
//! description: Cluster of skills for fact-finding tasks
//! skills:
//!   - reader
//!   - blog-researcher
//!   - science-researcher
//! extra_instructions: |
//!   Always cite sources with file:line or URL. Skeptical of any single
//!   data point — corroborate before quoting.
//! ```
//!
//! Hot-reload: callers check `mtime` of the file and reload if newer
//! than the cached copy. We don't auto-watch — keeping the module
//! dependency-free.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPack {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub extra_instructions: String,
}

impl SkillPack {
    /// Load a single pack from disk.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("read skill pack {}", path.display()))?;
        let pack: SkillPack = serde_yaml::from_str(&content)
            .with_context(|| format!("parse skill pack {}", path.display()))?;
        Ok(pack)
    }

    /// Format the pack for inclusion in an agent prompt.
    pub fn as_prompt_block(&self) -> String {
        let mut out = format!("## Skill pack: {}\n", self.name);
        if !self.description.is_empty() {
            out.push_str(&self.description);
            out.push_str("\n\n");
        }
        if !self.skills.is_empty() {
            out.push_str("Skills available:\n");
            for s in &self.skills {
                out.push_str(&format!("- /{}\n", s));
            }
            out.push('\n');
        }
        if !self.extra_instructions.is_empty() {
            out.push_str("Extra instructions:\n");
            out.push_str(self.extra_instructions.trim());
            out.push('\n');
        }
        out
    }
}

/// Default skill-packs directory: `~/.omega/skill-packs/`.
pub fn default_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    home.join(".omega/skill-packs")
}

/// Hot-reload-aware registry. Tracks (path → (mtime, pack)) and reloads
/// stale entries on next access.
#[derive(Debug, Default)]
pub struct SkillPackRegistry {
    dir: PathBuf,
    cache: HashMap<PathBuf, (SystemTime, SkillPack)>,
}

impl SkillPackRegistry {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            cache: HashMap::new(),
        }
    }

    pub fn default_path() -> Self {
        Self::new(default_dir())
    }

    /// List + freshness-check every pack in the directory. Returns the
    /// loaded packs in alphabetical order.
    pub fn all(&mut self) -> Result<Vec<SkillPack>> {
        if !self.dir.exists() {
            // Surface the missing-dir condition so callers don't read an
            // empty list as "zero packs installed" when the install is broken.
            // std-only (no log dep — see module note on staying dependency-free).
            eprintln!(
                "omega: skill-packs dir {} does not exist — returning no packs",
                self.dir.display()
            );
            return Ok(Vec::new());
        }
        let mut paths: Vec<PathBuf> = std::fs::read_dir(&self.dir)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                p.extension()
                    .map(|e| e == "yaml" || e == "yml")
                    .unwrap_or(false)
            })
            .collect();
        paths.sort();

        let mut out = Vec::new();
        for path in paths {
            let mtime = std::fs::metadata(&path)
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            let cached = self.cache.get(&path).cloned();
            let pack = match cached {
                Some((cached_mtime, pack)) if cached_mtime == mtime => pack,
                _ => {
                    let pack = SkillPack::load(&path)?;
                    self.cache.insert(path.clone(), (mtime, pack.clone()));
                    pack
                }
            };
            out.push(pack);
        }
        Ok(out)
    }

    /// Load a single pack by name (with or without .yaml extension).
    pub fn get(&mut self, name: &str) -> Result<Option<SkillPack>> {
        let stem = name.trim_end_matches(".yaml").trim_end_matches(".yml");
        for ext in ["yaml", "yml"] {
            let path = self.dir.join(format!("{}.{}", stem, ext));
            if path.exists() {
                let pack = SkillPack::load(&path)?;
                return Ok(Some(pack));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_pack(dir: &Path, name: &str, body: &str) -> PathBuf {
        let path = dir.join(format!("{}.yaml", name));
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        path
    }

    #[test]
    fn parse_minimal_pack() {
        let dir = std::env::temp_dir().join(format!("omega-pack-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = write_pack(
            &dir,
            "research",
            "name: research\nskills:\n  - reader\n  - science-researcher\n",
        );
        let pack = SkillPack::load(&p).unwrap();
        assert_eq!(pack.name, "research");
        assert_eq!(pack.skills.len(), 2);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn prompt_block_includes_skills() {
        let pack = SkillPack {
            name: "web".to_string(),
            description: "X".to_string(),
            skills: vec!["reader".to_string(), "blog-researcher".to_string()],
            extra_instructions: "Cite sources.".to_string(),
        };
        let block = pack.as_prompt_block();
        assert!(block.contains("## Skill pack: web"));
        assert!(block.contains("- /reader"));
        assert!(block.contains("Cite sources"));
    }

    #[test]
    fn registry_hot_reload() {
        let dir = std::env::temp_dir().join(format!("omega-reg-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut reg = SkillPackRegistry::new(dir.clone());
        write_pack(&dir, "a", "name: a\nskills:\n  - x\n");
        let first = reg.all().unwrap();
        assert_eq!(first.len(), 1);
        // Add another
        write_pack(&dir, "b", "name: b\nskills:\n  - y\n");
        let second = reg.all().unwrap();
        assert_eq!(second.len(), 2);
        std::fs::remove_dir_all(&dir).ok();
    }
}
