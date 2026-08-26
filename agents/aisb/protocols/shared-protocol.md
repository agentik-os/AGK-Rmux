# AISB shared protocol

Shipped with OmegaOS (`~/.omega/agents/aisb/protocols/shared-protocol.md`).
This is the only shared-protocol path. Do not look under `~/.claude/agents/`.

## Doctrine (current)

Laws L0–L6 outrank this file. Named rules replace retired R-18→R-35 numbers:

- **R-RUBRIC** — Done Criteria + Verify Command before any worker spawn
- **R-VERIFY** — a claim is false until a command or capture can fail it
- **R-CITE** — evidence is `file:line`, a pane capture, or a command + exit code
- **R-SCOPE** — one writer per file
- **R-GRAPH** — shape work as a graph; spawn workers, do not role-play them
- **R-BUDGET** — stop or escalate when the mission budget is spent
- **R-TEST** — run the real test layer

Never cite R-18, R-19, R-21, R-28, or R-35.

## Report shape

Every agent reports back in this form:

```
BRIEF: [1-line summary]
STATUS: DONE | WORKING | BLOCKED
CONFIDENCE: [0.0-1.0]
ARTIFACTS: [files created/modified]
```

Escalate when CONFIDENCE < 0.5 (research first), BLOCKED > 2 turns (re-route),
or the blast radius is irreversible (operator).

## LMC audits

SERAPH-grade audits follow `agents/aisb/lmc-protocol.md` (Lead–Manager–Checker).

## Nerve (optional)

When AISB Nerve is configured (`~/.omega/config/nerve.json`):

- `aisb-nerve check` before a dispatch
- `aisb-nerve decision log` for routing decisions
- `aisb-nerve agent register` when spawning
- `aisb-nerve progress emit` on long work
