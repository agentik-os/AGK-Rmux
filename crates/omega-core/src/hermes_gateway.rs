//! Hermes messaging gateway — service, PATH, and Telegram isolation.
//!
//! Hermes's gateway (`hermes gateway`) is a single background process that
//! talks to Telegram / Discord / Slack / …. It is not `omega-gateway` (the
//! OmegaOS HTTP API) and it is not the Omega Atlas Telegram bot. Two
//! getUpdates pollers on one BotFather token fight; this module refuses that.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use crate::hermes_sync::hermes_home;

const PLATFORM_ENV: &[(&str, &str)] = &[
    ("TELEGRAM_BOT_TOKEN", "telegram"),
    ("DISCORD_BOT_TOKEN", "discord"),
    ("SLACK_BOT_TOKEN", "slack"),
    ("SLACK_APP_TOKEN", "slack"),
    ("WHATSAPP_TOKEN", "whatsapp"),
    ("SIGNAL_ACCOUNT", "signal"),
    ("EMAIL_ADDRESS", "email"),
    ("MATRIX_ACCESS_TOKEN", "matrix"),
    ("TEAMS_APP_ID", "teams"),
    ("GATEWAY_RELAY_ID", "relay"),
];

pub const SYSTEMD_UNIT: &str = "hermes-gateway.service";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayService {
    Running,
    Stopped,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayReport {
    pub home: PathBuf,
    pub cli: Option<PathBuf>,
    pub platforms: Vec<String>,
    pub telegram_collision: bool,
    pub service: GatewayService,
}

impl GatewayReport {
    pub fn configured(&self) -> bool {
        !self.platforms.is_empty()
    }
}

pub fn inspect(user_home: &Path) -> GatewayReport {
    let home = hermes_home(user_home);
    let cli = find_hermes(user_home);
    let env_text = std::fs::read_to_string(home.join(".env")).unwrap_or_default();
    let platforms = configured_platforms(&env_text);
    let hermes_tg = env_value(&env_text, "TELEGRAM_BOT_TOKEN");
    let omega_tg = omega_telegram_token();
    let telegram_collision = tokens_collide(hermes_tg.as_deref(), omega_tg.as_deref());
    let service = if cli.is_some() {
        service_state(user_home)
    } else {
        GatewayService::Missing
    };
    GatewayReport {
        home,
        cli,
        platforms,
        telegram_collision,
        service,
    }
}

pub fn configured_platforms(env_text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (key, name) in PLATFORM_ENV {
        if env_value(env_text, key).is_some_and(|v| !v.is_empty()) && !out.iter().any(|n| n == name)
        {
            out.push((*name).to_string());
        }
    }
    out
}

pub fn tokens_collide(hermes_token: Option<&str>, omega_token: Option<&str>) -> bool {
    match (hermes_token, omega_token) {
        (Some(a), Some(b)) => !a.is_empty() && a == b,
        _ => false,
    }
}

pub fn env_value(env_text: &str, key: &str) -> Option<String> {
    for raw in env_text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some(rest) = line.strip_prefix(key) else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            continue;
        };
        let value = rest.trim().trim_matches('"').trim_matches('\'').trim();
        if value.is_empty() {
            return None;
        }
        return Some(value.to_string());
    }
    None
}

pub fn gateway_path(user_home: &Path) -> String {
    format!(
        "{home}/.local/bin:{home}/.hermes/bin:{home}/.hermes/hermes-agent/venv/bin:{home}/.bun/bin:{home}/.npm-global/bin:/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin",
        home = user_home.display()
    )
}

pub fn write_path_dropin(user_home: &Path) -> Result<PathBuf> {
    let dest = user_home
        .join(".config")
        .join("systemd")
        .join("user")
        .join(format!("{SYSTEMD_UNIT}.d"))
        .join("omegaos-path.conf");
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = format!(
        "# Written by omega sync / omega hermes-gateway install.\n\
         # Hermes launchd/systemd otherwise inherit a PATH without omega.\n\
         [Service]\n\
         Environment=HERMES_HOME={home}/.hermes\n\
         Environment=PATH={path}\n",
        home = user_home.display(),
        path = gateway_path(user_home)
    );
    std::fs::write(&dest, body)?;
    Ok(dest)
}

pub fn find_hermes(user_home: &Path) -> Option<PathBuf> {
    let candidates = [
        user_home.join(".local/bin/hermes"),
        user_home.join(".hermes/bin/hermes"),
        user_home.join(".hermes/hermes-agent/venv/bin/hermes"),
    ];
    for path in candidates {
        if path.is_file() {
            return Some(path);
        }
    }
    which("hermes")
}

pub fn run_hermes(user_home: &Path, args: &[&str], inherit: bool) -> Result<Output> {
    let bin = find_hermes(user_home).context("hermes CLI not found — run omega install hermes")?;
    let mut cmd = Command::new(bin);
    cmd.args(args)
        .env("HERMES_HOME", hermes_home(user_home))
        .env(
            "PATH",
            format!(
                "{}:{}",
                gateway_path(user_home),
                std::env::var("PATH").unwrap_or_default()
            ),
        );
    if inherit {
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
    }
    cmd.output().context("running hermes")
}

