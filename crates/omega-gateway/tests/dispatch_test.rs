//! `POST /v1/dispatch` — the one mutating endpoint in this plan.
//!
//! The most important test in this file is
//! `unknown_project_rejects_before_any_subprocess_spawn`: it proves that an
//! unrecognized project name is rejected with a 400 WITHOUT ever invoking
//! the `omega` binary — the capture file a spawned fake `omega` would have
//! written is asserted absent.

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// OMEGA_HOME and OMEGA_BIN are process-global; serialize every test in this
// binary that touches either (same pattern as oracles_test.rs's
// OMEGA_STATE_DIR/OMEGA_RMUX_BIN lock).
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

/// Writes an executable fake `omega` script and points `OMEGA_BIN` at it.
/// The script also appends its full argv (one per line, `--`-separated) to
/// a capture file under `capture_dir`, so a test can prove exactly what was
/// passed to the subprocess.
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

/// Creates a fake `$HOME` containing exactly one discoverable project (a
/// `.git`-marked directory named `name`), and points `OMEGA_HOME` at it —
/// the override `config::home_dir()` / `routes_projects.rs` /
/// `routes_dispatch.rs` all respect (Task 8's HOME-override fix).
fn install_fake_home(home_dir: &std::path::Path, project_name: &str) {
    std::fs::create_dir_all(home_dir.join(project_name).join(".git")).unwrap();
    std::env::set_var("OMEGA_HOME", home_dir);
}

async fn app_and_token(gateway_dir: &std::path::Path) -> (axum::Router, String) {
    let (_, token) = DeviceStore::open(gateway_dir).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.to_path_buf(),
        GatewayConfig::default(),
    ));
    (app, token)
}

#[tokio::test]
async fn happy_path_dispatches_and_returns_oracle_and_delivery() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        r#"printf '\xe2\x97\x86 Oracle dispatched: oracle-TestProj-1\nDISPATCH_DELIVERY=spawned\n  Mission: do the thing\n'; exit 0"#,
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/dispatch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "TestProj", "mission": "do the thing"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["oracle"], "oracle-TestProj-1");
    assert_eq!(body["delivery"], "spawned");

    // Prove no shell-string interpolation happened and no extra/missing args:
    // the recorded argv is exactly ["dispatch", "--", "TestProj", "do the
    // thing"] — flags first (none here), then the `--` separator, then the
    // two positionals (Task D / D4: clap-safe argv construction).
    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    assert_eq!(argv, vec!["dispatch", "--", "TestProj", "do the thing"]);

    std::env::remove_var("OMEGA_HOME");
    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn unknown_project_rejects_before_any_subprocess_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    // A real project exists ("TestProj"), but the request names a different,
    // random-UUID project that provably does not exist anywhere.
    install_fake_home(home_dir.path(), "TestProj");
    // Install a fake omega that would fail loudly if invoked at all, proving
    // a spawn never happens rather than merely happening to succeed.
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'SHOULD NEVER RUN' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let unknown = format!("definitely-not-a-real-project-{}", uuid_like());
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/dispatch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": unknown, "mission": "x"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains(&unknown));

    // The single most important assertion in this plan: the subprocess was
    // NEVER spawned, so the capture file the fake omega script writes on
    // every invocation must not exist.
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for an unknown project"
    );

    std::env::remove_var("OMEGA_HOME");
    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn subprocess_failure_surfaces_stderr_as_502() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'oracle registry lock held by another dispatch' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/dispatch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "TestProj", "mission": "do the thing"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 502);
    let body: serde_json::Value = res.json().await.unwrap();
    // M-1 (Codex cross-model review, 2026-08-11): raw subprocess
    // stdout/stderr is no longer echoed into the response body -- only a
    // sanitized, generic error. The full raw text still goes to the
    // gateway's own tracing log (not asserted here, out of this test's
    // reach), never the HTTP response.
    assert!(
        body.get("stderr").is_none(),
        "must not echo raw stderr: {body}"
    );
    assert!(
        body.get("stdout").is_none(),
        "must not echo raw stdout: {body}"
    );
    assert!(
        !body["error"]
            .as_str()
            .unwrap()
            .contains("oracle registry lock held"),
        "error message must not contain the raw subprocess text: {body}"
    );
    assert!(
        body.get("oracle").is_none(),
        "must never fabricate an oracle name on failure"
    );

    std::env::remove_var("OMEGA_HOME");
    std::env::remove_var("OMEGA_BIN");
}

