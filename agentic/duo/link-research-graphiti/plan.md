# OmegaOS Link Research and Graphiti Integration, Plan v2

This replaces Plan v1 after the mandatory Codex critique and Claude strategy
rewrite. The end goal remains complete, but delivery is phased and repository
mutation now requires explicit authenticated approval.

## Objective

An authorized URL shared to a dedicated Telegram research bot is captured,
studied automatically, challenged against primary evidence and Omega memory,
and classified as `adopt`, `adapt`, `defer`, or `reject`.

`adopt` and `adapt` stop at `awaiting_approval`. After explicit approval, a
Claude Oracle runs the exact `/duo` state machine, audits the result, preserves
install parity, commits, pushes, and records only verified conclusions in
Graphiti.

The workflow ships to every OmegaOS install, but the bot service is standby by
default. Exactly one explicitly activated machine owns the rotated Telegram
token and polls.

## Corrected premises

1. The Telegram token exposed in chat is revoked. Never repeat, use, log, test,
   commit, or distribute it. Activation accepts only a rotated secret through
   stdin or a mode-600 secret file.
2. One Telegram token means one poller. A 409 is conflict detection, not leader
   election. The poller enters persistent standby without exiting or creating a
   restart storm.
3. Telegram delivery, SQLite, rmux, Git, and Graphiti cannot share one atomic
   transaction. The system promises durable at-least-once processing with
   stable identities, receipts, reconciliation, and honest ambiguity, not
   impossible global exactly-once semantics.
4. Telegram update identity and research identity are separate:
   `(bot_id, update_id, url_index)` deduplicates delivery, while
   `(canonical_url_hash, captured_content_hash)` deduplicates research.
5. Initial `sendMessage` can be duplicated if the process dies between remote
   send and local receipt persistence. One persisted message id is edited for
   every later update so ambiguity cannot compound.
6. Web and Telegram prose are untrusted data. Agents receive immutable bounded
   captures, never permission to refetch the submitted URL.
7. Automatic research is safe; automatic repository mutation from an untrusted
   URL is not. `awaiting_approval` is a hard boundary.
8. GetZep Graphiti is temporal memory. The existing `graphify` skill is a
   different static code/corpus graph and remains untouched.
9. Graphiti uses its own provider credentials. Claude OAuth and Codex ChatGPT
   login do not authenticate it.
10. Oracle and Worker rule injection is already provider-neutral through
    `rules::agent_context_block`. Fix only the missing default-session skill and
    instruction discovery paths.
11. A `/duo` Codex-to-Claude fallback breaks the independent dual-model gate.
    Adoption records it as `single_model` and requires a second acknowledgement.
12. Existing rmux and Codex processes are diagnosed, never killed or restarted
    automatically.

## Architecture

```text
authorized Telegram link, one activated ingress
  -> SQLite WAL transaction: update, URLs, offset, state, outbox intent
  -> constrained fetcher: guarded DNS and redirects, bounded immutable capture
  -> automatic Claude research over the capture plus omega-mem lookup
  -> reject/defer, or awaiting_approval
  -> authenticated one-use approval
  -> durable omega dispatch receipt, explicit Claude Oracle
  -> Claude plan, Codex critique, Claude plan v2, Codex code, Claude review
  -> selected audits, install parity, commit, push
  -> verified Graphiti episode and Telegram message edit
```

`omega-mem` is the provider-neutral Graphiti interface. Graphiti MCP is
optional and never the source of truth.

## Phase 1: persistent Codex authentication and truthful `/duo`

Files:

- `crates/omega-core/src/codex_login.rs`
- `crates/omega-core/src/credentials.rs`
- `crates/omega-cli/src/main.rs`
- `tools/duo/bin/omega-duo`
- `skills/duo/SKILL.md`
- focused tests, `install.sh`, `scripts/verify-install.sh`

Changes:

1. Resolve `CODEX_HOME` and `config::omega_dir()` consistently.
2. Reconcile Codex credentials before login. After success, atomically adopt
   the fresh native file into the canonical store and restore the legacy
   symlink. On abandon, restore canonical then the symlink.
