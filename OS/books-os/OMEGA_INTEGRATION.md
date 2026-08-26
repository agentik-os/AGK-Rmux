# Omega Integration Contract

## Registration

- ID: `books`
- Version: `1.0.0`
- Default command: `/books-os`
- Canonical skill: `alexandria`
- Position: Systems group, learning and knowledge application

## Context injection order

1. `~/.omega/agents/librarian.md`
2. `~/.omega/skills/alexandria/SKILL.md`
3. authorized local ledger records
4. the current source material
5. the current user message

## Produces

- `books.insight.confirmed` -> Context & Memory OS.

Payload:

```json
{
  "insight_id": "string",
  "source": {"title": "string", "author": "string", "location": "string"},
  "claim": "string",
  "interpretation": "string",
  "confidence": "string",
  "recorded_at": "string (ISO-8601 timestamp)"
}
```

## Consumes

- `memory.context.compiled` from Context & Memory OS when the user authorizes retrieval.

## Boundaries

Books OS never invents quotations or source locations. Private books, notes and reading history stay local. Changes to schemas or retention rules require Review & Governance approval.
