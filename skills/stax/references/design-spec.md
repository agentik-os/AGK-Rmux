# Stax Design Spec — the pixel-level transformation contract

This is the authoritative UI/UX spec of the Stax panel system. Every value below is the
one shipped in `frameword/apps/crm-specimen/src/styles.css` + `tokens.css`. During a
migration, **every visual element of the old app is converted against this contract** —
icons, cards, badges, buttons, tables, spacing, type, color, states. Nothing is restyled
"by eye": each element maps to a row of the element matrix and each row lands on the
values in this file.

## 1 · Panel anatomy — interior margins are law

```
┌─ .panel ──────────────────────────────── radius 14 · 1px border · shadow-2xs ─┐
│ .panel-bar     h 44 · padding 0 10px 0 16px · gap 10 · 1px bottom border      │
│   .eyebrow     mono var(--fz-mono) · tracking 0.14em · uppercase · muted      │
│ .panel-body    padding 18px 18px 16px · flex column · scrolls (thin bar)      │
│   .panel-title serif var(--fz-title 27) · lh 1.08 · tracking -0.02em         │
│                margin 4px 0 8px                                               │
│   .panel-sub   0.96 × fz-body · muted · margin 0 0 18px · max 54ch           │
│   content …                                                                   │
│ .panel-foot    padding 7px 14px  · gap 8 · 1px top border · bg secondary      │
└───────────────────────────────────────────────────────────────────────────────┘
```

- Panel widths come from the registry only: **S 380 · M 480 · L 640 · XL 800 · XXL fluid**. XXL sizes to EXACTLY the
  stage's visible width (`calc(100% - stage-pad)`) so an aligned XXL panel
  breathes the same gutter on both sides; a size change re-scrolls the stage to
  the leaf; double-clicking any panel bar aligns that panel at the pad. Pinned
  references default to S but every size (XXL included) applies in pin mode.
  Never a hardcoded width on a panel.
- Stage: gap `var(--stage-gap, 14px)`, gutter `var(--stage-pad, 18px)` on BOTH sides
  (left padding + end spacer). A flexing panel (canvas/kanban) keeps the right gutter.
- Inside the body, blocks stack with these exact rhythms:
  - `.card` (doc block) — **FLAT**: no border, no bg, radius 0; hairline TOP, padding
    **14px 0 15px**; its `.lab` mono label gets margin-bottom 8. Borders belong to
    FLOATING surfaces (menus, popovers, pickers), INPUTS and MEDIA — in-flow content
    is flat, structured by hairlines and rhythm.
  - `.stats` — no tiles: a vertical hairline BETWEEN stats (padding 18 both sides);
    the block is a GRID of equal tracks (`grid-auto-columns: minmax(0, 1fr)`, never
    flex) so the hairlines land at exact even divisions; margin-bottom 12.
  - **Vertical rhythm law** — hairline-carrying blocks stack EDGE TO EDGE (gap 0):
    the air around a hairline comes from the blocks' own paddings (rows 12,
    cards 14/15, sections 15/16), never from ad-hoc spacer divs or inline margins.
    Surfaced blocks (code, demos) carry their own margin-bottom 14-16; the head
    closes with margin-bottom 18.
  - `.drills` — hairline top on the list, hairline between rows; the list bleeds
    **12px into the gutter** (margin 0 -12px, row padding 12px) so hover breathes while
    text keeps the flat-block left edge. Separators are drawn as 12px-INSET pseudo
    hairlines (`.drills::before`, `.drill::after`), never as borders on the bled box:
    only the FILL bleeds, never a line. Hover = secondary fill + the row's own 1px
    hairline turns **accent** and EXPANDS to the fill's edges (a line narrower than
    its surface reads broken); the line above the hovered row expands too, framing
    the fill symmetrically. The LAST row has no hairline at rest (the dedup law
    gives it to whatever follows), so hover paints none there: inventing a line
    under the cursor doubles the next block's separator. Index/arrow accent,
    arrow slides 3px.
    Row meta tags are flat mono uppercase, never pills. A row that pairs a drill
    with a TRAILING action (pin, edit) draws the separator on the ROW wrapper, not
    the drill, so the line still reaches the shared right bound.
  - **Separator law** — ONE hairline between any two stacked blocks, never two:
    containers absorb their first child's top hairline; lists drop the last row's
    bottom hairline. And ONE alignment: every separator in a panel shares the EXACT
    left/right bounds of the head hairline (the content box); a surface that bleeds
    into the gutter draws its lines inset back to those bounds.
  - **Focus ring** — neutral (foreground mix), suppressed while hovered/active
    (Safari fires :focus-visible on click); keyboard keeps it. Never accent.
  - **Exit transitions** — an EXPLICIT close gesture (×, the compact back ‹,
    ctrl+X, Escape on the leaf, the bridge's close()) leaves a 150ms ghost of the panel (opacity + 0.98 scale, the panel's
    own ease; `prefers-reduced-motion` skips it). Engine-driven removals (space
    switches, promotion) stay instant: those are navigations, not departures.
  - **A11y baseline** — toasts announce via a `role="status"` aria-live region;
    ArrowUp/Down walk the drill rows of the list the focus is in; overlays
    (sheet, drawer) move focus inside on open and RETURN it to the opener on
    close; every icon-only control carries a title/aria-label.
  - `.anat-row` — hairline top, padding 12/0, mono accent label column 104px.
  - Popover labels `.pop-sub` — margin-bottom 6.