3. A newer valid native credential always wins. Never restore an older
   `last_refresh` over a newer target. Preserve losing valid copies in a
   mode-700 quarantine until a later cleanup.
4. Add an exclusive per-flow lock and flow record. Reject concurrent login
   starts. Success belongs to the recorded child only and requires child exit
   plus fresh credential evidence.
5. Extend login status with `Unknown`, distinct from `NotLoggedIn`. Spawn,
   timeout, or parse failures never kill a process or restore credentials.
6. Add a real auth probe for doctor and `/duo`; the shallow
   `codex login status` is topology information, not proof of a usable session.
7. Reconcile Codex alongside Claude on Omega CLI startup.
8. Diagnose stale or mixed-version Codex processes without signalling them.
9. Pass `/duo` task content through stdin. Preflight the read-only sandbox.
   When bwrap cannot execute local reads, use a worktree write guard, set
   `sandbox_degraded=true`, and fail on any mutation. Never report a
   repository-blind critique as green.
10. Extend `omega-duo --self-test` for large stdin tasks, auth failure, degraded
    sandbox, read capability, and read-only violation.

Success:

```bash
cargo test -p omega-core codex_login
cargo test -p omega-core credentials
omega-duo --self-test
codex exec --sandbox read-only --skip-git-repo-check \
  "Reply with exactly AUTH_OK"
```

Failure tests cover stale backup rollback, corrupt JSON, missing Codex binary,
concurrent login, stale PID, alternate roots, and a plan agent that writes.

## Phase 2: provider discovery parity and explicit repair

Files:

- `crates/omega-core/src/config.rs`
- new `crates/omega-core/src/agent_home.rs`
- `crates/omega-core/src/doctor.rs`
- `crates/omega-cli/src/main.rs`
- `install.sh`, `scripts/verify-install.sh`

Changes:

1. `omega sync` uses the canonical Omega root and `CODEX_HOME`.
2. Publish Omega skills additively to Claude and Codex. Replace only Omega-owned
   links. Preserve and report every foreign collision.
3. Preserve user `AGENTS.md`. Maintain exactly one atomic, idempotent OmegaOS
   managed block with mode preservation and a cross-process lock. Leave
   unrelated symlinks alone.
4. Add explicit `omega repair` for idempotent installed-asset repair. Do not
   change the meaning of an already-current `omega update` or `--check`.
5. Verify default Claude and Codex sessions can discover `link-research`.
6. Do not redesign Oracle or Worker context injection.

Tests use temporary `HOME`, `OMEGA_DIR`, and `CODEX_HOME`, repeat sync, preserve
foreign skills and user text, repair duplicate markers, and race two syncs.

## Phase 3: rescue, pin, and package Graphiti

Files:

- `tools/memory/omega_mem.py`
- `tools/memory/pyproject.toml`
- `tools/memory/uv.lock`
- `tools/memory/docker-compose.yml`
- `tools/memory/scripts/inventory.sh`
- `tools/memory/scripts/snapshot.sh`
- `tools/memory/scripts/canary-restore.sh`
- `tools/memory/scripts/cutover.sh`
- `tools/memory/scripts/rollback.sh`
- `tools/memory/tests/test_omega_mem.py`
- `install.sh`, `scripts/verify-install.sh`

Order:

1. Inventory the live image digest, mounts, Redis/Falkor configuration, graph
   names, label and relationship counts, Python packages, ledger, and a known
   provenance query.
2. Snapshot the live `dump.rdb` from the current container before any recreate,
   hash it, copy it to a mode-700 timestamped backup, and record the manifest.
3. Restore the snapshot into a separately named canary container on loopback and
   a different port. Compare graph names, counts, and provenance query output.
4. Pin `graphiti-core==0.29.2` and the measured FalkorDB image digest. Enable
   AOF and a correct persistent host mount, bind only `127.0.0.1`, and set
   `GRAPHITI_TELEMETRY_ENABLED=false`.
5. Because the single-group Falkor routing fix is newer than 0.29.2, keep the
   existing database/group invariant and add a multi-process ingest/query
   contract test. Upgrade only after a tagged fixed release or an explicitly
   pinned reviewed commit.
