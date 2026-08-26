//! Integration tests for `GET /v1/new-project/stream` —
//! `crates/omega-gateway/src/routes_new_project.rs`.
//!
//! SAFETY (R-SEC / this task's explicit brief): every test here points
//! `OMEGA_BIN` at a FAKE script. `omega new-project` spawns an actual Codex
//! session running a real `/omega-new-project ...` bootstrap pipeline —
//! nothing in this file ever invokes the real binary.

use futures_util::StreamExt;
use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};
use tokio_tungstenite::connect_async;

// OMEGA_HOME and OMEGA_BIN are process-global; serialize every test in this
// binary that touches either (same pattern as orchestrate_test.rs's LOCK).
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

fn ws_url(base: &str, path: &str, token: &str) -> String {
    format!("{}{path}?token={token}", base.replacen("http", "ws", 1))
}

fn install_fake_omega(dir: &std::path::Path, script_body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("omega");
    std::fs::write(&path, format!("#!/usr/bin/env bash\n{script_body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_BIN", &path);
}

/// A fake `omega` that errors loudly if it is EVER invoked — proves the
/// caller rejected the request before spawning anything.
fn install_fake_omega_that_must_not_run(dir: &std::path::Path) {
    install_fake_omega(dir, "echo 'SHOULD NEVER RUN' >&2\nexit 1");
}

fn clear_env() {
    std::env::remove_var("OMEGA_HOME");
    std::env::remove_var("OMEGA_BIN");
}

#[tokio::test]
async fn stream_rejects_empty_name_before_any_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_omega_that_must_not_run(bin_dir.path());
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    // A REAL WS handshake attempt proves this hits our handler's own
    // validation (a plain HTTP 400), never an upgrade followed by an
    // in-loop error.
    let url = ws_url(&base, "/v1/new-project/stream", &token) + "&name=&category=works";
    let err = connect_async(url).await.unwrap_err();
    assert!(err.to_string().contains("400"), "unexpected error: {err}");

    clear_env();
}

#[tokio::test]
async fn stream_rejects_bad_name_charset_before_any_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_omega_that_must_not_run(bin_dir.path());
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/new-project/stream", &token) + "&name=My_Project&category=works";
    let err = connect_async(url).await.unwrap_err();
    assert!(err.to_string().contains("400"), "unexpected error: {err}");

    clear_env();
}

/// Finding 1 (adversarial review round): `name=--build` must never reach
/// the prompt string `cmd_new_project` builds — even though the `--` argv
/// separator itself protects clap's own flag parsing (confirmed correct by
/// the reviewer), the SAME literal text ends up in the `/omega-new-project
/// ...` prompt in a position indistinguishable from a real opt-in
/// `--build` flag. `is_slug` must reject any leading `-` outright, before
/// any spawn.
#[tokio::test]
async fn stream_rejects_name_starting_with_dash_build_before_any_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_omega_that_must_not_run(bin_dir.path());
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/new-project/stream", &token) + "&name=--build&category=works";
    let err = connect_async(url).await.unwrap_err();
    assert!(err.to_string().contains("400"), "unexpected error: {err}");

    clear_env();
}

/// Finding 1 continued: `name=--dry-run` is the same class of leak — the
/// literal text `--dry-run` in the prompt string is indistinguishable from
/// this endpoint's own (never-forwarded) real `--dry-run` flag.
#[tokio::test]
async fn stream_rejects_name_starting_with_dash_dry_run_before_any_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_omega_that_must_not_run(bin_dir.path());
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/new-project/stream", &token) + "&name=--dry-run&category=works";
    let err = connect_async(url).await.unwrap_err();
    assert!(err.to_string().contains("400"), "unexpected error: {err}");

    clear_env();
}

/// Finding 1 continued: `group` reuses `name`'s slug check, so a leading
/// `-` must be rejected there too — `group` flows into the SAME prompt
/// string via `flags`/positional forwarding downstream.
#[tokio::test]
async fn stream_rejects_group_starting_with_dash_before_any_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_omega_that_must_not_run(bin_dir.path());
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url =
        ws_url(&base, "/v1/new-project/stream", &token) + "&name=cool-app&category=works&group=-x";
    let err = connect_async(url).await.unwrap_err();
    assert!(err.to_string().contains("400"), "unexpected error: {err}");

    clear_env();
}

/// Finding 4 (adversarial review round): the real spawned session name is
/// `format!("{name}-setup")`, which then passes through
/// `omega_core::session::sanitize_session_name` (truncates at
/// `MAX_SESSION_NAME_LEN` = 48). A `name` of 49 lowercase `a`s individually
/// passes `is_slug`/`MAX_SLUG_LEN`, but `"{name}-setup"` is 55 bytes and
/// sanitizes down to 48 `a`s — a silent truncation this endpoint must
/// reject before any spawn, mirroring `routes_team.rs`'s own
/// `"Team-{project}"` round-trip check.
#[tokio::test]
async fn stream_rejects_name_that_would_truncate_session_name_after_setup_suffix_before_any_spawn()
{
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_omega_that_must_not_run(bin_dir.path());
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let name = "a".repeat(49);
    let url =
        ws_url(&base, "/v1/new-project/stream", &token) + &format!("&name={name}&category=works");
    let err = connect_async(url).await.unwrap_err();
    assert!(err.to_string().contains("400"), "unexpected error: {err}");

    clear_env();
}

