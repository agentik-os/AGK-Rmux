# Omega Gateway V1 Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the `omega-gateway` daemon foundation: an authenticated HTTP/WebSocket API on each OmegaOS box with device pairing, session listing, and live session streaming.

**Architecture:** New Rust crate `crates/omega-gateway` in the OmegaOS workspace. Axum server bound to loopback by default. Device tokens issued via a one-time pairing code (QR in terminal), stored hashed. Sessions are read by shelling out to the `rmux` binary (path overridable for tests), streamed as rendered-pane snapshots (the proven `omega stream` mechanism: capture, diff, push frame on change, never die on error).

**Tech Stack:** Rust, axum 0.8 (ws), tokio 1.48 (workspace), serde/serde_json (workspace), schemars 0.8, sha2 0.10, rand 0.8, qr2term 0.3, clap 4.5 (workspace). Dev-deps: reqwest 0.12 (workspace style), tokio-tungstenite 0.24, tempfile 3.

**Plan roadmap (this is Plan 1 of 4):**
1. **This plan** — gateway foundation: crate, config, pairing, auth, sessions list + stream, schema export, systemd + installer wiring.
2. Gateway part 2 — chat (headless Claude/Codex sessions), mission dispatch, progress ledger, alerts/push events.
3. Agentik cloud — Convex + Clerk box registry, APNs push relay.
4. omega-app monorepo — Electron macOS + Expo iOS, shared React core (Chat, Missions, Inbox, Fleet).

## Global Constraints

- Repo: `~/Station/SideBusiness/OmegaOS`. Sync (`git fetch && git rebase origin/main`) before starting; commit only your own files, never `git add -A` (concurrent sessions).
- Use workspace deps (`tokio.workspace = true`, etc.) where they exist; exact new versions: `axum = { version = "0.8", features = ["ws"] }`, `schemars = "0.8"`, `rand = "0.8"`, `qr2term = "0.3"`, dev `tokio-tungstenite = "0.24"`, dev `tempfile = "3"`, dev `reqwest = { version = "0.12", features = ["json"], default-features = false }` plus `rustls-tls`.
- Default bind: `127.0.0.1:4477`. Never bind non-loopback by default.
- Tokens: 32 random bytes hex-encoded; only the SHA-256 of a token is persisted.
- Gateway state dir: `~/.omega/gateway/`, overridable with env `OMEGA_GATEWAY_DIR` (tests rely on this).
- rmux binary: `$HOME/.local/bin/rmux`, overridable with env `OMEGA_RMUX_BIN` (tests rely on this).
- Stream loops NEVER exit on error (R-STREAM): errors become frames, the loop continues.
- All code, comments, commits in English. Run `cargo test -p omega-gateway` after each task; run `cargo clippy -p omega-gateway -- -D warnings` before each commit.

---

### Task 1: Crate scaffold + health endpoint

**Files:**
- Create: `crates/omega-gateway/Cargo.toml`
- Create: `crates/omega-gateway/src/main.rs`
- Create: `crates/omega-gateway/src/server.rs`
- Create: `crates/omega-gateway/tests/health_test.rs`
- Modify: `Cargo.toml` (workspace members — add `"crates/omega-gateway"` to the `members` array)

**Interfaces:**
- Produces: `server::build_router(state: AppState) -> axum::Router`, `server::AppState { pub dir: std::path::PathBuf }`, binary name `omega-gatewayd`, endpoint `GET /v1/health` → `200 {"ok":true,"version":"<crate version>"}`.

- [ ] **Step 1: Write the failing test**

`crates/omega-gateway/tests/health_test.rs`:
```rust
use omega_gateway::server::{build_router, AppState};

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

#[tokio::test]
async fn health_returns_ok_and_version() {
    let dir = tempfile::tempdir().unwrap();
    let app = build_router(AppState { dir: dir.path().to_path_buf() });
    let base = spawn(app).await;
    let body: serde_json::Value = reqwest::get(format!("{base}/v1/health"))
        .await.unwrap().json().await.unwrap();
    assert_eq!(body["ok"], true);
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
}
```

- [ ] **Step 2: Create the crate so the test can fail on behavior, not on parsing**

`crates/omega-gateway/Cargo.toml`:
```toml
[package]
name = "omega-gateway"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "omega-gatewayd"
path = "src/main.rs"

[lib]
path = "src/lib.rs"

[dependencies]
axum = { version = "0.8", features = ["ws"] }
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
clap.workspace = true
anyhow.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
dirs.workspace = true
toml.workspace = true
sha2 = "0.10"
rand = "0.8"
schemars = "0.8"
qr2term = "0.3"
chrono.workspace = true

[dev-dependencies]
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
tokio-tungstenite = "0.24"
tempfile = "3"
futures-util = "0.3"
```

`crates/omega-gateway/src/lib.rs`:
```rust
pub mod server;
```

`crates/omega-gateway/src/server.rs` (stub that compiles but fails the test):
```rust
use axum::Router;
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppState {
    pub dir: PathBuf,
}

pub fn build_router(_state: AppState) -> Router {
    Router::new()
}
```

`crates/omega-gateway/src/main.rs`:
```rust
fn main() {
    println!("omega-gatewayd: not wired yet");
}
```

