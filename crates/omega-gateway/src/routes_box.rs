//! Box health: `GET /v1/doctor`, `GET /v1/usage`, `GET /v1/box-info`,
//! `POST /v1/backup` (wave6 task C).
//!
//! `doctor` and `backup` shell out to the real `omega` CLI binary via
//! `omega_cli::run` (there is no in-process `doctor_checks()` to call —
//! it lives in the `omega-cli` BINARY crate, not a library this crate can
//! import); `usage` is a pure in-process read of
//! `omega_core::monitor::UsageSnapshot`; `box_info` mixes a shelled-out
//! `hostname` lookup, a shelled-out `omega --version`, and this crate's own
//! `CARGO_PKG_VERSION` + process uptime.

use crate::fsperm::{harden_dir, harden_file};
use crate::protocol::{
    BackupResponse, BoxIdResponse, BoxInfoResponse, DoctorCheckEntry, DoctorResponse, UsageResponse,
};
use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use std::path::{Path, PathBuf};

fn internal_err(msg: String) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": msg })),
    )
}

fn bad_gateway(msg: String) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_GATEWAY,
        Json(serde_json::json!({ "error": msg })),
    )
}

// ── GET /v1/doctor ───────────────────────────────────────────────────────

/// Parses `omega doctor`'s (bare, no flags) rendered stdout into a
/// [`DoctorResponse`]. Returns `None` only when the output is fundamentally
/// unparseable (missing the `"OmegaOS doctor"` header entirely) — a
/// non-zero exit code alone is NEVER a reason to fail here, since the real
/// CLI calls `std::process::exit(1)` on overall `Fail` and every check line
/// has already been printed before that exit.
///
/// A CHECK LINE is identified unambiguously as one starting with EXACTLY
/// two literal spaces followed by `[` (the CLI's own
/// `println!("  {} {:16} {}", ...)` format) — this is what distinguishes it
/// from the trailing, zero-indent summary line (`"[+] all systems
/// healthy"` and friends), which starts directly with `[` and is
/// deliberately NOT parsed: `overall` is aggregated from the parsed checks
/// themselves (mirroring `omega_core::doctor::overall()`'s own
/// fail-else-warn-else-ok logic) rather than re-matched out of that second
/// piece of text, which is strictly more robust.
///
/// Deliberately does NOT attempt to split each check's `name` from its
/// `detail`: the CLI's `{:16}` name column is a MINIMUM width, not a
/// delimiter — e.g. "binary provenance" is 18 characters, so for that line
/// there is only ONE literal space between the name and the detail, not a
/// fixed column boundary. `text` therefore carries everything after the
/// glyph, trimmed, as one string.
fn parse_doctor_output(stdout: &str) -> Option<DoctorResponse> {
    if !stdout.contains("OmegaOS doctor") {
        return None;
    }

    let mut checks = Vec::new();
    for line in stdout.lines() {
        // strip_prefix requires the line to start with EXACTLY two literal
        // spaces -- a line indented by 3+ spaces (none in real output, but
        // this stays defensive) fails this test at the THIRD char (a space,
        // not '['), and the zero-indent trailing summary line fails it
        // immediately. Both are correctly skipped.
        let Some(after_two_spaces) = line.strip_prefix("  ") else {
            continue;
        };
        if after_two_spaces.len() < 3 || !after_two_spaces.starts_with('[') {
            continue;
        }
        // `.get(0..3)` (not `&after_two_spaces[0..3]`) because this stdout is
        // an adversarial subprocess's output, not a trusted format: a
        // multi-byte UTF-8 character sitting where the glyph is expected
        // (e.g. a check line reading "  [€] ...") would put byte offset 3
        // in the MIDDLE of that character, and a raw slice panics ("byte
        // index 3 is not a char boundary") -- `.get` returns `None` on a
        // non-boundary range instead, so the line is just skipped like any
        // other unrecognized glyph.
        let Some(glyph) = after_two_spaces.get(0..3) else {
            continue;
        };
        let health = match glyph {
            "[+]" => "ok",
            "[!]" => "warn",
            "[x]" => "fail",
            _ => continue,
        };
        let text = after_two_spaces[3..].trim().to_string();
        checks.push(DoctorCheckEntry {
            health: health.to_string(),
            text,
        });
    }

    let overall = if checks.iter().any(|c| c.health == "fail") {
        "fail"
    } else if checks.iter().any(|c| c.health == "warn") {
        "warn"
    } else {
        "ok"
    };

    Some(DoctorResponse {
        overall: overall.to_string(),
        checks,
    })
}

