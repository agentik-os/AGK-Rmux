//! Adversarial review tests for the session-org overlay, written by an
//! independent reviewer (not the implementer) to verify claims made in the
//! diff/doc-comments rather than trust them:
//!
//! 1. Concurrent PUTs on N DISTINCT keys, fired at the real HTTP router,
//!    must not lose an update (the classic read-whole-map / mutate-one-key
//!    / write-whole-map race).
//! 2. A corrupted `session_org.json` on disk must not panic the gateway
//!    process when `GET /v1/session-org` is served -- it must degrade to
//!    an empty overlay (per `session_org.rs::read_all`'s doc comment).

use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

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

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn n_concurrent_puts_on_distinct_keys_never_lose_an_update() {
    const N: usize = 50;

    let gateway_dir = tempfile::tempdir().unwrap();
    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let mut handles = Vec::with_capacity(N);
    for i in 0..N {
        let base = base.clone();
        let token = token.clone();
        handles.push(tokio::spawn(async move {
            let client = reqwest::Client::new();
            let name = format!("oracle-Concurrent-{i}");
            let res = client
                .put(format!("{base}/v1/session-org/{name}"))
                .bearer_auth(&token)
                .json(&serde_json::json!({"label": format!("label-{i}"), "pinned": true}))
                .send()
                .await
                .unwrap();
            assert_eq!(res.status(), 200, "PUT for key {i} must succeed");
        }));
    }
    for h in handles {
        h.await.unwrap();
    }

    let client = reqwest::Client::new();
    let res = client
        .get(format!("{base}/v1/session-org"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let entries = body["entries"].as_object().unwrap();

    assert_eq!(
        entries.len(),
        N,
        "lost update detected: expected {N} distinct keys after {N} concurrent PUTs, got {} -- \
         a read-modify-write race clobbered at least one writer's upsert",
        entries.len()
    );
    for i in 0..N {
        let name = format!("oracle-Concurrent-{i}");
        assert_eq!(
            entries[&name]["label"],
            format!("label-{i}"),
            "key {name} missing or wrong after concurrent PUTs -- lost update"
        );
        assert_eq!(entries[&name]["pinned"], true);
    }
}

#[tokio::test]
async fn corrupted_session_org_json_degrades_to_empty_map_no_panic() {
    let gateway_dir = tempfile::tempdir().unwrap();
    // Pre-seed a garbage (non-JSON) file at the exact path the store reads,
    // simulating disk corruption / a torn write / manual tampering.
    std::fs::write(
        gateway_dir.path().join("session_org.json"),
        b"{not valid json!!!",
    )
    .unwrap();

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/session-org"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    // The whole point: this must be a normal 200 with an empty map, never a
    // 500, a hung connection (panic unwinding the task), or a crashed
    // process -- the axum::serve task would abort silently on panic and
    // every OTHER route on this gateway would go down with it.
    assert_eq!(
        res.status(),
        200,
        "corrupted session_org.json must not surface as a 500 or crash"
    );
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(
        body["entries"],
        serde_json::json!({}),
        "corrupted file must degrade to an empty overlay"
    );
}

#[tokio::test]
async fn corrupted_session_org_json_does_not_block_a_subsequent_put() {
    let gateway_dir = tempfile::tempdir().unwrap();
    std::fs::write(gateway_dir.path().join("session_org.json"), b"garbage").unwrap();

    let (app, token) = app_and_token(gateway_dir.path()).await;
    let base = spawn(app).await;

    let res = reqwest::Client::new()
        .put(format!("{base}/v1/session-org/oracle-Foo-1"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"label": "recovered"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "a PUT after a corrupted read must still succeed (treats corrupt as empty, then writes cleanly)");

    let res = reqwest::Client::new()
        .get(format!("{base}/v1/session-org"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["entries"]["oracle-Foo-1"]["label"], "recovered");
}