/// M-1 (Codex cross-model review, 2026-08-11): a secret-shaped string
/// written to stdout/stderr by a failing `omega dispatch` (an
/// environment-derived credential a future CLI diagnostic could emit) must
/// never reach the HTTP response body -- only the gateway's own log.
#[tokio::test]
async fn subprocess_failure_never_leaks_a_secret_shaped_string_into_the_response() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'sk-ProjSECRETVALUE1234567890'; echo 'ANTHROPIC_API_KEY=sk-ProjSECRETVALUE1234567890' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/dispatch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "TestProj", "mission": "do the thing"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 502);
    let raw_body = res.text().await.unwrap();
    assert!(
        !raw_body.contains("sk-ProjSECRETVALUE1234567890"),
        "response body leaked the raw secret-shaped subprocess output: {raw_body}"
    );
    assert!(
        !raw_body.contains("ANTHROPIC_API_KEY"),
        "response body leaked the raw secret-shaped subprocess output: {raw_body}"
    );

    std::env::remove_var("OMEGA_HOME");
    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn empty_project_and_mission_reject_before_discovery() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/dispatch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "  ", "mission": "x"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/dispatch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "TestProj", "mission": ""}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn post_dispatch_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/dispatch"))
        .json(&serde_json::json!({"project": "x", "mission": "y"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn dash_prefixed_values_land_as_positionals_after_separator() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    // Both the project and mission values start with `-`, which would be
    // misparsed as a CLI flag by `omega`'s own clap parser without the `--`
    // separator (Task D4). The project name is a real, `.git`-marked
    // directory so it passes the discovery check unrelated to this concern.
    install_fake_home(home_dir.path(), "-weird-project");
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        r#"printf '\xe2\x97\x86 Oracle dispatched: oracle-weird-1\nDISPATCH_DELIVERY=spawned\n  Mission: -rf everything\n'; exit 0"#,
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/dispatch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "-weird-project", "mission": "-rf everything"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    assert_eq!(
        argv,
        vec!["dispatch", "--", "-weird-project", "-rf everything"]
    );

    std::env::remove_var("OMEGA_HOME");
    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn unknown_agent_rejects_with_400_before_any_subprocess_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'SHOULD NEVER RUN' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/dispatch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "TestProj", "mission": "x", "agent": "not-a-real-agent"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("not-a-real-agent"));
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for an unknown agent"
    );

    std::env::remove_var("OMEGA_HOME");
    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn known_agent_dispatches_normally() {
    let _g = LOCK.lock().await;
    // Sanity check backing this test's choice of agent name: the roster is
    // non-empty and its first entry's `name()` is what gets sent over HTTP
    // below, so this test tracks the real roster rather than a hardcoded
    // guess.
    let agents = omega_core::agents::Agent::all();
    assert!(!agents.is_empty());
    let agent_name = agents[0].name();

    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        r#"printf '\xe2\x97\x86 Oracle dispatched: oracle-TestProj-1\nDISPATCH_DELIVERY=spawned\n  Mission: do the thing\n'; exit 0"#,
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/dispatch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "TestProj", "mission": "do the thing", "agent": agent_name}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    assert_eq!(
        argv,
        vec![
            "dispatch",
            "--agent",
            agent_name,
            "--",
            "TestProj",
            "do the thing"
        ]
    );

    std::env::remove_var("OMEGA_HOME");
    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn mission_with_nul_byte_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'SHOULD NEVER RUN' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/dispatch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "TestProj", "mission": "do the\u{0000}thing"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("NUL"));
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for a NUL-containing mission"
    );

    std::env::remove_var("OMEGA_HOME");
    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn mission_over_length_cap_rejects_with_400_no_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_home(home_dir.path(), "TestProj");
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'SHOULD NEVER RUN' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    // One byte over routes_dispatch.rs's MAX_MISSION_LEN (8000).
    let too_long = "x".repeat(8001);
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/dispatch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "TestProj", "mission": too_long}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("too long"));
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for an over-length mission"
    );

    std::env::remove_var("OMEGA_HOME");
    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn concurrency_cap_returns_429_when_dispatch_permits_exhausted() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_home(home_dir.path(), "TestProj");
    // Sleeps briefly before responding, so the test can reliably observe
    // MAX_CONCURRENT_DISPATCHES in-flight requests before firing the one
    // that should be rejected.
    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        r#"sleep 0.15; printf '\xe2\x97\x86 Oracle dispatched: oracle-TestProj-1\nDISPATCH_DELIVERY=spawned\n  Mission: slow\n'; exit 0"#,
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    // Must match server.rs's MAX_CONCURRENT_DISPATCHES.
    const MAX_CONCURRENT_DISPATCHES: usize = 4;

    let mut in_flight = Vec::new();
    for i in 0..MAX_CONCURRENT_DISPATCHES {
        let client = client.clone();
        let base = base.clone();
        let token = token.clone();
        in_flight.push(tokio::spawn(async move {
            client
                .post(format!("{base}/v1/dispatch"))
                .bearer_auth(&token)
                .json(
                    &serde_json::json!({"project": "TestProj", "mission": format!("mission {i}")}),
                )
                .send()
                .await
                .unwrap()
                .status()
                .as_u16()
        }));
    }

    // Give the in-flight requests time to acquire their permits and start
    // blocking on the (slow, sleeping) fake subprocess before firing the
    // one that should be rejected. Generous window to avoid flakiness.
    tokio::time::sleep(std::time::Duration::from_millis(60)).await;

    let busy_res = client
        .post(format!("{base}/v1/dispatch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "TestProj", "mission": "one too many"}))
        .send()
        .await
        .unwrap();
    assert_eq!(busy_res.status(), 429);
    let body: serde_json::Value = busy_res.json().await.unwrap();
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("too many concurrent dispatches"));

    for task in in_flight {
        let status = task.await.unwrap();
        assert_eq!(status, 200);
    }

    // Every permit has now been released — a fresh request succeeds again.
    let after_res = client
        .post(format!("{base}/v1/dispatch"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "TestProj", "mission": "after release"}))
        .send()
        .await
        .unwrap();
    assert_eq!(after_res.status(), 200);

    std::env::remove_var("OMEGA_HOME");
    std::env::remove_var("OMEGA_BIN");
}

/// A small dependency-free stand-in for a UUID: unique enough within one
/// test run that it provably won't collide with any real project name.
fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{nanos:x}")
}
