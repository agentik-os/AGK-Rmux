# OmegaOS — Where Everything Lives

> Quick reference: source code vs runtime data vs installed binary.

## 3 Locations You Need to Know

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. SOURCE CODE (the repo)                                       │
│    ~/Station/SideBusiness/OmegaOS/                              │
│    github.com/agentik-os/OmegaOS                                │
│    → Edit code here, push to GitHub                             │
├─────────────────────────────────────────────────────────────────┤
│ 2. INSTALLED BINARY                                             │
│    ~/.local/bin/omega                                           │
│    → The installed Rust executable                              │
│    → Installed by `./install.sh` or `omega update`              │
├─────────────────────────────────────────────────────────────────┤
│ 3. RUNTIME DATA (the master config)                             │
│    ~/.omega/                                                    │
│    → Credentials, rules, agents, skills, state, logs            │
│    → Created by install.sh, edited by `omega` commands          │
│    → THIS is what all LLMs read from                            │
└─────────────────────────────────────────────────────────────────┘
```

## Source Code Layout (`~/Station/SideBusiness/OmegaOS/`)

```
OmegaOS/
├── crates/                            ALL RUST CODE
│   ├── omega-core/src/                Core library
│   │   ├── account.rs                 Account management
│   │   ├── agents.rs                  LLM agent registry
│   │   ├── aisb.rs                    Legacy AISB viewer compatibility
│   │   ├── aisb_agents.rs             15 typed Matrix templates
│   │   ├── audit.rs                   23 Quality Arsenal audits
│   │   ├── bootstrap.rs               Project bootstrap pipeline
│   │   ├── credentials.rs             Multi-provider credential store
│   │   ├── dispatch.rs                Worker dispatch
│   │   ├── done.rs                    Done signal protocol
│   │   ├── formatting.rs              HTML/Markdown formatting
│   │   ├── gate.rs                    Quality gates (R-19/R-21/R-30)
│   │   ├── inbox.rs                   JSONL event queue
│   │   ├── intent.rs                  Intent parser
│   │   ├── mission.rs                 Mission tracking
│   │   ├── monitor.rs                 Billing/Telegram config
│   │   ├── oauth.rs                   OAuth flow
│   │   ├── oracle_lifecycle.rs        Oracle state machine
│   │   ├── orchestration.rs           4-level orchestration
│   │   ├── patrol.rs                  Stall detection
│   │   ├── planner.rs                 DAG-enforced planner
│   │   ├── project_manager.rs         Project CRUD
│   │   ├── providers.rs               Provider catalog
│   │   ├── router.rs                  Smart routing
│   │   ├── rubric.rs                  Success criteria
│   │   ├── rules.rs                   The typed doctrine (7 Laws + named Rules)
│   │   ├── scope.rs                   File-lock scope claims
│   │   ├── session.rs                 rmux SDK integration
│   │   ├── ship.rs                    12-step ship pipeline
│   │   ├── skill_registry.rs          Skill discovery
│   │   ├── team.rs                    Multi-agent teams
│   │   ├── verifier.rs                Intent verification
│   │   └── ... (more)
│   │
│   ├── omega-tui/src/                 Terminal UI (ratatui)
│   │   ├── app.rs                     App state (7 tabs)
│   │   ├── input.rs                   Keyboard + mouse handling
│   │   ├── theme.rs                   Theme engine (palette gallery, Settings → Theme — see docs/THEMES.md)
│   │   └── ui.rs                      Tab rendering
│   │
│   ├── omega-cli/src/                 CLI binary (`omega --help` is the inventory)
│   └── omega-gateway/src/             Optional HTTP gateway
│
├── agents/                            Agent system prompts (markdown)
│   ├── aisb-master.md
│   ├── oracle.md / worker.md / team-lead.md
│   └── aisb/                          15 Matrix agent templates
│
├── rules/                             The typed doctrine (.md) — 7 Laws + named R-* Rules
│   ├── L1-runtime-is-the-only-truth.md
│   ├── R-VERIFY-adversarial-verification.md
│   └── ... (`omega rules list` prints the current set)
│
├── skills/                            Bundled skills
│   ├── pdfgen/                        PDF generator (Next.js — see below)
│   └── audits/                        23 audit skills
│
├── docs/                              Documentation (see docs/README.md for the index)
│   ├── ARCHITECTURE.md                ← READ THIS for the full system
│   ├── ARCHITECTURE-V3.md             ← Credential architecture spec
│   ├── VERIFICATION-GATE.md
│   ├── plans/                         Historical planning notes (IMPLEMENTATION-PLAN, GAP-ANALYSIS, …)
│   └── reference/                     Reference materials
│       └── oauth/                     Python source for OAuth reference
│
├── telegram-bot/                      Telegram bot runtimes (Bun/TypeScript)
│   ├── omega-tg-bot.ts                Command bot (control center, installed to ~/.omega/telegram-bot/)
│   └── inbox-bot.ts                   Deposit/inbox bot (operator file drop)
│
├── tools/pdfgen/                      PDF generator (TypeScript + Next.js + Playwright)
│   ├── components/templates/          Whitepaper / Audit / Marketing / Doc
│   ├── bin/pdfgen.ts                  CLI entrypoint
│   └── package.json
│
├── config/default.toml                Default OmegaOS config (template)
├── OMEGA.md                           Universal agent prompt (deployed to ~/.omega/)
├── README.md                          Project intro
├── Cargo.toml                         Rust workspace
└── install.sh                         Reproducible installer
```

## Runtime Data Layout (`~/.omega/`)

```
~/.omega/                              MASTER — what all LLMs read from
├── OMEGA.md                           Copied from repo at install time
├── config.toml                        User config (editable)
├── providers.toml                     Per-provider settings
├── telegram.toml                      Telegram bot config (gitignored, secret)
├── projects.json                      Compatibility project-registry projection
│
├── credentials/                       OmegaOS-owned credential copies
│   ├── claude.json                    ← ~/.claude/.credentials.json points here
│   ├── codex.json                     ↔ CODEX_HOME/auth.json (reconciled)
│   └── accounts/                      Saved account profiles
│   # Gemini/Antigravity OAuth stays in provider-native keyring storage
│
├── rules/                             The typed doctrine (.md, synced from repo on install)
├── agents/                            Shared and role-specific agent prompts
├── skills/
│   ├── pdfgen/                        PDF generator
│   └── audits/                        23 audits
│
├── state/                             Runtime state and projections
│   └── mission-engine-v3.sqlite3      authoritative mission ledger
├── logs/                              Session logs
└── audit/                             Audit results
```

## Languages

| Component | Language | Why |
|-----------|----------|-----|
| **Core library** (omega-core) | **Rust** | 100% — type safety, performance |
| **TUI** (omega-tui) | **Rust** | 100% — ratatui native |
| **CLI binary** (omega-cli) | **Rust** | 100% — clap + tokio |
| **Telegram bots** (`telegram-bot/`) | Bun + TypeScript | Bot API, topic routing, and file handling |
| **Install and operational scripts** | Shell | Bootstrap and host integration |
| **PDF generator** (tools/pdfgen/) | TypeScript + Next.js | Playwright requires Node.js; rendering needs full DOM/CSS |
| **OAuth helper** | Bash | One-line wrapper around `claude /login` for fallback |
| Reference docs | Python (read-only) | Just documentation of the VPS Python implementation we ported |

Python is used by selected tests, tools, and some OS payloads. Files under
`docs/reference/oauth/` are read-only reference material for the Rust OAuth
implementation; they are not the live credential service.

## Install Flow

```
$ ./install.sh
  Phase 1: Detect OS + arch
  Phase 2: Install Rust (rustup) if missing
  Phase 3: Select verified release binaries when current, or build pinned rmux
  Phase 4: Verify/install omega, falling back to a locked source build
  Phase 5: Setup ~/.omega/
           - Create credentials/, state/, logs/, accounts/
           - reconcile Claude credentials with freshness-aware omega-core code
           - leave Gemini/Antigravity native keyring credentials provider-owned
           - reconcile Codex through omega-core (CODEX_HOME-aware)
           - Copy OMEGA.md, agents/, rules/, skills/pdfgen/, skills/audits/
           - Run `omega rules export` and `omega sync`
  Phase 6: Shell integration
           - Add ~/.local/bin to PATH
           - Install shell completions (bash/zsh/fish)
           - Add alias: om="omega menu"
```

## Daily Usage

| Action | Command |
|--------|---------|
| Launch TUI | `omega` or `om` |
| Send work from Telegram | Send it in the Atlas or project topic; Atlas resolves and dispatches |
| List sessions | `/sessions` on Telegram or `omega list` |
| Switch global provider/model | `/model` buttons or `omega config activate claude opus` |
| Manage accounts | `/account` (button menu) |
| New project | `/projects` → [+ New project] |
| Generate PDF | `omega pdf --template=audit --send` |
| Update OmegaOS | `omega update` (or run `./install.sh` from an exact source checkout) |

## Verification

```bash
# Source is in the right place
ls ~/Station/SideBusiness/OmegaOS/crates/

# Binary is installed
which omega && omega --version

# Runtime data exists
ls ~/.omega/credentials/

# Symlinks are correct
ls -la ~/.claude/.credentials.json   # → ~/.omega/credentials/claude.json

# Build still works against the lockfile
cd ~/Station/SideBusiness/OmegaOS && cargo build --release --locked
```
