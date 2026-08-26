# Omega Gateway — Multi-Account Implementation Plan (Plan 3a)

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]` checkboxes.

**Goal:** Let the app manage MULTIPLE Claude / Codex accounts on the box (add, list, remove, set default, check status) and run each chat under a chosen account, so the operator is not locked to one account. This is the product differentiator.

**Architecture:** Each account is an isolated credential directory. Claude: a `CLAUDE_CONFIG_DIR` per account (proven — an empty dir yields `Not logged in`). Codex: a `CODEX_HOME` per account. Accounts live under `~/.omega/gateway/accounts/<slug>/` (0700). Adding an account runs the real CLI login (`claude auth login` / `codex login [--with-api-key]`) with the env pointed at that slot; the gateway relays the login URL (Claude) or accepts an API key (Codex). The chat driver already accepts an `account_dir` — this plan builds the management + selection around it and wires the selected account into every turn.

**VERIFIED FACTS (probed on the box 2026-08-10, cost $0):**
- `CLAUDE_CONFIG_DIR=<empty dir> claude -p x --output-format json` → `"result":"Not logged in · Please run /login"`, `total_cost_usd:0`. So a fresh config dir is a clean, isolated account slot. It populates `<dir>/.claude.json` + `projects/` + `sessions/`.
- Claude auth CLI: `claude auth login` / `claude auth logout` / `claude auth status`. Also `claude setup-token` (long-lived token, requires subscription).
- Codex: `CODEX_HOME` env (default `~/.codex`); `codex login` (browser), `codex login --with-api-key` (reads key from stdin — fully headless), `codex login status`.
- The default box account today is a single symlinked file `~/.claude/.credentials.json → ~/.omega/credentials/claude.json` — DO NOT touch it; the default account = "use ambient config" (account_dir None), the existing V2 behavior. Named accounts are additive slots.

## Global Constraints
- Repo `~/Station/SideBusiness/OmegaOS`, work in the provided worktree on branch `omega-gateway-accounts`. Build on the merged V2 crate (87 tests). Sync before merge; commit only your own files.
- Protected routes mounted ABOVE the `route_layer` guard comment in `server.rs` (auth Bearer or `?token=`). Reuse `fsperm::harden_dir/harden_file` (0700/0600), `util::random_hex`, the `static LOCK: tokio::sync::Mutex<()>` env-guard test pattern.
- Account slug: `[a-z0-9-]{1,32}`, validated before any fs join (path-traversal guard, same discipline as `valid_chat_id`).
- SECURITY (R-ENV): account dirs hold real credentials. Dir 0700, never list credential file CONTENTS over the API, never log tokens. The API exposes only metadata (slug, label, agent, auth status, created_at). `accounts/` is under `~/.omega` (gitignored) — never in a repo.
- NEVER invoke a real paid `claude -p` turn in tests. Login/status probing that returns "not logged in" is free; use fake bins (`OMEGA_CHAT_BIN`, and add `OMEGA_CLAUDE_BIN`/`OMEGA_CODEX_BIN` overrides for the auth commands) so tests never hit the real CLIs.
- All code/comments/commits English. After each task: `cargo test -p omega-gateway` green, `cargo clippy -p omega-gateway --all-targets -- -D warnings` clean.

---

### Task 1: Account store (metadata + slots on disk)

**Files:** Create `crates/omega-gateway/src/accounts.rs`; modify `lib.rs`, `protocol.rs` (+ schema_test).

**Interfaces:**
- `protocol::AccountKind` enum `{ Claude, Codex }` (serde rename_all lowercase).
- `protocol::Account { slug: String, label: String, kind: AccountKind, created_at: String, is_default: bool }` (Serialize+JsonSchema) — metadata ONLY, never credentials.
- `accounts::AccountStore::open(gateway_dir: &Path) -> AccountStore` (reads `<dir>/accounts/accounts.json` = a registry of metadata; each slot's dir is `<dir>/accounts/<slug>/`).
- `create_slot(&self, slug, label, kind) -> Result<Account>` (validates slug, mkdirs `<dir>/accounts/<slug>/` hardened 0700, writes registry, first account of a kind becomes default), `list() -> Vec<Account>`, `get(slug) -> Option<Account>`, `remove(slug) -> bool` (removes registry entry + the slot dir), `set_default(slug) -> bool` (clears other defaults of the same kind), `slot_dir(slug) -> PathBuf`, `default_for(kind) -> Option<Account>`.
- `accounts::valid_slug(s: &str) -> bool`.

- [ ] Step 1: Failing tests (tempdir): create_slot roundtrip + dir 0700 + registry 0600; slug validation rejects `../x`, uppercase, empty, >32; first-of-kind is default, set_default moves it, default cleared on the loser; remove deletes dir + entry; persistence across reopen; schema_test asserts `Account`/`AccountKind`.
- [ ] Step 2: implement protocol additions + umbrella + schema_test.
- [ ] Step 3: implement `accounts.rs`.
- [ ] Step 4: tests green, clippy clean.
- [ ] Step 5: commit `feat(gateway): account store — isolated Claude/Codex credential slots`.

---

### Task 2: Account login driver (add an account for real)

**Files:** Create `crates/omega-gateway/src/account_login.rs`; modify `lib.rs`.

**Interfaces:**
- `account_login::claude_bin()` (env `OMEGA_CLAUDE_BIN` else `claude`), `codex_bin()` (env `OMEGA_CODEX_BIN` else `codex`).
- `account_login::status(account: &Account, slot: &Path) -> AuthStatus` — for Claude runs `CLAUDE_CONFIG_DIR=<slot> claude auth status`; for Codex `CODEX_HOME=<slot> codex login status`; parse to `enum AuthStatus { LoggedIn, LoggedOut, Unknown }`. (Free — no paid call.)
- Claude login (browser relay): `begin_claude_login(slot: &Path) -> LoginSession` — spawns `CLAUDE_CONFIG_DIR=<slot> claude auth login`, captures stdout, extracts the authorization URL (regex for an https URL), returns `{ url: String, child_handle }`; the caller streams the URL to the app and the flow completes when the user authorizes in a browser and the child exits. Provide `finish/poll` that checks `status()` for `LoggedIn`. If no TTY / no URL is emitted (login needs an interactive terminal), return `LoginNeedsBox` so the route can tell the app to fall back to the box-side flow (the hybrid contract).
- Codex login: `codex_login_with_api_key(slot: &Path, api_key: &str) -> Result<()>` — pipes the key to `CODEX_HOME=<slot> codex login --with-api-key` via stdin (fully headless). Also `begin_codex_login(slot)` (browser) mirroring Claude for the OAuth path.
- `logout(account, slot)` → `claude auth logout` / `codex logout` in the slot.

- [ ] Step 1: Failing tests with FAKE bins (OMEGA_CLAUDE_BIN/OMEGA_CODEX_BIN pointing at bash scripts): `status` parses a fake "logged in"/"not logged in" output to the right enum; `begin_claude_login` extracts an https URL from a fake script that prints "Visit https://claude.ai/oauth?x=1 to continue"; `LoginNeedsBox` when the fake prints nothing URL-like and exits non-zero; `codex_login_with_api_key` feeds a fake that echoes stdin to a file the test reads, asserting the key was piped (NEVER a real key). LOCK-guard env mutation.
- [ ] Step 2: implement, keeping parsing pure where possible (`parse_login_url(s: &str) -> Option<String>`, `parse_auth_status(s: &str) -> AuthStatus` are pure + unit-tested).
- [ ] Step 3: tests green, clippy clean.
- [ ] Step 4: commit `feat(gateway): account login driver — claude/codex auth status + login relay`.

---

### Task 3: Account routes (CRUD + login) + wire selection into chat

**Files:** Create `crates/omega-gateway/src/routes_accounts.rs`; modify `server.rs` (AppState gains `AccountStore`, mount routes above route_layer), `routes_chat.rs` (resolve account per chat), `chat_store.rs`/`protocol.rs` (ChatMeta gains `account_slug: Option<String>`), `lib.rs`.

**Interfaces:**
- `GET /v1/accounts` → `{accounts:[Account...]}` (with live auth status merged in). `POST /v1/accounts` `{slug,label,kind}` → 201 Account (creates the slot). `DELETE /v1/accounts/{slug}` → 204. `POST /v1/accounts/{slug}/default` → 200. `GET /v1/accounts/{slug}/login` (WS or SSE): begins the login, streams `{type:"login_url",url}` then `{type:"login_done"}` / `{type:"login_needs_box"}` / `{type:"error"}`. `POST /v1/accounts/{slug}/apikey` `{api_key}` → 200 (Codex headless path; the key is piped to codex, NEVER stored by the gateway or logged).
- Chat selection: `POST /v1/chats` body gains optional `account_slug`; if set, persisted on ChatMeta and the chat's turns run with `account_dir = accounts.slot_dir(slug)`; if unset, `account_dir = accounts.default_for(kind).map(slot_dir)`, else None (ambient default = today's behavior). `run_turn` already takes `account_dir` — pass it through (currently it's hardcoded None in routes_chat; change to the resolved dir).

- [ ] Step 1: Failing integration tests: create a Claude account slot via POST, list shows it (status "logged_out" via a fake claude), set default, a new chat created with that account_slug persists it and — using a FAKE OMEGA_CHAT_BIN that writes its CLAUDE_CONFIG_DIR env to a file — assert a chat turn ran with the account's slot dir as CLAUDE_CONFIG_DIR (proves selection reaches the process). DELETE removes it. Slug traversal rejected. The Codex `/apikey` route pipes to a fake codex and returns 200 without persisting the key.
- [ ] Step 2: implement routes + AppState wiring + chat account resolution.
- [ ] Step 3: tests green, clippy clean.
- [ ] Step 4: commit `feat(gateway): account CRUD + login routes; per-chat account selection`.

---

### Task 4: CLI + real runtime verification (two isolated accounts)

**Files:** modify `main.rs` (add `accounts` subcommand: list; `account-add <slug> <label> <claude|codex>` creates a slot and prints the login instruction).

- [ ] Step 1: add the CLI subcommands (symmetry with `devices`/`chats`), focused test.
- [ ] Step 2: REAL runtime verification (capture into report): rebuild+reinstall+restart the daemon. Then, WITHOUT spending on paid turns beyond what's already proven: (a) create two Claude account slots via the API; (b) show `claude auth status` per slot is "not logged in" (free) proving isolation; (c) create a chat pinned to slot A and a chat pinned to slot B, and using a fake OMEGA_CHAT_BIN in a scratch check (or by inspecting the persisted ChatMeta.account_slug + the resolved slot dir) prove each chat targets its own CLAUDE_CONFIG_DIR. (d) OPTIONAL, operator-driven, NOT automated: real `claude auth login` into one slot is a browser flow the operator does — document the exact command. Do not attempt an interactive login headlessly; if you try `claude auth login` and it needs a TTY, capture that it returns the box-fallback signal.
- [ ] Step 3: full suite green, both clippy forms clean, `cargo build --release`, install parity (install.sh already ships the binary; `accounts/` is runtime-created, add nothing to the repo).
- [ ] Step 4: commit `feat(gateway): accounts CLI + multi-account runtime verification`.
- [ ] Step 5: after whole-branch review: fetch+rebase origin/main, ff-merge, push, post-merge health + `omega-gatewayd accounts`, remove worktree.

## Out of scope (later)
- The full in-app browser-relay polish (auto-detecting login completion, refresh) beyond the URL relay + status poll.
- Account-scoped session isolation for the terminal/session mirror (this plan scopes CHAT to accounts; sessions stay box-wide).
- Rotating/expiring account tokens, per-account usage/cost metering (a metering plan later).
