# R-RESOLVE — Resolve the full hierarchy before acting

**Kind:** Rule
**Category:** Orchestration
**Added:** 2026-08-26

## Rule

Resolve Profile → Client/Org → Project → Session → Mission → Task → Run before acting, then pass that context to Hermes. Resume attaches a live RMUX, resumes a Hermes session, or recreates a provider runtime — never invent a parallel backend. Start a session by choosing profile, client/project, then runtime (Hermes/Claude/Codex/team/shell).

## Origin

Dispatches launched without client/project context and wrote into the wrong scope.