6. Cut over only after canary success. Keep the original container and snapshot
   until the new runtime survives ingest, restart, query, and provenance checks.
7. Installer preserves existing data and installs an idle runtime when Docker
   or provider credentials are absent.

`omega-mem` provides `inventory`, `status`, `stats`, `query`, `ingest`,
`snapshot`, `verify`, and `reconcile`. Graph writes are at-least-once with an
external stable job id and graph-side reconciliation.

## Phase 4: dedicated durable bot, standby everywhere

Files:

- `telegram-bot/link-research-bot.ts`
- `telegram-bot/lib/research-store.ts`
- `telegram-bot/lib/telegram-links.ts`
- focused Bun tests
- `scripts/omega-research-bot.sh`
- `scripts/omega-research-bot-up.sh`
- `crates/omega-core/src/backup.rs`
- `crates/omega-core/src/service.rs`
- `install.sh`, `scripts/verify-install.sh`

Store:

- Bun SQLite WAL under `${OMEGA_DIR}/research/research.db`.
- Tables for metadata/offset, Telegram updates, research requests, immutable
  transitions, captures, leases, approvals, dispatch receipts, and outbox.
- Every state change and transition row commit together.
- States:
  `received -> capturing -> captured -> researching -> awaiting_approval ->
  approved -> dispatching -> adopting -> verifying -> done`, plus
  `rejected`, `failed`, `dead`, and `superseded`.

Ownership:

1. Install the service disabled everywhere.
2. `omega-research-bot activate` reads a rotated token from stdin or a
   mode-600 secret, validates `getMe`, records a local machine binding, and
   enables only this unit.
3. Empty user or chat allow-lists deny everything.
4. A machine-binding mismatch never contacts Telegram.
5. HTTP 409 records `ingress_conflict` and enters a live standby loop.
6. Transfer deactivates the old ingress before printing activation instructions
   for the new one. Portable backups exclude the token and ownership marker.
7. Parse plain text, captions, `url`, and `text_link` entities. Cap URL count and
   text size.
8. Commit update rows, request rows, and the next Telegram offset in one
   transaction before the next poll.
9. Persist acknowledgement intent and message id. All progress and final output
   edits the persisted message.

Offline tests inject duplicate delivery, crash boundaries, 409, 429, competing
lease owners, unauthorized senders, corrupt SQLite, full disk, and restart.

## Phase 5: constrained fetcher and prompt-injection boundary

Files:

- `telegram-bot/lib/fetcher.ts`
- `telegram-bot/lib/capture.ts`
- `telegram-bot/lib/fixtures/injection/*`
- `telegram-bot/lib/fetcher.test.ts`
- `telegram-bot/lib/injection.test.ts`

Rules:

1. HTTP(S) only, ports 80/443, no userinfo.
2. Resolve at connection time and validate every A/AAAA result and redirect.
   Reject loopback, private, link-local, CGNAT, unique-local, multicast,
   reserved, metadata, IPv4-mapped, encoded, octal, and decimal variants.
3. Maximum five redirects, 30 seconds total, 10 seconds per connection, 5 MB
   compressed and 20 MB decompressed, strict textual MIME allow-list.
4. Strip ambient proxy variables. Never attach cookies, authorization, local
   credentials, or caller headers.
5. Write capture body mode 0400 and provenance sidecar under a mode-700
   directory using content hashes.
6. Downstream agents receive capture paths and metadata, never source-fetch
   permission. Telegram prose and captured content stay in an untrusted data
   envelope.
7. Accept only schema-valid decision JSON. Ignore free prose.
8. Fixtures try to reveal secrets, select tools/files, invoke `/duo`, approve,
   commit, push, close the envelope, and repeat attacks in French and HTML.
   Tests prove state cannot pass `awaiting_approval` and no write occurs outside
   the capture directory.

## Phase 6: automatic research and shared doctrine

Files:

