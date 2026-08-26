use axum::extract::{Path, State};
use axum::http::StatusCode;
use omega_gateway::config::GatewayConfig;
use omega_gateway::routes_chat;
use omega_gateway::server::AppState;

/// Regression for the path-traversal guard on `GET /v1/chats/{id}`: without
/// `valid_chat_id`, an id like `../planted` resolves outside the chats dir
/// (`ChatStore::dir_for` joins it straight onto `chats_dir`) and would
/// successfully read a real file planted next to the chats dir. With the
/// guard, the handler must reject it as 404 BEFORE the filesystem is ever
/// touched — proving this is "404, not a file read".
#[tokio::test]
async fn get_with_traversal_id_is_404_not_a_file_read() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let planted_dir = gateway_dir.path().join("planted");
    std::fs::create_dir_all(&planted_dir).unwrap();
    std::fs::write(
        planted_dir.join("meta.json"),
        r#"{"id":"planted","title":null,"agent":"claude","cwd":"/tmp","created_at":"t","updated_at":"t","provider_session_id":null}"#,
    )
    .unwrap();

    let state = AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default());
    // `chats_dir.join("../planted")` resolves (at the OS level) straight to
    // `gateway_dir/planted`, i.e. the planted file above, if unvalidated.
    let result = routes_chat::get(State(state), Path("../planted".to_string())).await;
    assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_with_deeper_traversal_id_is_404() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let state = AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default());
    let result = routes_chat::get(State(state), Path("../../etc/passwd".to_string())).await;
    assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_with_wrong_shape_id_is_404() {
    let gateway_dir = tempfile::tempdir().unwrap();
    let state = AppState::new(gateway_dir.path().to_path_buf(), GatewayConfig::default());
    // Uppercase hex, same length as a real id — still rejected.
    let result = routes_chat::get(State(state), Path("ABCDEF0123456789".to_string())).await;
    assert_eq!(result.unwrap_err(), StatusCode::NOT_FOUND);
}
