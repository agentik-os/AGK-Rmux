//! Skill registry — discovers and manages skills from ~/.omega/skills/.
//!
//! Skills are markdown-based instruction files that agents invoke by name.
//! Each skill directory contains a SKILL.md with frontmatter (name, description,
//! triggers, phases, scoring) and the full protocol body.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

const CATALOG_SCHEMA_VERSION: u32 = 1;
const MAX_SKILL_FILE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_FRONTMATTER_BYTES: usize = 256 * 1024;
const MAX_CATALOG_FILES: usize = 10_000;
const MAX_TRAVERSAL_DEPTH: usize = 16;
const EXCLUDED_CATALOG_DIRS: &[&str] = &[
    ".git",
    ".venv",
    "build",
    "dist",
    "node_modules",
    "target",
    "vendor",
];

/// A discovered skill with parsed metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub triggers: Vec<String>,
    pub phases: Option<u32>,
    pub max_score: Option<u32>,
    pub read_only: bool,
    pub category: SkillCategory,
    pub discovered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillCategory {
    Audit,
    Build,
    Design,
    Orchestration,
    Marketing,
    Utility,
    Custom,
}

impl SkillCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Audit => "Audit",
            Self::Build => "Build",
            Self::Design => "Design",
            Self::Orchestration => "Orchestration",
            Self::Marketing => "Marketing",
            Self::Utility => "Utility",
            Self::Custom => "Custom",
        }
    }

    fn from_path(path: &Path) -> Self {
        let path_str = path.to_string_lossy().to_lowercase();
        if path_str.contains("audit") {
            Self::Audit
        } else if path_str.contains("build") || path_str.contains("deploy") {
            Self::Build
        } else if path_str.contains("design") || path_str.contains("ui") {
            Self::Design
        } else if path_str.contains("orchestr") || path_str.contains("team") {
            Self::Orchestration
        } else if path_str.contains("market")
            || path_str.contains("seo")
            || path_str.contains("ads")
        {
            Self::Marketing
        } else {
            Self::Custom
        }
    }
}

/// One explicit source root owned by OmegaOS.
///
/// `id` is serialized into the canonical catalog while `path` is not. Using a
/// stable id keeps the digest independent from checkout location.
#[derive(Debug, Clone)]
pub struct OwnedSkillRoot {
    pub id: String,
    pub path: PathBuf,
}

impl OwnedSkillRoot {
    pub fn new(id: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
        }
    }
}

/// Progressive metadata parsed from a SKILL.md frontmatter.
///
/// Name and description are the only V1 hard requirements. The remaining
/// fields are emitted into the canonical catalog now and counted in structured
/// coverage when absent so the repository can migrate without a flag day or a
/// permanently noisy warning stream.
#[derive(Debug, Clone, Deserialize)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub triggers: Vec<String>,
    #[serde(default)]
    pub phases: Option<u32>,
    #[serde(default, alias = "maxScore")]
    pub max_score: Option<u32>,
    #[serde(default, alias = "readOnly")]
    pub read_only: bool,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub provenance: Option<ProvenanceInput>,
    #[serde(default)]
    pub compatibility: Option<CompatibilityInput>,
    #[serde(default)]
    pub risk: Option<String>,
    #[serde(default)]
    pub dependencies: Option<DependencyInput>,
    #[serde(default)]
    pub verify: Option<VerifyInput>,
    #[serde(default)]
    pub metadata: BTreeMap<String, serde_json::Value>,
}

