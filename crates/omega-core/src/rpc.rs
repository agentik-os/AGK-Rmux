use crate::config::OmegaConfig;
use crate::dispatch::Dispatcher;
use crate::done::{DoneSignal, DoneStatus};
use crate::routing;
use crate::session::SessionManager;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RpcCommand {
    Ping {
        #[serde(default)]
        id: Option<String>,
    },
    GetState {
        #[serde(default)]
        id: Option<String>,
    },
    ListSessions {
        #[serde(default)]
        id: Option<String>,
    },
    CreateSession {
        name: String,
        #[serde(default)]
        cmd: Option<String>,
        #[serde(default)]
        dir: Option<String>,
        #[serde(default)]
        id: Option<String>,
    },
    KillSession {
        name: String,
        #[serde(default)]
        id: Option<String>,
    },
    Dispatch {
        project: String,
        mission: String,
        #[serde(default)]
        id: Option<String>,
    },
    Done {
        session: String,
        status: String,
        summary: String,
        #[serde(default)]
        commit: Option<String>,
        #[serde(default)]
        id: Option<String>,
    },
    Send {
        session: String,
        text: String,
        #[serde(default)]
        id: Option<String>,
    },
    Capture {
        session: String,
        #[serde(default)]
        id: Option<String>,
    },
    Classify {
        mission: String,
        #[serde(default)]
        id: Option<String>,
    },
    Shutdown {
        #[serde(default)]
        id: Option<String>,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RpcResponse {
    Pong {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
    Ok {
        command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<serde_json::Value>,
    },
    Error {
        command: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
    Event {
        event_type: String,
        data: serde_json::Value,
    },
}

pub async fn run_rpc_loop() -> Result<()> {
    let config = OmegaConfig::load()
        .context("OmegaOS RPC refused to start with invalid runtime configuration")?;
    config.ensure_dirs()?;

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut stdout = tokio::io::stdout();
    let mut line = String::new();

    let banner = RpcResponse::Event {
        event_type: "ready".to_string(),
        data: serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "protocol": "omega-rpc-v1"
        }),
    };
    write_response(&mut stdout, &banner).await?;

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<RpcCommand>(trimmed) {
            Ok(cmd) => handle_command(&config, cmd).await,
            Err(e) => RpcResponse::Error {
                command: "parse".to_string(),
                message: format!("Invalid JSON: {}", e),
                id: None,
            },
        };

        let is_shutdown =
            matches!(response, RpcResponse::Ok { ref command, .. } if command == "shutdown");
        write_response(&mut stdout, &response).await?;

        if is_shutdown {
            break;
        }
    }
    Ok(())
}

async fn write_response(stdout: &mut tokio::io::Stdout, response: &RpcResponse) -> Result<()> {
    let line = serde_json::to_string(response)?;
    stdout.write_all(line.as_bytes()).await?;
    stdout.write_all(b"\n").await?;
    stdout.flush().await?;
    Ok(())
}

async fn handle_command(config: &OmegaConfig, cmd: RpcCommand) -> RpcResponse {
    match cmd {
        RpcCommand::Ping { id } => RpcResponse::Pong { id },
        RpcCommand::GetState { id } => match get_state(config).await {
            Ok(data) => RpcResponse::Ok {
                command: "get_state".to_string(),
                id,
                data: Some(data),
            },
            Err(e) => RpcResponse::Error {
                command: "get_state".to_string(),
                message: e.to_string(),
                id,
            },
        },
        RpcCommand::ListSessions { id } => match list_sessions().await {
            Ok(data) => RpcResponse::Ok {
                command: "list_sessions".to_string(),
                id,
                data: Some(data),
            },
            Err(e) => RpcResponse::Error {
                command: "list_sessions".to_string(),
                message: e.to_string(),
                id,
            },
        },
        RpcCommand::CreateSession { name, cmd, dir, id } => {
            match create_session(&name, cmd.as_deref(), dir.as_deref()).await {
                Ok(()) => RpcResponse::Ok {
                    command: "create_session".to_string(),
                    id,
                    data: Some(serde_json::json!({ "session": name })),
                },
                Err(e) => RpcResponse::Error {
                    command: "create_session".to_string(),
                    message: e.to_string(),
                    id,
                },
            }
        }
        RpcCommand::KillSession { name, id } => match kill_session(&name).await {
            Ok(()) => RpcResponse::Ok {
                command: "kill_session".to_string(),
                id,
                data: Some(serde_json::json!({ "session": name })),
            },
            Err(e) => RpcResponse::Error {
                command: "kill_session".to_string(),
                message: e.to_string(),
                id,
            },
        },
        RpcCommand::Dispatch {
            project,
            mission,
            id,
        } => match dispatch_oracle(config, &project, &mission).await {
            Ok(oracle) => RpcResponse::Ok {
                command: "dispatch".to_string(),
                id,
                data: Some(serde_json::json!({
                    "oracle": oracle,
                    "project": project,
                    "mission": mission,
                })),
            },
            Err(e) => RpcResponse::Error {
                command: "dispatch".to_string(),
                message: e.to_string(),
                id,
            },
        },
        RpcCommand::Done {
            session,
            status,
            summary,
            commit,
            id,
        } => match write_done(config, &session, &status, &summary, commit.as_deref()) {
            Ok(()) => RpcResponse::Ok {
                command: "done".to_string(),
                id,
                data: Some(serde_json::json!({ "session": session, "status": status })),
            },
            Err(e) => RpcResponse::Error {
                command: "done".to_string(),
                message: e.to_string(),
                id,
            },
        },
        RpcCommand::Send { session, text, id } => match send_text(&session, &text).await {
            Ok(()) => RpcResponse::Ok {
                command: "send".to_string(),
                id,
                data: None,
            },
            Err(e) => RpcResponse::Error {
                command: "send".to_string(),
                message: e.to_string(),
                id,
            },
        },
        RpcCommand::Capture { session, id } => match capture_pane(&session).await {
            Ok(content) => RpcResponse::Ok {
                command: "capture".to_string(),
                id,
                data: Some(serde_json::json!({ "content": content })),
            },
            Err(e) => RpcResponse::Error {
                command: "capture".to_string(),
                message: e.to_string(),
                id,
            },
        },
        RpcCommand::Classify { mission, id } => {
            let decision = routing::classify_mission(&mission);
            RpcResponse::Ok {
                command: "classify".to_string(),
                id,
                data: Some(serde_json::json!({
                    "complexity": decision.complexity.label(),
                    "reasoning": decision.reasoning,
                    "suggested_agent": decision.suggested_agent,
                    "decompose": decision.decompose,
                    "use_team": decision.use_team,
                    "use_quality_gate": decision.use_quality_gate,
                    "recommended_agents": decision.complexity.recommended_agents(),
                    "estimated_minutes": decision.complexity.estimated_minutes(),
                })),
            }
        }
        RpcCommand::Shutdown { id } => RpcResponse::Ok {
            command: "shutdown".to_string(),
            id,
            data: None,
        },
    }
}

async fn get_state(config: &OmegaConfig) -> Result<serde_json::Value> {
    let mgr = SessionManager::connect().await?;
    let sessions = mgr.list_sessions().await?;
    Ok(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "state_dir": config.state_dir.to_string_lossy(),
        "session_count": sessions.len(),
        "agent_command": config.agent_command,
    }))
}

