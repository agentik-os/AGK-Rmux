//! Authenticated skill catalog, detail, safe edit, and rmux delegation.

use crate::protocol::{
    CreateSessionRequest, SkillAgentRequest, SkillAgentResponse, SkillDeleteRequest, SkillDetail,
    SkillDetailResponse, SkillEntry, SkillRenameRequest, SkillUpdateRequest, SkillsResponse,
};
use crate::server::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use omega_core::skill_registry::{Skill, SkillRegistry};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path as FsPath, PathBuf};
use std::sync::{Mutex, OnceLock};

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 200;
const MAX_SKILL_CONTENT_BYTES: usize = 2 * 1024 * 1024;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn api_error(status: StatusCode, message: impl Into<String>) -> ApiError {
    (status, Json(serde_json::json!({ "error": message.into() })))
}

fn write_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn safe_skill_path(root: &FsPath, skill: &Skill) -> anyhow::Result<PathBuf> {
    let root = root.canonicalize()?;
    let metadata = fs::symlink_metadata(&skill.path)?;
    anyhow::ensure!(
        !metadata.file_type().is_symlink(),
        "symlinked skill files are not editable"
    );
    let path = skill.path.canonicalize()?;
    anyhow::ensure!(
        path.starts_with(&root),
        "skill path escapes the owned skills root"
    );
    anyhow::ensure!(
        path.file_name().and_then(|name| name.to_str()) == Some("SKILL.md"),
        "invalid skill file"
    );
    Ok(path)
}

fn detail_from(root: &FsPath, skill: &Skill) -> anyhow::Result<SkillDetail> {
    let path = safe_skill_path(root, skill)?;
    let content = fs::read_to_string(path)?;
    Ok(SkillDetail {
        name: skill.name.clone(),
        description: skill.description.clone(),
        category: skill.category.label().to_string(),
        content,
        read_only: skill.read_only,
    })
}

fn atomic_replace(path: &FsPath, bytes: &[u8]) -> anyhow::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("skill file has no parent"))?;
    let temporary = parent.join(format!(".SKILL.md.{}.tmp", crate::util::random_hex(8)));
    let result = (|| -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::set_permissions(&temporary, fs::metadata(path)?.permissions())?;
        fs::rename(&temporary, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn update_at_root(root: &FsPath, name: &str, content: &str) -> anyhow::Result<SkillDetail> {
    anyhow::ensure!(
        !content.contains('\0'),
        "skill content must not contain a NUL byte"
    );
    anyhow::ensure!(
        content.len() <= MAX_SKILL_CONTENT_BYTES,
        "skill content exceeds the 2 MiB limit"
    );
    let _guard = write_lock()
        .lock()
        .map_err(|_| anyhow::anyhow!("skill write lock poisoned"))?;

    let registry = SkillRegistry::discover(root)?;
    let skill = registry
        .get(name)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("skill not found"))?;
    anyhow::ensure!(!skill.read_only, "skill is read-only");
    let path = safe_skill_path(root, &skill)?;
    let original = fs::read(&path)?;

    atomic_replace(&path, content.as_bytes())?;
    let refreshed = SkillRegistry::discover(root);
    match refreshed.and_then(|registry| {
        let skill = registry.get(name).ok_or_else(|| {
            anyhow::anyhow!("edited content changed or invalidated the skill identity")
        })?;
        detail_from(root, skill)
    }) {
        Ok(detail) => Ok(detail),
        Err(error) => {
            atomic_replace(&path, &original)?;
            Err(error)
        }
    }
}

/// A skill directory name the operator may create. Deliberately narrower than
/// what the filesystem accepts: no separator, no dot-segment, no leading dot,
/// so a rename can never walk out of the skills root or shadow a hidden file.
fn validate_skill_name(name: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        (1..=64).contains(&name.len()),
        "a skill name is 1 to 64 characters"
    );
    anyhow::ensure!(
        name.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
        "a skill name may only contain letters, digits, '-' and '_'"
    );
    Ok(())
}

/// The directory that holds a skill: `<root>/<name>/SKILL.md` -> `<root>/<name>`.
/// Derived from the CANONICALIZED, root-checked file path, never from the
/// caller's string, so the traversal guard covers the directory too.
fn skill_dir_of(root: &FsPath, skill: &Skill) -> anyhow::Result<PathBuf> {
    let file = safe_skill_path(root, skill)?;
    let dir = file
        .parent()
        .ok_or_else(|| anyhow::anyhow!("skill file has no parent"))?
        .to_path_buf();
    anyhow::ensure!(
        dir != root.canonicalize()?,
        "a skill directory cannot be the skills root"
    );
    Ok(dir)
}

