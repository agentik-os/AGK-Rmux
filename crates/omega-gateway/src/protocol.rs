//! Centralized wire-protocol types for the omega-gateway HTTP/WS API.
//!
//! This module is the source of truth for the app's TypeScript types: every
//! request/response/frame shape crossing the gateway boundary is defined here
//! once, derives `JsonSchema`, and is exported via [`schema_json`] for the
//! `omega-gatewayd schema` CLI command. Route handlers in `routes_pair.rs` and
//! `routes_sessions.rs` import these types rather than redefining them.

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ChatAgent {
    Claude,
    Codex,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatMeta {
    pub id: String,
    pub title: Option<String>,
    pub agent: ChatAgent,
    pub cwd: String,
    pub created_at: String,
    pub updated_at: String,
    pub provider_session_id: Option<String>,
    /// The account slot this chat's turns run under, if one was chosen at
    /// creation (else the kind's default account is resolved per turn).
    /// `#[serde(default)]` so a pre-Task-3 `meta.json` without this field
    /// still deserializes.
    #[serde(default)]
    pub account_slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatMessage {
    pub role: String,
    pub text: String,
    pub ts: String,
}

/// `GET /v1/chats/{id}/messages` response body — a bounded, newest-first
/// page of a chat's transcript (see `ChatStore::tail_page`), plus the
/// cursor for the next (older) page.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ChatMessagesPage {
    /// Newest first.
    pub messages: Vec<ChatMessage>,
    pub next_cursor: Option<u64>,
}

/// `GET /v1/chats/{id}` response body — chat metadata plus the most recent
/// window of messages, chronological (oldest first, matching the pre-Task-B
/// contract). `next_cursor` lets a client page further back into the
/// transcript via `GET /v1/chats/{id}/messages`.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ChatDetailResponse {
    pub meta: ChatMeta,
    /// Chronological, oldest first.
    pub messages: Vec<ChatMessage>,
    pub next_cursor: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AccountKind {
    Claude,
    Codex,
}

/// Metadata for one isolated credential slot — METADATA ONLY, never
/// credentials themselves (those live under the slot's hardened directory,
/// out of band from this struct and never serialized here).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Account {
    pub slug: String,
    pub label: String,
    pub kind: AccountKind,
    pub created_at: String,
    pub is_default: bool,
}

/// One [`Account`]'s stored metadata plus its LIVE auth status
/// (`"logged_in"` / `"logged_out"` / `"unknown"`), as returned by
/// `GET /v1/accounts`. The status is read fresh from the provider CLI on
/// every list call — never persisted.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AccountWithStatus {
    #[serde(flatten)]
    pub account: Account,
    pub status: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct AccountCreateRequest {
    pub slug: String,
    pub label: String,
    pub kind: AccountKind,
}

/// Body of `POST /v1/accounts/{slug}/apikey` — the headless Codex API-key
/// login. `api_key` is piped straight to `codex login --with-api-key` and is
/// never stored, logged, or echoed back by the gateway.
#[derive(Deserialize, JsonSchema)]
pub struct ApiKeyRequest {
    pub api_key: String,
}

/// Server frames on the `GET /v1/accounts/{slug}/login` WebSocket.
#[derive(Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AccountLoginServerMsg {
    LoginUrl { url: String },
    LoginDone,
    LoginNeedsBox,
    Error { message: String },
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatStreamServerMsg {
    Delta {
        text: String,
    },
    AssistantMessage {
        text: String,
    },
    ToolEvent {
        name: String,
        detail: Option<String>,
    },
    TurnDone,
    Error {
        message: String,
    },
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatStreamClientMsg {
    UserMessage { text: String },
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct MissionTask {
    pub title: String,
    pub status: String,
}

/// A mirror of one oracle progress ledger
/// (`~/.omega/state/oracle-<key>.progress.json`) — read-only, never written
/// by the gateway. `title` is the first line of the ledger's free-text
/// `mission` field, truncated to 120 chars.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Mission {
    pub key: String,
    pub project: Option<String>,
    pub title: Option<String>,
    pub done: u32,
    pub total: u32,
    pub tasks: Vec<MissionTask>,
    pub updated_at: String,
}

/// Server-pushed events on the `/v1/events` WebSocket (mission updates,
/// alerts, heartbeat). Emitted by [`crate::events::EventHub`].
///
/// KNOWN LIMIT (V2): `Alert` is only ever emitted by an IN-PROCESS caller
/// (e.g. a test, or a future in-process alert source) — there is no
/// external alert ingestion yet (the real alert path is
/// `~/.omega/bin/omega-alert-send.sh`, which has no local success log to
/// tail). Wiring a real external alert source into the hub is a later plan.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GatewayEvent {
    MissionUpdated { key: String, updated_at: String },
    Alert { message: String, ts: String },
    Heartbeat { ts: String },
}

/// One entry of `GET /v1/rules`'s `laws` array — mirrors
/// `omega_core::rules::Rule` for a Law (`RuleKind::Law`): id, title,
/// category. Laws carry no scope/domain variance worth exposing beyond
/// category (they are universal by invariant).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct LawEntry {
    pub id: String,
    pub title: String,
    pub category: String,
}

/// One entry of `GET /v1/rules`'s `rules` array — mirrors
/// `omega_core::rules::Rule` for an operational Rule (`RuleKind::Rule`).
/// `category` is the `RuleCategory` enum's Debug form (`Universal` /
/// `QualityGate` / `Orchestration` / `Reporting` / `Safety`).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RuleEntry {
    pub id: String,
    pub title: String,
    pub category: String,
    pub added_at: String,
}

/// `GET /v1/rules` response body — the OmegaOS doctrine split into the two
/// SSOT tiers (`omega_core::rules::laws()` / `operational_rules()`), never
/// re-derived by filtering `all_rules()` here.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RulesResponse {
    pub laws: Vec<LawEntry>,
    pub rules: Vec<RuleEntry>,
}

/// One entry of `GET /v1/agents`'s `agents` array — mirrors
/// `omega_core::agents::Agent` 1:1: `name`/`display_name`/`available` map
/// directly to `Agent::name()`/`Agent::display_name()`/`Agent::is_available()`.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AgentEntry {
    pub name: String,
    pub display_name: String,
    pub available: bool,
}

/// `GET /v1/agents` response body — the fixed dispatch-target roster from
/// `omega_core::agents::Agent::all()`.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AgentsResponse {
    pub agents: Vec<AgentEntry>,
}

/// One persistent OmegaOS/AISB teammate. This registry is intentionally
/// distinct from [`AgentEntry`], which represents dispatch-provider engines.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SystemAgentEntry {
    pub name: String,
    pub model: String,
    pub role: String,
    pub tagline: String,
    pub tools: Vec<String>,
    pub responsibilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SystemAgentsResponse {
    pub agents: Vec<SystemAgentEntry>,
}

