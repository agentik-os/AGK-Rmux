# Stax conversion playbook — convert ANY project fully, phase by phase

Grounded in `~/.omega/repos/stax/PROMPT-KIT.md` (the canonical prompt kit). Paste the
**FRAMEWORD CANONICAL CONTEXT** (see `canonical-context.md`) once, then run each phase.
Every phase gates the next — do not write panel code before Phase 1 is internally
consistent (L2: challenge the premise; a bad panelization is worse than pages).

---

## Phase 0 — FIT (GO / ADAPT / DO NOT USE)

First inspect the target: routes, nav, components, domain model, top user workflows.

```text
Evaluate this product for Stax:
[PRODUCT / REPO BRIEF: routes, entities, workflows]

Identify:
- primary users and high-frequency jobs;
- entities and meaningful relationships;
- where conventional route changes lose context (the case FOR Stax);
- candidate Spaces (the sidebar sections);
- three golden ContextPaths (root → … → leaf) users walk daily;
- workflows that must stay normal pages (linear/sequential/unrelated);
- alternate representations that should be tabs INSIDE one panel;
- blocking decisions that stay dialogs;
- cross-context tools that should be a UtilityDrawer;
- comparison needs → candidate pinned panels;
- desktop vs phone expectations;
- a fit score 0–5 with evidence.

Reject panelization where data is shallow, sequential, or unrelated. End with
GO, ADAPT, or DO NOT USE and justify.
```

**Do not proceed on a DO NOT USE.** On ADAPT, panelize only the relational core and keep
the rest as pages.

---

## Phase 1 — BLUEPRINT (navigation, before any component)

```text
Using FRAMEWORD CANONICAL CONTEXT and this domain model:
[DOMAIN MODEL + the Spaces from Phase 0]

Produce a navigation blueprint. For every Space and panel type specify:
- panelType (registry key) and canonical resourceKey scheme;
- source & parent relationship (what drills into it);
- DrillTrigger (the click that opens it);
- size class S/M/L/XL (width belongs to the kind);
- title source and breadcrumb label source;
- body responsibilities;
- the ONE persistent PanelFooter action, if any (else none);
- preview vs pin policy;
- URL/resourceKey representation;
- states: loading, empty, error, not-found, permission-lost, stale, offline,
  deleted-resource, dirty-edit;
- desktop / compact / phone presentation.

Then give: three end-to-end ContextPaths; the behavior for open, reopen,
sibling-replace, pin, unpin, close, close-ancestor, Back/Forward, refresh, shared
deep link; areas deliberately EXCLUDED from panel navigation; open decisions.

Do not write components yet — make the IA and transition semantics consistent first.
```

Output artifact: the `PanelRegistry` map (`panelType → {size, label}`) + a drill graph
(who opens whom). This is the contract Phase 3 fills.

---

## Phase 2 — SCAFFOLD (mechanical wiring)

```sh
bash ~/.omega/skills/stax/scripts/stax-scaffold.sh <target-project-dir>
```

Vendors `panels-core` + `panels-react` (import rewritten to a local path — no npm
package needed, zero deps beyond React), plus a `stax/` shell (`Shell`, `Sidebar`,
`Topbar` breadcrumb, `ColumnHost`, `PushHost`, `Panel` = header/body/footer), `tokens.css`,
and a `registry.tsx` seeded from Phase 1. Idempotent; never overwrites your domain files.
After it runs you have a live empty workspace to port into.

If the target is a fresh Next.js App-Router app: keep exactly one route (`/`), render the
shell there, and let everything else be panel state (Prompt 6 below for the adapter).

---

## Phase 3 — PORT (convert the real UI)

