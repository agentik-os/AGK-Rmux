//! `POST /v1/team` — `crates/omega-gateway/src/routes_team.rs::create`.
//! Wraps `omega team [OPTIONS] <PROJECT> [MEMBERS]...`.

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// OMEGA_BIN and HOME are process-global; serialize every test in this binary
// that touches either (same pattern as sessions_create_test.rs's LOCK).
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

/// Writes an executable fake `omega` script that also appends its full argv
/// (one per line) to `capture_file` — same idiom `dispatch_test.rs::
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

async fn app_and_token(gateway_dir: &std::path::Path) -> (axum::Router, String) {
    let (_, token) = DeviceStore::open(gateway_dir).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.to_path_buf(),
        GatewayConfig::default(),
    ));
    (app, token)
}

fn clear_env() {
    std::env::remove_var("OMEGA_BIN");
    std::env::remove_var("HOME");
}

#[tokio::test]
async fn happy_path_with_members_builds_exact_argv() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "printf '\\xe2\\x97\\x86 Team spawned: Team-Acme\\n'; exit 0",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "project": "Acme",
            "count": 2,
            "members": ["alice:build the API", "bob:build the UI"]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["session"], "Team-Acme");
    assert!(body["output"].as_str().unwrap().contains("Team spawned"));

    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    assert_eq!(
        argv,
        vec![
            "team",
            "--count",
            "2",
            "--",
            "Acme",
            "alice:build the API",
            "bob:build the UI"
        ]
    );

    clear_env();
}

#[tokio::test]
async fn happy_path_with_dir_and_no_members_builds_exact_argv() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");
    let fake_home = tempfile::tempdir().unwrap();

    std::env::set_var("HOME", fake_home.path());
    install_fake_omega(bin_dir.path(), &capture_file, "exit 0");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let dir_str = fake_home.path().display().to_string();
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "Acme", "dir": dir_str }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["session"], "Team-Acme");

    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    // Finding 5 (adversarial review round): `--dir` is now a single
    // `=`-joined argv element, never two separate elements.
    let dir_flag = format!("--dir={dir_str}");
    assert_eq!(argv, vec!["team", dir_flag.as_str(), "--", "Acme"]);

    clear_env();
}

#[tokio::test]
async fn only_count_no_members_still_forwards_count_with_no_member_args() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_omega(bin_dir.path(), &capture_file, "exit 0");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "Acme", "count": 5 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let recorded = std::fs::read_to_string(&capture_file).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    assert_eq!(argv, vec!["team", "--count", "5", "--", "Acme"]);

    clear_env();
}

#[tokio::test]
async fn project_failing_slug_check_rejects_with_400_no_spawn() {
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

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "-leading-dash" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for an invalid project name"
    );

    clear_env();
}

#[tokio::test]
async fn count_zero_rejects_with_400_no_spawn() {
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

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "Acme", "count": 0 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for count=0"
    );

    clear_env();
}

#[tokio::test]
async fn count_nine_rejects_with_400_no_spawn() {
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

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "Acme", "count": 9 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for count=9"
    );

    clear_env();
}

#[tokio::test]
async fn count_one_and_eight_are_both_accepted() {
    let _g = LOCK.lock().await;
    for count in [1, 8] {
        let gateway_dir = tempfile::tempdir().unwrap();
        let bin_dir = tempfile::tempdir().unwrap();
        let capture_dir = tempfile::tempdir().unwrap();
        let capture_file = capture_dir.path().join("argv.txt");

        install_fake_omega(bin_dir.path(), &capture_file, "exit 0");

        let (app, token) = app_and_token(gateway_dir.path()).await;
        let base = spawn(app).await;

        let res = reqwest::Client::new()
            .post(format!("{base}/v1/team"))
            .bearer_auth(&token)
            .json(&serde_json::json!({ "project": "Acme", "count": count }))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 200, "count={count} should be accepted");
        assert!(capture_file.exists());
    }

    clear_env();
}

#[tokio::test]
async fn oversized_member_rejects_with_400_no_spawn() {
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

    let too_long = format!("alice:{}", "x".repeat(500));
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "Acme", "members": [too_long] }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("too long"));
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for an oversized member"
    );

    clear_env();
}

/// Finding 4 (adversarial review round): `cmd_team` builds the real
/// spawned session name literally as `format!("Team-{project}")`, which
/// then goes through `omega_core::session::sanitize_session_name`
/// (truncates at 48 chars). A `project` long enough that `project` ALONE
/// fits under `valid_new_session_name`'s 100-byte cap, but `"Team-" +
/// project` exceeds the real 48-char session-name cap, must be a
/// structural 400 -- otherwise the echoed `session` field would diverge
/// from the real (truncated) rmux session name.
#[tokio::test]
async fn project_whose_team_prefixed_name_sanitize_would_truncate_rejects_with_400_no_spawn() {
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

    // 50 chars: fits `valid_new_session_name`'s 100-byte cap on its own,
    // but "Team-" (5) + 50 = 55 > MAX_SESSION_NAME_LEN (48).
    let long_project = "a".repeat(50);
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": long_project }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for a project sanitize would truncate"
    );

    clear_env();
}