/// `POST /v1/agents/{name}/install` response body — a pure pre-flight
/// check, never a spawn: confirms `{name}` parses via
/// `omega_core::agents::Agent::from_name` and is installable
/// (`Agent::install_command().is_some()`), and reports whether it is
/// already on PATH (`Agent::is_available()`) so the app can offer an
/// "already installed — reinstall anyway?" prompt before opening the
/// `install/stream` WebSocket (see [`AgentInstallStreamMsg`]).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AgentInstallCheckResponse {
    pub agent: String,
    pub display_name: String,
    pub already_available: bool,
}

/// Server frames on `GET /v1/agents/{name}/install/stream`, which runs
/// `omega install <name>` and streams its output. `Line` tags every line of
/// child output by the pipe it came from (`"stdout"` | `"stderr"`) — the
/// real `omega install` shells out to `curl|sh`/`npm install` and its most
/// useful failure diagnostics often land on stderr, so stderr lines are
/// forwarded exactly like stdout ones, never dropped. `Exit` is always the
/// LAST frame the client receives; `code` is `None` only when the process
/// was terminated by a signal rather than exiting normally. `Error` covers
/// a spawn failure (binary missing) and is followed immediately by the
/// socket closing (no `Exit` frame follows it, since the child never ran).
#[derive(Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentInstallStreamMsg {
    Line { stream: String, text: String },
    Exit { success: bool, code: Option<i32> },
    Error { message: String },
}

/// One entry of `GET /v1/skills`'s `skills` array — mirrors
/// `omega_core::skill_registry::Skill` for display: `category` is
/// `skill.category.label()` (e.g. `"Audit"`/`"Design"`), not a raw Debug
/// form, since the label method exists precisely for this.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
    pub category: String,
}

/// `GET /v1/skills` response body — the OmegaOS skill catalog from
/// `omega_core::skill_registry::SkillRegistry::discover_default()`, optionally
/// narrowed by `?q=`/`?limit=`. `total` is always the UNFILTERED catalog
/// size, so the caller can show "showing N of total" even when `skills` is
/// narrowed.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SkillsResponse {
    pub skills: Vec<SkillEntry>,
    pub total: usize,
}

/// Editable detail for one skill. Absolute host paths are never serialized.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SkillDetail {
    pub name: String,
    pub description: String,
    pub category: String,
    pub content: String,
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SkillDetailResponse {
    pub skill: SkillDetail,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SkillUpdateRequest {
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SkillRenameRequest {
    pub new_name: String,
}

/// Deleting a skill is irreversible, so the caller repeats the exact name it
/// intends to destroy. A mis-click on a list row cannot satisfy this.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SkillDeleteRequest {
    pub confirm_name: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SkillAgentRequest {
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SkillAgentResponse {
    pub session: String,
}

/// One operative system from OmegaOS's compiled registry. `path` stays
/// relative and `bot` is empty when no dedicated bot is linked.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct OsEntry {
    pub slug: String,
    pub name: String,
    pub category: String,
    pub status: String,
    pub path: String,
    pub bot: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct OsResponse {
    pub os: Vec<OsEntry>,
}

/// One entry of `GET /v1/projects`'s `projects` array — a field-for-field
/// mirror of `omega_core::projects::DiscoveredProject`, minus `path` (a full
/// filesystem path is server-internal, not something the wire protocol
/// should leak to a mobile client — the same posture `Account` already
/// takes by never serializing credentials) and minus `score` (an internal
/// ranking heuristic, not product-facing; the response is already
/// best-first sorted, which is all the app needs).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ProjectEntry {
    pub name: String,
    pub container: String,
    pub stack: Vec<String>,
    pub last_active_days: Option<u64>,
}

/// `GET /v1/projects` response body — the auto-discovered project list from
/// `omega_core::projects::discover`, best-first sorted.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ProjectsResponse {
    pub projects: Vec<ProjectEntry>,
}

/// One entry of `GET /v1/marketing`'s `projects` array — a field-for-field
/// mirror of `omega_core::marketing::MarketingProject`, minus `path` (a full
/// filesystem path is server-internal, not something the wire protocol
/// should leak to a mobile client — the SAME posture `ProjectEntry` above
/// already takes for `/v1/projects`, deliberately re-applied here rather
/// than exposing it; `slug` is already the id `omega-zernio`/higgsfield use,
/// so nothing is lost by dropping it). `accounts` is ALWAYS `null` from this
/// endpoint: populating it needs `omega_core::marketing::project_accounts`,
/// which shells out to `omega-zernio` per project — explicitly out of scope
/// for this read-only listing endpoint (on-demand only, never per-frame, per
/// the source struct's own doc comment); emitted as an explicit `null`
/// rather than omitted, so a client sees the field exists and is simply
/// unpopulated here.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct MarketingProjectEntry {
    pub name: String,
    pub slug: String,
    pub has_content: bool,
    pub calendar_posts: usize,
    pub engine_on: bool,
    pub accounts: Option<usize>,
    pub accounts_tried: bool,
    pub has_context: bool,
    pub has_strategy: bool,
    pub has_copy: bool,
    pub has_visual: bool,
    pub has_branding: bool,
}

/// `GET /v1/marketing` response body — every marketing-enabled project (any
/// discovered project with a `<path>/marketing/` directory) from
/// `omega_core::marketing::list_marketing_projects`, name-sorted
/// (case-insensitive) by that function itself.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct MarketingResponse {
    pub projects: Vec<MarketingProjectEntry>,
}

/// One entry of `GET /v1/oracles`'s `oracles` array — a top-level mission
/// ledger ([`Mission`]) composed with its live rmux session status.
///
/// NAMING NOTE: `key` and `session` carry the SAME string. `Mission.key` is
/// populated verbatim from the ledger JSON's `"oracle"` field (see
/// `missions.rs`), and that field already IS the full session name (e.g.
/// `"oracle-dentistrygpt"`) — there is no separate bare identifier anywhere
/// in the ledger-parsing code to derive a shorter `key` from. `key` names
/// the entry's identity, `session` names its role as the exact string
/// checked against `rmux::list_sessions()` for `live`.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct OracleEntry {
    pub key: String,
    pub session: String,
    pub live: bool,
    pub mission: Option<Mission>,
}

/// `GET /v1/oracles` response body — the live oracle roster: every
/// top-level mission ledger from `missions::list()`, each annotated with
/// whether its session currently appears in `rmux::list_sessions()`.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct OraclesResponse {
    pub oracles: Vec<OracleEntry>,
}