Work **one golden ContextPath end-to-end first** (Phase 0's #1), verify it (Phase 5), then
expand. For each existing surface, apply the mapping table in `SKILL.md`. Concretely:

1. **Sidebar** — replace nav sections with `ws.openSpace(spaceId, target)` buttons.
2. **Each page/screen** — register a `panelType`; move its JSX into a `<PanelBody>`
   renderer keyed by `panel.target.panelType`. Fetch its data from
   `panel.target.resourceKey`/`params` (never store the fetched rows in state).
3. **Each navigation** — replace `router.push`/`<Link>` with `ws.openDetail(parentId,
   target)` for drills, or `ws.openSpace` for section switches.
4. **Modals** — a modal that *views/edits a record* becomes a DetailPanel (delete the
   modal). A modal that is a *blocking yes/no* stays a dialog.
5. **Tabs** — alternate views of ONE record → tabs inside that panel's body; different
   records → separate panels.
6. **Actions** — persistent commit/create → the `<PanelFooter>`; row/local actions stay
   inline. No floating buttons.
7. **Breadcrumb** — render from `ws.path` (`getContextPath`); each crumb = `ws.navigateTo`.
8. **Compare** — expose a "pin" affordance in the panel header (`ws.pinPanel`).
9. **Command palette (⌘K)** — optional but on-brand: a flat search that calls
   `openSpace`/`openDetail`/`openPath`.

Keep `params` JSON-only. Run `ws.violations` (=`validate`) in dev — it must stay `[]`.

### Progressive adoption in an existing dashboard (Prompt 11)

```text
Given this existing dashboard (routes / components / domain / workflows), plan
progressive Stax adoption: pick ONE high-value relational workflow first, preserve
existing routes as fallback, map routes → Spaces / panel types / resourceKeys /
ContextPaths / readable URLs. Return: fit, chosen golden workflow, install order,
route→component migration map, data-ownership boundaries, feature flags, telemetry,
usability baseline, rollout cohorts, rollback plan, success/stop metrics, risks.
Metrics: time-to-target, backtracking/reopening, wrong-context actions, orientation
confidence, horizontal-nav burden, keyboard & phone completion, deep-link restore rate.
```

### The Next.js adapter (Prompt 6, when the target is Next.js App Router)

```text
Design the Next.js App-Router integration: one route owns the workspace; parse & validate
the initial URL → state; an application-owned readable route codec; PanelLink behavior;
push-vs-replace history per event; on popstate parse+validate then reconcileLocation;
navigateUp distinct from browser Back; refresh & shared-link reconstruction; not-found /
unauthorized / stale routing; progressive enhancement; hydration-mismatch prevention.
Do NOT model the arbitrary-length rail as statically named parallel-route slots.
```

---

## Phase 4 — THEME

Map the brand onto the semantic tokens in `stax/tokens.css` (shadcn-compatible oklch:
`--background --foreground --card --primary --accent --muted --border --ring`, radius,
spacing, shadow, motion scales, plus dark-mode parallels). Never hardcode colours/spacing
inside a domain panel. Per-panel responsiveness uses container queries (`@container`), not
viewport media queries. Change the accent and the whole system follows (Law 5).

---

## Phase 5 — VERIFY (evidence, not vibes — L1/L4/R-CITE)

Run the panels-core laws test-kit
(`~/.omega/repos/stax/frameword/packages/panels-core/test/laws.test.ts` as the template)
and, at runtime, confirm the nine gates in `SKILL.md`. Then run the adversarial pass:

```text
Try to falsify this Stax implementation. Treat every invariant as a hypothesis.
Scenarios: 12-level ContextPath; duplicate target opened twice from the same parent then a
different parent; sibling open after pinning a descendant; closing a middle ancestor with
preview + retained descendants; reorder references; click an older visible panel and
continue; preview replacement with unchanged rail length; Back/Forward after URL restore;
refresh + shared deep link; malformed/previous-version snapshot; permission revoked while
open; resource deleted while open; slow/stale query; phone viewport + rotation + zoom +
RTL + long labels; keyboard-only & screen-reader; UtilityDrawer + Escape.
For each: setup, action, expected state, observed result, falsified invariant, and an
automated test. Give a conformance score ONLY after listing evidence.
```

A conversion is done when `validate()` is `[]`, the laws test-kit is green, the golden
ContextPath restores from a shared URL, and every gate has cited runtime evidence.

---

## Extra prompts (from PROMPT-KIT.md — read the file for the full set)

- **Prompt 1** — explain/position Stax to an audience.
- **Prompt 4** — specify the headless state machine (already shipped in `panels-core`).
- **Prompt 5** — design the React/shadcn API (already shipped in `panels-react`).
- **Prompt 8** — create a visual system & theme from a brand brief.
- **Prompt 9** — accessibility & responsive audit.
- **Prompt 12** — build a brand-new workspace with the installed packages.
- **Prompt 14** — review a proposed change to the framework (which layer does it touch?).
