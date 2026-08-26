# Omega Integration Contract

## Registration
- ID: `storyteller`
- Version: `1.0.0`
- Default command: `/story`
- Aliases: `/mine`, `/interview`, `/deepen`, `/shape`, `/write`, `/adapt`, `/truthcheck`, `/score`, `/rehearse`, `/storybank`
- Position: Commercial Stack (supporting): narrative craft, upstream of Content OS packaging/distribution

## Context injection order
1. `SKILL.md` operating contract
2. relevant authorized memory / prior story objects
3. current records and evidence (`references/*.md`)
4. selected mode (coach / interview / deepen / shape / write / adapt / truthcheck / score / rehearse / storybank)
5. current user message

Never inject the entire knowledge library by default.

## Ownership boundary: Storyteller vs Content
Storyteller owns narrative truth, story structure, voice, consent and story objects. It does NOT own editorial strategy, packaging, channel adaptation, publishing or content analytics - that is Content OS.

## Handoffs
- Content OS receives deepened story objects ready for packaging and adaptation.
- Content OS returns performance feedback for story-object learning only, never for publishing decisions.

## Event types
- story.ready_for_adaptation
- story.truth_verified

## Produces (pipeline wiring)
- `story.ready_for_adaptation` -> consumed by Content OS.

## Consumes
- `content.performance.feedback` from Content OS, for story-object learning only.

## State classification
- Canonical (routes through Context & Memory OS): confirmed story objects, truth-checked evidence, consent records.
- Local operational state: in-progress interviews, draft shapes/adaptations.
- Read: `memory.context.compiled`. Write: `memory.record.staged` for confirmed story objects; Context & Memory OS returns `memory.record.verified`.

## Change control
Changes to boundaries, schemas or quality gates require Review & Governance OS approval in production.
