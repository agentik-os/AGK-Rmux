//! Dynamic operative-system catalog for the Omega app.

use crate::protocol::{OsEntry, OsResponse};
use axum::Json;
use omega_core::os_products::OsGroup;

fn group_label(group: OsGroup) -> &'static str {
    match group {
        OsGroup::Personal => "Personal",
        OsGroup::BuildChain => "Build chain",
        OsGroup::Growth => "Growth",
        OsGroup::Systems => "Systems",
    }
}

pub async fn list() -> Json<OsResponse> {
    let os = tokio::task::spawn_blocking(|| {
        omega_core::os_products::list_os_entries()
            .into_iter()
            .map(|entry| {
                let slug = entry.product.slug.to_string();
                OsEntry {
                    name: entry.product.name.to_string(),
                    category: group_label(entry.product.group).to_string(),
                    status: entry.status_label().to_string(),
                    path: format!("OS/{slug}"),
                    bot: if entry.bot_linked {
                        format!("os-{slug}")
                    } else {
                        String::new()
                    },
                    slug,
                }
            })
            .collect()
    })
    .await
    .unwrap_or_default();
    Json(OsResponse { os })
}