/// Finding 4 continued: proves the COLLISION the round-trip check exists to
/// prevent would genuinely have happened — `"a".repeat(49) + "-setup"` and
/// `"a".repeat(50) + "-setup"` both sanitize down to the IDENTICAL 48-byte
/// session name (`"a".repeat(48)`), so both distinct `name`s must be
/// independently rejected before any spawn, never silently merged onto one
/// real session.
#[tokio::test]
async fn stream_rejects_both_of_two_names_that_would_collide_onto_the_same_session_before_any_spawn(
) {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_omega_that_must_not_run(bin_dir.path());
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    // Sanity-check the collision premise itself before asserting on the
    // endpoint: both names, once suffixed and sanitized, land on the exact
    // same real session name.
    let name_a = "a".repeat(49);
    let name_b = "a".repeat(50);
    let session_a = omega_core::session::sanitize_session_name(&format!("{name_a}-setup"));
    let session_b = omega_core::session::sanitize_session_name(&format!("{name_b}-setup"));
    assert_eq!(
        session_a, session_b,
        "test premise broken: these two names no longer collide"
    );

    let url_a =
        ws_url(&base, "/v1/new-project/stream", &token) + &format!("&name={name_a}&category=works");
    let err_a = connect_async(url_a).await.unwrap_err();
    assert!(
        err_a.to_string().contains("400"),
        "unexpected error for name_a: {err_a}"
    );

    let url_b =
        ws_url(&base, "/v1/new-project/stream", &token) + &format!("&name={name_b}&category=works");
    let err_b = connect_async(url_b).await.unwrap_err();
    assert!(
        err_b.to_string().contains("400"),
        "unexpected error for name_b: {err_b}"
    );

    clear_env();
}

#[tokio::test]
async fn stream_rejects_unknown_category_before_any_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_omega_that_must_not_run(bin_dir.path());
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/new-project/stream", &token)
        + "&name=cool-app&category=not-a-real-category";
    let err = connect_async(url).await.unwrap_err();
    assert!(err.to_string().contains("400"), "unexpected error: {err}");

    clear_env();
}

#[tokio::test]
async fn stream_happy_path_streams_lines_then_success_exit_and_defaults_category() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture = bin_dir.path().join("argv.txt");
    install_fake_omega(
        bin_dir.path(),
        &format!(
            r#"
printf '%s\n' "$@" > '{capture}'
if [ "$1" = "new-project" ]; then
    echo "New project 'cool-app' — bootstrap running"
    echo "warming up" >&2
    echo "session: cool-app-setup"
    exit 0
fi
exit 1
"#,
            capture = capture.display()
        ),
    );
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    // category omitted entirely: must default to "works" in the argv.
    let url = ws_url(&base, "/v1/new-project/stream", &token) + "&name=cool-app";
    let (mut ws, _) = connect_async(url).await.unwrap();

    let mut lines: Vec<serde_json::Value> = Vec::new();
    let exit = loop {
        let msg = ws.next().await.unwrap().unwrap();
        let v: serde_json::Value = serde_json::from_str(&msg.into_text().unwrap()).unwrap();
        if v["type"] == "exit" {
            break v;
        }
        lines.push(v);
    };

    assert_eq!(
        lines.len(),
        3,
        "2 stdout + 1 stderr line expected, got {lines:?}"
    );
    let texts: Vec<&str> = lines.iter().map(|l| l["text"].as_str().unwrap()).collect();
    assert!(texts.iter().any(|t| t.contains("bootstrap running")));
    assert!(texts.iter().any(|t| t.contains("session: cool-app-setup")));
    assert_eq!(exit["success"], true);
    assert_eq!(exit["code"], 0);

    // argv proves: no --group (omitted), then --, then the THREE
    // positionals in NAME STACK CATEGORY order, category defaulted to
    // "works".
    let recorded = std::fs::read_to_string(&capture).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    assert_eq!(argv[0], "new-project");
    assert_eq!(argv[1], "--");
    assert_eq!(argv[2], "cool-app");
    assert_eq!(argv[3], "nextstack");
    assert_eq!(argv[4], "works");
    assert!(
        !argv.contains(&"--group"),
        "group must not be forwarded when omitted: {argv:?}"
    );
    assert!(
        !argv.contains(&"--build"),
        "build must never be forwarded: {argv:?}"
    );
    assert!(
        !argv.contains(&"--dry-run"),
        "dry-run must never be forwarded: {argv:?}"
    );

    clear_env();
}

