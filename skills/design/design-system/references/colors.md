# Color System Reference

## The Rule
ALL colors are tokens. Zero hardcoded hex/rgb values in components.

## Semantic Color Tokens (shadcn/Tailwind pattern)

### Base
| Token | Role | Light | Dark |
|-------|------|-------|------|
| `background` | Page bg | white | zinc-950 |
| `foreground` | Default text | zinc-950 | zinc-50 |
| `card` | Card bg | white | zinc-900 |
| `card-foreground` | Card text | zinc-950 | zinc-50 |
| `popover` | Popover bg | white | zinc-900 |
| `popover-foreground` | Popover text | zinc-950 | zinc-50 |

### Interactive
| Token | Role |
|-------|------|
| `primary` | Brand color — CTAs, main actions |
| `primary-foreground` | Text on primary |
| `secondary` | Secondary buttons, less prominent actions |
| `secondary-foreground` | Text on secondary |
| `accent` | Hover states, subtle highlights |
| `accent-foreground` | Text on accent |
| `muted` | Subtle backgrounds, disabled states |
| `muted-foreground` | Secondary text, placeholders, metadata |

### Semantic
| Token | Role | Color Family |
|-------|------|-------------|
| `destructive` | Delete, remove, errors | Red |
| `destructive-foreground` | Text on destructive | White |
| `success` | Success states, confirmations | Green |
| `warning` | Warnings, caution states | Amber/Yellow |
| `info` | Information, neutral alerts | Blue |

### Border & Ring
| Token | Role |
|-------|------|
| `border` | Default borders | 
| `input` | Input borders (slightly more visible) |
| `ring` | Focus ring color |

## Contrast Requirements (WCAG AA)
- Normal text: minimum 4.5:1 contrast ratio
- Large text (18px+ or 14px bold): minimum 3:1
- UI components: minimum 3:1
- **muted-foreground** must meet 4.5:1 against **background**

## Gray Scale Guidance
Pick ONE gray family and stick to it:
- **Zinc** (neutral, cool) — Linear, Vercel style
- **Slate** (slightly blue) — Discord, GitHub style
- **Gray** (true neutral) — Safe default
- **Stone** (warm) — Notion style

Never mix gray families. If `zinc-500` is used for borders, ALL grays should be zinc.

## Hover/Active Transformations
Use consistent transformations, never random darkening:
- **Light mode hover:** darken by 1 shade (primary-600 -> primary-700)
- **Dark mode hover:** lighten by 1 shade (primary-400 -> primary-300)
- **Active/pressed:** darken by 2 shades from default
- **Disabled:** add opacity-50

## Red Flags
- `#334155` hardcoded in a component = FAIL (use token)
- Different shades of gray for the same role = FAIL
- Accent color that doesn't meet WCAG AA contrast = FAIL
- Destructive action without red visual indicator = FAIL
- Success/error using same color = CRITICAL FAIL