- The foot is the ONLY action zone, with a strict hierarchy: ONE primary CTA per
  foot (accent bg + accent border + `--accent-hover` on hover); SECONDARY actions
  are QUIET (hairline `--border`, ink-2 text, secondary fill on hover — never
  accent chrome); destructive = `--destructive` TEXT on transparent chrome, soft
  red fill on hover. Never object-state toggles (pin/unpin) in the foot; never
  metadata lines.
- AI actions are REGISTRY citizens: kind "ai" renders as the ✶ signature on an
  accent-SOFT chip (never accent-filled: the one-primary law counts fills); its
  run computes a REAL insight from the panel's data and hands it to the agent
  surface (stax:ai). The palette and the bridge list them like any action.
- The panel SEARCH lives in the FOOT, FAR LEFT: the quiet icon-only ⌕ is the
  foot's FIRST element, and its active input starts from the same left edge
  (passive and active states share the anchor). Clicking it swaps the foot
  content into a borderless search row (same 44px height, autofocus, a quiet
  ×; Escape restores the actions, never closes the panel). It
  live-filters the panel's lists; zero matches show a one-sentence empty state
  with the next action.

## 2 · Type & numbers

| Role | Font | Size | Rules |
|---|---|---|---|
| Display / panel titles | `--font-serif` (Newsreader) | `--fz-title` 27 | weight 400, tracking −0.02em |
| Body | `--font-sans` (Inter) | `--fz-body` 13.5 | derived sizes via `calc(var(--fz-body) * k)` |
| Labels / eyebrows / data | `--font-mono` (Geist Mono) | `--fz-mono` 10 | uppercase, tracking 0.08–0.14em |
| **Every numeric value** | `--font-mono` | context | `tabular-nums`, tracking −0.02em — **numbers are never serif** |

All sizes derive from the three `--fz-*` user preferences. A migrated element with a
pixel font-size that ignores them is a defect.

## 3 · Color — one accent, everything derived

- Base tokens only: `--background --card --secondary --border --rule-1 --foreground
  --muted-foreground --ink-2 --ink-4`.
- ONE accent (`--accent`, default Supreme red oklch(0.578 0.245 27)) with derived ramp:
  `--accent-soft` (pills), `--accent-hover` (hover — **mix in OKLCH, never oklab**:
  oklab browns reds), `--accent-2/3/4` (72/45/22% mixes) for multi-series data.
- Semantic green/amber exist ONLY for do/don't dots and notification dots. Charts,
  statuses, kanban dots, funnels: **accent ramp + neutral inks**.
- Text on accent = `--accent-foreground` (computed by luminance for custom accents).
- Every theme block must define every surface token (light AND dark — a token missing
  from one theme is a defect).

## 4 · Shape, depth, motion

- Radius scale: panels 14 · cards/stats 12 · drills/menus 10 · buttons 7-8 · inputs 7 ·
  pills 9999.
- Shadows: resting `--shadow-2xs`, floating menus `--shadow-md`, drawers `--shadow-lg`.
  The last panel carries the reading-order edge shadow; non-focused panel bars tint
  `--secondary`; pinned panels get `inset 0 2px 0 var(--accent)` on the bar.
- Motion: one ease `var(--ease)` (cubic-bezier(0.32,0.72,0,1)); panels 260ms in,
  menus 100-160ms fade/rise. No springs, no bounces.

## 5 · Icons

- Stroke icons only (Lucide-style): viewBox 24, **strokeWidth 1.6–1.7**, round caps and
  joins, `currentColor`. Sizes: topbar 14, toolbars 15, inline 12-13.
