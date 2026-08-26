# Omega Integration Contract

## Registration
- ID: `context-memory`
- Version: `1.0.0`
- Default command: `/memory`
- Position: Omega Core: canonical shared context layer for every OS

## Context injection order
1. `system/SYSTEM_PROMPT.md`
2. `system/PRINCIPLES.md`
3. relevant authorized memory
4. current records and evidence
5. selected specialist agent(s)
6. selected skill or protocol
7. current user message

Never inject the entire knowledge library by default.

## Handoffs
- All OSs request context packs rather than raw unrestricted memory.
- Librarian outputs enter as sourced knowledge records.
- Review & Governance OS audits decisions, permissions and stale records.
- Strategy & Portfolio OS receives versioned project/goal snapshots.

## Event types
- memory.source.ingested
- memory.record.staged
- memory.record.verified
- memory.record.superseded
- memory.context.compiled
- memory.contradiction.opened
- memory.correction.applied
- memory.deletion.completed
- memory.context.snapshot.created

## Produces (pipeline wiring)
- `memory.context.compiled` -> consumed by every OS as its context-injection input (see each OS's own "State classification" section).
- `memory.record.verified` -> consumed by the producing OS, confirming a canonical write.
- `memory.context.snapshot.created` -> consumed by Strategy & Portfolio OS, closing the Review -> Context -> Strategy learning loop.

## Consumes
- `memory.record.staged` from every OS in the suite (universal write path; this is the canonical persistence layer).
- `review.learning.pack.created` from Review & Governance OS.
- `books.insight.confirmed` from Books OS with source, location, claim, interpretation and confidence kept distinct.

## Canonical-state contract (suite-wide)
Every OS in the suite classifies its local state as one of: canonical (must route through Context & Memory via `memory.record.staged` / `memory.record.verified`), projection/cache (a local indexed view of canonical data, never a competing source of truth), or temporary (session-only, never persisted). See each OS's own "State classification" section for its specific split. No OS writes directly to an independent canonical store.

## State classification
- Canonical (this OS IS the canonical layer): all `memory.record.verified` records.
- Local operational state: in-flight ingestion/compilation working sets, unresolved contradictions.
- Read/Write: this OS is both the read and write authority; other OSes never bypass it for canonical data.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.
