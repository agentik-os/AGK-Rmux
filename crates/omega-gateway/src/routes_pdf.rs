//! PDF generation: `POST /v1/pdf` and `GET /v1/pdf/download?path=` — wave7
//! task E.
//!
//! `omega pdf --template=<t> --data=<path> --out=<path>` has no in-process
//! library entry point in this workspace (pdfgen is its own tool under
//! `tools/pdfgen/`, invoked as a subprocess by the CLI itself — see
//! `crates/omega-cli/src/main.rs::cmd_pdf`), so `POST /v1/pdf` shells out,
//! argv-only.
//!
//! `template` is validated against the literal known set
//! (`whitepaper`/`audit`/`marketing`/`doc`) BEFORE any spawn. `data` is
//! client-supplied JSON: written to a SERVER-CHOSEN scratch path under
//! [`pdf_data_dir`] (never a client-supplied path passed to `--data`),
//! mirroring `routes_box::backup`'s server-chosen-`--out` posture — an
//! authenticated caller can never direct a write to an arbitrary path the
//! gateway process can reach. `--out` is likewise always server-chosen
//! (under [`pdf_output_dir`]), never client-supplied.
//!
//! `GET /v1/pdf/download?path=` is scoped to [`pdf_output_dir`] ONLY: the
//! query value is reduced to its bare file NAME (`Path::file_name()`)
//! before ever touching the filesystem, then resolved through
//! `routes_files::resolve_scoped_path` — the SAME canonicalize-and-prefix-
//! check idiom `routes_files.rs` already carries, including its
//! ancestor-walk fix for the outside-root-403-vs-404 existence-oracle leak
//! wave6 found. Reducing to `file_name()` first means even a client that
//! echoes back an absolute path (or attempts `../../etc/passwd`) only ever
//! contributes a single path COMPONENT — traversal is structurally
//! impossible here, not merely rejected after the fact. `data` scratch
//! files live under the SEPARATE [`pdf_data_dir`] (never inside
//! `pdf_output_dir`), so the download endpoint's scope can never reach a
//! generation request's raw input JSON. It ALSO only ever serves a name
//! matching this endpoint's OWN generated shape (`omega-report-*.pdf`) —
//! see [`is_server_generated_pdf_name`].
//!
//! `--send`/`--caption` are NEVER passed by this endpoint (documented in
//! `.superpowers/sdd/progress.md`'s Task E ground-truth section): pushing
//! to the operator's real Telegram from an API call with no operator-side
//! confirmation would be exactly the kind of surprising, hard-to-reverse
//! side effect this crate's other mutating endpoints (e.g.
//! `routes_oracles::reap`/`resurrect`) go out of their way to avoid.
//!
//! REVIEW-FIX ROUND (an independent adversarial reviewer found these before
//! the branch shipped — see `.superpowers/sdd/progress.md`'s Task C+D+E
//! review-fix section):
//! 1. [`pdf_root_dir`]'s default used to live under `std::env::temp_dir()`
//!    (world-writable sticky `/tmp` on this box) — a local unprivileged
//!    attacker could pre-plant `/tmp/omega-gateway-pdf/output` as a symlink
//!    to e.g. `~/.ssh` before this endpoint's first `create_dir_all` call,
//!    and `GET /v1/pdf/download` would then serve real secrets (the
//!    traversal guard canonicalizes and prefix-checks correctly, but that
//!    proves nothing when the ROOT itself has been swapped out from under
//!    it). The default now lives under the operator's own `~/.omega/state`
//!    — the same trust boundary every other `~/.omega` file already
//!    carries, never a shared world-writable directory a co-tenant process
//!    can pre-plant a symlink into.
//! 2. `POST /v1/pdf` had no concurrency cap and no subprocess timeout,
//!    while `AppState` already carries four semaphore pools for exactly
//!    this shape of problem. Fixed with `AppState::pdf_permits` (429 on
//!    exhaustion, mirroring `dispatch_permits`) and a real, killable
//!    subprocess timeout (`tokio::process::Command` with
//!    `.kill_on_drop(true)`, wrapped in `tokio::time::timeout` — see
//!    [`run_omega_pdf`]) rather than the crate's usual blocking
//!    `omega_cli::run` (which cannot be cancelled once spawned).
//! 3. `GET /v1/pdf/download` read an unbounded file into memory. Fixed with
//!    [`MAX_PDF_DOWNLOAD_BYTES`], checked via `metadata().len()` before any
//!    read — the same discipline `routes_files::MAX_FILE_READ_BYTES`
//!    already carries.