/// `GET /v1/doctor` — runs BARE `omega doctor` (never `--fix`, which
/// mutates state, nor `--deep`, which burns live API quota) and parses its
/// stdout. 502 only when the output is fundamentally unparseable (missing
/// header) — never for a non-zero exit code alone.
pub async fn doctor() -> Result<Json<DoctorResponse>, (StatusCode, Json<serde_json::Value>)> {
    let output = tokio::task::spawn_blocking(|| crate::omega_cli::run(&["doctor"]))
        .await
        .map_err(|e| internal_err(format!("doctor task panicked: {e}")))?
        .map_err(|e| bad_gateway(format!("failed to run omega doctor: {e}")))?;

    parse_doctor_output(&output.stdout)
        .map(Json)
        .ok_or_else(|| bad_gateway("unparseable omega doctor output: missing header".to_string()))
}

// ── GET /v1/usage ────────────────────────────────────────────────────────

/// `GET /v1/usage` — a pure in-process read of `~/.omega/state/usage.json`
/// via `omega_core::monitor::UsageSnapshot::read()`, off the async runtime
/// thread via `spawn_blocking` (same idiom `routes_rules::list` /
/// `routes_projects::list` use for their own pure calls). `available:
/// false` covers BOTH the normal "cache file doesn't exist yet" case
/// (`Ok(None)`) and a malformed cache file (`Err`, e.g. corrupt JSON) — a
/// monitoring endpoint should never 500 the caller over a stale/corrupt
/// cache, so both collapse to the same "no data" response shape.
pub async fn usage() -> Json<UsageResponse> {
    let snap = tokio::task::spawn_blocking(omega_core::monitor::UsageSnapshot::read)
        .await
        .unwrap_or(Ok(None))
        .unwrap_or(None);

    Json(match snap {
        Some(s) => UsageResponse {
            available: true,
            session_pct: Some(s.session_pct),
            week_pct: Some(s.week_pct),
            sonnet_pct: Some(s.sonnet_pct),
            extra_pct: Some(s.extra_pct),
            tokens_5h: Some(s.tokens_5h),
            tokens_7d: Some(s.tokens_7d),
            account: Some(s.account),
            ts: Some(s.ts),
        },
        None => UsageResponse {
            available: false,
            session_pct: None,
            week_pct: None,
            sonnet_pct: None,
            extra_pct: None,
            tokens_5h: None,
            tokens_7d: None,
            account: None,
            ts: None,
        },
    })
}

// ── GET /v1/box-info ─────────────────────────────────────────────────────

/// Shells out to the `hostname` binary (no `hostname` crate in this
/// workspace's `Cargo.toml` — same "shell out to a small trusted local
/// binary" convention `routes_agents::kill_process_group` uses for `kill`).
/// Returns `"unknown"` rather than erroring the whole endpoint if the
/// binary is missing or fails.
fn read_hostname() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

