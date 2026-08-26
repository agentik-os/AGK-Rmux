# Typography Reference — Type Scale & Hierarchy

## The Rule
Maximum 7 font sizes. Each size has ONE semantic role. No exceptions.

## Scale

| Token | Size | Line Height | Weight | Role |
|-------|------|-------------|--------|------|
| `text-xs` | 12px | 16px (1.33) | 400/500 | Captions, timestamps, metadata, labels |
| `text-sm` | 14px | 20px (1.43) | 400/500 | Secondary text, table cells, sidebar items |
| `text-base` | 16px | 24px (1.5) | 400 | Body text, form inputs, descriptions |
| `text-lg` | 18px | 28px (1.56) | 500/600 | Card titles, section subtitles |
| `text-xl` | 20px | 28px (1.4) | 600 | Page subtitles, major emphasis |
| `text-2xl` | 24px | 32px (1.33) | 600/700 | Page titles |
| `text-3xl` | 30px | 36px (1.2) | 700 | Hero titles (use VERY sparingly) |

## Font Weight Rules

| Weight | Token | Use |
|--------|-------|-----|
| 400 | `font-normal` | Body text, descriptions |
| 500 | `font-medium` | Labels, sidebar items, table headers |
| 600 | `font-semibold` | Titles, buttons, emphasis |
| 700 | `font-bold` | Hero titles, major headings only |

**RULE:** Never use more than 3 font weights on a single page.

## Hierarchy Patterns

### Dashboard Page
```
text-2xl font-semibold  — Page title ("Dashboard")
text-sm text-muted      — Page description
text-lg font-medium     — Section/card title
text-sm                 — Card content / table cells
text-xs text-muted      — Timestamps, metadata
```

### Settings Page
```
text-2xl font-semibold  — Page title ("Settings")
text-xl font-semibold   — Section title ("General", "Security")
text-sm font-medium     — Form label
text-sm                 — Form help text
text-base               — Form input text
```

### Table
```
text-xs font-medium uppercase tracking-wider text-muted — Column header
text-sm                 — Cell content
text-xs text-muted      — Secondary cell info
```

## Red Flags (Auto-fail)
- `text-[15px]` or any arbitrary font size = FAIL
- More than 7 distinct font sizes on the site = FAIL
- Same font size used for heading AND body text = FAIL
- `font-bold` used on body text = FAIL
- No clear visual hierarchy (can't scan in 3 seconds) = FAIL
- Different font sizes for the same role across pages = FAIL
