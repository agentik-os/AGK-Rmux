use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

fn project_fixture() -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    let node = home.path().join("Station/Customer/alpha");
    let rust = home.path().join("work/beta");
    let bare = home.path().join("gamma");
    std::fs::create_dir_all(&node).unwrap();
    std::fs::create_dir_all(&rust).unwrap();
    std::fs::create_dir_all(bare.join(".git")).unwrap();
    std::fs::write(node.join("package.json"), "{}").unwrap();
    std::fs::write(
        rust.join("Cargo.toml"),
        "[package]\nname='beta'\nversion='0.1.0'\n",
    )
    .unwrap();
    home
}

fn clear_env() {
    std::env::remove_var("OMEGA_HOME");
}

#[tokio::test]
async fn get_projects_returns_the_discovered_project_list() {
    let _guard = LOCK.lock().await;
    let home = project_fixture();
    std::env::set_var("OMEGA_HOME", home.path());
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/projects"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();

    let projects = body["projects"].as_array().unwrap();
    assert_eq!(projects.len(), 3);
    let names = projects
        .iter()
        .filter_map(|project| project["name"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        names,
        std::collections::BTreeSet::from(["alpha", "beta", "gamma"])
    );

    let first = &projects[0];
    assert!(
        first["name"].as_str().is_some(),
        "first project missing name"
    );
    assert!(
        first["container"].as_str().is_some(),
        "first project missing container"
    );
    assert!(
        first["stack"].as_array().is_some(),
        "first project missing stack"
    );
    clear_env();
}

#[tokio::test]
async fn get_projects_requires_auth() {
    let _guard = LOCK.lock().await;
    let home = project_fixture();
    std::env::set_var("OMEGA_HOME", home.path());
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/projects"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
    clear_env();
}