use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use crate::protocol::PdfRequest;
use crate::server::AppState;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn bad_request(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": msg.into() })),
    )
}

fn not_found(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": msg.into() })),
    )
}

fn forbidden(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": msg.into() })),
    )
}

fn bad_gateway(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::BAD_GATEWAY,
        Json(serde_json::json!({ "error": msg.into() })),
    )
}

fn too_many_requests(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(serde_json::json!({ "error": msg.into() })),
    )
}

fn internal(msg: impl std::fmt::Display) -> ApiError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": msg.to_string() })),
    )
}

/// The literal `--template` values `omega pdf` accepts (`crates/omega-cli/
/// src/main.rs::Commands::Pdf` doc comment: "whitepaper, audit, marketing,
/// doc"). Validated BEFORE any subprocess spawn. The CLI itself does NOT
/// validate this value (`template: String`, no clap `value_parser`) — this
/// allowlist is the ONLY guard between a client-supplied string and
/// pdfgen's own template resolution, so it is deliberately kept in sync by
/// hand rather than re-derived from the CLI (which has nothing to derive
/// from).
const KNOWN_TEMPLATES: &[&str] = &["whitepaper", "audit", "marketing", "doc"];

/// Hard ceiling on how long `omega pdf`'s subprocess may run before this
/// endpoint kills it and reports a 502. Generous enough to cover a
/// cold-cache `npm install` (see `cmd_pdf`) over a normal network, small
/// enough that a hung or adversarially slow child can't pin a permit (and a
/// blocking OS thread) forever — see this module's review-fix doc comment,
/// finding 2.
const PDF_SUBPROCESS_TIMEOUT: Duration = Duration::from_secs(300);

/// `PDF_SUBPROCESS_TIMEOUT_SECS` env-var override for
/// [`PDF_SUBPROCESS_TIMEOUT`] — same env-var-overridable-constant-fn shape
/// `routes_duo.rs::duo_timeout()` establishes for a per-endpoint subprocess
/// timeout in this crate (added so a test can exercise the timeout path
/// without a real 300s wait).
fn pdf_timeout() -> Duration {
    std::env::var("PDF_SUBPROCESS_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(PDF_SUBPROCESS_TIMEOUT)
}

/// Cap on `GET /v1/pdf/download`'s read — mirrors
/// `routes_files::MAX_FILE_READ_BYTES`'s discipline (checked via
/// `metadata().len()` BEFORE any read). 64 MiB comfortably covers any real
/// generated report while bounding a single request's memory footprint.
const MAX_PDF_DOWNLOAD_BYTES: u64 = 64 * 1024 * 1024;

/// Root scratch directory for this endpoint's own files — `OMEGA_PDF_DIR`
/// env override, else `~/.omega/state/gateway-pdf`. Deliberately NOT
/// `std::env::temp_dir()` (see this module's review-fix doc comment,
/// finding 1): every other per-purpose OmegaOS directory in this crate
/// that a client-controlled read can later reach (`gateway_dir`,
/// `deposit_home_dir`) already lives under the operator's own `~/.omega`,
/// which is the right trust boundary here too — the shared world-writable
/// `/tmp` was the one directory in this crate an endpoint actually READS
/// back from by a predictable path, and that combination is what a
/// pre-planted symlink can exploit.
fn pdf_root_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OMEGA_PDF_DIR") {
        return PathBuf::from(dir);
    }
    dirs::home_dir()
        .expect("no home dir")
        .join(".omega")
        .join("state")
        .join("gateway-pdf")
}

/// Where generated PDFs land — the ONLY directory `GET /v1/pdf/download`
/// can ever serve from. Kept SEPARATE from [`pdf_data_dir`] so a download
/// request can never reach a request's raw input JSON.
fn pdf_output_dir() -> PathBuf {
    pdf_root_dir().join("output")
}

