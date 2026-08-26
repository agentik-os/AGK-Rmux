# Completion, Release Gates & Final Handoff

## Step completion

A step is DONE only when all configured Definition of Done checks are true.

## Slice completion

A vertical slice must prove integrated behavior, including all required layers.

Example “Experience booking” is incomplete if it lacks any required:

- access policy;
- concurrency;
- payment;
- Member UI;
- error state;
- tests;
- observability.

## Module completion

Module gate can require:

```text
all required module steps DONE
module E2E PASS
security PASS
architecture PASS
docs updated
no critical blockers
```

## Project release check

Required:

```text
all launch/P0 required steps DONE
all P0 acceptance tests PASS
security release gate PASS
AI eval release gate PASS where applicable
integration/E2E PASS
migration/release compatibility PASS
no critical unresolved blocker
```

## Final Build Report

Generate:

```text
Project
Blueprint version
Stepper version
Git release SHA
Modules complete
Slices complete
Steps complete
Weighted progress
Test summary
Security summary
AI eval summary
Known accepted risks
Open post-launch work
Release readiness
```

## Handoff to operations

The build is not operationally complete without:

- deployment procedure;
- rollback procedure;
- incident runbooks;
- monitoring dashboards;
- secrets/environment configuration;
- migration status;
- known-risk register.
