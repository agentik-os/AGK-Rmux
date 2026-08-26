# Omega Gateway — OmegaOS Control-Plane Surface (rules, agents, skills, projects, dispatch)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Extend `omega-gateway` so the Omega App can read the OmegaOS doctrine (Laws + Rules), the dispatch-agent roster, the skill catalog, the discovered-project list, and the live oracle roster — and can launch a new oracle mission. Five new read endpoints + one mutating endpoint, all under the existing device-auth guard.

**Architecture, decided from reading the actual source (not guessed):**

- **Pure, no-daemon data → depend on `omega-core` directly, in-process.** `crates/omega-core/src/rules.rs` (`all_rules()`, `laws()`, `operational_rules()`), `projects.rs` (`discover(home: &Path)`), `agents.rs` (`Agent::all()`, `Agent::is_available()`, `Agent::name()`/`display_name()`), and `skill_registry.rs` (`SkillRegistry::discover_default()` / `discover(dir)`) are **plain, synchronous, side-effect-free functions** — no rmux daemon, no network, no process spawn. `omega-cli`'s own `cmd_projects(json)` (main.rs:3463) calls `omega_core::projects::discover(&home)` and serializes it verbatim, and `cmd_agents()` (main.rs:4704) calls `omega_core::agents::Agent::all()` + `is_available()` — so depending on omega-core and calling the SAME functions is exact CLI parity, not a reinvented shape, and matches R-CLI's instruction to prefer a clean Rust API over scraping `omega rules list`'s text table.
- **Anything needing the live rmux daemon → shell out, exactly like the existing `rmux.rs` module already does.** `crates/omega-gateway/src/rmux.rs` does NOT depend on `rmux-sdk`/omega-core's `SessionManager` — it shells to the `rmux` binary (`Command::new(rmux_bin()).args(&[...])`) because a daemon connection is stateful, async infrastructure the gateway process doesn't want to own. `omega dispatch`'s implementation (`cmd_dispatch`, main.rs:6096) constructs `omega_core::dispatch::Dispatcher::new(SessionManager::connect().await?, config)` — `SessionManager::connect()` (session.rs:246) calls `Rmux::builder().connect_or_start()`, i.e. it can **spawn the rmux daemon as a side effect**. Duplicating that inside axum handlers would mean the gateway owns a second daemon-connection lifecycle alongside the CLI's, for a single mutating endpoint — not surgical (R-KARPATHY), and exactly what R-CLI says to avoid when a CLI already exists. So `POST /v1/dispatch` and `GET /v1/oracles` **shell out to the `omega` binary** via a new `omega_cli.rs` module, argv-only (never a shell string), mirroring `rmux.rs`'s own idiom (`OMEGA_BIN` env override for tests, exactly like `OMEGA_RMUX_BIN`). `omega dispatch`'s stdout has a documented, stable, machine-parseable contract (`DispatchOutcome::report_lines()`: line 0 is `"Oracle dispatched: <name>"`, the last line is `DISPATCH_DELIVERY=<tag>` — the SAME contract the Telegram bridge already parses), so the gateway parses that contract rather than inventing a new one.
- **Dependency cost is real but bounded, and mostly already paid.** `omega-core`'s `Cargo.toml` pulls `rmux-sdk`/`rmux-proto` (git), `rusqlite` (bundled SQLite — compiles vendored C), `walkdir`, `blake3`, `globset`, `serde_yaml`, `serde-saphyr`, `semver`, `toml_edit`, `chrono-tz`, `unicode-normalization`. This is a genuine compile-time and binary-size increase for `omega-gatewayd`. BUT: `crates/omega-cli` and `crates/omega-tui` are in the SAME cargo workspace and already depend on `omega-core` — so this doesn't add a new crate to the workspace's dependency graph or `Cargo.lock`, it links an already-built rlib into a 4th binary. No new runtime daemon connection is introduced: none of `rules::`, `projects::`, `agents::`, `skill_registry::` construct a `SessionManager`/`Rmux` client. Record this tradeoff explicitly in the PR description; do not silently absorb it.
- **Skills catalog source, confirmed on-box (not assumed):** `~/.omega/skills/` has 356 top-level entries, 341 with a `SKILL.md` reachable by `SkillRegistry::discover`'s exact recursion rule (top-level dir OR one level of grouping). `~/.claude/skills/` has 436 entries but only 59 `SKILL.md` at that same depth — most of the rest are nested skill-pack directories (e.g. the vendored 130+-skill design-intelligence pack) that `SkillRegistry`'s one-level recursion does not flatten. `omega skills validate|compile` (`resolve_skill_root`, main.rs:9839) defaults to `<repo>/skills` or `~/.omega/skills` — never `~/.claude/skills`. So `GET /v1/skills` reads `~/.omega/skills/` via `SkillRegistry::discover_default()`, matching the CLI's own SSOT; note in the endpoint doc-comment that this is deliberately narrower than the raw `~/.claude/skills` listing.
- **Agent roster scope, deliberately narrow:** `GET /v1/agents` returns the **dispatch-target roster** (`omega_core::agents::Agent`: claude/codex/gemini/pi/hermes/glm/kimi/shell — what `--agent` on `omega dispatch` accepts and what `omega agents` lists), NOT the 11 AISB persona files under `agents/*.md` (oracle.md, aisb-master.md, …) — those are internal role-prompt templates injected by the dispatch pipeline, not something an operator picks from a UI list. Conflating the two is scope creep past what the App's agent picker needs; note it as a later, separate concern if the App ever wants to expose oracle personas.

