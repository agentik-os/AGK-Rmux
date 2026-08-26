use crate::auth::{DeviceStore, PairingCode};
use crate::protocol::{PairRequest, PairResponse};
use crate::server::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;

pub async fn pair(
    State(state): State<AppState>,
    Json(req): Json<PairRequest>,
) -> Result<Json<PairResponse>, (StatusCode, Json<serde_json::Value>)> {
    if !PairingCode::consume(&state.dir, &req.code) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "invalid or expired code" })),
        ));
    }
    let mut store = DeviceStore::open(&state.dir);
    let (device, token) = store.issue(&req.device_name);
    Ok(Json(PairResponse {
        device_id: device.id,
        token,
    }))
}