Add `"crates/omega-gateway"` to `members` in the root `Cargo.toml`.

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p omega-gateway --test health_test`
Expected: FAIL (404 / JSON decode error, since the route does not exist yet)

- [ ] **Step 4: Implement the health route**

Replace `build_router` in `crates/omega-gateway/src/server.rs`:
```rust
use axum::{routing::get, Json, Router};
use serde_json::json;
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppState {
    pub dir: PathBuf,
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "ok": true, "version": env!("CARGO_PKG_VERSION") }))
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/v1/health", get(health))
        .with_state(state)
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p omega-gateway --test health_test`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock crates/omega-gateway
git commit -m "feat(gateway): scaffold omega-gateway crate with /v1/health"
```

---

### Task 2: Config loading

**Files:**
- Create: `crates/omega-gateway/src/config.rs`
- Modify: `crates/omega-gateway/src/lib.rs` (add `pub mod config;`)

**Interfaces:**
- Produces: `config::GatewayConfig { pub bind: String, pub stream_interval_ms: u64, pub stream_lines: u32 }`, `config::GatewayConfig::load(dir: &Path) -> GatewayConfig` (reads `<dir>/gateway.toml`, falls back to defaults), `config::gateway_dir() -> PathBuf` (env `OMEGA_GATEWAY_DIR`, else `~/.omega/gateway`).

- [ ] **Step 1: Write the failing tests (unit tests in the module)**

`crates/omega-gateway/src/config.rs`:
```rust
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GatewayConfig {
    pub bind: String,
    pub stream_interval_ms: u64,
    pub stream_lines: u32,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self { bind: "127.0.0.1:4477".into(), stream_interval_ms: 1000, stream_lines: 200 }
    }
}

impl GatewayConfig {
    pub fn load(dir: &Path) -> Self {
        unimplemented!()
    }
}

pub fn gateway_dir() -> PathBuf {
    unimplemented!()
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
    }

    #[test]
    fn file_overrides_partial_fields() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("gateway.toml"), "bind = \"127.0.0.1:9999\"\n").unwrap();
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
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p omega-gateway config::`
Expected: FAIL (panics on `unimplemented!()`)

- [ ] **Step 3: Implement**

Replace the two `unimplemented!()` bodies:
```rust
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
    dirs::home_dir().expect("no home dir").join(".omega").join("gateway")
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p omega-gateway config::`
Expected: PASS (3 tests)

- [ ] **Step 5: Commit**

```bash
git add crates/omega-gateway/src/config.rs crates/omega-gateway/src/lib.rs
git commit -m "feat(gateway): gateway.toml config with defaults and OMEGA_GATEWAY_DIR override"
```

---

### Task 3: Device store (tokens hashed, issue/verify/revoke)

**Files:**
- Create: `crates/omega-gateway/src/auth.rs`
- Modify: `crates/omega-gateway/src/lib.rs` (add `pub mod auth;`)

**Interfaces:**
- Produces: `auth::DeviceStore::open(dir: &Path) -> DeviceStore` (file `<dir>/devices.json`), `DeviceStore::issue(&mut self, name: &str) -> (Device, String)` (returns the plaintext token exactly once), `DeviceStore::verify(&self, token: &str) -> Option<Device>`, `DeviceStore::revoke(&mut self, device_id: &str) -> bool`, `auth::Device { pub id: String, pub name: String, pub token_sha256: String, pub created_at: String, pub revoked: bool }`, `auth::sha256_hex(s: &str) -> String`.

- [ ] **Step 1: Write the failing tests**

Append to `crates/omega-gateway/src/auth.rs` (write the full module with `unimplemented!()` bodies plus these tests):
```rust
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub token_sha256: String,
    pub created_at: String,
    pub revoked: bool,
}

pub struct DeviceStore {
    path: PathBuf,
    devices: Vec<Device>,
}

pub fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex_string(&h.finalize())
}

fn hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn random_hex(n_bytes: usize) -> String {
    use rand::RngCore;
    let mut buf = vec![0u8; n_bytes];
    rand::rngs::OsRng.fill_bytes(&mut buf);
    hex_string(&buf)
}

impl DeviceStore {
    pub fn open(dir: &Path) -> Self { unimplemented!() }
    pub fn issue(&mut self, name: &str) -> (Device, String) { unimplemented!() }
    pub fn verify(&self, token: &str) -> Option<Device> { unimplemented!() }
    pub fn revoke(&mut self, device_id: &str) -> bool { unimplemented!() }
    fn save(&self) { unimplemented!() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_then_verify_roundtrip_and_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = DeviceStore::open(dir.path());
        let (device, token) = store.issue("iphone");
        assert_eq!(token.len(), 64); // 32 bytes hex
        assert_eq!(device.token_sha256, sha256_hex(&token));
        // reopen from disk: token still verifies
        let store2 = DeviceStore::open(dir.path());
        assert_eq!(store2.verify(&token).unwrap().name, "iphone");
        assert!(store2.verify("wrong-token").is_none());
    }

    #[test]
    fn revoked_device_fails_verify() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = DeviceStore::open(dir.path());
        let (device, token) = store.issue("mac");
        assert!(store.revoke(&device.id));
        assert!(store.verify(&token).is_none());
        assert!(!store.revoke("no-such-id"));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p omega-gateway auth::`
