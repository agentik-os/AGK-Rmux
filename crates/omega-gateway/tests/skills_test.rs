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

fn skill_fixture() -> tempfile::TempDir {
    let omega = tempfile::tempdir().unwrap();
    let root = omega.path().join("skills");
    std::fs::create_dir_all(&root).unwrap();
    for index in 0..60 {
        let name = if index == 0 {
            "audit-fixture".to_string()
        } else {
            format!("fixture-{index:02}")
        };
        let dir = root.join(&name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("SKILL.md"),
            format!(
                "---\nname: {name}\ndescription: {} test skill\n---\n\n# {name}\n",
                if index == 0 { "Audit" } else { "Gateway" }
            ),
        )
        .unwrap();
    }
    let monitor = root.join("monitor");
    std::fs::create_dir_all(&monitor).unwrap();
    std::fs::write(
        monitor.join("SKILL.md"),
        "---\nname: monitor\ndescription: Monitor sessions\n---\n\n# Monitor\n",
    )
    .unwrap();
    omega
}

fn clear_env() {
    std::env::remove_var("OMEGA_DIR");
}

#[tokio::test]
async fn get_skills_returns_the_full_catalog_capped_at_the_default_limit() {
    let _guard = LOCK.lock().await;
    let omega = skill_fixture();
    std::env::set_var("OMEGA_DIR", omega.path());
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/skills"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();

    let total = body["total"].as_u64().unwrap();
    assert_eq!(total, 61);

    let skills = body["skills"].as_array().unwrap();
    assert!(
        skills.len() <= 50,
        "default cap is 50, got {}",
        skills.len()
    );
    clear_env();
}

#[tokio::test]
async fn get_skills_filters_by_q_and_caps_by_limit() {
    let _guard = LOCK.lock().await;
    let omega = skill_fixture();
    std::env::set_var("OMEGA_DIR", omega.path());
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/skills?q=audit&limit=5"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();

    let skills = body["skills"].as_array().unwrap();
    assert!(
        skills.len() <= 5,
        "limit=5 must cap the returned skills, got {}",
        skills.len()
    );
    assert!(
        !skills.is_empty(),
        "expected at least one skill matching 'audit'"
    );

    for skill in skills {
        let name = skill["name"].as_str().unwrap().to_lowercase();
        let description = skill["description"].as_str().unwrap().to_lowercase();
        assert!(
            name.contains("audit") || description.contains("audit"),
            "skill {name:?} / {description:?} does not match q=audit"
        );
    }
    clear_env();
}

#[tokio::test]
async fn get_skills_requires_auth() {
    let _guard = LOCK.lock().await;
    let omega = skill_fixture();
    std::env::set_var("OMEGA_DIR", omega.path());
    let gateway_dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/skills"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);
    clear_env();
}

#[tokio::test]
async fn get_skills_filters_by_server_category() {
    let _guard = LOCK.lock().await;
    let omega = skill_fixture();
    std::env::set_var("OMEGA_DIR", omega.path());
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let response = reqwest::Client::new()
        .get(format!("{base}/v1/skills?category=Audit&limit=200"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    let skills = body["skills"].as_array().unwrap();
    assert!(!skills.is_empty());
    assert!(skills.iter().all(|skill| skill["category"] == "Audit"));
    clear_env();
}

#[tokio::test]
async fn skill_detail_returns_content_but_never_a_host_path() {
    let _guard = LOCK.lock().await;
    let omega = skill_fixture();
    std::env::set_var("OMEGA_DIR", omega.path());
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let response = reqwest::Client::new()
        .get(format!("{base}/v1/skills/monitor"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["skill"]["name"], "monitor");
    assert!(body["skill"]["content"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
    assert!(body["skill"].get("path").is_none());
    clear_env();
}

#[tokio::test]
async fn missing_skill_install_returns_service_unavailable_not_empty_success() {
    let _guard = LOCK.lock().await;
    let omega = tempfile::tempdir().unwrap();
    std::env::set_var("OMEGA_DIR", omega.path());
    let gateway_dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(gateway_dir.path()).issue("t");
    let app = build_router(AppState::new(
        gateway_dir.path().to_path_buf(),
        GatewayConfig::default(),
    ));
    let base = spawn(app).await;
    let response = reqwest::Client::new()
        .get(format!("{base}/v1/skills"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 503);
    clear_env();
}
