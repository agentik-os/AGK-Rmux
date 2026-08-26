//! Telegram bridge control: `GET /v1/telegram/status`,
//! `POST /v1/telegram/enable`, `POST /v1/telegram/disable` — wave7 task D.
//! This is what makes Telegram OPTIONAL from the app: a client can check
//! whether the bridge is configured/enabled and flip it without shelling
//! into the box.
//!
//! `omega_core::monitor::OmegaTelegramConfig::{read,write}` is a pure
//! in-process typed store — the exact read/write `TelegramAction::Status`/
//! `Enable`/`Disable` (`crates/omega-cli/src/main.rs::cmd_telegram`)
//! already use, so this module mirrors that logic in-process rather than
//! shelling to `omega telegram ...` (there is no CLI subprocess here, so
//! "fake-bin tested" from the task brief doesn't apply — these routes are
//! tested against a scratch `$HOME` instead, see this crate's
//! `tests/telegram_test.rs`).
//!
//! PATH CAVEAT (documented in `.superpowers/sdd/progress.md`'s Task D
//! ground-truth section): `OmegaTelegramConfig::path()` hardcodes
//! `dirs::home_dir()` — it does NOT honor `$OMEGA_DIR`/`$OMEGA_HOME` the
//! way `ProvidersConfig::path()` (Task C) does. A hermetic test therefore
//! overrides the `$HOME` env var itself (same pattern wave6 used for
//! `UsageSnapshot`), never `$OMEGA_DIR`.
//!
//! SECURITY DECISION (same posture Task C takes for `api_key`):
//! `OmegaTelegramConfig` also holds a plaintext secret (`bot_token`) that
//! must never round-trip to a GET caller — [`TelegramStatusResponse`]
//! redacts it to `bot_token_set: bool`.
//!
//! LIVE-VERIFY CAVEAT (binding on whoever runs it, not just this file):
//! `status` is safe to live-check against the operator's real
//! `~/.omega/telegram.toml` (read-only). `enable`/`disable` MUST NEVER be
//! live-checked against the real config — either would flip the operator's
//! actual Telegram bridge. Both are fake-`$HOME`-only in this crate's own
//! test suite; live-verify only reads `status`.

use crate::protocol::{TelegramStatusResponse, TelegramToggleResponse};
use axum::http::StatusCode;
use axum::Json;
use omega_core::monitor::OmegaTelegramConfig;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn not_found(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": msg.into() })),
    )
}

fn internal(msg: impl std::fmt::Display) -> ApiError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": msg.to_string() })),
    )
}

fn to_status_response(cfg: Option<OmegaTelegramConfig>) -> TelegramStatusResponse {
    match cfg {
        Some(c) => TelegramStatusResponse {
            configured: true,
            enabled: Some(c.enabled),
            bot_token_set: Some(!c.bot_token.is_empty()),
            chat_id: Some(c.chat_id),
            relay_session: Some(c.relay_session),
            label: Some(c.label),
            allow_user_ids_count: Some(c.allow_user_ids.len()),
        },
        None => TelegramStatusResponse {
            configured: false,
            enabled: None,
            bot_token_set: None,
            chat_id: None,
            relay_session: None,
            label: None,
            allow_user_ids_count: None,
        },
    }
}

/// `GET /v1/telegram/status` — read-only, safe to call against the real
/// operator config. `configured: false` (no `~/.omega/telegram.toml` yet)
/// is a NORMAL response, never an error — mirrors `TelegramAction::Status`'s
/// own CLI arm, which renders "Not configured." rather than failing.
pub async fn status() -> Json<TelegramStatusResponse> {
    let cfg = tokio::task::spawn_blocking(OmegaTelegramConfig::read)
        .await
        .unwrap_or(None);
    Json(to_status_response(cfg))
}

/// Shared by [`enable`] and [`disable`]: read the config, flip `enabled` to
/// `target`, write it back — the exact in-process sequence
/// `TelegramAction::Enable`/`Disable` already run. 404 when unconfigured
/// (mirrors those CLI arms' own bail: "Not configured. Run: omega telegram
/// setup …") rather than fabricating a config that was never set up.
async fn toggle(target: bool) -> Result<Json<TelegramToggleResponse>, ApiError> {
    let result = tokio::task::spawn_blocking(move || -> Result<Option<bool>, String> {
        let Some(mut cfg) = OmegaTelegramConfig::read() else {
            return Ok(None);
        };
        cfg.enabled = target;
        cfg.write().map_err(|e| e.to_string())?;
        Ok(Some(cfg.enabled))
    })
    .await
    .map_err(|e| internal(format!("telegram toggle task panicked: {e}")))?;

    match result {
        Ok(Some(enabled)) => Ok(Json(TelegramToggleResponse { enabled })),
        Ok(None) => Err(not_found(
            "telegram bridge is not configured (run: omega telegram setup …)",
        )),
        Err(msg) => Err(internal(msg)),
    }
}

/// `POST /v1/telegram/enable` — NEVER exercised against the real bridge in
/// this crate's tests or in live-verify (see this module's doc comment).
pub async fn enable() -> Result<Json<TelegramToggleResponse>, ApiError> {
    toggle(true).await
}

/// `POST /v1/telegram/disable` — NEVER exercised against the real bridge in
/// this crate's tests or in live-verify (see this module's doc comment).
pub async fn disable() -> Result<Json<TelegramToggleResponse>, ApiError> {
    toggle(false).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_status_response_none_is_unconfigured_not_an_error() {
        let resp = to_status_response(None);
        assert!(!resp.configured);
        assert!(resp.enabled.is_none());
        assert!(resp.bot_token_set.is_none());
    }

    #[test]
    fn to_status_response_redacts_bot_token() {
        let cfg = OmegaTelegramConfig {
            bot_token: "123:ABC-super-secret".to_string(),
            chat_id: 42,
            allow_user_ids: vec![42],
            relay_session: "aisb-master".to_string(),
            label: "prod".to_string(),
            enabled: true,
        };
        let resp = to_status_response(Some(cfg));
        assert!(resp.configured);
        assert_eq!(resp.bot_token_set, Some(true));
        let json = serde_json::to_string(&resp).unwrap();
        assert!(!json.contains("123:ABC-super-secret"));
        assert_eq!(resp.chat_id, Some(42));
        assert_eq!(resp.allow_user_ids_count, Some(1));
    }
}