/// `GET /v1/box-info` — this box's identity (hostname, `omega --version`)
/// plus the gateway process's own liveness (its crate version, and seconds
/// elapsed since `AppState` was constructed — see `AppState::started_at`).
pub async fn box_info(State(state): State<AppState>) -> Json<BoxInfoResponse> {
    let hostname = tokio::task::spawn_blocking(read_hostname)
        .await
        .unwrap_or_else(|_| "unknown".to_string());

    let omega_version = tokio::task::spawn_blocking(|| crate::omega_cli::run(&["--version"]))
        .await
        .ok()
        .and_then(|r| r.ok())
        .map(|o| o.stdout.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    Json(BoxInfoResponse {
        hostname,
        omega_version,
        gateway_version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: state.started_at.elapsed().as_secs(),
    })
}

// ── GET /v1/box-id ───────────────────────────────────────────────────────

fn box_id_path(dir: &Path) -> PathBuf {
    dir.join("box_id.txt")
}

/// A valid persisted box id is EXACTLY what `random_hex(16)` can produce:
/// 32 lowercase hex chars. Anything else (empty, torn write, manual edit,
/// disk corruption, stray whitespace aside) is treated as absent and
/// regenerated -- an endpoint response must never echo back malformed
/// content just because *something* was on disk.
fn is_valid_box_id(s: &str) -> bool {
    s.len() == 32
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// REVIEW-FIX: the first pass raced on TWO different axes -- concurrent
/// callers when the file was absent (guarded by `create_new`) and,
/// separately, concurrent callers when the file existed but was empty (NOT
/// guarded at all: every racing caller took the direct-overwrite branch,
/// computed its OWN candidate, and each unconditionally re-wrote the file
/// with a different id, so two callers could receive two different ids from
/// one HTTP round-trip -- the exact "registers as two different boxes"
/// failure the whole point of a stable box id exists to prevent). A
/// process-wide mutex serializing the ENTIRE read-or-create sequence closes
/// both races in one stroke and is simpler than reasoning about two
/// independent atomic-file-op protocols: this endpoint is called rarely
/// (once per pairing, occasionally thereafter), so a coarse lock costs
/// nothing worth optimizing away. Global rather than per-`dir` because nothing
/// in this crate needs finer granularity and a global lock is trivially
/// correct; the only cost is serializing box-id reads/creates across
/// unrelated `gateway_dir`s within the SAME process (irrelevant in
/// production -- one process serves one dir -- and merely serializes,
/// rather than breaks, the handful of tests that use different tempdirs).
static BOX_ID_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Reads this box's stable id from `box_id.txt`, generating and persisting a
/// fresh 32-hex-char id (`random_hex(16)`) on first access if the file is
/// absent or its content isn't a valid id ([`is_valid_box_id`]).
///
/// The write itself is write-temp-then-rename, not a direct write to the
/// final path: `harden_file` runs on the TEMP path before the rename, so
/// `box_id.txt` is either absent or ALREADY 0600 at every instant it is
/// observable at its final name -- no window where a concurrent reader (a
/// different process, or a curious `ls` from another local user) could see
/// it at default (umask) permissions before it gets hardened.
pub fn read_or_create_box_id(dir: &Path) -> std::io::Result<String> {
    std::fs::create_dir_all(dir)?;
    harden_dir(dir);
    let path = box_id_path(dir);

    let _guard = BOX_ID_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if let Ok(existing) = std::fs::read_to_string(&path) {
        let trimmed = existing.trim();
        if is_valid_box_id(trimmed) {
            return Ok(trimmed.to_string());
        }
    }

    let candidate = crate::util::random_hex(16); // 16 bytes -> 32 hex chars
    let tmp_path = dir.join(format!("box_id.txt.tmp-{}", crate::util::random_hex(4)));
    std::fs::write(&tmp_path, &candidate)?;
    harden_file(&tmp_path);
    std::fs::rename(&tmp_path, &path)?;
    Ok(candidate)
}

/// `GET /v1/box-id` — this box's stable, non-secret identifier, generating
/// and persisting it on first call if absent (anywhere-access plan §5.2).
/// Device-token-guarded like every other route below (registered above
/// `require_device`'s `route_layer` in `server.rs`).
pub async fn box_id(
    State(state): State<AppState>,
) -> Result<Json<BoxIdResponse>, (StatusCode, Json<serde_json::Value>)> {
    let dir = state.dir.clone();
    let id = tokio::task::spawn_blocking(move || read_or_create_box_id(&dir))
        .await
        .map_err(|e| internal_err(format!("box-id task panicked: {e}")))?
        .map_err(|e| internal_err(format!("failed to read/create box_id.txt: {e}")))?;
    Ok(Json(BoxIdResponse { box_id: id }))
}

// ── POST /v1/backup ──────────────────────────────────────────────────────

/// Server-controlled backup output DIRECTORY (NEVER client-controlled — see
/// this module's + `BackupResponse`'s doc comments for why). `OMEGA_BACKUP_DIR`
/// env override, else `std::env::temp_dir()`, mirroring this crate's other
/// `OMEGA_*_DIR` overrides in `config.rs` (`OMEGA_GATEWAY_DIR`,
/// `OMEGA_DEPOSIT_DIR`).
fn backup_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OMEGA_BACKUP_DIR") {
        return PathBuf::from(dir);
    }
    std::env::temp_dir()
}

/// The exact literal prefixes `omega backup`'s own stdout uses (see
/// `crates/omega-cli/src/main.rs::cmd_backup`): `"archive"`/`"size"`/
/// `"included"` are hand-padded to an 8-char column before the colon
/// (`"archive :"`, `"size    :"`, `"included:"`), so these strings must
/// match byte-for-byte or the parse silently misses the line.
const ARCHIVE_PREFIX: &str = "archive :";
const SIZE_PREFIX: &str = "size    :";

