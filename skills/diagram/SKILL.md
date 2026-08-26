---
name: diagram
description: >
  Deterministic diagram / visual-explainer generator — turns intent into precise, editable, brand-themeable diagram-as-code (Mermaid or D2) and renders it to SVG + PNG via CLI. Use for flowcharts, architecture diagrams, sequence diagrams, state machines, ER/data models, agent/system flows, mindmaps, timelines/Gantt, and quadrant/comparison charts — anything that must be technically correct and editable, not photoreal. Use when the user says "/omg-diagram", "/diagram", "draw a diagram", "flowchart", "architecture diagram", "sequence diagram", "state diagram", "ER diagram", "mindmap", "diagram this", or in French "schéma", "diagramme", "organigramme", "fais-moi un schéma". NOT for quote-cards/posters/photoreal art (route to image-gen) or animated data-viz (route to /hyperframes).
triggers: ["omg-diagram","diagram","draw a diagram","flowchart","architecture diagram","sequence diagram","state diagram","ER diagram","mindmap","diagram this","schéma","diagramme","organigramme"]
allowed-tools: ["Read","Write","Edit","Bash","Glob","Grep"]
domain: design
read_only: false
argument-hint: "<what to diagram> (e.g. 'architecture of the auth flow')"
source: OmegaOS-native
license: MIT (OmegaOS)
---

> **OmegaOS skill** — native. Triggers `/omg-diagram` (and `/diagram`). Deterministic diagram-as-code → SVG/PNG via CLI (R-CLI: no MCP, no paid API). Heavy renderers (`d2`, mermaid Chromium) install lazily at runtime to user dirs, never via `install.sh`, never sudo (R-ENV) — same precedent as higgsfield / browser-use.

# Diagram

You are a diagramming engineer. Your job: produce a **technically correct, editable, brand-consistent figure** from the user's intent — never a vague picture. The deliverable is always TWO things: the rendered **asset** (`.svg` + `.png`) AND the **source** (`.mmd` or `.d2`) so the user can edit and re-render. Diagram-as-code, not image generation.

## When to use this (and when NOT)

**Use `/diagram` when** the figure must be:
- **Precise** — boxes/edges/labels map to a real system, not a vibe.
- **Editable** — the source is text the user (or another agent) can tweak and re-render.
- **Technical** — architecture, sequence, state, ER/schema, agent/data flow, mindmap, timeline, comparison/quadrant.
- **On-brand** — themed from the project palette, consistent across a deck.

**Do NOT use this — route elsewhere:**
- Quote-cards, posters, hero/photoreal art, illustrative scenes, "make it look beautiful" → **image-gen** (`/ad_designer`, higgsfield `/omg-higgsfield-generate`, `/imagegen-frontend-*`). AI image-gen CANNOT guarantee a correct architecture/sequence diagram — that's exactly the gap this skill fills, and the inverse is also true: don't ask this skill for painterly art.
- Animated / interactive / live data-driven visualizations → **`/hyperframes`** (motion, charts that move, video).
- Pure UI mockups / screens → the design skills (`/ui-ux-pro-max`, `/stitch-design-taste`).

When in doubt: *correct + editable + static = here; pretty + photoreal = image-gen; moving = hyperframes.*

## Decision table — intent → diagram type → syntax

Pick the type FIRST from the user's intent, then write that syntax. Default to **Mermaid** (zero-install via `npx`); reach for **D2** when you need richer styling, SQL-style tables, nested infra containers, or cleaner large-graph layouts.

| Intent / ask | Diagram type | Syntax | Engine |
|---|---|---|---|
| Flowchart, process, decision tree, agent flow, "how it works" | flowchart | Mermaid `flowchart` / `graph TD\|LR` | Mermaid |
| System / software / cloud **architecture** (services, queues, DBs) | architecture | Mermaid `flowchart` with subgraphs **or** D2 containers | Mermaid or **D2** |
| Sequence / interaction / API call order / request lifecycle | sequence | Mermaid `sequenceDiagram` | Mermaid |
| State machine, lifecycle, status transitions | state | Mermaid `stateDiagram-v2` | Mermaid |
| Database schema, data model, entities & relations | ER / schema | Mermaid `erDiagram` **or** D2 `sql_table` | Mermaid or **D2** |
| Mind map, brainstorm, topic tree | mindmap | Mermaid `mindmap` | Mermaid |
| Timeline, roadmap, project schedule | timeline / Gantt | Mermaid `gantt` (or `timeline`) | Mermaid |
| Comparison, prioritization, 2×2, positioning | quadrant | Mermaid `quadrantChart` | Mermaid |
| Complex styled infra, SQL-ish tables, deeply nested containers, big graphs that Mermaid lays out poorly | architecture / schema | **D2** (shapes, containers, `sql_table`, `classes`, `vars`) | **D2** |

See `references/syntax.md` for one minimal CORRECT example of each.

## Authoring workflow

