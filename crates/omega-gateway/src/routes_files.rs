//! `GET /v1/files` (scoped directory listing) and `GET /v1/files/read`
//! (scoped, size-capped, text-only file read) — SECURITY-CRITICAL: `path` is
//! attacker-controlled and MUST be confined under the requested project's
//! discovered root. See [`resolve_scoped_path`] for the guard and its own
//! `#[cfg(test)]` module for the traversal/symlink-escape regression tests.
//!
//! `project` is REQUIRED on both routes and validated against
//! `omega_core::projects::discover(&home)` — the exact allowlist pattern
//! `routes_dispatch.rs` uses for the same reason: browsing `$HOME` itself
//! would reach `~/.ssh`, `~/.omega/secrets`, etc. (R-ENV). Validation
//! completes fully, and the discovered project's `path` is resolved to the
//! scoped root, before any filesystem access beyond the read-only `discover`
//! walk itself.

use crate::protocol::{FileEntry, FileReadResponse, FilesResponse};
use crate::server::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

type ApiError = (StatusCode, Json<serde_json::Value>);

/// Cap on `GET /v1/files/read` — 512 KiB is far past any real source file or
/// config a client needs to view inline, while small enough that pointing
/// this at an oversized file can't balloon the gateway's memory or a mobile
/// client's response payload. Checked via `metadata().len()` BEFORE the file
/// is read into memory, never after (see [`read_capped_text`]).
pub const MAX_FILE_READ_BYTES: u64 = 512 * 1024;

fn bad_request(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": msg.into() })),
    )
}

fn not_found(msg: impl Into<String>) -> ApiError {
    (StatusCode::NOT_FOUND, Json(json!({ "error": msg.into() })))
}

fn forbidden(msg: impl Into<String>) -> ApiError {
    (StatusCode::FORBIDDEN, Json(json!({ "error": msg.into() })))
}

fn internal(msg: impl std::fmt::Display) -> ApiError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": msg.to_string() })),
    )
}

/// The three ways [`resolve_scoped_path`] can refuse to hand back a path.
/// Kept deliberately small and independent of axum so the traversal guard
/// itself has direct, fast unit tests (see this module's `#[cfg(test)]`)
/// with no HTTP server involved.
#[derive(Debug)]
pub enum PathError {
    /// `path` contained a NUL byte or was itself absolute — rejected BEFORE
    /// any filesystem touch (and, for the absolute case, before the `join`
    /// that would otherwise silently replace `root` — see the doc comment
    /// on `resolve_scoped_path`). Maps to 400.
    Invalid(String),
    /// The resolved, canonicalized path exists but lies outside `root` —
    /// either textual traversal (`../../etc/passwd`) or a symlink planted
    /// inside `root` that points outside it. Maps to 403.
    Escaped,
    /// Nothing exists at the resolved location (distinct from `Escaped`:
    /// this is "not there", not "there but forbidden"). Maps to 404.
    NotFound,
}

fn path_error_to_api(err: PathError) -> ApiError {
    match err {
        PathError::Invalid(msg) => bad_request(msg),
        PathError::Escaped => forbidden("path escapes the project root"),
        PathError::NotFound => not_found("no such file or directory"),
    }
}

