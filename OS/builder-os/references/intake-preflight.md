# Intake, Repository Preflight, and Setup

## Contents

1. Intake sequence
2. Contract validation
3. Repository cartography
4. Environment and toolchain
5. Secrets and external services
6. Baseline health
7. Bootstrap policy
8. Preflight output

## 1. Intake sequence

Run intake before editing product code:

1. resolve the project/repository target exactly;
2. load and fingerprint Blueprint and Stepper handoffs;
3. validate Stepper graph and Tracker state;
4. inspect repository instructions and existing user changes;
5. map toolchain, workspaces, services, CI, tests, environments, and releases;
6. determine capabilities and missing authorizations;
7. establish a clean or explicitly preserved base revision;
8. run non-destructive baseline checks;
9. let Stepper Planner select bootstrap work already present in the graph;
10. transition from `BUILD PREFLIGHT` only with evidence.

## 2. Contract validation

Verify:

- project identity matches across Blueprint, Stepper, and repository configuration;
- Blueprint status is exactly `BLUEPRINT COMPLETE — STEPPER READY`;
- fingerprints match frozen handoffs;
- Stepper status is execution-ready;
- every required step and dependency resolves;
- graph is acyclic;
- P0 requirements/acceptance tests are not orphaned;
- release target and manual gates are explicit;
- prohibited shortcuts and conditional Blueprint items are represented in steps or gates;
- paths referenced by Stepper are either present or intentionally created by predecessor steps.

Do not weaken validation because the repository is empty. An empty repository may be valid only when setup/bootstrap steps cover it.

## 3. Repository cartography

Record without speculative rewrites:

- root and nested repository boundaries;
- workspace/monorepo packages and dependency direction;
- language and runtime version files;
- package manager and lockfiles;
- build, dev, lint, typecheck, unit, integration, E2E, security, and eval commands;
- app/backend/worker/shared-package boundaries;
- database schemas, migrations, seeds, and generated artifacts;
- auth, payments, storage, messaging, analytics, AI, and external-service adapters;
- environment files and example templates without exposing secret values;
- CI/CD workflows, infrastructure configuration, and protected paths;
- repository instruction files and code ownership;
- docs that claim setup or architecture truth.

Use targeted search and existing manifests first. Avoid indexing vendored dependencies, build output, caches, or secret stores.

## 4. Environment and toolchain

Pin and record:

- operating system/runtime assumptions;
- compiler/interpreter versions;
- package-manager version;
- system dependencies;
- container/devcontainer behavior where present;
- service emulators or local dependencies;
- deterministic install command;
- cache strategy;
- reproducible test commands.

Prefer repository-declared versions. Installing or upgrading toolchains is a mutation that must be authorized and step-governed.

## 5. Secrets and external services

Represent capabilities, not values:

```text
stripe_test_api: available | missing | invalid | not_required
production_database_write: authorized | denied | unknown
```

Never print or persist secret values in prompts, logs, Tracker records, diffs, screenshots, command evidence, or reports. Redact environment output. Use least privilege and test/sandbox environments by default.

Treat unavailable external services as:

- mockable only if the Blueprint/Stepper explicitly permits a contract-faithful emulator;
- a blocker when real integration evidence is mandatory;
- a manual gate when human authorization or account setup is required.

## 6. Baseline health

Before changes, capture applicable baseline results:

- dependency install/lockfile consistency;
- build;
- typecheck;
- lint/format check;
- unit and fast integration tests;
- schema/migration status;
- repository security/secret scan;
- current CI status if available;
- known pre-existing failures.

Do not silently assume every baseline failure belongs to the current step. Record pre-existing failure signatures and determine whether they block execution, can be isolated, or require Stepper work.

## 7. Bootstrap policy

Bootstrap work includes repository initialization, workspace scaffolding, CI, test harness, design system, auth/data foundations, environment templates, and base observability. Execute it only through Stepper steps.

For an existing repository:

- preserve conventions and compatible dependencies;
- do not replace architecture wholesale for convenience;
- avoid mass formatting unless explicitly scoped;
- preserve user modifications and unrelated untracked files;
- request governance when the required architecture conflicts with repository reality.

## 8. Preflight output

Persist a signed/fingerprinted preflight report containing:

- input identities and hashes;
- repository base and dirty-state summary;
- topology/toolchain/environment inventory;
- baseline check results;
- available capabilities and missing permissions;
- pre-existing failures;
- active blockers/manual gates;
- selected Stepper bootstrap wave or reason execution cannot start.