/// Body of `POST /v1/dispatch` — mirrors `omega dispatch <PROJECT> <MISSION>
/// [--agent ...] [--new]` 1:1. `agent` and `new` are the exact `Option`s the
/// CLI's own clap flags model (`--agent` is `Option<String>`, `--new` is a
/// bare flag whose ABSENCE means "let the followup-or-spawn router decide",
/// not "force a spawn" — so this is `Option<bool>`, not `bool`, and
/// `routes_dispatch::create` only appends `--new` when it is exactly
/// `Some(true)`).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DispatchRequest {
    pub project: String,
    pub mission: String,
    pub agent: Option<String>,
    pub new: Option<bool>,
}

/// `POST /v1/dispatch` response body — `oracle` is the session name parsed
/// off `omega dispatch`'s stdout line 0 (`"◆ Oracle dispatched: <name>"`,
/// see `omega_core::dispatch::DispatchOutcome::report_lines`), `delivery` is
/// the `DispatchDelivery::tag()` value from the `DISPATCH_DELIVERY=<tag>`
/// line — passed through verbatim as a `String` rather than re-modeled as a
/// gateway-side enum, since the CLI already owns that vocabulary.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DispatchResponse {
    pub oracle: String,
    pub delivery: String,
}

/// Body of `POST /v1/sessions/{name}/keys` — sends keystrokes to a live
/// rmux session. `data` is sent literally (`rmux send-keys -l`); when
/// `enter` is true a separate `rmux send-keys Enter` call follows.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SendKeysRequest {
    pub data: String,
    #[serde(default)]
    pub enter: bool,
}

/// `POST /v1/sessions/{name}/keys` response body.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SendKeysResponse {
    pub ok: bool,
}

/// `POST /v1/sessions/{name}/close` response body — the classification +
/// outcome of running `omega kill <name>` (never `--force`: a REFUSED
/// live-workers case must surface to the app so it can render a strong
/// confirm, never be silently forced by the gateway). `killed` is `true`
/// only on `omega kill`'s own 2xx-equivalent (zero) exit — a REFUSED kill
/// (non-zero exit) is a NORMAL, expected outcome the app needs to render,
/// so the endpoint still answers HTTP 200 with `killed: false` rather than
/// a 502. `message` carries the raw operator-facing text `omega kill`
/// produced (preferring stdout, falling back to stderr — see
/// `routes_sessions::close`'s doc comment for why: the real CLI's REFUSED
/// bail lands on stderr, not stdout, contrary to a naive first read of its
/// call site).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CloseSessionResponse {
    pub killed: bool,
    pub already_closed: bool,
    pub is_oracle: bool,
    pub cascaded_count: u32,
    pub message: String,
}

/// Body of `POST /v1/sessions/{name}/rename`. `new_name` is validated as a
/// safe slug (see `routes_sessions::valid_new_session_name`) BEFORE it ever
/// reaches rmux — rmux silently REWRITES `:`/`.` to `_` rather than
/// rejecting them, and the caller expects `new_name` back verbatim.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RenameSessionRequest {
    pub new_name: String,
}

/// `POST /v1/sessions/{name}/rename` response body — `name` is the
/// already-validated `new_name`, i.e. the session's new effective name.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RenameSessionResponse {
    pub name: String,
}

/// Body of `POST /v1/sessions` — wraps `omega new [OPTIONS] <NAME>`.
/// `agent` is REQUIRED (unlike [`DispatchRequest::agent`]'s optional field):
/// this endpoint always spawns a fresh interactive session, which always
/// needs a real agent to launch. `name` is optional even though the CLI's
/// own `NAME` positional is required — when omitted, the gateway generates
/// one server-side (see `routes_sessions::create`'s doc comment) rather than
/// inventing an "auto" sentinel the CLI itself doesn't support. `--cmd`
/// (arbitrary shell exec) and `--files` (scope-claim) are deliberately NEVER
/// exposed here — too dangerous for an API / out of scope this task.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateSessionRequest {
    pub agent: String,
    pub name: Option<String>,
    pub dir: Option<String>,
    pub prompt: Option<String>,
}

/// `POST /v1/sessions` response body — `name`/`agent` are the
/// already-validated (or server-generated) values the handler chose BEFORE
/// spawning anything, never parsed off `cmd_new`'s own `"Created session:
/// {name}"` stdout line (that line would be redundant with what the caller
/// already knows — the same "we already know it" posture
/// [`RenameSessionResponse`] takes for `new_name`). `output` is the raw
/// stdout on success.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CreateSessionResponse {
    pub name: String,
    pub agent: String,
    pub output: String,
}

/// Body of `POST /v1/team` — wraps `omega team [OPTIONS] <PROJECT>
/// [MEMBERS]...`. `count`, when omitted, falls through to the real CLI's own
/// default (3). `members` are optional `"name:prompt"` specs — when omitted
/// (or empty) the CLI itself spawns `count` generic `worker-N` members.
/// There is deliberately NO `layout` field: the real CLI (`omega team
/// --help`) has no `--layout` flag at all, so this endpoint does not invent
/// one either (ground-truth verified against the live binary; see
/// `.superpowers/sdd/progress.md`'s Task A section).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TeamRequest {
    pub project: String,
    pub count: Option<u32>,
    pub dir: Option<String>,
    pub members: Option<Vec<String>>,
}

/// `POST /v1/team` response body — `session` is the ALREADY-KNOWN spawned
/// session name (`cmd_team` builds it literally as `format!("Team-{project}")`,
/// `crates/omega-cli/src/main.rs::cmd_team` — never parsed off stdout).
/// `output` is the raw stdout: this endpoint does NOT hand-parse the CLI's
/// own per-member text, mirroring wave7's `reap`/`resurrect` "honest raw
/// output" precedent ([`ReapResponse`]/[`ResurrectResponse`]).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct TeamResponse {
    pub session: String,
    pub output: String,
}

/// One session's client-organizational overlay: a folder, a custom label,
/// and a pinned flag. Purely metadata the gateway persists so it survives
/// app restarts and syncs across the operator's devices — it is NEVER
/// validated against a currently-live rmux session, the overlay can freely
/// reference a session that has since closed (see `session_org.rs`'s doc
/// comment). `pinned` is the only field that isn't optional: an entry with
/// no label and no folder is still meaningful once pinned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SessionOrgEntry {
    pub label: Option<String>,
    pub folder: Option<String>,
    #[serde(default)]
    pub pinned: bool,
}

/// `GET /v1/session-org` response body — the whole overlay map, session
/// name -> [`SessionOrgEntry`]. Empty when no session has ever been tagged.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SessionOrgResponse {
    pub entries: std::collections::HashMap<String, SessionOrgEntry>,
}

