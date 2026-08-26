---
name: open-design
description: >
  Drive Open Design (self-hosted nexu-io/open-design) to build design systems and generate brand
  visuals from OmegaOS. It turns an agent into a design engine: prototypes, dashboards, landing
  pages, decks, images and video, rendered from a brand contract (DESIGN.md) using 150+ built-in
  design systems (apple, airbnb, ant, agentic, …), 100+ functional skills and render templates.
  Results are viewable in the browser over Tailscale (the OmegaOS server surface), so the operator
  SEES the build, not just files. Use for: building or applying a design system, a brand kit, a UI
  prototype, a pitch deck, marketing creatives for the marketing machine, product screens, or any
  "make it look designed" artifact that should be viewable at a link. Triggers (EN): "design system",
  "brand kit", "build a prototype", "landing page design", "pitch deck", "generate a dashboard",
  "marketing creative", "open design", "make a deck/prototype I can view". Triggers (FR): "design
  system", "kit de marque", "construire un prototype", "maquette", "page de vente", "deck",
  "générer un dashboard", "créa marketing", "open design", "un design que je peux voir en lien".
  NOT for generating a single photoreal image/video (that is higgsfield-generate) or a code-only
  theme (theme-factory); Open Design is the multi-artifact design workspace with a live view.
allowed-tools: ["Bash", "Read", "Write", "Grep", "Glob"]
metadata:
  source: omegaos
  version: "1.0"
---

# Open Design — self-hosted design engine

Open Design runs as a local Docker daemon on the OmegaOS server and is served **tailnet-only** so
the operator can open the build in a browser. It is the OmegaOS surface for design systems and
brand visuals, including the marketing machine's creatives.

## The one command you start from

```bash
omega-design status        # is it up? prints the view URL
omega-design url           # the tailnet link to hand the operator
```

If it is not running: `omega-design up && omega-design serve`. The view URL is
`https://<tailnet-host>:7456` (e.g. `https://station.tail64d114.ts.net:7456`).

## How it works (the model)

- **Design systems** (`design-systems/*/DESIGN.md`): a brand contract — tokens, type, color, voice.
  150+ ship built in (`omega-design systems`). Pick one, or write a project's own `DESIGN.md`.
- **Skills + templates**: functional skills (`omega-design skills`) and render templates
  (`omega-design templates`) produce decks, prototypes, dashboards, images and video against the
  chosen design system.
- **Daemon API**: everything is `/api/*` on the daemon (`omega-design api /api/health`). The web UI
  and the CLI both call the same endpoints — the HTTP layer is the source of truth.

## Drive it from a chat (MCP)

The Open Design MCP is wired into Claude and Codex (`docker exec` into the daemon). Tools:
`list_projects`, `create_project`, `list_skills` (design research), `start_run` (commission a
design), `get_run`. The full capability list (162 skills by mode + 151 design systems) is in
`references/od-catalog.md`. Use these to research a design and generate a project straight from chat.

## How an OmegaOS session drives it

1. **Confirm it's live**: `omega-design status` (start with `omega-design up && omega-design serve`).
2. **Pick / write the brand contract**: choose a built-in system (`omega-design systems`) or write a
   `DESIGN.md` for the project under `agentic/design/` (tokens, type scale, color, tone).
3. **Generate** via the daemon API (`omega-design api <path> -X POST -d '…'`) or the web UI; artifacts
   land in the daemon's project and render in the browser.
4. **Hand back the link**: give the operator `omega-design url` so they SEE it (per R-TGDELIVER, push
   the link to Telegram too when it's a deliverable).

## Marketing machine

For OmegaOS marketing work (R-MARKETING / R-VISUAL-ID), use Open Design to build the visual system
and viewable creatives (landing pages, decks, ad mocks) that the operator reviews at the link before
anything is published through Zernio (R-ZERNIO). Open Design BUILDS + shows; Zernio DISTRIBUTES.

## Boundaries

- External dependency: the Docker image `ghcr.io/nexu-io/od` + a 503M clone at
  `~/.omega/repos/open-design` (pinned in `tools/open-design/README.md`). Opt-in install, not
  auto-run by OmegaOS install.sh (same boundary as ZernFlow / higgsfield).
- Token: `~/.omega/secrets/open-design.env` (never the repo). Served tailnet-only (no Funnel);
  the tailnet is the trusted auth layer, so daemon token-auth is disabled behind it.
- Live generation is not runtime-verifiable without the running daemon + the operator opening the link.
