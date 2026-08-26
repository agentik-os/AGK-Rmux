//! `GET /v1/missions` — the missions mirror over oracle progress ledgers.
//! Read-only; no state is needed beyond the ledger dir the OS already owns.

use axum::Json;
use serde_json::json;

pub async fn list() -> Json<serde_json::Value> {
    // missions::list() does blocking file I/O; keep it off the async runtime
    // thread, same as routes_sessions::list.
    let missions = tokio::task::spawn_blocking(crate::missions::list)
        .await
        .unwrap_or_default();
    Json(json!({ "missions": missions }))
}
