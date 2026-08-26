# R-TOPO — Profiles are logical — topology is configuration

**Kind:** Rule
**Category:** Universal
**Added:** 2026-08-26

## Rule

Operator / Agentik / Mission / Private are logical profiles, not Linux users. Topology.yaml (or the user-level equivalent) wins over auto-detect. Never hardcode usernames or assume single-user vs four users vs containers. Resolve paths through the Topology Manager. We ship an opinionated default, not a locked architecture.

## Origin

Domain logic was written against one VPS user layout and could not run on a simple agk@host install.
