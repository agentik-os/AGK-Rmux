# Spacing Reference — 8px Grid System

## The Rule
Every spacing value must be a multiple of 4px. The rhythm is 8px.

## Scale

| Token | Value | Use Case |
|-------|-------|----------|
| `0` | 0px | Flush positioning |
| `px` | 1px | Borders, hairlines |
| `0.5` | 2px | Tight internal gaps (icon-to-text) |
| `1` | 4px | Tight spacing (badge padding, chip gaps) |
| `1.5` | 6px | Compact spacing |
| `2` | 8px | Base unit — default gap between related items |
| `3` | 12px | Comfortable gap — form field spacing |
| `4` | 16px | Section gap — between groups of related content |
| `5` | 20px | Group separator |
| `6` | 24px | Card internal padding — standard |
| `8` | 32px | Section padding — major content groups |
| `10` | 40px | Large gap — between major sections |
| `12` | 48px | Section break |
| `16` | 64px | Page section divider |
| `20` | 80px | Hero spacing, page-level breathing |

## Common Patterns

### Card Padding
- **Compact card:** p-4 (16px)
- **Standard card:** p-6 (24px)
- **Spacious card:** p-8 (32px)
- **NEVER mix** within the same card type

### Page Container
- **Max width:** max-w-7xl (1280px) for dashboards
- **Side padding:** px-4 sm:px-6 lg:px-8
- **Top padding:** pt-6 or pt-8

### Section Spacing
- **Between sections:** space-y-8 (32px) minimum
- **Between cards in a grid:** gap-4 (16px) or gap-6 (24px) — pick ONE
- **Between form fields:** space-y-4 (16px) or space-y-6 (24px) — pick ONE

### Table Rows
- **Compact:** py-2 px-4 (h-10)
- **Standard:** py-3 px-4 (h-12)
- **Spacious:** py-4 px-6 (h-14)
- Row height MUST be consistent across ALL tables

## Red Flags (Auto-fail)
- `p-3` and `p-4` used on the same component type = FAIL
- `gap-3` and `gap-4` mixed in same layout = FAIL
- Arbitrary values like `p-[13px]` or `mt-[7px]` = FAIL
- Different padding on left vs right without reason = FAIL
