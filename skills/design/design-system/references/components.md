# Component Reference — Consistency Specs

## Buttons

### Size Scale
| Size | Height | Padding | Font | Radius | Icon Size |
|------|--------|---------|------|--------|-----------|
| xs | h-7 (28px) | px-2 | text-xs | rounded-md | 14px |
| sm | h-8 (32px) | px-3 | text-sm | rounded-md | 16px |
| md | h-9 (36px) | px-4 | text-sm | rounded-md | 16px |
| lg | h-10 (40px) | px-4 | text-base | rounded-md | 20px |
| xl | h-11 (44px) | px-6 | text-base | rounded-lg | 20px |

### Variant Specs
- **Primary:** Solid bg, white text, shadow-sm on hover
- **Secondary:** Border + bg-secondary, darker text
- **Ghost:** No border, no bg, hover:bg-accent
- **Destructive:** Red-based primary for delete/remove actions
- **Link:** No padding, underline, inline display

### Rules
- One default size per context (sm for tables, md for forms, lg for CTAs)
- ALWAYS include hover, focus, active, disabled states
- Icon buttons: square aspect ratio (w-9 h-9 for md)
- Loading state: spinner replaces text, button stays same width

## Inputs

### Size Scale
| Size | Height | Padding | Font | Radius |
|------|--------|---------|------|--------|
| sm | h-8 | px-3 | text-sm | rounded-md |
| md | h-9 | px-3 | text-sm | rounded-md |
| lg | h-10 | px-4 | text-base | rounded-md |

### States
- **Default:** border-input, bg-background
- **Focus:** ring-2 ring-ring ring-offset-2
- **Error:** border-destructive, text-destructive for helper text
- **Disabled:** opacity-50 cursor-not-allowed

### Rules
- Label always above input (not inline, not floating)
- Helper text below input, text-xs text-muted
- Error text replaces helper text (same position, red color)
- Required indicator: red asterisk after label

## Cards

### Specs
| Property | Value |
|----------|-------|
| Padding | p-6 (standard) or p-4 (compact) — pick ONE per card type |
| Radius | rounded-lg (8px) |
| Border | border (1px solid) or shadow-sm — pick ONE approach |
| Background | bg-card |
| Header gap | mb-4 between title and content |
| Footer gap | mt-4 or pt-4 border-t for action area |

### Rules
- ALL cards of the same type must have identical padding
- Card title: text-lg font-medium or text-base font-medium — pick ONE
- Card description: text-sm text-muted
- Card grid: gap-4 or gap-6 — pick ONE for the entire app

## Modals / Panels / Drawers

### Modal (Centered Overlay)
- Width: max-w-md (default), max-w-lg (large forms), max-w-sm (confirmations)
- Padding: p-6
- Radius: rounded-xl (12px)
- Overlay: bg-black/50, backdrop-blur-sm
- Close: X button top-right OR click-outside
- Animation: fade-in + scale-up (200ms ease-out)

### Side Panel (Sheet)
- Width: w-[400px] (default), w-[600px] (detail view), w-[320px] (compact)
- Padding: p-6
- Position: right side (always)
- Overlay: bg-black/30
- Animation: slide-in from right (300ms ease-out)

### Drawer (Bottom)
- Height: auto (content-based) with max-h-[80vh]
- Padding: p-6
- Radius: rounded-t-xl on top corners
- Handle: 4px x 48px centered bar at top

### Decision Rule
Pick ONE pattern per interaction type and use it EVERYWHERE:
- Quick confirmation -> Modal
- Create/edit with context -> Side Panel
- Detail view -> Side Panel
- Settings page -> Full page
- Mobile actions -> Bottom Drawer

## Tables

### Specs
| Property | Compact | Standard | Spacious |
|----------|---------|----------|----------|
| Row height | h-10 | h-12 | h-14 |
| Cell padding | py-2 px-4 | py-3 px-4 | py-4 px-6 |
| Header | text-xs font-medium uppercase tracking-wider text-muted-foreground | same | same |
| Border | border-b only | border-b only | border-b only |
| Hover | hover:bg-muted/50 | hover:bg-muted/50 | hover:bg-muted/50 |
| Stripe | NO (modern tables dont stripe) | NO | Optional |

### Rules
- ALL tables in the app use the SAME density
- Column alignment: text left, numbers right, actions center
- Sortable columns: show sort indicator on header hover
- Empty table: show empty state component, not blank space
- Pagination: consistent placement (bottom-right), same component

## Navigation

### Sidebar
- Width: w-64 (256px) expanded, w-16 (64px) collapsed
- Item height: h-9 or h-10 — consistent for all items
- Item padding: px-3 py-2
- Active state: bg-accent text-accent-foreground or left border highlight
- Hover state: bg-muted
- Section labels: text-xs font-medium uppercase tracking-wider text-muted mb-2
- Group spacing: mt-6 between groups

### Top Header
- Height: h-14 (56px) or h-16 (64px) — consistent
- Padding: px-4 sm:px-6
- Border: border-b
- Z-index: z-50 (always on top)