/// Body of `PUT /v1/session-org/{name}`. Standard HTTP PUT semantics: a
/// FULL REPLACE of that session's overlay entry, never a partial merge with
/// whatever was persisted before — an omitted `label`/`folder` becomes
/// `None`, an omitted `pinned` becomes `false`, exactly as if the entry were
/// newly created. `label`/`folder` are length-capped server-side (see
/// `routes_session_org::MAX_LABEL_LEN` / `MAX_FOLDER_LEN`) before ever
/// reaching the store.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SessionOrgUpdateRequest {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub folder: Option<String>,
    #[serde(default)]
    pub pinned: Option<bool>,
}

/// `POST /v1/deposit` response body — the HTTP twin of the Telegram DEPOSIT
/// bot's reply. `file` is the final timestamped filename written to the
/// inbox, `boxes` lists the named boxes the deposit actually reached (empty
/// when `held` is true), `held` is true when the upload looked like a
/// credential (see `deposit::looks_secret`) and was kept in the inbox only.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DepositResponse {
    pub file: String,
    pub boxes: Vec<String>,
    pub held: bool,
}

/// One entry of `GET /v1/files`'s `entries` array — a directory listing
/// item. `name` is the entry's bare filename (never the full path: the
/// client already knows the `path` it asked for and can join `name` onto
/// it). `size` is the file's byte length, `None` for a directory (a
/// directory's on-disk "size" is filesystem-metadata noise, never a
/// meaningful byte count to a client).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// `GET /v1/files` response body — a scoped directory listing under one
/// discovered project's root (see `routes_files::resolve_scoped_path`).
/// Sorted directories first, then alphabetically (byte order) within each
/// group; this ordering is arbitrary and not the focus of the endpoint.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct FilesResponse {
    pub entries: Vec<FileEntry>,
}

/// `GET /v1/files/read` response body — one file's full text content,
/// UTF-8 decoded. The handler rejects (before ever constructing this type)
/// any file over `routes_files::MAX_FILE_READ_BYTES`, any content
/// containing a NUL byte, and any content that fails UTF-8 decoding — this
/// is a text-only, read-only file-content endpoint, never a binary/diff
/// viewer.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct FileReadResponse {
    pub content: String,
}

/// One entry of `GET /v1/audits`'s `audits` array — mirrors
/// `omega_core::audit::AuditSkill` for display: `domain` is
/// `skill.domain.label()` (e.g. `"Code"`/`"Security"`), not a raw Debug form,
/// matching `SkillEntry::category`'s convention. `max_score` is the raw,
/// per-audit maximum (not `normalized_max`) since that is what `omega audit
/// run`'s own printed metadata line reports and this type mirrors that.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AuditEntry {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub phases: u32,
    pub max_score: u32,
    pub read_only: bool,
}

/// `GET /v1/audits` response body — the Quality Arsenal catalog from
/// `omega_core::audit::all_audits()` (23 forensic audits).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AuditsResponse {
    pub audits: Vec<AuditEntry>,
}

/// Body of `POST /v1/audit` — a pre-flight validation-only check, mirroring
/// `DispatchRequest`'s shape. `kind` is an audit id from `GET /v1/audits`
/// (e.g. `"codeaudit"`), `project` is a name from `GET /v1/projects`. Also
/// doubles as the `?project=&kind=` query-param shape `GET /v1/audit/stream`
/// validates with the exact same two checks before ever upgrading.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AuditRequest {
    pub project: String,
    pub kind: String,
}

/// `POST /v1/audit` response body — echoes back the resolved audit's own
/// metadata (mirrors what `omega audit run`'s own printed banner reports:
/// name, phase count, max score) so a client can render a confirm dialog
/// before opening `GET /v1/audit/stream`. Spawns nothing — the real run only
/// happens over that WebSocket.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AuditCheckResponse {
    pub kind: String,
    pub name: String,
    pub phases: u32,
    pub max_score: u32,
}

/// Server frames on `GET /v1/audit/stream`, which runs `omega audit run
/// <kind> --dir <project_path>` and streams its output — the exact shape of
/// [`AgentInstallStreamMsg`], for the same reasons (`Line` tags every line by
/// the pipe it came from, `Exit` is always the last frame on a completed run,
/// `Error` covers a spawn failure and is followed immediately by the socket
/// closing).
#[derive(Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuditStreamMsg {
    Line { stream: String, text: String },
    Exit { success: bool, code: Option<i32> },
    Error { message: String },
}

/// Server frames on `GET /v1/master/chat` — the WebSocket that mirrors
/// `omega aisb-chat`'s (`crates/omega-cli/src/main.rs::cmd_aisb_chat`) exact
/// file-based protocol: each inbound client text message is one line to
/// inject into `~/.omega/state/aisb-local-inbox.jsonl`, and the server polls
/// `~/.omega/state/aisb-conversation.log` for growth. `NotRunning` means the
/// `aisb-master` rmux session (the read-only viewer that gates whether
/// anyone is watching this conversation at all — see
/// `omega_core::aisb::MASTER_SESSION_NAME`) isn't live, so the inbox was
/// never touched for that message. `Reply` carries the delta text read off
/// the conversation log once it grows. `Timeout` means the poll budget (CLI
/// default: 180 attempts * 500ms = 90s, both overridable via
/// `OMEGA_AISB_POLL_ATTEMPTS`/`OMEGA_AISB_POLL_INTERVAL_MS` for tests)
/// elapsed with no growth — the same "no response within 90s" outcome the
/// CLI itself reports. `Error` covers a rejected inbound client message (see
/// `routes_master::MAX_MASTER_CHAT_MESSAGE_LEN`) — sent instead of touching
/// the inbox, and the loop keeps waiting for the next client message rather
/// than closing, the same "keep the socket open" posture `NotRunning`
/// already documents. Same derive/serde shape as [`AuditStreamMsg`].
#[derive(Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MasterChatMsg {
    NotRunning,
    Reply { text: String },
    Timeout,
    Error { message: String },
}

/// One entry of `GET /v1/doctor`'s `checks` array — one `omega doctor` check
/// line, parsed from its rendered stdout (see
/// `routes_box::parse_doctor_output`). `health` is `"ok"` / `"warn"` /
/// `"fail"`, derived from the check's glyph (`[+]`/`[!]`/`[x]`). `text` is
/// EVERYTHING after the glyph, trimmed — deliberately NOT split into a
/// `name`/`detail` pair, since `omega doctor`'s own `{:16}`-padded name
/// column is a MINIMUM width, not a delimiter (e.g. "binary provenance" is
/// 18 characters, so that boundary is genuinely ambiguous from the text
/// alone; see the parser's doc comment).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DoctorCheckEntry {
    pub health: String,
    pub text: String,
}