/// Where this endpoint writes the client's `data` JSON before invoking
/// `omega pdf --data=<this>`. Never exposed to `GET /v1/pdf/download`.
fn pdf_data_dir() -> PathBuf {
    pdf_root_dir().join("data")
}

/// True for exactly the filename shape [`create`] itself generates
/// (`omega-report-<ts>-<8 hex chars>.pdf`). `GET /v1/pdf/download` refuses
/// anything else, even though it is already scoped to [`pdf_output_dir`] —
/// defense in depth: this directory should only ever hold files this
/// endpoint wrote, so any other name present is itself a signal something
/// is wrong, not a legitimate download target.
fn is_server_generated_pdf_name(name: &str) -> bool {
    name.starts_with("omega-report-") && name.ends_with(".pdf")
}

/// Sends `SIGKILL` to the WHOLE process group rooted at `pid` — the same
/// negative-PID `kill -- -<pid>` idiom as `routes_duo.rs::
/// kill_process_group` / `routes_agents.rs::kill_process_group`. `pid` must
/// be the PID of a process spawned with `process_group(0)` (see
/// [`run_omega_pdf`]), so this reaches both it and every nested child it
/// forked that did not itself call `setpgid` — in particular the nested
/// Node/npm process `omega pdf` invokes (I-3/pdf-half, Codex cross-model
/// review, 2026-08-11: the old cleanup only ever reached the DIRECT `omega`
/// child via `kill_on_drop(true)`, which does not propagate to a nested
/// child on POSIX).
async fn kill_process_group(pid: u32) {
    let _ = tokio::task::spawn_blocking(move || {
        std::process::Command::new("kill")
            .arg("--")
            .arg(format!("-{pid}"))
            .status()
    })
    .await;
}

/// Runs `omega pdf <args>` as a killable, time-bounded async child —
/// deliberately NOT the crate's usual `omega_cli::run` (a blocking
/// `std::process::Command::output()` with no cancellation path once
/// spawned). `.kill_on_drop(true)` means a timeout that drops the
/// in-flight `wait_with_output()` future terminates the DIRECT child
/// process rather than leaking it — but `omega pdf` invokes Node/npm
/// tooling that can spawn a NESTED process, which `kill_on_drop` alone
/// never reaches (see this module's review-fix doc comment, finding 2, and
/// [`kill_process_group`]'s doc comment). `process_group(0)` places the
/// spawned `omega` into its own process group so a disconnect/timeout can
/// kill the WHOLE tree at once, the same idiom `routes_duo.rs::
/// run_omega_duo` and every WS-stream route in this crate already use.
async fn run_omega_pdf(args: &[&str]) -> Result<crate::omega_cli::CommandOutput, ApiError> {
    let bin = crate::omega_cli::omega_bin();
    let child = tokio::process::Command::new(&bin)
        .args(args)
        .kill_on_drop(true)
        .process_group(0)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| bad_gateway(format!("failed to spawn omega pdf: {e}")))?;
    // Captured now, before `wait_with_output` (which takes `self` by value)
    // ever consumes `child` — `Child::id()` returns `None` once the child
    // has been polled to completion.
    let child_pid = child.id();

    match tokio::time::timeout(pdf_timeout(), child.wait_with_output()).await {
        Ok(Ok(out)) => Ok(crate::omega_cli::CommandOutput {
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            success: out.status.success(),
        }),
        Ok(Err(e)) => Err(bad_gateway(format!("omega pdf process error: {e}"))),
        Err(_) => {
            // Past the ceiling: the timed-out future owned `child`, so
            // dropping it here (already dropped when the future was)
            // triggers `kill_on_drop` for the DIRECT child only — that
            // alone does not reach a nested Node/npm process, so also send
            // an explicit whole-group kill using the PID captured above
            // (see [`kill_process_group`]'s doc comment). Awaited so the
            // response is only returned once the kill syscall has actually
            // been issued.
            if let Some(pid) = child_pid {
                kill_process_group(pid).await;
            }
            Err(bad_gateway(format!(
                "omega pdf timed out after {}s and was killed",
                pdf_timeout().as_secs()
            )))
        }
    }
}

