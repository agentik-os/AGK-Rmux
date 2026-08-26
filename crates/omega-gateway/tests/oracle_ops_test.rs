//! Integration tests for the four per-session oracle mission-ops endpoints
//! (Task B1-B4) — `crates/omega-gateway/src/routes_oracles.rs`:
//! `GET /v1/oracles/{session}/timeline`, `GET /v1/oracles/{session}/gate`,
//! `POST /v1/oracles/{session}/reap`, `POST /v1/oracles/{session}/resurrect`.
//!
//! `timeline`/`gate` are pure in-process reads gated on `$OMEGA_DIR` (never
//! shelling to the CLI); `reap`/`resurrect` are real CLI subprocess wraps,
//! fake-`OMEGA_BIN`-tested only — NEVER exercised against a real session
//! here.

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// $OMEGA_DIR and OMEGA_BIN are process-global; serialize every test in this
// binary that touches either (same pattern as dispatch_test.rs's LOCK).
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

async fn app_and_token(gateway_dir: &std::path::Path) -> (axum::Router, String) {
    let (_, token) = DeviceStore::open(gateway_dir).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.to_path_buf(),
        GatewayConfig::default(),
    ));
    (app, token)
}

fn clear_env() {
    std::env::remove_var("OMEGA_DIR");
    std::env::remove_var("OMEGA_BIN");
}

/// Writes an executable fake `omega` script. The script also appends its
/// full argv (one per line) to a capture file, so a test can prove exactly
/// what was passed to the subprocess — same idiom `dispatch_test.rs::
/// install_fake_omega` uses.
fn install_fake_omega(
    bin_dir: &std::path::Path,
    capture_file: &std::path::Path,
    script_body: &str,
) {
    use std::os::unix::fs::PermissionsExt;
    let path = bin_dir.join("omega");
    let capture = capture_file.display();
    std::fs::write(
        &path,
        format!("#!/usr/bin/env bash\nprintf '%s\\n' \"$@\" > '{capture}'\n{script_body}\n"),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_BIN", &path);
}

// ── GET /v1/oracles/{session}/timeline ──────────────────────────────────

#[tokio::test]
async fn timeline_returns_the_merged_events_for_a_real_oracle_state() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let omega_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DIR", omega_dir.path());
    let state_dir = omega_dir.path().join("state");

    let t0 = chrono::DateTime::<chrono::Utc>::from_timestamp(1_700_000_000, 0).unwrap();
    let mission = omega_core::mission::Mission::new(
        "Acme",
        "ship the feature",
        std::path::PathBuf::from("/tmp"),
    );
    let mut state = omega_core::oracle_lifecycle::OracleState::new("oracle-Acme-1", &mission);
    state.started_at = t0;
    state.register_worker(omega_core::oracle_lifecycle::WorkerEntry {
        session_name: "Acme-worker-auth".into(),
        task_id: "t1".into(),
        task_name: "auth".into(),
        attempt_id: None,
        plan_revision: None,
        files_owned: vec![],
        dispatched_at: t0 + chrono::Duration::seconds(10),
        status: omega_core::oracle_lifecycle::WorkerEntryStatus::DoneClean,
    });
    state.write(&state_dir).unwrap();

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/oracles/oracle-Acme-1/timeline"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["oracle_name"], "oracle-Acme-1");
    assert_eq!(body["project"], "Acme");
    let events = body["events"].as_array().unwrap();
    assert_eq!(
        events.len(),
        2,
        "oracle-dispatched + worker-dispatch events, got {events:?}"
    );
    assert!(events[0]["text"]
        .as_str()
        .unwrap()
        .starts_with("oracle dispatched"));
    assert!(events[1]["text"]
        .as_str()
        .unwrap()
        .contains("dispatch worker 'auth'"));
    assert!(
        events[0]["at"].as_str().unwrap().contains("2023"),
        "at must be RFC3339: {:?}",
        events[0]["at"]
    );

    clear_env();
}

#[tokio::test]
async fn timeline_404s_cleanly_for_an_unknown_oracle() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let omega_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DIR", omega_dir.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/oracles/oracle-nope-1/timeline"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("oracle-nope-1"));

    clear_env();
}