/// True when `install.sh` owns this skill directory (it stamps `.omega-managed`
/// after mirroring). Those directories are SSOT mirrors kept in sync with
/// `rsync --delete`, so a rename would leave a duplicate under both names and
/// a delete would resurrect the skill on the next install. Refusing with a
/// reason beats a destructive action that silently undoes itself.
fn is_install_managed(dir: &FsPath) -> bool {
    dir.join(".omega-managed").exists()
}

/// Rewrite the frontmatter `name:` of a SKILL.md.
///
/// A skill's IDENTITY is that field, not its directory name, so moving the
/// directory alone leaves a skill still called by its old name sitting in a
/// directory called something else — it then resolves under neither. Only the
/// first `name:` inside the leading `---` block is touched; a `name:` in the
/// body is prose and must survive untouched.
fn rewrite_frontmatter_name(content: &str, new_name: &str) -> anyhow::Result<String> {
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();
    anyhow::ensure!(
        lines.first().map(|l| l.trim()) == Some("---"),
        "skill has no frontmatter block"
    );
    let end = lines
        .iter()
        .skip(1)
        .position(|l| l.trim() == "---")
        .map(|i| i + 1)
        .ok_or_else(|| anyhow::anyhow!("skill frontmatter is unterminated"))?;
    let target = lines[1..end]
        .iter()
        .position(|l| l.trim_start().starts_with("name:"))
        .map(|i| i + 1)
        .ok_or_else(|| anyhow::anyhow!("skill frontmatter has no name field"))?;
    lines[target] = format!("name: {new_name}");
    let mut out = lines.join("\n");
    if content.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

fn rename_at_root(root: &FsPath, name: &str, new_name: &str) -> anyhow::Result<()> {
    validate_skill_name(new_name)?;
    let _guard = write_lock()
        .lock()
        .map_err(|_| anyhow::anyhow!("skill write lock poisoned"))?;

    let registry = SkillRegistry::discover(root)?;
    let skill = registry
        .get(name)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("skill not found"))?;
    anyhow::ensure!(!skill.read_only, "skill is read-only");
    if new_name == name {
        return Ok(());
    }
    anyhow::ensure!(
        registry.get(new_name).is_none(),
        "a skill with that name already exists"
    );

    let from = skill_dir_of(root, &skill)?;
    anyhow::ensure!(!is_install_managed(&from), "skill is installer-managed");
    let to = root.canonicalize()?.join(new_name);
    anyhow::ensure!(!to.exists(), "a skill with that name already exists");

    // The directory move and the identity rewrite must both land or neither:
    // a skill whose frontmatter still says the old name resolves under NO
    // name at all once its directory has moved.
    let file = from.join("SKILL.md");
    let original = fs::read_to_string(&file)?;
    let renamed = rewrite_frontmatter_name(&original, new_name)?;
    fs::rename(&from, &to)?;
    let moved = to.join("SKILL.md");
    let settled = atomic_replace(&moved, renamed.as_bytes())
        .and_then(|()| SkillRegistry::discover(root))
        .and_then(|registry| {
            registry
                .get(new_name)
                .map(|_| ())
                .ok_or_else(|| anyhow::anyhow!("the renamed skill no longer resolves"))
        });
    if let Err(error) = settled {
        let _ = atomic_replace(&moved, original.as_bytes());
        let _ = fs::rename(&to, &from);
        return Err(error);
    }
    Ok(())
}

fn delete_at_root(root: &FsPath, name: &str, confirm_name: &str) -> anyhow::Result<()> {
    // Deleting a skill is irreversible and there is no undo behind this API,
    // so the caller must name the exact skill a second time (R-DESTRUCT).
    anyhow::ensure!(
        confirm_name == name,
        "confirm_name must repeat the skill name exactly"
    );
    let _guard = write_lock()
        .lock()
        .map_err(|_| anyhow::anyhow!("skill write lock poisoned"))?;

    let registry = SkillRegistry::discover(root)?;
    let skill = registry
        .get(name)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("skill not found"))?;
    anyhow::ensure!(!skill.read_only, "skill is read-only");

    let dir = skill_dir_of(root, &skill)?;
    anyhow::ensure!(!is_install_managed(&dir), "skill is installer-managed");
    fs::remove_dir_all(&dir)?;
    Ok(())
}