#[tokio::test]
async fn stream_forwards_group_and_explicit_category_in_correct_argv_order() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let capture = bin_dir.path().join("argv.txt");
    install_fake_omega(
        bin_dir.path(),
        &format!(
            r#"
printf '%s\n' "$@" > '{capture}'
echo "ok"
exit 0
"#,
            capture = capture.display()
        ),
    );
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/new-project/stream", &token)
        + "&name=acme-app&category=client&group=acme-client";
    let (mut ws, _) = connect_async(url).await.unwrap();

    loop {
        let msg = ws.next().await.unwrap().unwrap();
        let v: serde_json::Value = serde_json::from_str(&msg.into_text().unwrap()).unwrap();
        if v["type"] == "exit" {
            break;
        }
    }

    let recorded = std::fs::read_to_string(&capture).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    // Flags first (--group <value>), then --, then NAME STACK CATEGORY.
    assert_eq!(argv[0], "new-project");
    assert_eq!(argv[1], "--group");
    assert_eq!(argv[2], "acme-client");
    assert_eq!(argv[3], "--");
    assert_eq!(argv[4], "acme-app");
    assert_eq!(argv[5], "nextstack");
    assert_eq!(argv[6], "client");

    clear_env();
}

#[tokio::test]
async fn stream_nonzero_exit_reports_failure() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_omega(bin_dir.path(), "echo 'trying...'; echo 'boom' >&2; exit 1");
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/new-project/stream", &token) + "&name=cool-app&category=works";
    let (mut ws, _) = connect_async(url).await.unwrap();

    let exit = loop {
        let msg = ws.next().await.unwrap().unwrap();
        let v: serde_json::Value = serde_json::from_str(&msg.into_text().unwrap()).unwrap();
        if v["type"] == "exit" {
            break v;
        }
    };
    assert_eq!(exit["success"], false);
    assert_eq!(exit["code"], 1);

    clear_env();
}

/// Disconnect-mid-stream: proves the process-group-kill MECHANISM itself
/// works generically for any nested-child-forking subprocess — the fake
/// `omega` script here deliberately forks a nested `bash -c 'sleep 5 &&
/// touch ...'` child to exercise that mechanism, mirroring
/// `orchestrate_test.rs::
/// disconnect_mid_stream_kills_the_process_group_even_when_child_is_silent`.
///
/// NOTE (Finding 2, Task B round): this is a SYNTHETIC fake, not
/// representative of the real `omega new-project` process tree — the real
/// subprocess forks nothing (see `new_project_stream_loop`'s doc comment
/// in `routes_new_project.rs`), so this test proves the kill mechanism is
/// harmless defense-in-depth, not that closing the WebSocket cancels a
/// real project bootstrap (it does not; there is no cancellation for
/// this endpoint).
#[tokio::test]
async fn disconnect_mid_stream_kills_the_process_group_even_when_child_is_silent() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let marker = bin_dir.path().join("silent-orphan-still-ran.marker");
    install_fake_omega(
        bin_dir.path(),
        &format!(
            r#"
if [ "$1" = "new-project" ]; then
    echo "starting"
    bash -c '
        sleep 5
        touch "{marker}"
    '
fi
"#,
            marker = marker.display()
        ),
    );
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/new-project/stream", &token) + "&name=cool-app&category=works";
    let (mut ws, _) = connect_async(url).await.unwrap();

    let first = ws.next().await.unwrap().unwrap();
    let v: serde_json::Value = serde_json::from_str(&first.into_text().unwrap()).unwrap();
    assert_eq!(v["text"], "starting");
    ws.close(None).await.unwrap();
    drop(ws);

    assert!(
        !marker.exists(),
        "marker must not exist yet — the nested child hasn't reached it"
    );

    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(8);
    while tokio::time::Instant::now() < deadline {
        assert!(
            !marker.exists(),
            "the SILENT nested child kept running after a clean disconnect"
        );
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    assert!(
        !marker.exists(),
        "the silent nested child survived the disconnect"
    );

    clear_env();
}

#[tokio::test]
async fn concurrency_cap_returns_429_when_new_project_permits_exhausted() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    // Long-running fake so the permit stays held while the extra connection
    // attempt fires.
    install_fake_omega(bin_dir.path(), "echo starting; sleep 5; exit 0");
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    // Must match server.rs's MAX_CONCURRENT_NEW_PROJECT_SPAWNS.
    const MAX_CONCURRENT_NEW_PROJECT_SPAWNS: usize = 2;

    let mut held = Vec::new();
    for i in 0..MAX_CONCURRENT_NEW_PROJECT_SPAWNS {
        let url = ws_url(&base, "/v1/new-project/stream", &token)
            + &format!("&name=cool-app-{i}&category=works");
        let (mut ws, _) = connect_async(url).await.unwrap();
        let first = ws.next().await.unwrap().unwrap();
        let v: serde_json::Value = serde_json::from_str(&first.into_text().unwrap()).unwrap();
        assert_eq!(v["text"], "starting");
        held.push(ws);
    }

    let url = ws_url(&base, "/v1/new-project/stream", &token) + "&name=one-too-many&category=works";
    let err = connect_async(url).await.unwrap_err();
    assert!(err.to_string().contains("429"), "unexpected error: {err}");

    clear_env();
}

#[tokio::test]
async fn stream_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let url = format!(
        "{}/v1/new-project/stream?name=x&category=works",
        base.replacen("http", "ws", 1)
    );
    let err = connect_async(url).await.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("401") || msg.contains("Unauthorized"),
        "unexpected error: {msg}"
    );
}