/// `GET /v1/doctor` response body — runs bare `omega doctor` (never `--fix`,
/// which mutates state, nor `--deep`, which burns live API quota) and parses
/// its stdout. `overall` is AGGREGATED from the parsed `checks` here
/// (fail-if-any-fail, else warn-if-any-warn, else ok — mirroring
/// `omega_core::doctor::overall()`'s own logic), never re-parsed from the
/// CLI's own trailing summary line, since aggregating the already-parsed
/// checks is more robust than re-matching a second piece of text. A
/// non-zero `omega doctor` exit code (it calls `std::process::exit(1)` on
/// overall `Fail`) is a NORMAL outcome here, not an error — all check lines
/// were already printed before the exit.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DoctorResponse {
    pub overall: String,
    pub checks: Vec<DoctorCheckEntry>,
}

/// `GET /v1/usage` response body — a mirror of `omega_core::monitor::
/// UsageSnapshot::read()`'s live/no-cache distinction. `available` is
/// `false` (and every other field `None`) exactly when the underlying
/// `~/.omega/state/usage.json` cache doesn't exist yet (a normal, expected
/// "no data yet" state — e.g. the `omega usage --check` cron hasn't run) or
/// fails to parse; `true` with every field populated otherwise. Exposes the
/// subset of `UsageSnapshot`'s fields useful for a client's Fleet health
/// view (the 5h/week percentages, token counts, and the account label) —
/// not every field the snapshot carries.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct UsageResponse {
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_pct: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub week_pct: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sonnet_pct: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_pct: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_5h: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_7d: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts: Option<String>,
}

/// `GET /v1/box-info` response body — this box's identity + the gateway
/// process's own liveness. `hostname` shells out to the `hostname` binary
/// (no `hostname` crate in this workspace — same "shell out to a small
/// trusted local binary" convention `routes_agents::kill_process_group`
/// uses for `kill`), `omega_version` is `omega --version`'s trimmed stdout,
/// `gateway_version` is this crate's own `CARGO_PKG_VERSION` (same idiom
/// `server::health` already uses), and `uptime_secs` is the seconds elapsed
/// since `AppState` was constructed (i.e. since this gateway process
/// started serving).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct BoxInfoResponse {
    pub hostname: String,
    pub omega_version: String,
    pub gateway_version: String,
    pub uptime_secs: u64,
}

/// `GET /v1/box-id` response body — this box's STABLE, non-secret identifier
/// (32 lowercase hex chars, `crate::util::random_hex(16)`), generated once
/// on first access and persisted at `<gateway_dir>/box_id.txt` (0600). This
/// is the id the app registers into the Directory's `boxes` table as
/// `boxId` (anywhere-access plan §5.2) — it identifies the BOX, unlike
/// `Device.id` in `auth.rs`, which identifies one PAIRED APP INSTANCE. Not a
/// credential: knowing it grants no access (the endpoint itself is still
/// device-token-guarded like every other protected route), it just needs to
/// be stable across restarts so the same box is recognized as the same box
/// by every device that later pairs with it.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct BoxIdResponse {
    pub box_id: String,
}

/// `POST /v1/cloud/register` request. The Clerk session JWT is accepted only
/// on the existing device-authenticated channel and is never persisted.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CloudRegisterRequest {
    pub clerk_jwt: String,
}

/// `POST /v1/cloud/register` deliberately returns no relay credential. The
/// credential belongs exclusively to the gateway's outbound relay client.
#[derive(Serialize, JsonSchema)]
pub struct CloudRegisterResponse {
    pub ok: bool,
}

/// `POST /v1/backup` response body — `path` is the archive's final on-disk
/// path (parsed off `omega backup`'s own `"  archive : <path>"` stdout
/// line, falling back to the server-chosen `--out` path this endpoint
/// itself passed if that line is somehow missing), `size` is the
/// human-readable size from the `"  size    : <human>"` line (`None` if
/// that line is missing — never treated as an error, since `path` alone is
/// still a usable result). Takes NO client input: the caller supplies
/// nothing, and the server always picks the output path itself (see
/// `routes_box::backup_dir`) — never a client-controlled path.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct BackupResponse {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
}

/// One event of `GET /v1/oracles/{session}/timeline`'s `events` array — a
/// field-for-field mirror of `omega_core::timeline::TimelineEvent`. `at` is
/// RFC 3339 (`DateTime<Utc>::to_rfc3339()`), never the chrono type itself
/// (this crate's wire types stay JSON-schema-friendly strings for anything
/// timestamped — see `ChatMeta::created_at`/`updated_at` for the same
/// convention). `marker` is the raw glyph string (`"◆"`/`"→"`/`"●"`/a
/// done-status glyph) `omega_core::timeline::build` already assigns, passed
/// through verbatim rather than re-modeled as an enum, since `omega
/// timeline`'s own CLI output already treats it as display text, not a
/// closed vocabulary a client would switch on.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct TimelineEventEntry {
    pub at: String,
    pub marker: String,
    pub text: String,
}

/// `GET /v1/oracles/{session}/timeline` response body — a field-for-field
/// mirror of `omega_core::timeline::OracleTimeline`. That type derives only
/// `Debug, Clone` (no `Serialize`/`JsonSchema` — omega-core is a separate
/// crate this endpoint reads in-process but never modifies, R-KARPATHY
/// surgical), so the mapping happens gateway-side in
/// `routes_oracles::timeline_to_response`. A 404 (no `OracleState` on disk
/// for the session) never constructs this type — see
/// `routes_oracles::timeline`.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct TimelineResponse {
    pub oracle_name: String,
    pub project: String,
    pub mission: String,
    pub phase: String,
    pub events: Vec<TimelineEventEntry>,
}

/// One criterion of [`RubricResponse::criteria`] — mirrors
/// `omega_core::gate::RubricCriterion`. `category` is the `CriterionCategory`
/// enum's Debug form (`"Functional"`/`"Quality"`/`"Performance"`/`"Security"`),
/// matching `RuleEntry::category`'s established convention for a plain enum
/// with no `label()` method of its own.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RubricCriterionEntry {
    pub id: String,
    pub description: String,
    pub weight: f32,
    pub category: String,
}

/// A gate rubric with no graded result yet — mirrors `omega_core::gate::
/// Rubric`. `created_at` is RFC 3339. See [`GateStatusResponse`].
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RubricResponse {
    pub mission: String,
    pub criteria: Vec<RubricCriterionEntry>,
    pub created_at: String,
}

/// One grade of [`GateResultResponse::grades`] — mirrors `omega_core::gate::
/// GradeResult`. `verdict` is `GradeVerdict`'s Debug form
/// (`"Satisfied"`/`"NeedsRevision"`/`"Unmet"`/`"Blocked"`).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GateGradeEntry {
    pub criterion_id: String,
    pub verdict: String,
    pub confidence: f32,
    pub evidence: String,
}

/// One multi-grader vote of [`GateResultResponse::consensus_votes`] —
/// mirrors `omega_core::gate::ConsensusVote`.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GateConsensusVoteEntry {
    pub grader: String,
    pub verdict: String,
    pub confidence: f32,
    pub reasoning: String,
}