pub async fn list(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<SkillsResponse>, ApiError> {
    let response = tokio::task::spawn_blocking(move || -> anyhow::Result<SkillsResponse> {
        let registry = SkillRegistry::discover_default()?;

        let all = registry.list();
        let total = all.len();
        let query = params
            .get("q")
            .map(|value| value.to_lowercase())
            .filter(|value| !value.is_empty());
        let category = params
            .get("category")
            .map(|value| value.to_lowercase())
            .filter(|value| !value.is_empty() && value != "all");
        let limit = params
            .get("limit")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(DEFAULT_LIMIT)
            .min(MAX_LIMIT);

        let skills = all
            .into_iter()
            .filter(|skill| {
                category
                    .as_ref()
                    .map(|value| skill.category.label().to_lowercase() == *value)
                    .unwrap_or(true)
            })
            .filter(|skill| match &query {
                None => true,
                Some(value) => {
                    skill.name.to_lowercase().contains(value)
                        || skill.description.to_lowercase().contains(value)
                }
            })
            .take(limit)
            .map(|skill| SkillEntry {
                name: skill.name.clone(),
                description: skill.description.clone(),
                category: skill.category.label().to_string(),
            })
            .collect();

        Ok(SkillsResponse { skills, total })
    })
    .await
    .map_err(|error| {
        api_error(
            StatusCode::BAD_GATEWAY,
            format!("skill discovery task panicked: {error}"),
        )
    })?
    .map_err(|error| {
        tracing::warn!("skill discovery failed: {error}");
        api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "skill catalog unavailable; run ./install.sh or omega sync",
        )
    })?;
    Ok(Json(response))
}

pub async fn get(Path(name): Path<String>) -> Result<Json<SkillDetailResponse>, ApiError> {
    let detail = tokio::task::spawn_blocking(move || {
        let registry = SkillRegistry::discover_default()?;
        let skill = registry
            .get(&name)
            .ok_or_else(|| anyhow::anyhow!("skill not found"))?;
        detail_from(registry.skills_dir(), skill)
    })
    .await
    .map_err(|error| {
        api_error(
            StatusCode::BAD_GATEWAY,
            format!("skill task panicked: {error}"),
        )
    })?
    .map_err(|error| {
        let status = if error.to_string() == "skill not found" {
            StatusCode::NOT_FOUND
        } else if error
            .to_string()
            .contains("skills directory does not exist")
            || error
                .to_string()
                .contains("skills root must be a real directory")
        {
            StatusCode::SERVICE_UNAVAILABLE
        } else {
            StatusCode::BAD_REQUEST
        };
        api_error(status, error.to_string())
    })?;
    Ok(Json(SkillDetailResponse { skill: detail }))
}

/// Maps a skill write failure onto a status. Shared by rename and delete so
/// the two never drift into reporting the same condition differently.
fn write_error_status(message: &str) -> StatusCode {
    if message == "skill not found" {
        StatusCode::NOT_FOUND
    } else if message == "skill is read-only" {
        StatusCode::FORBIDDEN
    } else if message == "a skill with that name already exists" {
        StatusCode::CONFLICT
    } else if message == "skill is installer-managed" {
        StatusCode::FORBIDDEN
    } else {
        StatusCode::BAD_REQUEST
    }
}