- No emoji in UI chrome — editorial glyphs only (§ ¶ ✶ → ⌄ ✓ ⋯).
- Every icon of the old app is inventoried and either mapped to a stroke equivalent or
  dropped with a decision-log entry.

## 6 · Controls — conversion table (old world → Stax)

| Old element | Stax target |
|---|---|
| Modal / dialog | Panel (size by content) opened to the right — parent stays |
| Tabs | Sibling drills, or in-panel sections (never a tab strip) |
| Dropdown `<select>` for a status-like choice | Segmented `d-btn sm` buttons (filled = active) |
| Native date/time input | Popover calendar (`.dp-pop`, Monday-first) + 30-min time list |
| Inline rename / boxed edit-in-place | `.inline-edit` — text-like, zero border, zero ring |
| Focus ring on inputs | ONE accent-tinted border; outline reserved for buttons (keyboard) |
| Toolbar of 10+ flat buttons | Grouped smart menus (trigger shows active state) |
| List page with a search box | Foot ⌕ toggle: the foot swaps into a borderless search row (same height); Escape restores the actions |
| Menu / dropdown items | Sans 500 rows; MONO uppercase group labels; selected = accent-soft text + right ✓ — never a filled slab; segments keep the soft fill |
| Record side peek (Notion-style) | Row hover reveals an `Open` chip on the title cell → the ENTITY SHEET: a fixed right drawer (agent-drawer language, 440px) with header (table eyebrow · icon actions ↗/⋯/×), serif editable title, PIPELINE PILLS derived from the first select field (click = set stage, live in the grid), underline facet tabs (Fields · Page · Activity · n) and a per-row note/task stream with a composer. `Open as panel` converts the peek into a thread panel |
| Toast / snackbar stacks | `.toast` mono pill, bottom-center |
| Hover affordances | Reveal on row/element hover (edit pencils, open arrows at opacity 0 → 1) |
| Data grid | `.dt` table — sticky mono headers, hairline `--rule-1` grid, borderless cell
  inputs (focus = 5% accent tint), typed cells (mono numbers right-aligned, accent select
  pills, accent checks), row opens as the NEXT panel (a page), never instead of it.
  Views carry their OWN settings via the active tab's menu (rename, duplicate,
  Cozy/Compact density per view, reset, delete); the ⋯ menu exports a real CSV
  (visible fields, filtered+sorted rows); the row menu inserts a row below; the
  foot's New row adds IN PLACE (reveals the row, never opens a panel). The foot ⌕
  drives the row quick-search; rows multi-select via quiet hover checkboxes
  (header = select-all-visible) and a selection shows a BULK BAR in the toolbar
  (count · Duplicate · Delete red-text · clear) |
| Free-form board / diagram | Canvas panel — WhitePaper nodes, 4-way handles, accent edges |
| Wizard | Chained drills with foot CTAs |
| Form/inspector card ("Element", "Details"…) | **The title IS the field**: the entity's name/subtitle edit in place as the panel's serif title block (`.fs-head`/`.fs-title`/`.fs-sub`, rhythm 6/12/18); every remaining group is a flat `.section` (mono label + hairline top) — form cards are FORBIDDEN |
| Global search | ⌘K palette |
| Close the focused panel | ctrl+X (inert while typing) + the bar × + Escape on the leaf |
| Undo / redo a workspace intent | mod+Z / shift+mod+Z (bounded stack in the provider; a FOCUSED canvas keeps its own board history) |
| Chat / AI helper | Full-height drawer (⌘J) |

## 7 · States — all six, for every element

Every migrated element ships all of: default · hover · focus/active · empty ·
loading/skeleton · error. Empty states are sentences with a next action, not blank
space. A migrated element missing a state keeps its matrix row open.

## 8 · The Shell: the chrome contract (topbar · sidebar · crumbbar)

A converted app ships THIS shell — every element below is part of the transformation,
inventoried in the element matrix and rebuilt to this anatomy. Nothing of the old
chrome (headers, navbars, user menus, settings pages, notification trays) survives
outside it.

**Topbar (h 52, solid card bg — NEVER backdrop-filter: it breaks fixed descendants):**
- sidebar toggle (bare icon 16) · dashboard NAV as dropdown triggers (icon 14 + label
  + caret; menu = numbered dd-items, one per space, footer hint) · spacer ·
- GO TO pill (⌘K) · utility icons 15 (system spaces: data, notes, tasks, canvas) ·
- notification bell (unread dot; dropdown: search, All/Unread segment, kind dots,
  mark-all-read foot) — the ONLY floating menus allowed are these chrome dropdowns.

