# Git, Worktrees & Parallel Execution

## Objectives

- isolate agent changes;
- preserve reproducibility;
- prevent parallel corruption;
- make every completed step auditable.

## Default step flow

```text
main/base commit
↓
STEP branch/worktree
↓
implementation
↓
verification
↓
review
↓
STEP commit
↓
merge/integrate
↓
post-merge checks
```

## Branch naming

```text
step/STEP-000123-experience-eligibility
```

## Commit naming

```text
STEP-000123: Implement Experience eligibility resolver
```

## Worktrees

Recommended path:

```text
.stepper/worktrees/STEP-000123/
```

Parallel agents must use separate worktrees when changing files.

## Locking

Before scheduling:

```text
acquire declared locks
↓
create worktree
↓
execute
```

After integration:

```text
release locks
↓
unlock dependent steps
```

## Merge conflict policy

A conflict is not automatically resolved by taking either side.

The integration agent/reviewer must:

1. identify the contracts implemented by both steps;
2. preserve both when compatible;
3. rerun tests for both steps;
4. fail integration if architecture/behavior conflicts.

## Dirty working tree

By default, do not begin a new step from an untracked dirty base.

Existing unrelated user changes must be preserved, not discarded.

## Failed work

Do not merge failed work.

The failed worktree may remain temporarily for repair/diagnostics.