Expected: FAIL (`unimplemented!()`)

- [ ] **Step 3: Implement the store**

```rust
impl DeviceStore {
    pub fn open(dir: &Path) -> Self {
        std::fs::create_dir_all(dir).ok();
        let path = dir.join("devices.json");
        let devices = std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        Self { path, devices }
    }

    pub fn issue(&mut self, name: &str) -> (Device, String) {
        let token = random_hex(32);
        let device = Device {
            id: random_hex(8),
            name: name.to_string(),
            token_sha256: sha256_hex(&token),
            created_at: chrono::Utc::now().to_rfc3339(),
            revoked: false,
        };
        self.devices.push(device.clone());
        self.save();
        (device, token)
    }

    pub fn verify(&self, token: &str) -> Option<Device> {
        let hash = sha256_hex(token);
        self.devices.iter().find(|d| !d.revoked && d.token_sha256 == hash).cloned()
    }

    pub fn revoke(&mut self, device_id: &str) -> bool {
        let mut hit = false;
        for d in self.devices.iter_mut() {
            if d.id == device_id { d.revoked = true; hit = true; }
        }
        if hit { self.save(); }
        hit
    }

    fn save(&self) {
        let text = serde_json::to_string_pretty(&self.devices).expect("serialize devices");
        std::fs::write(&self.path, text).expect("write devices.json");
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p omega-gateway auth::`
Expected: PASS (2 tests)

- [ ] **Step 5: Commit**

```bash
git add crates/omega-gateway/src/auth.rs crates/omega-gateway/src/lib.rs
git commit -m "feat(gateway): device store with hashed tokens, issue/verify/revoke"
```

---

### Task 4: Pairing (one-time code + POST /v1/pair + CLI `pair`)

**Files:**
- Modify: `crates/omega-gateway/src/auth.rs` (add `PairingCode`)
- Create: `crates/omega-gateway/src/routes_pair.rs`
- Modify: `crates/omega-gateway/src/server.rs` (mount route; state gains config)
- Modify: `crates/omega-gateway/src/main.rs` (clap subcommands `serve` and `pair`)
- Modify: `crates/omega-gateway/src/lib.rs` (add `pub mod routes_pair;`)
- Create: `crates/omega-gateway/tests/pair_test.rs`

**Interfaces:**
- Consumes: `auth::DeviceStore`, `config::GatewayConfig`, `server::build_router`.
- Produces: `auth::PairingCode { pub code: String, pub expires_at: String }`, `auth::PairingCode::create(dir: &Path, ttl_secs: i64) -> PairingCode` (writes `<dir>/pairing.json`), `auth::PairingCode::consume(dir: &Path, code: &str) -> bool` (true once, deletes file), endpoint `POST /v1/pair` body `{"code": "...", "device_name": "..."}` → `200 {"device_id","token"}` or `403 {"error":"invalid or expired code"}`. `AppState` becomes `{ pub dir: PathBuf, pub cfg: GatewayConfig }`.

- [ ] **Step 1: Write the failing integration test**

`crates/omega-gateway/tests/pair_test.rs`:
```rust
use omega_gateway::auth::PairingCode;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

#[tokio::test]
async fn pair_with_valid_code_once_then_reject() {
    let dir = tempfile::tempdir().unwrap();
    let pairing = PairingCode::create(dir.path(), 300);
    let app = build_router(AppState {
        dir: dir.path().to_path_buf(),
        cfg: GatewayConfig::default(),
    });
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    let res = client.post(format!("{base}/v1/pair"))
        .json(&serde_json::json!({ "code": pairing.code, "device_name": "iphone" }))
        .send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["token"].as_str().unwrap().len(), 64);

    // second use of the same code: refused
    let res2 = client.post(format!("{base}/v1/pair"))
        .json(&serde_json::json!({ "code": pairing.code, "device_name": "mac" }))
        .send().await.unwrap();
    assert_eq!(res2.status(), 403);
}

#[tokio::test]
async fn expired_code_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let pairing = PairingCode::create(dir.path(), -1); // already expired
    let app = build_router(AppState {
        dir: dir.path().to_path_buf(),
        cfg: GatewayConfig::default(),
    });
    let base = spawn(app).await;
    let res = reqwest::Client::new().post(format!("{base}/v1/pair"))
        .json(&serde_json::json!({ "code": pairing.code, "device_name": "x" }))
        .send().await.unwrap();
    assert_eq!(res.status(), 403);
}
```

