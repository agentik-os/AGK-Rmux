# Stax design system — "WhitePaper"

The design is part of Stax, not a downstream choice. A conversion adopts this look by
default (the scaffold ships `tokens.css` + `stax-ui.css`); you swap only the accent.
Source of truth: `~/.omega/repos/stax/frameword/apps/crm-specimen/src/{tokens.css,styles.css}`.

## Foundations

- **Palette:** shadcn-compatible **oklch** tokens (`--background --foreground --card
  --border --muted-foreground --secondary --accent --accent-soft --accent-hover --ring`),
  with a parallel **dark mode** (`.dark`, `[data-theme="dark"]`). An accent ramp
  (`--accent-2/-3/-4`) is mixed from the one accent — change `--accent`, the system follows.
- **Type:** **Inter** (UI sans), **Newsreader** (display serif — panel titles + big
  numerals), **Geist Mono** (labels, eyebrows, data). Numbers are tabular/mono everywhere.
- **Radius:** 14px panels/cards, 7–8px controls, pill for tags. **Shadow:** soft neutral
  ramp (`--shadow-2xs` on panels, `--shadow-md` on the focused column).

## The chrome (what makes it read as Stax)

- **Stage = a dot-grid canvas** (`radial-gradient` dots on an 18px grid), horizontal rail,
  14px gap, 18px padding.
- **Panel = a card** — `var(--card)` on the dotted stage, 1px border, 14px radius, soft
  shadow, `panelIn` slide. A **reference** (detached pin) panel gets an accent-tinted
  border; the **focused/newest** column carries more light (`--shadow-md`).
- **Panel bar (56px):** a **mono, uppercase, letter-spaced eyebrow** as the label (not a
  big title), small 24–26px ghost controls, a mono `PIN`/★ toggle that turns accent when on.
- **Panel body:** the big heading inside is **Newsreader serif** (`--font-serif`), ~27px,
  tight tracking; supporting text in Inter; stats/labels in Geist Mono.
- **Footer = the one action zone:** recessed (`color-mix` tint), a single **accent CTA**
  in mono uppercase (`.foot-cta`), optional ghost/secondary variants. Never floating buttons.
- **Breadcrumb (34px):** mono, uppercase, letter-spaced; the current crumb is accent.
- **Sidebar (240px):** its own recessed card surface, mono section labels, quiet row items
  that fill with `--secondary` on hover/active.
- **Thin themed scrollbars** everywhere content scrolls.

## How to apply it

1. Import `tokens.css` then `stax-ui.css` (the scaffold wires this in `stax.css`).
2. Use the shell class names the scaffold emits (`stax-sidebar`, `stax-topbar`,
   `stax-rail`, `stax-panel`, `stax-panel-head`, `stax-panel-title`, `stax-panel-body`,
   `stax-panel-foot`, `stax-crumb`, `stax-ico`) — `stax-ui.css` styles them to WhitePaper.
3. **Rebrand = one line:** override `--accent` (and optionally `--accent-soft/-hover`) in a
   `:root` after the imports. Everything else follows.
4. Inside panel bodies, lead with a **serif** title, label meta in **mono**, keep numbers
   tabular. Prefer dividers/rhythm over decorative cards (the specimen's rule).
5. Reusing an app's existing components? Bridge its token names to the WhitePaper ones
   (e.g. `--ink: var(--foreground); --line: var(--border); --surface: var(--card)`), so the
   whole app inherits the palette + dark mode without a rewrite.

## Anti-patterns (a conversion that "looks generic" failed Phase 4)

- A flat sidebar + plain header with no dot-grid, no serif titles, no mono eyebrows.
- Big bold sans titles instead of Newsreader serif.
- Square/borderless panels with no card treatment or shadow.
- Floating action buttons instead of a recessed footer CTA.
- Hardcoded colours/spacing inside domain panels instead of tokens.