#[tokio::test]
async fn timeline_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let res = reqwest::Client::new()
        .get(format!("{base}/v1/oracles/oracle-x/timeline"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

// ── GET /v1/oracles/{session}/gate ──────────────────────────────────────

#[tokio::test]
async fn gate_returns_the_graded_result_when_one_exists() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let omega_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DIR", omega_dir.path());
    let state_dir = omega_dir.path().join("state");
    std::fs::create_dir_all(&state_dir).unwrap();

    let rubric = omega_core::gate::Rubric::new(
        "test mission",
        vec![omega_core::gate::RubricCriterion {
            id: "c1".into(),
            description: "it works".into(),
            weight: 1.0,
            category: omega_core::gate::CriterionCategory::Functional,
        }],
    );
    let grades = vec![omega_core::gate::GradeResult {
        criterion_id: "c1".into(),
        verdict: omega_core::gate::GradeVerdict::Satisfied,
        confidence: 0.9,
        evidence: "src/lib.rs:1".into(),
    }];
    let mut result = omega_core::gate::GateResult::evaluate(
        &rubric,
        grades,
        vec![],
        &omega_core::gate::FalsificationReport {
            challenges: vec![],
            minimum_required: 12,
            total_attempted: 12,
            defects_found: 0,
            uncited_rejected: 0,
            pass: true,
        },
        vec![],
        true,
        true,
        true,
    );
    result.oracle = "oracle-Acme-1".into();
    result.write(&state_dir).unwrap();

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/oracles/oracle-Acme-1/gate"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["status"], "result");
    assert_eq!(body["oracle"], "oracle-Acme-1");
    assert_eq!(body["grades"][0]["verdict"], "Satisfied");

    clear_env();
}

#[tokio::test]
async fn gate_falls_back_to_the_rubric_when_no_result_exists() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let omega_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DIR", omega_dir.path());
    let state_dir = omega_dir.path().join("state");
    std::fs::create_dir_all(&state_dir).unwrap();

    let rubric = omega_core::gate::Rubric::new(
        "test mission",
        vec![omega_core::gate::RubricCriterion {
            id: "F1".into(),
            description: "core feature".into(),
            weight: 3.0,
            category: omega_core::gate::CriterionCategory::Functional,
        }],
    );
    rubric.write(&state_dir, "oracle-Acme-1").unwrap();

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/oracles/oracle-Acme-1/gate"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["status"], "rubric_only");
    assert_eq!(body["mission"], "test mission");
    assert_eq!(body["criteria"][0]["id"], "F1");

    clear_env();
}

#[tokio::test]
async fn gate_404s_cleanly_when_neither_result_nor_rubric_exists() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let omega_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DIR", omega_dir.path());

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/oracles/oracle-nope-1/gate"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);

    clear_env();
}

#[tokio::test]
async fn gate_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let res = reqwest::Client::new()
        .get(format!("{base}/v1/oracles/oracle-x/gate"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

// ── POST /v1/oracles/{session}/reap ─────────────────────────────────────

#[tokio::test]
async fn reap_runs_omega_reap_with_exactly_the_session_argv() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo '  oracle-Acme-worker-auth: already closed — scope claim reclaimed'; exit 0",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/oracles/oracle-Acme-worker-auth/reap"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["reaped"], true);
    assert!(body["output"].as_str().unwrap().contains("already closed"));

    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    // Never bare `omega reap` (that sweeps every worker on the box) — always
    // scoped to exactly the one session named in the path, behind a "--"
    // separator (review-fix: without it, a session literally named "--"
    // would clap-parse as no positional at all and hit the bare sweep).
    assert_eq!(argv, vec!["reap", "--", "oracle-Acme-worker-auth"]);

    clear_env();
}

