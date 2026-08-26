//! Provider config: `GET /v1/config` (redacted read) and `PUT /v1/config`
//! (validated single key/value write) — wave7 task C.
//!
//! `omega_core::providers::ProvidersConfig::{load,save}` are the exact
//! in-process typed store `omega config`/`cmd_config`
//! (`crates/omega-cli/src/main.rs`) already use — no CLI subprocess for
//! this task; `ProvidersConfig::path()` resolves via
//! `omega_core::config::omega_dir()`, which honors `$OMEGA_DIR`, so a
//! hermetic test overrides that env var (same pattern
//! `tests/oracle_ops_test.rs` already uses for `timeline`/`gate`).
//!
//! SECURITY DECISION (documented in `.superpowers/sdd/progress.md`'s Task C
//! ground-truth section, load-bearing): `omega config show` LEAKS EVERY
//! PROVIDER API KEY IN PLAINTEXT — confirmed live. `GET /v1/config` NEVER
//! does that: every provider's `api_key` is redacted to `api_key_set: bool`
//! (non-empty vs empty), never the secret itself, even though the caller is
//! already device-authenticated. Rationale: a stolen/leaked device token
//! would otherwise hand over EVERY provider credential on the box in one
//! GET — a blast radius wildly disproportionate to what an app UI needs (it
//! needs to know a key IS configured, never the value). `PUT /v1/config`
//! still accepts a write to `*.api_key` (write-only — a client can SET a
//! key it already possesses, it can just never READ one back over this
//! API), validated against the SAME `(provider, field)` allowlist
//! [`apply_config_value`] carries — a byte-for-byte hand-kept twin of
//! `omega-cli::set_config_value`'s match arms (omega-cli is a BINARY crate,
//! not a library this crate can import; the same "twin" shape
//! `routes_agents.rs`/`routes_audit.rs` already carry for their own
//! CLI-mirroring logic).
//!
//! NOTE (2026-08-23): `openrouter.*` used to be readable via `GET` with no
//! writable field here, and that WAS faithful — `set_config_value` had no
//! `("openrouter", _)` arm either. It since grew one (model / api_key /
//! base_url), which left the twin genuinely one-sided: the CLI could
//! configure the provider and this API could not. The provider response and
//! write allowlist now stay symmetric, including Kimi and Antigravity.
//!
//! REVIEW-FIX ROUND (an independent adversarial reviewer found these before
//! the branch shipped — see `.superpowers/sdd/progress.md`'s Task C+D+E
//! review-fix section): `ProvidersConfig::load()` silently returns
//! `Default::default()` on ANY read/parse failure, including a genuinely
//! CORRUPT (but present) `providers.toml` — so a naive `PUT /v1/config`
//! against a hand-edited/truncated file would load-default, apply the ONE
//! field the caller asked to change, then `save()` that mostly-empty
//! struct, silently WIPING every other provider's `api_key` on the box, all
//! from a single mobile PUT with a 200 response. [`set`] therefore uses
//! [`load_config_or_refuse`] instead of `ProvidersConfig::load()` directly:
//! a present-but-unparsable file is a hard refusal (500), never a silent
//! reset. Also fixed in the same round: a `cfg.save()` I/O failure (disk
//! full, permissions) used to fold into the SAME 400 as an allowlist
//! validation failure, wrongly blaming the client for a server-side
//! problem — `set` now classifies validation errors as 400 and save/read
//! errors as 500, matching `routes_telegram.rs::toggle`'s existing split.

use crate::protocol::{
    AntigravityConfigEntry, ClaudeConfigEntry, CodexConfigEntry, ConfigResponse, ConfigSetRequest,
    GeminiConfigEntry, GlmConfigEntry, HermesConfigEntry, KimiConfigEntry, OpenRouterConfigEntry,
    PiConfigEntry,
};
use axum::http::StatusCode;
use axum::Json;
use omega_core::providers::ProvidersConfig;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn bad_request(msg: impl Into<String>) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "error": msg.into() })),
    )
}

fn internal(msg: impl std::fmt::Display) -> ApiError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": msg.to_string() })),
    )
}

/// Cap on `PUT /v1/config`'s `value` — generous enough for any real model
/// id / effort level / base URL / API key a provider issues, small enough
/// that a runaway payload can't bloat `providers.toml` unbounded (same
/// defensive posture `routes_session_org::MAX_LABEL_LEN` documents).
const MAX_CONFIG_VALUE_LEN: usize = 4096;

