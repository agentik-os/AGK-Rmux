# OS — the AgentikOS operative-systems suite

This directory holds the AgentikOS product line of operative systems (OS) shipped
with OmegaOS. Each subdirectory is ONE operative system. The suite is surfaced in
the TUI under the **OS** tab (`omega menu`) and installed to `~/.omega/os/` by
`install.sh`.

## The suite (in order)

The BUILD CHAIN is its own group in the TUI, in pipeline order, then the
personal OSes.

| # | OS | Slug | Focus | Status |
|---|----|------|-------|--------|
| 01 | Ideation OS | `ideation-os` | Brainstorm {OS} v3: imagination + decision council | **integrated** |
| 02 | Researcher OS | `researcher-os` | Market Research {OS}: evidence + validation compiler, bounded decisions | **integrated** |
| 03 | Blueprint OS | `blueprint-os` | The product-definition compiler (v3): 38 sections, stable IDs, 20 gates, frozen handoff | **integrated** |
| 04 | Designer OS (UX/UI) | `designer-os` | Design {OS}: challenge the blueprint into flows, screens, states | **integrated** |
| 05 | Stepper OS | `stepper-os` | Step-by-step execution of a blueprint | **integrated** |
| 06 | Builder OS | `builder-os` | The implementation runtime: the Stepper roadmap executed into release-ready code | **integrated** |

Personal OSes:

| OS | Slug | Focus | Status |
|----|------|-------|--------|
| Mindset OS | `mindset-os` | Jim Rohn identity/wellbeing/wealth coaching OS | **integrated** |
| Habits OS | `habits-os` | Habit Tracker {OS}: conversation-first habit system | **integrated** |
| Execution OS | `execution-os` | Execution {OS} v2: LLM-first personal delivery loop | **integrated** |
| Storytelling OS | `storytelling-os` | Storyteller {OS}: coach + shape truthful stories, a story bank | **integrated** |
| Alignment OS | `alignment-os` | Alignment Coach {OS}: wisdom + decision second brain | **integrated** |
| Journal OS | `journal-os` | Journal {OS}: MIRROR, the nightly journal that turns the day into evidence | **integrated** |
| Identity Shift OS | `identity-shift-os` | Identity Shift {OS}: declared identity into daily evidence, contradictions surfaced as data | **integrated** |
| Intuitive OS | `intuitive-os` | Intuitive {OS}: train intuition as a measurable skill, forecasting + calibration | **integrated** |
| Seductive OS | `seductive-os` | Seductive {OS}: personal magnetism — presence, conversation, style, romantic confidence, consent-first | **integrated** |

**Systems & AI:**

| OS | Slug | Focus | Status |
|----|------|-------|--------|
| AI Logic OS | `ai-logic-os` | Workflow optimizer + agentic-system challenger | **integrated** |
| Books OS | `books-os` | Your library as an OS: reading, retention, living knowledge | **integrated** |

Registry source of truth: `crates/omega-core/src/os_products.rs`
(`OsProduct::all()`). The TUI tab, statuses and paths all derive from it -
add or reorder an OS THERE, never in the UI code. The full integration
playbook (anatomy of an OS, the surfaces convention, the add/complete
processes) is `docs/OS-SUITE.md`.

## Master agent + Telegram bot

Every OS carries a `MASTER.md` - its MASTER AGENT system prompt. The TUI's
Enter opens a Claude session running that agent, and `T` (or
`omega-os-bot <slug> [token]`) links a dedicated Telegram bot whose brain is
the SAME master agent (agent-bots.json kind `persona`, ledger under the OS
folder). One brain, every surface.

## Integration pipeline (how an OS lands here)

1. The operator drops the OS payload (zip) in the Deposit box
   (`~/Deposit`, via the Telegram DEPOSIT bot).
2. Unpack it into `OS/<slug>/` (this repo), next to the placeholder README.
3. Document how it runs in `OS/<slug>/README.md` (entrypoint, deps, config).
4. Keep `install.sh` parity (Law 0): the `OS/` payload is copied to
   `~/.omega/os/` on install - a fresh clone + install must reproduce it.
5. Commit + push.

An OS whose directory only contains its placeholder README is shown as
`awaiting drop` in the TUI; anything more marks it `integrated`.

Secrets never live here (R-ENV): keys go to `~/.omega/secrets/`, the payload
references them by name.