async fn list_sessions() -> Result<serde_json::Value> {
    let mgr = SessionManager::connect().await?;
    let sessions = mgr.list_sessions().await?;
    let items: Vec<serde_json::Value> = sessions
        .iter()
        .map(|s| {
            serde_json::json!({
                "name": s.name,
                "role": format!("{:?}", s.role),
                "project": s.project,
                "oracle_index": s.oracle_index,
            })
        })
        .collect();
    Ok(serde_json::json!({ "sessions": items }))
}

async fn create_session(name: &str, cmd: Option<&str>, dir: Option<&str>) -> Result<()> {
    let mgr = SessionManager::connect().await?;
    mgr.create_session(name, dir, cmd).await?;
    Ok(())
}

async fn kill_session(name: &str) -> Result<()> {
    let mgr = SessionManager::connect().await?;
    mgr.kill_session(name).await
}

async fn dispatch_oracle(config: &OmegaConfig, project: &str, mission: &str) -> Result<String> {
    let mgr = SessionManager::connect().await?;
    let dispatcher = Dispatcher::new(mgr, config.clone());
    dispatcher.dispatch_oracle(project, mission).await
}

fn write_done(
    config: &OmegaConfig,
    session: &str,
    status: &str,
    summary: &str,
    commit: Option<&str>,
) -> Result<()> {
    let done_status = match status {
        "done_clean" => DoneStatus::DoneClean,
        "pending" => DoneStatus::Pending,
        "failed" => DoneStatus::Failed,
        "blocked" => DoneStatus::Blocked,
        _ => anyhow::bail!("Invalid status: {}", status),
    };
    let mut signal = DoneSignal::new(session, done_status, summary);
    signal.commit = commit.map(|s| s.to_string());
    signal.write(&config.state_dir)?;
    Ok(())
}

async fn send_text(session: &str, text: &str) -> Result<()> {
    let mgr = SessionManager::connect().await?;
    mgr.send_text(session, text).await
}

async fn capture_pane(session: &str) -> Result<String> {
    let mgr = SessionManager::connect().await?;
    mgr.capture_pane(session).await
}
