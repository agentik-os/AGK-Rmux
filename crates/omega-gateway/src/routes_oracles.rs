//! `GET /v1/oracles` — the live oracle roster: every top-level mission
//! ledger (`missions::list()`) composed with whether its session currently
//! appears in `rmux::list_sessions()`. Read-only, no gateway state written.
//!
//! NAMING NOTE (see `protocol::OracleEntry`): `Mission.key` is populated
//! verbatim from the ledger JSON's `"oracle"` field, and that field already
//! IS the full rmux session name (e.g. `"oracle-dentistrygpt"`) —
//! `missions.rs` never strips or re-derives a bare identifier from the
//! filename. So `key` and `session` below are the same string; there is no
//! `format!("oracle-{}", ...)` prefixing to apply (that would double the
//! prefix into `"oracle-oracle-dentistrygpt"`).
//!
//! Also four per-session oracle mission-ops endpoints (Task B):
//!
//! - `GET /v1/oracles/{session}/timeline` ([`timeline`]) and
//!   `GET /v1/oracles/{session}/gate` ([`gate`]) are PURE IN-PROCESS
//!   synchronous reads — `omega_core::timeline::build` and
//!   `omega_core::gate::{GateResult, Rubric}::read` are the exact same
//!   file-reading functions `omega timeline`/`omega gate` call, invoked
//!   here directly (via `spawn_blocking`) rather than shelled out to and
//!   text-parsed — simpler and far more robust, and documented as the
//!   deliberate choice in `.superpowers/sdd/progress.md`'s Task B
//!   ground-truth section. `state_dir` is resolved via
//!   `omega_core::config::OmegaConfig::load().state_dir` (honors
//!   `$OMEGA_DIR`, confirmed in `omega-core/src/config.rs`) — the exact
//!   same directory the CLI itself reads, so a hermetic test overrides
//!   `$OMEGA_DIR`.
//! - `POST /v1/oracles/{session}/reap` ([`reap`]) and
//!   `POST /v1/oracles/{session}/resurrect` ([`resurrect`]) are REAL CLI
//!   subprocess wraps — both mutate real state (scope claims, worktrees,
//!   session spawns), so they go through `omega_cli::run`, argv-only,
//!   ALWAYS with the path's session name as the one positional target
//!   (`omega reap <session>` / `omega resurrect <oracle>`, never the bare
//!   form — a bare `omega reap` sweeps EVERY worker on the box, which is
//!   not what a per-session endpoint should do). Fake-`OMEGA_BIN`-tested
//!   only; NEVER exercised against a real session in this crate's tests or
//!   in live-verify.

use crate::protocol::{
    GateAdversarialChallengeEntry, GateAuditResultEntry, GateConsensusVoteEntry, GateGradeEntry,
    GateResultResponse, GateStatusResponse, OracleEntry, OraclesResponse, ReapResponse,
    ResurrectResponse, RubricCriterionEntry, RubricResponse, TimelineEventEntry, TimelineResponse,
};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::Json;
use serde_json::json;

type ApiError = (StatusCode, Json<serde_json::Value>);

pub async fn list() -> Json<OraclesResponse> {
    let missions = tokio::task::spawn_blocking(crate::missions::list)
        .await
        .unwrap_or_default();
    // Never 500 on a degraded rmux read (same posture as routes_sessions::list):
    // a failed liveness probe just means every entry reports live: false.
    let live_sessions = match tokio::task::spawn_blocking(crate::rmux::list_sessions).await {
        Ok(Ok(names)) => names,
        _ => vec![],
    };

    let oracles = missions
        .into_iter()
        .map(|m| {
            let session = m.key.clone();
            let live = live_sessions.contains(&session);
            OracleEntry {
                key: m.key.clone(),
                session,
                live,
                mission: Some(m),
            }
        })
        .collect();
    Json(OraclesResponse { oracles })
}

fn not_found(msg: impl Into<String>) -> ApiError {
    (StatusCode::NOT_FOUND, Json(json!({ "error": msg.into() })))
}