/// One Popper challenge of [`GateResultResponse::adversarial_challenges`] —
/// mirrors `omega_core::gate::AdversarialChallenge`. `result` is
/// `ChallengeResult`'s Debug form (`"DefectFound"`/`"NoDefect"`/
/// `"Inconclusive"`).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GateAdversarialChallengeEntry {
    pub challenge: String,
    pub result: String,
    pub evidence: String,
}

/// One audit result of [`GateResultResponse::audit_results`] — mirrors
/// `omega_core::audit::AuditResult`. `confidence`/`verdict` are their enums'
/// Debug forms, `completed_at` is RFC 3339.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GateAuditResultEntry {
    pub audit_id: String,
    pub raw_score: f32,
    pub max_score: u32,
    pub normalized_score: f32,
    pub confidence: String,
    pub verdict: String,
    pub findings_count: u32,
    pub critical_findings: u32,
    pub worker_session: Option<String>,
    pub completed_at: String,
}

/// A graded quality-gate result — mirrors `omega_core::gate::GateResult`,
/// flattening its nested `GateDetails` (`grades`/`consensus_votes`/
/// `adversarial_challenges`) directly onto this struct rather than nesting a
/// `details` object one level deeper, since the wire shape has no other
/// consumer of `GateDetails` on its own. `timestamp` is RFC 3339.
/// `accepted_by`/`accepted_evidence` are `Some` only on a HUMAN acceptance
/// (`omega gate <oracle> --accept`, see `GateResult::human_acceptance`'s doc
/// comment) — `None` on every machine-graded result, so a client can always
/// tell a graded pass from an accepted one.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GateResultResponse {
    pub oracle: String,
    pub timestamp: String,
    pub rubric_pass: bool,
    pub consensus_pass: bool,
    pub adversarial_pass: bool,
    pub regression_pass: bool,
    pub audit_results: Vec<GateAuditResultEntry>,
    pub audit_pass: bool,
    pub token_budget_pass: bool,
    pub citation_pass: bool,
    pub overall_pass: bool,
    pub score: f32,
    pub grades: Vec<GateGradeEntry>,
    pub consensus_votes: Vec<GateConsensusVoteEntry>,
    pub adversarial_challenges: Vec<GateAdversarialChallengeEntry>,
    pub accepted_by: Option<String>,
    pub accepted_evidence: Option<String>,
}

/// `GET /v1/oracles/{session}/gate` response body — mirrors `cmd_gate`'s own
/// read-only fallback (`crates/omega-cli/src/main.rs::cmd_gate`, ~line
/// 8865): a graded [`GateResultResponse`] wins when one exists; otherwise the
/// [`RubricResponse`] the gate WILL grade against, if one was created. A 404
/// (neither exists) never constructs this type — see `routes_oracles::gate`.
/// Internally tagged on `status` (`"result"` / `"rubric_only"`) so a client
/// can switch on it directly rather than probing which optional field is
/// present. This endpoint NEVER calls `--accept`/`--mission`/`--approver`/
/// `--evidence` (all state-mutating) — read-only, full stop.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum GateStatusResponse {
    Result(GateResultResponse),
    RubricOnly(RubricResponse),
}

/// `POST /v1/oracles/{session}/reap` response body — wraps `omega reap
/// <session>` (never bare `omega reap`, which sweeps EVERY worker on the
/// box — the path parameter names exactly one session). `reaped` is the
/// CLI's own exit success (`true` on exit 0), NOT a semantic "something was
/// actually reaped" — a session with no terminal done signal is a NORMAL
/// "still working, left alone" outcome that still exits 0, so `output` (the
/// raw stdout) is what actually says what happened; a genuine CLI failure
/// (non-zero exit) is a 502 and never represented by this type — see
/// `routes_oracles::reap`.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ReapResponse {
    pub reaped: bool,
    pub output: String,
}

/// `POST /v1/oracles/{session}/resurrect` response body — wraps `omega
/// resurrect <oracle>` (never bare, for the same per-session reasoning as
/// [`ReapResponse`]). `resurrected` is the CLI's own exit success; `output`
/// (raw stdout) carries the actual per-oracle outcome line (`"resurrected"` /
/// `"already alive"` / `"already finished"` / `"no OracleState"`) — see
/// `routes_oracles::resurrect`.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ResurrectResponse {
    pub resurrected: bool,
    pub output: String,
}

/// Server frames on `GET /v1/orchestrate/stream` — the exact shape of
/// [`AuditStreamMsg`], for the same reasons (`Line` tags every line by the
/// pipe it came from, `Exit` is always the last frame on a completed run,
/// `Error` covers a spawn failure). A DEDICATED type rather than reusing
/// `AuditStreamMsg`, per this crate's established convention: every
/// WS-stream endpoint gets its own wire type even when the shape is
/// identical (compare `AgentInstallStreamMsg` vs `AuditStreamMsg`), so each
/// stays independently versionable.
#[derive(Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OrchestrateStreamMsg {
    Line { stream: String, text: String },
    Exit { success: bool, code: Option<i32> },
    Error { message: String },
}

// ── Task B (wave8): new-project bootstrap stream ─────────────────────────

/// Server frames on `GET /v1/new-project/stream` — the exact shape of
/// [`OrchestrateStreamMsg`] (`Line` tags every line by the pipe it came
/// from, `Exit` is always the last frame on a completed run, `Error` covers
/// a spawn failure). A DEDICATED type rather than reusing
/// `OrchestrateStreamMsg`, per this crate's established convention: every
/// WS-stream endpoint gets its own wire type even when the shape is
/// identical (compare `AgentInstallStreamMsg` vs `AuditStreamMsg` vs
/// `OrchestrateStreamMsg`), so each stays independently versionable.
#[derive(Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NewProjectStreamMsg {
    Line { stream: String, text: String },
    Exit { success: bool, code: Option<i32> },
    Error { message: String },
}

// ── Task C: provider config (`GET`/`PUT /v1/config`) ────────────────────

/// One provider's REDACTED config snapshot — mirrors `omega_core::providers::
/// ClaudeConfig`, except `api_key` never crosses the wire: `api_key_set` is
/// `true`/`false` (non-empty vs empty), never the secret itself. See
/// `routes_config.rs`'s doc comment for why.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ClaudeConfigEntry {
    pub model: String,
    pub effort: String,
    pub api_key_set: bool,
    pub dangerously_skip_permissions: bool,
}

/// Mirrors `omega_core::providers::CodexConfig`, `api_key` redacted (see
/// [`ClaudeConfigEntry`]).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CodexConfigEntry {
    pub model: String,
    pub api_key_set: bool,
    pub base_url: String,
    pub bypass_hook_trust: bool,
}

