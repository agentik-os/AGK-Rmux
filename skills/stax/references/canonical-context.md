# Stax — canonical context & the laws

Paste the **canonical context** block once at the start of any AI session doing a Stax
conversion, then use the phase prompts from `conversion-playbook.md`. Source of truth:
`~/.omega/repos/stax/{PANEL-LOGIC.md, CONCEPT-BRIEF.md, PROMPT-KIT.md}` — read them live.

## The premise

An app is navigation over a tree/graph of content. Pages, tabs and modals **destroy
context**: they swap the whole view and force a single focus. Stax keeps the trail
alive — the viewport is a horizontal rail of columns; navigating **pushes a panel to the
right**; you never leave, the parents stay visible. **One mechanic (open-right). One
action zone (footer). One way back (close / breadcrumb).**

## The 5 laws (short form — the product)

These are enforced in the reducer, never per screen:

1. **Opening a Space replaces the active thread** — the stack is *one* train of thought;
   two lineages never coexist. Pinned references ride across Spaces.
2. **One action zone per panel — the footer.** Never floating buttons. Read-only panels
   have no footer.
3. **Parents stay visible; depth reads left → right; ⌘K goes anywhere.** The root is
   anchored fixed-left; closing it ends the thread.
4. **State is a serializable list** — everything (URL, breadcrumb, agent context,
   persistence) *derives* from it. `encode(state) ↔ decode(string)` round-trips losslessly.
5. **Tokens only** — change the accent and the whole system follows. Colours, type, and
   specific screens are downstream and swappable; the logic never changes.

## The v2 model (what the shipped engine actually implements)

The engine evolved from a flat stack (PANEL-LOGIC §1–3, kept as pedagogy) to **semantic
ancestry** (CONCEPT-BRIEF). What ships in `panels-core`:

- Ancestry is a **tree of parent links** (`parentInstanceId`), never visual order.
- **ContextPath is DERIVED** from parent links (root → leaf) — it drives the breadcrumb,
  close behavior, and the URL. It is the only URL-shareable thing.
- A panel is a `preview` (transient — the next open replaces it) until **pinned**
  (`retained`). A retained panel orphaned by a branch change **detaches** into the
  **ReferenceRail** — a pin outlives its Space.
- **Context-scoped identity:** opening the same target from the *same* parent reveals &
  focuses it (never duplicates); the same target under a *different* parent may create a
  distinct attached instance. Detached references dedup by canonical target within a Space.
- Navigation state is **JSON only** — no JSX, functions, fetched rows, auth vendors, or
  backend types. A typed **registry** resolves a `panelType` to a renderer, label, width.

## FRAMEWORD CANONICAL CONTEXT (paste block)

```text
FRAMEWORD CANONICAL CONTEXT

Frameword/Stax is a context-preserving UX framework for relational, data-dense
operational software. A top-level Space opens a RootPanel. Opening a related record
creates a DetailPanel beside its source, preserving context while the user investigates
or acts.

The Sidebar launches Spaces (the only thing that changes lineage). The Topbar holds the
derived breadcrumb (ContextPath), search, and global utilities. The Stage is the
horizontal rail of panels. A UtilityDrawer is an overlay for cross-context tools and is
NEVER part of the stack.

ContextPath is the current semantic ancestry of the context leaf and drives breadcrumbs,
close behavior, and deep links. A DrillTrigger opens a related DetailPanel. A preview
panel is transient; a pinned panel is retained and becomes an explicit detached reference
if its branch is replaced. Persistent resource-level commit/create actions belong in the
PanelFooter; navigation, row actions, and local secondary actions remain near their object.

Navigation state contains validated JSON descriptors: stable instanceId, target
(panelType + resourceKey + JSON params), spaceId, parentInstanceId, role, retention,
placement. It NEVER contains JSX, component functions, callbacks, fetched records, auth
vendors, or backend type imports. A typed registry resolves descriptors to renderers,
labels, URLs, and widths.

Opening the same target from the same contextual parent reveals and focuses it. The same
target under a different parent may have a distinct attached instance. Closing a parent
closes preview descendants and detaches retained descendants into the ReferenceRail.
Active ancestry cannot be arbitrarily reordered. Browser Back/Forward are URL
reconciliation and are distinct from Navigate Up. On narrow screens the same ContextPath
becomes one focused panel (a back-stack) rather than a forced desktop rail.

Stax is not a chart library, card collection, admin template, backend, freeform window
manager, visual brand, or universal replacement for pages, tabs, dialogs, and drawers.
Describe unbounded logical depth with a bounded rendered presentation — never market
"infinite visible depth".
```

## Separate these layers in every answer

1. engine invariants; 2. adaptive layout behavior; 3. accessibility & focus behavior;
4. panel anatomy; 5. visual theme tokens; 6. optional recipes (AI chat, kanban, KPI, a
graph/React-Flow panel type); 7. application domain data. Keep domain data OUT of the
framework; keep theme rules OUT of state invariants.

## The shell (brand-agnostic spatial contract)

```
┌────────┬──────────────────────────────────────────────┬ ─ ─ ─ ┐
│ SIDE   │ TOPBAR : breadcrumb(derived) · search · ⌘K    │ DRAWER│
│ BAR    ├──────────────────────────────────────────────┤ (over-│
│ opens  │ STAGE : rail of panels (scrolls right)        │ lay,  │
│ Spaces │ ┌──────┐┌──────┐┌──────┐                      │ NOT on│
│        │ │ root ││child ││child │  →                   │ stack)│
└────────┴──────────────────────────────────────────────┴ ─ ─ ─ ┘
```

A panel's internal contract: **HEADER** (fixed height — move ‹ ›, pin, close; separators
align across the rail) / **BODY** (scrollable vertically — NEVER horizontal-scroll inside;
title, content) / **FOOTER** (optional, anchored — the ONE action zone).

## Sizing grammar (iPhone → iPad → Mac)

Branch on the **container's** width class, never the device: `compact` (<600) / `medium`
(600–1100) / `expanded` (>1100). Slot budget `k = max(1, floor(width / minPanelWidth))`
panels render expanded (phone k=1, iPad k=2, Mac k=3–4). Panels outside the window collapse
to vertical spines (~34px) that stay visible — the trail never leaves the screen. Width is
set by panel *type* (narrow ~280 / normal ~340 / wide ~520 / fill for canvas & kanban). All
of this is presentation (`{sizeClass, slots, anchor, userWidths, zoom}` — device-local),
never navigation state. Content adapts to its **panel's** width via container queries
(`@container`), never viewport media queries.