Update `AppState` construction in `tests/health_test.rs` the same way (`cfg: GatewayConfig::default()`).

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p omega-gateway --test pair_test`
Expected: FAIL to compile (PairingCode missing) — add the stubs below, then FAIL with 404.

- [ ] **Step 3: Implement PairingCode in `auth.rs`**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingCode {
    pub code: String,
    pub expires_at: String,
}

impl PairingCode {
    pub fn create(dir: &Path, ttl_secs: i64) -> Self {
        std::fs::create_dir_all(dir).ok();
        let pc = Self {
            code: random_hex(4), // 8 hex chars, human-typable
            expires_at: (chrono::Utc::now() + chrono::Duration::seconds(ttl_secs)).to_rfc3339(),
        };
        std::fs::write(
            dir.join("pairing.json"),
            serde_json::to_string(&pc).expect("serialize pairing"),
        ).expect("write pairing.json");
        pc
    }

    /// Returns true exactly once for a live matching code; deletes the file on success.
    pub fn consume(dir: &Path, code: &str) -> bool {
        let path = dir.join("pairing.json");
        let Ok(text) = std::fs::read_to_string(&path) else { return false };
        let Ok(pc) = serde_json::from_str::<PairingCode>(&text) else { return false };
        let live = pc.code == code
            && chrono::DateTime::parse_from_rfc3339(&pc.expires_at)
                .map(|t| t > chrono::Utc::now())
                .unwrap_or(false);
        if live { std::fs::remove_file(&path).ok(); }
        live
    }
}
```

- [ ] **Step 4: Implement the route and mount it**

`crates/omega-gateway/src/routes_pair.rs`:
```rust
use crate::auth::{DeviceStore, PairingCode};
use crate::server::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct PairRequest {
    pub code: String,
    pub device_name: String,
}

pub async fn pair(
    State(state): State<AppState>,
    Json(req): Json<PairRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    if !PairingCode::consume(&state.dir, &req.code) {
        return (StatusCode::FORBIDDEN, Json(json!({ "error": "invalid or expired code" })));
    }
    let mut store = DeviceStore::open(&state.dir);
    let (device, token) = store.issue(&req.device_name);
    (StatusCode::OK, Json(json!({ "device_id": device.id, "token": token })))
}
```

In `server.rs`, extend state and mount:
```rust
use crate::config::GatewayConfig;

#[derive(Clone)]
pub struct AppState {
    pub dir: PathBuf,
    pub cfg: GatewayConfig,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/pair", axum::routing::post(crate::routes_pair::pair))
        .with_state(state)
}
```

- [ ] **Step 5: Wire the CLI in `main.rs`**

```rust
use clap::{Parser, Subcommand};
use omega_gateway::auth::PairingCode;
use omega_gateway::config::{gateway_dir, GatewayConfig};
use omega_gateway::server::{build_router, AppState};

#[derive(Parser)]
#[command(name = "omega-gatewayd", about = "OmegaOS gateway daemon")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run the gateway server (default)
    Serve,
    /// Print a one-time pairing code + QR (valid 5 minutes)
    Pair,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let dir = gateway_dir();
    match cli.command.unwrap_or(Command::Serve) {
        Command::Pair => {
            let pc = PairingCode::create(&dir, 300);
            let host = hostname_or_default();
            let payload = format!("omega://pair?host={host}&code={}", pc.code);
            qr2term::print_qr(&payload).ok();
            println!("Pairing code: {}  (valid 5 minutes)", pc.code);
            println!("Payload: {payload}");
        }
        Command::Serve => {
            let cfg = GatewayConfig::load(&dir);
            let bind = cfg.bind.clone();
            let app = build_router(AppState { dir, cfg });
            let listener = tokio::net::TcpListener::bind(&bind).await?;
            tracing::info!("omega-gateway listening on {bind}");
            axum::serve(listener, app).await?;
        }
    }
    Ok(())
}

fn hostname_or_default() -> String {
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".into())
}
```

- [ ] **Step 6: Run all tests to verify they pass**

Run: `cargo test -p omega-gateway`
Expected: PASS (health, config, auth, pair tests)

- [ ] **Step 7: Commit**

```bash
git add crates/omega-gateway
git commit -m "feat(gateway): one-time pairing code with QR, POST /v1/pair issues device token"
```

---

### Task 5: Auth middleware on /v1/* (except health and pair)

**Files:**
- Modify: `crates/omega-gateway/src/server.rs`
- Create: `crates/omega-gateway/tests/auth_middleware_test.rs`

**Interfaces:**
- Consumes: `auth::DeviceStore::verify`.
- Produces: middleware `server::require_device` accepting `Authorization: Bearer <token>` header OR `?token=<token>` query param (WebSocket clients cannot always set headers); protected demo route `GET /v1/whoami` → `200 {"device_id","name"}`. Later tasks mount their routes on the protected router returned by `build_router`.

- [ ] **Step 1: Write the failing test**

`crates/omega-gateway/tests/auth_middleware_test.rs`:
```rust
use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

#[tokio::test]
async fn whoami_requires_valid_token() {
    let dir = tempfile::tempdir().unwrap();
    let (_, token) = DeviceStore::open(dir.path()).issue("iphone");
    let app = build_router(AppState { dir: dir.path().to_path_buf(), cfg: GatewayConfig::default() });
    let base = spawn(app).await;
    let client = reqwest::Client::new();

    // no token → 401
    assert_eq!(client.get(format!("{base}/v1/whoami")).send().await.unwrap().status(), 401);
    // bad token → 401
    assert_eq!(client.get(format!("{base}/v1/whoami"))
        .bearer_auth("bad").send().await.unwrap().status(), 401);
    // good token via header → 200 with device name
    let res = client.get(format!("{base}/v1/whoami")).bearer_auth(&token).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["name"], "iphone");
    // good token via query param → 200
    assert_eq!(client.get(format!("{base}/v1/whoami?token={token}")).send().await.unwrap().status(), 200);
    // health stays public
    assert_eq!(client.get(format!("{base}/v1/health")).send().await.unwrap().status(), 200);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p omega-gateway --test auth_middleware_test`
