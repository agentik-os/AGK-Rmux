use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GatewayConfig {
    pub bind: String,
    pub stream_interval_ms: u64,
    pub stream_lines: u32,
    pub chat_turn_timeout_ms: u64,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1:4477".into(),
            stream_interval_ms: 1000,
            stream_lines: 200,
            chat_turn_timeout_ms: 300_000,
        }
    }
}

impl GatewayConfig {
    pub fn load(dir: &Path) -> Self {
        let path = dir.join("gateway.toml");
        match std::fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text).unwrap_or_else(|e| {
                tracing::warn!("invalid {}: {e}; using defaults", path.display());
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }
}

pub fn gateway_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OMEGA_GATEWAY_DIR") {
        return PathBuf::from(dir);
    }
    dirs::home_dir()
        .expect("no home dir")
        .join(".omega")
        .join("gateway")
}

/// `$OMEGA_HOME` when set, else the real `$HOME` (`dirs::home_dir()`) — the
/// directory `omega_core::projects::discover` walks to find projects.
///
/// Threaded through everywhere project discovery is done (`routes_projects.rs`,
/// `routes_dispatch.rs`) so a hermetic test can point discovery at a tempdir
/// instead of the operator's real home, the same override shape
/// `gateway_dir()` already gives `OMEGA_GATEWAY_DIR` and `missions::ledger_dir()`
/// gives `OMEGA_STATE_DIR` — this is that same pattern's `$HOME` sibling.
pub fn home_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OMEGA_HOME") {
        return PathBuf::from(dir);
    }
    dirs::home_dir().expect("no home dir")
}

/// `$OMEGA_DEPOSIT_DIR` when set, else `~/.omega` — the root that contains
/// the deposit inbox (`<root>/inbox/`), the shared boxes (`<root>/deposit/`),
/// and the deposit config (`<root>/deposit.toml`). This mirrors the REAL
/// on-box layout (`~/.omega/inbox` + `~/.omega/deposit` + `~/.omega/deposit.toml`)
/// under a single override so a test never touches the operator's real
/// deposit data.
pub fn deposit_home_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OMEGA_DEPOSIT_DIR") {
        return PathBuf::from(dir);
    }
    dirs::home_dir().expect("no home dir").join(".omega")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_yields_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = GatewayConfig::load(dir.path());
        assert_eq!(cfg.bind, "127.0.0.1:4477");
        assert_eq!(cfg.stream_interval_ms, 1000);
        assert_eq!(cfg.stream_lines, 200);
        assert_eq!(cfg.chat_turn_timeout_ms, 300_000);
    }

    #[test]
    fn file_overrides_partial_fields() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("gateway.toml"),
            "bind = \"127.0.0.1:9999\"\n",
        )
        .unwrap();
        let cfg = GatewayConfig::load(dir.path());
        assert_eq!(cfg.bind, "127.0.0.1:9999");
        assert_eq!(cfg.stream_lines, 200);
    }

    #[test]
    fn env_overrides_gateway_dir() {
        std::env::set_var("OMEGA_GATEWAY_DIR", "/tmp/omega-gw-test");
        assert_eq!(gateway_dir(), PathBuf::from("/tmp/omega-gw-test"));
        std::env::remove_var("OMEGA_GATEWAY_DIR");
    }

    #[test]
    fn env_overrides_home_dir() {
        std::env::set_var("OMEGA_HOME", "/tmp/omega-home-test");
        assert_eq!(home_dir(), PathBuf::from("/tmp/omega-home-test"));
        std::env::remove_var("OMEGA_HOME");
    }

    #[test]
    fn env_overrides_deposit_home_dir() {
        std::env::set_var("OMEGA_DEPOSIT_DIR", "/tmp/omega-deposit-test");
        assert_eq!(deposit_home_dir(), PathBuf::from("/tmp/omega-deposit-test"));
        std::env::remove_var("OMEGA_DEPOSIT_DIR");
    }
}
