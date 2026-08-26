//! Integration tests for `GET /v1/audits`, `POST /v1/audit` (pre-flight),
//! and `GET /v1/audit/stream` (the live WebSocket) — `crates/omega-gateway/
//! src/routes_audit.rs`.
//!
//! SAFETY (R-SEC / this task's explicit brief): every test here that could
//! reach a real subprocess spawn sets `OMEGA_BIN` to a FAKE script FIRST. No
//! test in this file ever calls the audit-stream WS route with `OMEGA_BIN`
//! unset, which would spawn the box's real `~/.local/bin/omega audit run`.
//! The "rejects before spawning" tests go one step further and point
//! `OMEGA_BIN` at a script that errors loudly if it is ever invoked at all
//! (mirrors `agents_install_test.rs::install_fake_omega_that_must_not_run`).

use futures_util::StreamExt;
use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};
use tokio_tungstenite::connect_async;

// OMEGA_HOME and OMEGA_BIN are process-global; serialize every test in this
// binary that touches either (same pattern as dispatch_test.rs's LOCK).
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

/// Creates a fake `$HOME` containing exactly one discoverable project (a
/// `.git`-marked directory named `name`) and points `OMEGA_HOME` at it — the
/// override `config::home_dir()` / `routes_audit.rs::resolve_project_path`
/// respects (same as `dispatch_test.rs::install_fake_home`). Returns the
/// project's real on-disk root.
fn install_fake_home(home_dir: &std::path::Path, project_name: &str) -> std::path::PathBuf {
    let root = home_dir.join(project_name);
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::env::set_var("OMEGA_HOME", home_dir);
    root
}

/// Writes an executable fake `omega` script that, when invoked, prints canned
/// stdout/stderr lines then exits with the given code. Points `OMEGA_BIN` at
/// it.
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
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("omega");
    std::fs::write(
        &path,
        "#!/usr/bin/env bash\necho 'SHOULD NEVER RUN' >&2\nexit 1\n",
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_BIN", &path);
}

fn clear_env() {
    std::env::remove_var("OMEGA_HOME");
    std::env::remove_var("OMEGA_BIN");
}

// ── GET /v1/audits ───────────────────────────────────────────────────────

#[tokio::test]
async fn list_returns_the_full_catalog_including_codeaudit() {
    let _g = LOCK.lock().await;
    let dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/audits"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let audits = body["audits"].as_array().unwrap();
    assert_eq!(
        audits.len(),
        23,
        "expected the full 23-audit Quality Arsenal catalog"
    );
    assert!(
        audits
            .iter()
            .any(|a| a["id"] == "codeaudit" && a["domain"] == "Code"),
        "codeaudit entry missing or malformed: {audits:?}"
    );
}

#[tokio::test]
async fn list_requires_auth() {
    let dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/audits"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

// ── POST /v1/audit ───────────────────────────────────────────────────────

#[tokio::test]
async fn check_rejects_unknown_kind_and_never_spawns() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_home(home_dir.path(), "TestProj");
    install_fake_omega_that_must_not_run(bin_dir.path());
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/audit"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "TestProj", "kind": "not-a-real-audit"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("not-a-real-audit"));

    clear_env();
}

#[tokio::test]
async fn check_rejects_unknown_project_and_never_spawns() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    // A real project exists ("TestProj"), but the request names a different
    // one that provably does not exist.
    install_fake_home(home_dir.path(), "TestProj");
    install_fake_omega_that_must_not_run(bin_dir.path());
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/audit"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "definitely-not-a-real-project", "kind": "codeaudit"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("definitely-not-a-real-project"));

    clear_env();
}

#[tokio::test]
async fn check_accepts_a_real_audit_and_project_and_spawns_nothing() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_home(home_dir.path(), "TestProj");
    // check() spawns nothing (pure pre-flight), so OMEGA_BIN is left pointed
    // at the must-not-run script too, to prove it.
    install_fake_omega_that_must_not_run(bin_dir.path());
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .post(format!("{base}/v1/audit"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"project": "TestProj", "kind": "codeaudit"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["kind"], "codeaudit");
    assert!(body["name"].is_string());
    assert!(body["phases"].as_u64().unwrap() > 0);
    assert!(body["max_score"].as_u64().unwrap() > 0);

    clear_env();
}

// ── GET /v1/audit/stream ─────────────────────────────────────────────────

#[tokio::test]
async fn stream_rejects_unknown_kind_before_any_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_home(home_dir.path(), "TestProj");
    install_fake_omega_that_must_not_run(bin_dir.path());
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    // A REAL WS handshake attempt (valid auth token, real Upgrade headers via
    // connect_async) proves this hits our handler's own validation, not just
    // axum's upgrade-header extraction: the server answers with a plain HTTP
    // 400 instead of 101 Switching Protocols, so connect_async errors.
    let url = ws_url(&base, "/v1/audit/stream", &token) + "&project=TestProj&kind=not-a-real-audit";
    let err = connect_async(url).await.unwrap_err();
    assert!(err.to_string().contains("400"), "unexpected error: {err}");

    clear_env();
}