**Sidebar (w 240, overflow hidden):**
- org/tenant switcher head (logo tile 15, name + tier, ⇅; menu = org rows + ⌘1-3) ·
- Quick open row (⌘K, filled) ·
- the SPACE area: dashboard's space list at rest; a DEDICATED space menu when one is
  open (‹ back arrow 26 with instant tooltip = dashboard name; head row = plain sans
  600 row, click focuses/reopens the main; collapse toggle always rendered, disabled
  when inert; numbered children 01…, one-level expansion ≤14, active thread
  highlighted) · compact adds a 4-icon dashboard switcher row ·
- SPACE KPIs block: when the active space's ROOT declares kpis, the sidebar
  mirrors them right above the pipeline block (same source of truth as the panel's
  .stats, never a second dataset): equal grid tracks, mono tabular values, mono
  uppercase labels (ellipsised), a full-width hairline below so the pipeline keeps
  its own separator. No kpis on the root → the block does not render ·
- usage/pipeline block (mono label, value, hairline track) ·
- account chip (avatar 28 + name + role/email, ⋯): menu = Profile · Settings ·
  Documentation · separator · Language accordion (inline rows, flag 18 SVGs, no
  side-flyout: the sidebar clips) · separator · Sign out. Avatar everywhere follows
  the profile store instantly.

**Crumbbar (h 34):**
- home glyph → thread crumbs (mono, focused = accent, click = navigateTo rewind) ·
  spacer · transient toasts (inline text, never floating pills over the UI) ·
  repo/GitHub link · theme toggle — both as bare 24px icons, menus open UPWARD.

**Shareable workspaces:** "Copy workspace link" carries the FULL state
(gzip+base64url in `#ws=…`, no backend); a receiver restores it at boot OR on
live hash paste, then the hash rewrites to the normal thread encoding.

**Devtools (sys panel):** the workspace inspected by itself — live state JSON,
the intent history (each entry = the state an intent replaced), Jump =
time travel through ws.restore (undoable). The keyboard map ships as the "?"
overlay (palette chrome, Escape closes it first).

**The design laws are CI:** `stax-migrate verify --url --themes light,dark`
runs on every push/PR (build → preview → scan, BOTH themes): L-ALIGN,
L-RHYTHM, L-FOOT, L-FLOW fail the build. `stax-migrate doctor` prints the
adoption health report (contract, pending upgrade units, hardcoded-value
drift) with its prescription.

**The copilot bridge:** the shell exposes `window.stax` (serializable state,
intents, the action registry) so agents DRIVE the workspace instead of faking
clicks; the agent drawer's /commands speak it. Contract: agents.md M8.

**Panels-only surfaces:** settings = a sys PANEL (appearance, fonts, accent, zoom,
shortcuts) — never a page; profile = entity panel (fs-head name/role); language and
theme are DEVICE-LOCAL prefs (localStorage), never navigation state.

**Responsive (three tiers):** ≥900 the sidebar is DOCKED (240px beside the stage);
640-900 it AUTO-CLOSES and reopens as an OVERLAY with backdrop (a docked 240px
sidebar starves the stage: a 640 L panel never fits), any nav click closes it;
≤640 is PushHost: one card + back, ref chips with remove ×, the overlay sidebar
carries the dashboard switcher; ≤760 hides the topbar nav (the sidebar switcher
takes over). Fixed drawers (agent 380, sheet 440, palette 560) all clamp to 88-92%
of the frame; stat labels ellipsise instead of clipping.

**The shell at a glance:**

```
┌──────────────────── topbar h52 ────────────────────┐
│ ☰ · nav dropdowns · spacer · GO TO ⌘K · utils · 🔔 │
├─────────┬──────────────────────────────────────────┤
│ sidebar │  STAGE (scrolls horizontally)            │
│  org ⇅  │  ┌─ panel ─┐ ┌─ panel ─┐ ┌─ ref S ─┐    │
│  ⌘K row │  │ bar h44 │ │         │ │ (rail)  │    │
│  spaces │  │ body    │ │         │ │         │    │
│  ────── │  │ foot    │ │         │ │         │    │
│  KPIs*  │  └─────────┘ └─────────┘ └─────────┘    │
│  ────── │   *KPIs: only when the space's root      │
│  pipe   │    declares them (mirrors .stats)        │
│  avatar │                                          │
├─────────┴──────────────────────────────────────────┤
│ crumbbar h34: home → crumbs · toasts · gh · theme  │
└────────────────────────────────────────────────────┘
```
