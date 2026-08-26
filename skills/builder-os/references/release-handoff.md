# Release Readiness and Final Handoff

## Contents

1. Release boundary
2. Release-check algorithm
3. Final report
4. Operations handoff
5. Deployment distinction
6. Completion and blocked states

## 1. Release boundary

Builder proves an integrated release candidate satisfies the approved Blueprint and Stepper release target. Builder does not infer that production is live. Shipping requires explicit deployment authority and evidence.

Freeze the candidate revision during final release checks. If it changes, invalidate affected evidence and rerun the required matrix.

## 2. Release-check algorithm

1. verify Blueprint and Stepper fingerprints;
2. verify the candidate repository revision and artifact digests;
3. require all launch/P0 steps `DONE` and no stale required step;
4. require all Stepper module/slice/project gates;
5. evaluate Builder gates BG01–BG20 on the candidate revision;
6. verify P0 requirement/acceptance traceability to code/tests/evidence;
7. verify no critical blocker/finding/follow-up remains;
8. verify conditional noncritical gates have owners and pre-use validation boundaries;
9. verify deployment, migration, rollback, monitoring, secrets/environment, runbooks, accepted risks, and post-launch register;
10. create immutable final report/handoff with checksum.

Any required failed or unevaluated gate prevents terminal success.

## 3. Final report

Produce human-readable Markdown and machine-readable JSON containing:

- project, repository, Blueprint and Stepper versions/checksums;
- Builder runtime/schema version;
- release target, candidate revision, artifact digests;
- module/slice/step totals and weighted progress;
- test/build/typecheck/lint/coverage summaries;
- integration/E2E/acceptance evidence;
- security/privacy/abuse and architecture reviews;
- data/migration and compatibility status;
- UX/accessibility/visual evidence;
- AI eval/tool/safety status when applicable;
- performance/reliability/observability status;
- documentation and runbook status;
- accepted risks and owners;
- open post-launch work;
- deployment/rollback/monitoring readiness;
- gate table BG01–BG20;
- final status and handoff checksum.

## 4. Operations handoff

Require applicable:

- environment inventory and responsibility map;
- secret/configuration names and acquisition/rotation procedures, never values;
- deployment order, migration sequence, health checks, canary/rollback thresholds;
- rollback or forward-recovery procedure;
- dashboards, alerts, SLOs, logs/traces, and ownership;
- backup/restore and incident runbooks;
- external-provider dependencies and degraded-mode behavior;
- data/privacy/security operational obligations;
- known risks and first post-release checks;
- support/escalation and post-launch verification schedule.

## 5. Deployment distinction

Use these truths separately:

```text
BUILD COMPLETE — RELEASE READY
DEPLOYMENT AUTHORIZED
DEPLOYMENT EXECUTED
PRODUCTION VERIFICATION PASS
OPERATIONAL ACCEPTANCE PASS
```

Never collapse them into “done.” If Stepper explicitly contains authorized deployment steps, record those outcomes, but preserve the distinct state transitions.

## 6. Completion and blocked states

Allowed Builder project statuses:

- `BUILD PREFLIGHT`
- `BUILD IN PROGRESS`
- `BUILD BLOCKED`
- `BUILD PAUSED`
- `BUILD COMPLETE — RELEASE READY`

For `BUILD BLOCKED`, report exact blockers, evidence, owner/decision needed, impact, safe independent work, and resume condition. For `BUILD PAUSED`, checkpoint exact next action and active-resource disposition.