/// Compatibility has two intentionally separate dialects.
///
/// The Agent Skills schema uses a descriptive scalar for runtime requirements,
/// while OmegaOS also supports an explicit provider-state map. Free-form prose
/// is never interpreted as provider evidence.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum CompatibilityInput {
    Providers(BTreeMap<String, ProviderCompatibilityInput>),
    Requirements(String),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ProvenanceInput {
    Source(String),
    Detail(SkillProvenance),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillProvenance {
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ProviderCompatibilityInput {
    State(String),
    Detail {
        state: String,
        #[serde(default)]
        reason: Option<String>,
        #[serde(default)]
        missing_capabilities: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ProviderState {
    Enabled,
    Excluded { reason: String },
    Unsupported { missing_capabilities: Vec<String> },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillDependencies {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum DependencyInput {
    Skills(Vec<String>),
    Detail(SkillDependencies),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum VerifyInput {
    Command(String),
    Detail(VerificationMetadata),
}

/// Parser seam for the catalog compiler. The first implementation uses
/// serde-saphyr, but callers and tests can substitute another bounded parser.
pub trait SkillFrontmatterParser {
    fn parse(&self, yaml: &str) -> Result<SkillFrontmatter>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SerdeSaphyrSkillParser;

impl SkillFrontmatterParser for SerdeSaphyrSkillParser {
    fn parse(&self, yaml: &str) -> Result<SkillFrontmatter> {
        if yaml.len() > MAX_FRONTMATTER_BYTES {
            anyhow::bail!(
                "skill frontmatter is {} bytes; maximum is {}",
                yaml.len(),
                MAX_FRONTMATTER_BYTES
            );
        }
        let options = serde_saphyr::options! {
            budget: serde_saphyr::budget! {
                max_events: 20_000,
                max_aliases: 64,
                max_anchors: 64,
                max_depth: 32,
                max_inclusion_depth: 0,
                max_documents: 1,
                max_nodes: 10_000,
                max_total_scalar_bytes: MAX_FRONTMATTER_BYTES,
                max_total_comment_bytes: 64 * 1024,
                max_merge_keys: 0,
            },
        };
        serde_saphyr::from_str_with_options(yaml, options)
            .map_err(anyhow::Error::new)
            .context("parsing bounded skill YAML")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogWarning {
    pub code: String,
    pub skill: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillCatalogEntry {
    pub name: String,
    pub description: String,
    pub root_id: String,
    pub relative_path: String,
    pub content_digest: String,
    pub aliases: Vec<String>,
    pub triggers: Vec<String>,
    pub category: SkillCategory,
    pub phases: Option<u32>,
    pub max_score: Option<u32>,
    pub read_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<SkillProvenance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
    pub dependencies: SkillDependencies,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<VerificationMetadata>,
    /// Descriptive Agent Skills compatibility requirements. These are retained
    /// as requirements, not projected into provider support claims.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility_requirements: Option<String>,
    pub provider_states: BTreeMap<String, ProviderState>,
    /// Explicit migration debt without turning every legacy skill into a
    /// warning. Consumers can report coverage or select migration batches.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_progressive_fields: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetadataCoverage {
    pub total_skills: usize,
    pub complete_skills: usize,
    pub missing_by_field: BTreeMap<String, usize>,
}

/// One owned source tree used to compile the catalog.
///
/// `path` is a local refresh hint and is deliberately excluded from every
/// deterministic digest. It lets installed readers prove that the shadow
/// catalog still describes a checkout that remains available on the machine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogSourceRoot {
    pub id: String,
    pub path: PathBuf,
}

/// Canonical, deterministic shadow catalog.
///
/// No wall clock, mtime, checkout path, or filesystem enumeration order enters
/// `content_digest`. Consumers can therefore use it as their drift key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillCatalogV1 {
    pub schema_version: u32,
    pub content_digest: String,
    /// Digest of `(root_id, relative_path, normalized SKILL.md digest)` only.
    /// Unlike `content_digest`, this can be recomputed without parsing YAML.
    #[serde(default)]
    pub source_tree_digest: String,
    /// Local source hints, not part of either deterministic digest.
    #[serde(default)]
    pub source_roots: Vec<CatalogSourceRoot>,
    #[serde(default)]
    pub metadata_coverage: MetadataCoverage,
    pub skills: Vec<SkillCatalogEntry>,
    pub warnings: Vec<CatalogWarning>,
}

impl SkillCatalogV1 {
    pub fn compile(roots: &[OwnedSkillRoot]) -> Result<Self> {
        Self::compile_with_parser(roots, &SerdeSaphyrSkillParser)
    }

    pub fn compile_with_parser(
        roots: &[OwnedSkillRoot],
        parser: &dyn SkillFrontmatterParser,
    ) -> Result<Self> {
        if roots.is_empty() {
            anyhow::bail!("SkillCatalogV1 requires at least one owned root");
        }

        let mut entries = Vec::new();
        let mut warnings = Vec::new();
        let mut root_ids = BTreeSet::new();
        let mut source_roots = Vec::new();
        for root in roots {
            if root.id.trim().is_empty() {
                anyhow::bail!("owned skill root id cannot be empty");
            }
            if !root_ids.insert(root.id.clone()) {
                anyhow::bail!("duplicate owned skill root id: {}", root.id);
            }
            let root_metadata = fs::symlink_metadata(&root.path)
                .with_context(|| format!("inspecting owned skill root {}", root.path.display()))?;
            if !root_metadata.file_type().is_dir() || root_metadata.file_type().is_symlink() {
                anyhow::bail!(
                    "owned skill root must be a real directory, not a symlink: {}",
                    root.path.display()
                );
            }
            let root_path = root.path.canonicalize().with_context(|| {
                format!("canonicalizing owned skill root {}", root.path.display())
            })?;
            if !root_path.is_dir() {
                anyhow::bail!(
                    "owned skill root is not a directory: {}",
                    root.path.display()
                );
            }
            source_roots.push(CatalogSourceRoot {
                id: root.id.clone(),
                path: root_path.clone(),
            });
            let mut skill_files = Vec::new();
            collect_skill_files(&root_path, &root_path, 0, &mut skill_files)?;
            skill_files.sort();
            if entries.len() + skill_files.len() > MAX_CATALOG_FILES {
                anyhow::bail!(
                    "catalog exceeds the {} owned skill file limit",
                    MAX_CATALOG_FILES
                );
            }
            for skill_file in skill_files {
                entries.push(compile_skill(
                    root,
                    &root_path,
                    &skill_file,
                    parser,
                    &mut warnings,
                )?);
            }
        }

        entries.sort_by(|a, b| {
            normalized_collision_key(&a.name)
                .cmp(&normalized_collision_key(&b.name))
                .then_with(|| a.root_id.cmp(&b.root_id))
                .then_with(|| a.relative_path.cmp(&b.relative_path))
        });
        validate_catalog(&entries)?;
        warnings.sort_by(|a, b| {
            normalized_collision_key(&a.skill)
                .cmp(&normalized_collision_key(&b.skill))
                .then_with(|| a.code.cmp(&b.code))
                .then_with(|| a.message.cmp(&b.message))
        });

        let metadata_coverage = metadata_coverage(&entries);
        let hash_input = serde_json::to_vec(&(CATALOG_SCHEMA_VERSION, &entries))
            .context("serializing canonical skill catalog")?;
        let content_digest = sha256_hex(&hash_input);
        let mut source_tree_input = entries
            .iter()
            .map(|entry| {
                (
                    entry.root_id.clone(),
                    entry.relative_path.clone(),
                    entry.content_digest.clone(),
                )
            })
            .collect::<Vec<_>>();
        source_tree_input.sort();
        let source_tree_digest = sha256_hex(
            &serde_json::to_vec(&source_tree_input)
                .context("serializing skill source-tree digest input")?,
        );
        source_roots.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(Self {
            schema_version: CATALOG_SCHEMA_VERSION,
            content_digest,
            source_tree_digest,
            source_roots,
            metadata_coverage,
            skills: entries,
            warnings,
        })
    }

    /// Write the shadow artifact atomically within one directory. Installation
    /// generation switching remains the caller's responsibility.
    pub fn write_json(&self, output: &Path) -> Result<()> {
        let parent = output
            .parent()
            .context("skill catalog output has no parent directory")?;
        fs::create_dir_all(parent)
            .with_context(|| format!("creating catalog output dir {}", parent.display()))?;
        let data = serde_json::to_vec_pretty(self).context("serializing SkillCatalogV1")?;
        crate::config::atomic_write_private(output, &data)
            .with_context(|| format!("publishing catalog {}", output.display()))
    }
}

fn collect_skill_files(
    owned_root: &Path,
    dir: &Path,
    depth: usize,
    out: &mut Vec<PathBuf>,
) -> Result<()> {
    if depth > MAX_TRAVERSAL_DEPTH {
        anyhow::bail!(
            "skill traversal exceeds depth {} at {}",
            MAX_TRAVERSAL_DEPTH,
            dir.display()
        );
    }
    let mut children = fs::read_dir(dir)
        .with_context(|| format!("reading skill directory {}", dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    children.sort_by_key(|entry| entry.file_name());

    for child in children {
        let path = child.path();
        let file_type = child
            .file_type()
            .with_context(|| format!("reading file type for {}", path.display()))?;
        if file_type.is_symlink() {
            // Never follow links while compiling an owned tree. This is stricter
            // than merely rejecting escaping links and removes a TOCTOU surface.
            continue;
        }
        if file_type.is_dir() {
            let name = child.file_name();
            if EXCLUDED_CATALOG_DIRS
                .iter()
                .any(|excluded| name == std::ffi::OsStr::new(excluded))
            {
                continue;
            }
            collect_skill_files(owned_root, &path, depth + 1, out)?;
        } else if file_type.is_file() && child.file_name() == "SKILL.md" {
            let canonical = path
                .canonicalize()
                .with_context(|| format!("canonicalizing skill file {}", path.display()))?;
            if !canonical.starts_with(owned_root) {
                anyhow::bail!("skill escapes owned root: {}", path.display());
            }
            out.push(canonical);
        }
    }
    Ok(())
}

fn compile_skill(
    root: &OwnedSkillRoot,
    canonical_root: &Path,
    skill_file: &Path,
    parser: &dyn SkillFrontmatterParser,
    warnings: &mut Vec<CatalogWarning>,
) -> Result<SkillCatalogEntry> {
    let metadata = fs::symlink_metadata(skill_file)
        .with_context(|| format!("reading metadata for {}", skill_file.display()))?;
    validate_owned_regular_file(skill_file, &metadata, "skill source")?;
    if metadata.len() > MAX_SKILL_FILE_BYTES {
        anyhow::bail!(
            "{} is {} bytes; maximum skill size is {}",
            skill_file.display(),
            metadata.len(),
            MAX_SKILL_FILE_BYTES
        );
    }
    let raw = fs::read_to_string(skill_file)
        .with_context(|| format!("reading {}", skill_file.display()))?;
    let normalized = raw.replace("\r\n", "\n");
    let yaml = extract_frontmatter(&normalized)
        .with_context(|| format!("extracting frontmatter from {}", skill_file.display()))?;
    let frontmatter = parser
        .parse(yaml)
        .with_context(|| format!("parsing {}", skill_file.display()))?;
    let name = frontmatter.name.trim().to_string();
    let description = frontmatter.description.trim().to_string();
    if name.is_empty() {
        anyhow::bail!("{} has an empty required name", skill_file.display());
    }
    if description.is_empty() {
        anyhow::bail!("{} has an empty required description", skill_file.display());
    }

    let relative_path = skill_file
        .strip_prefix(canonical_root)
        .with_context(|| format!("{} is outside its owned root", skill_file.display()))?;
    let relative_path = path_to_slash(relative_path)?;

    let version = frontmatter.version.clone().or_else(|| {
        frontmatter
            .metadata
            .get("version")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
    });
    if let Some(ref version_value) = version {
        if semver::Version::parse(version_value).is_err() {
            warnings.push(CatalogWarning {
                code: "legacy_version_not_semver".to_string(),
                skill: name.clone(),
                message: format!("version {version_value:?} is retained but is not strict SemVer"),
            });
        }
    }

    let provenance = match frontmatter.provenance.clone() {
        Some(ProvenanceInput::Source(source)) => Some(SkillProvenance {
            source,
            ..SkillProvenance::default()
        }),
        Some(ProvenanceInput::Detail(detail)) => {
            if detail.source.trim().is_empty() {
                anyhow::bail!("{name}: provenance.source cannot be empty");
            }
            Some(detail)
        }
        None => frontmatter
            .source
            .clone()
            .or_else(|| {
                frontmatter
                    .metadata
                    .get("source")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
            })
            .map(|source| SkillProvenance {
                source,
                ..SkillProvenance::default()
            }),
    };
    let dependencies = match frontmatter.dependencies.clone() {
        Some(DependencyInput::Skills(skills)) => SkillDependencies {
            skills,
            ..SkillDependencies::default()
        },
        Some(DependencyInput::Detail(detail)) => detail,
        None => SkillDependencies::default(),
    };
    let verification = match frontmatter.verify.clone() {
        Some(VerifyInput::Command(command)) => Some(VerificationMetadata {
            command: Some(command),
            ..VerificationMetadata::default()
        }),
        Some(VerifyInput::Detail(detail)) => Some(detail),
        None => None,
    };
    if let Some(ref verify) = verification {
        validate_verification_reference(&name, skill_file, verify)?;
    }
    let (compatibility_requirements, provider_states) = match &frontmatter.compatibility {
        Some(CompatibilityInput::Providers(declared)) => {
            (None, compile_provider_states(&name, declared)?)
        }
        Some(CompatibilityInput::Requirements(requirements)) => {
            let requirements = requirements.trim();
            if requirements.is_empty() {
                anyhow::bail!("{name}: compatibility requirements cannot be empty");
            }
            // A descriptive requirement can name tools, runtimes, or a host
            // product, but it is not a provider-state declaration. An empty
            // projection is safer than claiming support the source did not.
            (Some(requirements.to_string()), BTreeMap::new())
        }
        None => (None, compile_provider_states(&name, &BTreeMap::new())?),
    };

    let mut missing = Vec::new();
    if version.is_none() {
        missing.push("version");
    }
    if provenance.is_none() {
        missing.push("provenance");
    }
    if !matches!(
        frontmatter.compatibility,
        Some(CompatibilityInput::Providers(ref declared)) if !declared.is_empty()
    ) {
        missing.push("compatibility");
    }
    if frontmatter.risk.is_none() {
        missing.push("risk");
    }
    if frontmatter.dependencies.is_none() {
        missing.push("dependencies");
    }
    if frontmatter.verify.is_none() {
        missing.push("verify");
    }
    let mut aliases = frontmatter.aliases;
    aliases.sort_by_key(|value| normalized_collision_key(value));
    aliases.dedup_by(|a, b| normalized_collision_key(a) == normalized_collision_key(b));
    let mut triggers = frontmatter.triggers;
    if triggers.is_empty() {
        triggers.push(name.clone());
    }
    triggers.sort_by_key(|value| normalized_collision_key(value));
    triggers.dedup_by(|a, b| normalized_collision_key(a) == normalized_collision_key(b));

    Ok(SkillCatalogEntry {
        name,
        description,
        root_id: root.id.clone(),
        relative_path,
        content_digest: sha256_hex(normalized.as_bytes()),
        aliases,
        triggers,
        category: SkillCategory::from_path(skill_file),
        phases: frontmatter.phases,
        max_score: frontmatter.max_score,
        read_only: frontmatter.read_only,
        version,
        provenance,
        risk: frontmatter.risk,
        dependencies,
        verification,
        compatibility_requirements,
        provider_states,
        missing_progressive_fields: missing.into_iter().map(str::to_string).collect(),
    })
}

fn metadata_coverage(entries: &[SkillCatalogEntry]) -> MetadataCoverage {
    let mut missing_by_field = BTreeMap::new();
    for entry in entries {
        for field in &entry.missing_progressive_fields {
            *missing_by_field.entry(field.clone()).or_insert(0) += 1;
        }
    }
    MetadataCoverage {
        total_skills: entries.len(),
        complete_skills: entries
            .iter()
            .filter(|entry| entry.missing_progressive_fields.is_empty())
            .count(),
        missing_by_field,
    }
}

fn extract_frontmatter(content: &str) -> Result<&str> {
    let mut lines = content.split_inclusive('\n');
    let first = lines.next().unwrap_or_default().trim_end_matches('\n');
    if first.trim_end() != "---" {
        anyhow::bail!("SKILL.md must begin with YAML frontmatter");
    }
    let start = first.len() + usize::from(content.as_bytes().get(first.len()) == Some(&b'\n'));
    let remaining = &content[start..];
    let mut offset = 0;
    for line in remaining.split_inclusive('\n') {
        if line.trim() == "---" {
            return Ok(&remaining[..offset]);
        }
        offset += line.len();
        if offset > MAX_FRONTMATTER_BYTES {
            anyhow::bail!("skill frontmatter exceeds {} bytes", MAX_FRONTMATTER_BYTES);
        }
    }
    anyhow::bail!("SKILL.md frontmatter is not terminated")
}

fn compile_provider_states(
    skill_name: &str,
    declared: &BTreeMap<String, ProviderCompatibilityInput>,
) -> Result<BTreeMap<String, ProviderState>> {
    let mut providers = BTreeSet::from([
        "claude".to_string(),
        "codex".to_string(),
        "gemini".to_string(),
        "omegaos".to_string(),
    ]);
    providers.extend(declared.keys().map(|provider| provider.to_lowercase()));
    let mut states = BTreeMap::new();
    for provider in providers {
        let input = declared
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(&provider))
            .map(|(_, state)| state);
        let state = match input {
            Some(ProviderCompatibilityInput::State(state)) => {
                provider_state_from_parts(skill_name, &provider, state, None, &[])?
            }
            Some(ProviderCompatibilityInput::Detail {
                state,
                reason,
                missing_capabilities,
            }) => provider_state_from_parts(
                skill_name,
                &provider,
                state,
                reason.as_deref(),
                missing_capabilities,
            )?,
            // Compatibility is progressive metadata. Missing declarations
            // inherit portable/enabled behavior and are recorded in the
            // catalog's coverage data. Only an explicit excluded/unsupported
            // declaration may disable a provider.
            None => ProviderState::Enabled,
        };
        states.insert(provider, state);
    }
    Ok(states)
}

fn provider_state_from_parts(
    skill_name: &str,
    provider: &str,
    state: &str,
    reason: Option<&str>,
    missing_capabilities: &[String],
) -> Result<ProviderState> {
    match state.trim().to_ascii_lowercase().as_str() {
        "enabled" => Ok(ProviderState::Enabled),
        "excluded" => Ok(ProviderState::Excluded {
            reason: reason
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("declared excluded")
                .to_string(),
        }),
        "unsupported" => Ok(ProviderState::Unsupported {
            missing_capabilities: if missing_capabilities.is_empty() {
                vec!["unspecified".to_string()]
            } else {
                missing_capabilities.to_vec()
            },
        }),
        other => anyhow::bail!("{skill_name}: invalid provider state {other:?} for {provider}"),
    }
}

fn validate_verification_reference(
    skill_name: &str,
    skill_file: &Path,
    verify: &VerificationMetadata,
) -> Result<()> {
    if verify
        .command
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        anyhow::bail!("{skill_name}: verify.command cannot be empty");
    }
    let Some(script) = verify.script.as_deref() else {
        return Ok(());
    };
    let script_path = Path::new(script);
    if script_path.is_absolute()
        || script_path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        anyhow::bail!("{skill_name}: verify.script must stay inside the skill directory");
    }
    let skill_dir = skill_file
        .parent()
        .context("skill file has no parent directory")?;
    let resolved = skill_dir.join(script_path);
    let metadata = fs::symlink_metadata(&resolved)
        .with_context(|| format!("{skill_name}: missing verify script {}", resolved.display()))?;
    validate_owned_regular_file(&resolved, &metadata, "verify script")
        .with_context(|| format!("{skill_name}: invalid verification authority"))?;
    Ok(())
}

fn validate_owned_regular_file(
    path: &Path,
    metadata: &std::fs::Metadata,
    kind: &str,
) -> Result<()> {
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        anyhow::bail!("{kind} must be a regular owned file: {}", path.display());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 {
            anyhow::bail!("{kind} must not be hard-linked: {}", path.display());
        }
    }
    Ok(())
}

fn validate_catalog(entries: &[SkillCatalogEntry]) -> Result<()> {
    let mut normalized_names: BTreeMap<String, &SkillCatalogEntry> = BTreeMap::new();
    let mut provider_slugs: BTreeMap<String, (&str, &str)> = BTreeMap::new();
    for entry in entries {
        let key = normalized_collision_key(&entry.name);
        if let Some(previous) = normalized_names.insert(key.clone(), entry) {
            anyhow::bail!(
                "duplicate skill name after Unicode/case normalization: {:?} ({}) and {:?} ({})",
                previous.name,
                previous.relative_path,
                entry.name,
                entry.relative_path
            );
        }
        for invocation_name in std::iter::once(&entry.name).chain(entry.aliases.iter()) {
            let slug = provider_slug(invocation_name);
            if slug.is_empty() {
                anyhow::bail!(
                    "{} has an invocation name with an empty provider slug",
                    entry.name
                );
            }
            if let Some((owner, previous_name)) =
                provider_slugs.insert(slug.clone(), (&entry.name, invocation_name))
            {
                if owner != entry.name
                    || normalized_collision_key(previous_name)
                        != normalized_collision_key(invocation_name)
                {
                    anyhow::bail!(
                        "provider slug collision {slug:?}: {owner}/{previous_name} and {}/{}",
                        entry.name,
                        invocation_name
                    );
                }
            }
        }
    }

    let names: BTreeSet<_> = entries.iter().map(|entry| entry.name.as_str()).collect();
    for entry in entries {
        for dependency in &entry.dependencies.skills {
            if !names.contains(dependency.as_str()) {
                anyhow::bail!("{} depends on missing skill {}", entry.name, dependency);
            }
        }
    }
    validate_dependency_cycles(entries)
}

fn validate_dependency_cycles(entries: &[SkillCatalogEntry]) -> Result<()> {
    fn visit<'a>(
        name: &'a str,
        graph: &BTreeMap<&'a str, Vec<&'a str>>,
        visiting: &mut BTreeSet<&'a str>,
        visited: &mut BTreeSet<&'a str>,
        stack: &mut Vec<&'a str>,
    ) -> Result<()> {
        if visited.contains(name) {
            return Ok(());
        }
        if !visiting.insert(name) {
            let start = stack.iter().position(|item| *item == name).unwrap_or(0);
            let mut cycle = stack[start..].to_vec();
            cycle.push(name);
            anyhow::bail!("skill dependency cycle: {}", cycle.join(" -> "));
        }
        stack.push(name);
        if let Some(dependencies) = graph.get(name) {
            for dependency in dependencies {
                visit(dependency, graph, visiting, visited, stack)?;
            }
        }
        stack.pop();
        visiting.remove(name);
        visited.insert(name);
        Ok(())
    }

    let graph: BTreeMap<_, _> = entries
        .iter()
        .map(|entry| {
            (
                entry.name.as_str(),
                entry
                    .dependencies
                    .skills
                    .iter()
                    .map(String::as_str)
                    .collect(),
            )
        })
        .collect();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut stack = Vec::new();
    for name in graph.keys().copied() {
        visit(name, &graph, &mut visiting, &mut visited, &mut stack)?;
    }
    Ok(())
}

fn normalized_collision_key(value: &str) -> String {
    value.nfkc().collect::<String>().to_lowercase()
}

fn provider_slug(value: &str) -> String {
    let normalized = normalized_collision_key(value);
    let mut slug = String::new();
    let mut previous_dash = false;
    for ch in normalized.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            previous_dash = false;
        } else if !previous_dash && !slug.is_empty() {
            slug.push('-');
            previous_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn path_to_slash(path: &Path) -> Result<String> {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(value) => {
                components.push(value.to_string_lossy().to_string())
            }
            _ => anyhow::bail!(
                "catalog path is not a normalized relative path: {}",
                path.display()
            ),
        }
    }
    Ok(components.join("/"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// The skill registry: discovers, indexes, and looks up skills.
#[derive(Debug, Clone)]
pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
    skills_dir: PathBuf,
}

impl SkillRegistry {
    /// Create a registry from the default skills directory (~/.omega/skills/).
    pub fn discover_default() -> Result<Self> {
        let skills_dir = crate::config::omega_dir().join("skills");
        Self::discover(&skills_dir)
    }

    /// Create a registry by scanning a specific directory.
    pub fn discover(skills_dir: &Path) -> Result<Self> {
        let mut skills = HashMap::new();

        let root_metadata = match std::fs::symlink_metadata(skills_dir) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                anyhow::bail!("skills directory does not exist: {}", skills_dir.display())
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("inspecting skills root {}", skills_dir.display()))
            }
        };
        if !root_metadata.file_type().is_dir() || root_metadata.file_type().is_symlink() {
            // A missing skills dir is a config/install gap, not a valid empty
            // registry. Returning Ok here made callers claim zero installed
            // skills while hiding a broken installation.
            anyhow::bail!(
                "skills root must be a real directory, not a symlink: {}",
                skills_dir.display()
            );
        }

        let entries = std::fs::read_dir(skills_dir)
            .with_context(|| format!("reading {}", skills_dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Deny symlinked entries: a symlink under skills_dir could point to
            // ../../sensitive/path and pull skills from outside the intended tree
            // (path traversal). file_type() does NOT follow the link, unlike
            // path.is_dir(). Skip non-directories and symlinks alike.
            if !is_plain_dir(&entry)? {
                continue;
            }

            // Top-level skill dir (~/.omega/skills/<name>/SKILL.md).
            if try_register_skill(&path, &mut skills)? {
                continue;
            }

            // Otherwise recurse one level: treat this dir as a grouping
            // (e.g. ~/.omega/skills/audits/) whose children are skill dirs.
            let sub_entries = std::fs::read_dir(&path)
                .with_context(|| format!("reading skill group {}", path.display()))?;
            for sub_entry in sub_entries {
                let sub_entry = sub_entry
                    .with_context(|| format!("reading entry in skill group {}", path.display()))?;
                // Same symlink denial at the nested level.
                if is_plain_dir(&sub_entry)? {
                    try_register_skill(&sub_entry.path(), &mut skills)?;
                }
            }
        }

        tracing::info!(count = skills.len(), dir = %skills_dir.display(), "skill discovery complete");

        Ok(Self {
            skills,
            skills_dir: skills_dir.to_path_buf(),
        })
    }

    /// Compatibility projection from the canonical shadow catalog.
    ///
    /// This lets callers migrate one reader at a time while retaining the
    /// existing `SkillRegistry` API. `discovered_at` remains a runtime field
    /// and is deliberately absent from the catalog digest.
    pub fn from_catalog(catalog: &SkillCatalogV1, skills_dir: &Path) -> Self {
        let discovered_at = Utc::now();
        let skills = catalog
            .skills
            .iter()
            .map(|entry| {
                let skill = Skill {
                    name: entry.name.clone(),
                    description: entry.description.clone(),
                    path: skills_dir.join(&entry.relative_path),
                    triggers: entry.triggers.clone(),
                    phases: entry.phases,
                    max_score: entry.max_score,
                    read_only: entry.read_only,
                    category: entry.category,
                    discovered_at,
                };
                (skill.name.clone(), skill)
            })
            .collect();
        Self {
            skills,
            skills_dir: skills_dir.to_path_buf(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    pub fn list(&self) -> Vec<&Skill> {
        let mut skills: Vec<_> = self.skills.values().collect();
        skills.sort_by_key(|s| &s.name);
        skills
    }

    pub fn list_by_category(&self, category: SkillCategory) -> Vec<&Skill> {
        self.skills
            .values()
            .filter(|s| s.category == category)
            .collect()
    }

    pub fn find_by_trigger(&self, text: &str) -> Vec<&Skill> {
        let lower = text.to_lowercase();
        self.skills
            .values()
            .filter(|s| s.triggers.iter().any(|t| lower.contains(&t.to_lowercase())))
            .collect()
    }

    pub fn count(&self) -> usize {
        self.skills.len()
    }

    pub fn skills_dir(&self) -> &Path {
        &self.skills_dir
    }

    /// Register a skill programmatically (for built-in audit skills).
    pub fn register(&mut self, skill: Skill) {
        self.skills.insert(skill.name.clone(), skill);
    }

    /// Register all 23 Quality Arsenal audits from the audit registry.
    pub fn register_audits(&mut self) {
        for audit in crate::audit::all_audits() {
            let skill = Skill {
                name: audit.id.to_string(),
                description: audit.description.to_string(),
                path: self.skills_dir.join(audit.skill_path),
                triggers: audit.triggers.iter().map(|t| t.to_string()).collect(),
                phases: Some(audit.phases),
                max_score: Some(audit.max_score),
                read_only: audit.read_only,
                category: SkillCategory::Audit,
                discovered_at: Utc::now(),
            };
            self.skills.insert(skill.name.clone(), skill);
        }
    }
}

/// True iff `entry` is a real directory and NOT a symlink. Used to deny
/// symlinked skill dirs that could traverse outside `skills_dir`. Unlike
/// `Path::is_dir()`, `DirEntry::file_type()` does not follow symlinks; on the
/// rare metadata-read error we err on the side of skipping the entry.
fn is_plain_dir(entry: &std::fs::DirEntry) -> Result<bool> {
    let file_type = entry
        .file_type()
        .with_context(|| format!("reading file type for {}", entry.path().display()))?;
    Ok(file_type.is_dir() && !file_type.is_symlink())
}

/// Try to register a skill from `dir/SKILL.md`. Returns true if `dir` was a
/// skill directory (had a SKILL.md), regardless of parse success.
fn try_register_skill(dir: &Path, skills: &mut HashMap<String, Skill>) -> Result<bool> {
    let skill_file = dir.join("SKILL.md");
    match std::fs::symlink_metadata(&skill_file) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("inspecting skill file {}", skill_file.display()))
        }
        Ok(metadata) => validate_owned_regular_file(&skill_file, &metadata, "skill source")?,
    }

    let skill = parse_skill_file(&skill_file)
        .with_context(|| format!("registering skill at {}", skill_file.display()))?;
    let collision = skills.keys().find(|existing| {
        normalized_collision_key(existing) == normalized_collision_key(&skill.name)
    });
    if let Some(existing) = collision {
        anyhow::bail!(
            "duplicate discovered skill name after normalization: {existing:?} and {:?}",
            skill.name
        );
    }
    tracing::debug!(name = %skill.name, "discovered skill");
    skills.insert(skill.name.clone(), skill);
    Ok(true)
}

/// Parse a SKILL.md file to extract skill metadata from YAML-style frontmatter.
fn parse_skill_file(path: &Path) -> Result<Skill> {
    let metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("inspecting {}", path.display()))?;
    validate_owned_regular_file(path, &metadata, "skill source")?;
    if metadata.len() > MAX_SKILL_FILE_BYTES {
        anyhow::bail!(
            "{} is {} bytes; maximum skill size is {}",
            path.display(),
            metadata.len(),
            MAX_SKILL_FILE_BYTES
        );
    }
    let content =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;

    let mut name = path
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut description = String::new();
    let mut triggers = Vec::new();
    let mut phases = None;
    let mut max_score = None;
    let mut read_only = false;

    // Parse YAML-like frontmatter between --- markers
    let lines: Vec<&str> = content.lines().collect();
    let mut in_frontmatter = false;
    let mut frontmatter_seen = false;

    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed == "---" {
            if !frontmatter_seen {
                in_frontmatter = true;
                frontmatter_seen = true;
                i += 1;
                continue;
            } else {
                break;
            }
        }

        if in_frontmatter {
            if let Some((key, val)) = trimmed.split_once(':') {
                let key = key.trim();
                let raw = val.trim();
                if raw == ">" || raw == "|" || raw == ">-" || raw == "|-" {
                    // YAML block scalar: the value is the following more-indented
                    // lines (a bare split_once(':') would yield just ">"). Gather
                    // them until a line that is not indented more than the key.
                    let mut block = String::new();
                    let mut j = i + 1;
                    while j < lines.len() {
                        let l = lines[j];
                        if l.trim() == "---" {
                            break;
                        }
                        // Continuation lines are indented (space/tab); a new key at
                        // column 0 ends the block.
                        if !l.is_empty() && !l.starts_with(' ') && !l.starts_with('\t') {
                            break;
                        }
                        if !block.is_empty() {
                            block.push(' ');
                        }
                        block.push_str(l.trim());
                        j += 1;
                    }
                    let block = block.trim().to_string();
                    match key {
                        "name" => name = block,
                        "description" => description = block,
                        _ => {}
                    }
                    i = j;
                    continue;
                }
                let val = raw.trim_matches('"').trim_matches('\'');
                match key {
                    "name" => name = val.to_string(),
                    "description" => description = val.to_string(),
                    "read_only" | "readOnly" => read_only = val == "true",
                    "phases" => phases = val.parse().ok(),
                    "max_score" | "maxScore" => max_score = val.parse().ok(),
                    _ => {}
                }
            }
        }
        i += 1;
    }

    // If no frontmatter, try to extract name from first heading
    if description.is_empty() {
        for line in &lines {
            let trimmed = line.trim();
            if let Some(stripped) = trimmed.strip_prefix("# ") {
                description = stripped.to_string();
                break;
            }
        }
    }

    // Parse trigger lines (common pattern: "trigger:" or "Use when user says")
    for line in &lines {
        let trimmed = line.trim().to_lowercase();
        if trimmed.starts_with("trigger") || trimmed.contains("use when user says") {
            if let Some(quotes_content) = extract_quoted_strings(line) {
                triggers.extend(quotes_content);
            }
        }
    }

    if triggers.is_empty() {
        triggers.push(name.clone());
    }

    let category = SkillCategory::from_path(path);

    if name.trim().is_empty() {
        anyhow::bail!("{} has an empty skill name", path.display());
    }

    Ok(Skill {
        name,
        description,
        path: path.to_path_buf(),
        triggers,
        phases,
        max_score,
        read_only,
        category,
        discovered_at: Utc::now(),
    })
}

fn extract_quoted_strings(text: &str) -> Option<Vec<String>> {
    let mut strings = Vec::new();
    let mut in_quote = false;
    let mut current = String::new();
    let mut quote_char = '"';

    for ch in text.chars() {
        if !in_quote && (ch == '"' || ch == '\'') {
            in_quote = true;
            quote_char = ch;
            current.clear();
        } else if in_quote && ch == quote_char {
            in_quote = false;
            if !current.is_empty() {
                strings.push(current.clone());
            }
        } else if in_quote {
            current.push(ch);
        }
    }

    if strings.is_empty() {
        None
    } else {
        Some(strings)
    }
}

/// Audit orchestrator: auto-selects relevant audits for a mission.
#[derive(Debug, Clone)]
pub struct AuditOrchestrator {
    registry: SkillRegistry,
}

impl AuditOrchestrator {
    pub fn new(registry: SkillRegistry) -> Self {
        Self { registry }
    }

    /// Select audits relevant to a mission based on keywords.
    /// Returns skill names to invoke.
    pub fn select_for_mission(&self, mission_text: &str) -> Vec<String> {
        let ids = crate::audit::select_audits(mission_text, &[]);
        ids.into_iter().map(|id| id.to_string()).collect()
    }

    /// Select all audits (full audit mode).
    pub fn select_all(&self) -> Vec<String> {
        crate::audit::all_audits()
            .into_iter()
            .map(|a| a.id.to_string())
            .collect()
    }

    /// Get the full Skill metadata for an audit by id.
    pub fn get_audit_skill(&self, id: &str) -> Option<&Skill> {
        self.registry.get(id)
    }
}

/// Audit tracker: monitors freshness and scores across audits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTracker {
    pub entries: HashMap<String, AuditTrackerEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrackerEntry {
    pub audit_id: String,
    pub project: String,
    pub last_run: DateTime<Utc>,
    pub last_score: f32,
    pub max_score: u32,
    pub normalized_score: f32,
    pub trend: ScoreTrend,
    pub run_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreTrend {
    Improving,
    Stable,
    Declining,
    New,
}

impl AuditTracker {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Load tracker state from disk.
    pub fn load(path: &Path) -> Result<Self> {
        let content = crate::config::read_private_optional_string(path)?
            .with_context(|| format!("audit tracker does not exist at {}", path.display()))?;
        serde_json::from_str(&content).context("parsing audit tracker")
    }

    /// Save tracker state to disk.
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        crate::config::atomic_write_private(path, json.as_bytes())
            .with_context(|| format!("writing tracker to {}", path.display()))
    }

    /// Record a new audit result.
    pub fn record(&mut self, project: &str, audit_id: &str, score: f32, max_score: u32) {
        let key = format!("{}:{}", project, audit_id);
        let normalized = if max_score > 0 {
            (score / max_score as f32) * 100.0
        } else {
            0.0
        };

        let entry = self
            .entries
            .entry(key)
            .or_insert_with(|| AuditTrackerEntry {
                audit_id: audit_id.to_string(),
                project: project.to_string(),
                last_run: Utc::now(),
                last_score: 0.0,
                max_score,
                normalized_score: 0.0,
                trend: ScoreTrend::New,
                run_count: 0,
            });

        let prev_score = entry.normalized_score;
        entry.last_run = Utc::now();
        entry.last_score = score;
        entry.max_score = max_score;
        entry.normalized_score = normalized;
        entry.run_count += 1;

        entry.trend = if entry.run_count <= 1 {
            ScoreTrend::New
        } else if normalized > prev_score + 2.0 {
            ScoreTrend::Improving
        } else if normalized < prev_score - 2.0 {
            ScoreTrend::Declining
        } else {
            ScoreTrend::Stable
        };
    }

    /// Get stale audits (older than `max_age`).
    pub fn stale_audits(&self, max_age: chrono::Duration) -> Vec<&AuditTrackerEntry> {
        let cutoff = Utc::now() - max_age;
        self.entries
            .values()
            .filter(|e| e.last_run < cutoff)
            .collect()
    }

    /// Get scores for a specific project.
    pub fn project_scores(&self, project: &str) -> Vec<&AuditTrackerEntry> {
        self.entries
            .values()
            .filter(|e| e.project == project)
            .collect()
    }

    /// Get the latest entry for a specific audit on a project.
    pub fn latest(&self, project: &str, audit_id: &str) -> Option<&AuditTrackerEntry> {
        let key = format!("{}:{}", project, audit_id);
        self.entries.get(&key)
    }
}

impl Default for AuditTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn skill_category_from_path() {
        assert_eq!(
            SkillCategory::from_path(Path::new("/skills/audits/codeaudit/SKILL.md")),
            SkillCategory::Audit
        );
        assert_eq!(
            SkillCategory::from_path(Path::new("/skills/deploy/SKILL.md")),
            SkillCategory::Build
        );
        assert_eq!(
            SkillCategory::from_path(Path::new("/skills/random/SKILL.md")),
            SkillCategory::Custom
        );
    }

    #[test]
    fn parse_skill_with_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        let skill_file = skill_dir.join("SKILL.md");
        fs::write(
            &skill_file,
            "---\nname: test-skill\ndescription: A test skill\nphases: 10\nmax_score: 200\n---\n# Test Skill\nBody here.",
        )
        .unwrap();

        let skill = parse_skill_file(&skill_file).unwrap();
        assert_eq!(skill.name, "test-skill");
        assert_eq!(skill.description, "A test skill");
        assert_eq!(skill.phases, Some(10));
        assert_eq!(skill.max_score, Some(200));
    }

    #[test]
    fn parse_skill_block_scalar_description() {
        // `description: >` (YAML folded block scalar) used to yield ">" because
        // split_once(':') took the bare indicator. Now the indented continuation
        // lines are folded into the real description.
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("blocky");
        fs::create_dir_all(&skill_dir).unwrap();
        let skill_file = skill_dir.join("SKILL.md");
        fs::write(
            &skill_file,
            "---\nname: blocky\ndescription: >\n  First line of the folded\n  description spanning two lines.\nphases: 5\n---\n# Body\n",
        )
        .unwrap();
        let skill = parse_skill_file(&skill_file).unwrap();
        assert_eq!(skill.name, "blocky");
        assert_eq!(
            skill.description,
            "First line of the folded description spanning two lines."
        );
        assert_eq!(skill.phases, Some(5));
    }

    #[test]
    fn parse_skill_without_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        let skill_file = skill_dir.join("SKILL.md");
        fs::write(&skill_file, "# My Great Skill\nDoes things.").unwrap();

        let skill = parse_skill_file(&skill_file).unwrap();
        assert_eq!(skill.name, "my-skill");
        assert_eq!(skill.description, "My Great Skill");
    }

    #[test]
    fn registry_discover_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let registry = SkillRegistry::discover(dir.path()).unwrap();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn registry_rejects_missing_root_instead_of_claiming_empty_install() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing-skills");
        let error = SkillRegistry::discover(&missing).unwrap_err().to_string();
        assert!(error.contains("skills directory does not exist"), "{error}");
    }

    #[cfg(unix)]
    #[test]
    fn registry_and_catalog_reject_symlinked_roots() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        fs::create_dir(&real).unwrap();
        let linked = dir.path().join("linked");
        std::os::unix::fs::symlink(&real, &linked).unwrap();

        let discover_error = SkillRegistry::discover(&linked).unwrap_err().to_string();
        assert!(discover_error.contains("not a symlink"), "{discover_error}");
        let catalog_error = SkillCatalogV1::compile(&[OwnedSkillRoot::new("omegaos", &linked)])
            .unwrap_err()
            .to_string();
        assert!(catalog_error.contains("not a symlink"), "{catalog_error}");
    }

    #[test]
    fn registry_discover_with_skills() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("alpha");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: alpha\ndescription: Alpha skill\n---\n",
        )
        .unwrap();

        let registry = SkillRegistry::discover(dir.path()).unwrap();
        assert_eq!(registry.count(), 1);
        assert!(registry.get("alpha").is_some());
    }

    #[test]
    fn registry_discover_nested_skills() {
        let dir = tempfile::tempdir().unwrap();

        // Top-level skill.
        let top = dir.path().join("alpha");
        fs::create_dir_all(&top).unwrap();
        fs::write(top.join("SKILL.md"), "---\nname: alpha\n---\n").unwrap();

        // Nested skill under a grouping dir (e.g. audits/codeaudit/SKILL.md).
        let nested = dir.path().join("audits").join("codeaudit");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("SKILL.md"), "---\nname: codeaudit\n---\n").unwrap();

        let registry = SkillRegistry::discover(dir.path()).unwrap();
        assert_eq!(registry.count(), 2);
        assert!(registry.get("alpha").is_some());
        assert!(registry.get("codeaudit").is_some());
    }

    #[cfg(unix)]
    #[test]
    fn registry_and_catalog_reject_hardlinked_skill_authority() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.md");
        fs::write(&source, "---\nname: aliased\ndescription: Alias\n---\n").unwrap();
        let skill_dir = dir.path().join("aliased");
        fs::create_dir(&skill_dir).unwrap();
        fs::hard_link(&source, skill_dir.join("SKILL.md")).unwrap();

        let discover_error = SkillRegistry::discover(dir.path()).unwrap_err().to_string();
        assert!(discover_error.contains("hard-linked"), "{discover_error}");
        let catalog_error = SkillCatalogV1::compile(&[OwnedSkillRoot::new("omegaos", dir.path())])
            .unwrap_err()
            .to_string();
        assert!(catalog_error.contains("hard-linked"), "{catalog_error}");
    }

    #[test]
    fn registry_rejects_normalized_duplicate_names() {
        let dir = tempfile::tempdir().unwrap();
        for (folder, name) in [("one", "Alpha"), ("two", "alpha")] {
            let skill_dir = dir.path().join(folder);
            fs::create_dir(&skill_dir).unwrap();
            fs::write(
                skill_dir.join("SKILL.md"),
                format!("---\nname: {name}\ndescription: test\n---\n"),
            )
            .unwrap();
        }
        let error = SkillRegistry::discover(dir.path()).unwrap_err().to_string();
        assert!(error.contains("duplicate discovered skill name"), "{error}");
    }

    #[test]
    fn registry_register_audits() {
        let dir = tempfile::tempdir().unwrap();
        let mut registry = SkillRegistry::discover(dir.path()).unwrap();
        registry.register_audits();
        assert!(registry.count() >= 17);
        assert!(registry.get("codeaudit").is_some());
        assert!(registry.get("retentionaudit").is_some());
    }

    #[test]
    fn register_audits_joined_path_resolves() {
        // S3: skill_path consts must NOT re-include the leading "skills/" — the
        // join is onto skills_dir which already ends in "skills". A representative
        // audit's joined path must resolve to <skills_dir>/audits/<name>/SKILL.md
        // with no doubled "skills/skills" segment.
        let dir = tempfile::tempdir().unwrap();
        let skills_dir = dir.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();
        let mut registry = SkillRegistry::discover(&skills_dir).unwrap();
        registry.register_audits();

        let code = registry.get("codeaudit").unwrap();
        assert_eq!(code.path, skills_dir.join("audits/codeaudit/SKILL.md"));
        assert!(!code.path.to_string_lossy().contains("skills/skills"));
    }

    #[test]
    fn registry_find_by_trigger() {
        let dir = tempfile::tempdir().unwrap();
        let mut registry = SkillRegistry::discover(dir.path()).unwrap();
        registry.register_audits();
        let matches = registry.find_by_trigger("fix the security vulnerability");
        assert!(matches.iter().any(|s| s.name == "secaudit"));
    }

    #[test]
    fn audit_tracker_record_and_query() {
        let mut tracker = AuditTracker::new();
        tracker.record("Causio", "codeaudit", 336.0, 420);
        assert_eq!(tracker.entries.len(), 1);

        let entry = tracker.latest("Causio", "codeaudit").unwrap();
        assert_eq!(entry.run_count, 1);
        assert!((entry.normalized_score - 80.0).abs() < 0.1);
        assert_eq!(entry.trend, ScoreTrend::New);
    }

    #[test]
    fn audit_tracker_trend_detection() {
        let mut tracker = AuditTracker::new();
        tracker.record("P", "codeaudit", 200.0, 420);
        tracker.record("P", "codeaudit", 336.0, 420);
        let entry = tracker.latest("P", "codeaudit").unwrap();
        assert_eq!(entry.trend, ScoreTrend::Improving);
        assert_eq!(entry.run_count, 2);
    }

    #[test]
    fn audit_tracker_save_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tracker.json");

        let mut tracker = AuditTracker::new();
        tracker.record("Causio", "codeaudit", 336.0, 420);
        tracker.save(&path).unwrap();

        let loaded = AuditTracker::load(&path).unwrap();
        assert_eq!(loaded.entries.len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn audit_tracker_load_rejects_symlink_and_hardlink_authority() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target.json");
        fs::write(&target, r#"{"entries":{}}"#).unwrap();
        let symlink = dir.path().join("symlink.json");
        std::os::unix::fs::symlink(&target, &symlink).unwrap();
        assert!(AuditTracker::load(&symlink).is_err());

        let hardlink = dir.path().join("hardlink.json");
        fs::hard_link(&target, &hardlink).unwrap();
        assert!(AuditTracker::load(&hardlink).is_err());
    }

    #[test]
    fn audit_orchestrator_selects_audits() {
        let dir = tempfile::tempdir().unwrap();
        let mut registry = SkillRegistry::discover(dir.path()).unwrap();
        registry.register_audits();
        let orchestrator = AuditOrchestrator::new(registry);

        let selected = orchestrator.select_for_mission("fix auth flow security");
        assert!(selected.contains(&"secaudit".to_string()));
    }

    #[test]
    fn audit_orchestrator_select_all() {
        let dir = tempfile::tempdir().unwrap();
        let registry = SkillRegistry::discover(dir.path()).unwrap();
        let orchestrator = AuditOrchestrator::new(registry);
        assert_eq!(orchestrator.select_all().len(), 23);
    }

    #[test]
    fn extract_quoted_strings_works() {
        let result = extract_quoted_strings(r#"trigger: "foo", "bar baz""#).unwrap();
        assert_eq!(result, vec!["foo", "bar baz"]);
    }

    #[test]
    fn extract_quoted_strings_single_quotes() {
        let result = extract_quoted_strings("trigger: 'hello'").unwrap();
        assert_eq!(result, vec!["hello"]);
    }

    #[test]
    fn extract_quoted_strings_empty() {
        assert!(extract_quoted_strings("no quotes here").is_none());
    }

    fn write_catalog_skill(root: &Path, relative: &str, yaml: &str) {
        let dir = root.join(relative);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("SKILL.md"), format!("---\n{yaml}\n---\n# Body\n")).unwrap();
    }

    #[test]
    fn catalog_recurses_and_excludes_vendor_trees() {
        let dir = tempfile::tempdir().unwrap();
        write_catalog_skill(
            dir.path(),
            "design/deep/high-end-visual-design",
            "name: high-end-visual-design\ndescription: Premium UI",
        );
        write_catalog_skill(
            dir.path(),
            "node_modules/playwright/skill",
            "name: vendor-playwright\ndescription: Must stay excluded",
        );
        write_catalog_skill(
            dir.path(),
            ".venv/lib/python/skill",
            "name: venv-skill\ndescription: Must stay excluded",
        );

        let catalog =
            SkillCatalogV1::compile(&[OwnedSkillRoot::new("omegaos", dir.path())]).unwrap();
        assert_eq!(catalog.skills.len(), 1);
        assert_eq!(catalog.skills[0].name, "high-end-visual-design");
        assert_eq!(
            catalog.skills[0].relative_path,
            "design/deep/high-end-visual-design/SKILL.md"
        );
    }

    #[test]
    fn catalog_digest_is_independent_of_root_and_creation_order() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        write_catalog_skill(
            first.path(),
            "group/beta",
            "name: beta\ndescription: Beta skill",
        );
        write_catalog_skill(
            first.path(),
            "alpha",
            "name: alpha\ndescription: Alpha skill",
        );
        write_catalog_skill(
            second.path(),
            "alpha",
            "name: alpha\ndescription: Alpha skill",
        );
        write_catalog_skill(
            second.path(),
            "group/beta",
            "name: beta\ndescription: Beta skill",
        );

        let a = SkillCatalogV1::compile(&[OwnedSkillRoot::new("omegaos", first.path())]).unwrap();
        let b = SkillCatalogV1::compile(&[OwnedSkillRoot::new("omegaos", second.path())]).unwrap();
        assert_eq!(a.content_digest, b.content_digest);
        assert_eq!(a.source_tree_digest, b.source_tree_digest);
        assert_eq!(a.skills, b.skills);
        assert_ne!(a.source_roots[0].path, b.source_roots[0].path);

        fs::write(
            second.path().join("alpha/SKILL.md"),
            "---\nname: alpha\ndescription: Alpha skill changed\n---\n# Body\n",
        )
        .unwrap();
        let changed =
            SkillCatalogV1::compile(&[OwnedSkillRoot::new("omegaos", second.path())]).unwrap();
        assert_ne!(a.content_digest, changed.content_digest);
        assert_ne!(a.source_tree_digest, changed.source_tree_digest);
    }

    #[test]
    fn catalog_rejects_normalized_name_and_provider_slug_collisions() {
        let duplicate = tempfile::tempdir().unwrap();
        write_catalog_skill(duplicate.path(), "one", "name: Alpha\ndescription: First");
        write_catalog_skill(duplicate.path(), "two", "name: alpha\ndescription: Second");
        let error = SkillCatalogV1::compile(&[OwnedSkillRoot::new("omegaos", duplicate.path())])
            .unwrap_err()
            .to_string();
        assert!(error.contains("duplicate skill name"), "{error}");

        let slug = tempfile::tempdir().unwrap();
        write_catalog_skill(slug.path(), "one", "name: alpha-one\ndescription: First");
        write_catalog_skill(
            slug.path(),
            "two",
            "name: beta\ndescription: Second\naliases: [alpha_one]",
        );
        let error = SkillCatalogV1::compile(&[OwnedSkillRoot::new("omegaos", slug.path())])
            .unwrap_err()
            .to_string();
        assert!(error.contains("provider slug collision"), "{error}");
    }

    #[test]
    fn catalog_rejects_missing_dependencies_and_cycles() {
        let missing = tempfile::tempdir().unwrap();
        write_catalog_skill(
            missing.path(),
            "alpha",
            "name: alpha\ndescription: Alpha\ndependencies:\n  skills: [missing]",
        );
        let error = SkillCatalogV1::compile(&[OwnedSkillRoot::new("omegaos", missing.path())])
            .unwrap_err()
            .to_string();
        assert!(error.contains("depends on missing skill"), "{error}");

        let cycle = tempfile::tempdir().unwrap();
        write_catalog_skill(
            cycle.path(),
            "alpha",
            "name: alpha\ndescription: Alpha\ndependencies:\n  skills: [beta]",
        );
        write_catalog_skill(
            cycle.path(),
            "beta",
            "name: beta\ndescription: Beta\ndependencies:\n  skills: [alpha]",
        );
        let error = SkillCatalogV1::compile(&[OwnedSkillRoot::new("omegaos", cycle.path())])
            .unwrap_err()
            .to_string();
        assert!(error.contains("dependency cycle"), "{error}");
    }

    #[test]
    fn catalog_requires_name_and_description() {
        let dir = tempfile::tempdir().unwrap();
        write_catalog_skill(dir.path(), "bad", "name: bad\ndescription: ''");
        let error = SkillCatalogV1::compile(&[OwnedSkillRoot::new("omegaos", dir.path())])
            .unwrap_err()
            .to_string();
        assert!(error.contains("empty required description"), "{error}");
    }

    #[test]
    fn catalog_emits_explicit_provider_states_and_progressive_coverage() {
        let dir = tempfile::tempdir().unwrap();
        write_catalog_skill(
            dir.path(),
            "portable",
            "name: portable\ndescription: Portable\ncompatibility:\n  codex: enabled\n  gemini:\n    state: excluded\n    reason: no adapter",
        );
        let catalog =
            SkillCatalogV1::compile(&[OwnedSkillRoot::new("omegaos", dir.path())]).unwrap();
        let skill = &catalog.skills[0];
        assert_eq!(skill.compatibility_requirements, None);
        assert_eq!(skill.provider_states["codex"], ProviderState::Enabled);
        assert_eq!(
            skill.provider_states["gemini"],
            ProviderState::Excluded {
                reason: "no adapter".to_string()
            }
        );
        assert!(skill
            .missing_progressive_fields
            .contains(&"version".to_string()));
        assert_eq!(catalog.metadata_coverage.total_skills, 1);
        assert_eq!(catalog.metadata_coverage.complete_skills, 0);
        assert_eq!(catalog.metadata_coverage.missing_by_field["version"], 1);
    }

    #[test]
    fn catalog_accepts_descriptive_compatibility_without_claiming_provider_support() {
        let dir = tempfile::tempdir().unwrap();
        write_catalog_skill(
            dir.path(),
            "agent-skill",
            "name: agent-skill\ndescription: Common Agent Skills shape\ncompatibility: Requires Claude Code and Python 3.11+ for quality scoring",
        );

        let catalog =
            SkillCatalogV1::compile(&[OwnedSkillRoot::new("installed", dir.path())]).unwrap();
        let skill = &catalog.skills[0];
        assert_eq!(
            skill.compatibility_requirements.as_deref(),
            Some("Requires Claude Code and Python 3.11+ for quality scoring")
        );
        assert!(
            skill.provider_states.is_empty(),
            "descriptive requirements are not evidence that any provider is enabled"
        );
        assert!(skill
            .missing_progressive_fields
            .contains(&"compatibility".to_string()));
    }

    #[test]
    fn catalog_accepts_bounded_nested_agent_skill_metadata() {
        let dir = tempfile::tempdir().unwrap();
        write_catalog_skill(
            dir.path(),
            "nested-metadata",
            "name: nested-metadata\ndescription: Nested metadata\nmetadata:\n  version: '1.2.3'\n  source: agent-skills\n  openclaw:\n    emoji: telescope\n    requires:\n      bins: [agent-reach]\n      config:\n        network:\n          enabled: true",
        );

        let catalog =
            SkillCatalogV1::compile(&[OwnedSkillRoot::new("installed", dir.path())]).unwrap();
        let skill = &catalog.skills[0];
        assert_eq!(skill.version.as_deref(), Some("1.2.3"));
        assert_eq!(
            skill.provenance.as_ref().map(|value| value.source.as_str()),
            Some("agent-skills")
        );
    }

    #[test]
    fn catalog_keeps_nested_metadata_within_yaml_depth_budget() {
        let dir = tempfile::tempdir().unwrap();
        let mut yaml = "name: too-deep\ndescription: Too deep\nmetadata:\n".to_string();
        for depth in 0..40 {
            yaml.push_str(&"  ".repeat(depth + 1));
            yaml.push_str(&format!("level-{depth}:\n"));
        }
        yaml.push_str(&"  ".repeat(41));
        yaml.push_str("value: bounded");
        write_catalog_skill(dir.path(), "too-deep", &yaml);

        let error =
            SkillCatalogV1::compile(&[OwnedSkillRoot::new("installed", dir.path())]).unwrap_err();
        let error = format!("{error:#}");
        assert!(
            error.contains("depth") || error.contains("budget"),
            "{error}"
        );
    }

    #[test]
    fn catalog_rejects_empty_descriptive_compatibility() {
        let dir = tempfile::tempdir().unwrap();
        write_catalog_skill(
            dir.path(),
            "empty-compatibility",
            "name: empty-compatibility\ndescription: Empty compatibility\ncompatibility: ''",
        );

        let error = SkillCatalogV1::compile(&[OwnedSkillRoot::new("installed", dir.path())])
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("compatibility requirements cannot be empty"),
            "{error}"
        );
    }

    #[test]
    fn missing_progressive_compatibility_keeps_existing_providers_enabled() {
        let dir = tempfile::tempdir().unwrap();
        write_catalog_skill(
            dir.path(),
            "legacy-portable",
            "name: legacy-portable\ndescription: Portable before V1 metadata",
        );
        let catalog =
            SkillCatalogV1::compile(&[OwnedSkillRoot::new("omegaos", dir.path())]).unwrap();
        let skill = &catalog.skills[0];
        for provider in ["claude", "codex", "gemini", "omegaos"] {
            assert_eq!(
                skill.provider_states[provider],
                ProviderState::Enabled,
                "{provider} must remain enabled during progressive migration"
            );
        }
        assert!(skill
            .missing_progressive_fields
            .contains(&"compatibility".to_string()));
        assert_eq!(
            catalog.metadata_coverage.missing_by_field["compatibility"],
            1
        );
    }

    #[test]
    fn repository_catalog_has_expected_closure() {
        let repo_skills = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
        if !repo_skills.is_dir() {
            return;
        }
        let catalog =
            SkillCatalogV1::compile(&[OwnedSkillRoot::new("omegaos", repo_skills)]).unwrap();
        // 249 = 232 + omniroute + alexandria's split-out + stepper-os +
        // builder-os + market-research-os + mindset-os + brainstorm-os +
        // habit-tracker-os + design-os + execution-os + storyteller-os +
        // alignment-os + ai-logic-os (OS suite, 2026-08) + seductive-os
        // (personal magnetism, 2026-08-12) + intuitive-os (judgment
        // calibration, 2026-08-14) + identity-shift-os (evidence-based
        // transformation, 2026-08-14) + journal-os (the MIRROR nightly
        // review, 2026-08-14).
        // 253 = 249 + the four that landed without bumping this constant:
        // seductive-os (985cf1e), intuitive-os (8af1ff7), identity-shift-os
        // (242fed7) and cookbook (5cfb8a0).
        // 254 = 253 + agentic-engineering-lab (this branch). The count is the
        // point — a skill that ships without appearing here is a skill nobody
        // counted.
        assert_eq!(catalog.skills.len(), 254);
        let names: BTreeSet<_> = catalog
            .skills
            .iter()
            .map(|skill| skill.name.as_str())
            .collect();
        // 7609727 shipped skills/open-design/SKILL.md without bumping the count
        // above, so this assertion has been red on main since. Named here
        // because the closure is asserted by NUMBER: adding a skill without
        // bumping it fails a test that looks unrelated to the skill you added.
        assert!(names.contains("open-design"));
        assert!(names.contains("high-end-visual-design"));
        assert!(names.contains("caio-ai-readiness-assessment"));
        assert!(names.contains("marketing-master"));
        assert!(names.contains("stepper-os"));
        assert!(names.contains("builder-os"));
        assert!(names.contains("market-research-os"));
        assert!(names.contains("mindset-os"));
        assert!(names.contains("brainstorm-os"));
        assert!(names.contains("habit-tracker-os"));
        assert!(names.contains("design-os"));
        assert!(names.contains("execution-os"));
        assert!(names.contains("storyteller-os"));
        assert!(names.contains("alignment-os"));
        assert!(names.contains("seductive-os"));
        assert!(names.contains("intuitive-os"));
        assert!(names.contains("identity-shift-os"));
        assert!(names.contains("journal-os"));
        assert!(names.contains("ai-logic-os"));
        assert!(names.contains("agentic-engineering-lab"));
        assert!(catalog
            .skills
            .iter()
            .all(|skill| { skill.provider_states.get("codex") == Some(&ProviderState::Enabled) }));
        assert!(!names.contains("vendor-playwright"));
    }
}
