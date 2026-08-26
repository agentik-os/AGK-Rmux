//! `GET /v1/rules` — the OmegaOS doctrine (Laws + Rules) from
//! `omega_core::rules`. Pure, synchronous, side-effect-free data (no daemon,
//! no network, no filesystem) — same `spawn_blocking` idiom
//! `routes_missions::list` uses for its own blocking call, kept off the
//! async runtime thread even though this call is CPU-only, for consistency.
//!
//! Sources laws()/operational_rules() directly (the SSOT split already
//! defined in rules.rs) rather than calling `all_rules()` and filtering by
//! `kind` here — the split is not re-derived.

use crate::protocol::{LawEntry, RuleEntry, RulesResponse};
use axum::Json;

pub async fn list() -> Json<RulesResponse> {
    let response = tokio::task::spawn_blocking(|| {
        let laws = omega_core::rules::laws()
            .into_iter()
            .map(|r| LawEntry {
                id: r.id.to_string(),
                title: r.title.to_string(),
                category: format!("{:?}", r.category),
            })
            .collect();
        let rules = omega_core::rules::operational_rules()
            .into_iter()
            .map(|r| RuleEntry {
                id: r.id.to_string(),
                title: r.title.to_string(),
                category: format!("{:?}", r.category),
                added_at: r.added_at.to_string(),
            })
            .collect();
        RulesResponse { laws, rules }
    })
    .await
    .unwrap_or(RulesResponse {
        laws: vec![],
        rules: vec![],
    });
    Json(response)
}
