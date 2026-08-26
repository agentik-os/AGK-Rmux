//! `GET /v1/projects` — the discovered-project list, from
//! `omega_core::projects::discover(&home)`. Pure filesystem walk (no
//! network), kept off the async runtime thread via `spawn_blocking`, same
//! idiom `routes_skills::list` / `routes_agents::list` use. The response is
//! already best-first sorted by `discover`, which is all a mobile client
//! needs — no `?q=`/`?limit=` narrowing here, unlike `/v1/skills`, since a
//! project list is small enough (this box discovers dozens, not hundreds)
//! to ship whole.
//!
//! `home` is resolved via `config::home_dir()` (respects `$OMEGA_HOME`),
//! not `dirs::home_dir()` directly, so a hermetic test can point discovery
//! at a tempdir — the same override `routes_dispatch.rs`'s project
//! validation step needs and shares (Task 8).

use crate::protocol::{ProjectEntry, ProjectsResponse};
use axum::Json;

pub async fn list() -> Json<ProjectsResponse> {
    let response = tokio::task::spawn_blocking(|| {
        let home = crate::config::home_dir();
        let projects = omega_core::projects::discover(&home)
            .into_iter()
            .map(|p| ProjectEntry {
                name: p.name,
                container: p.container,
                stack: p.stack,
                last_active_days: p.last_active_days,
            })
            .collect();
        ProjectsResponse { projects }
    })
    .await
    .unwrap_or(ProjectsResponse { projects: vec![] });
    Json(response)
}
