# R-AGK — Four AGK scopes — switch is a full context change

**Kind:** Rule
**Category:** Universal
**Added:** 2026-08-26

## Rule

Four AGK scopes exist (operator, agentik, mission, private); `agk switch` changes the complete context (Hermes profile, workspace, secrets, RMUX namespace, OS, skills, MCP, credentials) — work only in `$HOME/workspace/$OMEGA_SCOPE`. Operator operates the machine; Agentik the organization; Mission other organizations; Private the self. Mission/Private YOLO requires bwrap sibling-hide. `omega scope` remains file-claim locks, not AGK tenancy.

## Origin

A UI label change without workspace/secrets/Hermes profile switch leaked client work into private.
