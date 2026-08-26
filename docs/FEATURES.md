# OmegaOS — Feature Catalog

Everything OmegaOS does and how to reach it. This is the "nothing hidden" map: each subsystem,
what it's for, the command / trigger, and where it lives. Deeper references are linked per section.
The live truth is always `omega --help`, `omega rules list`, and `omega-skills`.

> Start here, then: [ARCHITECTURE.md](ARCHITECTURE.md) (how it's built) · [MAP.md](MAP.md) (where
> files live) · [GETTING-STARTED.md](GETTING-STARTED.md) (post-install).

---

## 1. Orchestration — Oracles & Workers

The 4-level model: **Human (Telegram/CLI) → Atlas Telegram service → Oracle
(1 per project, strategic) → Workers (ephemeral, parallel, file-scoped)**. An
oracle decomposes a mission and dispatches workers; it never edits code itself.
Workers can run in isolated worktrees and submit completion candidates to the
mission gate.

| Do this | Command |
|---|---|
| Dispatch a mission to a project oracle | `omega dispatch <Project> "<mission>"` |
| Open a project as a clean **oracle** session (prompt preloaded) | TUI → Projects → open (every provider now seeds the oracle identity) |
| Spawn a worker under the current oracle | `omega spawn-worker …` (worktree isolation) |
| Watch / audit a running session | `omega monitor` · `/monitor` |
| Resurrect a crashed oracle | `omega resurrect <oracle>` |
| See a mission timeline | `omega timeline <oracle>` |
| Team / parallel fan-out | `omega team`, or the Workflow primitive (`/dynamic`) |

Doctrine: R-ORCH (workflow-first), R-MASTER (dispatch-only orchestration), R-LOOP (bounded retries),
R-VERIFY (adversarial verification). Reference: [ARCHITECTURE.md](ARCHITECTURE.md).

## 2. Telegram

The phone interface. A project bot shows a live progress card; alerts funnel to a dedicated topic;
an allow-list restricts who can drive the box.

| Do this | How |
|---|---|
| Set up the bot | `omega telegram setup <YOUR_ID> --user-id <YOUR_ID>` (token via `OMEGA_TG_TOKEN`) |
| Start / wire the bot | `omega-tg-up` |
| Send a message to the operator | `omega send <session> <text>` / the alert funnel |
| Project topics | group + Topics ON + bot admin → `/setupgroup` → `/sync` |

Doctrine: R-TGSEC (allow-list), R-TGDELIVER (push every deliverable link/file), the routing doctrine
(Atlas topic = discussion, Alerts topic = alerts, project topic = reports).

## 3. Rules & Laws (the doctrine engine)

**7 Laws (L0–L6)** inviolable + **52 Rules (R-*)** operational, compiled in
`crates/omega-core/src/rules.rs` (the SSOT) and injected into every dispatched agent via
`agent_context_block(scope)`. `omega rules list` is the live inventory.

| Do this | Command |
|---|---|
| List every law + rule | `omega rules list` |
| See the exact context an oracle/worker/master receives | `omega rules context <oracle\|worker\|master>` |
| Full doctrine, rendered | [RULES.md](../RULES.md) |
| Add a rule | a `Rule{…}` in `rules.rs` (scopes, domains) + a matching `rules/<ID>-*.md` (parity test) |

Key recent rules: **R-SKILL-ATLAS** (discover skills via the atlas + RAG), **R-PRODUCT** (work
product through the Product Development System), R-DESIGN (design router), R-AUDIT (invoke the real
audit skill). A rule's first sentence is what survives injection — front-load the actionable core.

## 4. Skills — Atlas, RAG, Power-Ups

The shipped skill catalog plus user-installed libraries is discoverable through
one atlas. Catalog size changes as skills are added, so validate the live tree
instead of relying on a copied count.

| Do this | Command |
|---|---|
| List / search native skills with their `/command` | `omega-skills` · `omega-skills <term>` |
| **Semantic** search by meaning (native + library) | `omega-skills --rag "<need>"` |
| Search the Power-Up library | `omega-skills --powerups <term>` |
| The served, searchable catalog | `omega-skills --html` → the tailnet atlas page |
| Invoke a skill | type its `/command`, or ask by name (the Skill tool) |
| Validate the repository catalog | `omega skills validate --root skills` |

