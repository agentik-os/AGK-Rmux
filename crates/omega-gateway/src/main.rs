use clap::{Parser, Subcommand};
use omega_gateway::account_login::{self, AuthStatus};
use omega_gateway::accounts::AccountStore;
use omega_gateway::auth::{DeviceStore, PairingCode};
use omega_gateway::chat_store::ChatStore;
use omega_gateway::config::{gateway_dir, GatewayConfig};
use omega_gateway::protocol::AccountKind;
use omega_gateway::server::{build_router, AppState};

#[derive(Parser)]
#[command(name = "omega-gatewayd", about = "OmegaOS gateway daemon")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run the gateway server (default)
    Serve,
    /// Print a one-time pairing code + QR (valid 5 minutes)
    Pair,
    /// Print the wire-protocol JSON Schema (for TS type generation)
    Schema,
    /// List paired devices (id, name, created_at, revoked)
    Devices,
    /// List chats (id, title, agent, updated_at)
    Chats,
    /// Revoke a device by id (its token stops verifying immediately)
    Revoke { device_id: String },
    /// List account slots (slug, label, kind, default, live auth status)
    Accounts,
    /// Create a new isolated credential slot (kind: claude|codex)
    AccountAdd {
        slug: String,
        label: String,
        kind: String,
    },
}

