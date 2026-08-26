# Git, Worktrees, and Integration Protocol

## Contents

1. Safety first
2. Base and dirty state
3. Branch/worktree model
4. Commit contract
5. Integration
6. Conflict resolution
7. Remote operations
8. Rollback

## 1. Safety first

Preserve user work. Never use destructive resets, broad checkouts, mass deletion, or history rewriting to simplify execution. Do not stage unrelated files. Resolve exact targets before mutations.

## 2. Base and dirty state

Record:

- current branch and HEAD;
- intended integration branch and base revision;
- tracked modifications, staged changes, untracked files, and submodule state;
- which changes belong to Builder attempts versus pre-existing user work;
- repository instructions and protected paths.

If unrelated changes overlap the step, block or isolate safely. Do not overwrite or silently incorporate them.

## 3. Branch/worktree model

Default:

```text
verified base revision
→ isolated worktree/branch per parallel step or tightly coupled slice
→ implementation + local verification
→ evidence-linked commit
→ controlled integration queue
→ post-integration verification
```

Use deterministic names such as `builder/STEP-000123-attempt-02`. Store worktree path and base/head revisions in canonical state. Enforce resource locks in addition to worktree isolation; worktrees do not prevent semantic conflicts.

Run sequentially on one branch only when Stepper declares it safe and user work remains protected.

## 4. Commit contract

Create a commit only when the step's required pre-integration checks pass. Use a traceable subject such as:

```text
STEP-000123: Implement Experience eligibility resolver
```

Record step ID, attempt ID, Blueprint/Stepper fingerprints, checks, and documentation impact in machine-readable evidence or commit metadata. Do not include secrets, raw private logs, or excessive generated output.

Keep commits scoped. Separate generated lockfile/schema/client changes only when project policy requires it; otherwise include required generated artifacts in the same verified step.

## 5. Integration

Before integration:

1. revalidate successful attempt evidence and head digest;
2. confirm integration target and policy;
3. check newer base changes for contract impact;
4. acquire integration/affected-domain locks;
5. refresh/rebase/cherry-pick/merge only as policy permits;
6. resolve conflicts using both step contracts and current Blueprint decisions;
7. run post-integration checks on the resulting revision;
8. record integrated SHA and release-lock implications;
9. release locks after durable state update.

Do not mark `DONE` because a merge command succeeded.

## 6. Conflict resolution

Classify conflicts:

- textual and contract-neutral: resolve, inspect diff, reverify;
- semantic but covered by both contracts: reconcile and run combined acceptance;
- architecture/product conflict: raise decision/change request;
- migration/order conflict: stop until safe deploy/recovery order is defined;
- overlap with user-owned changes: ask or isolate; never choose ownership silently.

## 7. Remote operations

Treat push, pull request, merge, deployment trigger, release/tag creation, and branch protection changes as external mutations requiring the task's authorization and configured capability. Prefer non-interactive commands and exact targets. Never force-push unless the user explicitly authorizes a narrowly defined history rewrite.

## 8. Rollback

Distinguish:

- discard of an uncommitted Builder-only attempt;
- revert of an isolated commit;
- compensating change after integration;
- data migration rollback/forward recovery;
- production rollback, which belongs to explicit Ship/operations authority.

Preserve evidence of failed/reverted attempts. A rollback does not erase history or automatically restore external side effects.
