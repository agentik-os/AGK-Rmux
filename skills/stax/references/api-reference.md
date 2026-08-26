# Stax API reference (snapshot of shipped `panels-core` + `panels-react`)

> Snapshot for orientation. The **live source wins** — read
> `~/.omega/repos/stax/frameword/packages/panels-core/src/index.ts` and
> `.../panels-react/src/index.tsx` before wiring, since the checkout tracks latest main.

## `@frameword/panels-core` — pure state machine (no React, no DOM, no backend)

### Types

```ts
type JsonValue = null | boolean | number | string | JsonValue[] | { [k: string]: JsonValue };

interface PanelTarget {          // WHAT a panel points at (JSON only)
  panelType: string;             // a registered renderer key
  resourceKey: string;           // canonical id of the content (e.g. "contact:42")
  params?: Record<string, JsonValue>;
}

type PanelRole      = "root" | "detail";
type PanelRetention = "preview" | "retained";     // preview = transient; retained = pinned
type PanelPlacement = "context" | "reference";    // context = in the ancestry; reference = detached pin

interface PanelInstance {
  instanceId: string;            // "p1", "p2", … (minted by the reducer — deterministic)
  target: PanelTarget;
  spaceId: string;
  parentInstanceId: string | null;
  role: PanelRole;
  retention: PanelRetention;
  placement: PanelPlacement;
}

interface WorkspaceState {       // THE serializable state — everything derives from it
  schemaVersion: 1;
  spaceId: string | null;
  rootInstanceId: string | null;
  contextLeafId: string | null;  // the current leaf of the ancestry
  focusedPanelId: string | null;
  panelsById: Record<string, PanelInstance>;
  referenceRailOrder: string[];  // ordered detached pins
  nextId: number;
}
```

### Intent commands — each is `(state, …args) => newState` (pure)

| Command | Meaning |
|---|---|
| `emptyWorkspace()` | the initial empty state |
| `openSpace(state, spaceId, target)` | enter a section — **replaces the active thread** (retained descendants detach to the ReferenceRail, previews close, existing references survive). Re-opening the same root reveals it. |
| `openDetail(state, parentInstanceId, target)` | **drill = open-right.** Same target + same parent → reveal (no dup). Else previews below the parent close, retained detach, a new `preview` detail is appended and becomes the leaf. |
| `pinPanel(state, instanceId)` | `preview` → `retained` (survives sibling opens & branch changes). Root/reference no-op. |
| `unpinPanel(state, instanceId)` | attached → back to `preview`; a detached reference → **closes** it. |
| `closePanel(state, instanceId)` | close a panel; its previews close, its retained descendants detach. Closing the **root** ends the thread. |
| `navigateTo(state, instanceId)` | breadcrumb click — make `instanceId` the leaf; the branch below follows the subtree policy. |
| `navigateUp(state)` | leaf → its parent (distinct from browser Back). |
| `focusPanel(state, instanceId)` | move focus only. |
| `openPath(state, spaceId, targets[])` | open/reveal a whole chain root-first (deep-link / resume reconstruction). |
| `resumeReference(state, instanceId)` | → `{ state, target }`; removes the reference and returns its target so the app rebuilds a fresh ContextPath. |
| `moveReference(state, instanceId, dir: -1|1)` | reorder the ReferenceRail. |

### Derivations & guards (never store what you can derive)

```ts
getContextPath(state): string[]            // root → leaf instance ids (the breadcrumb + URL)
getSubtree(state, parentId): string[]      // attached descendants (depth-first)
getAttachedChild(state, parentId, target)  // the dedup lookup
validate(state): string[]                  // engine invariants 1–10 — MUST return [] (the laws)
encodeLocation(state): string | null       // ContextPath → URL-safe string
decodeLocation(encoded): EncodedLocation | null
reconcileLocation(state, loc): WorkspaceState   // URL → workspace (Back/Forward, refresh, deep link)
```

`validate()` catches: >1 root, dangling ids, orphan attached panels, cycles, a leaf/focus
pointing at a detached reference, references with an active parent, rail entries missing.
Run it after every reducer call in dev; it is the runtime proof of the 5 laws.

## `@frameword/panels-react` — bindings only (zero styling)

```ts
type PanelSize = "S" | "M" | "L" | "XL";
const PANEL_WIDTHS = { S: 380, M: 480, L: 640, XL: 800 };   // width belongs to the KIND

interface PanelTypeDef { size: PanelSize; label?: (resourceKey: string) => string; }
type PanelRegistry = Record<string /* panelType */, PanelTypeDef>;

// Provider — reads URL hash → localStorage snapshot → empty, and writes both on change
<WorkspaceProvider registry={REGISTRY} urlSync storageKey="my-app">…</WorkspaceProvider>

// The hook — the whole API, wrapped to setState:
const ws = useWorkspace();
//  ws.state, ws.path (getContextPath), ws.violations (validate), ws.registry,
//  ws.openSpace, ws.openDetail, ws.pinPanel, ws.unpinPanel, ws.closePanel,
//  ws.navigateTo, ws.navigateUp, ws.focusPanel, ws.openPath,
//  ws.resumeReference(id, rebuildFn), ws.moveReference, ws.reset

usePanel(id): PanelInstance | undefined
panelWidth(registry, panel, override?): number
useIsCompact(breakpoint = 640): boolean        // true → render PushHost (one-column back-stack)
```

`WorkspaceProvider` precedence on load: **URL ContextPath → device-local snapshot → empty**.
`urlSync` writes the encoded ContextPath into `location.hash`; `storageKey` snapshots the
whole workspace to `localStorage`. A malformed snapshot degrades to empty — never throws.

## Minimal wiring (what the scaffold generates)

```tsx
// registry.tsx — width belongs to the kind, not the user
export const REGISTRY: PanelRegistry = {
  "space":    { size: "M", label: (k) => k },
  "contact":  { size: "M", label: (k) => `Contact ${k}` },
  "deal":     { size: "L", label: (k) => `Deal ${k}` },
  "canvas":   { size: "XL" },
};

// App.tsx
export function App() {
  return (
    <WorkspaceProvider registry={REGISTRY} urlSync storageKey="my-app">
      <Shell />   {/* Sidebar (openSpace triggers) + Topbar (breadcrumb) + Stage */}
    </WorkspaceProvider>
  );
}

// Stage — swap host by container width, SAME state
function Stage() {
  const compact = useIsCompact();
  return compact ? <PushHost /> : <ColumnHost />;
}

// a drill: opening a related record to the right of panel `id`
onClick={() => ws.openDetail(id, { panelType: "deal", resourceKey: `deal:${deal.id}` })}
```

Panel anatomy in the shell: `<PanelHeader>` (move/pin/close), `<PanelBody>` (the ported
screen — vertical scroll only), `<PanelFooter>` (the one commit/create action, optional).