Expected: FAIL (404 on /v1/whoami)

- [ ] **Step 3: Implement middleware + whoami in `server.rs`**

```rust
use crate::auth::{Device, DeviceStore};
use axum::{
    extract::{Query, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Extension, Json, Router,
};
use std::collections::HashMap;

pub async fn require_device(
    State(state): State<AppState>,
    Query(query): Query<HashMap<String, String>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let header_token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_string);
    let token = header_token.or_else(|| query.get("token").cloned());
    let Some(token) = token else { return Err(StatusCode::UNAUTHORIZED) };
    let Some(device) = DeviceStore::open(&state.dir).verify(&token) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    req.extensions_mut().insert(device);
    Ok(next.run(req).await)
}

async fn whoami(Extension(device): Extension<Device>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "device_id": device.id, "name": device.name }))
}

pub fn build_router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/v1/whoami", get(whoami))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_device));
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/pair", axum::routing::post(crate::routes_pair::pair))
        .merge(protected)
        .with_state(state)
}
```

- [ ] **Step 4: Run all tests to verify they pass**

Run: `cargo test -p omega-gateway`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/omega-gateway/src/server.rs crates/omega-gateway/tests/auth_middleware_test.rs
git commit -m "feat(gateway): bearer/query device-token middleware, /v1/whoami"
```

---

### Task 6: rmux wrapper + GET /v1/sessions

**Files:**
- Create: `crates/omega-gateway/src/rmux.rs`
- Create: `crates/omega-gateway/src/routes_sessions.rs`
- Modify: `crates/omega-gateway/src/server.rs` (mount `/v1/sessions` on the protected router)
- Modify: `crates/omega-gateway/src/lib.rs` (add `pub mod rmux; pub mod routes_sessions;`)
- Create: `crates/omega-gateway/tests/sessions_test.rs`

**Interfaces:**
- Consumes: auth middleware from Task 5.
- Produces: `rmux::rmux_bin() -> PathBuf` (env `OMEGA_RMUX_BIN`, else `$HOME/.local/bin/rmux`), `rmux::list_sessions() -> anyhow::Result<Vec<String>>` (runs `rmux ls -F '#S'`), `rmux::capture_pane(session: &str, lines: u32) -> anyhow::Result<String>` (runs `rmux capture-pane -p -t <session> -S -<lines>`), endpoint `GET /v1/sessions` → `200 {"sessions":[{"name":"..."}]}`. On rmux failure: `200 {"sessions":[],"error":"<stderr>"}` (a broken rmux must not 500 the API).

- [ ] **Step 1: Write the failing test with a fake rmux fixture**

`crates/omega-gateway/tests/sessions_test.rs`:
```rust
use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

/// Writes an executable fake rmux script and points OMEGA_RMUX_BIN at it.
fn install_fake_rmux(dir: &std::path::Path, script_body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("rmux");
    std::fs::write(&path, format!("#!/usr/bin/env bash\n{script_body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_RMUX_BIN", &path);
}

#[tokio::test]
async fn lists_sessions_from_rmux() {
    let dir = tempfile::tempdir().unwrap();
    install_fake_rmux(dir.path(), r#"
if [ "$1" = "ls" ]; then printf 'oracle-Verba-1\nworker-a\n'; exit 0; fi
exit 1"#);
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState { dir: dir.path().to_path_buf(), cfg: GatewayConfig::default() });
    let base = spawn(app).await;
    let body: serde_json::Value = reqwest::Client::new()
        .get(format!("{base}/v1/sessions")).bearer_auth(&token)
        .send().await.unwrap().json().await.unwrap();
    let names: Vec<&str> = body["sessions"].as_array().unwrap()
        .iter().map(|s| s["name"].as_str().unwrap()).collect();
    assert_eq!(names, vec!["oracle-Verba-1", "worker-a"]);
}

#[tokio::test]
async fn rmux_failure_yields_empty_list_with_error_not_500() {
    let dir = tempfile::tempdir().unwrap();
    install_fake_rmux(dir.path(), "echo 'no server running' >&2; exit 1");
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let app = build_router(AppState { dir: dir.path().to_path_buf(), cfg: GatewayConfig::default() });
    let base = spawn(app).await;
    let res = reqwest::Client::new()
        .get(format!("{base}/v1/sessions")).bearer_auth(&token).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["sessions"].as_array().unwrap().len(), 0);
    assert!(body["error"].as_str().unwrap().contains("no server running"));
}
```

Note: both tests mutate the process-global `OMEGA_RMUX_BIN`; run this test file single-threaded. Add at the top of the file: `// Run with --test-threads=1 semantics via serial guard:` and use a `static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());` acquired at the start of each test (`let _g = LOCK.lock().unwrap();`).

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p omega-gateway --test sessions_test`
Expected: FAIL to compile (modules missing) — add stubs, then FAIL with 404.

- [ ] **Step 3: Implement `rmux.rs`**

```rust
use anyhow::{bail, Result};
use std::path::PathBuf;
use std::process::Command;

