# Stepper Contract Schemas

## Module

```yaml
module_id: MOD-010
name: Experiences
purpose: Manage Experience lifecycle, eligibility, inventory and attendance.
blueprint_refs:
  - volume: 3
    sections: [384, 405, 422]
depends_on:
  - MOD-003
  - MOD-004
  - MOD-011
provides:
  - experience-domain
  - booking-capability
priority: P0
risk_level: CRITICAL
owner_role: backend
```

## Epic

```yaml
epic_id: EPIC-EXP-ELIGIBILITY
module_id: MOD-010
name: Experience Eligibility
objective: Server-authoritative eligibility decisions for Experience access.
requirements: [EXP-003, EXP-SYS-001, UX-040]
depends_on: [EPIC-MEM-ENTITLEMENTS, EPIC-TRUST-RESTRICTIONS]
```

## Vertical slice

```yaml
slice_id: SLICE-EXP-002
module_id: MOD-010
epic_id: EPIC-EXP-ELIGIBILITY
name: Eligible Member can request or book an Experience
user_outcome: Member receives a correct actionable access state and can execute allowed action.
depends_on:
  - SLICE-MEM-001
acceptance_tests:
  - SYS-AT-003
  - UX-AT-005
```

## Full Step schema

```yaml
step_id: STEP-000123
schema_version: 1

title: Implement Experience eligibility resolver
module: MOD-010
epic: EPIC-EXP-ELIGIBILITY
slice: SLICE-EXP-002

priority: P0
status: PENDING
weight: 5

risk:
  level: HIGH
  reasons:
    - authorization
    - booking access

objective:
  concise: Implement the server-authoritative Experience eligibility resolver.
  outcome: Return a complete typed access decision consumed by Member clients and AI tools.

why:
  - Prevent client-side eligibility authority.
  - Centralize Experience access policy.

blueprint_references:
  - doc: blueprint/volume-3.md
    sections: [387, 388, 498, 500]
  - doc: blueprint/volume-2.md
    sections: [126, 272]

requirements:
  - EXP-003
  - EXP-SYS-001
  - UX-040

decisions:
  - DEC-068
  - DEC-080

invariants:
  - Client never computes authoritative eligibility.
  - Restriction overrides entitlement.
  - Blocked/suspended actors fail closed.

attention:
  critical:
    - Never trust tier or eligibility state from client input.
  watch_for:
    - expired invitations
    - stale Circle membership
    - full capacity
  do_not:
    - duplicate membership logic
    - modify unrelated booking state machine

forbidden_changes:
  - membership pricing
  - Experience capacity semantics
  - public API names outside the declared change

dependencies:
  hard:
    - STEP-000095
    - STEP-000110
  soft: []

blocks:
  - STEP-000124
  - STEP-000190

preconditions:
  - Membership entitlement resolver exists.
  - Trust restriction resolver exists.
  - Circle membership accessor exists.

context_files:
  read:
    - convex/domains/experiences/types.ts
    - convex/domains/membership/policies.ts
    - convex/domains/trust/restrictions.ts
  optional:
    - docs/domain/experience.md

expected_files:
  create:
    - convex/domains/experiences/eligibility.ts
  modify:
    - convex/domains/experiences/queries.ts

implementation_prompt: |
  Implement ExperienceEligibilityService using the canonical domain contracts.
  The service must remain read-only and server-authoritative.
  Reuse Membership and Trust policies; do not duplicate them.
  Return an ExperienceAccessDecision with state, allowedActions,
  reasonCodes and outstanding requirements.
  Preserve stable reason codes for UI and AI consumers.

expected_contract:
  inputs:
    memberId: internal Member ID
    experienceId: internal Experience ID
  outputs:
    state: ExperienceAccessState
    allowedActions: list[ExperienceAction]
    reasonCodes: list[str]
    requirements: list[AccessRequirement]

edge_cases:
  - blocked Member
  - suspended Member
  - Circle-only Experience
  - tier-restricted Experience
  - expired invitation
  - full Experience
  - host approval required
  - active restriction despite Principal tier

tests_required:
  unit:
    - eligible Member returns ELIGIBLE_TO_BOOK
    - blocked Member returns NOT_ELIGIBLE
    - restriction overrides plan entitlement
  integration:
    - Circle access policy respected
    - invitation expiration respected
  security:
    - client-supplied tier cannot change result

commands:
  test:
    - pnpm test experience-eligibility
  typecheck:
    - pnpm typecheck
  lint:
    - pnpm lint

acceptance_checks:
  - type: file_exists
    path: convex/domains/experiences/eligibility.ts
  - type: command
    command: pnpm test experience-eligibility
  - type: command
    command: pnpm typecheck
  - type: grep_absent
    path: apps/mobile
    pattern: "tier ==="

acceptance_criteria:
  - Every applicable access state is represented.
  - Denials include stable reason codes.
  - Client code contains no authoritative eligibility rule.
  - Required tests pass.
  - Typecheck and lint pass.

security_checks:
  - Authorization is server-side.
  - No private trust evidence appears in result.

observability:
  domain_events: []
  analytics:
    - experience_eligibility_denied
  logs:
    - policy reason code only
  pii_policy:
    - no sensitive Member context in logs

documentation:
  update:
    - docs/domain/experience.md

review_roles:
  - backend
  - security
  - architecture

locks:
  - domain: experiences

provides:
  - ExperienceEligibilityService
  - ExperienceAccessDecision

rollback:
  strategy: Revert resolver wiring and return previous non-actionable Experience projection.

definition_of_done:
  - implementation_complete
  - required_tests_pass
  - typecheck_pass
  - lint_pass
  - security_review_pass
  - architecture_review_pass
  - documentation_updated
  - acceptance_pass
```

## Step rule

A generated step that lacks sufficient information to execute independently is invalid and must be refined before entering READY state.
