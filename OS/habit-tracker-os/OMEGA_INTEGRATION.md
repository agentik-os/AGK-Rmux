# Omega Integration Contract

## Registration
- ID: `habit-tracker`
- Version: `1.0.0`
- Default command: `/habits`
- Position: Personal Stack: recurring behavior contracts and check-in evidence

## Context injection order
1. `SKILL.md` operating contract plus `references/system-prompt.md`
2. relevant authorized memory
3. current records and evidence
4. selected protocol (`references/conversation-protocols.md`, `references/behavior-science.md`)
5. current user message

Never inject the entire knowledge library by default.

## Naming fix
This OS previously described integrating with "Mindset {OS}, Life {OS}, or Omega {OS}." There is no `life-os` in this suite; that reference is removed from `SKILL.md`. The real suite integrations are Mindset OS (behavior contracts) and Context & Memory OS (canonical observation storage). Any external life-tracking app is an explicit out-of-suite dependency, never an implied member.

## Handoffs
- Mindset OS supplies behavior contracts to track.
- Health & Energy OS supplies agreed routines.
- Context & Memory OS stores canonical check-in observations; this OS keeps only a local indexed projection.

## Event types
- habit.observation.recorded
- habit.review.completed

## Produces (pipeline wiring)
- `habit.review.completed` -> consumed by Mindset OS (closes the `Mindset intent -> Habit contract -> Daily evidence -> Pattern/review -> Mindset reflection` loop).
- `habit.observation.recorded` is a local domain event; each confirmed observation is staged canonically via `memory.record.staged` (see State classification), it is not itself consumed by name outside this OS.

## Consumes
- `mindset.behavior_contract.created` from Mindset OS.
- `handoff.habits.created` from Health & Energy OS.

## State classification
- Canonical (routes through Context & Memory OS): confirmed check-in observations, weekly/monthly reviews.
- Local operational state (`scripts/habit_os.py`): a local indexed projection of observations for fast streak/analytics lookups, never the source of truth.
- Read: `memory.context.compiled`. Write: `memory.record.staged` for confirmed observations; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.