/// Finding 3 (adversarial review round): `members` had no length cap at
/// all -- the reviewer reproduced 200,000 accepted members in one request
/// (which would spawn 200,000 rmux panes end-to-end, since `cmd_team`
/// ignores `count` whenever `members` is non-empty). Reuses `MAX_COUNT`
/// (8): one team member per requested pane is the same underlying concept
/// as `count`.
#[tokio::test]
async fn nine_members_rejects_with_400_no_spawn() {
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

    let members: Vec<String> = (0..9).map(|i| format!("member{i}:do the thing")).collect();
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "Acme", "members": members }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("too many members"));
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for 9 members"
    );

    clear_env();
}

/// Companion to [`nine_members_rejects_with_400_no_spawn`]: exactly 8
/// (the same bound `count` already uses) is still accepted and reaches the
/// real argv.
#[tokio::test]
async fn eight_members_is_accepted() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_omega(bin_dir.path(), &capture_file, "exit 0");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let members: Vec<String> = (0..8).map(|i| format!("member{i}:do the thing")).collect();
    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "Acme", "members": members }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    assert!(capture_file.exists());

    clear_env();
}

#[tokio::test]
async fn member_with_nul_byte_rejects_with_400_no_spawn() {
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

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "Acme", "members": ["alice:has\u{0000}nul"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert!(
        !capture_file.exists(),
        "omega subprocess was spawned for a NUL-containing member"
    );

    clear_env();
}

#[tokio::test]
async fn nonzero_exit_surfaces_stdout_and_stderr_as_502() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'partial output'; echo 'rmux daemon unreachable' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "Acme" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 502);
    let body: serde_json::Value = res.json().await.unwrap();
    // M-1 (Codex cross-model review, 2026-08-11): raw subprocess
    // stdout/stderr is no longer echoed into the response body -- only a
    // sanitized, generic error. The full raw text still goes to the
    // gateway's own tracing log, never the HTTP response.
    assert!(
        body.get("stdout").is_none(),
        "must not echo raw stdout: {body}"
    );
    assert!(
        body.get("stderr").is_none(),
        "must not echo raw stderr: {body}"
    );
    assert!(
        !body["error"]
            .as_str()
            .unwrap()
            .contains("rmux daemon unreachable"),
        "error message must not contain the raw subprocess text: {body}"
    );
    assert!(
        body.get("session").is_none(),
        "must never fabricate a session on failure"
    );

    clear_env();
}

/// M-1 (Codex cross-model review, 2026-08-11): a secret-shaped string
/// written to stdout/stderr by a failing `omega team` must never reach the
/// HTTP response body -- only the gateway's own log.
#[tokio::test]
async fn nonzero_exit_never_leaks_a_secret_shaped_string_into_the_response() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_omega(
        bin_dir.path(),
        &capture_file,
        "echo 'sk-ProjSECRETVALUE1234567890'; echo 'ANTHROPIC_API_KEY=sk-ProjSECRETVALUE1234567890' >&2; exit 1",
    );

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "Acme" }))
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

    clear_env();
}

/// Finding 2 (adversarial review round): `POST /v1/team` now shares
/// `AppState::session_spawn_permits` with `POST /v1/sessions` (server.rs)
/// -- exhausted permits get a 429, never an unbounded pile of concurrent
/// `omega team` subprocesses (each of which can itself spawn up to
/// `MAX_COUNT` sub-panes). Same idiom `pdf_test.rs`'s
/// `concurrency_cap_returns_429_when_pdf_permits_exhausted` uses.
#[tokio::test]
async fn concurrency_cap_returns_429_when_session_spawn_permits_exhausted() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture_dir = tempfile::tempdir().unwrap();
    let capture_file = capture_dir.path().join("argv.txt");

    install_fake_omega(bin_dir.path(), &capture_file, "sleep 0.15\nexit 0");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    // Must match server.rs's MAX_CONCURRENT_SESSION_SPAWNS.
    const MAX_CONCURRENT_SESSION_SPAWNS: usize = 4;

    let mut in_flight = Vec::new();
    for i in 0..MAX_CONCURRENT_SESSION_SPAWNS {
        let client = client.clone();
        let base = base.clone();
        let token = token.clone();
        in_flight.push(tokio::spawn(async move {
            client
                .post(format!("{base}/v1/team"))
                .bearer_auth(&token)
                .json(&serde_json::json!({ "project": format!("Acme{i}") }))
                .send()
                .await
                .unwrap()
                .status()
                .as_u16()
        }));
    }

    tokio::time::sleep(std::time::Duration::from_millis(60)).await;

    let busy_res = client
        .post(format!("{base}/v1/team"))
        .bearer_auth(&token)
        .json(&serde_json::json!({ "project": "AcmeBusy" }))
        .send()
        .await
        .unwrap();
    assert_eq!(busy_res.status(), 429);
    let body: serde_json::Value = busy_res.json().await.unwrap();
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("too many concurrent"));

    for task in in_flight {
        let status = task.await.unwrap();
        assert_eq!(status, 200);
    }

    clear_env();
}

#[tokio::test]
async fn post_team_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/team"))
        .json(&serde_json::json!({ "project": "Acme" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}