/// Mirrors `omega_core::providers::GeminiConfig`, `api_key` redacted.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GeminiConfigEntry {
    pub model: String,
    pub api_key_set: bool,
}

/// Mirrors `omega_core::providers::AntigravityConfig`.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AntigravityConfigEntry {
    pub model: String,
    pub effort: String,
    pub dangerously_skip_permissions: bool,
}

/// Mirrors `omega_core::providers::GlmConfig`, `api_key` redacted.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GlmConfigEntry {
    pub model: String,
    pub api_key_set: bool,
}

/// Mirrors `omega_core::providers::OpenRouterConfig`, `api_key` redacted.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct OpenRouterConfigEntry {
    pub model: String,
    pub api_key_set: bool,
    pub base_url: String,
}

/// Mirrors `omega_core::providers::PiConfig`, `api_key` redacted.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct PiConfigEntry {
    pub provider: String,
    pub model: String,
    pub api_key_set: bool,
}

/// Mirrors `omega_core::providers::HermesConfig`, `api_key` redacted.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct HermesConfigEntry {
    pub provider: String,
    pub model: String,
    pub api_key_set: bool,
}

/// Mirrors `omega_core::providers::KimiConfig`, `api_key` redacted.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct KimiConfigEntry {
    pub model: String,
    pub api_key_set: bool,
    pub base_url: String,
    pub provider_type: String,
}

/// `GET /v1/config` and `PUT /v1/config` response body — the full
/// `omega_core::providers::ProvidersConfig` snapshot, every provider's
/// `api_key` redacted to a boolean. Applies to ALL sessions on this box (a
/// single shared `providers.toml`), not per-caller state.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ConfigResponse {
    pub claude: ClaudeConfigEntry,
    pub codex: CodexConfigEntry,
    pub gemini: GeminiConfigEntry,
    pub antigravity: AntigravityConfigEntry,
    pub glm: GlmConfigEntry,
    pub openrouter: OpenRouterConfigEntry,
    pub pi: PiConfigEntry,
    pub hermes: HermesConfigEntry,
    pub kimi: KimiConfigEntry,
}

/// Body of `PUT /v1/config` — one `provider.field` key/value pair, matching
/// `omega config set <key> <value>`'s own CLI shape 1:1. `key` is validated
/// against the exact allowlist `omega-cli`'s `set_config_value` match arms
/// use (see `routes_config.rs::apply_config_value`) — an unknown key is a
/// clean 400, never a silently-ignored no-op.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ConfigSetRequest {
    pub key: String,
    pub value: String,
}

// ── Task D: Telegram bridge control ──────────────────────────────────────

/// `GET /v1/telegram/status` response body — mirrors `omega_core::monitor::
/// OmegaTelegramConfig`, REDACTED: `bot_token` never crosses the wire (same
/// posture [`ClaudeConfigEntry`] takes for `api_key`), only
/// `bot_token_set: bool`. `configured: false` (every other field `None`) is
/// the normal "no `~/.omega/telegram.toml` yet" state — the exact case
/// `TelegramAction::Status`'s CLI arm already renders as "Not configured.",
/// never an error.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct TelegramStatusResponse {
    pub configured: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot_token_set: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relay_session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_user_ids_count: Option<usize>,
}

/// `POST /v1/telegram/enable` / `POST /v1/telegram/disable` response body —
/// the just-written config's new `enabled` state, redacted the same way as
/// [`TelegramStatusResponse`]. A 404 (never this type) when no
/// `telegram.toml` exists yet — mirrors `TelegramAction::Enable`/`Disable`'s
/// own CLI bail ("Not configured. Run: omega telegram setup …").
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct TelegramToggleResponse {
    pub enabled: bool,
}

// ── Task E: PDF generation ───────────────────────────────────────────────

/// Body of `POST /v1/pdf` — `template` is validated against the literal
/// known set (`whitepaper`/`audit`/`marketing`/`doc`) before any subprocess
/// spawns; `data` is arbitrary client-supplied JSON, written server-side to
/// a SERVER-CHOSEN scratch path (never a client-supplied path passed to
/// `--data` — see `routes_pdf.rs`'s doc comment).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct PdfRequest {
    pub template: String,
    pub data: serde_json::Value,
}

/// `POST /v1/pdf` response body — `path` is the generated PDF's absolute
/// on-disk path (the exact value to hand back to `GET /v1/pdf/download?path=`),
/// `size_bytes` is the file's byte length. Never `--send`/`--caption`: this
/// endpoint only generates + returns a path, it never pushes to the
/// operator's real Telegram (that stays a CLI/operator action).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct PdfResponse {
    pub path: String,
    pub size_bytes: u64,
}

// ── wave8 Task D: DUO binome bridge (`POST /v1/duo`) ──────────────────────

/// Body of `POST /v1/duo` — wraps `omega-duo run --task <scratch-file>
/// --cwd <resolved> --mode <mapped>` (see `routes_duo.rs`'s doc comment for
/// the full ground truth read off the real bridge binary). Exactly ONE of
/// `project`/`dir` must be given; `profile` is validated against the closed
/// set `build`/`review`/`reflect` (the `/duo` skill's own three real
/// profiles), mapping 1:1 to the bridge's `--mode` (`code`/`review`/`plan`).
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DuoRequest {
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub dir: Option<String>,
    pub prompt: String,
    pub profile: String,
}

/// Mirrors the bridge's `Capabilities` interface — see `routes_duo.rs`'s
/// doc comment for the full `BridgeResult` contract [`DuoResponse`] mirrors
/// field-for-field.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuoCapabilities {
    pub shell_exec: bool,
    pub worktree_read: bool,
}

/// Mirrors the bridge's `GuardError` interface. `code` is always the
/// literal `"guard-error"` on the wire — kept as a plain `String` rather
/// than a unit enum since this crate never branches on it, only surfaces it
/// to the caller verbatim.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuoGuardError {
    pub code: String,
    pub message: String,
}

/// Mirrors the bridge's `VerifyReport` interface. This endpoint never
/// passes `--verify` (see `routes_duo.rs`'s doc comment), so in practice
/// this is always `None` in a response this wave produces — the field
/// stays on the wire type because the bridge's own JSON contract always
/// includes the key (as `null` when absent) and a future wave may opt in.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuoVerifyReport {
    pub cmd: String,
    pub exit_code: i64,
    pub ok: bool,
    pub timed_out: bool,
    pub tail: String,
}

/// Mirrors the bridge's `Checkpoint` interface. The wire key is literally
/// named `ref` (`#[serde(rename = "ref")]`) — `ref` is a Rust keyword, so
/// the struct field is named `git_ref` instead.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuoCheckpoint {
    pub head: Option<String>,
    pub stash: Option<String>,
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,
}