/// THE security boundary for both routes in this file: resolves `rel`
/// (client-supplied, attacker-controlled) against `root` (the discovered
/// project's real on-disk path) and proves the result cannot escape `root`.
///
/// Order matters and mirrors the threat model exactly:
/// 1. Reject a NUL byte in `rel` outright — nonsensical input, never handed
///    to the filesystem.
/// 2. Reject `rel` when it is itself an ABSOLUTE path, BEFORE any `join`.
///    `PathBuf::join` silently REPLACES the base when the joined-in
///    component is absolute (`root.join("/etc/passwd") == "/etc/passwd"`),
///    which would otherwise let `path=/etc/passwd` escape `root` entirely
///    regardless of any later check — so this must be caught here, not
///    inferred from the joined result.
/// 3. Join component-wise (never string concatenation) and CANONICALIZE
///    both the joined path and `root` (`std::fs::canonicalize`, which also
///    resolves symlinks). Reject unless the canonicalized joined path
///    `.starts_with()` the canonicalized root — this is what catches BOTH
///    textual traversal AND a symlink planted inside `root` that points
///    outside it, because canonicalize resolves the symlink's real target
///    before the `starts_with` check ever runs.
/// 4. A `canonicalize` failure on the full joined path means the LEAF isn't
///    there. That alone is not enough to call it `NotFound` (404). THE
///    INVARIANT: 403-vs-404 may depend ONLY on whether the NEAREST EXISTING
///    ancestor of the requested location lies inside `root` — never on what
///    does or does not exist OUTSIDE it. Anything weaker turns the status
///    code into an existence oracle for the whole filesystem, since an
///    attacker who can observe only the status, never the content, still
///    learns which paths are real (`../../../etc/shadow` vs
///    `../../../etc/definitely-not-a-real-file`).
///    So when the leaf fails to resolve, walk UP the ancestor chain to the
///    first entry that DOES resolve: inside `root` → `NotFound` (404, a
///    genuine in-scope miss, and the client can already enumerate in-scope
///    structure via `list` anyway); outside `root` → `Escaped` (403).
///    Checking only ONE level (the leaf's immediate parent) is NOT enough:
///    it merely relocates the same oracle one directory up, where
///    `../<dir>/leaf` still answers 403-vs-404 according to whether `<dir>`
///    exists outside `root`.
///
/// A `..` component that stays WITHIN `root` after resolution is allowed
/// implicitly (canonicalize handles that correctly): the guard is about the
/// FINAL resolved location, never a textual ban on the substring `..` (which
/// would also incorrectly reject a real subdirectory literally named e.g.
/// `foo..bar`).
pub fn resolve_scoped_path(root: &Path, rel: &str) -> Result<PathBuf, PathError> {
    if rel.contains('\0') {
        return Err(PathError::Invalid(
            "path must not contain a NUL byte".into(),
        ));
    }
    let rel_path = Path::new(rel);
    if rel_path.is_absolute() {
        return Err(PathError::Invalid("path must not be absolute".into()));
    }
    let joined = root.join(rel_path);
    let canon_root = std::fs::canonicalize(root).map_err(|_| PathError::NotFound)?;
    match std::fs::canonicalize(&joined) {
        Ok(canon_joined) => {
            if canon_joined.starts_with(&canon_root) {
                Ok(canon_joined)
            } else {
                Err(PathError::Escaped)
            }
        }
        Err(_) => {
            // Nothing resolves at the leaf. Climb to the NEAREST ancestor
            // that DOES resolve and classify on that, so the answer depends
            // only on where the request landed, never on what happens to
            // exist outside `root` (see point 4 above).
            //
            // `ancestors()` is textual, which is exactly right here: each
            // candidate is still handed to `canonicalize`, so a `..` chain is
            // resolved for real before the `starts_with` test. The walk
            // always terminates: `joined` is absolute (`root` is, and `rel`
            // was proven relative above), so the chain ends at `/`, which
            // always canonicalizes -- and, not being under `root`, correctly
            // yields `Escaped`. An ancestor that fails to canonicalize for a
            // reason OTHER than absence (e.g. EACCES on a search bit) is
            // skipped too, which keeps the fallthrough on the safe side.
            for ancestor in joined.ancestors().skip(1) {
                let Ok(canon_ancestor) = std::fs::canonicalize(ancestor) else {
                    continue;
                };
                return if canon_ancestor.starts_with(&canon_root) {
                    Err(PathError::NotFound)
                } else {
                    Err(PathError::Escaped)
                };
            }
            Err(PathError::NotFound)
        }
    }
}

/// Reads `path`'s content with the same discipline as `routes_deposit.rs`'s
/// size gate: `metadata().len()` is checked BEFORE any read, so an oversized
/// file is rejected without ever being pulled into memory. After the capped
/// read, the raw bytes are rejected if they contain a NUL byte or fail UTF-8
/// decoding — this endpoint is text-only, never a binary/diff viewer.
fn read_capped_text(path: &Path) -> Result<String, ApiError> {
    let meta = std::fs::metadata(path).map_err(|e| internal(format!("stat failed: {e}")))?;
    if meta.len() > MAX_FILE_READ_BYTES {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(json!({
                "error": format!(
                    "file too large ({} bytes, max {})",
                    meta.len(),
                    MAX_FILE_READ_BYTES
                )
            })),
        ));
    }
    let bytes = std::fs::read(path).map_err(|e| internal(format!("read failed: {e}")))?;
    if bytes.contains(&0u8) {
        return Err(bad_request(
            "file contains a NUL byte, refusing as non-text",
        ));
    }
    String::from_utf8(bytes)
        .map_err(|_| bad_request("file is not valid UTF-8, refusing as non-text"))
}

