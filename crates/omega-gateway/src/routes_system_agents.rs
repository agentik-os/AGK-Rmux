//! Read-only AISB registry for the Omega app. Dispatch engines remain on
//! `/v1/agents`; this route exposes the named OmegaOS system teammates.

use crate::protocol::{SystemAgentEntry, SystemAgentsResponse};
use axum::Json;

pub async fn list() -> Json<SystemAgentsResponse> {
    let agents = omega_core::aisb_agents::AisbAgent::all()
        .iter()
        .map(|agent| {
            let definition = agent.definition();
            SystemAgentEntry {
                name: definition.name.to_string(),
                model: definition.model.name().to_string(),
                role: definition.role.to_string(),
                tagline: definition.tagline.to_string(),
                tools: definition
                    .tools
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect(),
                responsibilities: definition
                    .responsibilities
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect(),
            }
        })
        .collect();
    Json(SystemAgentsResponse { agents })
}