Doctrine: R-SKILL-ATLAS. Reference: [skill-atlas.md](skill-atlas.md),
[third-party-skills.md](third-party-skills.md). The Power-Up library (907 purchased skills) is
routed by the `powerup-library` / `powerups` skill, not loaded into the active namespace.

## 5. Design

| Need | Use |
|---|---|
| A live, viewable design engine (prototypes/dashboards/decks/images/video), redesign a project | **Open Design** — `omega-design open` → [open-design.md](open-design.md) |
| Route a design request to the right skill | R-DESIGN (the design router) |
| Premium/agency UI, brutalist, minimalist, motion, image-to-code | the design-intelligence skills (`/high-end-visual-design`, `/motion`, …) |
| A design system as a doc / tokens | `/design-system`, `theme-factory`, `stitch-design-taste` |
| Generate images / video / voiceover, identity-consistent | `/higgsfield-generate` + `/higgsfield-soul-id` (R-VISUAL-ID) |
| A scored, forensic UI/UX audit | `/uiuxaudit` (R-AUDIT) |

## 6. Product Development System

The OmegaOS way to work a feature: never idea→build. Chain: Outcome → Opportunity → Idea → Feature
(Discovery → Prioritization → Specification) → Workflow → Build → Measure. 7 sub-systems.

- Skill: `product-development-system` (auto-surfaces on feature/product work; RAG-indexed).
- Doctrine: **R-PRODUCT** (injected into every oracle/worker). Objects persist under a project's
  `agentic/product/` (vision/ ideas/ opportunities/ features/ workflows/).

## 7. Marketing machine

Produce → publish → engage, all gated.

| Layer | Use |
|---|---|
| Strategy / GTM / content / ads | the `/omg-*` marketing suite (R-MARKETING order) |
| Publish (organic + ads) | `omega-zernio` (R-ZERNIO — the single publishing funnel) |
| Inbound DM/comment automation | ZernFlow (R-ZERNFLOW) |
| Visual creatives, viewable | Open Design (§5) + higgsfield (R-VISUAL-ID) |

Reference: [marketing-machine-ssot.md](marketing-machine-ssot.md).

## 8. Audits & Council

- Forensic scored audits: `/uiuxaudit`, `/codeaudit`, `/secaudit`, `/a11yaudit`, `/perfaudit`, … (R-AUDIT).
- Multi-model decision: `/council` (Opus 5 + Sonnet + Haiku + Fable, blind peer-review) for
  high-stakes / irreversible / contested calls (R-COUNCIL).

## 9. Reports & artifacts (the Tailscale surface)

Deliverable reports default to a **local self-hosted artifact** served tailnet-only
(`https://station.tail64d114.ts.net:8443/`), plus an HTML twin (R-ARTIFACT / R-HTML). PDFs via
`omega pdf` (R-PDF). Open Design's own view sits on `:7456`.

## 10. Self-improvement & upkeep

| Feature | What |
|---|---|
| Daily auto-update | `omega update` (manual) / cron 03:30 — git pull main + rebuild (`auto_update=apply`) |
| Absorb the Claude Code changelog | `/changelog-adopt` (armed = auto-adopt vetted upgrades) |
| Ecosystem radar | `/ecosystem-watch` |
| X growth (bounded, gated) | `/growth-engine` |
| Backup / rebuild a box | `omega backup` → [RESET-RECOVERY.md](RESET-RECOVERY.md) |
| Health check | `omega doctor` |

## The CLI at a glance

`omega` subcommands: `dispatch new new-project spawn-worker team orchestrate` (work) ·
`monitor stream timeline patrol resurrect gate` (observe) · `rules skills agents config doctor
status list` (inspect) · `telegram send inbox` (comms) · `update backup sync ship pdf provision`
(ops). Helpers on PATH: `omega-skills`, `omega-design`, `omega-zernio`, `omega-mem`, `omega-duo`,
`omega-tg-up`, `omega-git-merge`, `omega-open`. Full list: `omega --help`.
