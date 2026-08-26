# Rebrand — OmegaOS → AGK-Rmux

This repo is the destination. [OmegaOS](https://github.com/agentik-os/OmegaOS)
stays the current public line until the rename is done on purpose.

## Product name

| Today | Target |
|---|---|
| OmegaOS | Agentik OS / AGK |
| `omega` CLI | keep working; `agk` is the profile/control entry |
| rmux | unchanged — execution plane |

## What must not change with the name

- Logical profiles: operator, agentik, mission, private
- Canonical IDs (never Linux usernames)
- Hermes = official brain (not a fork)
- BOT ≠ AGENT (four Discord bots = four profiles)
- Topology is configuration (`topology.yaml` wins)

## What to rename later (not this commit)

- crate names (`omega-core` → decide)
- `npx omega-os` package
- install paths that still say OmegaOS in user-facing copy
- CI badges still pointing at `agentik-os/OmegaOS`

Do not silently migrate anyone's existing `~/.omega` tree.
