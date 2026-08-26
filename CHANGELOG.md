# Changelog

All notable changes to OmegaOS are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project aims
for [semantic versioning](https://semver.org) once it reaches 1.0. Until then,
`main` is the only supported line.

## [Unreleased]

- Topology-agnostic AGK: the same monorepo runs single-user, multi-user,
  container, or hybrid. `~/.omega/topology.yaml` wins; `omega agk topology
  {status,detect,plan,init}` never migrates automatically. Paths and Hermes
  homes are derived from `profile_id`.
- Control-plane contracts: canonical IDs, event bus, approval engine, policy
  engine, observability snapshot, specialist registry + OS team composition,
  Discord surface (exactly four bots = four profiles; bot ≠ agent).
- Encoded AGK doctrine as operational rules **R-PLANE**, **R-AGK**, **R-LAYER**,
  **R-CANON**, **R-RESOLVE**, **R-TOPO**, **R-SURFACE**, **R-ID**, **R-EVENT**,
  **R-APPROVE**, **R-POLICY**, **R-TEAM**. The compact kernel now carries an
  AGK layers block for every provider. `omega agk provision` seeds canonical
  workspace trees and writes `~/.omega/AGK.md`.
- Hermes messaging gateway is wired into OmegaOS: `omega hermes-gateway`
  (install/setup/start/status), systemd PATH drop-in so `omega` is visible,
  `omega doctor` health + `--fix` start, and a hard fail if Hermes reuses the
  Atlas Telegram token. `install.sh` installs the unit after the Hermes CLI.
- Hermes is a first-class Home stream: `install.sh` runs `omega install hermes`
  (non-interactive: `--skip-setup --skip-browser --skip-computer-use`), and
  `omega sync` always creates `~/.hermes` with a SOUL.md kernel pointer,
  AGENTS.md link, curated skill links, `skills.external_dirs`, and the
  `/omegaos` bundle. Home panes export `HERMES_HOME` and prepend Hermes bins
  on PATH. Hermes stays Home-only — never a dispatch writer.
- Superpowers + gstack third-party packs are opt-in (`OMEGA_WITH_THIRD_PARTY=1`)
  instead of always-on. `OMEGA_SKIP_THIRD_PARTY=1` still skips.

- Restored agent-pane colors when rmux inherits Cursor's `NO_COLOR`.
- Separated Pi (standalone) from OpenRouter. AISB doctrine is 15 agents
  including Trinity, with named rules (`R-RUBRIC` / `R-VERIFY` / `R-CITE`)
  and a shared quality kernel.
- Mapped Laws/Rules per harness: Claude, Codex, Gemini, and Other
  (Hermes / OpenCode / Pi / Kimi) share the same kernel and get a native-tool
  overlay. `omega sync` now writes `~/.config/opencode/AGENTS.md`, a Hermes
  `SOUL.md` pointer, and AISB protocols. `omega rules context --provider`
  previews the overlay. Oracle briefs no longer assume TaskCreate / `/goal`.
- Published the matching npm bootstrap as `omega-os@1.5.15`.

- Published the matching npm bootstrap as `omega-os@1.5.14` so `npx omega-os`
  clones `main` with worker project cwd, record-only Verify Command, and the
  Codex `--sandbox` / `--ask-for-approval never` launch pair.

## [0.1.14] — 2026-08-24

### Provider compatibility and installation

- Updated Codex, Claude, Hermes, Kimi, Gemini, Antigravity, GLM, Pi, and
  OpenRouter launch contracts and current model catalogs.
- Added streamed Codex gateway chat, provider-aware resume/resurrection,
  version diagnostics, safe Codex 0.149 migration, and configurable hook trust.
- Restored fresh-install parity: Codex is provisioned as the default, gateway
  binaries ship in release archives, Rust is pinned, and managed assets prune
  stale files without discarding operator configuration.
- Expanded CI to the complete workspace and made gateway/project/skill tests
  hermetic. Published the matching npm bootstrap as `omega-os@1.5.13`.

### Documentation and release operations

- Aligned current operator docs with the runtime registries: 7 Laws, 52
  operational Rules, 23 forensic audits, 15 typed Matrix agents, and 24
  canonical OS products plus five compatibility aliases.
- Corrected the pending reauthentication marker to
  `~/.omega/state/pending-reauth.json` and documented its five-minute lifetime
  separately from the 30-second trigger cooldown.
- Added a release and rollback runbook with CI, artifact, provenance, and
  known-good-tag verification. The runbook explicitly treats main-branch
  protection as an external setting that must be checked, not assumed.

### Changed
- **Orchestration V3 release candidate.** Mission and task-attempt state now
  live in a SQLite WAL event ledger with expected-version CAS, idempotency,
  fenced leases, and an outbox. Worker completion is a candidate claim, not
  acceptance; Oracle delivery requires accepted attempts and recorded gates.
- **Codex is the fresh-install default.** New OmegaOS, Oracle, and project
  sessions select Codex while preserving every existing explicit provider
  choice. Session provider identity is persisted for the TUI.
- **Codex/rmux view fidelity.** Long Unicode input reflows to pane width, the
  cursor follows the reflowed row, ANSI color survives capture, and the active
  provider has a stable visual accent.
- **Canonical skills and audits.** `SkillCatalogV1` drives install, Atlas, RAG,
  and provider activation. The audit TOML now drives the Rust registry, while
  the audit runner rejects incomplete evidence and validates final verdicts.
- **Compact doctrine compiler.** All seven Laws remain universal; operational
  rules are selected by role, mission, risk, and provider under a hard 24 KB
  budget with a deterministic digest. Codex plans are recognized by the finish
  guard alongside Claude plans.
- **R-MODEL: Mythos safety boundary.** Fable 5 (`claude-fable-5`) ships safety
  classifiers that decline cybersecurity / bio / chem / model-distillation work
  (`stop_reason: "refusal"`). On the raw API a server-side fallback re-serves on
  Opus 4.8, but in an agent context a refusal is an ABORT (L5). R-MODEL now
  DISQUALIFIES Fable 5 from security/pentest/red-team missions (R-SEC, R-TRINITY,
  /hack, /secaudit) and bio/chem/distillation work — tier them to Opus 4.8 — and
  treats a false-positive classifier block on benign adjacent work as a re-tier
  signal, not a done. Sourced from a Fable-5 self-improving-systems article and
  verified against the claude-api SSOT (the article's "~5× Opus cost" claim was
  wrong — Fable 5 is ~2×; the classifier-decline behavior was confirmed).

### Added
- **Daily update check + automatic apply.** Every install now schedules
  `omega update --auto` at 03:30 (`OMEGA-CRON-AUTO-UPDATE-v1`): it checks for a
  new commit and installs it, so a box stays current without anyone logging in.
  The apply is deliberately timid and every refusal is logged with its reason —
  it never touches a checkout with local changes or unpushed commits, defers a
  night when an agent is mid-turn rather than rebuilding under a running mission,
  stops after `FAILURE_CAP` (3) failed installs of the same commit and escalates
  instead of thrashing (R-LOOP), and holds a single-flight lock (stale after 6h)
  so two runs can never write the binary at once. Telegram alerts on applied /
  available / needs-you; silence means nothing to do. Auto-installing renews the
  repo's trust nightly, so the switch is one command:
  `omega config set auto_update apply|check|off` (default `apply`). The decision
  layer is pure and unit-tested (`omega_core::auto_update::decide`).
- **System tab — the doctrine, the agents and the manual, in the TUI.** The menu
  that showed the Laws, the Rules and the AISB roster had lost its own tab (Info
  → renamed Agentic → Agentic repurposed into Projects), surviving only as a
  buried group above the project list. It is a top-level tab again, and now also
  carries an Overview (the four levels, live registry counts, install paths), the
  installed Skills, and the whole Documentation tree — readable offline from
  `~/.omega/docs`, which the installer mirrors along with the root canon
  (README, CLAUDE, RULES, GUIDE, …) under `docs/canon/`.
- **Native `/loop` doctrine (R-LOOP extended).** After the Claude Code `/loop`
  launch, every loop-driving OmegaOS agent now knows the two native loop layers
  compose: the OmegaOS *mission* loop and the *native `/loop`* (fixed-interval
  cron-backed vs dynamic self-paced via `ScheduleWakeup`). Sessions running
  inside a native loop pace by the 5-minute prompt-cache window (60-270s warm /
  1200-1800s idle / never 300s), never poll harness-tracked background work, keep
  a long fallback heartbeat, and keep the same bounded-retry ceilings (thrash →
  `escalate_to_human`). Encoded in `rules.rs` (SSOT, R-LOOP) and injected into the
  oracle, worker, Atlas, and Nova/companion identities.
- **Claude Changelog Adopt (`/changelog-adopt`) — the self-improvement loop.**
  A daily, DISARMED-by-default pass that reads the OFFICIAL Claude Code changelog
  (`anthropics/claude-code`, @claudecodelog as mirror), diffs it against the last
  version OmegaOS absorbed, classifies each NEW entry for an OmegaOS/agent
  improvement (opus), adversarially gates the proposals (opus, anti-fabrication),
  writes a self-contained HTML report + Telegram alert, and — only when ARMED —
  dispatches vetted, in-scope adoptions (doctrine/agents/skills/install.sh, never
  core-Rust) to an oracle behind the quality gate (In-Review handoff, never
  auto-Done). This is the standing, automatic version of the manual `/loop`→R-LOOP
  adoption above. Skill + scripts + disarmed 08:15 cron shipped via install.sh;
  first real run classified `/review`→`/code-review` as a high-relevance adoption.

## [0.1.6] — 2026-06-11

The full-system audit release: 90 adversarially-verified findings, ~70 fixes
across the mission state machine (patrol races, worker done-signal freshness,
progress protocol revived), the plan engine (crash-resume adoption, Guardian
timeout, honest gates), the TUI (per-frame costs, preview fidelity), the
Telegram layer (deposit bot shipped, /council wired), install reproducibility
(Nova vendored, honest verify gates, safe migrations, prebuilt can no longer
downgrade a fresh build), rmux colors via the bumped rmux-sdk pin `2488ef5`
(redraw style continuity, OSC 10/11 replies, per-client truecolor downgrade),
plus GUIDE.md and the doc-layer realignment. Details in the commits below.

### Added
- **llm-council** — convene four different Claude models on one question:
  independent answers in parallel, anonymous peer review, an Opus president
  synthesizes the verdict and surfaces the dissent. Ships as the
  `/omg-llm-council` skill, the `@council` Matrix agent (the 14th), and the
  R-COUNCIL rule. Runs 100% on the Workflow primitive inside your existing
  Claude Code session — no API keys, no extra cost.
- **browser-use** — agentic cloud-browser skill (`/omg-browser-use`) plus the
  R-BROWSER rule (when to use agentic browsing vs scripted Playwright), with
  least-privilege key handling and error redaction.
- **Marketing + visual-identity pack** — 10 vendored skills (market-research,
  marketing-strategist, product-marketing-context, content-strategy,
  social-content, cold-email, ad-creative, launch-strategy, and the Higgsfield
  soul-id/generate pair), governed by the new R-MARKETING and R-VISUAL-ID rules.
- **Worktree isolation for parallel workers** — `omega spawn-worker --worktree`
  gives each parallel worker its own git worktree on top of the scope-claim
  file locks, with a clean merge back when workers finish.
- **Mission PDF reports** — every mission ends with a branded PDF report
  (including a linked "steps to verify" section) delivered to the project's
  Telegram topic.
- **Live Telegram progress card** — one card per oracle, edited in place as
  plan tasks complete (`omega progress`), with a hard flood ceiling so a
  mission never spams the topic.
- **Maintenance skills** — `cleanup` (disk/session/cache hygiene),
  `project-tidy` (de-sprawl a repo polluted by agent output), and `ramflush`
  (kernel cache purge + perf report), wired into the Telegram bot's Clean hub.
  New R-SKILLPUB rule: every new skill ships to the library + OmegaOS.
- **Companion agent bots** — a "Cowork" bot kind: an instant Haiku co-worker on
  its own Telegram bot, evolved into the self-improving personal companion
  (inline `/menu`, Composio app connections).
- **Deposit bot** (`omega-os` 1.5.4) — a private Telegram inbox so the operator
  can send photos/notes from their phone that an agent reads in `~/.omega/inbox/`
  (timestamped, captioned, indexed). Ships + auto-starts like the command bot
  (`telegram-bot/inbox-bot.ts`, `omega-inbox-bot.service`); connect with
  `inbox-bot-up <BOT_TOKEN>` (or `OMEGA_DEPOSIT_TOKEN=<TOKEN> inbox-bot-up`).
  Token in `~/.omega/deposit.toml` (0600, R-TGSEC); the bot self-locks to the
  first chat that messages it. The command bot now ingests **any** file type the
  operator sends, not just images.
- Claude **Fable 5** (`claude-fable-5`) wired into the provider catalog and
  alias resolver.

### Changed
- The canonical Telegram setup command is the env-prefix form
  (`OMEGA_TG_TOKEN=<TOKEN> omega telegram setup …`) everywhere — installer
  wizard, runtime echoes, and docs; the token never appears in argv.

### Fixed
- Doctor: robust Telegram-bot poller detection, no false "duplicate pollers"
  warning on a headless Mac, and the expected-rules counter follows the
  registry.
- macOS release builds: the retired `macos-13` Intel runner is replaced by
  cross-compiling `x86_64-apple-darwin` from `macos-14`.

## [0.1.5] — 2026-06-06

### Added
- Guided Telegram setup in the npx installer (`omega-os` 1.5.0): before the
  Matrix animation takes the screen, an interactive wizard walks through
  BotFather bot creation, validates the token live (`getMe`), auto-detects
  your chat id from your first message to the bot (`getUpdates`), then wires
  everything via `omega telegram setup` once install.sh succeeds. Skipped
  when non-interactive (no TTY / CI), with `--no-telegram`, or when
  `~/.omega/telegram.toml` already exists (re-installs keep the config).
  Queue-based prompt input — multi-line pastes and scripted stdin are never
  dropped; stdin EOF degrades to "skip", never a hang.
- **`/omg-acceptance`** — autonomous browser-acceptance + self-heal gate:
  Playwright-sweeps every route, captures every console error and failed
  network request, walks the authenticated golden path with a real persisted
  write, then fixes what it finds and re-runs until green. Builds must prove
  they WORK at runtime, not just compile.
- Transparent Dark & Light TUI themes (no painted background — the terminal's
  own bg shows through), bringing the gallery to 17 palettes; the Omega chrome
  theme applied to rmux itself; a 1-row/2-col breathing-room margin around the
  whole TUI.

### Fixed
- Pasting very long text (≳8 KB, e.g. 10k characters) into an attached rmux
  client corrupted the paste: rmux pin bumped `726d9e7` → `4455da0`, whose
  stateful `PasteFilter` keeps a paste spanning several client `read()`
  bursts ONE bracketed block. Previously the per-burst heuristic re-wrapped
  the middle bursts of a host-bracketed paste (the synthetic `201~` closed
  the paste early — the rest went in raw and every embedded newline submitted
  as Enter), and split an unbracketed (SSH/Termius) paste into several
  `[Pasted text]` blocks. Runtime A/B proven with a 10,200-byte paste
  recorded off the pane PTY: old = 2-3 corrupted blocks with an injected
  `\r` at the read boundary, new = 1 block, body intact byte-for-byte.
- Telegram bot never ran on macOS: the service install was systemd-only, so
  `omega telegram setup` wrote the config but every message went unanswered.
  `install.sh` now installs a launchd LaunchAgent on Darwin
  (`~/Library/LaunchAgents/os.omega.tg-bot.plist`, RunAtLoad + KeepAlive,
  logs in `~/.omega/logs/tg-bot.log`) with the same semantics as the Linux
  unit: always running, waits for the token, auto-restarts.
- macOS install hung at ~20% behind the npx Matrix animation: Phase 2 wrote
  the Debian-ism `/etc/default/locale` via `sudo`, whose password prompt was
  invisible behind the full-screen animation. `ensure_utf8_locale` now no-ops
  on Darwin (natively UTF-8), every Phase-2 sudo is non-interactive
  (`sudo -n` — fail loud, never prompt), and `bootstrap_os_packages` gains a
  Homebrew branch plus a Darwin-without-brew soft path (missing rsync/jq
  warn-and-continue; only git/curl are fatal).
- Five adversarial hardening passes (fix4–fix7b) across the TUI, CLI/core,
  installer, and Telegram bot: state-driven confirm flows (no armed-confirm
  class), a11y/uiux remediation of the theme engine, tokens kept off argv,
  portable poller healing, runtime bun resolution in agent-bot units, and
  npm-wizard failure-path hardening.

## [0.1.4] — 2026-06-05

### Added
- TUI theme selector (Settings → Theme): 15 selectable palettes — Omega
  (default), Matrix, Terminal, Amber, Noir, Paper, Monogram, Dracula, Nord,
  Gruvbox, Solarized Dark, Tokyo Night, Synthwave, Ocean, Crimson. Every
  theme except Omega paints its own full-screen background and text color
  and follows the "Monogram model": a quiet grayscale chrome plus ONE
  signature accent per theme (Matrix green, Dracula purple, neon pink, …);
  Omega keeps the terminal's own background and classic multicolor chrome.
  The selector live-previews each theme while arrowing through it, the
  gallery renders each row on that theme's background, the choice persists
  in `~/.omega/config.toml` (`theme = "..."`), and every chrome color in the
  TUI goes through semantic theme roles. The session-pane preview keeps the
  agent's own colors untouched.
- WCAG-AA contrast contract in the TUI theme engine, enforced by unit tests:
  every text-bearing role (text/dim/info/error/warn/bright and the accent
  family) ≥ 4.5:1 vs background, selection text ≥ 4.5:1 on both accent and
  accent2 bars, a contrast-vs-background hierarchy `dim2 < dim < text` (the
  raw luminance order inverts on light themes), and a role-vs-role
  distinctness floor — warn and error vs the accent at CIE76 ΔE ≥ 30, so an
  alert never blends into active text (Noir and Paper exempt, mono by
  design) — palettes can no longer regress below readable. New semantic
  `warn` role (the blocked badge is now themed; it was hardcoded orange);
  orange-accent themes (Amber, Gruvbox) move warn to the alert-red family;
  per-theme dim/dim2 retuned to meet AA while staying visually quieter than
  body text. Omega stays 100% named ANSI — warn included, an adaptive light
  red — so it keeps inheriting the terminal's own palette. Documented in
  `docs/THEMES.md`.
- `omega plan-run` strict pre-run validation: refuses skip-prone or
  fake-completing plans (trivial `verify_command`s rejected), and every
  worker brief gets the mandatory `omega done <session>` completion signal
  injected — the reason builds used to stall at 0%.

### Fixed
- Mouse-wheel scroll dead in every rmux pane (regression in rmux `0e4abb2`):
  the client-side paste heuristic wrapped a batched wheel burst (3+ SGR mouse
  reports ≥ 32 bytes, no newline) as a bracketed paste, so the server pasted
  the sequences into the PTY instead of decoding scroll. rmux pin bumped to
  `726d9e7`, which exempts ESC-initiated bursts from paste synthesis.
- TUI chat focus: session selection re-anchors by NAME across refreshes (the
  chat keystream can never silently retarget another session) and Esc in chat
  focus returns to the list — local, never forwarded to the agent.
- Per-project Telegram agent-bot units are resurrected at startup, closing
  the reinstall gap; album/caption-split Telegram fragments aggregate into
  ONE mission.

## [0.1.3] — 2026-06-05

### Added
- Full new-project pipeline in the product: vision → PRD → brand-identity
  (opt-in) → planner → build, with custom stack choice and a dedicated vision
  oracle; Claude Design import; optional Telegram step in the flow.
- Visible project actions in the TUI Agentic tab with a Telegram-parity
  3-tier delete menu (one Actions menu, line per action).
- Project folders are pre-trusted before every claude launch (no interactive
  trust dialog in dispatched sessions).

### Fixed
- Telegram photo/image messages are no longer silently dropped.
- Patrol stale-reap kill chain + lost gate-upgrade notification.
- `install.sh` reads `OMEGA_VERSION` from the workspace Cargo.toml (was
  hardcoded 0.1.0).

## [0.1.2] — 2026-06-05

### Added
- Guided first-run onboarding: `omega guide`, `~/.omega/GETTING-STARTED.md`,
  and an ordered install epilogue.
- R-PDF rule: all PDFs go through the OmegaOS pdfgen (single source of truth).
- Telegram: Projects → Import from GitHub workflow; provider API keys can be
  deleted from the Model menu.

### Fixed
- Workers died at login: `--bare` dropped from the Claude worker launch (bare
  mode skipped OAuth credential loading).
- Full bypass-permissions on every session + no reports in the Atlas topic
  (routing fix); oracles auto-close when finished (L4 gate-pending upgrade +
  deterministic patrol reap).
- Guard: no more false-blocking of files under `$HOME`; catastrophic alerts
  route through the canonical alert funnel to the Alerts topic.

## [0.1.1] — 2026-06-05

### Added
- Telegram hub maturity: free text routed to the AISB Master brain, one topic
  per project, a dedicated undeletable Alerts topic, designed Ω report cards
  with action buttons, live progress-bar card, conversation history +
  reply-to-message routing, voice input (Whisper), `/start` welcome + `/guide`,
  3-tier project delete, per-project dedicated-bot menu, morning briefing.
- Oracle engineering contract: git-sync, plan/100%, branch-per-worker,
  audit-on-code, L4 completeness gate (done_clean downgraded to pending if the
  plan isn't 100%), end-of-mission notifier (done.json → Telegram), and
  stuck-oracle alerts.
- `omega doctor --fix`, the self-heal daemon, token-budget alerts at
  80/85/90/95%, and a destructive-op audit tripwire (PreToolUse hook).
- GitHub Actions CI: build the workspace with `-D warnings` and run the test
  suite as hard gates; clippy and rustfmt run as advisory steps.
- Hand-written, human-voiced README with French, Russian, and Chinese
  translations, plus a "How a mission runs" section explaining the
  Master → oracle → worker → workflow flow.
- Contributor docs: this changelog, `CONTRIBUTING.md`, `SECURITY.md`,
  `CODE_OF_CONDUCT.md`, and issue/PR templates.

### Changed
- Terminal output (TUI and CLI) is now emoji-free, using the `[+]/[~]/[x]`
  ASCII convention. Telegram messages keep their emoji.
- TUI menu reorganized into Sessions · Menu · Agentic · Settings · Help, with
  the brain retired from the session menu (it lives in the Atlas Telegram
  topic).

### Fixed
- Atomic credential writes in the OAuth helper — no more 0-byte window — and
  fresh tokens are no longer discarded when healing the legacy symlink.
- `credentials` test that was flaky under parallel runs (it mutated the global
  `HOME`); the HOME-touching tests are now serialized.
- Dead code removed across the orchestration and TUI crates so the workspace
  builds with zero warnings.
- Oracle respawn no longer trusts a stale registry entry, and the patrol daemon
  re-checks session liveness before auto-marking a worker done.
- Dispatch: "Session ID already in use" — a persisted `--session-id` is never
  reused.

## [0.1.0] — 2026-06-03

Initial public cut. The `omega` CLI and TUI, the rmux-backed session model, the
typed doctrine (the 6 Laws and the named Rules) injected into every dispatched
agent, the oracle/worker orchestration layer, the Quality Arsenal audits, and
the optional Telegram bridge.