**Tech Stack:** Rust, axum 0.8, tokio (`spawn_blocking` for the new synchronous omega-core calls and for subprocess calls — same pattern `missions::list()` and `rmux::list_sessions()` already use), serde/serde_json, schemars (existing `Protocol` umbrella). New workspace-internal dependency: `omega-core = { workspace = true }` added to `crates/omega-gateway/Cargo.toml`. No new external crates.

## Global Constraints

- Repo `~/Station/SideBusiness/OmegaOS`; work in the provided worktree on branch `omega-gateway-surface`. Sync before merge (`git fetch origin && git rebase origin/main`); commit only files this plan touches.
- Every new protected route is added to `server.rs` **ABOVE** the `route_layer` guard comment — a route added below it ships unauthenticated. This has bitten this crate before (see the comment already in `server.rs:115-117`); do not re-derive the mistake.
- Every new wire type goes into `protocol.rs`, derives `Serialize`/`Deserialize` (as needed) + `JsonSchema`, is added to the `Protocol` umbrella struct, and `tests/schema_test.rs` gets the new type names appended to its assertion list (extend the existing list, don't replace it).
- `omega-core` calls that touch the filesystem (rules/projects/agents/skills are all synchronous, blocking) run inside `tokio::task::spawn_blocking`, exactly like `crate::missions::list` is called in `routes_missions.rs`.
- `omega_cli.rs`'s subprocess calls follow `rmux.rs`'s exact shape: `Command::new(omega_bin()).args(&[...])` (argv array, **never** a shell string — no injection surface even from a malicious project/mission string), bounded output capture, non-zero exit surfaced as a typed error, never a panic.
- Test env-var overrides (`OMEGA_BIN` for the new module, alongside the existing `OMEGA_RMUX_BIN`) are process-global, so any test that sets them reuses the `static LOCK: tokio::sync::Mutex<()>` pattern already in `tests/sessions_test.rs` / `tests/stream_test.rs` (or, if omega_cli.rs's own unit tests set it, an equivalent local lock) — never let two tests race on the same env var.
- `POST /v1/dispatch` is the ONE mutating endpoint added by this plan: validate `project` against the real discovered-projects list (`projects::discover`) BEFORE spawning anything (unknown project → 400, no subprocess launched), pass `project`/`mission`/`agent` as separate argv elements to `omega dispatch` (never interpolated into a shell string), and never fabricate a session id from a failed dispatch.
- All code/comments/commits English (R-STYLE). After each task: `cargo test -p omega-gateway` green, `cargo clippy -p omega-gateway --all-targets -- -D warnings` clean, and (once `omega-core` is added) `cargo build -p omega-gateway` succeeds cleanly the first time the dependency is wired (Task 1's own acceptance).
- NEVER launch a real oracle from a test. `POST /v1/dispatch` tests point `OMEGA_BIN` at a fake `omega` script that records its argv to a file and prints a canned `report_lines()`-shaped stdout; assert the recorded argv, never touch a real rmux daemon.

---

### Task 1: Add the `omega-core` dependency and prove it links clean

**Files:**
- Modify: `crates/omega-gateway/Cargo.toml` (add `omega-core = { workspace = true }` under `[dependencies]`)
- Modify: `crates/omega-gateway/src/lib.rs` (no new `pub mod` yet — this task only proves the link)

**Interfaces:** none new; this task is a build-health gate the rest of the plan depends on.

- [ ] **Step 1:** Add the dependency line. Run `cargo build -p omega-gateway` and time it (`time cargo build -p omega-gateway`) — record the delta in the task's commit message (compare against a clean `cargo build -p omega-gateway` on `main` before this change, e.g. via `git stash`). This is the concrete evidence for the "dependency cost is real but bounded" claim above — don't leave it asserted, verify it.
- [ ] **Step 2:** `cargo test -p omega-gateway` — must still be 100% green (adding an unused dependency changes nothing behaviorally; this just confirms the crate still compiles and links against the workspace's `omega-core` rlib without version conflicts).
- [ ] **Step 3: Commit** `build(gateway): depend on omega-core for the doctrine/projects/agents/skills read surface`.

---

### Task 2: `GET /v1/rules` — Laws + Rules from `omega-core::rules`

**Files:**
- Create: `crates/omega-gateway/src/routes_rules.rs`
- Modify: `crates/omega-gateway/src/lib.rs` (`pub mod routes_rules;`)
- Modify: `crates/omega-gateway/src/protocol.rs` (add `LawEntry`, `RuleEntry`, `RulesResponse`; add to `Protocol` umbrella)
- Modify: `crates/omega-gateway/src/server.rs` (route, above the guard)
- Modify: `crates/omega-gateway/tests/schema_test.rs`
- Create: `crates/omega-gateway/tests/rules_test.rs`

**Interfaces:**
- `protocol::LawEntry { id: String, title: String, category: String }` — Laws have no `RuleCategory`-scoped variance worth exposing beyond category; `id`/`title` come straight off `omega_core::rules::Rule.id`/`.title` (both `&'static str`, clone to `String` for the wire type).
- `protocol::RuleEntry { id: String, title: String, category: String, added_at: String }` — `category` is `format!("{:?}", rule.category)` (the `RuleCategory` enum's Debug form: `Universal`/`QualityGate`/`Orchestration`/`Reporting`/`Safety` — matches the categories already visible in `CLAUDE.md`'s own "Laws vs Rules" section, so this is not an invented taxonomy).
- `protocol::RulesResponse { laws: Vec<LawEntry>, rules: Vec<RuleEntry> }`.
- `routes_rules::list(State(AppState)) -> Json<RulesResponse>`: `tokio::task::spawn_blocking(|| { let laws = omega_core::rules::laws(); let rules = omega_core::rules::operational_rules(); ... })`. **Do not** call `all_rules()` and filter by `kind` yourself — `laws()`/`operational_rules()` already exist as the exact split (rules.rs:930, rules.rs:938); use the SSOT split, don't re-derive it.

- [ ] **Step 1: Write a failing integration test** `rules_test.rs`: spin up the router (no fake bin needed — this is pure in-process data), GET `/v1/rules` with a valid device token, assert `laws` has exactly 7 entries (`L0`..`L6`) and `rules` has at least 40 entries (the doc says ~50; assert a lower bound, not an exact count, so the test doesn't break every time a rule is added), and assert one KNOWN law (`"L0"`) and one KNOWN rule (`"R-CLI"`) are present by id.

Run: `cargo test -p omega-gateway rules_test` → FAIL (route missing).

- [ ] **Step 2:** Add `LawEntry`/`RuleEntry`/`RulesResponse` to `protocol.rs` + `Protocol` umbrella; implement `routes_rules.rs`; wire `.route("/v1/rules", get(crate::routes_rules::list))` into `server.rs` ABOVE the `route_layer` line.
- [ ] **Step 3:** Extend `schema_test.rs`'s type-name list with `"LawEntry"`, `"RuleEntry"`, `"RulesResponse"`.
- [ ] **Step 4:** `cargo test -p omega-gateway` green, clippy clean.
- [ ] **Step 5: Runtime verify (L1):** with the daemon running locally, `curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:4477/v1/rules | jq '.laws | length, .rules | length'` — paste the real output into the task's commit message or the plan's progress log, not a predicted number.
- [ ] **Step 6: Commit** `feat(gateway): GET /v1/rules — Laws + Rules from omega-core::rules`.

---

### Task 3: `GET /v1/agents` — the dispatch-target agent roster

**Files:**
- Create: `crates/omega-gateway/src/routes_agents.rs`
- Modify: `crates/omega-gateway/src/lib.rs`, `protocol.rs` (add `AgentEntry`, `AgentsResponse`), `server.rs`, `tests/schema_test.rs`
- Create: `crates/omega-gateway/tests/agents_test.rs`

**Interfaces:**
- `protocol::AgentEntry { name: String, display_name: String, available: bool }` — `name`/`display_name`/`available` map 1:1 to `Agent::name()`, `Agent::display_name()`, `Agent::is_available()`.
- `protocol::AgentsResponse { agents: Vec<AgentEntry> }`.
- `routes_agents::list() -> Json<AgentsResponse>`: `spawn_blocking(|| omega_core::agents::Agent::all().iter().map(|a| AgentEntry { name: a.name().into(), display_name: a.display_name().into(), available: a.is_available() }).collect())`. `is_available()` does a PATH lookup per agent (cheap, ~8 calls) — no network.

- [ ] **Step 1: Failing test:** GET `/v1/agents`, assert exactly 8 entries (`Agent::all()` is a fixed, hard-coded 8-element slice — an exact-count assertion is correct here, unlike the rules count), and assert `"claude"` is present with a boolean `available` field (don't assert its VALUE — availability is environment-dependent, that would make the test flaky on a box without `claude` on PATH).

Run → FAIL.

- [ ] **Step 2:** Implement + wire, above the guard.
- [ ] **Step 3:** Extend `schema_test.rs`.
- [ ] **Step 4:** Tests green, clippy clean.
- [ ] **Step 5: Runtime verify:** `curl … /v1/agents | jq '.agents[] | select(.name=="claude")'` — confirm `available:true` on this box (Claude is installed here) and paste the real response.
- [ ] **Step 6: Commit** `feat(gateway): GET /v1/agents — the dispatch-target roster`.

---

### Task 4: `GET /v1/skills` — the OmegaOS skill catalog, with `?q=` filter + `?limit=`

**Files:**
- Create: `crates/omega-gateway/src/routes_skills.rs`
- Modify: `lib.rs`, `protocol.rs` (add `SkillEntry`, `SkillsResponse`), `server.rs`, `tests/schema_test.rs`
- Create: `crates/omega-gateway/tests/skills_test.rs`

**Interfaces:**
- `protocol::SkillEntry { name: String, description: String, category: String }` — `category` is `skill.category.label()` (the existing human-readable label method on `SkillCategory`, e.g. `"Audit"`/`"Design"` — reuse it rather than a raw Debug string, since it already exists precisely for display).
- `protocol::SkillsResponse { skills: Vec<SkillEntry>, total: usize }` — `total` is the UNFILTERED count, so the App can show "showing 40 of 341" when `?q=`/`?limit=` narrow the returned `skills`.
- `routes_skills::list(Query(params): Query<HashMap<String,String>>) -> Json<SkillsResponse>`: `spawn_blocking(|| SkillRegistry::discover_default())`, then filter by `q` (case-insensitive substring match against `name` OR `description`, only when `q` is present and non-empty) and cap by `limit` (parse as `usize`, default 50, hard ceiling 200 — never let an unbounded `?limit=` return the entire catalog uncapped and never let a bad/non-numeric `limit` value panic, fall back to the default).
- `SkillRegistry::discover_default()` returns `Result<Self>`; a discovery failure (e.g. dir unreadable) degrades to an empty `skills: [], total: 0` response with a `tracing::warn!`, same "never 500 on a degraded read" posture `routes_sessions::list` already uses for a failed `rmux` call.

- [ ] **Step 1: Failing test:** GET `/v1/skills` with no query params, assert `total >= 300` (a loose lower bound — 341 confirmed on this box, but don't hard-code an exact count a future skill addition would break) and `skills.len() <= 50` (default cap). GET `/v1/skills?q=audit&limit=5`, assert `skills.len() <= 5` and every returned `name`/`description` case-insensitively contains `"audit"`.

Run → FAIL.

- [ ] **Step 2:** Implement + wire, above the guard.
- [ ] **Step 3:** Extend `schema_test.rs`.
- [ ] **Step 4:** Tests green, clippy clean.
- [ ] **Step 5: Runtime verify:** `curl … '/v1/skills?q=audit&limit=5' | jq` — paste real output; confirm `total` matches (roughly) the 341 figure found during investigation, and note in the commit if it has drifted (a drift is expected over time, not a bug).
- [ ] **Step 6: Commit** `feat(gateway): GET /v1/skills — filtered, capped catalog read`.

---

### Task 5: `GET /v1/projects` — discovered projects

**Files:**
- Create: `crates/omega-gateway/src/routes_projects.rs`
- Modify: `lib.rs`, `protocol.rs` (add `ProjectEntry`, `ProjectsResponse`), `server.rs`, `tests/schema_test.rs`
- Create: `crates/omega-gateway/tests/projects_test.rs`

**Interfaces:**
- `protocol::ProjectEntry { name: String, container: String, stack: Vec<String>, last_active_days: Option<u64> }` — a direct field-for-field mirror of `omega_core::projects::DiscoveredProject`, minus `path` (a full filesystem path is server-internal, not something the wire protocol should leak to a mobile client — same posture `Account` already takes by never serializing credentials) and minus `score` (an internal ranking heuristic, not product-facing; the response is already best-first sorted, which is all the App needs).
- `protocol::ProjectsResponse { projects: Vec<ProjectEntry> }`.
- `routes_projects::list() -> Json<ProjectsResponse>`: `spawn_blocking(|| { let home = dirs::home_dir()...; omega_core::projects::discover(&home) })`, mapped to `ProjectEntry`.

- [ ] **Step 1: Failing test:** GET `/v1/projects`, assert `projects.len() >= 10` (this box has 49 discovered — a loose lower bound again, machine-dependent), and assert the response is a JSON array under `projects` with `name`/`container`/`stack` fields present on the first entry.

Run → FAIL.

- [ ] **Step 2:** Implement + wire, above the guard.
- [ ] **Step 3:** Extend `schema_test.rs`.
- [ ] **Step 4:** Tests green, clippy clean.
- [ ] **Step 5: Runtime verify:** `curl … /v1/projects | jq '.projects | length'` — paste the real count (expect roughly 49, per the ground truth in the brief) and one full entry.
- [ ] **Step 6: Commit** `feat(gateway): GET /v1/projects — the discovered-project list`.

---

### Task 6: `omega_cli.rs` — the subprocess wrapper module (shared by Tasks 7 + 8)

**Files:**
- Create: `crates/omega-gateway/src/omega_cli.rs`
- Modify: `lib.rs` (`pub mod omega_cli;`)

**Interfaces:**
- `omega_cli::omega_bin() -> PathBuf` — `OMEGA_BIN` env override, else `dirs::home_dir().join(".local/bin/omega")` (same shape as `rmux::rmux_bin()`; confirm the real installed path with `which omega` during Step 1 and use that as the production default — do not guess a path Task 1's investigation didn't confirm).
- `omega_cli::run(args: &[&str]) -> Result<CommandOutput>` where `CommandOutput { stdout: String, stderr: String, success: bool }` — captures both streams (never merges them), never panics on a non-zero exit (that's a normal outcome the caller inspects, e.g. "unknown project"), only errors on a spawn failure (binary missing/not executable).
- This module has NO route handlers and NO knowledge of dispatch/oracle semantics — it is a thin, generic "run the omega binary with these args, hand back stdout/stderr/success" primitive, exactly as `rmux.rs` is generic over `rmux` subcommands. Tasks 7 and 8 build the typed logic on top.

- [ ] **Step 1:** `which omega` on this box; confirm/record the real path (likely `~/.local/bin/omega`, matching every other OmegaOS component's PATH convention — `rmux_bin()` uses the identical `~/.local/bin/<name>` shape).
- [ ] **Step 2: Failing unit test** (in `omega_cli.rs`'s own `#[cfg(test)] mod tests`, with a local `static LOCK`): write a fake `omega` script to a tempdir printing fixed stdout/stderr and exiting 0, point `OMEGA_BIN` at it, call `run(&["projects", "--json"])`, assert `stdout` matches and `success == true`. A second test: fake script exits 1 with stderr text, assert `success == false` and `stderr` is captured (not silently dropped).

Run → FAIL.

- [ ] **Step 3:** Implement `omega_bin()` + `run()`.
- [ ] **Step 4:** Tests green, clippy clean.
- [ ] **Step 5: Commit** `feat(gateway): omega_cli subprocess wrapper (argv-only, mirrors rmux.rs)`.

---

### Task 7: `GET /v1/oracles` — live oracle roster (session liveness + progress ledger)

**Files:**
- Create: `crates/omega-gateway/src/routes_oracles.rs`
- Modify: `lib.rs`, `protocol.rs` (add `OracleEntry`, `OraclesResponse`), `server.rs`, `tests/schema_test.rs`
- Create: `crates/omega-gateway/tests/oracles_test.rs`

**Interfaces:**
- `protocol::OracleEntry { key: String, session: String, live: bool, mission: Option<Mission> }` — `session` is the full `oracle-<key>` session name; `live` is whether that name currently appears in `rmux::list_sessions()`; `mission` reuses the ALREADY-EXISTING `protocol::Mission` type (from `missions.rs`, Task-nothing — it shipped in V2) for the ledger data, so this endpoint does NOT duplicate the ledger-parsing logic, it composes `missions::list()` (which already returns `Vec<Mission>`, one per top-level `oracle-*.progress.json`) with `rmux::list_sessions()` for the liveness bit.
- `protocol::OraclesResponse { oracles: Vec<OracleEntry> }`.
- `routes_oracles::list() -> Json<OraclesResponse>`: two `spawn_blocking` calls (or one joint one) — `let missions = missions::list(); let live_sessions = rmux::list_sessions().unwrap_or_default();` — then `missions.into_iter().map(|m| OracleEntry { session: format!("oracle-{}", m.key), live: live_sessions.contains(&format!("oracle-{}", m.key)), key: m.key.clone(), mission: Some(m) })`. **Note the naming risk explicitly:** confirm during Step 1 whether `Mission.key` already IS the bare key (e.g. `"Verba"`) or the full session name (e.g. `"oracle-Verba-1"`) by reading `missions.rs`'s `is_top_level_oracle_ledger`/parsing code again — the doc comment at the top of `missions.rs` shows a ledger JSON with `"oracle":"oracle-dentistrygpt"` as a full field, separate from whatever `key` is derived from the filename. Get this right from the real code, not from the sketch above; the sketch's `format!("oracle-{}", ...)` may be wrong and must be corrected against the actual `Mission`/`LedgerFile` field semantics before implementing.

- [ ] **Step 1: Read `missions.rs` in full** (past line 70, not shown in the earlier excerpt) to nail down exactly how `key` is derived from the ledger filename and whether it already includes the `oracle-` prefix, so `OracleEntry.session` is constructed correctly rather than guessed.
- [ ] **Step 2: Failing test:** using the `OMEGA_RMUX_BIN` fake-bin pattern from `tests/sessions_test.rs` (a fake `rmux ls -F #S` printing a known session name) PLUS a tempdir with one hand-written `oracle-<key>.progress.json` ledger file (same shape as `missions_test.rs` already uses — read that existing test file first for the exact fixture format), GET `/v1/oracles`, assert one entry with the right `key`, `live: true` (matching the fake session), `mission.done`/`mission.total` populated from the fixture. A second ledger with NO matching live session asserts `live: false`.

Run → FAIL.

- [ ] **Step 3:** Implement + wire, above the guard.
- [ ] **Step 4:** Extend `schema_test.rs`.
- [ ] **Step 5:** Tests green, clippy clean.
- [ ] **Step 6: Runtime verify:** with the daemon running and at least one real oracle ledger on disk (or launch a harmless test one if none exists — do NOT dispatch a new one just to test this endpoint, reuse whatever's already in `~/.omega/state/`), `curl … /v1/oracles | jq` — paste real output.
- [ ] **Step 7: Commit** `feat(gateway): GET /v1/oracles — live session + progress ledger composed`.

---

### Task 8: `POST /v1/dispatch` — launch an oracle (the one mutating endpoint)

**Files:**
- Create: `crates/omega-gateway/src/routes_dispatch.rs`
- Modify: `lib.rs`, `protocol.rs` (add `DispatchRequest`, `DispatchResponse`), `server.rs`, `tests/schema_test.rs`
- Create: `crates/omega-gateway/tests/dispatch_test.rs`

**Interfaces:**
- `protocol::DispatchRequest { project: String, mission: String, agent: Option<String>, new: Option<bool> }` — mirrors `omega dispatch <PROJECT> <MISSION> [--agent ...] [--new]` 1:1; `mission` has no length cap here (R-GOAL's 4000-char limit is a `/goal` primitive concern, not this endpoint's — but DO reject an empty/whitespace-only `mission` and an empty `project` as 400s before any validation-against-the-project-list step, since those are free and catch the common client bug of an empty form field).
- `protocol::DispatchResponse { oracle: String, delivery: String }` — `oracle` is the session name parsed off `omega dispatch`'s first stdout line (`"Oracle dispatched: <name>"` — strip the fixed prefix; if the line doesn't match that exact shape, treat the whole call as failed rather than guessing), `delivery` is the value after `DISPATCH_DELIVERY=` on the last non-empty stdout line. **Read `DispatchOutcome::report_lines()` in `dispatch.rs` in full before implementing the parser** — Task 8's own investigation must confirm the exact line contract (the plan's earlier grep only confirmed line 0 and the last line's tag name, not the full enum of `delivery` values or whether a followup-vs-new-oracle dispatch changes the line count) rather than the parser being written against a guess.
- `routes_dispatch::create(Json(req): Json<DispatchRequest>) -> Result<Json<DispatchResponse>, (StatusCode, Json<serde_json::Value>)>`:
  1. Reject empty `project`/`mission` → 400.
  2. `spawn_blocking(|| omega_core::projects::discover(&home))`, reject if `req.project` doesn't match any discovered project's `name` → 400 with the rejected value named in the error body (never spawn a subprocess for an unknown project — this is the "validate before spawn" contract from Global Constraints).
  3. Build argv: `["dispatch", &req.project, &req.mission]` + `["--agent", agent]` if present + `["--new"]` if `req.new == Some(true)`. Pass through `omega_cli::run` (Task 6), in `spawn_blocking` (subprocess spawn + wait is blocking).
  4. Non-zero exit or unparseable stdout → 502 with `stderr`/raw `stdout` in the error body (never fabricate an `oracle` name on a failure).
  5. Success → parse and return `DispatchResponse`.

- [ ] **Step 1: Read `DispatchOutcome::report_lines()` and the surrounding `DispatchDelivery`/`DispatchRoute` enums in full** (dispatch.rs, the struct starting at line 179) to ground the exact parser contract — this is the step that turns the sketch above into a real implementation plan; do not skip it or implement against assumption.
- [ ] **Step 2: Failing test — happy path:** fake `omega` script (`OMEGA_BIN` override) that, when called with `dispatch <project> <mission>`, prints a fixture matching the REAL contract confirmed in Step 1 (e.g. `Oracle dispatched: oracle-TestProj-1` on line 0, `DISPATCH_DELIVERY=new_oracle` on the last line) and records its full argv to a capture file. Seed a fake discovered-project (see Step 3 below for how) named `TestProj`. POST `/v1/dispatch {"project":"TestProj","mission":"do the thing"}`, assert `200`, `oracle == "oracle-TestProj-1"`, `delivery == "new_oracle"`, AND read the capture file to assert the recorded argv is EXACTLY `["dispatch", "TestProj", "do the thing"]` (proving no shell-string interpolation happened and no extra/missing args).
- [ ] **Step 3: Figure out how to make `projects::discover` see the fake project inside a test** — it walks the REAL `$HOME` (`dirs::home_dir()`), which a hermetic test cannot control by pointing at a tempdir the way `OMEGA_GATEWAY_DIR`/`OMEGA_RMUX_BIN` do (`projects::discover(home: &Path)` takes an explicit `home` argument in its signature, but `routes_projects.rs`/this route currently call `dirs::home_dir()` directly). Resolve this for real rather than skipping the validation test: either (a) thread a `HOME`-like override through `AppState`/config the same way `gateway_dir()` respects `OMEGA_GATEWAY_DIR`, calling `projects::discover(&effective_home)` everywhere `dirs::home_dir()` is currently hard-coded (Tasks 5 and 8 both then benefit — revisit Task 5 to use the same override for consistency), or (b) if that's too invasive for this plan's scope, mark this specific sub-case as a `#[ignore]`d test with a comment naming the gap and cover ONLY the "unknown project → 400, no subprocess spawned" path with a project name that provably won't exist on any real `$HOME` (e.g. a random UUID string) — record which option was taken and why in the commit message; do not silently drop the happy-path test coverage without saying so.

Run (both tests) → FAIL.

- [ ] **Step 4: Failing test — unknown project:** POST `/v1/dispatch {"project":"definitely-not-a-real-project-<uuid>","mission":"x"}` with NO fake `omega` script installed (or one that would fail loudly if invoked) — assert `400` and assert the fake binary's capture file was NEVER created (proving the subprocess was never spawned).
- [ ] **Step 5: Failing test — subprocess failure:** fake `omega` script that exits 1 with stderr text — assert `502` and the stderr text surfaces in the response body.
- [ ] **Step 6:** Implement `routes_dispatch.rs` + the argv/parsing logic against the Step-1 ground truth; wire `.route("/v1/dispatch", axum::routing::post(crate::routes_dispatch::create))` ABOVE the guard (this is a real side-effecting action — it MUST require device auth, same as every other protected route; do not accidentally reuse the unauthenticated pre-guard block `/v1/health`/`/v1/pair` live in).
- [ ] **Step 7:** Extend `schema_test.rs` with `DispatchRequest`, `DispatchResponse`.
- [ ] **Step 8:** `cargo test -p omega-gateway` green, clippy clean.
- [ ] **Step 9: Runtime verify, WITHOUT launching a real oracle (per the brief's explicit caution):** curl the endpoint with a deliberately-unknown project name and confirm a real `400` from the live daemon (`curl -X POST … -d '{"project":"nope-xyz","mission":"x"}' -w '%{http_code}'`) — this proves the validation gate is live without spending a real dispatch. Do NOT dispatch a real mission against this endpoint as "verification" — that violates the brief's own guardrail and R-DESTRUCT's spirit for a mutating action exercised outside an explicit operator ask.
- [ ] **Step 10: Commit** `feat(gateway): POST /v1/dispatch — launch an oracle via the omega CLI, argv-only`.

---

### Task 9: Wire everything into `server.rs` and do a final live-daemon pass

**Files:**
- Modify: `crates/omega-gateway/src/server.rs` (confirm all 6 new routes are present and above the guard — this task is a review/consolidation pass, not new routes; Tasks 2–8 each already wire their own route, so if every prior task did its job this task is verification-only)

- [ ] **Step 1:** Re-read the final `server.rs` top to bottom; confirm `/v1/rules`, `/v1/agents`, `/v1/skills`, `/v1/projects`, `/v1/oracles`, `/v1/dispatch` are ALL registered before `.route_layer(...)`, and that NONE of them accidentally landed in the pre-guard `Router::new()` block at the bottom of `build_router`.
- [ ] **Step 2:** `cargo test -p omega-gateway` full suite green (should now be ~110 + this plan's new tests), `cargo clippy -p omega-gateway --all-targets -- -D warnings` clean.
- [ ] **Step 3: Full runtime pass (L1) against the live daemon:** re-run every `curl` from Tasks 2, 3, 4, 5, 7 in one sitting against the actually-running `omega-gatewayd`, plus the Task 8 negative-case curl, and paste the consolidated real output into the final commit message or a short note in this plan file's own git history — this is the evidence that the whole surface works together, not just each route in isolation.
- [ ] **Step 4: Commit** `chore(gateway): omega-surface — verified end to end against the live daemon` (only if Step 1 found something to fix; otherwise fold this verification into Task 8's commit and skip an empty commit).

---

## Explicit known limits / deferred (name them, don't silently drop them)

- **AISB persona agents** (`agents/*.md`) are out of scope for `GET /v1/agents` (see Architecture above) — a future plan item if the App wants an oracle-persona picker.
- **`~/.claude/skills`'s nested library packs** (the ~130-skill design-intelligence pack, etc.) are NOT flattened by `GET /v1/skills` — it mirrors `~/.omega/skills` (341 skills, matching `omega skills validate/compile`'s own SSOT), not the raw 436-entry `~/.claude/skills` directory.
- **`projects::discover` reads the REAL `$HOME`** — Task 8's Step 3 may leave the dispatch happy-path test `#[ignore]`d if a clean `HOME` override isn't threaded through in this plan's scope; if so, that's a named gap for a follow-up plan, not a silently-skipped test.
- **No WebSocket/live-update surface for rules/agents/skills/projects** — these are request/response GETs, matching the brief; `/v1/oracles` liveness is a snapshot per request, not pushed over `/v1/events` (a future `GatewayEvent::OracleUpdated` variant would need its own plan).
