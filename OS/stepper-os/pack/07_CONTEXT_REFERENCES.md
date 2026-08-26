# Context Compiler & References

## Problem

Large Blueprints may contain millions of tokens. A coding agent should not receive all of them for every atomic step.

## Context compiler

For each step compile:

```text
STEP CONTRACT
+
exact Blueprint sections
+
referenced decisions
+
referenced invariants
+
relevant source files
+
dependency artifacts
+
recent failure evidence
+
repository conventions
```

## Reference format

Recommended:

```yaml
blueprint_references:
  - doc: blueprint/03-system-architecture.md
    sections: [387, 388, 498, 500]
  - doc: blueprint/02-ux.md
    sections: [126, 272]
```

## Context selection rules

### Always include

- objective;
- requirements;
- decisions;
- invariants;
- Definition of Done;
- attention/forbidden changes.

### Include when relevant

- domain files;
- screen contracts;
- previous step output;
- schema;
- tests;
- ADRs.

### Do not include automatically

- unrelated product areas;
- full private production data;
- secrets;
- huge logs;
- previous agent chain-of-thought.

## Dependency artifacts

Completed steps should expose concise artifacts such as:

```yaml
provides:
  - MembershipStatus
  - memberships table
  - getEffectiveEntitlements()
```

Dependent step compilation should resolve these against current repository state.

## Prompt size strategy

If context becomes large:

1. keep contract verbatim;
2. keep invariants/decisions verbatim;
3. summarize long documentation with references;
4. include exact source snippets only where needed;
5. retain pointers to full files.

The agent must be able to inspect the repository during execution.