- `skills/link-research/SKILL.md`
- `skills/link-research/agents/openai.yaml`
- `telegram-bot/lib/research.ts`
- `telegram-bot/lib/research.test.ts`
- `rules/R-LINKRESEARCH-verified-link-to-improvement.md`
- `crates/omega-core/src/rules.rs`
- `crates/omega-core/src/doctor.rs`
- `OMEGA.md`
- `install.sh`, `scripts/verify-install.sh`

Create the skill with the official skill-creator skeleton and replace every
placeholder.

The processor leases one request, reads immutable captures, fetches any primary
source only through the constrained fetcher, queries `omega-mem`, and emits
schema-valid `adopt`, `adapt`, `defer`, or `reject`.

`defer` and `reject` become terminal reports. `adopt` and `adapt` move to
`awaiting_approval` and generate a six-character, 24-hour, single-use approval
code; only its hash is stored. Three bounded attempts per stage precede `dead`.

`R-LINKRESEARCH` is scoped to Master, Oracle, and Worker. It requires primary
evidence, citations, untrusted-source isolation, explicit approval, exact
`/duo`, audits, install parity, and verified-memory-only writes.

The skill becomes discoverable to both default providers through Phase 2, while
the same rule text already reaches Claude and Codex Oracle/Worker prompts.

## Phase 7: approved adoption with durable dispatch

Files:

- `crates/omega-core/src/dispatch.rs`
- `crates/omega-cli/src/main.rs`
- `telegram-bot/lib/adoption.ts`
- `telegram-bot/lib/adoption.test.ts`
- `skills/link-research/SKILL.md`
- `scripts/omega-atlas-liveness.sh`

Approval:

- `/approve <request-id> <code>` works only for allow-listed senders.
- CLI approval works only on the ingress machine.
- Code comparison is constant-time; expiry and single use are transactional.
- Wrong, expired, reused, or unauthorized approvals are rejected and logged.

Dispatch:

1. `omega dispatch --request-id <id> --agent claude --json` writes an intent
   receipt before reserving a name or creating an rmux session.
2. It finalizes the receipt with Oracle and session identifiers.
3. `--ensure` reconciles incomplete receipts against live sessions before any
   retry creates a session.
4. One stable request never creates two Oracles.

Adoption:

1. Require a clean OmegaOS tree, sync Git, and acquire the repository writer
   lock with stale-holder detection.
2. Run exactly: Claude plan, Codex read-only critique, Claude plan v2, Codex
   full-auto implementation, Claude real-diff/runtime review, maximum three FIX
   rounds.
3. Degraded sandbox or single-model fallback requires another authenticated
   acknowledgement.
4. Run selected real audits, fix in-scope failures, repeat audits, preserve
   `install.sh`, build and test.
5. Commit and push, write `done.json`, edit the Telegram acknowledgement, and
   write only verified conclusion, provenance, dissent, test evidence, and
   final commit through `omega-mem`.
6. If memory write-back fails, the pushed commit remains valid and the episode
   stays queued for reconciliation.

## Global verification

Run focused tests after every phase, then:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
bun test telegram-bot/lib/
python -m unittest discover -s tools/memory/tests
omega-duo --self-test
omega doctor
omega audit select \
  "Codex auth Telegram link ingestion Graphiti memory skill sync dispatch installer"
OMEGA_FROM_SOURCE=1 bash install.sh
```

Run every selected real audit, fix in-scope findings, and rerun it.

Delivery order must respect the existing verifier:

1. Review the real diff and runtime evidence.
2. Run builds, tests, audits, and local idempotent install.
3. Fetch/rebase on a clean integration point.
4. Commit and push without force or bypass.
5. Run `./scripts/verify-install.sh` last because it requires a clean,
   pushed tree.

## Invariants

- No secret or token-shaped fixture is tracked.
- Fresh installs without secrets succeed and all new services remain idle.
- Existing configs, secrets, user instructions, Graphiti data, rmux sessions,
  and Codex sessions survive.
- One writer owns each file and repository mutation is lock-scoped.
- No `latest` image, public database listener, telemetry default, or destructive
  live-memory migration remains.
- Claims are cited; runtime gates, not delegate summaries, decide completion.
- The only operator actions are rotating/provisioning the bot secret and
  approving an adoption or a deliberately degraded dual-model run.