/// `POST /v1/duo` response body — a field-for-field mirror of the bridge's
/// real `BridgeResult` JSON contract (see `routes_duo.rs`'s doc comment,
/// which cites the exact binary source lines this was read from). Parsed
/// DIRECTLY from `omega-duo run`'s single stdout JSON line via
/// `serde_json::from_str::<DuoResponse>` — every field name matches the
/// bridge's own snake_case keys exactly, so no separate raw/intermediate
/// struct is needed. `ok`/`agent_ok` are the MEANINGFUL success signal,
/// never this endpoint's own HTTP status — see `routes_duo.rs`'s doc
/// comment for why a non-zero `omega-duo` process exit is NOT automatically
/// surfaced as an HTTP error.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuoResponse {
    pub agent: Option<String>,
    pub ok: bool,
    pub output: String,
    pub fell_back: bool,
    pub reason: Option<String>,
    pub exit_code: i64,
    pub log: Option<String>,
    pub sandbox_degraded: bool,
    pub capabilities: DuoCapabilities,
    pub guard_error: Option<DuoGuardError>,
    pub verify: Option<DuoVerifyReport>,
    pub checkpoint: Option<DuoCheckpoint>,
    pub diffstat: Option<String>,
    pub agent_ok: Option<bool>,
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
    pub chat_meta: ChatMeta,
    pub chat_message: ChatMessage,
    pub chat_messages_page: ChatMessagesPage,
    pub chat_detail_response: ChatDetailResponse,
    pub chat_stream_server_msg: ChatStreamServerMsg,
    pub chat_stream_client_msg: ChatStreamClientMsg,
    pub mission: Mission,
    pub mission_task: MissionTask,
    pub gateway_event: GatewayEvent,
    pub account: Account,
    pub account_kind: AccountKind,
    pub account_with_status: AccountWithStatus,
    pub account_create_request: AccountCreateRequest,
    pub api_key_request: ApiKeyRequest,
    pub account_login_server_msg: AccountLoginServerMsg,
    pub law_entry: LawEntry,
    pub rule_entry: RuleEntry,
    pub rules_response: RulesResponse,
    pub agent_entry: AgentEntry,
    pub agents_response: AgentsResponse,
    pub system_agent_entry: SystemAgentEntry,
    pub system_agents_response: SystemAgentsResponse,
    pub agent_install_check_response: AgentInstallCheckResponse,
    pub agent_install_stream_msg: AgentInstallStreamMsg,
    pub skill_entry: SkillEntry,
    pub skills_response: SkillsResponse,
    pub skill_detail: SkillDetail,
    pub skill_detail_response: SkillDetailResponse,
    pub skill_update_request: SkillUpdateRequest,
    pub skill_agent_request: SkillAgentRequest,
    pub skill_agent_response: SkillAgentResponse,
    pub os_entry: OsEntry,
    pub os_response: OsResponse,
    pub project_entry: ProjectEntry,
    pub projects_response: ProjectsResponse,
    pub marketing_project_entry: MarketingProjectEntry,
    pub marketing_response: MarketingResponse,
    pub oracle_entry: OracleEntry,
    pub oracles_response: OraclesResponse,
    pub dispatch_request: DispatchRequest,
    pub dispatch_response: DispatchResponse,
    pub send_keys_request: SendKeysRequest,
    pub send_keys_response: SendKeysResponse,
    pub close_session_response: CloseSessionResponse,
    pub rename_session_request: RenameSessionRequest,
    pub rename_session_response: RenameSessionResponse,
    pub create_session_request: CreateSessionRequest,
    pub create_session_response: CreateSessionResponse,
    pub team_request: TeamRequest,
    pub team_response: TeamResponse,
    pub deposit_response: DepositResponse,
    pub session_org_entry: SessionOrgEntry,
    pub session_org_response: SessionOrgResponse,
    pub session_org_update_request: SessionOrgUpdateRequest,
    pub file_entry: FileEntry,
    pub files_response: FilesResponse,
    pub file_read_response: FileReadResponse,
    pub audit_entry: AuditEntry,
    pub audits_response: AuditsResponse,
    pub audit_request: AuditRequest,
    pub audit_check_response: AuditCheckResponse,
    pub audit_stream_msg: AuditStreamMsg,
    pub master_chat_msg: MasterChatMsg,
    pub doctor_check_entry: DoctorCheckEntry,
    pub doctor_response: DoctorResponse,
    pub usage_response: UsageResponse,
    pub box_info_response: BoxInfoResponse,
    pub box_id_response: BoxIdResponse,
    pub cloud_register_request: CloudRegisterRequest,
    pub cloud_register_response: CloudRegisterResponse,
    pub backup_response: BackupResponse,
    pub timeline_event_entry: TimelineEventEntry,
    pub timeline_response: TimelineResponse,
    pub rubric_criterion_entry: RubricCriterionEntry,
    pub rubric_response: RubricResponse,
    pub gate_grade_entry: GateGradeEntry,
    pub gate_consensus_vote_entry: GateConsensusVoteEntry,
    pub gate_adversarial_challenge_entry: GateAdversarialChallengeEntry,
    pub gate_audit_result_entry: GateAuditResultEntry,
    pub gate_result_response: GateResultResponse,
    pub gate_status_response: GateStatusResponse,
    pub reap_response: ReapResponse,
    pub resurrect_response: ResurrectResponse,
    pub orchestrate_stream_msg: OrchestrateStreamMsg,
    pub new_project_stream_msg: NewProjectStreamMsg,
    pub claude_config_entry: ClaudeConfigEntry,
    pub codex_config_entry: CodexConfigEntry,
    pub gemini_config_entry: GeminiConfigEntry,
    pub antigravity_config_entry: AntigravityConfigEntry,
    pub glm_config_entry: GlmConfigEntry,
    pub openrouter_config_entry: OpenRouterConfigEntry,
    pub pi_config_entry: PiConfigEntry,
    pub hermes_config_entry: HermesConfigEntry,
    pub kimi_config_entry: KimiConfigEntry,
    pub config_response: ConfigResponse,
    pub config_set_request: ConfigSetRequest,
    pub telegram_status_response: TelegramStatusResponse,
    pub telegram_toggle_response: TelegramToggleResponse,
    pub pdf_request: PdfRequest,
    pub pdf_response: PdfResponse,
    pub duo_request: DuoRequest,
    pub duo_capabilities: DuoCapabilities,
    pub duo_guard_error: DuoGuardError,
    pub duo_verify_report: DuoVerifyReport,
    pub duo_checkpoint: DuoCheckpoint,
    pub duo_response: DuoResponse,
}

pub fn schema_json() -> String {
    let schema = schema_for!(Protocol);
    serde_json::to_string_pretty(&schema).expect("serialize schema")
}