fn bad_gateway(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::BAD_GATEWAY,
        Json(json!({ "error": msg.into() })),
    )
}

fn bad_request(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": msg.into() })),
    )
}

fn gateway_timeout(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::GATEWAY_TIMEOUT,
        Json(json!({ "error": msg.into() })),
    )
}

fn timeline_to_response(tl: omega_core::timeline::OracleTimeline) -> TimelineResponse {
    TimelineResponse {
        oracle_name: tl.oracle_name,
        project: tl.project,
        mission: tl.mission,
        phase: tl.phase,
        events: tl
            .events
            .into_iter()
            .map(|e| TimelineEventEntry {
                at: e.at.to_rfc3339(),
                marker: e.marker.to_string(),
                text: e.text,
            })
            .collect(),
    }
}

/// `GET /v1/oracles/{session}/timeline` — see this module's doc comment.
/// 404 (never a fabricated empty timeline) when `omega_core::timeline::build`
/// returns `None` — no `OracleState` on disk for `session`.
pub async fn timeline(Path(session): Path<String>) -> Result<Json<TimelineResponse>, ApiError> {
    let oracle_name = session.clone();
    let built = tokio::task::spawn_blocking(move || {
        let config = omega_core::config::OmegaConfig::load().unwrap_or_default();
        omega_core::timeline::build(&config.state_dir, &oracle_name)
    })
    .await
    .map_err(|e| bad_gateway(format!("timeline task panicked: {e}")))?
    .map_err(|e| bad_gateway(format!("failed to read timeline: {e}")))?;

    match built {
        Some(tl) => Ok(Json(timeline_to_response(tl))),
        None => Err(not_found(format!(
            "no timeline for oracle '{session}' (no OracleState on disk)"
        ))),
    }
}

fn rubric_to_response(r: omega_core::gate::Rubric) -> RubricResponse {
    RubricResponse {
        mission: r.mission,
        criteria: r
            .criteria
            .into_iter()
            .map(|c| RubricCriterionEntry {
                id: c.id,
                description: c.description,
                weight: c.weight,
                category: format!("{:?}", c.category),
            })
            .collect(),
        created_at: r.created_at.to_rfc3339(),
    }
}

fn gate_result_to_response(r: omega_core::gate::GateResult) -> GateResultResponse {
    GateResultResponse {
        oracle: r.oracle,
        timestamp: r.timestamp.to_rfc3339(),
        rubric_pass: r.rubric_pass,
        consensus_pass: r.consensus_pass,
        adversarial_pass: r.adversarial_pass,
        regression_pass: r.regression_pass,
        audit_results: r
            .audit_results
            .into_iter()
            .map(|a| GateAuditResultEntry {
                audit_id: a.audit_id,
                raw_score: a.raw_score,
                max_score: a.max_score,
                normalized_score: a.normalized_score,
                confidence: format!("{:?}", a.confidence),
                verdict: format!("{:?}", a.verdict),
                findings_count: a.findings_count,
                critical_findings: a.critical_findings,
                worker_session: a.worker_session,
                completed_at: a.completed_at.to_rfc3339(),
            })
            .collect(),
        audit_pass: r.audit_pass,
        token_budget_pass: r.token_budget_pass,
        citation_pass: r.citation_pass,
        overall_pass: r.overall_pass,
        score: r.score,
        grades: r
            .details
            .grades
            .into_iter()
            .map(|g| GateGradeEntry {
                criterion_id: g.criterion_id,
                verdict: format!("{:?}", g.verdict),
                confidence: g.confidence,
                evidence: g.evidence,
            })
            .collect(),
        consensus_votes: r
            .details
            .consensus_votes
            .into_iter()
            .map(|v| GateConsensusVoteEntry {
                grader: v.grader,
                verdict: format!("{:?}", v.verdict),
                confidence: v.confidence,
                reasoning: v.reasoning,
            })
            .collect(),
        adversarial_challenges: r
            .details
            .adversarial_challenges
            .into_iter()
            .map(|c| GateAdversarialChallengeEntry {
                challenge: c.challenge,
                result: format!("{:?}", c.result),
                evidence: c.evidence,
            })
            .collect(),
        accepted_by: r.accepted_by,
        accepted_evidence: r.accepted_evidence,
    }
}