/// `POST /v1/pdf` — validates `template`, acquires a
/// [`AppState::pdf_permits`] permit (429 on exhaustion), writes `data` to a
/// server-chosen scratch file, runs `omega pdf --template=<t>
/// --data=<scratch> --out=<scratch>` under a hard timeout, and returns the
/// generated path + size. A non-zero exit is a real error (502) — there is
/// no "expected non-zero" outcome for PDF generation the way there is for
/// `omega doctor`.
pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<PdfRequest>,
) -> Result<Json<crate::protocol::PdfResponse>, ApiError> {
    if !KNOWN_TEMPLATES.contains(&req.template.as_str()) {
        return Err(bad_request(format!(
            "unknown template '{}' (expected one of: {})",
            req.template,
            KNOWN_TEMPLATES.join(", ")
        )));
    }

    let Ok(_permit) = state.pdf_permits.clone().try_acquire_owned() else {
        return Err(too_many_requests(
            "too many concurrent PDF generations, try again shortly",
        ));
    };

    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S%.f").to_string();
    // A random suffix (not just the millisecond timestamp) rules out any
    // filename collision between two requests landing in the same tick —
    // review-fix finding: chrono's `%.f` prints nothing at all when the
    // sub-second field is exactly zero, which would otherwise degrade to
    // second resolution on a small fraction of requests.
    let suffix = crate::util::random_hex(4);
    let data_dir = pdf_data_dir();
    let out_dir = pdf_output_dir();
    let data_path = data_dir.join(format!("data-{ts}-{suffix}.json"));
    let out_path = out_dir.join(format!("omega-report-{ts}-{suffix}.pdf"));
    let data_path_str = data_path.to_string_lossy().to_string();
    let out_path_str = out_path.to_string_lossy().to_string();
    let data = req.data.clone();

    // Directory setup + writing the data file are plain sync fs calls —
    // kept in spawn_blocking (this crate's convention for sync fs work off
    // the async runtime thread); the subprocess itself runs via
    // `run_omega_pdf` below (async, killable, timed).
    tokio::task::spawn_blocking({
        let data_dir = data_dir.clone();
        let out_dir = out_dir.clone();
        let data_path = data_path.clone();
        move || -> Result<(), ApiError> {
            std::fs::create_dir_all(&data_dir)
                .map_err(|e| internal(format!("mkdir {}: {e}", data_dir.display())))?;
            std::fs::create_dir_all(&out_dir)
                .map_err(|e| internal(format!("mkdir {}: {e}", out_dir.display())))?;
            let json = serde_json::to_string_pretty(&data)
                .map_err(|e| internal(format!("serialize data: {e}")))?;
            std::fs::write(&data_path, json)
                .map_err(|e| internal(format!("write data file: {e}")))?;
            Ok(())
        }
    })
    .await
    .map_err(|e| internal(format!("pdf setup task panicked: {e}")))??;

    let output = run_omega_pdf(&[
        "pdf",
        "--template",
        &req.template,
        "--data",
        &data_path_str,
        "--out",
        &out_path_str,
    ])
    .await?;

    // M-1 (Codex cross-model review, 2026-08-11): the raw stdout/stderr used
    // to be echoed straight into the HTTP response, which can leak
    // environment-derived secrets or other sensitive text `omega pdf`'s
    // underlying Node/npm tooling writes on failure. The FULL raw text
    // still goes to the gateway's own tracing log; the client only ever
    // sees a generic, sanitized message.
    if !output.success {
        tracing::error!(
            template = %req.template,
            stdout = %output.stdout,
            stderr = %output.stderr,
            "omega pdf exited non-zero"
        );
        return Err(bad_gateway("omega pdf failed (see gateway logs)"));
    }

    let size_bytes = tokio::fs::metadata(&out_path)
        .await
        .map(|m| m.len())
        .map_err(|e| {
            bad_gateway(format!(
                "omega pdf reported success but no file at {out_path_str}: {e}"
            ))
        })?;

    Ok(Json(crate::protocol::PdfResponse {
        path: out_path_str,
        size_bytes,
    }))
}

