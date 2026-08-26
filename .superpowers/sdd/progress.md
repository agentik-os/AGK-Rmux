# omega-gateway wave8 — final endpoint wave (A-D)

Merge-base: origin/main @ d256af8 (wave7 merged, 435 tests).
Branch: omega-gateway-wave8.

Tasks A-D are SERIALIZED, not parallel fan-out: every task touches
protocol.rs, server.rs, lib.rs, schema_test.rs (R-SCOPE — one writer per
file). Each still gets a FRESH implementer + FRESH reviewer per the SDD
contract, one task at a time, its own commit(s).

## Ground truth gathered before implementing (controller read, not guessed)

- **Task A / `omega new` — NAME is a REQUIRED positional in the real CLI**,
  but the brief's request shape has `name?` optional. Resolution: when the
  caller omits `name`, the gateway generates one server-side
  (`gw-{util::random_hex(6)}`) rather than making the field truly optional in
  the CLI sense — never invents an "auto" sentinel the CLI itself doesn't
  support. `cmd_new` (main.rs:3132) supports `--dir`, `--cmd` (OVERRIDES
  `--agent` — never expose `--cmd` over the API, arbitrary shell exec), `-a
  agent`, `-p prompt`, `--files`. This endpoint exposes `agent` (validated
  against `omega_core::agents::Agent::all()`, same posture
  `routes_dispatch.rs` uses), `dir` (validated under the SAME allowed root as
  every other filesystem-touching endpoint in this crate: canonicalizes and
  prefix-checks against `dirs::home_dir()` — no client-supplied path ever
  reaches a subprocess unchecked), `prompt` (length-capped, mirrors
  `MAX_MISSION_LEN`). Never exposes `--cmd` or `--files` (scope-claim has its
  own semantics this wave doesn't touch). `cmd_new` prints `"Created session:
  {name}"` on success — but the response doesn't need to parse it: the name
  was already chosen/validated server-side before the spawn, so it's echoed
  back directly (same "we already know it" posture
  `routes_sessions::rename` uses for `new_name`).
- **Task A / `omega team` — the brief's `{n, agents?, layout?}` shape does
  NOT match the real CLI.** `Commands::Team` (main.rs ~1085) is `<PROJECT>
  [MEMBERS]... -c/--count -d/--dir` — no `--layout` flag exists anywhere in
  `cmd_team` (main.rs:7520) or its clap definition; `layout` is dropped as a
  documented gap, matching wave7's `--agent`-not-forwarded precedent for
  `orchestrate`, not silently invented. `project` becomes literally
  `format!("Team-{project}")` as the spawned session's name (cmd_team:7531) —
  so it's validated with the SAME strict slug charset
  `routes_sessions::valid_new_session_name` already enforces for exactly this
  reason (a session-name COMPONENT), not the looser `valid_session_name`.
  `members` are optional `"name:prompt"` strings passed positionally
  (`--count` alone spawns N generic `worker-N` members when `members` is
  empty, confirmed at cmd_team:7553); each member string is only
  length-capped + NUL-checked here (the CLI itself parses `splitn(2, ':')`
  leniently, no name-charset validation exists in the real command either).
  `count` bounded `1..=8` (CLI default 3, no hard cap in the CLI itself —
  this endpoint adds one, matching `MAX_MISSION_LEN`-style defense-in-depth
  for an unbounded numeric field). `dir` reuses Task A's own allowed-root
  check.
- **Task B / `new-project` — `cmd_new_project` (main.rs:3191) is FAST, NOT
  long-running**, contradicting the brief's own premise. Non-dry-run,  it
  does exactly ONE thing beyond printing: `mgr.create_session_with_agent(...)
  .await?` — spawns a Codex rmux session with the `/omega-new-project ...`
  prompt and returns in well under a second, identical in shape to
  `cmd_new`/`cmd_dispatch`. The WHOLE vision→prd→brand→planner→build pipeline
  then runs ASYNCHRONOUSLY *inside* that spawned session — invisible to the
  CLI process, and out of scope for this wave to watch (that would mean
  streaming an arbitrary rmux pane, R-STREAM's job, not this endpoint's).
  Decision (documented, not silently deviated): build EXACTLY what the brief
  asked — `GET /v1/new-project/stream?name=&category=&group=` — because
  it's still the correct, useful shape (mirrors `install_stream`'s
  pre-upgrade-validation + Line/Exit-frame pattern 1:1, surfaces a real spawn
  failure over the socket, and an Exit frame confirms the session was
  created) even though in practice it will emit ~3 short lines then Exit
  almost immediately — same "browsers can't upgrade a POST" reasoning wave7
  already used to collapse `POST /v1/orchestrate` into a GET WS stream, so no
  separate synchronous `POST /v1/new-project` ships alongside it (one
  endpoint, not two, avoiding a redundant second code path for the same
  subprocess). `name` validated against the CLI's own documented charset
  (`^[a-z0-9-]+$`, non-empty, capped at 64). `category` validated against the
  literal closed set from `--help`: `works | client | 1-life | AgentikOS`.
  `stack` is NEVER exposed as a param — always hardcoded to `"nextstack"`
  ("the only stack today" per the CLI's own help text) and passed explicitly
  as the 2nd positional so `category` lands correctly as the 3rd. `--build`
  is NEVER passed (opt-in only on the real CLI; this endpoint stays at the
  planning-only default). NEVER run for real in tests — every
  `tests/new_project_test.rs` case points `OMEGA_BIN` at a fake script.
- **Task C / `omega marketing list` — pure in-process, no CLI subprocess at
  all.** `omega_core::marketing::list_marketing_projects()`
  (`crates/omega-core/src/marketing.rs:20`) is a synchronous,
  side-effect-free filesystem scan (same shape `routes_rules::list` /
  Task B-of-wave7's timeline/gate reads already use) — calling it in-process
  via `spawn_blocking` is simpler and more robust than shelling to `omega
  marketing list --json` and parsing rendered/JSON text through a second
  process. Verified live: real JSON shape confirmed
  (`name, slug, path, has_content, calendar_posts, engine_on, accounts:
  Option<usize>, accounts_tried, has_context, has_strategy, has_copy,
  has_visual, has_branding`). `accounts` is a **count**, not a list (`Some(n)
  => "{n} accounts"` at cmd_marketing:3637) — mirrored as `Option<usize>` in
  the new `MarketingProjectEntry`, NEVER populated by this endpoint (that
  needs `project_accounts()`, which shells to `omega-zernio` — out of scope,
  "Read-only" per the brief; `accounts`/`accounts_tried` always reflect the
  list-only values, i.e. `accounts: None`). `path` (`PathBuf`) is exposed as
  a plain string, same transparency posture `ProjectEntry.path` already
  carries for `/v1/projects`.
- **Task D / DUO — investigated the real bridge (`~/.local/bin/omega-duo`,
  the Bun binary the `/duo` skill drives) before designing anything.**
  `"duo"` is **NOT** a member of `omega_core::agents::Agent::all()` (roster:
  claude, codex, gemini, pi, hermes, glm, kimi, shell) — so option (a) from
  the brief (`POST /v1/sessions` with `agent="duo"`) is a non-starter, not a
  style choice: there is no such agent to spawn a persistent rmux session
  for. The REAL mechanism is a clean, well-documented, ALREADY deterministic
  one-shot CLI contract: `omega-duo run --task <file.md> --cwd <dir> --mode
  <plan|code|review> [--agent codex|claude|glm] [--verify "<cmd>"]` reads the
  task file's content itself (`readFileSync(taskFile, ...)`, confirmed at
  line 1221 of the binary — `--task` is a real filesystem PATH, not a stdin
  hand-off, contra a loose reading of the skill doc's stdin-transmission
  line, which describes what THIS binary does with the file's content
  internally, not how the CALLER supplies it) and prints EXACTLY ONE JSON
  line to stdout on completion (`console.log(JSON.stringify(result))`,
  binary line 287) — no progressive stdout to stream, so a WS Line/Exit
  pattern would carry zero real frames until the very end anyway. Decision:
  ship the brief's option (b), the DEDICATED endpoint, using the bridge
  DIRECTLY — this is the "thinnest honest version" already, not a fallback
  from it; the bridge is not stateful or hard to wrap, it's a bounded
  subprocess with a typed JSON contract. `POST /v1/duo {project?, dir?,
  prompt, profile}` where `profile ∈ {build, review, reflect}` maps
  1:1 to bridge `--mode` (`build→code`, `review→review`, `reflect→plan`) —
  the skill's own three profiles, not invented. Exactly one of
  `project`/`dir` required (ambiguous-or-missing target is a 400): `project`
  resolves through the SAME discovered-project allowlist
  `routes_dispatch.rs`/`routes_orchestrate.rs` already use; `dir` reuses Task
  A's allowed-root check. `prompt` is written to a SERVER-CHOSEN scratch file
  under `~/.omega/state/gateway-duo/tasks/<random_hex>.md` (mirrors
  `routes_pdf.rs`'s `data`-scratch pattern — NEVER a client-supplied path
  reaching `--task`). `--agent` is NEVER forwarded (no explicit agent
  override exposed this wave — GLM is opt-in-only doctrine and this is a
  programmatic caller, not an operator explicitly asking for GLM; the bridge
  defaults to codex-first/claude-fallback on its own). `--verify` is NEVER
  passed (no safe, generic success command exists for an arbitrary caller-
  supplied prompt). Response mirrors the bridge's real `BridgeResult`
  JSON contract field-for-field (`agent, ok, agent_ok, output, fell_back,
  reason, exit_code, sandbox_degraded, capabilities{shell_exec,
  worktree_read}, guard_error{code,message}, verify{cmd,exit_code,ok,
  timed_out,tail}, checkpoint{head,stash,ref}, diffstat, log`) — a new
  `duo_bin()` resolver in `omega_cli.rs`-sibling shape (env override
  `OMEGA_DUO_BIN`, else `~/.local/bin/omega-duo`, confirmed the real
  installed path via `which omega-duo`; the skill doc's fuller 3-tier
  fallback is NOT replicated — two tiers matches this crate's existing
  `omega_bin()`/`rmux_bin()` simplicity convention and the box's real
  install). Run via `tokio::process::Command` (async, killable,
  `kill_on_drop(true)`, own process group) wrapped in a bounded
  `tokio::time::timeout` (`DUO_TIMEOUT_SECS`, default 1800s — a real Codex
  "code" run can genuinely take minutes, unlike PDF's 300s npm-install
  bound; overridable via env for fast tests) — NOT `omega_cli::run`'s
  blocking `Command::output()`, which cannot be cancelled once spawned (same
  deviation wave7's PDF adversarial-review round already established for
  exactly this reason). Concurrency: a NEW `AppState::duo_permits`
  (`MAX_CONCURRENT_DUO_RUNS = 2`, mirrors `pdf_permits`) PLUS an in-process
  per-resolved-cwd lock (`AppState::duo_active_dirs: Arc<Mutex<HashSet<
  PathBuf>>>`) — the skill's own doc is explicit that two `omega-duo run`s on
  the SAME cwd corrupt each other's checkpoint guard ("jamais deux runs sur
  le meme worktree"), so this endpoint refuses a second concurrent run
  against a cwd already in flight (409, not a queue) rather than silently
  letting two duo runs race on one repo. NEVER run a real Codex/Claude/GLM
  turn in tests (fake `OMEGA_DUO_BIN` only) and NEVER in live-verify either —
  there is no cheap "scratch duo session to spawn and kill" the way A/B/C
  have (this isn't an rmux session at all, it's a bounded subprocess call
  that necessarily burns a real quota turn the instant it's invoked for
  real) — live-verify instead runs the FREE, real, no-quota `omega-duo
  doctor --json` directly (outside the gateway) to confirm the bridge itself
  is healthy on this box, and documents plainly that `POST /v1/duo`'s own
  live path is fake-only this wave.

## Task A — status: DONE (routes_sessions.rs::create + routes_team.rs::create,
tests/sessions_create_test.rs + tests/team_test.rs). Commit 7e9601b (435→469
tests), then a fresh adversarial reviewer found 4 Important findings (no
Critical): a `dir_under_home` traversal bypass via `..` behind a
not-yet-existing path component, no concurrency permit on either new heavy
endpoint, an unbounded `members` vector on `/v1/team` (200k members accepted
in one request), and the echoed `name`/`session` could diverge from the REAL
rmux session name (`sanitize_session_name` truncates/trims differently than
this crate's `valid_new_session_name` charset check). All 4 fixed TDD, plus
a Minor (`--prompt`/`--dir` values starting with `-` now emitted as single
`--flag=value` argv tokens) — commit 9c10ea0.
**Test count after Task A: 480, 0 failed. Clippy clean.**
## Task B — status: DONE (routes_new_project.rs::stream,
tests/new_project_test.rs). Ground truth re-confirmed live via `omega
new-project --help` before implementing: matches progress.md exactly
(NAME charset `^[a-z0-9-]+$`, STACK positional 2 hardcoded `nextstack`,
CATEGORY positional 3 default `works`, closed set `works | client | 1-life
| AgentikOS`, `--group <GROUP>` real flag defaulting to `"default"` in the
CLI itself when omitted here). New `NewProjectStreamMsg` (protocol.rs) +
`AppState::new_project_permits` (`MAX_CONCURRENT_NEW_PROJECT_SPAWNS = 2`,
mirrors `orchestrate_permits`). Route wired above `route_layer`.
**Test count after Task B: 501, 0 failed (480→501, 21 new: 9 integration +
12 inline unit). Clippy clean. Release build OK.**

