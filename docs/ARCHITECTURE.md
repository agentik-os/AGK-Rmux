# OmegaOS — Architecture Documentation

> **Scope:** authoritative full-system reference (crates, orchestration,
> agent levels, CLI, channels). For the `~/.omega/` centralized runtime
> layout (credentials, models, settings), see [ARCHITECTURE-V3.md](ARCHITECTURE-V3.md).
>
> Complete technical reference for the OmegaOS system.
> Read this first to understand how everything fits together.

## Table of Contents

1. [System Overview](#1-system-overview)
2. [Directory Layout](#2-directory-layout)
3. [Credentials & Multi-Provider](#3-credentials--multi-provider)
4. [4-Level Agent Hierarchy](#4-4-level-agent-hierarchy)
5. [Communication Channels](#5-communication-channels)
6. [Quality Gates](#6-quality-gates)
7. [Installation Flow](#7-installation-flow)
8. [Configuration Reference](#8-configuration-reference)

---

## 1. System Overview

OmegaOS is a Rust-first agentic terminal operating system. Its core, CLI, TUI,
and orchestration engine are Rust; Bun/TypeScript powers Telegram and supporting
tools, while shell scripts bootstrap and operate the install. It supports
Claude, Codex, Gemini, GLM, OpenRouter, Pi, Hermes, Kimi, and local shell
sessions through a unified configuration and orchestration layer.

**Key components:**

| Component | Role |
|-----------|------|
| `omega` CLI | Main binary; `omega --help` is the command source of truth |
| TUI | 7-tab manager: Sessions, Projects, OS, Menu, System, Help, Settings |
| rmux SDK | Terminal multiplexer (sessions, panes, send/capture) |
| Atlas Telegram service | Persistent orchestration process that classifies and routes messages |
| AISB conversation viewer | Optional read-only `aisb-master` rmux mirror; not an agent |
| Quality Arsenal | 23 forensic audits (code, UX, perf, security, etc.) |

---

## 2. Directory Layout

```
~/.omega/                          MASTER — single source of truth
│
├── OMEGA.md                       Universal system prompt (all LLMs load this)
├── config.toml                    General settings
├── providers.toml                 Per-provider config
├── telegram.toml                  Telegram bridge config (gitignored)
├── projects.json                  Compatibility project-registry projection
│
├── credentials/                   OmegaOS-owned credential copies
│   ├── claude.json                ← ~/.claude/.credentials.json (symlink)
│   ├── codex.json                 ↔ CODEX_HOME/auth.json (reconciled)
│   ├── gemini.json                ← ~/.gemini/oauth_creds.json (symlink)
│   ├── glm.json
│   ├── openrouter.json
│   └── accounts/                  Saved account profiles
│       ├── claude-gareth.json
│       └── ...
│
├── rules/                         the typed doctrine (.md, editable — `omega rules list` prints the current set)
│   ├── L0-ship-the-truth...md
│   ├── L1-runtime-is-the-only-truth.md
│   └── ... (the Laws + registry-owned named R-* rules; inspect with `omega rules list`)
│
├── agents/                        Agent system prompts
│   ├── aisb-master.md             orchestration prompt retained for service compatibility
│   ├── oracle.md / worker.md / team-lead.md
│   └── aisb/                      15 typed Matrix agent templates
│       ├── oracle.md / morpheus.md / seraph.md
│       └── ... (15 total, including Trinity)
│
├── skills/                        OmegaOS-shipped skills (audits, design, planner…)
│   ├── pdfgen/                    PDF generator (Next.js + Playwright)
│   └── audits/                    Quality Arsenal audits
│
├── lib/                          Audit runtime (consolidated — NO ~/.aisb)
│   ├── audit-runner.sh           hybrid audit orchestrator
│   ├── audit-gather/             per-audit gatherers (.sh + -summarize.py)
│   └── safe-npm-build.sh         build mutex
├── bin/                          audit-notify.sh + helper binaries
├── repos/                        ← cloned GitHub repos (repos/Agentik-Skills = skills library)
├── tools/                        ← third-party tools an agent installs
├── prompts/                      runtime prompt scratch (oracle/worker dispatch)
│
├── docs/                          Reference docs (this file)
├── projects/<slug>/               Per-project overrides
│
├── state/                         Runtime (not in git)
│   ├── mission-engine-v3.sqlite3  Authoritative mission event ledger
│   ├── sessions/                  Active session metadata
│   ├── locks/                     Scope-claim file locks
│   ├── done/                      .done.json files
│   └── active-model.json          Global provider+model diagnostic mirror
│
├── logs/                          Session logs
└── audit/                         Audit results
```

### Placement convention — where new things go

One home, everything ordered. When an agent/LLM installs something, it lands here:

| Installing… | Goes to | How |
|---|---|---|
| OmegaOS skill (ships w/ product) | `~/.omega/skills/<name>/` | add to repo `skills/`; install.sh copies + makes `/<name>` |
| Global skill for every agent (SST) | `~/.claude/skills/<name>/` | `bunx skills add <repo> --skill <name> -g` |
| A GitHub repo (service/dep) | `~/.omega/repos/<name>/` | `git clone … ~/.omega/repos/<name>` |
| A third-party tool/binary | `~/.omega/tools/<name>/` + symlink in `~/.local/bin` | install there, link the entrypoint |
| An LLM provider CLI | provider default (npm-global / `~/.local/bin`) | `omega install <provider>` (on-demand) |
| Runtime scratch / state | `~/.omega/state` or `~/.omega/logs` | never the repo, never `~` root |

**No `~/.aisb` dual-home** — the audit runtime + state were consolidated into
`~/.omega/{lib,bin,state}`. Secrets live only in `~/.omega/credentials` /
`~/.omega/provisioning` (gitignored). Install parity (Law 0): if a fresh
`git clone && ./install.sh` wouldn't reproduce it, wire it into `install.sh`.

---

## 3. Credentials & Multi-Provider

### Principle

OmegaOS keeps canonical Claude and Codex credential copies under
`~/.omega/credentials/`. Gemini 0.56+ and Antigravity keep OAuth in their
native keyring/hybrid stores; Omega does not replace those stores with
symlinks. Codex is a special two-copy topology: its native `auth.json` stays under `CODEX_HOME`
(default `~/.codex`) while `omega codex-reconcile` compares, validates,
quarantines conflicts, and updates the canonical OmegaOS copy. This way:

- LLM CLIs still find their creds at the expected paths
- Omega-managed credentials are covered by `~/.omega/`; native Gemini,
  Antigravity, Hermes, and Kimi auth stores require provider-native backup/login
- Account switching = updating one file
- Migration is a one-time operation

### Supported Providers

| Provider | Type | Credential file | Default model |
|----------|------|-----------------|---------------|
| Claude | OAuth | `credentials/claude.json` | opus |
| Codex | ChatGPT device auth or API key | native `CODEX_HOME/auth.json`, reconciled with `credentials/codex.json` | gpt-5.6 |
| Gemini | Enterprise OAuth or API key | Gemini native keyring/store; API key in `providers.toml` | auto |
| Antigravity | Google sign-in | native keyring | native account default |
| GLM | API key | `providers.toml` | glm-5.3 |
| OpenRouter | API key, launched through Pi | `providers.toml` | anthropic/claude-opus-5 |
| Pi | OpenRouter config | `providers.toml` | anthropic/claude-opus-5 |
| Hermes | native provider or OpenRouter key | Hermes native config + `providers.toml` | anthropic/claude-opus-5 |
| Kimi | OAuth or API key | Kimi native OAuth or `providers.toml` | kimi-for-coding |
| Shell | local process | none | none |

### Account Switching

Named credential profiles live in `~/.omega/credentials/accounts/`. Manage
them through the TUI Settings surface or the Telegram `/account` menu; there is
no top-level `omega accounts` command.

```
/account                              Show and manage provider accounts
```

Behind the scenes: switching updates `~/.omega/credentials/claude.json`
to be a copy/symlink of the saved profile.

### Global provider/model selection

```
/model                       Show current + list available
/model codex                 Switch to Codex (OmegaOS default)
/model claude opus           Switch to Claude with opus
/model codex gpt-5.6         Switch to Codex with GPT-5.6
/model openrouter            Switch to OpenRouter (default model)
```

The same operation is available as
`omega config activate <provider> [model]`. It updates
`config.toml:agent_command`, the selected provider model, and the diagnostic
mirror at `~/.omega/state/active-model.json`. A mission-level `--agent`
override still wins for that mission.

---

## 4. 4-Level Agent Hierarchy

```
┌─────────────────────────────────────────────────────────────────┐
│  Level 1 — Human Interface                                      │
│  TUI (7 tabs) · CLI (`omega --help`) · Telegram Bridge          │
│                      ↓ intent                                    │
├─────────────────────────────────────────────────────────────────┤
│  Level 2 — Atlas Telegram orchestration service (persistent)    │
│  Matrix agent registry:                                          │
│    Oracle · Morpheus · Seraph · Keymaker · Smith · Niobe         │
│    Architect · Merovingian · Neo · Zion · Link · Construct       │
│    Pythia · Council · Trinity                                    │
│                      ↓ dispatch                                  │
├─────────────────────────────────────────────────────────────────┤
│  Level 3 — Oracle (1 per project)                                │
│  Classify → Plan (Keymaker) → Dispatch workers → Gate (Seraph)  │
│                      ↓ decompose                                 │
├─────────────────────────────────────────────────────────────────┤
│  Level 4 — Workers (ephemeral, parallel, file-lock scoped)      │
│  Execute → Verify → done.json → Oracle acks → close             │
└─────────────────────────────────────────────────────────────────┘
```

### Session Roles

| Role | Icon | Pattern | Purpose |
|------|------|---------|---------|
| AISB viewer | (hidden) | `aisb-master` | Optional read-only Telegram conversation mirror |
| Oracle | ◆ | `oracle-{Project}` | Strategic — classify, plan, dispatch |
| Worker | ● | `{Project}-worker-{task}` | Tactical — one task, scope-claimed |
| Home | ⌂ | `claude-1`, `codex-2` | Interactive user sessions |
| System | ⚙ | e.g. `omega-tg-bot` | Infrastructure services and viewers |

### Plan execution engine

For multi-step builds, the prose plan is replaced by a typed DAG:

1. **Planner** (`/omg-planner`) decomposes the work into single-worker-dispatch
   steps and emits `.planner/tracker.json` (typed steps, dependencies, a real
   `verify_command` per step).
2. **`omega plan-run`** drives the plan: it spawns a real worker per step in
   dependency order. Pre-run validation refuses skip-prone or fake-completing
   plans (trivial `verify_command`s like `true` are rejected).
3. **Gate** — structural can't-skip enforcement: a step only advances when its
   worker's `done.json` lands and the step's invariants hold. Sequencing is
   enforced by the engine, not by instructions.
4. **Guardian** — independent verification: the step's `verify_command` is run
   by the engine itself, so a worker's self-report alone never completes a step.

`omega plan-status` prints progress from `.planner/tracker.json` read-only.

### Worker isolation

Three mechanisms keep parallel workers from trampling each other:

- **Scope-claim file locks** — before writing, a worker claims its files with
  real advisory locks (`fs2`, `scope.rs`). Overlapping claims are rejected at
  dispatch (`omega scope`).
- **Worktree isolation** — `omega spawn-worker --worktree` gives each parallel
  worker its own git worktree, so simultaneous mutations never share a checkout.
- **Clean merge** — `omega-git-merge` folds worker worktrees back into the main
  branch when they finish.

---

## 5. Communication Channels

### LLM ↔ OmegaOS

Each LLM CLI reads OmegaOS config:

| LLM | Mechanism |
|-----|-----------|
| Claude Code | `~/.claude/rules/omega-*.md` → symlinks to `~/.omega/rules/` |
| Gemini CLI | `~/.gemini/GEMINI.md` includes `@import ~/.omega/OMEGA.md` (Enterprise/API-key lane) |
| Antigravity | Agent Skills / project instruction discovery through `agy` |
| Codex | `~/.codex/AGENTS.md` → symlink to `~/.omega/OMEGA.md` |
| Pi / OpenRouter / Hermes / GLM / Kimi | Role-scoped Laws and Rules are included in the dispatched mission prompt |

### User ↔ OmegaOS

| Channel | How |
|---------|-----|
| TUI | `omega` or `omega menu` — full session manager |
| CLI | `omega <cmd>`; inspect the current surface with `omega --help` |
| Telegram | `/help`, `/list`, `/model`, `/newproject`, `/account`, etc. |

### Inter-process

| Mechanism | Used for |
|-----------|----------|
| rmux SDK (Unix socket) | Session create/kill, send_text, capture_pane |
| JSONL files | done.json signals, inbox events |
| Telegram Bot API | Long-poll, sendMessage, sendDocument, sendChatAction |

---

## 6. Quality Gates

Every mission passes through the rules:

| Rule | Enforces | Implementation |
|------|----------|----------------|
| L1 | Runtime truth | `gate.rs` requires log/pane evidence |
| L2 | Researcher mindset | Worker prompt template |
| L3 | Decide & proceed | `oracle_lifecycle.rs` no idle stops |
| R-PROD | Ship verification | `ship.rs` polls deploy URL until 200 |
| R-RUBRIC | Rubric upfront | `rubric.rs` writes criteria before dispatch |
| R-VERIFY | Multi-grader (2/3) + Popper falsification | `gate.rs` runs 3 adversarial lenses |
| R-BUDGET | Token budget | `mission.rs` tracks spend, hard cap |
| R-CITE | Citation required | `verifier.rs` rejects uncited claims |
| R-TGSEC | Telegram allow-list | `monitor.rs::is_authorized` |
| R-SCOPE | One writer per file | `scope.rs` rejects overlap at dispatch |

---

## 7. Installation Flow

### Fresh install (`./install.sh`)

```
Phase 1: Detect OS + arch
Phase 2: Install Rust (rustup) if missing
Phase 3: Build rmux from source
Phase 4: Build omega CLI
Phase 5: Setup ~/.omega/ + migrate existing credentials
         - Create credentials/, accounts/, state/, bin/
         - Move ~/.claude/.credentials.json → ~/.omega/credentials/claude.json
         - Create symlinks back to legacy paths
         - Migrate Claude and Gemini compatibility credentials
         - Reconcile Codex's native and canonical copies without replacing
           its native home or ignoring CODEX_HOME
         - Copy OMEGA.md, agents/, rules/, skills/
         - Run `omega rules export` and `omega sync`
Phase 6: Shell integration (PATH, completions, aliases)
```

### Post-install (`omega install <provider>`)

When a user installs a new LLM CLI:
1. Run the official installer (curl|sh pattern with TTY preserved)
2. `omega sync` — symlinks into LLM config dirs
3. Reconcile credentials with the provider-specific topology. Claude/Gemini
   can use compatibility migration; Codex uses `omega codex-reconcile`.

---

## 8. Configuration Reference

### `~/.omega/config.toml`

```toml
state_dir = "~/.omega/state"
logs_dir = "~/.omega/logs"
auto_spawn_master = false      # optional legacy read-only viewer
auto_naming = true             # Sessions named claude-1, codex-2, ...
```

### `~/.omega/providers.toml`

```toml
[claude]
model = "opus"

[codex]
model = "gpt-5.6"

# The built-in provider default is Codex. `providers.toml` contains typed
# per-provider settings; `omega config` manages the active selection.
```

### `~/.omega/telegram.toml` (gitignored)

```toml
bot_token = "..."
chat_id = ...
allow_user_ids = [...]
enabled = true
```

---

## CLI Quick Reference

```
omega                    Launch TUI
omega menu               Same as omega
omega aisb-view          Open read-only AISB conversation viewer
omega master             Compatibility alias for omega aisb-view
omega aisb-chat          Interactive local chat through the Telegram service
omega list               List sessions
omega new <name>         Create session
omega kill <name>        Kill session
omega dispatch <P> <M>   Send mission to oracle
omega orchestrate <P> <M>  Full pipeline (classify → plan → dispatch → gate)
omega rules list         Show the current Laws + operational Rule registry
omega rules export       Write to ~/.omega/rules/
omega sync               Symlink to all LLMs
omega config show        Show provider configuration (secrets redacted)
omega config models      List canonical providers and models
omega config set <provider>.<key> <value>
OMEGA_TG_TOKEN=<TOKEN> omega telegram setup <CHAT> --user-id <UID>
omega telegram run       Start bridge
omega pdf --template=... --send
omega install <agent>    Install LLM CLI + auto-sync
omega projects           Auto-discover projects
```

---

## Verification

To verify the architecture is correctly set up:

```bash
ls ~/.omega/                              # Should show: credentials/ rules/ agents/ skills/ ...
ls ~/.omega/credentials/                  # Canonical OmegaOS credential copies
omega codex-reconcile --json              # Validate/reconcile Codex topology
omega rules list                          # Should match the runtime Rule registry
cargo build --workspace --locked          # Should be 0 errors
```