#[tokio::test]
async fn stream_rejects_unknown_project_before_any_spawn() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_home(home_dir.path(), "TestProj");
    install_fake_omega_that_must_not_run(bin_dir.path());
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/audit/stream", &token) + "&project=nope-not-real&kind=codeaudit";
    let err = connect_async(url).await.unwrap_err();
    assert!(err.to_string().contains("400"), "unexpected error: {err}");

    clear_env();
}

#[tokio::test]
async fn stream_happy_path_streams_lines_then_success_exit_and_resolves_real_project_path() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let project_root = install_fake_home(home_dir.path(), "TestProj");
    let capture = bin_dir.path().join("argv.txt");
    install_fake_omega(
        bin_dir.path(),
        &format!(
            r#"
printf '%s\n' "$@" > '{capture}'
if [ "$1" = "audit" ] && [ "$2" = "run" ]; then
    echo "Audit: Code Architecture Audit (Code)"
    echo "Phases: 23, Max score: /420" >&2
    echo "Skill: audits/codeaudit/SKILL.md"
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

    let url = ws_url(&base, "/v1/audit/stream", &token) + "&project=TestProj&kind=codeaudit";
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
    assert!(texts
        .iter()
        .any(|t| t.contains("Audit: Code Architecture Audit")));
    assert!(texts.iter().any(|t| t.contains("Phases: 23")));
    assert!(texts.iter().any(|t| t.contains("Skill:")));
    let stderr_count = lines.iter().filter(|l| l["stream"] == "stderr").count();
    assert_eq!(stderr_count, 1);

    assert_eq!(exit["success"], true);
    assert_eq!(exit["code"], 0);

    // Proves argv was built server-side from the resolved project ROOT, not
    // a client-supplied path: ["audit", "run", "codeaudit", "--dir",
    // <real canonicalized project root>].
    let recorded = std::fs::read_to_string(&capture).unwrap();
    let argv: Vec<&str> = recorded.lines().collect();
    assert_eq!(argv[0], "audit");
    assert_eq!(argv[1], "run");
    assert_eq!(argv[2], "codeaudit");
    assert_eq!(argv[3], "--dir");
    assert_eq!(std::path::Path::new(argv[4]), project_root.as_path());

    clear_env();
}

#[tokio::test]
async fn stream_nonzero_exit_reports_failure() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_home(home_dir.path(), "TestProj");
    install_fake_omega(bin_dir.path(), "echo 'trying...'; echo 'boom' >&2; exit 7");
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let url = ws_url(&base, "/v1/audit/stream", &token) + "&project=TestProj&kind=codeaudit";
    let (mut ws, _) = connect_async(url).await.unwrap();

    let exit = loop {
        let msg = ws.next().await.unwrap().unwrap();
        let v: serde_json::Value = serde_json::from_str(&msg.into_text().unwrap()).unwrap();
        if v["type"] == "exit" {
            break v;
        }
    };

    assert_eq!(exit["success"], false);
    assert_eq!(exit["code"], 7);

    clear_env();
}

/// Disconnect-mid-stream: proves the spawned `omega audit run` child's
/// PROCESS GROUP is actually killed when the client disconnects while the
/// fake command has gone SILENT (no further output, so a send-driven-only
/// disconnect check would never fire) — mirrors
/// `agents_install_adversarial_test.rs::silent_child_after_clean_disconnect_still_gets_killed`,
/// the exact regression `audit_stream_loop`'s dual-armed `tokio::select!` is
/// meant to catch (a client that goes quiet is noticed via the socket-read
/// branch, independent of any child output).
#[tokio::test]
async fn disconnect_mid_stream_kills_the_process_group_even_when_child_is_silent() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    install_fake_home(home_dir.path(), "TestProj");
    let marker = bin_dir.path().join("silent-orphan-still-ran.marker");
    install_fake_omega(
        bin_dir.path(),
        &format!(
            r#"
if [ "$1" = "audit" ]; then
    echo "starting"
    # A nested foreground child of THIS script (mirrors omega_cli's own
    # nested subprocess shape), silent for well past the disconnect+wait
    # window before touching the marker.
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

    let url = ws_url(&base, "/v1/audit/stream", &token) + "&project=TestProj&kind=codeaudit";
    let (mut ws, _) = connect_async(url).await.unwrap();

    // Read exactly the first frame, then send a CLEAN close handshake —
    // before the nested silent sleep even starts counting down.
    let first = ws.next().await.unwrap().unwrap();
    let v: serde_json::Value = serde_json::from_str(&first.into_text().unwrap()).unwrap();
    assert_eq!(v["text"], "starting");
    ws.close(None).await.unwrap();
    drop(ws);

    assert!(
        !marker.exists(),
        "marker must not exist yet — the nested child hasn't reached it"
    );

    // Give the server up to 8s — comfortably past the nested child's 5s
    // silent sleep — to notice the clean close via the socket-read branch
    // and group-kill the child.
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(8);
    while tokio::time::Instant::now() < deadline {
        assert!(
            !marker.exists(),
            "the marker WAS written: the SILENT nested child kept running to completion \
             after a clean client disconnect the gateway never noticed. A send-driven-only \
             disconnect check never fires when the child produces no further output — the \
             loop must also read the socket itself (tokio::select! on socket.recv()) to \
             catch this."
        );
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    assert!(
        !marker.exists(),
        "the marker WAS written at the deadline: the silent nested child survived the \
         disconnect — the process-group kill did not actually stop the real work."
    );

    clear_env();
}
