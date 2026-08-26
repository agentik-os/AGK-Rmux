# Agentik PDF

Unified PDF generation system. **One stack, four templates, three themes.**

Replaces every legacy PDF generator on this VPS (`@react-pdf/renderer`, ReportLab, Chrome+HTML, LaTeX).

```
Next.js 15 + Tailwind v4 + CSS Paged Media + Playwright Chromium
```

## Quick start

```bash
# Demo all four templates
pdfgen --smoke

# Render a specific template with demo data
pdfgen --template=audit --demo --out=/tmp/audit.pdf

# Render from a JSON data file
pdfgen --template=whitepaper --data=./whitepaper.json --out=./out.pdf

# Send the result via Telegram through the allow-listed OmegaOS configuration
omega pdf --template=marketing --data=./mkt.json --send

# Raw pdfgen requires an explicit allow-listed destination
OMEGA_TELEGRAM_CHAT_ID=<chat-id> pdfgen --template=marketing --data=./mkt.json --send-dm
pdfgen --template=audit --data=./audit.json --send-topic=32 --send-group=-1003587170167
```

## Templates

| Template | Purpose |
|---|---|
| `whitepaper` | Long-form, cover + TOC + sections |
| `audit` | Score gauge, KPIs, findings, action plan |
| `marketing` | Personas, charts, recommendations |
| `doc` | Generic markdown → editorial PDF |

## Themes

| Theme | Look |
|---|---|
| `agentik` | Cream + electric blue (default) |

## Design language

- **Type**: Fraunces (display serif, variable) + Inter (sans, variable) + JetBrains Mono (mono, variable)
- **Page**: A4, hand-tuned 14mm gutters, true CSS Paged Media break control
- **Color**: Monochrome with a single accent stroke
- **Charts**: Recharts, no animation, monochrome bars
- **Numbers**: Tabular lining figures (`font-variant-numeric: tabular-nums`)

## Project layout

```
agentik-pdf/
├── app/
│   ├── layout.tsx              # imports globals + themes
│   ├── globals.css             # @font-face + @page + tokens + page primitive
│   ├── page.tsx                # dev index
│   └── render/[template]/      # dynamic route, props via search params
├── components/
│   ├── primitives/             # Cover, TOC, SectionTitle, KPI, ScoreGauge, BarChart, …
│   └── templates/              # 1 file per template
├── themes/
│   └── agentik.css
├── lib/
│   ├── schemas.ts              # strict input shapes
│   ├── samples.ts              # demo data for each template
│   ├── render.ts               # Playwright page.pdf() wrapper
│   └── server.ts               # Next start/stop helpers
├── bin/
│   ├── pdfgen.ts               # CLI
│   └── smoke.ts                # render all four with demo data
└── public/fonts/               # self-hosted variable fonts
```

## Adding a template

1. Add the schema in `lib/schemas.ts`.
2. Build the component under `components/templates/`.
3. Wire it into `app/render/[template]/page.tsx`.
4. Add a sample to `lib/samples.ts`.

## Adding a theme

Create `themes/<name>.css` defining the 9 design tokens on `.theme-<name>`, then import it from `app/layout.tsx`.

---

Built 2026-05-14 to replace the legacy PDF stack.
