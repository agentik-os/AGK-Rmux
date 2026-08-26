//! `GET /v1/files` + `GET /v1/files/read` — handler-level tests over a real
//! HTTP server. The pure traversal-guard unit tests live inline in
//! `routes_files.rs`'s own `#[cfg(test)]` module; this file proves the same
//! guarantees hold end-to-end through axum's `Query` extraction and the
//! discovered-project allowlist.

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

// OMEGA_HOME is process-global — serialize every test in this binary that
// touches it (same pattern as dispatch_test.rs's LOCK).
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

/// Creates a fake `$HOME` containing exactly one discoverable project (a
/// `.git`-marked directory named `project_name`), points `OMEGA_HOME` at it,
/// and returns the project's real on-disk root — the same override
/// `config::home_dir()` / `routes_files.rs` respects (mirrors
/// `dispatch_test.rs::install_fake_home`).
fn install_fake_home(home_dir: &std::path::Path, project_name: &str) -> std::path::PathBuf {
    let project_root = home_dir.join(project_name);
    std::fs::create_dir_all(project_root.join(".git")).unwrap();
    std::env::set_var("OMEGA_HOME", home_dir);
    project_root
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
async fn list_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/files?project=whatever"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn read_requires_auth() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/files/read?project=whatever&path=x"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn list_rejects_unknown_project_before_touching_disk() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    install_fake_home(home_dir.path(), "TestProj");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/files?project=NopeNotReal"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    std::env::remove_var("OMEGA_HOME");
}

#[tokio::test]
async fn list_returns_project_root_entries_dirs_first_then_alpha() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let root = install_fake_home(home_dir.path(), "TestProj");
    std::fs::write(root.join("b.txt"), "b").unwrap();
    std::fs::write(root.join("a.txt"), "a").unwrap();
    std::fs::create_dir(root.join("zdir")).unwrap();

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/files?project=TestProj"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let entries = body["entries"].as_array().unwrap();
    let names: Vec<&str> = entries
        .iter()
        .map(|e| e["name"].as_str().unwrap())
        .collect();
    // .git is also present (created by install_fake_home) but ordering only
    // needs to hold: every dir before every file, alpha within each group.
    let zdir_idx = names.iter().position(|n| *n == "zdir").unwrap();
    let a_idx = names.iter().position(|n| *n == "a.txt").unwrap();
    let b_idx = names.iter().position(|n| *n == "b.txt").unwrap();
    assert!(zdir_idx < a_idx, "dirs must sort before files");
    assert!(a_idx < b_idx, "files must sort alphabetically");

    std::env::remove_var("OMEGA_HOME");
}

#[tokio::test]
async fn list_rejects_traversal_via_http() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    install_fake_home(home_dir.path(), "TestProj");
    std::fs::write(home_dir.path().join("secret.txt"), "top secret").unwrap();

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!(
            "{base}/v1/files?project=TestProj&path=..%2Fsecret.txt"
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 403);

    std::env::remove_var("OMEGA_HOME");
}

#[tokio::test]
async fn read_rejects_symlink_escape_via_http() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let root = install_fake_home(home_dir.path(), "TestProj");
    let outside = home_dir.path().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("secret.txt"), "top secret").unwrap();
    std::os::unix::fs::symlink(outside.join("secret.txt"), root.join("link")).unwrap();

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/files/read?project=TestProj&path=link"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 403);

    std::env::remove_var("OMEGA_HOME");
}

/// Existence-oracle regression at the HTTP layer: two requests that both
/// escape the project root must be INDISTINGUISHABLE by status code,
/// whether or not the outside directory they name actually exists. Before
/// the fix the guard only checked the leaf's immediate parent, so the first
/// of these returned 403 and the second 404 — enough for an authenticated
/// caller to enumerate arbitrary directories on the box (proven live:
/// `/home/vibe/.ssh` answered 403, `/home/vibe/.no-such-dir` answered 404).
#[tokio::test]
async fn escape_status_is_identical_whether_the_outside_dir_exists_via_http() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    install_fake_home(home_dir.path(), "TestProj");
    // A real directory OUTSIDE the project root, and (implicitly) a sibling
    // name that does not exist at all.
    std::fs::create_dir_all(home_dir.path().join("real-outside-dir")).unwrap();

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    let status_of = |path: &'static str| {
        let client = client.clone();
        let base = base.clone();
        let token = token.clone();
        async move {
            client
                .get(format!("{base}/v1/files/read?project=TestProj&path={path}"))
                .bearer_auth(&token)
                .send()
                .await
                .unwrap()
                .status()
        }
    };

    let exists = status_of("..%2Freal-outside-dir%2Fleaf.txt").await;
    let missing = status_of("..%2Fno-such-outside-dir%2Fleaf.txt").await;
    assert_eq!(exists, 403);
    assert_eq!(
        missing, exists,
        "status must not reveal whether the outside directory exists"
    );

    std::env::remove_var("OMEGA_HOME");
}

#[tokio::test]
async fn read_returns_content_for_a_real_text_file() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let root = install_fake_home(home_dir.path(), "TestProj");
    std::fs::write(root.join("hello.txt"), "hello gateway\n").unwrap();

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!(
            "{base}/v1/files/read?project=TestProj&path=hello.txt"
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["content"], "hello gateway\n");

    std::env::remove_var("OMEGA_HOME");
}

#[tokio::test]
async fn read_requires_path() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    install_fake_home(home_dir.path(), "TestProj");

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/files/read?project=TestProj"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    std::env::remove_var("OMEGA_HOME");
}

#[tokio::test]
async fn read_rejects_oversized_file() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let root = install_fake_home(home_dir.path(), "TestProj");
    let path = root.join("big.bin");
    let f = std::fs::File::create(&path).unwrap();
    f.set_len(600_000).unwrap();

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!(
            "{base}/v1/files/read?project=TestProj&path=big.bin"
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 413);

    std::env::remove_var("OMEGA_HOME");
}

#[tokio::test]
async fn read_rejects_binary_file() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let root = install_fake_home(home_dir.path(), "TestProj");
    std::fs::write(root.join("binary.dat"), [0xFFu8, 0xFE, 0xFD, 0x80]).unwrap();

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!(
            "{base}/v1/files/read?project=TestProj&path=binary.dat"
        ))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    std::env::remove_var("OMEGA_HOME");
}

#[tokio::test]
async fn read_rejects_a_directory_path() {
    let _g = LOCK.lock().await;
    let gateway_dir = tempfile::tempdir().unwrap();
    let home_dir = tempfile::tempdir().unwrap();
    let root = install_fake_home(home_dir.path(), "TestProj");
    std::fs::create_dir(root.join("subdir")).unwrap();

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/files/read?project=TestProj&path=subdir"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    std::env::remove_var("OMEGA_HOME");
}
