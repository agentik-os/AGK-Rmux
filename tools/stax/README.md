# stax — OmegaOS tool (the dashboard vision structure)

**Stax** is the panels-inside-panels UX framework: the whole interface is a horizontal
rail of in-page panels (Miller columns, evolved). Click anything with depth and a panel
opens to its right; the parent stays. **No pages, no modals, no tabs** — one mechanic
(open-right), one action zone (the footer), one way back (close / breadcrumb). The entire
UI derives from a single serializable, URL-synced workspace state.

- **Upstream (SSOT):** https://github.com/agentik-os/stax
- **Tracking:** **latest `main`** — NOT a frozen pin. This is the deliberate opposite of
  the pinned `tools/zernflow` / higgsfield installers: Stax is our evolving dashboard
  vision, so OmegaOS follows main and auto-syncs daily.
- **License:** MIT (Agentik OS).

## What ships where

| Piece | Location | Role |
|---|---|---|
| Framework checkout | `~/.omega/repos/stax` | live clone of `main`; the `/stax` skill reads it |
| Installer | `tools/stax/install-stax.sh` | clone/refresh the checkout (git only, no npm) |
| Sync script | `~/.omega/skills/stax/scripts/stax-sync.sh` | daily ff-only pull + Telegram notify on change |
| Converter skill | `skills/stax/` → `~/.omega/skills/stax/` | `/stax` — convert any project to the panel grammar |
| Scaffold | `~/.omega/skills/stax/scripts/stax-scaffold.sh` | vendor engine + shell + tokens into a target app |

The framework itself is a monorepo (`frameword/`): `packages/panels-core` (pure-TS
serializable state machine + laws test-kit), `packages/panels-react` (provider, registry,
URL sync), and `apps/crm-specimen` (the full reference app). Concept + prompts live in
`PANEL-LOGIC.md`, `CONCEPT-BRIEF.md`, `PROMPT-KIT.md`.

## Install (cheap — run by install.sh)

```sh
bash tools/stax/install-stax.sh          # clone/refresh ~/.omega/repos/stax
```

Unlike zernflow/higgsfield this is NOT heavy (no app build), so `install.sh` runs it.
The specimen app build (`cd ~/.omega/repos/stax/frameword && bun install`) stays opt-in.

## Auto-update (R-STAX)

The daily cron `OMEGA-CRON-STAX-SYNC-v1` runs `stax-sync.sh`, which fast-forwards every
local Stax checkout (`~/.omega/repos/stax` + the operator's `Station/SideBusiness/Stax`
dev checkout) to `origin/main` and pings Telegram only when main moved. Fast-forward
only — a checkout with local commits is reported and skipped, never clobbered
(R-DESTRUCT / R-SYNC). Log: `~/.omega/logs/stax-sync.cron.log`. On demand:

```sh
bash ~/.omega/skills/stax/scripts/stax-sync.sh
```

## Convert a project

```sh
bash ~/.omega/skills/stax/scripts/stax-scaffold.sh <target-project-dir>
```

Then follow `~/.omega/skills/stax/references/conversion-playbook.md` (Phases 0–5). The
`/stax` skill drives the whole pipeline: fit → blueprint → scaffold → port → theme →
verify. Governed by the R-DESIGN router (a "convert to Stax / panelize" request routes here).
