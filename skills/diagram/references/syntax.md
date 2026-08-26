# Diagram-as-code cheat-sheet (Mermaid + D2)

Correct, minimal examples. Copy, adapt, render with `render.sh`. Put the theme
block on line 1. Default to Mermaid; reach for D2 for styled infra / SQL tables /
big graphs.

---

## Mermaid

Save as `*.mmd`. Optional theme on line 1:

```
%%{init: {'theme':'base','themeVariables':{'primaryColor':'#2563EB','primaryTextColor':'#FFFFFF','primaryBorderColor':'#1D4ED8','lineColor':'#94A3B8','fontFamily':'Inter, system-ui, sans-serif'}}}%%
```

### flowchart (process, decision, agent flow, architecture)
```
flowchart LR
  A([Start]) --> B{Authorized?}
  B -- yes --> C[Run task]
  B -- no  --> D[Reject]
  C --> E([Done])
```
Architecture = `flowchart` + `subgraph` groups:
```
flowchart TB
  subgraph Client
    UI[Web UI]
  end
  subgraph Backend
    API[API] --> Q[(Queue)]
    Q --> W[Worker]
    W --> DB[(Postgres)]
  end
  UI -->|HTTPS| API
```
Node shapes: `[rect]` `([stadium])` `{diamond}` `[(database)]` `((circle))` `[[subroutine]]`.
Edges: `-->` `-- text -->` `-.->` (dotted) `==>` (thick).

### sequenceDiagram (interaction / API call order)
```
sequenceDiagram
  participant U as User
  participant A as API
  participant D as DB
  U->>A: POST /checkout
  A->>D: INSERT order
  D-->>A: ok
  A-->>U: 201 Created
  Note over A,D: wrapped in a txn
```
Arrows: `->>` solid call, `-->>` dashed return, `-x` lost message.

### stateDiagram-v2 (lifecycle / status)
```
stateDiagram-v2
  [*] --> Draft
  Draft --> Review: submit
  Review --> Published: approve
  Review --> Draft: reject
  Published --> [*]
```

### erDiagram (data model)
```
erDiagram
  CUSTOMER ||--o{ ORDER : places
  ORDER ||--|{ LINE_ITEM : contains
  CUSTOMER {
    string id PK
    string email
  }
  ORDER {
    string id PK
    string customer_id FK
  }
```
Cardinality: `||` one, `o{` zero-or-many, `|{` one-or-many.

### mindmap (brainstorm / topic tree)
```
mindmap
  root((OmegaOS))
    Orchestration
      Oracle
      Worker
    Interface
      Telegram
      TUI
```
Indentation = hierarchy. Root shape `((text))`.

### gantt (timeline / roadmap)
```
gantt
  title Launch plan
  dateFormat YYYY-MM-DD
  section Build
  Scaffold      :a1, 2026-01-01, 7d
  Acceptance    :after a1, 3d
  section GTM
  Landing page  :2026-01-12, 5d
```

### quadrantChart (2×2 / prioritization)
```
quadrantChart
  title Effort vs Impact
  x-axis Low Effort --> High Effort
  y-axis Low Impact --> High Impact
  quadrant-1 Quick wins
  quadrant-2 Big bets
  quadrant-3 Drop
  quadrant-4 Fill-ins
  Feature A: [0.3, 0.8]
  Feature B: [0.7, 0.6]
```

---

## D2

Save as `*.d2`. Layout is automatic (dagre). Reach for D2 when you need rich
styling, SQL tables, nested containers, or cleaner large-graph layout.

### Shapes & connections
```d2
api: API Gateway
db: Database { shape: cylinder }
user: { shape: person }
queue: { shape: queue }
user -> api: request
api -> db: query
api -> queue: enqueue
```
Common shapes: `rectangle` (default) `cylinder` `person` `queue` `hexagon`
`cloud` `oval` `diamond` `page` `document` `stored_data`.

### Containers (architecture grouping)
```d2
backend: {
  api: API
  worker: Worker
  api -> worker
}
db: Database { shape: cylinder }
backend.api -> db
```
Nest with `.` or a `{ }` block. Edges cross containers via the dotted path.

### sql_table (schema / ER)
```d2
users: {
  shape: sql_table
  id: int {constraint: primary_key}
  email: varchar
}
orders: {
  shape: sql_table
  id: int {constraint: primary_key}
  user_id: int {constraint: foreign_key}
}
orders.user_id -> users.id
```

### classes + vars (reusable theme)
```d2
vars: {
  primary: "#2563EB"
  accent:  "#6C5CE7"
}
classes: {
  primary: { style: { fill: "${primary}"; font-color: "#FFFFFF"; stroke: "${primary}"; border-radius: 8 } }
  accent:  { style: { fill: "${accent}";  font-color: "#FFFFFF"; stroke: "${accent}";  border-radius: 8 } }
}
api: API { class: primary }
db:  DB  { shape: cylinder; class: accent }
api -> db
```
Per-shape style without a class: `node.style.fill: "#EEE"`. Direction:
`direction: right` (or `down`/`left`/`up`).

---

## Brand theming — quick reference

Resolve a palette from `.agents/brand.json` (or `.agents/brand.md`,
`tailwind.config.*` `theme.colors`). None found → clean neutral default:
ink `#1A1A1A`, bg `#FFFFFF`, primary `#2563EB`, accent `#6C5CE7`, muted
`#E2E8F0`, font `Inter, system-ui, sans-serif`. Keep ≤ 5 colors, high contrast.

- **Mermaid** → the `%%{init: {'theme':'base','themeVariables':{…}}}%%` block above
  (set `primaryColor`, `primaryTextColor`, `lineColor`, `fontFamily`, `background`).
- **D2** → the `vars` + `classes` block above; assign with `class:` and theme the
  whole figure from a handful of vars.
