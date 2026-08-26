# R-ID — Route by canonical IDs, never paths or usernames

**Kind:** Rule
**Category:** Orchestration
**Added:** 2026-08-26

## Rule

Every object has a stable id: profile_id, client_id, project_id, session_id, mission_id, task_id, run_id, agent_id, agent_run_id, runtime_id, artifact_id. Paths and Linux users are derived deployment metadata. Migration must preserve IDs.

## Origin

Routing by folder name or unix user broke sessions when topology changed.
