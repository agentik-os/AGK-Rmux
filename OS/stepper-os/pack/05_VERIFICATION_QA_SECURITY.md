# Verification, QA & Security Gates

## Core principle

The coding agent is not allowed to certify its own work.

Stepper Verifier independently checks reality.

## Verification pipeline

```text
agent implementation
↓
expected files/artifacts
↓
unit/domain tests
↓
integration/contract tests
↓
typecheck
↓
lint
↓
security checks
↓
architecture checks
↓
UX/AI checks when applicable
↓
acceptance predicates
↓
DONE
```

## Risk-based gates

### LOW

- focused tests;
- typecheck/lint.

### MEDIUM

- unit;
- integration;
- typecheck;
- lint;
- architecture sanity.

### HIGH

- unit;
- domain/integration;
- security tests;
- relevant E2E;
- reviewer gate.

### CRITICAL

- all applicable automated checks;
- security review;
- architecture review;
- rollback verification;
- release/canary constraints where applicable.

## Acceptance check types

Support at least:

```text
file_exists
file_absent
grep_present
grep_absent
command
pytest
js_test
json_schema
custom_python
artifact_exists
review_gate
```

## Security checks

Depending on step:

- authentication bypass;
- authorization bypass;
- cross-tenant access;
- secret leakage;
- injection;
- unsafe shell invocation;
- data exposure;
- client-authority mistakes;
- AI tool over-permission.

## Architecture checks

Review for:

- duplicated business logic;
- forbidden imports;
- domain boundary violations;
- direct DB mutation bypassing domain service;
- vendor leakage across abstractions;
- new undocumented architecture decision.

## UX checks

UI steps must include applicable:

- loading state;
- empty state;
- error state;
- permission state;
- accessibility;
- design-system usage;
- responsive/safe-area behavior;
- no accidental generic SaaS regression.

## AI checks

AI steps must verify:

- tool schema;
- tool authorization;
- context minimization;
- uncertainty preservation;
- execution truth;
- prompt injection resistance;
- eval suite;
- cost/latency instrumentation.

## Repair loop

A failed check should produce concise machine-readable evidence.

Example:

```json
{
  "check": "command",
  "command": "pnpm test experience-eligibility",
  "exit_code": 1,
  "summary": "2 tests failed",
  "evidence": ["blocked member expected NOT_ELIGIBLE..."]
}
```

Repair prompt includes original contract + failures, not unrelated history.