pub fn install_unit(user_home: &Path, force: bool) -> Result<()> {
    write_path_dropin(user_home)?;
    let mut args = vec!["gateway", "install"];
    if force {
        args.push("--force");
    }
    let output = run_hermes(user_home, &args, false)?;
    if !output.status.success() {
        anyhow::bail!(
            "hermes gateway install failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output();
    Ok(())
}

pub fn start(user_home: &Path) -> Result<()> {
    let report = inspect(user_home);
    if report.telegram_collision {
        anyhow::bail!(
            "Hermes TELEGRAM_BOT_TOKEN matches the Omega Atlas bot. \
             Create a second bot with @BotFather — two pollers on one token fight."
        );
    }
    if !report.configured() {
        anyhow::bail!("no Hermes messaging platform configured — run omega hermes-gateway setup");
    }
    hermes_ok(user_home, &["gateway", "start"])
}

pub fn stop(user_home: &Path) -> Result<()> {
    hermes_ok(user_home, &["gateway", "stop"])
}

pub fn restart(user_home: &Path) -> Result<()> {
    let report = inspect(user_home);
    if report.telegram_collision {
        anyhow::bail!(
            "Hermes TELEGRAM_BOT_TOKEN matches the Omega Atlas bot. \
             Create a second bot with @BotFather."
        );
    }
    hermes_ok(user_home, &["gateway", "restart"])
}

fn hermes_ok(user_home: &Path, args: &[&str]) -> Result<()> {
    let output = run_hermes(user_home, args, false)?;
    if !output.status.success() {
        anyhow::bail!(
            "hermes {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

fn service_state(user_home: &Path) -> GatewayService {
    if let Ok(output) = run_hermes(user_home, &["gateway", "status"], false) {
        let text = format!(
            "{} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .to_ascii_lowercase();
        if text.contains("running") || text.contains("active") {
            return GatewayService::Running;
        }
        if output.status.success()
            || text.contains("inactive")
            || text.contains("stopped")
            || text.contains("not running")
        {
            return GatewayService::Stopped;
        }
        if text.contains("not installed") || text.contains("no service") {
            return GatewayService::Missing;
        }
    }
    systemd_fallback()
}

fn systemd_fallback() -> GatewayService {
    let out = Command::new("systemctl")
        .args(["--user", "is-active", SYSTEMD_UNIT])
        .output();
    match out {
        Ok(output) => {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            match s.as_str() {
                "active" => GatewayService::Running,
                "" => GatewayService::Missing,
                _ => GatewayService::Stopped,
            }
        }
        Err(_) => GatewayService::Missing,
    }
}

fn omega_telegram_token() -> Option<String> {
    crate::monitor::OmegaTelegramConfig::read()
        .map(|cfg| cfg.bot_token.trim().to_string())
        .filter(|t| !t.is_empty())
}

fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_parser_skips_comments_and_empty_values() {
        let env = "\
# TELEGRAM_BOT_TOKEN=nope\n\
TELEGRAM_BOT_TOKEN=\n\
DISCORD_BOT_TOKEN=\"abc\"\n\
SLACK_BOT_TOKEN='xyz'\n";
        assert_eq!(env_value(env, "TELEGRAM_BOT_TOKEN"), None);
        assert_eq!(env_value(env, "DISCORD_BOT_TOKEN").as_deref(), Some("abc"));
        assert_eq!(
            configured_platforms(env),
            vec!["discord".to_string(), "slack".to_string()]
        );
    }

    #[test]
    fn inspect_without_hermes_is_idle_and_never_prints_tokens() {
        let tmp = tempfile::TempDir::new().unwrap();
        let home = tmp.path();
        std::fs::create_dir_all(home.join(".hermes")).unwrap();
        std::fs::write(
            home.join(".hermes/.env"),
            "TELEGRAM_BOT_TOKEN=111:SECRETTOKENVALUE\n",
        )
        .unwrap();
        let report = inspect(home);
        assert_eq!(report.platforms, vec!["telegram".to_string()]);
        assert!(!report.telegram_collision);
        let debug = format!("{report:?}");
        assert!(
            !debug.contains("SECRETTOKENVALUE"),
            "gateway report must not leak tokens: {debug}"
        );
    }

    #[test]
    fn atlas_and_hermes_must_not_share_a_bot_token() {
        assert!(!tokens_collide(None, Some("a")));
        assert!(!tokens_collide(Some("a"), None));
        assert!(!tokens_collide(Some("a"), Some("b")));
        assert!(tokens_collide(Some("same"), Some("same")));
    }

    #[test]
    fn path_dropin_exports_omega_and_hermes_home() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dest = write_path_dropin(tmp.path()).unwrap();
        let body = std::fs::read_to_string(dest).unwrap();
        assert!(body.contains("HERMES_HOME="));
        assert!(body.contains(".local/bin"));
        assert!(body.contains(".hermes/bin"));
    }
}