Commit `b9cb7e5`, then a fresh adversarial reviewer found 4 Important
findings (no Critical — the `--` separator was proven correct, not a repeat
of the wave7 bug class): `is_slug` allowed a leading `-` that leaked past
clap into the downstream agent PROMPT string (`cmd_new_project`'s `flags`
field), a `process_group(0)` doc comment copy-pasted from
orchestrate/install claiming disconnect-kill safety that is FALSE for this
subcommand (rmux daemon setsid()s its own children — no real cancellation
exists), a concurrency-cap doc comment overclaiming what it bounds
(in-flight spawn requests, not live bootstrap pipelines), and a
session-name truncation/collision risk (`{name}-setup` not validated
against `sanitize_session_name`, same defect class Task A's own review
round already found in `routes_team.rs`). All 4 fixed — commit `ea014f7`.
**Test count after Task B fix round: 510, 0 failed. Clippy clean.**

## Task C — status: DONE (routes_marketing.rs::list, tests/marketing_test.rs).
`GET /v1/marketing` calls `omega_core::marketing::list_marketing_projects()`
in-process via `spawn_blocking` (no subprocess), mapping every
`MarketingProject` field into a new `MarketingProjectEntry` EXCEPT `path`.
Ground-truth correction found while implementing: the brief's premise that
"`ProjectEntry.path` already carries" a plain-string path-exposure
precedent is factually wrong — `protocol.rs`'s real `ProjectEntry` (commit
`370ec43`) deliberately DROPS `path`, with an explicit doc comment ("a full
filesystem path is server-internal, not something the wire protocol should
leak to a mobile client"). `MarketingProjectEntry` follows that REAL,
deliberate, documented convention instead (path omitted; `slug` already IS
the id `omega-zernio`/higgsfield use, so nothing actionable is lost).
`accounts: Option<usize>` (matches the existing `Option<usize>` precedent
at `TelegramStatusResponse.allow_user_ids_count`), always `None`/`null`
from this endpoint (never shells to `omega-zernio` — out of scope, listing
stays read-only). Also found: `ProjectRegistry::load()` (one of
`list_marketing_projects`'s two sources) hardcodes `dirs::home_dir()`
directly and is NOT configurable via `OMEGA_STATION_DIR` — tests override
both `$HOME` (neutralizes the real `~/.omega/projects.json`, which on this
box has 11 real marketing-enabled projects) and `OMEGA_STATION_DIR` (scopes
the filesystem-scan half), same dual-env pattern `telegram_test.rs` uses
for its own `$HOME`-hardcoded lookup. Route wired above `route_layer`
(`/v1/marketing`, between `/v1/projects` and `/v1/files`).
**Test count after Task C: 514, 0 failed (510→514, 4 new integration
tests: empty station, full status flags + accounts never populated,
multi-project name-sort, 401). Clippy clean. Release build OK.**

Commit `51d95c4`, then a fresh adversarial reviewer verdict "SHIP WITH
MINOR FIXES" (no Critical): the module doc comment falsely claimed no
subprocess, but `list_marketing_projects()` actually forks an UNBOUNDED
`crontab -l` per call (unlike `project_accounts()`'s 4s-bounded call — same
"copy-pasted claim, not re-verified" class as Task B's Finding 2); a
non-unique `slug` (derived from directory basename, while dedup happens on
lowercased NAME) let two entries for the SAME on-disk project appear with
an identical `slug` and no `path` field left to disambiguate; and the
shipped tests read the operator's REAL crontab for `engine_on`, making them
environment-dependent/flaky. Fixed: doc comment corrected + the whole
`spawn_blocking` call wrapped in a `tokio::time::timeout` (10s default,
`OMEGA_MARKETING_LIST_TIMEOUT_MS` override, mirrors `routes_pdf.rs`'s
idiom) → 504 on timeout; gateway-side dedupe by canonicalized path (never
touching the shared `omega-core` function); tests now fake `crontab` on
`PATH` for determinism. Commit `6af3240`.
**Test count after Task C fix round: 516, 0 failed. Clippy clean.**

## Task D — status: DONE

`POST /v1/duo` (new `routes_duo.rs`), wrapping the real `omega-duo` bridge
directly (option (b) from the wave brief — `"duo"` is not a member of
`omega_core::agents::Agent::all()`, so option (a) was never viable).
`{project?, dir?, prompt, profile}`, exactly one of `project`/`dir`
required, `profile ∈ {build,review,reflect}` → bridge `--mode
{code,review,plan}`. `project` resolved via the discovered-project
allowlist; `dir` reuses `routes_sessions::dir_under_home` verbatim
(including its `..`-component guard from Task A's own fix round). `prompt`
written to a server-chosen scratch file under
`~/.omega/state/gateway-duo/tasks/<random_hex>.md`. Response mirrors the
real `BridgeResult` JSON field-for-field into new gateway-local protocol
types. Two ground-truth corrections found while implementing (both cited
with binary line numbers in `routes_duo.rs`'s doc comment): `--flag=value`
does NOT work against `omega-duo`'s own `parseArgs` (it only understands
two-token `--flag value`, confirmed by reading the binary source — using
the `=` form would silently corrupt `--cwd`), and the bridge's own process
exit code carries no information beyond its JSON body's `ok` field
(`process.exitCode = result.ok ? 0 : ...`), so HTTP status is driven by
"did stdout parse as the expected JSON" (502 if not), never by the
subprocess exit code. Concurrency: a flat `duo_permits` semaphore
(`MAX_CONCURRENT_DUO_RUNS=2`) PLUS a per-resolved-cwd lock
(`duo_active_dirs`, RAII-released) enforcing the skill's own documented
"never two duo runs on the same cwd" constraint (409 on collision). Never
passes `--agent`/`--verify`. Timeout 1800s (env-overridable), real
process-group kill on fire (verified: `omega-duo` spawns Codex/Claude
without `detached:true`, so unlike `new-project` a group kill here
genuinely reaches the nested agent turn). Never runs a real Codex/Claude/
GLM in tests (fake `OMEGA_DUO_BIN` only). Commit `a7b01e6`.
**Test count after Task D: 539, 0 failed (516→539, 23 new). Clippy clean.
Release build OK.**

Commit `a7b01e6`, then a fresh adversarial reviewer verdict "BLOCKED ON 2
IMPORTANT FINDINGS" (no Critical, and BOTH of the implementer's own
ground-truth corrections were independently re-verified true): the per-cwd
lock keyed on the RAW resolved path while `omega-duo` itself operates on
the REPO ROOT, so two textually-different paths into the same git repo
(a project root and a subdirectory, or a symlinked spelling) bypassed the
lock and raced two runs on one checkpoint guard; and a client disconnect
(vs an explicit timeout) only SIGKILLed the direct `omega-duo` child, never
the nested agent's process group, orphaning a live, unbounded, file-editing
AI agent while `CwdLockGuard::drop` released the lock anyway, letting a
second request start a genuinely concurrent run against the same worktree.
Plus 2 Minor (a `--`-prefixed `dir` reaching the bridge's argv unguarded;
a whole-buffer JSON parse throwing away a successful run on one stray
stdout line, when the bridge's own test harness only trusts the LAST
line). Fixed: the lock now keys on the canonicalized repo root (walks up
for the nearest `.git`, `--cwd` itself deliberately left as the raw
resolved path — the bridge resolves its own root internally); a new
`KillGroupOnDrop` RAII guard fires an unconditional group-kill on ANY drop
of the run future (disconnect, timeout, or an early return), disarmed only
on confirmed normal completion; an absolute-path guard rejects a
non-absolute resolved target before any spawn; the JSON parse now takes
the last non-empty stdout line. Commit `1a5ef34`.
**Test count after Task D fix round: 543, 0 failed. Clippy clean.**

## Wave summary — all 4 tasks (A-D) done, each with an implementer +
independent adversarial reviewer + TDD fix round. Total: 435 → 543 tests
(+108). Next: rebase on origin/main (done), a final opus whole-branch
review that RUNS THE BINARY against live HTTP, controller runtime verify
of the safe endpoints, report.