/// Parses the CLI's free-text `kind` argument into an [`AccountKind`].
/// Pure and case-insensitive so `Claude`/`CODEX` are accepted alongside the
/// documented lowercase form; anything else is `None` (the caller reports
/// the exact invalid value and exits non-zero).
fn parse_account_kind(s: &str) -> Option<AccountKind> {
    match s.to_lowercase().as_str() {
        "claude" => Some(AccountKind::Claude),
        "codex" => Some(AccountKind::Codex),
        _ => None,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let dir = gateway_dir();
    match cli.command.unwrap_or(Command::Serve) {
        Command::Pair => {
            let pc = PairingCode::create(&dir, 300)?;
            let host = hostname_or_default();
            let payload = pairing_deep_link(&host, &pc.code);
            match directory_url() {
                // Simple path: the operator types ONE code into the signed-in
                // app, no host, no URL, no QR to scan from a headless VPS.
                Some(base) => {
                    println!("Pairing code: {}  (valid 5 minutes)", pc.code);
                    println!("Type it into the Omega App, signed in. Waiting…");
                    let box_id = omega_gateway::routes_box::read_or_create_box_id(&dir)?;
                    let cfg = GatewayConfig::load(&dir);
                    let port = cfg
                        .bind
                        .rsplit(':')
                        .next()
                        .and_then(|p| p.parse::<u16>().ok())
                        .unwrap_or(4477);
                    let body = redeem_request(&pc.code, &box_id, &host, port, None);
                    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
                    if await_claim(&base, &body, deadline).await {
                        println!("Paired. This machine is now on your account.");
                    } else {
                        println!("Pairing code expired before it was claimed. Run `pair` again.");
                        std::process::exit(1);
                    }
                }
                None => {
                    qr2term::print_qr(&payload).ok();
                    println!("Pairing code: {}  (valid 5 minutes)", pc.code);
                    println!("Payload: {payload}");
                }
            }
        }
        Command::Schema => println!("{}", omega_gateway::protocol::schema_json()),
        Command::Devices => {
            let store = DeviceStore::open(&dir);
            let devices = store.list();
            if devices.is_empty() {
                println!("no paired devices");
            } else {
                println!("{:<18} {:<20} {:<28} REVOKED", "ID", "NAME", "CREATED_AT");
                for d in devices {
                    println!(
                        "{:<18} {:<20} {:<28} {}",
                        d.id, d.name, d.created_at, d.revoked
                    );
                }
            }
        }
        Command::Chats => {
            let store = ChatStore::open(&dir);
            let chats = store.list();
            if chats.is_empty() {
                println!("no chats");
            } else {
                println!("{:<18} {:<24} {:<8} UPDATED_AT", "ID", "TITLE", "AGENT");
                for c in chats {
                    let agent = match c.agent {
                        omega_gateway::protocol::ChatAgent::Claude => "claude",
                        omega_gateway::protocol::ChatAgent::Codex => "codex",
                    };
                    let title = c.title.as_deref().unwrap_or("-");
                    println!("{:<18} {:<24} {:<8} {}", c.id, title, agent, c.updated_at);
                }
            }
        }
        Command::Revoke { device_id } => {
            let mut store = DeviceStore::open(&dir);
            if store.revoke(&device_id) {
                println!("revoked device {device_id}");
            } else {
                eprintln!("no such device: {device_id}");
                std::process::exit(1);
            }
        }
        Command::Accounts => {
            let store = AccountStore::open(&dir);
            let accounts = store.list();
            if accounts.is_empty() {
                println!("no accounts");
            } else {
                println!(
                    "{:<16} {:<20} {:<8} {:<8} STATUS",
                    "SLUG", "LABEL", "KIND", "DEFAULT"
                );
                for a in accounts {
                    let slot = store.slot_dir(&a.slug);
                    let status = match account_login::status(&a, &slot) {
                        AuthStatus::LoggedIn => "logged in",
                        AuthStatus::LoggedOut => "logged out",
                        AuthStatus::Unknown => "unknown",
                    };
                    let kind = match a.kind {
                        AccountKind::Claude => "claude",
                        AccountKind::Codex => "codex",
                    };
                    println!(
                        "{:<16} {:<20} {:<8} {:<8} {}",
                        a.slug, a.label, kind, a.is_default, status
                    );
                }
            }
        }
        Command::AccountAdd { slug, label, kind } => {
            let Some(kind) = parse_account_kind(&kind) else {
                eprintln!("unknown account kind: {kind:?} (expected claude|codex)");
                std::process::exit(1);
            };
            let store = AccountStore::open(&dir);
            match store.create_slot(&slug, &label, kind) {
                Ok(account) => {
                    let slot_dir = store.slot_dir(&account.slug);
                    println!("created account slot: {} ({})", account.slug, account.label);
                    match kind {
                        AccountKind::Claude => {
                            println!("next step, log this slot in — either:");
                            println!(
                                "  run: CLAUDE_CONFIG_DIR={} claude auth login",
                                slot_dir.display()
                            );
                            println!("  or:  complete login via the app's account pairing flow");
                        }
                        AccountKind::Codex => {
                            println!("next step, log this slot in — either:");
                            println!("  run: CODEX_HOME={} codex login", slot_dir.display());
                            println!(
                                "  or:  POST an API key to /v1/accounts/{}/apikey via the app",
                                account.slug
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("failed to create account: {e}");
                    std::process::exit(1);
                }
            }
        }
        Command::Serve => {
            let cfg = GatewayConfig::load(&dir);
            let bind = cfg.bind.clone();
            let state = AppState::new(dir, cfg.clone());
            omega_gateway::events::spawn_background_emitters(state.events.clone(), &cfg);
            let listener = tokio::net::TcpListener::bind(&bind).await?;
            state.relay.start(listener.local_addr()?).await?;
            let app = build_router(state);
            tracing::info!("omega-gateway listening on {bind}");
            axum::serve(listener, app).await?;
        }
    }
    Ok(())
}

/// The Convex directory this box registers into when the operator has pointed
/// it at one. Absent = the pre-existing behaviour: print the code and let the
/// app take the host by hand.
fn directory_url() -> Option<String> {
    std::env::var("OMEGA_DIRECTORY_URL")
        .ok()
        .map(|s| s.trim().trim_end_matches('/').to_string())
        .filter(|s| !s.is_empty())
}

/// SHA-256 hex of the pairing code. The directory stores and compares only
/// this, so the code itself never leaves the machine and a directory dump
/// never yields a usable one.
fn code_hash(code: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The Convex HTTP mutation envelope for `boxPairing:redeem`. Built as data so
/// the shape is testable without a directory to talk to.
fn redeem_request(
    code: &str,
    box_id: &str,
    label: &str,
    gateway_port: u16,
    tailscale_host: Option<&str>,
) -> serde_json::Value {
    let mut args = serde_json::json!({
        "codeHash": code_hash(code),
        "boxId": box_id,
        "label": label,
        "gatewayPort": gateway_port,
    });
    if let Some(host) = tailscale_host {
        args["tailscaleHost"] = serde_json::Value::String(host.to_string());
    }
    serde_json::json!({ "path": "boxPairing:redeem", "args": args, "format": "json" })
}

/// Polls the directory until the operator has typed the code into the signed-in
/// app. The code is minted locally and never uploaded, so this call only ever
/// succeeds AFTER an authenticated owner claimed it — that ordering is what
/// keeps the flow free of an unauthenticated mint endpoint.
async fn await_claim(base: &str, body: &serde_json::Value, deadline: std::time::Instant) -> bool {
    let client = reqwest::Client::new();
    let endpoint = format!("{base}/api/mutation");
    while std::time::Instant::now() < deadline {
        if let Ok(res) = client.post(&endpoint).json(body).send().await {
            if let Ok(payload) = res.json::<serde_json::Value>().await {
                if payload.get("status").and_then(|s| s.as_str()) == Some("success") {
                    return true;
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    false
}

fn hostname_or_default() -> String {
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".into())
}

/// Builds the explicit mobile-app pairing route printed in the terminal QR.
/// Pairing remains user-confirmed by the app; the link only carries the
/// one-time gateway URL and code values needed to prefill its pairing form.
fn pairing_deep_link(host: &str, code: &str) -> String {
    let gateway_url = gateway_url(host);
    format!(
        "omegaapp://pair?host={}&code={}",
        percent_encode_query_component(&gateway_url),
        percent_encode_query_component(code)
    )
}

/// Turns the hostname printed by `omega-gatewayd pair` into the HTTP endpoint
/// consumed by the mobile client. The daemon's default port is 4477; preserve
/// an explicitly supplied port and bracket bare IPv6 literals for URL syntax.
fn gateway_url(host: &str) -> String {
    let host = host.trim();
    if host.parse::<std::net::Ipv6Addr>().is_ok() {
        format!("http://[{host}]:4477")
    } else if host.starts_with('[')
        || host
            .rsplit_once(':')
            .is_some_and(|(_, port)| port.parse::<u16>().is_ok())
    {
        format!("http://{host}")
    } else {
        format!("http://{host}:4477")
    }
}

/// RFC 3986 query-component encoding for a deep-link parameter.
fn percent_encode_query_component(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push_str(&format!("{byte:02X}"));
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_hash_is_sha256_hex_and_never_the_code() {
        // Known vector: SHA-256("abc").
        assert_eq!(
            code_hash("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let hashed = code_hash("a1b2c3d4");
        assert_eq!(hashed.len(), 64);
        assert!(hashed
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
        assert!(!hashed.contains("a1b2c3d4"));
    }

    #[test]
    fn redeem_request_sends_the_hash_and_omits_an_absent_tailscale_host() {
        let body = redeem_request("a1b2c3d4", "station-vps", "Station VPS", 4477, None);
        assert_eq!(body["path"], "boxPairing:redeem");
        assert_eq!(body["format"], "json");
        assert_eq!(body["args"]["codeHash"], code_hash("a1b2c3d4"));
        assert_eq!(body["args"]["boxId"], "station-vps");
        assert_eq!(body["args"]["gatewayPort"], 4477);
        assert!(body["args"].get("tailscaleHost").is_none());
        // The plaintext code must never appear anywhere in the payload.
        assert!(!body.to_string().contains("a1b2c3d4"));
    }

    #[test]
    fn redeem_request_carries_a_tailscale_host_when_there_is_one() {
        let body = redeem_request(
            "a1b2c3d4",
            "b",
            "L",
            4477,
            Some("station.tail64d114.ts.net"),
        );
        assert_eq!(body["args"]["tailscaleHost"], "station.tail64d114.ts.net");
    }

    #[test]
    fn parse_account_kind_accepts_claude_and_codex_case_insensitively() {
        assert_eq!(parse_account_kind("claude"), Some(AccountKind::Claude));
        assert_eq!(parse_account_kind("Claude"), Some(AccountKind::Claude));
        assert_eq!(parse_account_kind("codex"), Some(AccountKind::Codex));
        assert_eq!(parse_account_kind("CODEX"), Some(AccountKind::Codex));
    }

    #[test]
    fn parse_account_kind_rejects_unknown() {
        assert_eq!(parse_account_kind("bogus"), None);
        assert_eq!(parse_account_kind(""), None);
    }

    #[test]
    fn cli_parses_accounts_subcommand() {
        let cli = Cli::try_parse_from(["omega-gatewayd", "accounts"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Accounts)));
    }

    #[test]
    fn cli_parses_account_add_subcommand_with_positional_args() {
        let cli = Cli::try_parse_from([
            "omega-gatewayd",
            "account-add",
            "work-a",
            "Work A",
            "claude",
        ])
        .unwrap();
        match cli.command {
            Some(Command::AccountAdd { slug, label, kind }) => {
                assert_eq!(slug, "work-a");
                assert_eq!(label, "Work A");
                assert_eq!(kind, "claude");
            }
            other => panic!("expected AccountAdd, got {other:?}"),
        }
    }

    #[test]
    fn cli_rejects_account_add_missing_args() {
        assert!(Cli::try_parse_from(["omega-gatewayd", "account-add", "only-slug"]).is_err());
    }

    #[test]
    fn pairing_deep_link_targets_the_omega_app_scheme() {
        assert_eq!(
            pairing_deep_link("gateway.tailnet:4477", "8bebbdbf"),
            "omegaapp://pair?host=http%3A%2F%2Fgateway.tailnet%3A4477&code=8bebbdbf"
        );
        assert_eq!(
            pairing_deep_link("gateway.tailnet", "8bebbdbf"),
            "omegaapp://pair?host=http%3A%2F%2Fgateway.tailnet%3A4477&code=8bebbdbf"
        );
    }

    #[test]
    fn pairing_deep_link_brackets_and_encodes_ipv6_hosts() {
        assert_eq!(
            pairing_deep_link("2001:db8::1", "8bebbdbf"),
            "omegaapp://pair?host=http%3A%2F%2F%5B2001%3Adb8%3A%3A1%5D%3A4477&code=8bebbdbf"
        );
    }

    #[test]
    fn account_add_creates_slot_via_store() {
        // Exercises the same store call the CLI handler makes end to end
        // (no real login, no process spawn) — the CLI branch itself is a
        // thin println wrapper around AccountStore::create_slot, already
        // covered exhaustively in accounts.rs.
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        let kind = parse_account_kind("claude").unwrap();
        let account = store.create_slot("work-a", "Work A", kind).unwrap();
        assert_eq!(account.slug, "work-a");
        assert_eq!(account.kind, AccountKind::Claude);
        assert!(account.is_default);
    }
}