pub fn rmux_bin() -> PathBuf {
    if let Ok(bin) = std::env::var("OMEGA_RMUX_BIN") {
        return PathBuf::from(bin);
    }
    dirs::home_dir().expect("no home dir").join(".local/bin/rmux")
}

fn run(args: &[&str]) -> Result<String> {
    let out = Command::new(rmux_bin()).args(args).output()?;
    if !out.status.success() {
        bail!("{}", String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn list_sessions() -> Result<Vec<String>> {
    let out = run(&["ls", "-F", "#S"])?;
    Ok(out.lines().map(str::to_string).filter(|l| !l.is_empty()).collect())
}

pub fn capture_pane(session: &str, lines: u32) -> Result<String> {
    let start = format!("-{lines}");
    run(&["capture-pane", "-p", "-t", session, "-S", &start])
}
```

- [ ] **Step 4: Implement `routes_sessions.rs` and mount**

```rust
use axum::Json;
use serde_json::json;

pub async fn list() -> Json<serde_json::Value> {
    match tokio::task::spawn_blocking(crate::rmux::list_sessions).await {
        Ok(Ok(names)) => Json(json!({
            "sessions": names.iter().map(|n| json!({ "name": n })).collect::<Vec<_>>()
        })),
        Ok(Err(e)) => Json(json!({ "sessions": [], "error": e.to_string() })),
        Err(e) => Json(json!({ "sessions": [], "error": e.to_string() })),
    }
}
```

In `server.rs`, add to the protected router:
```rust
.route("/v1/sessions", get(crate::routes_sessions::list))
```

- [ ] **Step 5: Run all tests to verify they pass**

Run: `cargo test -p omega-gateway`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/omega-gateway
git commit -m "feat(gateway): rmux wrapper and GET /v1/sessions (errors render, never 500)"
```

---

### Task 7: WebSocket session stream (frames on change, errors as frames)

**Files:**
- Modify: `crates/omega-gateway/src/routes_sessions.rs`
- Modify: `crates/omega-gateway/src/server.rs` (mount `/v1/sessions/{name}/stream`)
- Create: `crates/omega-gateway/tests/stream_test.rs`

**Interfaces:**
- Consumes: `rmux::capture_pane`, `cfg.stream_interval_ms`, `cfg.stream_lines`, auth middleware (token via `?token=` for WS).
- Produces: WS endpoint `GET /v1/sessions/{name}/stream`; messages are JSON text frames: `{"type":"frame","text":"<rendered pane>"}` sent on first capture and whenever content changes; `{"type":"error","message":"..."}` on capture failure (loop continues, R-STREAM). Frame type lives in `routes_sessions::StreamFrame`.

- [ ] **Step 1: Write the failing test**

`crates/omega-gateway/tests/stream_test.rs`:
```rust
use futures_util::StreamExt;
use omega_gateway::auth::DeviceStore;
use omega_gateway::config::GatewayConfig;
use omega_gateway::server::{build_router, AppState};
use tokio_tungstenite::connect_async;

static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn install_fake_rmux(dir: &std::path::Path, script_body: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("rmux");
    std::fs::write(&path, format!("#!/usr/bin/env bash\n{script_body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("OMEGA_RMUX_BIN", &path);
}

#[tokio::test]
async fn stream_sends_frame_then_only_on_change() {
    let _g = LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    // fake rmux: capture-pane output changes based on a counter file
    install_fake_rmux(dir.path(), &format!(r#"
counter="{}/count"
n=$(cat "$counter" 2>/dev/null || echo 0)
echo $((n+1)) > "$counter"
if [ $n -lt 2 ]; then echo "SCREEN-A"; else echo "SCREEN-B"; fi"#,
        dir.path().display()));
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let mut cfg = GatewayConfig::default();
    cfg.stream_interval_ms = 50;
    let app = build_router(AppState { dir: dir.path().to_path_buf(), cfg });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let url = format!("ws://{addr}/v1/sessions/demo/stream?token={token}");
    let (mut ws, _) = connect_async(url).await.unwrap();

    let first = ws.next().await.unwrap().unwrap().into_text().unwrap();
    let f1: serde_json::Value = serde_json::from_str(&first).unwrap();
    assert_eq!(f1["type"], "frame");
    assert!(f1["text"].as_str().unwrap().contains("SCREEN-A"));

    // next frame arrives only when content changes to SCREEN-B
    let second = ws.next().await.unwrap().unwrap().into_text().unwrap();
    let f2: serde_json::Value = serde_json::from_str(&second).unwrap();
    assert!(f2["text"].as_str().unwrap().contains("SCREEN-B"));
}

#[tokio::test]
async fn capture_failure_becomes_error_frame_and_loop_survives() {
    let _g = LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    install_fake_rmux(dir.path(), "echo 'session not found' >&2; exit 1");
    let (_, token) = DeviceStore::open(dir.path()).issue("t");
    let mut cfg = GatewayConfig::default();
    cfg.stream_interval_ms = 50;
    let app = build_router(AppState { dir: dir.path().to_path_buf(), cfg });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let url = format!("ws://{addr}/v1/sessions/ghost/stream?token={token}");
    let (mut ws, _) = tokio_tungstenite::connect_async(url).await.unwrap();
    let msg = ws.next().await.unwrap().unwrap().into_text().unwrap();
    let frame: serde_json::Value = serde_json::from_str(&msg).unwrap();
    assert_eq!(frame["type"], "error");
    assert!(frame["message"].as_str().unwrap().contains("session not found"));
    // the connection is still alive: another error frame arrives instead of a close
    let msg2 = ws.next().await.unwrap().unwrap();
    assert!(msg2.is_text());
}
```

Also add the same `LOCK` guard usage to `sessions_test.rs` tests (shared env var).

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p omega-gateway --test stream_test`
Expected: FAIL (404 → WS handshake error)

- [ ] **Step 3: Implement the stream handler**

Append to `routes_sessions.rs`:
```rust
use crate::server::AppState;
use axum::extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Path, State};
use axum::response::Response;
use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamFrame {
    Frame { text: String },
    Error { message: String },
}

pub async fn stream(
    ws: WebSocketUpgrade,
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| stream_loop(socket, name, state))
}

async fn stream_loop(mut socket: WebSocket, name: String, state: AppState) {
    let interval = std::time::Duration::from_millis(state.cfg.stream_interval_ms);
    let lines = state.cfg.stream_lines;
    let mut last: Option<String> = None;
    // R-STREAM: this loop never exits on error; errors are rendered as frames.
    loop {
        let session = name.clone();
        let captured = tokio::task::spawn_blocking(move || crate::rmux::capture_pane(&session, lines)).await;
        let frame = match captured {
            Ok(Ok(text)) => {
                if last.as_deref() == Some(text.as_str()) {
                    None
                } else {
                    last = Some(text.clone());
                    Some(StreamFrame::Frame { text })
                }
            }
            Ok(Err(e)) => Some(StreamFrame::Error { message: e.to_string() }),
            Err(e) => Some(StreamFrame::Error { message: e.to_string() }),
        };
        if let Some(frame) = frame {
            let text = serde_json::to_string(&frame).expect("serialize frame");
            if socket.send(Message::Text(text.into())).await.is_err() {
                return; // client went away: the ONLY exit
            }
        }
        tokio::time::sleep(interval).await;
    }
}
```

Mount in `server.rs` on the protected router:
```rust
.route("/v1/sessions/{name}/stream", get(crate::routes_sessions::stream))
```

- [ ] **Step 4: Run all tests to verify they pass**

Run: `cargo test -p omega-gateway`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/omega-gateway
git commit -m "feat(gateway): WebSocket session stream, frames on change, errors as frames"
```

---

### Task 8: Protocol schema export (source of truth for the app's TS types)

**Files:**
- Create: `crates/omega-gateway/src/protocol.rs`
- Modify: `crates/omega-gateway/src/lib.rs` (add `pub mod protocol;`)
- Modify: `crates/omega-gateway/src/main.rs` (add `Schema` subcommand)
- Create: `crates/omega-gateway/tests/schema_test.rs`

**Interfaces:**
- Produces: `protocol::Protocol` — a `#[derive(JsonSchema)]` umbrella struct referencing every wire type (`PairRequest`, `PairResponse`, `SessionsResponse`, `StreamFrame`, `WhoamiResponse`), `protocol::schema_json() -> String`, CLI `omega-gatewayd schema` printing that JSON. Plan 4 (the app) generates `packages/protocol` TS types from this output; the schema is the contract.

- [ ] **Step 1: Write the failing test**

`crates/omega-gateway/tests/schema_test.rs`:
```rust
#[test]
fn schema_contains_all_wire_types() {
    let schema = omega_gateway::protocol::schema_json();
    let v: serde_json::Value = serde_json::from_str(&schema).unwrap();
    let defs = v["definitions"].as_object().or_else(|| v["$defs"].as_object()).unwrap();
    for ty in ["PairRequest", "PairResponse", "SessionsResponse", "StreamFrame", "WhoamiResponse"] {
        assert!(defs.contains_key(ty), "missing {ty} in schema");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p omega-gateway --test schema_test`
Expected: FAIL to compile (module missing)

- [ ] **Step 3: Implement `protocol.rs`**

Centralize the wire types (move/redefine them here; `routes_pair.rs` and `routes_sessions.rs` import from `protocol` — replace their local definitions with `use crate::protocol::*;` and delete the duplicates; `StreamFrame` moves here with `#[derive(Serialize, JsonSchema)]`):
```rust
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, JsonSchema)]
pub struct PairRequest {
    pub code: String,
    pub device_name: String,
}

#[derive(Serialize, JsonSchema)]
pub struct PairResponse {
    pub device_id: String,
    pub token: String,
}

#[derive(Serialize, JsonSchema)]
pub struct SessionEntry {
    pub name: String,
}

#[derive(Serialize, JsonSchema)]
pub struct SessionsResponse {
    pub sessions: Vec<SessionEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamFrame {
    Frame { text: String },
    Error { message: String },
}

#[derive(Serialize, JsonSchema)]
pub struct WhoamiResponse {
    pub device_id: String,
    pub name: String,
}

/// Umbrella type so one schema document carries every wire type.
/// Only JsonSchema is needed: this type is never serialized itself.
#[derive(JsonSchema)]
pub struct Protocol {
    pub pair_request: PairRequest,
    pub pair_response: PairResponse,
    pub sessions_response: SessionsResponse,
    pub stream_frame: StreamFrame,
    pub whoami_response: WhoamiResponse,
}

pub fn schema_json() -> String {
    let schema = schema_for!(Protocol);
    serde_json::to_string_pretty(&schema).expect("serialize schema")
}
```

Adjust route handlers to return the typed structs (`Json<SessionsResponse>` etc.) instead of `serde_json::json!` blobs, and update the pair handler to `Json(PairResponse { device_id, token })`. Existing tests keep passing because the JSON shape is unchanged.

Add to `main.rs` `Command` enum:
```rust
    /// Print the wire-protocol JSON Schema (for TS type generation)
    Schema,
```
and to the match:
```rust
        Command::Schema => println!("{}", omega_gateway::protocol::schema_json()),
```

- [ ] **Step 4: Run all tests to verify they pass**

Run: `cargo test -p omega-gateway`
Expected: PASS (all suites; route refactor did not change JSON shapes)

- [ ] **Step 5: Commit**

```bash
git add crates/omega-gateway
git commit -m "feat(gateway): typed wire protocol with JSON Schema export (omega-gatewayd schema)"
```

---

### Task 9: Service unit + installer wiring + reproducibility (L0)

**Files:**
- Create: `config/omega-gateway.service`
- Modify: `install.sh` (append gateway install block near where other binaries are installed — locate the section installing `omega` into `~/.local/bin` with `grep -n '.local/bin' install.sh`)
- Modify: `verify-install.sh` (add a gateway check; locate with `grep -n 'omega -V\|checks' verify-install.sh`)

**Interfaces:**
- Consumes: the `omega-gatewayd` binary produced by the workspace build.
- Produces: user-level systemd unit `omega-gateway.service`; fresh `./install.sh` installs and enables it; `verify-install.sh` fails if the binary or unit is missing.

- [ ] **Step 1: Write the service unit**

`config/omega-gateway.service`:
```ini
[Unit]
Description=OmegaOS gateway daemon (app API)
After=network.target

[Service]
ExecStart=%h/.local/bin/omega-gatewayd serve
Restart=on-failure
RestartSec=5
Environment=PATH=%h/.local/bin:/usr/local/bin:/usr/bin:/bin

[Install]
WantedBy=default.target
```

Note the PATH line: systemd user units do not inherit the login PATH, and the gateway shells out to `~/.local/bin/rmux` (a failure OmegaOS has hit before with other units).

- [ ] **Step 2: Wire install.sh**

Append after the existing binary-install section (adapt the marker to what grep found):
```bash
# --- omega-gateway (app API daemon) ---
install -m 0755 target/release/omega-gatewayd "$HOME/.local/bin/omega-gatewayd"
mkdir -p "$HOME/.config/systemd/user"
cp config/omega-gateway.service "$HOME/.config/systemd/user/omega-gateway.service"
if command -v systemctl >/dev/null 2>&1 && [ -d /run/systemd/system ]; then
  systemctl --user daemon-reload || true
  systemctl --user enable --now omega-gateway.service || true
fi
```

Also ensure the build line that produces release binaries includes the new bin (if install.sh builds with `cargo build --release`, the workspace already covers it; if it builds per-crate, add `-p omega-gateway`).

- [ ] **Step 3: Wire verify-install.sh**

Add alongside the existing checks:
```bash
check "omega-gatewayd binary" test -x "$HOME/.local/bin/omega-gatewayd"
check "omega-gateway unit" test -f "$HOME/.config/systemd/user/omega-gateway.service"
```
(Match the local `check` helper's actual signature — read the surrounding lines first and imitate them exactly.)

- [ ] **Step 4: Verify against runtime**

Run:
```bash
cargo build --release -p omega-gateway
./install.sh >/dev/null 2>&1 || true   # or the targeted section if full install is heavy
test -x "$HOME/.local/bin/omega-gatewayd" && echo BIN-OK
systemctl --user status omega-gateway --no-pager | head -5
curl -s http://127.0.0.1:4477/v1/health
./verify-install.sh | tail -5
```
Expected: `BIN-OK`, unit `active (running)`, health returns `{"ok":true,...}`, verify-install passes.

- [ ] **Step 5: Full workspace gate, then commit and push**

Run: `cargo test -p omega-gateway && cargo clippy -p omega-gateway -- -D warnings && cargo build --release`
Expected: all green.

```bash
git add config/omega-gateway.service install.sh verify-install.sh crates/omega-gateway
git commit -m "feat(gateway): systemd unit + installer wiring, verify-install checks"
git -c credential.helper= -c credential.helper="store --file=$HOME/.omega/secrets/agentik-os.git-credentials" push origin main
```

---

## Out of scope for this plan (next plans)

- Chat sessions (headless Claude/Codex), mission dispatch, progress, alerts/push events → Plan 2.
- Convex/Clerk registry + APNs relay → Plan 3.
- The omega-app monorepo (Electron + Expo + shared React core) → Plan 4; it consumes `omega-gatewayd schema` output for its `packages/protocol`.
- Tailscale serve fronting and non-loopback binds (stay loopback + tailnet until Plan 2 hardening).