1. **Classify** the intent with the table above → choose type + syntax (Mermaid default; D2 for the heavy/styled cases).
2. **Resolve brand theme** (next section) — find a palette or fall back to the clean neutral default.
3. **Write CORRECT diagram-as-code** to a file. Mermaid → `name.mmd`; D2 → `name.d2`. Put the theme/init block at the top. Get the syntax right the first time — consult `references/syntax.md`; do not guess node/edge syntax.
   - Save sources under the project's `agentic/` (per the repo convention) or a path the user names, e.g. `agentic/diagrams/auth-flow.d2`. For ad-hoc/non-repo work use `/tmp`.
4. **Render** with the shipped script:
   ```bash
   bash ~/.omega/skills/diagram/render.sh <input.(mmd|d2)> [out_basename]
   ```
   It detects syntax by extension, renders SVG, and (since `rsvg-convert` is present on the box) also emits PNG. It prints the absolute `SVG:` / `PNG:` / `SRC:` paths.
5. **Verify** the asset exists and is sane (SVG contains `<svg`, PNG starts with the PNG magic bytes). If the render errored, READ the error — it's almost always a syntax slip in the source; fix and re-render (L1: runtime is the only truth — a diagram you didn't render is not done).
6. **Deliver BOTH**: present the rendered asset (path) AND the source path, and offer to tweak. Never hand over only an image with no editable source.

## Brand theming

A themed figure beats a default one. Before writing, look for a palette:

- Read **`.agents/brand.json`** if it exists (the OmegaOS brand-token convention, mirrors `.agents/product-marketing.md`). Also check `.agents/brand.md`, `brandkit.json`, or a `tailwind.config.*` `theme.colors` block. Pull: primary, secondary/accent, background, text, and a border/muted shade.
- **No palette found → use the clean neutral default**: ink `#1A1A1A` on background `#FFFFFF`, primary `#2563EB`, accent `#6C5CE7`, muted border `#E2E8F0`, font `Inter, system-ui, sans-serif`. Tasteful, legible, slide-ready — never garish.

**Mermaid** — put a `themeVariables` init block on line 1 of the `.mmd`:
```
%%{init: {'theme':'base','themeVariables':{
  'primaryColor':'#2563EB','primaryTextColor':'#FFFFFF','primaryBorderColor':'#1D4ED8',
  'lineColor':'#94A3B8','secondaryColor':'#6C5CE7','tertiaryColor':'#F1F5F9',
  'fontFamily':'Inter, system-ui, sans-serif','background':'#FFFFFF'}}}%%
flowchart LR
  A[Start] --> B{Decision} --> C[Done]
```

**D2** — declare a `vars` block + reusable `classes` and assign them with `class:`:
```d2
vars: {
  primary: "#2563EB"
  accent:  "#6C5CE7"
  ink:     "#1A1A1A"
  muted:   "#E2E8F0"
}
classes: {
  primary: { style: { fill: "${primary}"; font-color: "#FFFFFF"; stroke: "${primary}"; border-radius: 8 } }
  accent:  { style: { fill: "${accent}";  font-color: "#FFFFFF"; stroke: "${accent}";  border-radius: 8 } }
}
api: API { class: primary }
db:  DB  { shape: cylinder; class: accent }
api -> db: query
```

Keep ≤ 5 colors. Match the project's palette when one exists; otherwise the neutral default. `references/syntax.md` has copy-paste theme snippets for both.

## How STATION / marketing skills call this

This is the missing render layer for diagram-as-code across the catalog:

- **social-content / carousel-designer** — a carousel slide that's a process/architecture figure: call `/diagram` for the slide art (themed SVG/PNG), then place it in the carousel.
- **blog / longform** — those skills emit `[VISUAL: flowchart …]` / `[VISUAL: architecture …]` placeholders that nothing renders today. Resolve each placeholder by generating the matching diagram here and dropping the asset inline.
- **prd / planner / vision / architecture docs** — system/agent-flow and sequence figures for the doc.
- **Any agent** that needs a correct technical figure (not photoreal art) should shell out to `render.sh` rather than reaching for image-gen.

Caller contract: hand `/diagram` the **intent** ("sequence diagram of the checkout flow") and, if known, the brand palette path; it returns the asset paths + the editable source.

## Renderer & lazy-install note

`render.sh` is the single entry point. Renderers are **NOT** bundled and **NOT** in `install.sh` — they install lazily on first use to user dirs (R-ENV), the same precedent as higgsfield / browser-use:

- **D2** → installed to `$HOME/.local/bin/d2` via the official installer (`curl -fsSL https://d2lang.com/install.sh | sh -s -- --prefix "$HOME/.local"`). Single static Go binary, no runtime deps, no sudo.
- **Mermaid** → run on demand via `npx -y @mermaid-js/mermaid-cli` (cached under `~/.npm`); a puppeteer config passes `--no-sandbox` so it works headless. First run downloads a Chromium — slower and heavier than D2, so for a browserless box **prefer D2** when the figure allows.
- **PNG** → produced from the SVG with `rsvg-convert` (librsvg) when present; otherwise SVG-only with a note.

If no renderer can be obtained, `render.sh` does **not** fail silently — it prints the source path plus exact install instructions and exits non-zero (L0/L1). The source is always written first, so the diagram-as-code survives even if rendering is unavailable.
