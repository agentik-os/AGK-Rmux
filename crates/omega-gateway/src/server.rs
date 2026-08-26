use crate::accounts::AccountStore;
use crate::auth::{Device, DeviceStore};
use crate::chat_store::ChatStore;
use crate::config::GatewayConfig;
use crate::events::EventHub;
use crate::protocol::WhoamiResponse;
use crate::relay::RelayManager;
use crate::session_org::SessionOrgStore;
use axum::{
    extract::{Query, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Extension, Json, Router,
};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// Global cap on concurrently-running chat turns (across all devices/chats).
const MAX_CONCURRENT_CHAT_TURNS: usize = 8;

/// Global cap on concurrently-running `POST /v1/dispatch` requests. A
/// dispatch spawns a whole oracle session (much heavier than a single chat
/// turn), so this stays well below [`MAX_CONCURRENT_CHAT_TURNS`].
const MAX_CONCURRENT_DISPATCHES: usize = 4;

/// Global cap on concurrently-open `GET /v1/master/chat` WebSockets. Each
/// connection holds its permit for the WHOLE socket lifetime (not per
/// round-trip), and a single round-trip can hold the connection open for up
/// to a 90s poll plus fire a `spawn_blocking` task every ~500ms tick — a
/// heavier, longer-held operation than a single chat turn, so this mirrors
/// [`MAX_CONCURRENT_DISPATCHES`]'s reasoning rather than
/// [`MAX_CONCURRENT_CHAT_TURNS`]'s.
const MAX_CONCURRENT_MASTER_CHATS: usize = 4;

/// Global cap on concurrently-open `GET /v1/orchestrate/stream` WebSockets.
/// `omega orchestrate` is the heaviest, longest-running, most
/// state-mutating operation this crate exposes (a REAL oracle, real
/// workers, a real quality gate, up to its own 3600s default timeout) — one
/// permit held for the WHOLE connection lifetime, capped BELOW
/// [`MAX_CONCURRENT_DISPATCHES`] (see `routes_orchestrate.rs`'s doc
/// comment).
const MAX_CONCURRENT_ORCHESTRATIONS: usize = 2;

/// Global cap on concurrently-open `GET /v1/new-project/stream` WebSockets.
///
/// REVIEW-FIX (Finding 3, Task B round): the comment used to claim this
/// bounds concurrent BOOTSTRAP PIPELINES, mirroring
/// [`MAX_CONCURRENT_ORCHESTRATIONS`]'s reasoning — that comparison does not
/// hold. `orchestrate_permits` is held for `orchestrate`'s ENTIRE
/// multi-minute/hour run, so it genuinely bounds concurrently-running
/// orchestrations. THIS permit is only held for
/// `new_project_stream_loop`'s lifetime, which ends when the fast `omega
/// new-project` CLI process exits — well under a second, since its only
/// real work is `mgr.create_session_with_agent(...)` returning (see
/// `routes_new_project.rs`'s doc comment). So the real effect of this cap
/// is "at most `MAX_CONCURRENT_NEW_PROJECT_SPAWNS` SPAWN REQUESTS in flight
/// at any instant" — a basic DoS/resource-exhaustion throughput limiter on
/// this endpoint's own subprocess-spawn rate, measured in spawns/second,
/// NOT "at most N live project bootstraps". An authenticated caller that
/// loops this endpoint quickly can still start an effectively unbounded
/// number of live `*-setup` bootstrap sessions over TIME — nothing in this
/// wave bounds the total count of concurrently-LIVE `*-setup` sessions.
/// This is an explicitly recorded gap, not a silently missing one.
const MAX_CONCURRENT_NEW_PROJECT_SPAWNS: usize = 2;

/// Global cap on concurrently-running `POST /v1/pdf` generations. `omega
/// pdf` shells to `npx tsx bin/pdfgen.ts` and, on a cold cache, a full `npm
/// install` first (see `crates/omega-cli/src/main.rs::cmd_pdf`) — an
/// unbounded, potentially slow, node/npm-spawning operation with no
/// in-process equivalent. Added in this wave's adversarial review round
/// (finding: an authenticated caller could otherwise fire unboundedly many
/// concurrent generations with no cap at all), mirroring
/// [`MAX_CONCURRENT_ORCHESTRATIONS`]'s reasoning rather than
/// [`MAX_CONCURRENT_DISPATCHES`]'s.
const MAX_CONCURRENT_PDF_GENERATIONS: usize = 2;

/// Global cap on concurrently-running `POST /v1/sessions` and `POST
/// /v1/team` requests. Added in the Task A review-fix round (finding: an
/// authenticated device could otherwise fire unboundedly many concurrent
/// `omega new`/`omega team` spawns with no cap at all -- the exact shape of
/// problem [`MAX_CONCURRENT_DISPATCHES`] exists for). ONE semaphore shared
/// across BOTH endpoints rather than a second, separate pool: `omega team`
/// spawning up to `MAX_COUNT` sub-panes under a single call
/// (`routes_team.rs`) is exactly the kind of heavier operation the cap
/// should also gate, and there is no reason a caller could pace
/// `/v1/sessions` and `/v1/team` spawns independently of each other --
/// both ultimately compete for the same underlying rmux/session-spawn
/// capacity. Sized the same as [`MAX_CONCURRENT_DISPATCHES`]: a session or
/// team spawn is a comparably heavy, subprocess-spawning operation to a
/// dispatch.
const MAX_CONCURRENT_SESSION_SPAWNS: usize = 4;

/// Global cap on concurrently-running `POST /v1/duo` runs. `omega-duo run`
/// is a bounded subprocess call into a real Codex/Claude turn — no
/// in-process equivalent, and a single run can genuinely take many minutes
/// (see `routes_duo.rs`'s doc comment on [`crate::routes_duo::duo_bin`]'s
/// timeout). Mirrors [`MAX_CONCURRENT_PDF_GENERATIONS`]'s reasoning
/// (heavier than a single request/response, no WS connection to hold a
/// permit across) rather than [`MAX_CONCURRENT_DISPATCHES`]'s.
const MAX_CONCURRENT_DUO_RUNS: usize = 2;

#[derive(Clone)]
pub struct AppState {
    pub dir: PathBuf,
    pub cfg: GatewayConfig,
    pub chats: Arc<ChatStore>,
    /// Isolated per-account Claude/Codex credential slots. Stateless
    /// (file-backed), so `AccountStore` is cheaply `Clone` itself rather
    /// than `Arc`-wrapped.
    pub accounts: AccountStore,
    pub chat_permits: Arc<Semaphore>,
    /// Caps concurrently-running `POST /v1/dispatch` requests, mirroring
    /// `chat_permits` — see [`MAX_CONCURRENT_DISPATCHES`].
    pub dispatch_permits: Arc<Semaphore>,
    /// Caps concurrently-open `GET /v1/master/chat` WebSockets, one permit
    /// held for the WHOLE connection lifetime — see
    /// [`MAX_CONCURRENT_MASTER_CHATS`].
    pub master_chat_permits: Arc<Semaphore>,
    /// Caps concurrently-open `GET /v1/orchestrate/stream` WebSockets, one
    /// permit held for the WHOLE connection lifetime — see
    /// [`MAX_CONCURRENT_ORCHESTRATIONS`].
    pub orchestrate_permits: Arc<Semaphore>,
    /// Caps concurrently-open `GET /v1/new-project/stream` WebSockets, one
    /// permit held for the WHOLE connection lifetime — see
    /// [`MAX_CONCURRENT_NEW_PROJECT_SPAWNS`].
    pub new_project_permits: Arc<Semaphore>,
    /// Caps concurrently-running `POST /v1/pdf` generations — see
    /// [`MAX_CONCURRENT_PDF_GENERATIONS`].
    pub pdf_permits: Arc<Semaphore>,
    /// Caps concurrently-running `POST /v1/sessions` AND `POST /v1/team`
    /// requests, SHARED across both — see
    /// [`MAX_CONCURRENT_SESSION_SPAWNS`].
    pub session_spawn_permits: Arc<Semaphore>,
    /// Caps concurrently-running `POST /v1/duo` runs — see
    /// [`MAX_CONCURRENT_DUO_RUNS`].
    pub duo_permits: Arc<Semaphore>,
    /// IN-PROCESS per-resolved-cwd lock for `POST /v1/duo`: two
    /// `omega-duo run`s against the SAME cwd corrupt each other's
    /// git-checkpoint guard (the `/duo` skill's own doc is explicit —
    /// "jamais deux runs sur le meme worktree"), so a resolved cwd already
    /// present here means a second concurrent request against it is
    /// refused with 409 rather than allowed to race. A plain
    /// `std::sync::Mutex`, not `tokio::sync::Mutex`: every access is a
    /// synchronous insert/remove with no `.await` held across the lock, so
    /// the lighter std primitive is the right one (see
    /// `routes_duo.rs::CwdLockGuard`).
    pub duo_active_dirs: Arc<std::sync::Mutex<std::collections::HashSet<PathBuf>>>,
    /// Event bus for `/v1/events` (mission updates, alerts, heartbeat).
    /// Cloning `AppState` shares this hub, so a test (or a future
    /// in-process alert source) can hold its own clone and call
    /// `emit(...)` while the router forwards from the same bus.
    pub events: EventHub,
    /// Session organization overlay (folder/label/pinned) -- `Arc`-wrapped
    /// like `chats`, since it guards its own writes internally (see
    /// `session_org.rs`'s `Mutex`) and is shared across every clone of
    /// `AppState`.
    pub session_org: Arc<SessionOrgStore>,
    /// When this gateway process's `AppState` was constructed -- the base
    /// point `routes_box::box_info`'s `uptime_secs` measures `.elapsed()`
    /// against. `Instant` is `Copy`, so cloning `AppState` carries the SAME
    /// start point rather than resetting it.
    pub started_at: std::time::Instant,
    /// Owns the optional outbound relay supervisor. It is dormant until a
    /// Clerk-authenticated cloud registration has been persisted.
    pub relay: RelayManager,
}

impl AppState {
    /// Builds the full app state, opening the chat + account stores rooted
    /// at `dir` and sizing the global chat-turn semaphore to
    /// [`MAX_CONCURRENT_CHAT_TURNS`] and the dispatch semaphore to
    /// [`MAX_CONCURRENT_DISPATCHES`].
    pub fn new(dir: PathBuf, cfg: GatewayConfig) -> Self {
        let chats = Arc::new(ChatStore::open(&dir));
        let accounts = AccountStore::open(&dir);
        let chat_permits = Arc::new(Semaphore::new(MAX_CONCURRENT_CHAT_TURNS));
        let dispatch_permits = Arc::new(Semaphore::new(MAX_CONCURRENT_DISPATCHES));
        let master_chat_permits = Arc::new(Semaphore::new(MAX_CONCURRENT_MASTER_CHATS));
        let orchestrate_permits = Arc::new(Semaphore::new(MAX_CONCURRENT_ORCHESTRATIONS));
        let new_project_permits = Arc::new(Semaphore::new(MAX_CONCURRENT_NEW_PROJECT_SPAWNS));
        let pdf_permits = Arc::new(Semaphore::new(MAX_CONCURRENT_PDF_GENERATIONS));
        let session_spawn_permits = Arc::new(Semaphore::new(MAX_CONCURRENT_SESSION_SPAWNS));
        let duo_permits = Arc::new(Semaphore::new(MAX_CONCURRENT_DUO_RUNS));
        let duo_active_dirs = Arc::new(std::sync::Mutex::new(std::collections::HashSet::new()));
        let events = EventHub::new();
        let session_org = Arc::new(SessionOrgStore::open(&dir));
        let started_at = std::time::Instant::now();
        let relay = RelayManager::production(dir.clone());
        Self {
            dir,
            cfg,
            chats,
            accounts,
            chat_permits,
            dispatch_permits,
            master_chat_permits,
            orchestrate_permits,
            new_project_permits,
            pdf_permits,
            session_spawn_permits,
            duo_permits,
            duo_active_dirs,
            events,
            session_org,
            started_at,
            relay,
        }
    }

    /// Test/integration constructor for a local JWKS and relay endpoint.
    /// Existing tests continue using [`Self::new`] unchanged.
    #[cfg(test)]
    pub(crate) fn with_relay(dir: PathBuf, cfg: GatewayConfig, relay: RelayManager) -> Self {
        let mut state = Self::new(dir, cfg);
        state.relay = relay;
        state
    }
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "ok": true, "version": env!("CARGO_PKG_VERSION") }))
}

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
    // Query tokens exist for WebSocket clients that cannot set headers.
    // Request logging must therefore never log full request URIs.
    let token = header_token.or_else(|| query.get("token").cloned());
    let Some(token) = token else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let Some(device) = DeviceStore::open(&state.dir).verify(&token) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    req.extensions_mut().insert(device);
    Ok(next.run(req).await)
}

