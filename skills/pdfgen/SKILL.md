---
name: pdfgen
description: Generate polished OmegaOS PDF deliverables with the canonical branded renderer and optionally send them to the operator on Telegram. Use whenever the user explicitly asks for a PDF, a printable report, a whitepaper, an audit PDF, a marketing PDF, or a document that must be delivered as a PDF.
---

# OmegaOS PDF generator

Use `omega pdf`. Do not substitute ReportLab, LaTeX, browser print scripts, or another PDF library.

## Workflow

1. Choose the template:
   - `whitepaper` for long-form reports with sections and a table of contents.
   - `audit` for a score, domains, findings, evidence, and an action plan.
   - `marketing` for personas, KPIs, channels, and recommendations.
   - `doc` for an editorial Markdown document.
2. Read `~/.omega/skills/pdfgen/lib/schemas.ts` when constructing custom JSON. Use only fields accepted by the selected template.
3. Keep the source JSON in the project deliverable directory. Exclude credentials, tokens, private keys, and hidden reasoning.
4. Validate the source with `jq empty <data.json>`.
5. Render with:

   ```bash
   omega pdf --template=<whitepaper|audit|marketing|doc> \
     --data=<data.json> \
     --out=<report.pdf> \
     --send \
     --caption="<short Telegram caption>"
   ```

   Omit `--send` only when the user explicitly declines Telegram delivery.
6. Treat a render error or Telegram error as a failure. Do not claim delivery from a local file alone.
7. Verify the result with `file`, `pdfinfo`, and `pdftotext`. Inspect representative pages as images when the report is visual or long.
8. Report the output path, byte size, page count, and Telegram delivery result.

## Quality gates

- Use a reader-facing title, date, author, document ID, and OmegaOS brand.
- Cite load-bearing audit or research claims in the PDF.
- Keep every requested section in the source data and confirm it survives text extraction.
- Avoid em and en dash punctuation in user-facing copy.
- Never include secrets or direct personal identifiers in the PDF or Telegram caption.