/// Parses `omega backup`'s stdout for the archive path + human-readable
/// size. `fallback_path` (the `--out` path this endpoint itself chose and
/// passed) is used for `path` only in the near-impossible case the
/// `"archive :"` line is missing from an otherwise-successful run —
/// `size` stays `None` if its line is missing, never treated as an error,
/// since `path` alone is still a usable result.
fn parse_backup_output(stdout: &str, fallback_path: &str) -> BackupResponse {
    let mut path = None;
    let mut size = None;
    for line in stdout.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix(ARCHIVE_PREFIX) {
            path = Some(rest.trim().to_string());
        } else if let Some(rest) = trimmed.strip_prefix(SIZE_PREFIX) {
            size = Some(rest.trim().to_string());
        }
    }
    BackupResponse {
        path: path.unwrap_or_else(|| fallback_path.to_string()),
        size,
    }
}

/// `POST /v1/backup` — runs `omega backup --out <server-chosen-path>`. The
/// ONLY mutating endpoint this task adds (writes one new archive file under
/// [`backup_dir`]; never deletes or mutates anything else) — still
/// auth-gated like every other route, still registered above
/// `require_device`'s `route_layer`. Takes NO client input: the request
/// body is empty, and the output path is ALWAYS server-chosen (never a
/// client-supplied `--out`), so an authenticated caller can never direct an
/// archive write to an arbitrary path the gateway process can reach.
///
/// Unlike `doctor`, a non-zero exit here IS a real error (502) — there is
/// no "expected non-zero exit" semantic for a backup the way there is for a
/// health check that legitimately finds problems.
pub async fn backup() -> Result<Json<BackupResponse>, (StatusCode, Json<serde_json::Value>)> {
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S%.f").to_string();
    let path = backup_dir().join(format!("omega-gateway-backup-{ts}.tgz"));
    let path_str = path.to_string_lossy().to_string();
    let fallback = path_str.clone();

    let output =
        tokio::task::spawn_blocking(move || crate::omega_cli::run(&["backup", "--out", &path_str]))
            .await
            .map_err(|e| internal_err(format!("backup task panicked: {e}")))?
            .map_err(|e| bad_gateway(format!("failed to run omega backup: {e}")))?;

    if !output.success {
        return Err(bad_gateway(format!(
            "omega backup exited non-zero: {}",
            output.stderr
        )));
    }

    Ok(Json(parse_backup_output(&output.stdout, &fallback)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_doctor_output_none_when_header_missing() {
        assert!(parse_doctor_output("nothing useful here\n").is_none());
    }

    #[test]
    fn parse_doctor_output_skips_the_zero_indent_summary_line() {
        let resp = parse_doctor_output(
            "OmegaOS doctor\n\n  [+] daemon running\n\n[+] all systems healthy\n",
        )
        .unwrap();
        assert_eq!(resp.checks.len(), 1);
        assert_eq!(resp.overall, "ok");
    }

    #[test]
    fn parse_backup_output_extracts_path_and_size() {
        let resp = parse_backup_output(
            "OmegaOS backup\n\n  archive : /tmp/x.tgz\n  size    : 1.2 MB\n  included: home\n",
            "/fallback.tgz",
        );
        assert_eq!(resp.path, "/tmp/x.tgz");
        assert_eq!(resp.size.as_deref(), Some("1.2 MB"));
    }

    #[test]
    fn parse_backup_output_falls_back_when_archive_line_missing() {
        let resp = parse_backup_output("OmegaOS backup\n\n  included: home\n", "/fallback.tgz");
        assert_eq!(resp.path, "/fallback.tgz");
        assert_eq!(resp.size, None);
    }

    /// Adversarial: a check line whose glyph position holds a multi-byte
    /// UTF-8 character (e.g. `€`, 3 bytes) instead of the expected
    /// single-byte ASCII glyph. `after_two_spaces[0..3]` would byte-slice
    /// into the MIDDLE of that character and panic ("byte index 3 is not a
    /// char boundary") -- this must instead be skipped like any other
    /// unrecognized glyph, never crash the request.
    #[test]
    fn parse_doctor_output_skips_multibyte_glyph_without_panicking() {
        let resp =
            parse_doctor_output("OmegaOS doctor\n\n  [\u{20ac}] weird glyph\n\n[+] ok\n").unwrap();
        assert_eq!(
            resp.checks.len(),
            0,
            "the multibyte-glyph line is unrecognized and skipped"
        );
        assert_eq!(resp.overall, "ok");
    }
}
