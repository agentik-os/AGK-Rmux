# R-EVENT — Surfaces subscribe to events — they do not scrape terminals

**Kind:** Rule
**Category:** Reporting
**Added:** 2026-08-26

## Rule

Publish session.created, mission.created, run.started, agent.started|waiting|completed, artifact.created, approval.required, run.failed|completed. Desktop, Web, and Discord read the event bus. Never mirror raw runtime logs into Discord.

## Origin

UIs grepped pane text and invented state that Hermes and RMUX already knew.
