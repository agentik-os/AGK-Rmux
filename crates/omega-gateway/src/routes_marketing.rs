//! `GET /v1/marketing` — the marketing-enabled project list, from
//! `omega_core::marketing::list_marketing_projects()`. The filesystem-scan
//! half of that function IS pure/synchronous/side-effect-free (a project is
//! "marketing-enabled" when it has a `<path>/marketing/` directory), but the
//! function as a whole is NOT side-effect-free: it also calls
//! `read_crontab()` (`crates/omega-core/src/marketing.rs`), which spawns a
//! REAL `crontab -l` SUBPROCESS on every single call, with NO timeout —
//! unlike `project_accounts()` in the same file, which is deliberately
//! bounded to a 4s timeout for its own subprocess call. A prior version of
//! this doc comment claimed "no daemon, no network" / "no CLI subprocess",
//! which was false (an independent adversarial review reproduced it live: a
//! fake `crontab` binary placed earlier on `PATH` was invoked by a single
//! `GET /v1/marketing` and influenced the response's `engine_on` field). A
//! hung or adversarially slow `crontab` on `PATH` can pin the
//! `spawn_blocking` thread this handler runs on indefinitely — this crate's
//! shared blocking-thread pool is used by other endpoints too
//! (`routes_sessions`, `routes_projects`, `routes_skills`), so an unbounded
//! subprocess here is a real, if narrow, resource-starvation risk.
//!
//! This endpoint does NOT fix the underlying unbounded subprocess in
//! `omega-core::marketing::read_crontab()` — that function is shared with
//! the TUI and other CLI callers, and changing its behavior/signature is out
//! of scope for a gateway-only fix. Instead, the ENTIRE `spawn_blocking`
//! call is wrapped in [`marketing_list_timeout`] via `tokio::time::timeout`
//! (mirroring `routes_pdf.rs`'s `run_omega_pdf` idiom for its own
//! subprocess-timeout bound): on timeout this handler returns `504 Gateway
//! Timeout` rather than hanging the HTTP request forever. The `crontab -l`
//! subprocess itself is NOT killed by this timeout (`std::process::Command`
//! here is the crate's synchronous blocking call, not a killable
//! `tokio::process::Command` — there is no cheap way to cancel it from
//! outside `spawn_blocking`), so a hung `crontab` still leaks one blocking
//! thread from the pool even after this endpoint has already answered; this
//! bounds the CALLER's wait, not the child process's lifetime.
//!
//! Read-only, list-only: `accounts` (connected social-account count) is
//! NEVER populated by this endpoint. Populating it needs
//! `omega_core::marketing::project_accounts`, which shells out to
//! `omega-zernio` per project and is bounded by its own 4s timeout — fine
//! for an on-demand detail lookup, wrong for a list endpoint that would
//! then shell out once per discovered project on every call. So `accounts`
//! (and `accounts_tried`) always mirror the list-only values the source
//! function itself never populates (`None` / `false`).
//!
//! `slug` is derived from the on-disk directory basename, while dedup in
//! `list_marketing_projects()` keys on the (possibly different) registry
//! `name` — so a registry entry whose display `name` differs from its
//! directory name can survive dedup and appear as two source entries
//! sharing one `slug`. This handler dedupes the RESPONSE by canonicalized
//! `path` (the one field guaranteed unique per real on-disk project) before
//! mapping into the wire type, which never carries `path` — see
//! [`dedupe_by_canonical_path`].

use crate::protocol::{MarketingProjectEntry, MarketingResponse};
use axum::http::StatusCode;
use axum::Json;
use std::time::Duration;

/// Hard ceiling on how long the blocking marketing scan (filesystem walk +
/// one `crontab -l` subprocess) may run before this endpoint gives up and
/// reports a 504. Generous by default (the scan is normally fast — local
/// filesystem only, one small subprocess — so this is a safety net against
/// a hung `crontab` on `PATH`, not a tight budget); overridable via
/// `OMEGA_MARKETING_LIST_TIMEOUT_MS` so a test can prove the bound actually
/// fires without waiting out the real default.
fn marketing_list_timeout() -> Duration {
    if let Ok(ms) = std::env::var("OMEGA_MARKETING_LIST_TIMEOUT_MS") {
        if let Ok(ms) = ms.trim().parse::<u64>() {
            return Duration::from_millis(ms);
        }
    }
    Duration::from_secs(10)
}

/// Dedupe by canonicalized filesystem path, keeping the FIRST-SEEN entry per
/// path (the input is already sorted by name, so "first-seen" is
/// deterministic and stable across calls). `path` is never on the wire, but
/// it is the one field `list_marketing_projects()` guarantees unique per
/// real on-disk project — `slug` (derived from the directory basename) and
/// `name` (the registry display name, which can legitimately differ from
/// the directory name) are both NOT guaranteed unique, and deduping upstream
/// only keys on lowercased `name`. A path that fails to canonicalize (e.g.
/// it no longer exists) falls back to the raw path string so it still
/// participates in the dedup rather than silently bypassing it.
fn dedupe_by_canonical_path(
    projects: Vec<omega_core::marketing::MarketingProject>,
) -> Vec<omega_core::marketing::MarketingProject> {
    let mut seen = std::collections::HashSet::new();
    projects
        .into_iter()
        .filter(|p| {
            let key = p
                .path
                .canonicalize()
                .map(|c| c.to_string_lossy().into_owned())
                .unwrap_or_else(|_| p.path.to_string_lossy().into_owned());
            seen.insert(key)
        })
        .collect()
}

pub async fn list() -> Result<Json<MarketingResponse>, (StatusCode, Json<serde_json::Value>)> {
    let timeout = marketing_list_timeout();
    let scan = tokio::time::timeout(
        timeout,
        tokio::task::spawn_blocking(|| {
            let projects =
                dedupe_by_canonical_path(omega_core::marketing::list_marketing_projects())
                    .into_iter()
                    .map(|p| MarketingProjectEntry {
                        name: p.name,
                        slug: p.slug,
                        has_content: p.has_content,
                        calendar_posts: p.calendar_posts,
                        engine_on: p.engine_on,
                        accounts: p.accounts,
                        accounts_tried: p.accounts_tried,
                        has_context: p.has_context,
                        has_strategy: p.has_strategy,
                        has_copy: p.has_copy,
                        has_visual: p.has_visual,
                        has_branding: p.has_branding,
                    })
                    .collect();
            MarketingResponse { projects }
        }),
    )
    .await;

    match scan {
        Ok(Ok(response)) => Ok(Json(response)),
        Ok(Err(e)) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("marketing scan task panicked: {e}") })),
        )),
        Err(_) => Err((
            StatusCode::GATEWAY_TIMEOUT,
            Json(serde_json::json!({
                "error": format!(
                    "marketing scan timed out after {}ms (a crontab -l subprocess may be hung)",
                    timeout.as_millis()
                )
            })),
        )),
    }
}
