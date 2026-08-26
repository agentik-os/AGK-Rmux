//! OmegaOS MCP server wiring (Lane A interactive).
//!
//! Generates the `--mcp-config` JSON that exposes OmegaOS's own tools to a
//! spawned Claude session as **stdio MCP servers**. The three servers
//! (`session-tracker`, `decision-logger`, `patrol-status`) are small CLI
//! binaries that live under `<omega>/bin` and speak the MCP stdio protocol on
//! their stdin/stdout — exactly like any other local MCP server Claude can
//! launch.
//!
//! ## Lane discipline (why this is interactive-safe)
//!
//! `--mcp-config` is one of the **interactive-safe** flags (see
//! `agents.rs::LaunchOptions::mcp_config`). It does NOT detach the TTY: Claude
//! still renders into the attachable rmux pane, and the MCP servers are
//! ordinary child processes Claude spawns over stdio. This is **Lane A** — the
//! same lane every dispatched oracle, the `aisb-master` mirror pane, and every
//! worker/team pane run in. None of the headless-only flags
//! (`--print` / `--output-format` / `--input-format` /
//! `--include-partial-messages`) appear here; those belong to **Lane B**
//! (`omega-gateway::chat_driver` / the Bun Telegram bot), with no human attach
//! point. Wiring MCP servers therefore keeps the pane fully attachable.
//!
//! ## Path policy
//!
//! Everything resolves under the OmegaOS system root (`config::omega_dir()`):
//! binaries at `<omega>/bin/<name>`, runtime state under `<omega>/state`. We
//! use `~/.omega` (the system root), **never** `~/.aisb`.
//!
//! ## Binary existence
//!
//! The three server binaries may not be built yet. This function does NOT check
//! for them and does NOT claim they exist — it emits a config that *references*
//! `<omega>/bin/<name>`. Those binaries are to be built separately; until then
//! Claude will report the server failed to start (a clean, visible failure),
//! which is the correct behavior for a not-yet-built tool.

use crate::config::{omega_dir, OmegaConfig};
use anyhow::Result;
use serde_json::json;

/// The OmegaOS tools exposed as stdio MCP servers. Each entry is the
/// `(mcp_server_name, binary_name)` pair; the binary resolves to
/// `<omega>/bin/<binary_name>`.
const OMEGA_MCP_SERVERS: &[(&str, &str)] = &[
    ("session-tracker", "session-tracker"),
    ("decision-logger", "decision-logger"),
    ("patrol-status", "patrol-status"),
];

/// Build the `--mcp-config` JSON wiring OmegaOS's own tools as stdio MCP
/// servers for a spawned (Lane A, interactive) Claude session.
///
/// Returns the JSON **as a string**. The caller (`dispatch.rs`) writes it to a
/// file under `<omega>/state` (e.g. `<omega>/state/<session>.mcp.json`) and
/// passes that path through `LaunchOptions::mcp_config` — `--mcp-config` takes a
/// path, and `mcp_config: Option<Vec<String>>` is a list of such paths.
///
/// `session_name` namespaces each server via the `OMEGA_SESSION` env var so a
/// server knows which dispatched session it is serving (e.g. which `.done.json`
/// / decisions log to write). All paths derive from `config::omega_dir()`
/// (`~/.omega`), never `~/.aisb`.
///
/// Binaries are referenced at `<omega>/bin/<name>`; they are to be built
/// separately and are not assumed to exist (see module docs).
pub fn generate_mcp_config(config: &OmegaConfig, session_name: &str) -> Result<String> {
    // System root: prefer the config's state_dir parent so a relocated /
    // $OMEGA_DIR-overridden install stays consistent with the rest of the
    // process; fall back to the canonical resolver.
    let omega_root = config
        .state_dir
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(omega_dir);
    let bin_dir = omega_root.join("bin");
    let state_dir = &config.state_dir;

    let mut servers = serde_json::Map::new();
    for (server_name, binary_name) in OMEGA_MCP_SERVERS {
        let bin_path = bin_dir.join(binary_name);
        servers.insert(
            (*server_name).to_string(),
            json!({
                "type": "stdio",
                "command": bin_path.to_string_lossy(),
                "args": [],
                "env": {
                    // Which dispatched session this server serves.
                    "OMEGA_SESSION": session_name,
                    // Where to read/write OmegaOS state (~/.omega/state).
                    "OMEGA_STATE_DIR": state_dir.to_string_lossy(),
                }
            }),
        );
    }

    let config_json = json!({ "mcpServers": servers });
    Ok(serde_json::to_string_pretty(&config_json)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> OmegaConfig {
        // Default config resolves state_dir = <omega>/state under ~/.omega
        // (or $OMEGA_DIR / consolidated layout). We only assert on shape +
        // the ~/.omega path policy, so the default is sufficient.
        OmegaConfig::default()
    }

    #[test]
    fn mcp_config_is_valid_json() {
        let s = generate_mcp_config(&test_config(), "oracle-OmegaOS-1").unwrap();
        // Round-trips as valid JSON.
        let v: serde_json::Value = serde_json::from_str(&s).expect("valid JSON");
        let servers = v
            .get("mcpServers")
            .and_then(|m| m.as_object())
            .expect("mcpServers object");
        // All three OmegaOS tools are wired.
        assert!(servers.contains_key("session-tracker"));
        assert!(servers.contains_key("decision-logger"));
        assert!(servers.contains_key("patrol-status"));
        assert_eq!(servers.len(), 3);
        // Each server is a stdio server pointed at a binary.
        for (_name, srv) in servers {
            assert_eq!(srv.get("type").and_then(|t| t.as_str()), Some("stdio"));
            assert!(srv
                .get("command")
                .and_then(|c| c.as_str())
                .map(|c| !c.is_empty())
                .unwrap_or(false));
        }
    }

    #[test]
    fn mcp_config_uses_omega_never_aisb() {
        // Force a known root so the assertion is deterministic regardless of
        // the host's resolution order.
        let mut cfg = test_config();
        cfg.state_dir = std::path::PathBuf::from("/home/hacker/.omega/state");
        let s = generate_mcp_config(&cfg, "oracle-OmegaOS-1").unwrap();
        assert!(
            s.contains("/.omega/"),
            "config must use ~/.omega paths, got: {s}"
        );
        assert!(
            !s.contains(".aisb"),
            "config must NEVER reference ~/.aisb, got: {s}"
        );
        // Binary resolves under <omega>/bin.
        assert!(s.contains("/.omega/bin/session-tracker"));
    }

    #[test]
    fn mcp_config_namespaces_session() {
        let s = generate_mcp_config(&test_config(), "oracle-Causio-7").unwrap();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        let env = v["mcpServers"]["decision-logger"]["env"].clone();
        assert_eq!(env["OMEGA_SESSION"], "oracle-Causio-7");
    }
}