async fn whoami(Extension(device): Extension<Device>) -> Json<WhoamiResponse> {
    Json(WhoamiResponse {
        device_id: device.id,
        name: device.name,
    })
}

pub fn build_router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/v1/whoami", get(whoami))
        .route(
            "/v1/sessions",
            get(crate::routes_sessions::list).post(crate::routes_sessions::create),
        )
        .route(
            "/v1/sessions/{name}/stream",
            get(crate::routes_sessions::stream),
        )
        .route(
            "/v1/sessions/{name}/keys",
            axum::routing::post(crate::routes_sessions::send_keys),
        )
        .route(
            "/v1/sessions/{name}/close",
            axum::routing::post(crate::routes_sessions::close),
        )
        .route(
            "/v1/sessions/{name}/rename",
            axum::routing::post(crate::routes_sessions::rename),
        )
        .route("/v1/team", axum::routing::post(crate::routes_team::create))
        .route(
            "/v1/chats",
            get(crate::routes_chat::list).post(crate::routes_chat::create),
        )
        .route("/v1/chats/{id}", get(crate::routes_chat::get))
        .route("/v1/chats/{id}/messages", get(crate::routes_chat::messages))
        .route("/v1/chats/{id}/stream", get(crate::routes_chat::stream))
        .route("/v1/missions", get(crate::routes_missions::list))
        .route("/v1/oracles", get(crate::routes_oracles::list))
        .route("/v1/rules", get(crate::routes_rules::list))
        .route("/v1/agents", get(crate::routes_agents::list))
        .route("/v1/system-agents", get(crate::routes_system_agents::list))
        .route(
            "/v1/agents/{name}/install",
            axum::routing::post(crate::routes_agents::install_check),
        )
        .route(
            "/v1/agents/{name}/install/stream",
            get(crate::routes_agents::install_stream),
        )
        .route("/v1/skills", get(crate::routes_skills::list))
        .route(
            "/v1/skills/{name}",
            get(crate::routes_skills::get)
                .put(crate::routes_skills::update)
                .delete(crate::routes_skills::delete),
        )
        .route(
            "/v1/skills/{name}/rename",
            axum::routing::post(crate::routes_skills::rename),
        )
        .route(
            "/v1/skills/{name}/agent",
            axum::routing::post(crate::routes_skills::ask_agent),
        )
        .route("/v1/os", get(crate::routes_os::list))
        .route("/v1/projects", get(crate::routes_projects::list))
        .route("/v1/marketing", get(crate::routes_marketing::list))
        .route("/v1/files", get(crate::routes_files::list))
        .route("/v1/files/read", get(crate::routes_files::read))
        .route("/v1/audits", get(crate::routes_audit::list))
        .route("/v1/audit", axum::routing::post(crate::routes_audit::check))
        .route("/v1/audit/stream", get(crate::routes_audit::stream))
        .route(
            "/v1/dispatch",
            axum::routing::post(crate::routes_dispatch::create),
        )
        // B1 fix: axum 0.8's `Multipart` extractor falls back to its own
        // internal 2 MiB default body limit when no `DefaultBodyLimit` layer
        // is set, silently overriding `MAX_DEPOSIT_BYTES` (the crate's own
        // 50 MiB cap in routes_deposit.rs) for any upload between 2 MiB and
        // 50 MiB. Scoped to THIS route only (via `.layer()` on the
        // `MethodRouter`, not the outer `Router`) so no other route's body
        // limit changes. The +8192 margin covers multipart boundary/header
        // overhead so a file at exactly MAX_DEPOSIT_BYTES still clears this
        // layer and reaches the crate's own size check, which is the one
        // that must make the final call.
        .route(
            "/v1/deposit",
            axum::routing::post(crate::routes_deposit::create).layer(
                axum::extract::DefaultBodyLimit::max(
                    crate::routes_deposit::MAX_DEPOSIT_BYTES + 8192,
                ),
            ),
        )
        .route("/v1/events", get(crate::routes_events::events))
        .route(
            "/v1/accounts",
            get(crate::routes_accounts::list).post(crate::routes_accounts::create),
        )
        .route(
            "/v1/accounts/{slug}",
            axum::routing::delete(crate::routes_accounts::delete),
        )
        .route(
            "/v1/accounts/{slug}/default",
            axum::routing::post(crate::routes_accounts::set_default),
        )
        .route(
            "/v1/accounts/{slug}/login",
            get(crate::routes_accounts::login),
        )
        .route(
            "/v1/accounts/{slug}/apikey",
            axum::routing::post(crate::routes_accounts::apikey),
        )
        .route("/v1/session-org", get(crate::routes_session_org::get_all))
        .route(
            "/v1/session-org/{name}",
            axum::routing::put(crate::routes_session_org::set),
        )
        .route("/v1/master/chat", get(crate::routes_master::chat))
        .route(
            "/v1/oracles/{session}/timeline",
            get(crate::routes_oracles::timeline),
        )
        .route(
            "/v1/oracles/{session}/gate",
            get(crate::routes_oracles::gate),
        )
        .route(
            "/v1/oracles/{session}/reap",
            axum::routing::post(crate::routes_oracles::reap),
        )
        .route(
            "/v1/oracles/{session}/resurrect",
            axum::routing::post(crate::routes_oracles::resurrect),
        )
        .route(
            "/v1/orchestrate/stream",
            get(crate::routes_orchestrate::stream),
        )
        .route(
            "/v1/new-project/stream",
            get(crate::routes_new_project::stream),
        )
        .route("/v1/doctor", get(crate::routes_box::doctor))
        .route("/v1/usage", get(crate::routes_box::usage))
        .route("/v1/box-info", get(crate::routes_box::box_info))
        .route("/v1/box-id", get(crate::routes_box::box_id))
        .route(
            "/v1/cloud/register",
            axum::routing::post(crate::routes_cloud::register)
                .layer(axum::extract::DefaultBodyLimit::max(16 * 1024)),
        )
        .route("/v1/backup", axum::routing::post(crate::routes_box::backup))
        .route(
            "/v1/config",
            get(crate::routes_config::get).put(crate::routes_config::set),
        )
        .route("/v1/telegram/status", get(crate::routes_telegram::status))
        .route(
            "/v1/telegram/enable",
            axum::routing::post(crate::routes_telegram::enable),
        )
        .route(
            "/v1/telegram/disable",
            axum::routing::post(crate::routes_telegram::disable),
        )
        .route("/v1/pdf", axum::routing::post(crate::routes_pdf::create))
        .route("/v1/pdf/download", get(crate::routes_pdf::download))
        .route("/v1/duo", axum::routing::post(crate::routes_duo::create))
        // IMPORTANT: route_layer only wraps routes registered BEFORE it is
        // called. Add every new protected .route(...) ABOVE this line, or it
        // ships unauthenticated.
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_device,
        ));
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/pair", axum::routing::post(crate::routes_pair::pair))
        .merge(protected)
        .with_state(state)
}