fn to_response(cfg: &ProvidersConfig) -> ConfigResponse {
    ConfigResponse {
        claude: ClaudeConfigEntry {
            model: cfg.claude.model.clone(),
            effort: cfg.claude.effort.clone(),
            api_key_set: !cfg.claude.api_key.is_empty(),
            dangerously_skip_permissions: cfg.claude.dangerously_skip_permissions,
        },
        codex: CodexConfigEntry {
            model: cfg.codex.model.clone(),
            api_key_set: !cfg.codex.api_key.is_empty(),
            base_url: cfg.codex.base_url.clone(),
            bypass_hook_trust: cfg.codex.bypass_hook_trust,
        },
        gemini: GeminiConfigEntry {
            model: cfg.gemini.model.clone(),
            api_key_set: !cfg.gemini.api_key.is_empty(),
        },
        antigravity: AntigravityConfigEntry {
            model: cfg.antigravity.model.clone(),
            effort: cfg.antigravity.effort.clone(),
            dangerously_skip_permissions: cfg.antigravity.dangerously_skip_permissions,
        },
        glm: GlmConfigEntry {
            model: cfg.glm.model.clone(),
            api_key_set: !cfg.glm.api_key.is_empty(),
        },
        openrouter: OpenRouterConfigEntry {
            model: cfg.openrouter.model.clone(),
            api_key_set: !cfg.openrouter.api_key.is_empty(),
            base_url: cfg.openrouter.base_url.clone(),
        },
        pi: PiConfigEntry {
            provider: cfg.pi.provider.clone(),
            model: cfg.pi.model.clone(),
            api_key_set: !cfg.pi.api_key.is_empty(),
        },
        hermes: HermesConfigEntry {
            provider: cfg.hermes.provider.clone(),
            model: cfg.hermes.model.clone(),
            api_key_set: !cfg.hermes.api_key.is_empty(),
        },
        kimi: KimiConfigEntry {
            model: cfg.kimi.model.clone(),
            api_key_set: !cfg.kimi.api_key.is_empty(),
            base_url: cfg.kimi.base_url.clone(),
            provider_type: cfg.kimi.provider_type.clone(),
        },
    }
}

/// `GET /v1/config` — see this module's doc comment. Uses
/// [`load_config_or_refuse`] rather than `ProvidersConfig::load()` directly
/// (review-fix, Minor: the final whole-branch review found the write path
/// hardened against a corrupt `providers.toml` but the READ path still
/// silently reported "nothing configured" — a box that is actually FULLY
/// configured would render as empty to the app, which is misleading in the
/// exact same way the write-path bug was dangerous). A file that exists but
/// fails to parse is now a 500, matching `set`'s posture; a missing file is
/// still the normal "nothing configured yet" 200.
pub async fn get() -> Result<Json<ConfigResponse>, ApiError> {
    let cfg = tokio::task::spawn_blocking(load_config_or_refuse)
        .await
        .map_err(|e| internal(format!("config task panicked: {e}")))?
        .map_err(internal)?;
    Ok(Json(to_response(&cfg)))
}

/// Strictly loads `providers.toml` through the same authority path as the CLI.
/// `try_load` rejects corrupt content and captures the current private-file
/// revision, so a second gateway PUT can save without being mistaken for a
/// stale writer. The old hand-written TOML parse lost that revision and every
/// PUT after the first returned 500.
fn load_config_or_refuse() -> Result<ProvidersConfig, String> {
    ProvidersConfig::try_load().map_err(|error| format!("{error:#}"))
}

