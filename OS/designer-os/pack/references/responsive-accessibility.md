# Responsive and accessibility contract

## Contents

1. Conformance target
2. One state, multiple hosts
3. Responsive transformation matrix
4. Keyboard, focus, and dismissal
5. Semantic components
6. Content and sensory access
7. Test matrix

## 1. Conformance target

Target WCAG 2.2 AA for web surfaces unless a stricter legal/product requirement applies. Treat the W3C specification and ARIA Authoring Practices as primary references. Do not assume a shadcn primitive makes a composed product surface accessible automatically.

Include WCAG 2.2 concerns often missed in design:

- focus not obscured;
- dragging alternatives;
- minimum target size;
- redundant-entry avoidance;
- accessible authentication;
- consistent help;
- reflow, contrast, text spacing, status messages, and pointer cancellation.

## 2. One state, multiple hosts

Keep domain/navigation state independent from presentation. Select hosts by container capability, not device name:

- `compact`: under roughly 600–640 px or reflow/zoom equivalent;
- `medium`: roughly 600–1100 px;
- `expanded`: above roughly 1100 px.

Exact breakpoints must come from content stress tests. A tablet split view may be compact. A desktop embedded panel may be compact. Use container queries for component layout and viewport queries for true viewport/chrome behavior.

Preserve task, context, and state across hosts. Change presentation, not meaning.

## 3. Responsive transformation matrix

For each `SURF-###`, define:

| Concern | Expanded | Medium | Compact |
| --- | --- | --- | --- |
| Navigation | Full shell/rail | Reduced context | Focused surface + semantic back/up |
| Secondary context | Adjacent panel | Collapsible/one ancestor | Pushed surface or tray |
| Artifact/source | Resizable side surface | Overlay/split by space | Full-height pushed/modal host |
| Table | Full columns | Prioritized columns | List/cards only if semantics preserved |
| Actions | Persistent footer/toolbar | Compact overflow | Thumb-reachable primary + menu |
| Filters | Inline/side | Popover/drawer | Sheet/full-height filter task |
| Composer | Full controls | Reduced labels | Essential controls + overflow |
| Drag | Direct manipulation | Direct + buttons | Buttons/menu alternative mandatory |

Never hide a capability solely because the viewport is narrow. If deferred, provide an accessible route and state preservation.

Account for:

- safe areas and software keyboard;
- portrait/landscape;
- touch, mouse, trackpad, stylus, and keyboard;
- coarse pointer hover absence;
- standalone/PWA/browser chrome;
- RTL logical direction;
- translated expansion;
- offline/slow network.

## 4. Keyboard, focus, and dismissal

### Global laws

- Complete every P0 flow by keyboard.
- Keep focus visibly distinct from selection and active state.
- Never move focus because content streamed above it.
- Opening a surface moves focus to its heading or first meaningful control.
- Closing restores the trigger when it still exists.
- Route/panel navigation updates document title/landmark/heading context.
- Do not put every ancestor panel's controls into one enormous Tab sequence; provide panel traversal.

### Escape ladder

Declare one ordered dismissal registry, for example:

1. text-selection toolbar;
2. tooltip/popover/menu/combobox;
3. dialog or mobile modal sheet;
4. command palette;
5. utility drawer;
6. active preview/detail panel;
7. root/home only when product policy allows.

Do not let multiple components independently capture Escape.

### Shortcuts

- Centralize keys and conflicts.
- Ignore global printable shortcuts while editing unless explicitly scoped.
- Provide platform labels and remapping where power use justifies it.
- Expose a generated shortcut guide.
- Test browser/assistive-technology conflicts.

## 5. Semantic components

Prefer native HTML. Use ARIA patterns only when behavior matches.

- Every input has a visible or programmatic name, description, and error relationship.
- Menus contain actions; navigation links are links; selectable options use listbox/combobox patterns.
- A visual horizontal panel rail is not automatically a tree/treegrid.
- A data table remains a table; an interactive grid adopts grid keyboard behavior only if fully implemented.
- Modal dialogs trap focus, make background inert, receive a sensible initial focus, and restore focus.
- Nonmodal desktop drawers keep coherent reading/Tab order; their mobile modal host follows dialog rules.
- Progress and status changes use appropriate live-region behavior without chatty announcements.
- Removable context chips expose label, type, removal, and state.
- Pin/toggle states expose `aria-pressed`/checked state and meaningful labels.
- Resizable separators are keyboard operable and announce values when relevant.

## 6. Content and sensory access

- Minimum contrast follows WCAG target; test actual rendered states, not token names.
- Focus rings remain visible against every surface and are not clipped/obscured.
- State never depends only on hue, motion, position, icon, or sound.
- Text can resize/reflow without two-dimensional page scrolling, except essential canvases/tables with alternatives.
- Tooltips are supplemental; critical labels and consequences are visible.
- Images have product-owned alt-text policy; decorative images are ignored.
- Audio/video supports captions/transcripts and player keyboard behavior where used.
- Errors identify fields, explain correction, and preserve valid input.
- Authentication avoids cognitive-function-only puzzles and supports password managers/paste.
- Timed undo/status controls pause/extend or remain recoverable in activity history when consequence warrants it.

## 7. Test matrix

Create `EVAL-###` cases for:

- keyboard-only P0 journeys;
- screen reader with representative browser/platform combinations;
- 200% zoom and 320 CSS px reflow where applicable;
- high contrast/forced colors;
- reduced motion;
- touch target and dragging alternative;
- long localized strings and RTL;
- empty/loading/error/offline/permission-lost states;
- software keyboard and mobile safe area;
- focus restoration after menus/dialogs/panels/branch changes;
- live streaming without focus or scroll theft;
- table/list responsive semantics;
- color contrast in all states and themes.

Record automated checks and manual checks separately. Automated accessibility tests are necessary but cannot approve interaction semantics alone.