/// Review-fix regression (final whole-branch review, CRITICAL): a session
/// path segment starting with `-` used to reach `omega reap`'s argv with NO
/// `"--"` separator, so `POST /v1/oracles/--/reap` ran `omega reap --`,
/// which clap parses IDENTICALLY to bare `omega reap` (no positional) —
/// proven live against the real binary before this fix landed. Now rejected
/// at the validation layer, before any spawn, AND the argv itself carries a
/// `"--"` separator as a second independent layer of defense.
#[tokio::test]
async fn reap_rejects_a_dash_leading_session_before_any_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'SHOULD NEVER RUN' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    for evil in ["--", "-x", "--dry-run"] {
        let res = reqwest::Client::new()
            .post(format!("{base}/v1/oracles/{evil}/reap"))
            .bearer_auth(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 400, "session={evil}");
        assert!(
            !capture_file.exists(),
            "omega subprocess was spawned for session={evil}"
        );
    }

    clear_env();
}

#[tokio::test]
async fn reap_nonzero_exit_surfaces_as_502() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'reap ABORTED — the session daemon is unreachable' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/oracles/oracle-x/reap"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 502);
    let body: serde_json::Value = res.json().await.unwrap();
    // M-1 (Codex cross-model review, 2026-08-11; gap found during
    // whole-branch review): the raw stderr must never reach the client, only
    // a generic sanitized message; the full raw text still goes to the
    // gateway's own tracing log (not asserted here).
    assert!(body.get("stderr").is_none());
    assert!(body.get("stdout").is_none());
    assert!(!body["error"].as_str().unwrap().contains("ABORTED"));

    clear_env();
}

#[tokio::test]
async fn reap_rejects_nul_byte_session_before_any_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'SHOULD NEVER RUN' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    // A NUL byte survives URL path decoding as %00; axum decodes it into the
    // Path<String> extractor.
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/oracles/oracle-x%00y/reap"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("NUL"));
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for a NUL-containing session"
    );

    clear_env();
}

#[tokio::test]
async fn reap_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/oracles/oracle-x/reap"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

// ── POST /v1/oracles/{session}/resurrect ────────────────────────────────

#[tokio::test]
async fn resurrect_runs_omega_resurrect_with_exactly_the_oracle_argv() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "printf '\\xe2\\x97\\x86 resurrected oracle-Acme-1\\n'; exit 0",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/oracles/oracle-Acme-1/resurrect"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["resurrected"], true);
    assert!(body["output"]
        .as_str()
        .unwrap()
        .contains("resurrected oracle-Acme-1"));

    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    // "--" separator: same review-fix reasoning as reap's argv assertion above.
    assert_eq!(argv, vec!["resurrect", "--", "oracle-Acme-1"]);

    clear_env();
}

#[tokio::test]
async fn resurrect_nonzero_exit_surfaces_as_502() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'session daemon unreachable' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/oracles/oracle-x/resurrect"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 502);
    let body: serde_json::Value = res.json().await.unwrap();
    // M-1 (Codex cross-model review, 2026-08-11; gap found during
    // whole-branch review): see `reap_nonzero_exit_surfaces_as_502`'s
    // identical comment above.
    assert!(body.get("stderr").is_none());
    assert!(body.get("stdout").is_none());
    assert!(!body["error"].as_str().unwrap().contains("unreachable"));

    clear_env();
}

#[tokio::test]
async fn resurrect_rejects_empty_session_before_any_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'SHOULD NEVER RUN' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    // A session of all-whitespace, urlencoded, still reaches the handler as
    // a non-empty (but blank) string.
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/oracles/%20%20/resurrect"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for a blank session"
    );

    clear_env();
}

/// Review-fix regression (final whole-branch review, CRITICAL): same class
/// of bug as `reap_rejects_a_dash_leading_session_before_any_spawn` — a
/// dash-leading oracle name used to reach `omega resurrect`'s argv with no
/// "--" separator, which clap would parse identically to the bare
/// "resurrect every dead oracle" form.
#[tokio::test]
async fn resurrect_rejects_a_dash_leading_session_before_any_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'SHOULD NEVER RUN' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    for evil in ["--", "-x", "--help"] {
        let res = reqwest::Client::new()
            .post(format!("{base}/v1/oracles/{evil}/resurrect"))
            .bearer_auth(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 400, "session={evil}");
        assert!(
            !capture_file.exists(),
            "omega subprocess was spawned for session={evil}"
        );
    }

    clear_env();
}

#[tokio::test]
async fn resurrect_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/oracles/oracle-x/resurrect"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}