/// `GET /v1/pdf/download?path=` — see this module's doc comment. Streams
/// the raw PDF bytes with `Content-Type: application/pdf`.
pub async fn download(Query(query): Query<HashMap<String, String>>) -> Result<Response, ApiError> {
    let raw = query.get("path").cloned().unwrap_or_default();
    if raw.trim().is_empty() {
        return Err(bad_request("path is required"));
    }
    // Reduce to the bare file name FIRST -- see this module's doc comment
    // for why this makes traversal structurally impossible rather than
    // merely rejected.
    let Some(name) = std::path::Path::new(&raw)
        .file_name()
        .and_then(|n| n.to_str())
    else {
        return Err(bad_request("path must contain a valid file name"));
    };
    if !is_server_generated_pdf_name(name) {
        return Err(not_found("no such generated pdf"));
    }
    let name = name.to_string();

    let root = pdf_output_dir();
    let bytes = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, ApiError> {
        let resolved = crate::routes_files::resolve_scoped_path(&root, &name).map_err(|e| match e {
            crate::routes_files::PathError::Invalid(msg) => bad_request(msg),
            crate::routes_files::PathError::Escaped => forbidden("path escapes the pdf output directory"),
            crate::routes_files::PathError::NotFound => not_found("no such generated pdf"),
        })?;
        let meta = std::fs::metadata(&resolved).map_err(|e| internal(format!("stat failed: {e}")))?;
        if !meta.is_file() {
            return Err(bad_request("path is not a file"));
        }
        if meta.len() > MAX_PDF_DOWNLOAD_BYTES {
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                Json(serde_json::json!({
                    "error": format!("file too large ({} bytes, max {})", meta.len(), MAX_PDF_DOWNLOAD_BYTES)
                })),
            ));
        }
        std::fs::read(&resolved).map_err(|e| internal(format!("read failed: {e}")))
    })
    .await
    .map_err(|e| internal(format!("download task panicked: {e}")))??;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (header::CONTENT_DISPOSITION, "attachment".to_string()),
        ],
        bytes,
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    // pdf_root_dir_honors_env_override and
    // pdf_root_dir_default_lives_under_omega_state_never_world_writable_tmp
    // both mutate the process-global OMEGA_PDF_DIR env var; without a lock
    // they race across cargo's parallel test threads (review-fix: reproduced
    // live, ~1-in-8 flake rate — the crate's established convention is a
    // guarding mutex for exactly this, see omega_cli.rs's own LOCK).
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn pdf_root_dir_honors_env_override() {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("OMEGA_PDF_DIR", "/tmp/omega-pdf-test-root");
        assert_eq!(pdf_root_dir(), PathBuf::from("/tmp/omega-pdf-test-root"));
        assert_eq!(
            pdf_output_dir(),
            PathBuf::from("/tmp/omega-pdf-test-root/output")
        );
        assert_eq!(
            pdf_data_dir(),
            PathBuf::from("/tmp/omega-pdf-test-root/data")
        );
        std::env::remove_var("OMEGA_PDF_DIR");
    }

    #[test]
    fn pdf_root_dir_default_lives_under_omega_state_never_world_writable_tmp() {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("OMEGA_PDF_DIR");
        let dir = pdf_root_dir();
        assert!(
            dir.ends_with(".omega/state/gateway-pdf"),
            "expected a path under ~/.omega/state, got {}",
            dir.display()
        );
    }

    #[test]
    fn known_templates_match_the_cli() {
        for t in ["whitepaper", "audit", "marketing", "doc"] {
            assert!(KNOWN_TEMPLATES.contains(&t));
        }
        assert!(!KNOWN_TEMPLATES.contains(&"evil"));
    }

    #[test]
    fn is_server_generated_pdf_name_accepts_only_the_real_shape() {
        assert!(is_server_generated_pdf_name(
            "omega-report-20260810-abcd1234.pdf"
        ));
        assert!(!is_server_generated_pdf_name("passwd"));
        assert!(!is_server_generated_pdf_name("data-20260810-abcd1234.json"));
        assert!(!is_server_generated_pdf_name("omega-report-evil.pdf.txt"));
        assert!(!is_server_generated_pdf_name(".pdf"));
    }
}
