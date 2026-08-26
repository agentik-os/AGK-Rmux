//! `POST /v1/deposit` — the HTTP twin of the Telegram DEPOSIT bot
//! (`~/.omega/telegram-bot/inbox-bot.ts`, read in full as the ground-truth
//! spec): upload a file as multipart, land it in the deposit inbox, and fan
//! it out into the named shared boxes — unless it looks like a credential,
//! in which case it is held in the inbox only. All the actual logic lives in
//! `crate::deposit::deposit`; this module is a thin multipart-parsing +
//! `spawn_blocking` wrapper around it, same shape as `routes_dispatch.rs`.

use crate::config::deposit_home_dir;
use crate::deposit::deposit;
use crate::protocol::DepositResponse;
use axum::extract::Multipart;
use axum::{http::StatusCode, Json};
use serde_json::json;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn bad_request(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": msg.into() })),
    )
}

/// Hard cap on a single deposit upload. Checked AFTER the `file` field is
/// fully buffered (axum's `Multipart` gives no reliable pre-flight
/// Content-Length per part) but strictly BEFORE `deposit::deposit` (and
/// therefore before anything ever reaches disk).
pub const MAX_DEPOSIT_BYTES: usize = 50 * 1024 * 1024;

pub async fn create(mut multipart: Multipart) -> Result<Json<DepositResponse>, ApiError> {
    let mut file_name: Option<String> = None;
    let mut bytes: Option<Vec<u8>> = None;
    let mut box_tag: Option<String> = None;
    let mut share = false;

    loop {
        let field = match multipart.next_field().await {
            Ok(Some(f)) => f,
            Ok(None) => break,
            Err(e) => return Err(bad_request(format!("invalid multipart body: {e}"))),
        };
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                let original_name = field.file_name().map(str::to_string);
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| bad_request(format!("failed to read file field: {e}")))?;
                if data.len() > MAX_DEPOSIT_BYTES {
                    return Err(bad_request(format!(
                        "file too large: {} bytes (max {MAX_DEPOSIT_BYTES})",
                        data.len()
                    )));
                }
                file_name = original_name;
                bytes = Some(data.to_vec());
            }
            "box" => {
                box_tag = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| bad_request(format!("invalid box field: {e}")))?,
                );
            }
            "share" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| bad_request(format!("invalid share field: {e}")))?;
                share = text == "true" || text == "1";
            }
            _ => {
                // Unknown field: drain it so the multipart stream doesn't stall.
                let _ = field.bytes().await;
            }
        }
    }

    let Some(original_name) = file_name.filter(|n| !n.trim().is_empty()) else {
        return Err(bad_request("missing file"));
    };
    let Some(bytes) = bytes else {
        return Err(bad_request("missing file"));
    };

    let outcome = tokio::task::spawn_blocking(move || {
        let home = deposit_home_dir();
        deposit(&home, &original_name, &bytes, box_tag.as_deref(), share)
    })
    .await
    .map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": format!("deposit task panicked: {e}") })),
        )
    })?
    .map_err(|e| bad_request(e.to_string()))?;

    Ok(Json(DepositResponse {
        file: outcome.file,
        boxes: outcome.boxes,
        held: outcome.held,
    }))
}