/// Validates `project_name` against the discovered-project allowlist and
/// returns that project's real on-disk root — the exact pattern
/// `routes_dispatch.rs::create` uses (Step 3 there) for the same reason: no
/// filesystem access beyond this read-only `discover` walk happens for an
/// unknown project.
async fn resolve_project_root(project_name: &str) -> Result<PathBuf, ApiError> {
    if project_name.trim().is_empty() {
        return Err(bad_request("project is required"));
    }
    if project_name.contains('\0') {
        return Err(bad_request("project must not contain a NUL byte"));
    }
    let name = project_name.to_string();
    let found = tokio::task::spawn_blocking(move || {
        let home = crate::config::home_dir();
        omega_core::projects::discover(&home)
            .into_iter()
            .find(|p| p.name == name)
            .map(|p| p.path)
    })
    .await
    .unwrap_or(None);
    found.ok_or_else(|| bad_request(format!("unknown project: {project_name}")))
}

/// `GET /v1/files?project=<name>&path=<rel>` — `project` is required
/// (validated per [`resolve_project_root`]); `path` is optional and
/// defaults to `""` (the project root).
pub async fn list(
    State(_state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<FilesResponse>, ApiError> {
    let project = query.get("project").cloned().unwrap_or_default();
    let root = resolve_project_root(&project).await?;
    let rel = query.get("path").cloned().unwrap_or_default();

    let entries = tokio::task::spawn_blocking(move || -> Result<Vec<FileEntry>, ApiError> {
        let resolved = resolve_scoped_path(&root, &rel).map_err(path_error_to_api)?;
        let meta =
            std::fs::metadata(&resolved).map_err(|e| internal(format!("stat failed: {e}")))?;
        if !meta.is_dir() {
            return Err(bad_request("path is not a directory"));
        }
        let read_dir =
            std::fs::read_dir(&resolved).map_err(|e| internal(format!("read_dir failed: {e}")))?;
        let mut out = Vec::new();
        for entry in read_dir {
            let entry = entry.map_err(|e| internal(format!("read_dir entry failed: {e}")))?;
            let file_type = entry
                .file_type()
                .map_err(|e| internal(format!("file_type failed: {e}")))?;
            let is_dir = file_type.is_dir();
            let size = if is_dir {
                None
            } else {
                entry.metadata().ok().map(|m| m.len())
            };
            out.push(FileEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                is_dir,
                size,
            });
        }
        // Directories first, then alphabetical (byte order) within each
        // group — arbitrary but documented; not the focus of this endpoint.
        out.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });
        Ok(out)
    })
    .await
    .map_err(|e| internal(format!("task panicked: {e}")))??;

    Ok(Json(FilesResponse { entries }))
}