/// `GET /v1/oracles/{session}/gate` — see this module's doc comment. Mirrors
/// `cmd_gate`'s own read-only fallback: a graded `GateResult` wins, else the
/// `Rubric` alone, else a clean 404 — never fabricated. Never calls
/// `--accept`/`--mission`/`--approver`/`--evidence` (all state-mutating).
pub async fn gate(Path(session): Path<String>) -> Result<Json<GateStatusResponse>, ApiError> {
    let oracle_name = session.clone();
    let (result_read, rubric_read) = tokio::task::spawn_blocking(move || {
        let config = omega_core::config::OmegaConfig::load().unwrap_or_default();
        (
            omega_core::gate::GateResult::read(&config.state_dir, &oracle_name),
            omega_core::gate::Rubric::read(&config.state_dir, &oracle_name),
        )
    })
    .await
    .map_err(|e| bad_gateway(format!("gate task panicked: {e}")))?;

    let result =
        result_read.map_err(|e| bad_gateway(format!("failed to read gate result: {e}")))?;
    if let Some(r) = result {
        return Ok(Json(GateStatusResponse::Result(gate_result_to_response(r))));
    }
    let rubric = rubric_read.map_err(|e| bad_gateway(format!("failed to read rubric: {e}")))?;
    if let Some(r) = rubric {
        return Ok(Json(GateStatusResponse::RubricOnly(rubric_to_response(r))));
    }
    Err(not_found(format!(
        "no gate result or rubric for oracle '{session}'"
    )))
}

/// Shared by [`reap`] and [`resurrect`]: a session/oracle name must be
/// non-empty and NUL-free before it ever reaches a subprocess argv — same
/// defensive posture `routes_dispatch.rs::create` uses for `project`/
/// `mission`.
///
/// REVIEW-FIX (final whole-branch review, CRITICAL): also rejects a name
/// that STARTS WITH `-`. This is belt-and-braces on top of the real fix in
/// [`reap`]/[`resurrect`] (a `"--"` argv separator before the positional,
/// the exact idiom `routes_dispatch.rs` and `routes_orchestrate.rs` already
/// use) — without EITHER fix, a session path segment of `"--"` reaches
/// clap as `reap --` / `resurrect --`, which clap parses as "no positional
/// given" (proven live against the real binary: `omega workers` and `omega
/// workers --` render identically, while `omega workers <anything-else>`
/// clearly takes a positional). `omega reap` with `name: None` sweeps EVERY
/// live Worker session on the box; `omega resurrect` with `oracle: None`
/// resurrects every dead oracle — the exact "never the bare form" behavior
/// this module's own doc comment says never happens. Kept as a second,
/// independent layer (not the only fix) because a value of exactly `"-"`
/// or `"-x"` a caller did not intend as a flag should still be rejected
/// outright rather than silently reaching the CLI as a positional that
/// looks like an unknown flag to a human reading logs.
fn validate_session_name(name: &str) -> Result<(), ApiError> {
    if name.trim().is_empty() {
        return Err(bad_request("session must not be empty"));
    }
    if name.contains('\0') {
        return Err(bad_request("session must not contain a NUL byte"));
    }
    if name.starts_with('-') {
        return Err(bad_request("session must not start with '-'"));
    }
    Ok(())
}

