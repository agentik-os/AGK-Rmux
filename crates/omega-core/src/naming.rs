use crate::agents::Agent;
use crate::session::SessionManager;
use anyhow::Result;

/// Generate a unique session name for a given agent.
/// Format: `{agent}-{N}` where N is the next available index.
pub async fn auto_name(agent: Agent, mgr: &SessionManager) -> Result<String> {
    let existing = mgr.list_sessions().await?;
    let prefix = agent.name();

    let max_index = existing
        .iter()
        .filter_map(|s| {
            let name = &s.name;
            if let Some(rest) = name.strip_prefix(&format!("{}-", prefix)) {
                rest.parse::<u32>().ok()
            } else {
                None
            }
        })
        .max()
        .unwrap_or(0);

    Ok(format!("{}-{}", prefix, max_index + 1))
}

/// Auto-generate a name for a worker under a project.
/// Format: `{Project}-worker-{N}`.
pub async fn auto_worker_name(project: &str, mgr: &SessionManager) -> Result<String> {
    let existing = mgr.list_sessions().await?;
    let prefix = format!("{}-worker-", project);

    let max_index = existing
        .iter()
        .filter_map(|s| {
            s.name
                .strip_prefix(&prefix)
                .and_then(|rest| rest.parse::<u32>().ok())
        })
        .max()
        .unwrap_or(0);

    Ok(format!("{}{}", prefix, max_index + 1))
}
