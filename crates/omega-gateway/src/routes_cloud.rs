//! Device-authenticated bootstrap of this box's outbound relay identity.

use crate::protocol::{CloudRegisterRequest, CloudRegisterResponse};
use crate::relay::RelayError;
use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

type RouteError = (StatusCode, Json<serde_json::Value>);

fn error(status: StatusCode, code: &'static str) -> RouteError {
    (status, Json(serde_json::json!({ "error": code })))
}

/// `POST /v1/cloud/register` combines two independent proofs: the existing
/// middleware has already verified this app's box-local device token, and this
/// handler verifies its Clerk session JWT. Only then may the box mint or reuse
/// its private outbound relay credential.
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<CloudRegisterRequest>,
) -> Result<Json<CloudRegisterResponse>, RouteError> {
    let claims = state
        .relay
        .verify_clerk(&request.clerk_jwt)
        .await
        .map_err(|_| error(StatusCode::UNAUTHORIZED, "invalid_clerk_jwt"))?;

    let dir = state.dir.clone();
    let box_id =
        tokio::task::spawn_blocking(move || crate::routes_box::read_or_create_box_id(&dir))
            .await
            .map_err(|_| error(StatusCode::INTERNAL_SERVER_ERROR, "box_id_unavailable"))?
            .map_err(|_| error(StatusCode::INTERNAL_SERVER_ERROR, "box_id_unavailable"))?;

    state
        .relay
        .register(claims.sub, box_id)
        .await
        .map_err(|relay_error| match relay_error {
            RelayError::OwnershipConflict => error(StatusCode::CONFLICT, "box_already_registered"),
            RelayError::InvalidConfiguration => {
                error(StatusCode::SERVICE_UNAVAILABLE, "relay_unavailable")
            }
            _ => error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "relay_registration_failed",
            ),
        })?;

    Ok(Json(CloudRegisterResponse { ok: true }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::DeviceStore;
    use crate::clerk::testkit::{sign_token, start_jwks_stub, TEST_ISSUER};
    use crate::config::GatewayConfig;
    use crate::relay::RelayManager;
    use crate::server::{build_router, AppState};
    use std::path::Path;
    use std::time::Duration;

    async fn spawn_gateway(
        dir: &Path,
        jwks_url: &str,
    ) -> (String, String, tokio::task::JoinHandle<()>) {
        let relay = RelayManager::for_test(
            dir.to_path_buf(),
            "ws://127.0.0.1:9/v1/register",
            jwks_url,
            TEST_ISSUER,
            Duration::from_secs(20),
            Duration::from_secs(60),
        )
        .unwrap();
        let (_, device_token) = DeviceStore::open(dir).issue("test-device");
        let app = build_router(AppState::with_relay(
            dir.to_path_buf(),
            GatewayConfig::default(),
            relay,
        ));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        (format!("http://{address}"), device_token, task)
    }

    #[tokio::test]
    async fn route_requires_device_auth_before_clerk_and_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let jwks = start_jwks_stub().await;
        let (base, _, task) = spawn_gateway(dir.path(), &jwks).await;
        let response = reqwest::Client::new()
            .post(format!("{base}/v1/cloud/register"))
            .json(&serde_json::json!({ "clerk_jwt": "not-a-token" }))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(!dir.path().join("relay_registration.json").exists());
        assert!(!dir.path().join("box_id.txt").exists());
        task.abort();
    }

    #[tokio::test]
    async fn invalid_expired_and_jwks_outage_fail_before_persistence() {
        let now = chrono::Utc::now().timestamp();
        for (jwks_url, token) in [
            (
                "http://127.0.0.1:9/jwks".to_string(),
                sign_token("user_1", TEST_ISSUER, now, now + 300),
            ),
            (
                {
                    let url = start_jwks_stub().await;
                    url
                },
                sign_token("user_1", TEST_ISSUER, now - 600, now - 1),
            ),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let (base, device_token, task) = spawn_gateway(dir.path(), &jwks_url).await;
            let response = reqwest::Client::new()
                .post(format!("{base}/v1/cloud/register"))
                .bearer_auth(device_token)
                .json(&serde_json::json!({ "clerk_jwt": token }))
                .send()
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            assert!(!dir.path().join("relay_registration.json").exists());
            assert!(!dir.path().join("box_id.txt").exists());
            task.abort();
        }
    }

    #[tokio::test]
    async fn active_registration_returns_only_ok_reuses_secret_and_rejects_new_owner() {
        let dir = tempfile::tempdir().unwrap();
        let jwks = start_jwks_stub().await;
        let (base, device_token, task) = spawn_gateway(dir.path(), &jwks).await;
        let now = chrono::Utc::now().timestamp();
        let client = reqwest::Client::new();
        let first = client
            .post(format!("{base}/v1/cloud/register"))
            .bearer_auth(&device_token)
            .json(&serde_json::json!({
                "clerk_jwt": sign_token("user_1", TEST_ISSUER, now, now + 300)
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK);
        let response_text = first.text().await.unwrap();
        assert_eq!(response_text, r#"{"ok":true}"#);
        let stored_before =
            std::fs::read_to_string(dir.path().join("relay_registration.json")).unwrap();
        let stored_json: serde_json::Value = serde_json::from_str(&stored_before).unwrap();
        let credential = stored_json["relay_credential"].as_str().unwrap();
        assert_eq!(credential.len(), 64);
        assert!(!response_text.contains(credential));

        let repeat = client
            .post(format!("{base}/v1/cloud/register"))
            .bearer_auth(&device_token)
            .json(&serde_json::json!({
                "clerk_jwt": sign_token("user_1", TEST_ISSUER, now, now + 300)
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(repeat.status(), StatusCode::OK);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("relay_registration.json")).unwrap(),
            stored_before
        );

        let other_owner = client
            .post(format!("{base}/v1/cloud/register"))
            .bearer_auth(&device_token)
            .json(&serde_json::json!({
                "clerk_jwt": sign_token("user_2", TEST_ISSUER, now, now + 300)
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(other_owner.status(), StatusCode::CONFLICT);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("relay_registration.json")).unwrap(),
            stored_before
        );
        task.abort();
    }

    #[tokio::test]
    async fn unknown_json_field_is_rejected_without_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let jwks = start_jwks_stub().await;
        let (base, device_token, task) = spawn_gateway(dir.path(), &jwks).await;
        let now = chrono::Utc::now().timestamp();
        let response = reqwest::Client::new()
            .post(format!("{base}/v1/cloud/register"))
            .bearer_auth(device_token)
            .json(&serde_json::json!({
                "clerk_jwt": sign_token("user_1", TEST_ISSUER, now, now + 300),
                "clerk_user_id": "forged"
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(!dir.path().join("relay_registration.json").exists());
        task.abort();
    }
}