/// The exact `(provider, field)` allowlist `omega-cli::set_config_value`
/// (`crates/omega-cli/src/main.rs` ~line 4473) uses — kept byte-for-byte in
/// sync by hand (see this module's doc comment for why it can't be
/// imported). An unrecognized `(provider, field)` pair is `Err`, never a
/// silent no-op.
///
/// DELIBERATE STRICTNESS BEYOND THE CLI TWIN: the CLI's own
/// `dangerously_skip_permissions` arm is `value.parse().unwrap_or(false)`
/// (a malformed value silently becomes `false`); this gateway endpoint
/// rejects a malformed boolean with a 400 instead. A network API caller
/// gets no visual confirmation of what was actually persisted the way a CLI
/// operator watching the terminal does, so a silent "your typo became
/// false" is a worse failure mode here than a loud 400 — a judgment call,
/// not a functional deviation for any value that already parses.
fn apply_config_value(cfg: &mut ProvidersConfig, key: &str, value: &str) -> Result<(), String> {
    let mut parts = key.splitn(2, '.');
    let provider = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or("missing provider")?;
    let field = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or("missing field (use provider.field)")?;
    match (provider, field) {
        ("claude", "model") => cfg.claude.model = value.to_string(),
        ("claude", "effort") => cfg.claude.effort = value.to_string(),
        ("claude", "api_key") => cfg.claude.api_key = value.to_string(),
        // NOTE (review-fix, doc-only): this is the single highest-impact
        // field this endpoint can remotely write -- setting it `true` stops
        // every future agent spawn on the box from asking permission at
        // all. In-allowlist and CLI-parity, so left writable (an operator
        // who can already reach this endpoint can already do far more via
        // `POST /v1/dispatch`), but flagged here explicitly since this
        // module's security reasoning otherwise discusses only `api_key`
        // READ blast radius, not this field's WRITE blast radius.
        ("claude", "dangerously_skip_permissions") => {
            cfg.claude.dangerously_skip_permissions = value.parse().map_err(|_| {
                "dangerously_skip_permissions must be 'true' or 'false'".to_string()
            })?;
        }
        ("codex", "model") => cfg.codex.model = value.to_string(),
        ("codex", "api_key") => cfg.codex.api_key = value.to_string(),
        ("codex", "base_url") => cfg.codex.base_url = value.to_string(),
        ("codex", "bypass_hook_trust") => {
            cfg.codex.bypass_hook_trust = value
                .parse()
                .map_err(|_| "bypass_hook_trust must be 'true' or 'false'".to_string())?;
        }
        ("gemini", "model") => cfg.gemini.model = value.to_string(),
        ("gemini", "api_key") => cfg.gemini.api_key = value.to_string(),
        ("antigravity", "model") => cfg.antigravity.model = value.to_string(),
        ("antigravity", "effort") => cfg.antigravity.effort = value.to_string(),
        ("antigravity", "dangerously_skip_permissions") => {
            cfg.antigravity.dangerously_skip_permissions = value.parse().map_err(|_| {
                "dangerously_skip_permissions must be 'true' or 'false'".to_string()
            })?;
        }
        ("openrouter", "model") => cfg.openrouter.model = value.to_string(),
        ("openrouter", "api_key") => cfg.openrouter.api_key = value.to_string(),
        ("openrouter", "base_url") => cfg.openrouter.base_url = value.to_string(),
        ("pi", "provider") => cfg.pi.provider = value.to_string(),
        ("pi", "model") => cfg.pi.model = value.to_string(),
        ("pi", "api_key") => cfg.pi.api_key = value.to_string(),
        ("glm", "model") => cfg.glm.model = value.to_string(),
        ("glm", "api_key") => cfg.glm.api_key = value.to_string(),
        ("glm", "dangerously_skip_permissions") => {
            cfg.glm.dangerously_skip_permissions = value.parse().map_err(|_| {
                "dangerously_skip_permissions must be 'true' or 'false'".to_string()
            })?;
        }
        ("codex", "ask_for_approval_never") | ("codex", "yolo") => {
            cfg.codex.ask_for_approval_never = value
                .parse()
                .map_err(|_| "ask_for_approval_never must be 'true' or 'false'".to_string())?;
        }
        ("hermes", "yolo") => {
            cfg.hermes.yolo = value
                .parse()
                .map_err(|_| "yolo must be 'true' or 'false'".to_string())?;
        }
        ("gemini", "yolo") => {
            cfg.gemini.yolo = value
                .parse()
                .map_err(|_| "yolo must be 'true' or 'false'".to_string())?;
        }
        ("kimi", "auto") | ("kimi", "yolo") => {
            cfg.kimi.auto = value
                .parse()
                .map_err(|_| "auto must be 'true' or 'false'".to_string())?;
        }
        ("pi", "approve") | ("pi", "yolo") => {
            cfg.pi.approve = value
                .parse()
                .map_err(|_| "approve must be 'true' or 'false'".to_string())?;
        }
        ("hermes", "provider") => cfg.hermes.provider = value.to_string(),
        ("hermes", "model") => cfg.hermes.model = value.to_string(),
        ("hermes", "api_key") => cfg.hermes.api_key = value.to_string(),
        ("kimi", "model") => cfg.kimi.model = value.to_string(),
        ("kimi", "api_key") => cfg.kimi.api_key = value.to_string(),
        ("kimi", "base_url") => cfg.kimi.base_url = value.to_string(),
        ("kimi", "provider_type") => cfg.kimi.provider_type = value.to_string(),
        _ => return Err(format!("unknown key: {key}")),
    }
    Ok(())
}

