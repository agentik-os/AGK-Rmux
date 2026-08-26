# Install into Omega OS

## Required
Copy this directory under:
`omega-os/os/alignment-coach/`

## Register
Register:
- `config/os.yaml`
- `system/SYSTEM_PROMPT.md`
- all `skills/*.md`
- all `agents/*.md`

## Suggested slash command
`/coach` → Alignment Coach OS

Aliases:
`/align`, `/council`, `/stoic`, `/tao`, `/rohn`, `/manifest`, `/quantum`

## Context injection order
1. SYSTEM_PROMPT
2. PRINCIPLES
3. user-authorized identity/values memory
4. relevant recent context
5. one or more selected knowledge packs
6. selected skill
7. current user message

Do not inject the entire library every turn.

## Handoff
Alignment Coach OS may emit:
- `ROUTE:MINDSET`
- `ROUTE:HABITS`
- `ROUTE:EXECUTION`
- `ROUTE:STRATEGY`
- `ROUTE:MEMORY_WRITE_REQUEST`
