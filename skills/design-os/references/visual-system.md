# Modern elegant visual-system protocol

## Contents

1. Visual thesis
2. Reference translation
3. Semantic foundations
4. Layout, density, and surface grammar
5. Type, icons, and content
6. Motion and feedback
7. Data and generated content
8. Visual QA and anti-patterns

## 1. Visual thesis

Derive a product-specific thesis with four parts:

```text
[emotional quality] + [information behavior] + [material language] + [signature]
```

Example structure, not a default: “calm authority + progressive disclosure + paper/ink surfaces + one precise accent.”

Tie each phrase to audience, context, frequency, trust, and brand evidence. Avoid empty words such as clean, premium, futuristic, or elegant without observable rules.

Modern elegance usually comes from restraint, proportion, typography, state clarity, and response quality—not glass, gradient, blur, large radius, or animation by themselves.

## 2. Reference translation

Use ChatGPT, Claude, Grok, Linear, iOS, or other products as pattern references, never as trade-dress templates.

Translate structural strengths:

| Reference family | Learn from | Do not copy |
| --- | --- | --- |
| ChatGPT-like | Composer centrality, low-chrome conversation, tool/source visibility | Exact shell, icons, spacing, labels, or brand treatment |
| Claude-like | Calm reading rhythm, artifact separation, long-form legibility | Brand palette, typography, or artifact chrome |
| Grok-like | Directness, mode visibility, responsive media/research states | Distinctive brand motifs or interaction quirks without user value |
| STAX | Context-preserving drills, serializable workspace, panel discipline | Panels everywhere or fixed WhitePaper aesthetic |
| shadcn | Composable accessible open code and registry distribution | Default demo appearance as final product identity |

For every reference screenshot or link, record the observed problem solved, transferred principle, adapted rule, and rejected imitation.

## 3. Semantic foundations

### 3.1 Color

Define OKLCH semantic roles before values:

- canvas, surface, elevated surface, input, overlay;
- primary, secondary, muted, inverse text;
- subtle, default, and strong borders;
- accent and accent foreground;
- success, warning, destructive, information;
- focus ring, selection, skeleton, and chart series.

Use one intentional accent family by default. Add semantic colors because meaning requires them, not decoration. Define light, dark, forced-colors, and print mappings independently; dark mode is not an inversion filter.

Do not use color alone for state. Pair with label, icon, pattern, position, or shape.

### 3.2 Typography

Define roles:

- display/brand;
- surface/page title;
- section heading;
- body and long-form body;
- UI label/control;
- caption/metadata;
- mono/data/code.

Use tabular numerals for changing/comparable data. Limit long-form line length. Preserve hierarchy at compact sizes instead of scaling every role proportionally.

Choose font licenses, loading strategy, locale coverage, fallback metrics, and variable axes. Avoid pairing serif + mono + sans merely to look editorial; each role must improve scanning or identity.

### 3.3 Spacing and density

Use a 4 px or similarly coherent base and named semantic spacing:

- inline control gap;
- control padding;
- row rhythm;
- section rhythm;
- surface gutter;
- shell gutter;
- safe-area inset.

Offer density only when task frequency/data volume justifies it. Cozy/compact/dense must preserve target size, focus, errors, and content hierarchy.

### 3.4 Shape and elevation

Use a small radius scale. Assign by function: control, floating surface, panel, media—not random per component.

Prefer border, tonal separation, and whitespace for in-flow structure. Reserve shadow/elevation for floating or truly layered surfaces. Avoid “card soup,” where every section is a rounded container.

## 4. Layout, density, and surface grammar

Start from the primary user question and data shape:

| Data/task shape | Preferred visual grammar |
| --- | --- |
| Editable homogeneous records | Real table/list with scan and bulk action |
| Magnitude by entity | Ladder/proportional rows |
| Computation to a result | Walk with terms, totals, and meaning-aware delta |
| Quantity over time | Metered series plus exact values |
| Live events | Append-only stream/log |
| Relationship exploration | Graph/canvas plus inspector |
| Long reusable output | Document/artifact surface |
| Ambiguous orchestration | Conversation plus visible context/tools |
| Configuration | Sections/fields with consequence near control |

Do not wrap a table in cards and call it a dashboard. Derive totals from one source. Reserve numeric width. Use container queries when a component can be narrow inside a wide viewport.

Use optical alignment, not mathematical centering alone. Define text rails and edge alignment. Repeated hairlines and block padding should produce rhythm; do not add arbitrary spacer elements.

## 5. Type, icons, and content

### Icons

- Use one stroke/fill language and a small set of optical sizes.
- Import per icon when bundle behavior matters.
- Give icon-only actions accessible names and tooltips after a short delay.
- Use icons only when they accelerate recognition.
- Never rely on three similar sparkles to distinguish unrelated AI actions.

### Microcopy

- Label actions with verbs and objects.
- Name status with specific system truth: “Uploading 2 of 5,” not “Working.”
- Write empty states as outcome + next action.
- Put consequence near the control.
- Use sentence case unless brand/locale rules require otherwise.
- Localize expansion-prone labels and bidirectional text in prototypes.

### AI copy

- Distinguish “searching,” “reading,” “analyzing,” “calling tool,” and “drafting.”
- Avoid claiming certainty, understanding, or completion unsupported by system state.
- Say whether content is generated, cited, inferred, or user-edited when it matters.

## 6. Motion and feedback

Define durations by distance and cognitive change:

- micro-state: approximately 80–140 ms;
- floating surface: approximately 120–200 ms;
- panel/route transition: approximately 180–320 ms;
- progress: driven by real events, not a fake fixed duration.

Use easing tokens. Motion communicates origin, hierarchy, cause, and continuity. Do not animate stable reading content for decoration.

Reduced motion preserves state clarity with opacity, instant placement, or minimal transition. Never remove the only cue that something changed.

Feedback hierarchy:

1. local inline response;
2. persistent status/progress in the affected surface;
3. lightweight toast/status region for completion + undo;
4. modal interruption only when attention/approval is required.

## 7. Data and generated content

### Data visualization

- Start from the user question and decision.
- Prefer exact labels and accessible table alternatives.
- Use consistent scales and meaning-aware sign/color.
- Define zero, missing, partial, estimated, and stale data.
- Test dark mode, color blindness, high contrast, long values, negative values, and localization.
- Do not add a chart library when a simple semantic bar or inline SVG is more robust.

### Generated images/media

- Define aspect ratio, crop/focal behavior, loading, error, moderation, alt-text ownership, and provenance.
- Distinguish preview from final asset.
- Preserve prompt/source/version relationships where editing is supported.
- Avoid synthetic decorative imagery that weakens trust in a serious workflow.

## 8. Visual QA and anti-patterns

Review each P0 surface at compact/medium/expanded widths, light/dark, empty/loading/error, long localization, 200% zoom, reduced motion, and representative real data.

Reject:

- default shadcn demo styling presented as a finished brand;
- gratuitous gradient, glass, neon, glow, or oversized blur;
- excessive rounded cards and floating pills;
- weak gray-on-gray hierarchy;
- tiny low-contrast metadata carrying essential meaning;
- uppercase tracking on long labels;
- enormous dashboard KPIs without task rationale;
- inconsistent radii, shadows, icon strokes, or accent usage;
- animation that blocks action or changes scroll unexpectedly;
- skeletons unrelated to final geometry;
- dark mode with bright grids/hairlines or vanishing secondary surfaces;
- visual variants with no semantic/API contract;
- pixel values copied from STAX or a reference without product-specific justification.

Approval evidence must include token inventory, component states, representative surface captures/prototypes, and explicit diffs against the approved design contracts.