/// `PUT /v1/config` — validates `{key, value}` against
/// [`apply_config_value`]'s allowlist, applies it to the shared
/// `providers.toml` (which affects every session's future spawn, not just
/// the caller's), and returns the fresh redacted [`ConfigResponse`] so the
/// caller can confirm the write without ever reading a secret back.
pub async fn set(Json(req): Json<ConfigSetRequest>) -> Result<Json<ConfigResponse>, ApiError> {
    if req.key.trim().is_empty() {
        return Err(bad_request("key is required"));
    }
    if req.key.contains('\0') || req.value.contains('\0') {
        return Err(bad_request("key/value must not contain a NUL byte"));
    }
    if req.value.len() > MAX_CONFIG_VALUE_LEN {
        return Err(bad_request(format!(
            "value too long (max {MAX_CONFIG_VALUE_LEN} bytes)"
        )));
    }

    let key = req.key.clone();
    let value = req.value.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<ProvidersConfig, ApiError> {
        let mut cfg = load_config_or_refuse().map_err(internal)?;
        apply_config_value(&mut cfg, &key, &value).map_err(bad_request)?;
        cfg.save().map_err(|e| internal(e.to_string()))?;
        Ok(cfg)
    })
    .await
    .map_err(|e| internal(format!("config task panicked: {e}")))??;

    Ok(Json(to_response(&result)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_config_value_rejects_unknown_key() {
        let mut cfg = ProvidersConfig::default();
        let err = apply_config_value(&mut cfg, "claude.nonexistent", "x").unwrap_err();
        assert!(err.contains("unknown key"));
    }

    #[test]
    fn apply_config_value_rejects_missing_field() {
        let mut cfg = ProvidersConfig::default();
        let err = apply_config_value(&mut cfg, "claude", "x").unwrap_err();
        assert!(err.contains("missing field"));
    }

    #[test]
    fn apply_config_value_rejects_malformed_bool() {
        let mut cfg = ProvidersConfig::default();
        let err =
            apply_config_value(&mut cfg, "claude.dangerously_skip_permissions", "yes").unwrap_err();
        assert!(err.contains("true"));
    }

    #[test]
    fn apply_config_value_sets_known_key() {
        let mut cfg = ProvidersConfig::default();
        apply_config_value(&mut cfg, "claude.model", "opus").unwrap();
        assert_eq!(cfg.claude.model, "opus");
    }

    #[test]
    fn apply_config_value_writes_the_openrouter_fields_the_cli_writes() {
        let mut cfg = ProvidersConfig::default();
        apply_config_value(&mut cfg, "openrouter.model", "stealth/ox-alpha").unwrap();
        apply_config_value(
            &mut cfg,
            "openrouter.base_url",
            "https://openrouter.ai/api/v1",
        )
        .unwrap();
        apply_config_value(&mut cfg, "openrouter.api_key", "sk-or-v1-test").unwrap();
        assert_eq!(cfg.openrouter.model, "stealth/ox-alpha");
        assert_eq!(cfg.openrouter.base_url, "https://openrouter.ai/api/v1");
        // Write-only over this API, like every other provider: the key goes
        // in, and the response can only ever report that one is SET.
        assert!(to_response(&cfg).openrouter.api_key_set);
    }

    #[test]
    fn to_response_never_carries_api_key_bytes() {
        let mut cfg = ProvidersConfig::default();
        cfg.claude.api_key = "sk-super-secret".to_string();
        let resp = to_response(&cfg);
        assert!(resp.claude.api_key_set);
        // The struct itself has no field capable of carrying the secret —
        // this assertion is here as a documented tripwire: if a future edit
        // ever adds a raw `api_key` field to `ClaudeConfigEntry`, this test
        // still passes (it doesn't reference such a field), but the crate's
        // schema_test + adversarial review are the real backstop for that.
        let json = serde_json::to_string(&resp).unwrap();
        assert!(!json.contains("sk-super-secret"));
    }
}