/// `GET /v1/files/read?project=<name>&path=<rel>` — both `project` and
/// `path` are required (unlike `list`, there is no sensible default for
/// reading a single file).
pub async fn read(
    State(_state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<FileReadResponse>, ApiError> {
    let project = query.get("project").cloned().unwrap_or_default();
    let root = resolve_project_root(&project).await?;
    let rel = query.get("path").cloned().unwrap_or_default();
    if rel.trim().is_empty() {
        return Err(bad_request("path is required"));
    }

    let content = tokio::task::spawn_blocking(move || -> Result<String, ApiError> {
        let resolved = resolve_scoped_path(&root, &rel).map_err(path_error_to_api)?;
        let meta =
            std::fs::metadata(&resolved).map_err(|e| internal(format!("stat failed: {e}")))?;
        if meta.is_dir() {
            return Err(bad_request("path is a directory, not a file"));
        }
        read_capped_text(&resolved)
    })
    .await
    .map_err(|e| internal(format!("task panicked: {e}")))??;

    Ok(Json(FileReadResponse { content }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_nul_byte_in_path() {
        let base = tempfile::tempdir().unwrap();
        let err = resolve_scoped_path(base.path(), "foo\0bar").unwrap_err();
        assert!(matches!(err, PathError::Invalid(_)));
    }

    #[test]
    fn rejects_absolute_path_before_any_join() {
        let base = tempfile::tempdir().unwrap();
        // /etc/passwd definitely exists on this box; if the absolute check
        // ran AFTER a join+canonicalize, this would still (correctly) end up
        // Escaped or resolve to /etc/passwd itself depending on the bug's
        // exact shape — the point of THIS test is that Invalid fires first,
        // proving the guard rejects before ever touching the filesystem via
        // a join.
        let err = resolve_scoped_path(base.path(), "/etc/passwd").unwrap_err();
        assert!(matches!(err, PathError::Invalid(_)));
    }

    #[test]
    fn rejects_textual_traversal_escaping_the_root() {
        let base = tempfile::tempdir().unwrap();
        let root = base.path().join("project");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(base.path().join("secret.txt"), "top secret").unwrap();

        let err = resolve_scoped_path(&root, "../secret.txt").unwrap_err();
        assert!(matches!(err, PathError::Escaped));
    }

    #[test]
    fn rejects_symlink_escaping_the_root() {
        let base = tempfile::tempdir().unwrap();
        let root = base.path().join("project");
        let outside = base.path().join("outside");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("secret.txt"), "top secret").unwrap();
        std::os::unix::fs::symlink(outside.join("secret.txt"), root.join("link")).unwrap();

        let err = resolve_scoped_path(&root, "link").unwrap_err();
        assert!(matches!(err, PathError::Escaped));
    }

    #[test]
    fn nonexistent_path_is_not_found_not_escaped() {
        let base = tempfile::tempdir().unwrap();
        let root = base.path().join("project");
        std::fs::create_dir_all(&root).unwrap();

        let err = resolve_scoped_path(&root, "nope.txt").unwrap_err();
        assert!(matches!(err, PathError::NotFound));
    }

    #[test]
    fn rejects_traversal_to_a_nonexistent_leaf_under_an_existing_outside_dir() {
        // Existence-oracle regression: canonicalizing the FULL joined path
        // before choosing Escaped-vs-NotFound means a leaf that doesn't
        // exist returns NotFound (404) even when its PARENT directory
        // plainly escapes root — so an attacker can distinguish "this file
        // exists outside my project root" (403 Escaped) from "it doesn't"
        // (404 NotFound) purely from the status code, without ever reading
        // a byte of content. The parent chain must be checked even when the
        // leaf itself fails to canonicalize.
        let base = tempfile::tempdir().unwrap();
        let root = base.path().join("project");
        let outside = base.path().join("outside");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&outside).unwrap(); // the outside DIR is real...
                                                    // ...but this specific leaf file does not exist.
        let err = resolve_scoped_path(&root, "../outside/does-not-exist.txt").unwrap_err();
        assert!(
            matches!(err, PathError::Escaped),
            "expected Escaped, got {err:?}"
        );
    }

    #[test]
    fn escape_status_never_depends_on_what_exists_outside_the_root() {
        // Existence-oracle regression, SECOND level. The first fix closed the
        // LEAF case by falling back to canonicalizing the leaf's PARENT --
        // but only ONE level up, so the very same status-code oracle survived
        // one directory higher: `../<dir>/leaf` answered 403 when `<dir>`
        // existed outside root and 404 when it did not, letting an
        // authenticated caller enumerate arbitrary DIRECTORY paths anywhere on
        // the box (proven live against the running binary: probing
        // `/home/vibe/.ssh/x` returned 403 while `/home/vibe/.no-such-dir/x`
        // returned 404).
        //
        // The invariant that actually closes it: the status may depend ONLY
        // on whether the NEAREST EXISTING ancestor is inside `root` -- never
        // on what does or does not exist outside it. So both of these must be
        // Escaped, indistinguishably.
        let base = tempfile::tempdir().unwrap();
        let root = base.path().join("project");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(base.path().join("real-dir")).unwrap();

        let outside_dir_exists = resolve_scoped_path(&root, "../real-dir/leaf.txt").unwrap_err();
        let outside_dir_missing =
            resolve_scoped_path(&root, "../no-such-dir/leaf.txt").unwrap_err();
        assert!(
            matches!(outside_dir_exists, PathError::Escaped),
            "expected Escaped, got {outside_dir_exists:?}"
        );
        assert!(
            matches!(outside_dir_missing, PathError::Escaped),
            "expected Escaped (else the status leaks that the outside dir is absent), \
             got {outside_dir_missing:?}"
        );
    }

    #[test]
    fn escape_status_is_uniform_however_deep_the_outside_chain_goes() {
        // Same oracle, several levels down -- the ancestor walk must keep
        // climbing until something actually resolves, not stop at the leaf's
        // immediate parent.
        let base = tempfile::tempdir().unwrap();
        let root = base.path().join("project");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(base.path().join("real-dir")).unwrap();

        let deep_exists = resolve_scoped_path(&root, "../real-dir/a/b/c.txt").unwrap_err();
        let deep_missing = resolve_scoped_path(&root, "../no-such-dir/a/b/c.txt").unwrap_err();
        assert!(
            matches!(deep_exists, PathError::Escaped),
            "got {deep_exists:?}"
        );
        assert!(
            matches!(deep_missing, PathError::Escaped),
            "got {deep_missing:?}"
        );
    }

    #[test]
    fn a_miss_under_a_nonexistent_subdir_inside_the_root_is_still_not_found() {
        // The other side of the invariant: climbing the ancestor chain must
        // NOT over-broaden into 403 for a path that is legitimately scoped
        // inside `root` and simply is not there. Its nearest existing
        // ancestor is `root` itself, which does not escape.
        let base = tempfile::tempdir().unwrap();
        let root = base.path().join("project");
        std::fs::create_dir_all(&root).unwrap();

        let err = resolve_scoped_path(&root, "no-such-subdir/nope.txt").unwrap_err();
        assert!(
            matches!(err, PathError::NotFound),
            "expected NotFound, got {err:?}"
        );
    }

    #[test]
    fn allows_dotdot_that_stays_within_the_root() {
        let base = tempfile::tempdir().unwrap();
        let root = base.path().join("project");
        std::fs::create_dir_all(root.join("sub")).unwrap();
        std::fs::write(root.join("file.txt"), "hi").unwrap();

        let resolved = resolve_scoped_path(&root, "sub/../file.txt").unwrap();
        assert_eq!(
            resolved,
            std::fs::canonicalize(root.join("file.txt")).unwrap()
        );
    }

    #[test]
    fn allows_a_real_subdir_literally_named_with_a_dotdot_substring() {
        // Banning the substring ".." textually (instead of resolving via
        // canonicalize) would incorrectly reject this real, legitimate
        // subdirectory name.
        let base = tempfile::tempdir().unwrap();
        let root = base.path().join("project");
        std::fs::create_dir_all(root.join("foo..bar")).unwrap();

        let resolved = resolve_scoped_path(&root, "foo..bar").unwrap();
        assert!(resolved.ends_with("foo..bar"));
    }

    #[test]
    fn read_capped_text_rejects_oversized_file_without_reading_it() {
        let base = tempfile::tempdir().unwrap();
        let path = base.path().join("big.txt");
        // Sparse-ish large file: content doesn't matter, only its length —
        // this proves the metadata().len() check runs before any read.
        let f = std::fs::File::create(&path).unwrap();
        f.set_len(MAX_FILE_READ_BYTES + 1).unwrap();

        let err = read_capped_text(&path).unwrap_err();
        assert_eq!(err.0, StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[test]
    fn read_capped_text_rejects_nul_byte_content() {
        let base = tempfile::tempdir().unwrap();
        let path = base.path().join("nul.txt");
        std::fs::write(&path, b"hello\0world").unwrap();

        let err = read_capped_text(&path).unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn read_capped_text_rejects_invalid_utf8_content() {
        let base = tempfile::tempdir().unwrap();
        let path = base.path().join("binary.dat");
        std::fs::write(&path, [0xFF, 0xFE, 0xFD, 0x80]).unwrap();

        let err = read_capped_text(&path).unwrap_err();
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn read_capped_text_returns_content_for_a_normal_text_file() {
        let base = tempfile::tempdir().unwrap();
        let path = base.path().join("ok.txt");
        std::fs::write(&path, "hello world\n").unwrap();

        let content = read_capped_text(&path).unwrap();
        assert_eq!(content, "hello world\n");
    }
}