/// `POST /v1/oracles/{session}/reap` — runs `omega reap -- <session>` (never
/// bare). The `"--"` separator is REQUIRED, not cosmetic — see
/// [`validate_session_name`]'s doc comment for the live-proven clap
/// behavior it closes. A non-zero exit is a REAL error here (502, with
/// stdout/stderr) — unlike `omega doctor`, `omega reap` either does its job
/// or it doesn't, there is no "expected non-zero" outcome to special-case.
/// On success, `output` is the raw stdout rather than a hand-parsed
/// structure: the CLI's own per-session lines (`"already closed"` / `"WOULD
/// be reaped"` / `"no done signal — still working, left alone"`) are
/// loosely-structured operator text, and a brittle parser over it would
/// break the moment the CLI's wording changes — see `cmd_reap`,
/// `crates/omega-cli/src/main.rs` ~line 5481.
pub async fn reap(Path(session): Path<String>) -> Result<Json<ReapResponse>, ApiError> {
    validate_session_name(&session)?;
    let target = session.clone();
    // I-2 (Codex cross-model review, 2026-08-11): see
    // routes_sessions.rs::create's identical comment.
    let output = tokio::task::spawn_blocking(move || {
        crate::omega_cli::run_with_timeout(
            &["reap", "--", target.as_str()],
            crate::omega_cli::cli_timeout(),
        )
    })
    .await
    .map_err(|e| bad_gateway(format!("reap task panicked: {e}")))?
    .map_err(|e| {
        if crate::omega_cli::is_timeout(&e) {
            gateway_timeout(e.to_string())
        } else {
            bad_gateway(format!("failed to spawn omega: {e}"))
        }
    })?;

    // M-1 (Codex cross-model review, 2026-08-11; gap found during
    // whole-branch review, same vulnerability class as the review's 5
    // originally-listed sites): the raw stdout/stderr used to be echoed
    // straight into the HTTP response, which can leak environment-derived
    // secrets or other sensitive text a future CLI diagnostic writes. The
    // FULL raw text still goes to the gateway's own tracing log; the client
    // only ever sees a generic, sanitized message.
    if !output.success {
        tracing::error!(
            session = %session,
            stdout = %output.stdout,
            stderr = %output.stderr,
            "omega reap failed"
        );
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": "omega reap failed (see gateway logs)" })),
        ));
    }
    Ok(Json(ReapResponse {
        reaped: true,
        output: output.stdout,
    }))
}

/// `POST /v1/oracles/{session}/resurrect` — runs `omega resurrect --
/// <oracle>` (never bare, for the same per-session reasoning and the same
/// REQUIRED `"--"` separator as [`reap`]). `output` carries the real
/// per-oracle line (`"resurrected"` / `"already alive"` / `"already
/// finished"` / `"no OracleState"` — see `cmd_resurrect`,
/// `crates/omega-cli/src/main.rs` ~line 7271) rather than a hand-parsed
/// structure, for the same reason [`reap`] documents.
pub async fn resurrect(Path(session): Path<String>) -> Result<Json<ResurrectResponse>, ApiError> {
    validate_session_name(&session)?;
    let target = session.clone();
    // I-2 (Codex cross-model review, 2026-08-11): see
    // routes_sessions.rs::create's identical comment.
    let output = tokio::task::spawn_blocking(move || {
        crate::omega_cli::run_with_timeout(
            &["resurrect", "--", target.as_str()],
            crate::omega_cli::cli_timeout(),
        )
    })
    .await
    .map_err(|e| bad_gateway(format!("resurrect task panicked: {e}")))?
    .map_err(|e| {
        if crate::omega_cli::is_timeout(&e) {
            gateway_timeout(e.to_string())
        } else {
            bad_gateway(format!("failed to spawn omega: {e}"))
        }
    })?;

    // M-1 (Codex cross-model review, 2026-08-11; gap found during
    // whole-branch review): see `reap`'s identical comment above.
    if !output.success {
        tracing::error!(
            session = %session,
            stdout = %output.stdout,
            stderr = %output.stderr,
            "omega resurrect failed"
        );
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": "omega resurrect failed (see gateway logs)" })),
        ));
    }
    Ok(Json(ResurrectResponse {
        resurrected: true,
        output: output.stdout,
    }))
}
