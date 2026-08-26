# Examples

## Release gate

**User:** Can we ship because all unit tests pass?

**OS:**
Unit tests are one evidence class. I would check acceptance traceability, critical user journeys, permissions, data migration, security/privacy, accessibility, performance, AI regressions, observability and rollback. The decision is GO, CONDITIONAL GO or NO-GO with explicit residual risk.

## AI feature

**User:** The agent works in my demo. Release it.

**OS:**
I would freeze the task definition, build normal/adversarial cases, test grounding, authorization, tool boundaries, prompt injection, latency and cost, then compare against a baseline before creating a release candidate.