pub async fn rename(
    Path(name): Path<String>,
    Json(request): Json<SkillRenameRequest>,
) -> Result<StatusCode, ApiError> {
    tokio::task::spawn_blocking(move || {
        let registry = SkillRegistry::discover_default()?;
        rename_at_root(registry.skills_dir(), &name, request.new_name.trim())
    })
    .await
    .map_err(|error| {
        api_error(
            StatusCode::BAD_GATEWAY,
            format!("skill task panicked: {error}"),
        )
    })?
    .map_err(|error| {
        let message = error.to_string();
        api_error(write_error_status(&message), message)
    })?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete(
    Path(name): Path<String>,
    Json(request): Json<SkillDeleteRequest>,
) -> Result<StatusCode, ApiError> {
    tokio::task::spawn_blocking(move || {
        let registry = SkillRegistry::discover_default()?;
        delete_at_root(registry.skills_dir(), &name, request.confirm_name.trim())
    })
    .await
    .map_err(|error| {
        api_error(
            StatusCode::BAD_GATEWAY,
            format!("skill task panicked: {error}"),
        )
    })?
    .map_err(|error| {
        let message = error.to_string();
        api_error(write_error_status(&message), message)
    })?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update(
    Path(name): Path<String>,
    Json(request): Json<SkillUpdateRequest>,
) -> Result<Json<SkillDetailResponse>, ApiError> {
    let detail = tokio::task::spawn_blocking(move || {
        let registry = SkillRegistry::discover_default()?;
        update_at_root(registry.skills_dir(), &name, &request.content)
    })
    .await
    .map_err(|error| {
        api_error(
            StatusCode::BAD_GATEWAY,
            format!("skill task panicked: {error}"),
        )
    })?
    .map_err(|error| {
        let message = error.to_string();
        let status = if message == "skill not found" {
            StatusCode::NOT_FOUND
        } else if message == "skill is read-only" {
            StatusCode::FORBIDDEN
        } else {
            StatusCode::BAD_REQUEST
        };
        api_error(status, message)
    })?;
    Ok(Json(SkillDetailResponse { skill: detail }))
}

pub async fn ask_agent(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(request): Json<SkillAgentRequest>,
) -> Result<Json<SkillAgentResponse>, ApiError> {
    let (directory, read_only) = tokio::task::spawn_blocking({
        let name = name.clone();
        move || -> anyhow::Result<(String, bool)> {
            let registry = SkillRegistry::discover_default()?;
            let skill = registry
                .get(&name)
                .ok_or_else(|| anyhow::anyhow!("skill not found"))?;
            let path = safe_skill_path(registry.skills_dir(), skill)?;
            Ok((
                path.parent()
                    .expect("validated skill parent")
                    .to_string_lossy()
                    .into_owned(),
                skill.read_only,
            ))
        }
    })
    .await
    .map_err(|error| {
        api_error(
            StatusCode::BAD_GATEWAY,
            format!("skill task panicked: {error}"),
        )
    })?
    .map_err(|error| {
        let status = if error.to_string() == "skill not found" {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::BAD_REQUEST
        };
        api_error(status, error.to_string())
    })?;
    if read_only {
        return Err(api_error(StatusCode::FORBIDDEN, "skill is read-only"));
    }

    let operator_request = request.prompt.unwrap_or_else(|| {
        "Review this skill and improve it where runtime evidence supports a change.".to_string()
    });
    let prompt = format!(
        "Work on the OmegaOS skill in ./SKILL.md. Preserve its identity, follow the repository AGENTS.md, verify any change, and report exact evidence. Do not edit files unless the operator request requires it. Operator request: {operator_request}"
    );
    let created = crate::routes_sessions::create(
        State(state),
        Json(CreateSessionRequest {
            agent: "codex".to_string(),
            name: None,
            dir: Some(directory),
            prompt: Some(prompt),
        }),
    )
    .await?;
    Ok(Json(SkillAgentResponse {
        session: created.0.name,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill(root: &FsPath, name: &str, read_only: bool) {
        let directory = root.join(name);
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join("SKILL.md"),
            format!(
                "---\nname: {name}\ndescription: Original\nread_only: {read_only}\n---\n# Original\n"
            ),
        )
        .unwrap();
    }

    /// An installer-owned skill is a `rsync --delete` mirror: renaming it
    /// would leave a duplicate under both names and deleting it would
    /// resurrect it on the next install. Both must be refused, not performed.
    #[test]
    fn an_installer_managed_skill_can_be_neither_renamed_nor_deleted() {
        let root = tempfile::tempdir().unwrap();
        skill(root.path(), "shipped", false);
        fs::write(root.path().join("shipped").join(".omega-managed"), "").unwrap();

        assert!(rename_at_root(root.path(), "shipped", "mine").is_err());
        assert!(delete_at_root(root.path(), "shipped", "shipped").is_err());
        assert!(root.path().join("shipped").join("SKILL.md").exists());
        assert!(!root.path().join("mine").exists());

        // A skill the operator created carries no stamp and stays editable.
        skill(root.path(), "mine-own", false);
        rename_at_root(root.path(), "mine-own", "renamed").unwrap();
        delete_at_root(root.path(), "renamed", "renamed").unwrap();
        assert!(!root.path().join("renamed").exists());
    }

    /// A skill's identity is its frontmatter `name:`, not its directory, so a
    /// rename that only moves the directory produces a skill resolvable under
    /// NEITHER name. This is the regression that test caught.
    #[test]
    fn rename_moves_the_identity_too_not_just_the_directory() {
        let root = tempfile::tempdir().unwrap();
        skill(root.path(), "before", false);
        rename_at_root(root.path(), "before", "after").unwrap();

        let registry = SkillRegistry::discover(root.path()).unwrap();
        assert!(registry.get("after").is_some(), "the new name must resolve");
        assert!(
            registry.get("before").is_none(),
            "the old name must be gone"
        );
        let content = fs::read_to_string(root.path().join("after").join("SKILL.md")).unwrap();
        assert!(content.contains("name: after"));
    }

    #[test]
    fn rewrite_frontmatter_name_touches_only_the_frontmatter() {
        let content = "---\nname: old\ndescription: D\n---\n# Body\nname: not-frontmatter\n";
        let rewritten = rewrite_frontmatter_name(content, "new").unwrap();
        assert!(rewritten.starts_with("---\nname: new\ndescription: D\n---\n"));
        assert!(
            rewritten.contains("name: not-frontmatter"),
            "body prose survives"
        );
        assert!(rewritten.ends_with('\n'), "the trailing newline survives");

        assert!(rewrite_frontmatter_name("# no frontmatter\n", "new").is_err());
        assert!(rewrite_frontmatter_name("---\ndescription: D\n---\n", "new").is_err());
    }

    #[test]
    fn rename_moves_the_directory_and_keeps_the_content() {
        let root = tempfile::tempdir().unwrap();
        skill(root.path(), "alpha", false);
        rename_at_root(root.path(), "alpha", "beta").unwrap();
        assert!(!root.path().join("alpha").exists());
        assert!(root.path().join("beta").join("SKILL.md").exists());
    }

    #[test]
    fn rename_refuses_a_name_that_could_escape_the_root() {
        let root = tempfile::tempdir().unwrap();
        skill(root.path(), "alpha", false);
        for candidate in ["../escaped", "a/b", "..", ".hidden", ""] {
            assert!(
                rename_at_root(root.path(), "alpha", candidate).is_err(),
                "should refuse {candidate:?}"
            );
        }
        // The skill is untouched by every refusal.
        assert!(root.path().join("alpha").join("SKILL.md").exists());
    }

    #[test]
    fn rename_refuses_an_occupied_name_and_a_read_only_skill() {
        let root = tempfile::tempdir().unwrap();
        skill(root.path(), "alpha", false);
        skill(root.path(), "beta", false);
        assert!(rename_at_root(root.path(), "alpha", "beta").is_err());
        assert!(root.path().join("alpha").join("SKILL.md").exists());

        skill(root.path(), "locked", true);
        assert!(rename_at_root(root.path(), "locked", "unlocked").is_err());
        assert!(root.path().join("locked").join("SKILL.md").exists());
    }

    #[test]
    fn delete_requires_the_name_repeated_exactly() {
        let root = tempfile::tempdir().unwrap();
        skill(root.path(), "alpha", false);
        assert!(delete_at_root(root.path(), "alpha", "Alpha").is_err());
        assert!(delete_at_root(root.path(), "alpha", "").is_err());
        assert!(root.path().join("alpha").exists());

        delete_at_root(root.path(), "alpha", "alpha").unwrap();
        assert!(!root.path().join("alpha").exists());
    }

    #[test]
    fn delete_refuses_a_read_only_skill_and_an_unknown_one() {
        let root = tempfile::tempdir().unwrap();
        skill(root.path(), "locked", true);
        assert!(delete_at_root(root.path(), "locked", "locked").is_err());
        assert!(root.path().join("locked").exists());
        assert!(delete_at_root(root.path(), "ghost", "ghost").is_err());
    }

    #[test]
    fn update_is_atomic_and_preserves_identity() {
        let root = tempfile::tempdir().unwrap();
        skill(root.path(), "alpha", false);
        let detail = update_at_root(
            root.path(),
            "alpha",
            "---\nname: alpha\ndescription: Updated\nread_only: false\n---\n# Updated\n",
        )
        .unwrap();
        assert_eq!(detail.description, "Updated");

        let error = update_at_root(
            root.path(),
            "alpha",
            "---\nname: renamed\ndescription: Bad\n---\n",
        )
        .unwrap_err();
        assert!(error.to_string().contains("identity"));
        assert!(fs::read_to_string(root.path().join("alpha/SKILL.md"))
            .unwrap()
            .contains("name: alpha"));
    }

    #[test]
    fn update_rejects_read_only_and_symlinked_files() {
        let root = tempfile::tempdir().unwrap();
        skill(root.path(), "locked", true);
        assert_eq!(
            update_at_root(root.path(), "locked", "changed")
                .unwrap_err()
                .to_string(),
            "skill is read-only"
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let outside = root.path().join("outside.md");
            fs::write(&outside, "# Outside").unwrap();
            let directory = root.path().join("linked");
            fs::create_dir(&directory).unwrap();
            symlink(&outside, directory.join("SKILL.md")).unwrap();
            let fake = Skill {
                name: "linked".to_string(),
                description: String::new(),
                path: directory.join("SKILL.md"),
                triggers: vec![],
                phases: None,
                max_score: None,
                read_only: false,
                category: omega_core::skill_registry::SkillCategory::Custom,
                discovered_at: chrono::Utc::now(),
            };
            assert!(safe_skill_path(root.path(), &fake)
                .unwrap_err()
                .to_string()
                .contains("symlinked"));
        }
    }
}
