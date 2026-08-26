# Omega Gateway V2 — Chat, Missions, Events Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Extend `omega-gateway` so the app can chat with a real Claude/Codex agent running on the box, see live missions, and receive an event stream (mission done, alerts) — the substance behind the app's Chat / Missions / Inbox screens.

**Architecture:** Chat = a headless `claude -p --output-format stream-json` (or `codex exec`) process per conversation, spawned in the chat's working dir, its NDJSON stdout parsed into typed `ChatStreamServerMsg` frames pushed over a WebSocket. Conversation metadata + transcripts persist as JSON under `~/.omega/gateway/chats/`. Missions are read from the OmegaOS progress ledgers (`oracle-*.progress.json`) — read-only mirror, the gateway does not run oracles. Events = a broadcast channel the box feeds (mission transitions, alerts) delivered on `/v1/events`.

**Tech Stack:** Rust, axum 0.8 (ws), tokio (process, broadcast), serde/serde_json, existing crate deps. New: `tokio` `process` feature (already via `full`), `notify` NOT used (poll ledgers on an interval — simpler, matches R-MONITOR watcher cadence).

## Global Constraints

- Repo `~/Station/SideBusiness/OmegaOS`; work in the provided worktree on branch `omega-gateway-v2`. Sync before merge; commit only your own files.
- Build on the V1 crate: `AppState { dir, cfg }`, protected routes mounted ABOVE the `route_layer` guard comment in `server.rs` (auth via Bearer or `?token=`), the R-STREAM WS discipline (frames, never exit on error except client-gone), typed wire structs in `protocol.rs`.
- Chat process command (CONFIRMED by the scout on claude 2.1.226): `claude -p "<prompt>" --output-format stream-json --verbose` emits NDJSON lines. A `type:"system"` init line carries `session_id`. Assistant text arrives on `type:"assistant"` lines as `message.content[]` blocks of `type:"thinking"` or `type:"text"` (map only `text` blocks to frames; a `thinking` block MAY be surfaced as a `ToolEvent{name:"thinking"}` or ignored — ignore for V1). A final `type:"result"` line (`is_error`, `stop_reason`, `result`) ends the turn. Resume with `-r/--resume <session_id>` (verified: preserves the session, reuses the prompt cache). Model via `--model` (accepts aliases `fable`/`opus`/`sonnet` or full ids). Codex path: `codex exec [--config k=v] <prompt>` with a `resume` subcommand (exists; exact streaming-JSON shape unverified — implement Claude fully, Codex returns `Error{"codex chat not yet supported"}` as a KNOWN LIMIT until a follow-up verifies its output format).
- COST (load-bearing, from the scout): a cold `claude -p` first turn cost **$1.57** because the full OmegaOS skill/slash-command catalog bakes into the system prompt (77k cache-creation tokens); the resumed second turn cost **$0.12** (cache reused). Therefore the gateway MUST persist and reuse `provider_session_id` on every turn after the first (already in the design), and Task 3 SHOULD pass through an optional `--model` so the app can pick a cheaper tier. Do NOT wrap bare `claude -p` per message without resume — that is a $1.57-per-message product. (Investigating a prompt-shrink flag like disabling slash-commands is a Plan 3 optimization, not V2.)
- ACCOUNT MODEL (corrected by the scout): there is NO `~/.omega/accounts/` dir and `CLAUDE_CONFIG_DIR` is not surfaced in `claude --help`; today Claude auth is a single symlinked credentials file. So V2 chat runs under the box's DEFAULT account only — `run_turn`'s `account_dir` param stays in the signature (plumbing for later) but is always `None` in V2, and true multi-account switching (the app's account manager) is deferred to Plan 3. Do not claim multi-account works in V2.
- Chat state dir: `<gateway_dir>/chats/<chat_id>/` with `meta.json` + `transcript.jsonl`; dir 0700, files 0600 (reuse Task-9 `harden_dir`/`harden_file` from auth.rs — move them to a shared `fsperm.rs` module in this plan's Task 1 so chat + auth both use them).
- Chat process safety: cap concurrent chat processes per device at 4; kill the child when the WS closes; a chat turn has a hard wall-clock (default 300s) after which the child is killed and a `turn_done` is sent.
- All code/comments/commits English. After each task: `cargo test -p omega-gateway` green, `cargo clippy -p omega-gateway --all-targets -- -D warnings` clean.

---

### Task 1: Extract shared fs-permission helpers into `fsperm.rs`

**Files:**
- Create: `crates/omega-gateway/src/fsperm.rs`
- Modify: `crates/omega-gateway/src/auth.rs` (use the shared helpers, delete the local `harden_dir`/`harden_file`)
- Modify: `crates/omega-gateway/src/lib.rs` (`pub mod fsperm;`)

**Interfaces:**
- Produces: `fsperm::harden_dir(dir: &Path)` (0700 on unix, no-op elsewhere), `fsperm::harden_file(path: &Path)` (0600 on unix, no-op elsewhere). Both infallible (log on error, never panic) — match the existing behavior in auth.rs exactly.

- [ ] **Step 1: Read the current helpers in auth.rs** to copy their exact bodies (they were added in gateway Task 9). Confirm names and signatures.

- [ ] **Step 2: Write a failing test** in `fsperm.rs` (unix-only): create a tempdir, `harden_dir` it, assert mode `& 0o777 == 0o700`; write a file, `harden_file`, assert `& 0o777 == 0o600`.

Run: `cargo test -p omega-gateway fsperm::` → FAIL (module missing).

- [ ] **Step 3: Implement `fsperm.rs`** by moving the two helpers verbatim (with `#[cfg(unix)]` / `#[cfg(not(unix))]` arms and `use std::os::unix::fs::PermissionsExt;`).

- [ ] **Step 4: Rewire auth.rs** to `use crate::fsperm::{harden_dir, harden_file};` and delete its local copies. The existing auth permission test must still pass unchanged.

- [ ] **Step 5:** `cargo test -p omega-gateway` green, clippy clean.

- [ ] **Step 6: Commit** `refactor(gateway): shared fsperm helpers for 0700/0600 hardening`.

---

### Task 2: Chat store — metadata + transcript persistence

**Files:**
- Create: `crates/omega-gateway/src/chat_store.rs`
- Modify: `crates/omega-gateway/src/lib.rs` (`pub mod chat_store;`)
- Modify: `crates/omega-gateway/src/protocol.rs` (add `ChatMeta`, `ChatMessage`, `ChatAgent`, `ChatStreamServerMsg`, `ChatStreamClientMsg` with JsonSchema + serde tags; add them to the `Protocol` umbrella)

**Interfaces:**
- Produces:
  - `protocol::ChatAgent` enum `{ Claude, Codex }` (`#[serde(rename_all="lowercase")]`).
  - `protocol::ChatMeta { id, title, agent, cwd, created_at, updated_at, provider_session_id: Option<String> }` (provider_session_id is the claude/codex resume id, not exposed as product copy but part of the wire struct).
  - `protocol::ChatMessage { role: String, text: String, ts: String }`.
  - `protocol::ChatStreamServerMsg` (tagged `type` snake_case): `Delta{text}`, `AssistantMessage{text}`, `ToolEvent{name, detail: Option<String>}`, `TurnDone`, `Error{message}`.
  - `protocol::ChatStreamClientMsg` (tagged): `UserMessage{text}`.
  - `chat_store::ChatStore::open(gateway_dir: &Path) -> ChatStore`.
  - `create(&self, agent: ChatAgent, cwd: String, title: Option<String>) -> ChatMeta` (mkdir `<dir>/chats/<id>/` hardened, writes meta.json).
  - `list(&self) -> Vec<ChatMeta>` (sorted updated_at desc).
  - `get(&self, id: &str) -> Option<ChatMeta>`.
  - `append_message(&self, id: &str, msg: &ChatMessage)` (appends to transcript.jsonl, bumps meta.updated_at).
  - `transcript(&self, id: &str) -> Vec<ChatMessage>`.
  - `set_provider_session(&self, id: &str, provider_session_id: &str)`.
  - id = `random_hex(8)` (reuse the helper — expose `auth::random_hex` as `pub(crate)` or move it to a `util.rs`; prefer moving to `util.rs` and re-exporting).

- [ ] **Step 1: Write failing tests** (tempdir-based): create → get roundtrip; append two messages → transcript returns both in order; create bumps nothing but list returns it; set_provider_session persists; second `open` on the same dir sees the created chat (persistence). Assert the chat dir is 0700 and meta.json 0600 (unix).

Run: `cargo test -p omega-gateway chat_store::` → FAIL.

- [ ] **Step 2: Implement `protocol.rs` additions** (the structs/enums above with derives), add to `Protocol` umbrella, and the schema_test from gateway Task 8 must be EXTENDED to assert the new type names appear — update that test.

- [ ] **Step 3: Implement `util.rs`** with `random_hex(n)` moved from auth.rs (auth.rs re-imports it), then `chat_store.rs`.

- [ ] **Step 4:** tests green, clippy clean.

- [ ] **Step 5: Commit** `feat(gateway): chat store — persisted conversation metadata + transcripts`.

---

### Task 3: Chat process driver — spawn `claude -p` and parse NDJSON to typed frames

**Files:**
- Create: `crates/omega-gateway/src/chat_driver.rs`
- Modify: `crates/omega-gateway/src/lib.rs`

**Interfaces:**
- Produces: `chat_driver::run_turn(meta: &ChatMeta, user_text: &str, account_dir: Option<&Path>, timeout: Duration) -> impl Stream<Item = ChatStreamServerMsg>` — spawns the agent process, streams parsed frames, kills the child on drop and on timeout. Concretely, implement as `run_turn(..., tx: tokio::sync::mpsc::Sender<ChatStreamServerMsg>)` (an async fn that sends frames and returns the discovered `provider_session_id: Option<String>` so the caller persists it).
- Produces: `chat_driver::agent_command(meta, user_text, account_dir) -> tokio::process::Command` (pure builder, unit-testable without spawning): for `ChatAgent::Claude` → `claude -p <user_text> --output-format stream-json --verbose [--resume <provider_session_id>] [--model ...]`, `current_dir(meta.cwd)`, env `CLAUDE_CONFIG_DIR=account_dir` when set. For `ChatAgent::Codex` → the codex equivalent (`codex exec` — VERIFY exact flags with the scout; if codex streaming JSON is not confirmed, implement Claude fully and return `Error{message:"codex chat not yet supported"}` for Codex, tracked as a KNOWN LIMIT).
- Produces: `chat_driver::parse_line(line: &str) -> Vec<ChatStreamServerMsg>` + a side-channel for the session id — pure function over one NDJSON line, THE core unit-tested piece. Map: init/system line with `session_id` → capture it (no frame, or a benign one); assistant content text → `Delta` (or `AssistantMessage` if the CLI emits whole messages not deltas — pick per observed shape); tool_use lines → `ToolEvent`; result line → `TurnDone`; unparseable/other → ignored.

- [ ] **Step 1: Write failing unit tests for `parse_line`** using these REAL line shapes (confirmed by the scout on claude 2.1.226 — construct minimal valid fixtures of each):
  - init: `{"type":"system","subtype":"init","session_id":"3d48bb5b-...","cwd":"/tmp","model":"claude-fable-5"}` → `ParsedLine::Session("3d48bb5b-...")`
  - assistant text: `{"type":"assistant","message":{"content":[{"type":"text","text":"PONG"}]},"session_id":"..."}` → `Frame(AssistantMessage{text:"PONG"})` (a `content[]` with a `type:"thinking"` block yields `Ignore`)
  - result: `{"type":"result","is_error":false,"stop_reason":"end_turn","result":"PONG","session_id":"..."}` → `Frame(TurnDone)`
  - a `{"type":"rate_limit_event",...}` or `{"type":"system","subtype":"hook_started"}` line → `Ignore`
  Also a test for `agent_command`: assert the built `Command` (inspect via `get_program`/`get_args`) contains `-p`, `--output-format`, `stream-json`, `--verbose`, and `--resume <id>` when provider_session_id is set, and `--model <m>` when a model is given.

Run: `cargo test -p omega-gateway chat_driver::` → FAIL.

- [ ] **Step 2: Implement `agent_command` and `parse_line`.** Keep `parse_line` pure (no I/O). Get the session_id out via a small `enum ParsedLine { Frame(ChatStreamServerMsg), Session(String), Ignore }` returned by parse_line, so the test asserts on `Session(...)`.

- [ ] **Step 3: Implement `run_turn`** spawning the command with piped stdout, reading lines via `tokio::io::BufReader::lines`, feeding each through `parse_line`, forwarding frames on the mpsc `tx`, capturing the session id, enforcing the timeout with `tokio::time::timeout` around the whole read loop (on timeout: kill child, send `Error` then `TurnDone`), and killing the child on early `tx` closure. Send a final `TurnDone` when the process exits.

- [ ] **Step 4: Integration test with a FAKE agent binary** (same pattern as rmux tests): a bash script that prints canned NDJSON lines then exits; point env `OMEGA_CHAT_BIN` at it (add that override to `agent_command` — default `claude`), drive `run_turn`, collect frames from the mpsc receiver, assert the sequence ends with `TurnDone` and that a mid-stream assistant line produced a text frame. A second fake that sleeps forever proves the timeout kills it and still yields `TurnDone`.

- [ ] **Step 5:** tests green, clippy clean.

- [ ] **Step 6: Commit** `feat(gateway): chat process driver — spawn agent, parse NDJSON to typed frames`.

---

### Task 4: Chat REST + WebSocket routes

**Files:**
- Create: `crates/omega-gateway/src/routes_chat.rs`
- Modify: `crates/omega-gateway/src/server.rs` (mount ABOVE route_layer; AppState gains a `ChatStore` handle + a semaphore for the per-device process cap)
- Modify: `crates/omega-gateway/src/lib.rs`

**Interfaces:**
- Consumes: `ChatStore`, `chat_driver::run_turn`, auth middleware (device in extensions).
- Produces:
  - `GET /v1/chats` → `{"chats":[ChatMeta...]}`.
  - `POST /v1/chats` body `{agent, cwd, title?}` → `201 ChatMeta`.
  - `GET /v1/chats/{id}` → `{"meta":ChatMeta,"messages":[ChatMessage...]}` or 404.
  - `GET /v1/chats/{id}/stream` (WebSocket): client sends `ChatStreamClientMsg::UserMessage{text}`; the gateway persists the user message, runs a turn via `run_turn` (streaming `ChatStreamServerMsg` frames to the socket), persists the assistant message + provider_session_id on `TurnDone`. R-STREAM discipline: parse/agent errors become `Error` frames, only a dead socket or an explicit close ends the loop. The account dir is resolved from the box default profile (Plan: `<gateway_dir>/accounts/default` if present, else None → the box's ambient claude config).

- [ ] **Step 1: Write a failing integration test** using the `OMEGA_CHAT_BIN` fake agent: pair a device, `POST /v1/chats` (agent claude, cwd /tmp), open the WS, send a `UserMessage`, assert a text frame then `TurnDone` arrive, then `GET /v1/chats/{id}` shows the user + assistant messages persisted. Use the same `static LOCK` (tokio::sync::Mutex) env-guard pattern as the rmux tests.

Run → FAIL (routes missing).

- [ ] **Step 2: Extend AppState** with `chats: ChatStore` (constructed in `build_router` from `state.dir`) and a `tokio::sync::Semaphore` (Arc) sized to the per-device cap × a small fleet — simplest: a global cap of 8 concurrent chat turns, acquire a permit around each turn, send `Error{"busy"}` if none available.

- [ ] **Step 3: Implement `routes_chat.rs`** (the 3 REST handlers + the WS handler), mount all four routes above the route_layer guard.

- [ ] **Step 4:** tests green, clippy clean.

- [ ] **Step 5: Commit** `feat(gateway): chat REST + WebSocket routes, streaming agent turns`.

---

### Task 5: Missions mirror — read oracle progress ledgers

**Files:**
- Create: `crates/omega-gateway/src/missions.rs`
- Create: `crates/omega-gateway/src/routes_missions.rs`
- Modify: `crates/omega-gateway/src/server.rs`, `lib.rs`, `protocol.rs` (add `Mission`, `MissionTask` to wire + umbrella + schema test)

**Interfaces:**
- Produces:
  - `protocol::MissionTask { title: String, status: String }`, `protocol::Mission { key: String, project: Option<String>, title: Option<String>, done: u32, total: u32, tasks: Vec<MissionTask>, updated_at: String }`.
  - `missions::ledger_dir() -> PathBuf` (env `OMEGA_STATE_DIR` else `~/.omega/state`).
  - REAL LEDGER SCHEMA (confirmed by the scout, `~/.omega/state/oracle-<key>.progress.json`): top-level `{ oracle: String, project: String, mission: String, done: u32, total: u32, ts: String, tasks: [ {s, t, updated_at} ], bot, chat, thread, msgId }`. The parser maps: `oracle`→`key`, `project`→`project`, first line of `mission`→`title` (the mission text can be long and carry another project's client-facing content — expose it to the box owner as-is, that is their own box, but truncate `title` to the first line / 120 chars), `ts`→`updated_at`, and each task `{s→status, t→title}`. IGNORE the `bot/chat/thread/msgId` Telegram-render coordinates. Deserialize with `#[serde(rename="s")]`/`#[serde(rename="t")]` into an internal struct, then map to the wire `MissionTask`.
  - `missions::list() -> Vec<Mission>` (glob `oracle-*.progress.json` in the ledger dir — INCLUDE `oracle-*-worker-*.progress.json` too? No: list top-level oracles only, filter out `*-worker-*` so the app shows missions not individual workers; a worker's progress is a drill-down for later). Parse each, sorted `updated_at` desc. Tolerate malformed/foreign JSON: skip with a warn, never panic.
  - `GET /v1/missions` → `{"missions":[Mission...]}` (protected).

- [ ] **Step 1: Write failing tests**: write two fake `oracle-X.progress.json` files (using the REAL schema from the scout report) into a temp `OMEGA_STATE_DIR`, assert `list()` returns both parsed with the right task statuses; write one malformed json, assert it is skipped not fatal. Integration test: `GET /v1/missions` returns them for an authed device.

Run → FAIL.

- [ ] **Step 2: Implement** `missions.rs` (parser adapted to the real ledger schema) + `routes_missions.rs`, mount above route_layer.

- [ ] **Step 3:** tests green, clippy clean.

- [ ] **Step 4: Commit** `feat(gateway): missions mirror from oracle progress ledgers, GET /v1/missions`.

---

### Task 6: Event stream — `/v1/events` WebSocket (missions + alerts)

**Files:**
- Create: `crates/omega-gateway/src/events.rs`
- Modify: `crates/omega-gateway/src/routes_missions.rs` or a new `routes_events.rs`, `server.rs`, `lib.rs`, `protocol.rs` (add `GatewayEvent` to wire + umbrella + schema test)

**Interfaces:**
- Produces:
  - `protocol::GatewayEvent` (tagged `type` snake_case): `MissionUpdated{key, updated_at}`, `Alert{message, ts}`, `Heartbeat{ts}`.
  - `events::EventHub` — wraps a `tokio::sync::broadcast::Sender<GatewayEvent>`; `EventHub::new()`, `subscribe()`, `emit(ev)`.
  - A background task (spawned in `build_router` or `main serve`) that polls the missions ledger dir every `cfg.stream_interval_ms`×N and emits `MissionUpdated` when a ledger's updated_at changes (diff against a cached map — R-MONITOR "one variable per signal" discipline, no re-fire on frozen state), plus a `Heartbeat` every ~30s.
  - `GET /v1/events` (WebSocket, protected): subscribes and forwards every `GatewayEvent` as a JSON text frame; never exits on lag error (on `RecvError::Lagged` send a resync hint and continue), only on dead socket.

- [ ] **Step 1: Write a failing test**: construct an `EventHub`, subscribe, `emit` an `Alert`, assert the subscriber receives it. Integration test: authed device opens `/v1/events`, another task calls `hub.emit(Alert)`, assert the frame arrives on the socket. (Wire the hub into AppState so the test can reach it, or expose a test-only emit endpoint — prefer wiring the hub into AppState and calling emit directly in the test via a shared handle.)

Run → FAIL.

- [ ] **Step 2: Implement** `events.rs`, wire `EventHub` into AppState, spawn the poller, implement `/v1/events`.

- [ ] **Step 3:** tests green, clippy clean.

- [ ] **Step 4: Commit** `feat(gateway): /v1/events WebSocket — mission updates, alerts, heartbeat`.

---

### Task 7: Wire everything + reproducibility check

**Files:**
- Modify: `crates/omega-gateway/src/main.rs` (ensure `serve` starts the event poller; add a `chats` CLI subcommand listing chats, symmetry with `devices`)
- Modify: `docs/` or crate README as needed

- [ ] **Step 1:** Runtime verification (L1): rebuild, reinstall the daemon, restart the service, then with a paired device token drive the real endpoints from `curl`/`websocat`: `POST /v1/chats`, open the chat WS with a real `claude` turn (this actually talks to the box's Claude — expect a real assistant reply frame then TurnDone), `GET /v1/missions`, `GET /v1/events`. Capture the outputs in the report.

- [ ] **Step 2:** `cargo test -p omega-gateway` full green + both clippy forms clean + `cargo build --release`.

- [ ] **Step 3: Commit** `feat(gateway): start event poller in serve, chats CLI subcommand`.

- [ ] **Step 4:** After the whole-branch review passes: fetch+rebase origin/main, ff-merge to main, push, post-merge runtime re-verify, remove the worktree.

---

## Out of scope (later plans)

- Push relay to APNs / the Agentik cloud (Plan 3).
- Rate limiting on /v1/pair and per-device stream caps beyond the global chat semaphore (hardening pass).
- Multi-account add-from-app device-auth relay (uses the account dir plumbing this plan lays down, but the interactive login flow is Plan 3).
