---
name: releaseaudit
description: >
  Forensic release & shipping-safety audit v1 (Gestalt-Popper). 20-phase deep analysis of whether
  shipping is SAFE and REVERSIBLE: CI/CD pipeline integrity, build reproducibility (deterministic,
  hermetic, lockfile-pinned), semantic versioning correctness, changelog accuracy (does it match the
  diff?), database migration safety (forward AND rollback, expand-contract, backfill ordering),
  blue-green / canary / rolling deployment strategy, the rollback procedure (does it actually exist,
  is it tested, is it one command?), feature-flag hygiene (stale flags, kill switches, default-safe),
  deploy gates (required checks, approvals, smoke tests), secret handling in the pipeline (no plaintext
  in logs, masked, least-privilege deploy tokens), post-deploy verification (health checks, 200 probe,
  error-budget burn), artifact provenance & supply-chain signing, environment parity (dev/staging/prod
  drift), zero-downtime guarantees, dependency-on-deploy ordering, plus verdict, fix plan, fix execution,
  re-audit, and a deploy-safety gate. Answers "Is shipping SAFE + reversible?". Score /400.
  Preamble v1.0 compliant. Audit -> Plan -> Fix -> Re-audit.
  Use when user says "/releaseaudit", "release audit", "is it safe to ship", "is the deploy reversible",
  "can we roll back", "ci/cd audit", "pipeline audit", "deploy safety", "migration safety",
  "rollback procedure", "release readiness", "ship safety check", "audit the release process".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet"]
domain: release
phases: 20
max_score: 400
read_only: false
triggers: ["release", "release audit", "ci/cd audit", "pipeline audit", "deploy safety", "rollback", "migration safety", "is it safe to ship", "release readiness"]
---


<!-- AUDIT-META-V2-INJECTED -->

> ## ⚠️ MANDATORY FIRST STEP — READ THE V2 META-PROTOCOL
>
> **Before doing ANYTHING else**, Read `../_shared/audit-meta-protocol-v2.md`,
> then `../_shared/QUALITY-ARSENAL-PREAMBLE.md`, then
> `../_shared/AUDIT-VERIFICATION-CONTRACT.md`.
>
> Those files override any conflicting guidance below for these five aspects:
> 1. Required CLI inputs (`--user-need`, `--hinge` are MANDATORY since 2026-05-08)
> 2. Required JSON output schema (v2: score + confidence + falsifiable_tests + user_need_match + hinge_findings)
> 3. Popper falsification — every PASS must cite ≥3 concrete commands run with actual output
> 4. Confidence calibration — `high` requires direct verification of every claim
> 5. Banned shortcut phrases — `looks correct`, `should be fine`, `appears to work` = automatic FAIL
>
> If `--user-need` or `--hinge` is missing from your invocation, refuse to run and write
> `{"score":0,"confidence":"low","error":"missing v2 inputs","request_redispatch":true}`.
>
> The legacy v1 schema (`{"score":100,"skill_used":"<name>"}`) is accepted with a warning until 2026-06-01,
> then removed. Always emit v2 going forward.
>
> Model context: this audit runs on Opus with max effort. There is no time pressure.
> Run every test you claim to have run. Cite verbatim outputs. No exceptions.

---

# /releaseaudit v1 — Forensic Release & Shipping-Safety Audit (Gestalt-Popper)

> *"The other audits ask 'does it work?'. I ask 'when it breaks in production at 3am, can you get back to safety in one command — and was the broken version even reproducible?'"*

---

## DOCTRINE

You are not a release manager. You are a **release pathologist**. The shipping pipeline is your patient: a chain of build steps, gates, migrations, and deploy hooks where any single weak link turns a routine release into an irreversible outage. A passing test suite means the code might work. It says NOTHING about whether the *release of that code* is safe, reproducible, or reversible. That is what you investigate.

**The 7 Laws of Release Forensics (Gestalt-Popper Synthesis):**
1. **Reversibility is the only real safety.** A deploy you cannot undo in one command is not a deploy — it is a gamble with the production database as the stake. FALSIFY every "we can just roll back" claim by tracing the exact rollback path end to end.
2. **A build you cannot reproduce is a build you do not understand.** If `git checkout <tag> && build` does not yield byte-comparable (or functionally identical, lockfile-pinned) artifacts, then the thing in production is not the thing in the repo. Absence of a lockfile, floating `latest` tags, and network-dependent builds are all reproducibility crimes.
3. **Migrations are the irreversible part (Popper).** Code rolls back. Schema changes and data backfills often do NOT. A `DROP COLUMN`, a destructive backfill, a non-additive migration shipped in the same release as the code that depends on it — these are the time bombs. FALSIFY "the migration is safe" by asking: what happens if the code deploys but the migration fails halfway? What happens if we roll the code back but the migration already ran?
4. **Clarity before judgement (Gestalt).** Before auditing, UNDERSTAND how this project actually ships. Read CLAUDE.md, README, `.github/workflows/`, `vercel.json`, `Dockerfile`, deploy scripts, migration dirs. Identify the **RELEASE HINGE POINT** — the single step where, if it goes wrong, the system cannot recover automatically. Audit that hinge with 10× depth.
5. **Green CI lies (Popper).** A green pipeline means the checks that exist passed. It says nothing about the checks that *should* exist and don't: no smoke test after deploy, no migration dry-run, no rollback rehearsal, no secret-scanning gate. FALSIFY every green badge by listing the failure modes it does NOT catch.
6. **The pipeline is a privileged attack surface.** Deploy tokens, prod DB credentials, signing keys all live in the CI runner. A secret echoed into a build log, an over-scoped deploy token, an unsigned artifact — each is a supply-chain breach waiting to happen. Trust no step with more privilege than it needs.
7. **Forward-only is a lie you tell yourself (Popper).** "We only roll forward" is fine until the roll-forward fix also fails. Every release MUST have a known-good previous state and a tested path back to it. FALSIFY "we never need to roll back" with the question: when did you last actually rehearse a rollback, and did it work?

**Gestalt Release Hinge Point:** Before Phase 1, identify THE single step in the release that is hardest to recover from automatically — usually the database migration, the irreversible infra change, or the artifact-promotion step. THIS step gets every phase at maximum depth. If it fails mid-release, what is the recovery?

**Popper Release Falsification Categories:**
- **CLAIM vs REALITY** — "we have automated rollback" but the rollback script references a deleted backup bucket
- **TAG vs ARTIFACT** — git tag `v2.3.1` vs what's actually running in prod (build drift)
- **CODE vs SCHEMA** — code rolled back to N-1 but the migration to N already ran and is non-reversible
- **GATE vs BYPASS** — "required checks" that maintainers can `--force` / admin-merge past
- **CHANGELOG vs DIFF** — changelog says "bug fixes" but the diff includes a breaking API change
- **STAGING vs PROD** — staging passed but prod has different env vars / region / data volume

---

## SCOPE DETECTION (automatic from user prompt)

Read the user's prompt and determine scope automatically. No extra flags needed.

```
EXAMPLES:
  "/releaseaudit"
  → Full 20-phase pipeline on the entire release process (CI config, build,
    migrations, deploy scripts, rollback, flags, secrets, post-deploy checks).

  "/releaseaudit the migration that drops the orders.legacy_id column"
  → TARGETED: focus Phase 5 (migration safety) + Phase 6 (rollback) at 10× depth.
    Trace forward + rollback path for that specific migration.

  "/releaseaudit can we actually roll back the last deploy?"
  → ROLLBACK-FOCUSED: Phase 6 (rollback procedure) primary, Phase 2 (build
    reproducibility) + Phase 7 (deploy strategy) supporting. Prove the rollback works.

  "/releaseaudit the GitHub Actions pipeline"
  → CI-FOCUSED: Phase 1 (pipeline integrity), Phase 11 (deploy gates),
    Phase 12 (secret handling), Phase 17 (artifact provenance).

  "/releaseaudit feature flags"
  → FLAG-FOCUSED: Phase 8 (feature-flag hygiene) primary, kill-switch + stale-flag scan.

  "/releaseaudit are we ready to ship v3?"
  → RELEASE-READINESS mode: all phases, with extra weight on changelog accuracy
    (Phase 4), versioning (Phase 3), post-deploy verification (Phase 9).

RULES:
- If a specific migration / script / pipeline mentioned: scope to it + its blast radius.
- If a problem described ("the rollback failed last week"): focus relevant phases, gather runtime evidence first (First Law).
- If "all" / "everything" / "release readiness": all phases.
- If audits/.releaseaudit/fix-plan.json exists and no new scope: resume fixing.
- Parse the intent, don't ask for clarification.
```

---

## NON-UI / CONTEXT GATE

`/releaseaudit` runs on ANY shippable target: web app, API, CLI, library, daemon, container, monorepo. It does NOT abort on non-UI projects. It DOES adapt:

- **No CI config found** → that itself is a CRITICAL finding (Phase 1), not a reason to abort. Manual / undocumented deploys are the highest-risk release mode.
- **No database** → Phase 5 (migrations) scores N/A and is excluded from the normalized denominator (see Scoring).
- **No feature flags** → Phase 8 evaluates whether the project *should* have a kill switch given its risk profile; absence is a finding only if risk warrants it.
- **Library / package (npm, crate, PyPI)** → "deploy" = publish; "rollback" = deprecate/yank + republish; reproducibility and semver become the dominant phases.

---

## OUTPUT CONTRACT — Omega Integration

Every `/releaseaudit` run produces these files. Oracles, AISB, and the monitor read them.

```
audits/.releaseaudit/
├── session.log                      # Audit start/end timestamps
├── discovery/
│   ├── pipeline-inventory.json      # All CI/CD configs, jobs, triggers, gates
│   ├── deploy-targets.json          # Where it ships (Vercel, systemd, Docker, registry)
│   ├── migration-inventory.json     # All migrations, ordering, reversibility flags
│   ├── flag-inventory.json          # All feature flags, defaults, owners, age
│   └── version-baseline.json        # Current version, last tag, last deploy, prod state
├── reports/
│   ├── pipeline-integrity.md        # Phase 1
│   ├── build-reproducibility.md     # Phase 2
│   ├── versioning-semver.md         # Phase 3
│   ├── changelog-accuracy.md        # Phase 4
│   ├── migration-safety.md          # Phase 5
│   ├── rollback-procedure.md        # Phase 6
│   ├── deploy-strategy.md           # Phase 7
│   ├── feature-flag-hygiene.md      # Phase 8
│   ├── post-deploy-verification.md  # Phase 9
│   ├── env-parity.md                # Phase 10
│   ├── deploy-gates.md              # Phase 11
│   ├── pipeline-secrets.md          # Phase 12
│   ├── zero-downtime.md             # Phase 13
│   ├── deploy-ordering.md           # Phase 14
│   ├── release-observability.md     # Phase 15
│   ├── release-time-bombs.md        # Phase 16
│   ├── artifact-provenance.md       # Phase 17
│   └── release-runbook.md           # Phase 18
├── verdict.json                     # Machine-readable v2 schema
├── verdict.md                       # Human-readable final report
├── fix-plan.json                    # {tasks: [{id, finding, file, line, fix, status, severity}]}
├── fix-plan.md
├── progress.json                    # Live progress for the Telegram monitor
├── before-after.md                  # Regression matrix (per AUDIT-VERIFICATION-CONTRACT)
├── telemetry.json                   # duration, tokens, phases, fixes, preamble_version
└── fix-log.md                       # Append-only log of each fix applied
```

**CRITICAL:** `progress.json` is read by the Telegram bot monitor for live progress cards.
Format: `{"total": 31, "done": 9, "failed": 1, "skipped": 2, "remaining": 19, "current": "FIX-010 — add rollback smoke test"}`

**CRITICAL:** `fix-plan.json` is read by oracles to resume interrupted audits.

---

## PHASE 0 — PROGRAMMATIC GATHER (HYBRID, runs FIRST)

> Before any LLM analysis, gather every machine-checkable fact deterministically.
> The LLM then READS the resulting JSON instead of hand-grepping. Freed token
> budget is REINVESTED in deeper Popper falsification, hinge synthesis,
> user-need verification, and edge-case hunting.

### 0.1 Run the gather (or gather manually if no runner)

```bash
~/.omega/lib/audit-runner.sh release "$PROJECT_PATH" \
  --files="$FILES_MODIFIED" \
  --url="$URL" \
  --user-need="$USER_NEED_QUOTE" \
  --hinge="$HINGE_POINT" \
  --ticket="$TICKET_ID"
```

If no release-specific gather exists yet, gather these deterministically and write `$PROJECT_PATH/audits/.releaseaudit/evidence-summary.json`:

```
- CI config inventory: .github/workflows/*.yml, .gitlab-ci.yml, circle/, Jenkinsfile, .buildkite/
- Deploy config: vercel.json, fly.toml, Dockerfile, docker-compose*.yml, *.service (systemd), Procfile, deploy/*.sh
- Lockfile presence + freshness: package-lock.json / pnpm-lock.yaml / yarn.lock / Cargo.lock / poetry.lock / requirements.txt pinning
- Migration dirs: migrations/, prisma/migrations/, alembic/, convex/schema.ts diffs, supabase/migrations/
- Version sources: package.json version, Cargo.toml, git tags (git tag --sort=-creatordate | head), CHANGELOG.md
- Secret references in CI: grep workflows for `secrets.`, `env:`, `${{ }}`, hardcoded-looking tokens
- Deploy tokens scope: any `--token`, `VERCEL_TOKEN`, registry creds in scripts
```

### 0.2 What you do AFTER the gather (this replaces hand-greps)

1. **Read `evidence-summary.json` in full.** This is your evidence base.
2. **Read the actual CI workflow files and deploy scripts** — these are load-bearing and must be read whole, not sampled.
3. **DO NOT re-grep what the gather already covered.**
4. **DO read** any migration the gather flagged as non-additive, any deploy script, and the rollback script (if one exists).

### 0.3 Banned operations after Phase 0

- ❌ Re-running `git tag` / `find migrations/` loops the gather already did.
- ❌ Generic "let me list every workflow file" — read the inventory JSON.

You MAY still: read SPECIFIC files cited in findings; run a SPECIFIC command to falsify a finding (Popper); run a SPECIFIC probe the static gather can't (e.g. actually invoke the rollback script in a dry-run / staging context).

### 0.4 Cross-audit synthesis (read sibling summaries)

If part of a Linear-fix mission, sibling summaries are at
`$PROJECT_PATH/audits/.linear-fix/<TICKET>/.<other-audit-id>/evidence-summary.json`.
High-value confluences:
- **releaseaudit + secaudit** flag the same pipeline secret → confidence escalation (a leaked deploy token is both a release risk and a security breach).
- **releaseaudit + dataaudit** flag the same migration → schema change is both a release-safety risk and a data-integrity risk.
- **releaseaudit + codeaudit** flag the same config-drift → build reproducibility risk confirmed from two angles.

Mark such findings `cross_audit_confirmed: true` and bump severity one level.

---

## PHASE 0b: RELEASE RECONNAISSANCE

> *"Map how it ships before you judge whether it ships safely."*

```bash
SESSION_ID="releaseaudit-$(date +%Y%m%d-%H%M%S)"
mkdir -p audits/.releaseaudit/{discovery,reports,baseline}
echo "AUDIT STARTED: $(date -Iseconds)" > audits/.releaseaudit/session.log
```

```
1. SHIPPING MODEL DISCOVERY
   → Read CLAUDE.md, README, package.json/Cargo.toml/pyproject.toml
   → Identify: how does code reach prod? (CI auto-deploy, manual script, push-to-branch, registry publish)
   → Identify: hosting (Vercel, Fly, AWS, systemd on a VPS, npm registry)
   → Identify: environments (dev / preview / staging / prod) and how each is reached

2. RELEASE ARTIFACT MAP
   → What is the unit of release? (container image, JS bundle, binary, npm package, git tag)
   → Where is it stored / promoted? (registry, Vercel build, GitHub release)
   → Is it versioned? Immutable? Signed?

3. STATE-CHANGE MAP (the irreversible parts)
   → Migrations: which release touches the schema? data?
   → Infra changes: DNS, env vars, feature flags flipped as part of release?
   → External side effects: webhooks re-registered, cron schedules changed?

4. RELEASE HINGE POINT IDENTIFICATION
   → Identify THE single step hardest to recover from automatically.
   → Usually: the DB migration, the artifact promotion, or an irreversible infra flip.
   → This becomes ground zero for 10× depth across all phases.
```

---

## PHASE 1: CI/CD PIPELINE INTEGRITY

> *"A pipeline you can't read is a pipeline you can't trust."*

```
1. PIPELINE EXISTENCE & COVERAGE
   → Is there an automated pipeline at all? (no pipeline = CRITICAL: manual deploys are unauditable)
   → What triggers it? (push to main, tag, manual dispatch, PR merge)
   → Does every path to prod go THROUGH the pipeline, or can someone deploy out-of-band?
   → Is the pipeline config itself version-controlled and reviewed?

2. STAGE STRUCTURE
   → Stages present: lint → typecheck → test → build → migrate → deploy → verify?
   → Which stages are MISSING? (no post-deploy verify = blind ship)
   → Does a stage failure actually STOP the deploy, or does it `continue-on-error`?
   → Grep for `continue-on-error: true`, `|| true`, `if: always()` masking failures.

3. PIPELINE DETERMINISM
   → Does the pipeline pin its own tool versions? (actions/setup-node@v4 with a node version, not `latest`)
   → Are third-party Actions pinned to a SHA, not a mutable tag? (supply-chain: `uses: foo/bar@v1` is mutable)
   → Are runners reproducible? (specific ubuntu-22.04 vs ubuntu-latest)

4. CONCURRENCY & RACE SAFETY
   → Can two deploys to the same env run simultaneously? (concurrency group set?)
   → Is there a deploy lock / serialization? (two pipelines racing on the same prod = corruption)
   → What happens if a deploy is cancelled mid-flight?

5. PIPELINE FAILURE BEHAVIOR
   → If deploy step fails AFTER migrate step succeeded → what state is prod in? (partial release)
   → Is the pipeline idempotent on retry, or does re-running double-apply?

SCORE: 0 = no pipeline / out-of-band deploys possible, 3 = pipeline exists but failures don't block, 5 = blocks but non-deterministic, 8 = deterministic + serialized, 10 = pinned + serialized + every prod path gated + partial-failure handled
```

---

## PHASE 2: BUILD REPRODUCIBILITY

> *"If you can't rebuild it byte-for-byte, you don't know what's in prod."*

```
1. LOCKFILE INTEGRITY
   → Lockfile present and committed? (package-lock/pnpm-lock/yarn.lock/Cargo.lock/poetry.lock)
   → Does the pipeline install from the lockfile? (`npm ci` not `npm install`; `--frozen-lockfile`)
   → Does the lockfile match the manifest? (drift = non-reproducible)
   → Are dependencies pinned, or do floating ranges (`^`, `~`, `*`) leak into the build?

2. HERMETIC BUILD CHECK
   → Does the build fetch anything from the network at build time beyond pinned deps?
   → Curl-piped installs in the build? (`curl ... | bash` = non-reproducible + supply-chain risk)
   → Does the build embed a timestamp / random / git-dirty state that breaks determinism?

3. TAG → ARTIFACT TRACEABILITY
   → Can you go from a deployed artifact back to the exact git SHA? (build embeds commit SHA?)
   → Is the prod-running version queryable? (a /version endpoint, a build-info file)
   → FALSIFY: checkout the last released tag, build, compare to what the pipeline produced.

4. BUILD CACHE CORRECTNESS
   → Is the build cache keyed on the lockfile hash? (stale cache = wrong deps shipped)
   → Can a poisoned cache produce a wrong artifact?

5. CROSS-ENV BUILD CONSISTENCY
   → Same build command in CI and locally? (or does local diverge from CI?)
   → Same node/rust/python version everywhere?

SCORE: 0 = no lockfile / floating deps / curl|bash, 3 = lockfile but `install` not `ci`, 5 = pinned but build embeds non-determinism, 8 = hermetic + lockfile-installed, 10 = hermetic + SHA-traceable + reproducible artifact verified
```

---

## PHASE 3: VERSIONING & SEMVER CORRECTNESS

> *"A version number is a promise to your consumers. Break semver, break trust."*

```
1. VERSION SOURCE OF TRUTH
   → Single source for the version, or does it drift across package.json / git tag / CHANGELOG / deployed artifact?
   → Is the version bumped automatically or manually? (manual = forgotten-bump risk)
   → Does the git tag match the manifest version at release time?

2. SEMVER COMPLIANCE
   → Breaking changes → MAJOR bump? (a removed/renamed API in a MINOR release = semver violation)
   → New features → MINOR? Bug fixes → PATCH?
   → For libraries: does the diff justify the bump? (FALSIFY: diff the public API surface between tags)
   → Pre-release / build metadata used correctly? (-rc.1, +build.123)

3. RELEASE IMMUTABILITY
   → Are released versions immutable? (can someone re-publish v2.3.1 with different content?)
   → npm: is the version already published and unchangeable? Git tag: is it protected from force-move?

4. DEPENDENCY VERSION CONTRACT
   → Does this release bump dependency ranges in a way that forces consumers to upgrade?
   → Peer-dependency ranges sane?

5. DEPRECATION SIGNALING
   → Are deprecated features marked before removal? (removing without a deprecation cycle = surprise break)

SCORE: 0 = version drift across sources / re-mutable releases, 3 = single source but manual & error-prone, 5 = semver mostly right with gaps, 8 = automated + immutable, 10 = automated + immutable + API-diff-verified semver + deprecation cycle
```

---

## PHASE 4: CHANGELOG ACCURACY

> *"A changelog that lies is worse than no changelog — it tells operators the release is safe when it isn't."*

```
1. EXISTENCE & FORMAT
   → CHANGELOG.md (or release notes) present? Updated for this release?
   → Follows a parseable convention (Keep a Changelog, conventional-commits-derived)?
   → Entries grouped (Added / Changed / Fixed / Removed / Security / BREAKING)?

2. CHANGELOG vs DIFF (the falsification)
   → For the pending release, diff the actual commits/files against the changelog entries.
   → Is every user-facing change in the changelog? (silent breaking change = CRITICAL)
   → Is every changelog entry backed by a real change? (phantom entry)
   → Are BREAKING changes explicitly flagged, with migration guidance?

3. SECURITY & MIGRATION CALLOUTS
   → Security fixes called out separately (so operators prioritize)?
   → Schema/migration changes mentioned with operational impact (downtime, backfill duration)?
   → Required env-var / config changes documented for this release?

4. AUDIENCE FIT
   → Library: API-consumer-focused (what breaks, how to migrate)?
   → App/service: operator-focused (what to watch, what to roll back if it breaks)?

5. AUTOMATION INTEGRITY
   → If changelog is auto-generated from commits: are commit messages disciplined enough to produce a truthful changelog?
   → FALSIFY: does the auto-changelog miss changes made via squash/force-push?

SCORE: 0 = no changelog or it contradicts the diff, 3 = exists but incomplete, 5 = mostly accurate, missing BREAKING flags, 8 = accurate + categorized, 10 = diff-verified accurate + BREAKING + security + migration callouts
```

---

## PHASE 5: DATABASE MIGRATION SAFETY (forward + rollback)

> *"Code rolls back in seconds. A bad migration rolls back never. This is the phase that decides whether the whole release is reversible."*

```
THIS PHASE IS USUALLY THE HINGE. Apply 10× scrutiny.

1. MIGRATION INVENTORY & ORDERING
   → Every pending migration listed, in apply order?
   → Are migrations idempotent / safe to re-run after a partial failure?
   → Is there a transaction boundary per migration? (a migration that fails halfway leaving partial schema = corruption)
   → Convex/Prisma/Alembic/raw SQL — which engine, what guarantees?

2. REVERSIBILITY (the core question)
   → Does each migration have a DOWN / rollback path? (additive-only migrations are reversible by being no-ops to undo)
   → Destructive ops (DROP COLUMN, DROP TABLE, NOT NULL on existing, type narrowing) → IRREVERSIBLE without data loss. Flagged?
   → FALSIFY "we can roll back": if the code rolls back to N-1 but migration-N already ran, does N-1 code still work against the N schema?

3. EXPAND-CONTRACT DISCIPLINE
   → Are schema changes done as expand (add new, backfill, dual-write) → deploy code → contract (remove old) across SEPARATE releases?
   → Or is a breaking schema change shipped IN THE SAME release as the code that needs it? (= no safe rollback window) → CRITICAL
   → Does old code (N-1) tolerate the new schema during the deploy window? (rolling deploy runs both simultaneously)

4. BACKFILL SAFETY
   → Long-running backfills run inside the deploy, blocking it? (deploy timeout / lock)
   → Backfill batched + resumable, or one giant UPDATE that locks the table?
   → What happens if the backfill fails at row 500k of 1M?

5. MIGRATION TESTING & DRY-RUN
   → Are migrations tested against a prod-like dataset (volume, not just an empty test DB)?
   → Is there a dry-run / plan step before apply? (Prisma migrate diff, sqitch verify)
   → Backup taken IMMEDIATELY before migration apply? (recovery anchor)

6. ZERO-DOWNTIME SCHEMA CHANGES
   → Adding a NOT NULL column without default on a large table = table lock = downtime. Detected?
   → Index creation: CONCURRENTLY (Postgres) or blocking?

SCORE (N/A if no database — excluded from denominator):
  0 = destructive migration shipped with dependent code, no rollback, no backup
  3 = migrations exist but no down-path, no expand-contract
  5 = reversible for simple cases, risky on destructive ops
  8 = expand-contract + backup + batched backfill
  10 = expand-contract enforced + dry-run + prod-volume tested + zero-downtime + verified rollback path
```

---

## PHASE 6: ROLLBACK PROCEDURE

> *"'We can roll back' is a hypothesis until you've actually done it. Most teams discover their rollback is broken during the outage."*

```
1. ROLLBACK EXISTENCE
   → Is there a documented, single-command rollback? (or is it tribal knowledge / improvised?)
   → Where is it? (runbook, script, platform feature like Vercel instant rollback)
   → Does it cover code AND config AND (where reversible) schema?

2. ROLLBACK TRIGGERS & DECISION
   → Who/what decides to roll back? (manual judgement vs automated on health-check failure)
   → Is there a clear "if X, roll back" criterion (error rate threshold, failed smoke test)?
   → How long from "we should roll back" to "we are rolled back"? (MTTR for rollback)

3. ROLLBACK CORRECTNESS (falsify it)
   → Does the rollback target a KNOWN-GOOD previous artifact (immutable, still available)?
   → Or does it rebuild from source — re-introducing the build-reproducibility risk (Phase 2)?
   → Does rollback handle the migration that already ran? (forward-fix vs roll-back decision tree)
   → FALSIFY: trace the rollback script line by line — does every referenced bucket/tag/image still exist?

4. ROLLBACK REHEARSAL
   → When was rollback last actually exercised? (never = it's broken, you just don't know yet)
   → Is rollback tested in CI / staging as part of the pipeline?
   → Does the runbook match the current infra, or is it stale?

5. PARTIAL-FAILURE RECOVERY
   → Canary failed at 10% → can you halt + revert just the canary?
   → Deploy failed after migrate → forward-fix path documented?

SCORE: 0 = no rollback / improvised, 3 = documented but stale/untested, 5 = works for code but not migrations, 8 = tested rollback for code+config, 10 = one-command + rehearsed + migration-aware decision tree + MTTR measured
```

---

## PHASE 7: DEPLOYMENT STRATEGY (blue-green / canary / rolling)

> *"All-at-once deploys make every release a coin flip. A gradual rollout makes failure observable before it's total."*

```
1. STRATEGY IDENTIFICATION
   → How does new code reach 100% of traffic? (big-bang, rolling, blue-green, canary)
   → Is there ANY gradual exposure, or does one deploy flip all users at once?
   → For static/serverless (Vercel): atomic swap + instant rollback (good); for stateful services: rolling?

2. HEALTH-GATED PROMOTION
   → Does promotion to the next stage/percentage depend on health checks passing?
   → Canary: what % first? What metrics gate promotion (error rate, latency, saturation)?
   → Automated rollback on canary failure, or manual?

3. TRAFFIC & SESSION SAFETY
   → During a rolling deploy, old + new versions serve simultaneously — are they compatible? (ties back to Phase 5 expand-contract)
   → Sticky sessions / in-flight requests drained gracefully? (connection draining)
   → WebSocket / long-lived connections handled on version switch?

4. BLAST RADIUS CONTROL
   → Can a bad deploy be limited to one region / one cohort first?
   → Is there a kill switch independent of the deploy (Phase 8)?

5. DEPLOY WINDOW & TIMING
   → Deploys gated to low-traffic windows where risk warrants?
   → Freeze windows respected (no Friday-5pm prod deploys without override)?

SCORE: 0 = big-bang, no health gate, 3 = atomic swap but no gradual exposure, 5 = rolling but no health-gated promotion, 8 = canary or blue-green with health gate, 10 = health-gated canary + auto-rollback + blast-radius control + connection draining
```

---

## PHASE 8: FEATURE-FLAG HYGIENE

> *"Flags decouple deploy from release — but a stale flag is a landmine and a flag with no kill switch is a deploy you can't undo without one."*

```
1. FLAG INVENTORY & OWNERSHIP
   → Every flag listed with: default value, owner, creation date, intended lifetime (release-toggle vs permanent ops-switch vs experiment)?
   → Flags with no owner / no expiry = debt.

2. STALE FLAG DETECTION
   → Flags that are 100% on (or off) everywhere for > N weeks → should be removed and code path collapsed.
   → Dead flag branches (the off-path is unreachable) → confusion + risk.
   → FALSIFY: grep for the flag key — is the alternate branch still reachable?

3. KILL-SWITCH PRESENCE & SPEED
   → For risky features: is there a flag that disables it WITHOUT a redeploy?
   → How fast does a flag flip propagate? (instant, cached, requires restart?)
   → This is the cheap rollback — does it exist for the things most likely to fail?

4. DEFAULT-SAFE & FAIL-SAFE
   → If the flag service is unreachable, does the flag default to the SAFE value (feature off)?
   → New flags default off (opt-in) rather than on (opt-out)?

5. FLAG-DEPLOY COUPLING
   → Is a flag flipped as part of the deploy in a way that can't be undone independently?
   → Combinatorial risk: do multiple in-flight flags interact untested?

SCORE (N/A if no flags AND risk profile doesn't warrant one): 0 = risky features with no kill switch, 3 = flags exist but stale/unowned, 5 = owned but no fail-safe default, 8 = kill switches + default-safe, 10 = inventoried + owned + expiring + fail-safe + fast kill switch on every risky path
```

---

## PHASE 9: POST-DEPLOY VERIFICATION

> *"A deploy that ends at 'pipeline green' is a deploy that ends blind. Prod is the only truth (First Law)."*

```
1. SMOKE TEST AFTER DEPLOY
   → Does the pipeline hit the deployed URL and assert 200 on key routes AFTER deploy? (R-14)
   → Does it exercise the golden path (auth → core feature), not just "/"? 
   → Does it check the deployed VERSION matches the intended version? (catch silent deploy-of-wrong-thing)

2. HEALTH & READINESS PROBES
   → /health and /ready endpoints exist and are checked?
   → DB connectivity, downstream-dependency reachability verified post-deploy?

3. ERROR-BUDGET / METRIC BURN
   → Is error rate / latency watched for a window after deploy (auto-rollback trigger)?
   → Console errors / 5xx surfacing checked (rule 51: console is a fix list)?

4. VERIFICATION → ROLLBACK LINKAGE
   → If post-deploy verification fails, does it AUTO-trigger rollback (Phase 6), or just alert?
   → Is the failure loud (paging) or silent (a log line nobody reads)?

5. SCOPE OF VERIFICATION
   → Migration applied successfully verified (row counts, schema present)?
   → Feature flags in expected state post-deploy?
   → Cron / scheduled jobs still scheduled after deploy?

SCORE: 0 = ends at "build passed", 3 = checks "/" returns 200 only, 5 = golden-path smoke but no auto-rollback link, 8 = golden-path + health + version assert, 10 = golden-path + version + error-budget window + auto-rollback on fail
```

---

## PHASE 10: ENVIRONMENT PARITY (dev / staging / prod drift)

> *"Staging passed' means nothing if staging isn't prod. Drift is where 'works on staging' becomes a 2am incident."*

```
1. CONFIG PARITY
   → Same env-var KEYS across environments (values differ, keys shouldn't)?
   → Phantom env vars: referenced in code, present in staging, MISSING in prod?
   → Drift between .env.staging and .env.production beyond intended differences?

2. INFRA PARITY
   → Same runtime versions (node/rust/python) across envs?
   → Same region / DB engine version / resource class (or documented why not)?
   → Same dependency versions deployed (no "staging has the patched lib, prod doesn't")?

3. DATA PARITY (for testing realism)
   → Does staging have prod-like data VOLUME for migration/perf testing (Phase 5)?
   → Seed/anonymized prod data, or empty toy DB (false confidence)?

4. PIPELINE PARITY
   → Does the staging deploy use the SAME pipeline path as prod? (or a different, untested path?)
   → Is "deploy to staging" a true rehearsal of "deploy to prod"?

5. SECRET PARITY
   → Each env has its OWN secrets (no prod creds in staging, no shared keys)?
   → Rotation applied to all envs?

SCORE: 0 = prod config undocumented / phantom vars in prod, 3 = key drift between envs, 5 = config parity but data/infra drift, 8 = config + infra parity, 10 = config + infra + pipeline parity + prod-like staging data
```

---

## PHASE 11: DEPLOY GATES

> *"A gate a maintainer can click past is not a gate. It's a speed bump."*

```
1. REQUIRED CHECKS
   → Which checks are REQUIRED before merge/deploy to prod? (tests, typecheck, build, security scan)
   → Are they enforced by branch protection, or just convention?
   → Can an admin / force-push / `--no-verify` bypass them? (FALSIFY: check branch protection settings + `allow force pushes`)

2. APPROVAL GATES
   → Human approval required for prod deploy? (environment protection rules)
   → Code review required (min approvers, code owners)?
   → Can the deployer self-approve?

3. AUTOMATED QUALITY GATES
   → Is a build/lint/typecheck failure a hard stop, or `continue-on-error`?
   → Is a security scan (secrets, deps) a deploy gate or advisory?
   → Coverage / quality thresholds enforced or decorative?

4. MANUAL-STEP DISCIPLINE
   → Any deploy steps that are manual and easily skipped/forgotten? (the forgotten step is the one that breaks prod)
   → Checklist enforced programmatically, or a wiki page nobody opens?

5. FREEZE / OVERRIDE CONTROLS
   → Is there a deploy freeze mechanism (the ship-freeze lock pattern)?
   → Overrides logged + attributed?

SCORE: 0 = no gates / fully bypassable, 3 = gates exist but admin-bypassable, 5 = required checks but no approval, 6 = approval but self-approvable, 8 = required + non-bypassable + approval, 10 = required + code-owner approval + security gate + freeze control + audited overrides
```

---

## PHASE 12: SECRET HANDLING IN THE PIPELINE

> *"The CI runner holds the keys to prod. One echoed secret in a log and they're public forever."*

```
1. SECRET STORAGE
   → Secrets in the CI secret store (not plaintext in the workflow YAML / repo)?
   → grep workflows/scripts for hardcoded-looking tokens, keys, connection strings.
   → .env with real secrets committed? (cross-ref secaudit)

2. SECRET EXPOSURE IN LOGS
   → Are secrets masked in CI output? (GitHub masks `secrets.*`, but `echo $TOKEN` in a script can leak)
   → `set -x` / verbose modes that echo secret-bearing commands?
   → Build artifacts / source maps embedding server secrets? (NEXT_PUBLIC_ misuse)

3. DEPLOY TOKEN SCOPE (least privilege)
   → Is the deploy token scoped to ONLY what it needs? (a token that can deploy AND read all repos = over-scoped)
   → Prod DB credentials in CI: read-only where possible? migration creds separate + time-boxed?
   → Can a malicious PR exfiltrate secrets? (pull_request_target misuse, secrets exposed to fork PRs)

4. SECRET ROTATION
   → Rotation mechanism + cadence? Last rotated when?
   → Revocation path if a token leaks?

5. THIRD-PARTY ACTION TRUST
   → Do pinned-by-SHA Actions (Phase 1) prevent a compromised action from stealing secrets?
   → Any Action with access to secrets that doesn't need them?

SCORE: 0 = plaintext secrets in repo / leaked in logs, 3 = stored but over-scoped, 5 = scoped but not rotated, 8 = scoped + masked + rotated, 10 = least-privilege + masked + rotated + fork-PR-safe + SHA-pinned actions
```

---

## PHASE 13: ZERO-DOWNTIME GUARANTEES

> *"Downtime during a deploy is a self-inflicted outage. The question is whether the deploy mechanism even tries to avoid it."*

```
1. CONNECTION DRAINING
   → On version switch, are in-flight requests allowed to complete before old instance dies?
   → Graceful shutdown handlers (SIGTERM → drain → exit)?

2. READINESS BEFORE TRAFFIC
   → New instance receives traffic only AFTER it's ready (health check passes), not on process-start?
   → Warm-up / cache-prime before cutover?

3. STATEFUL TRANSITIONS
   → Session store survives deploy (external, not in-process)?
   → In-flight jobs / queues drained or resumable across deploy?
   → DB connection pool churn handled?

4. SCHEMA-DURING-DEPLOY (ties to Phase 5)
   → During rolling deploy, old+new code both work against current schema (expand-contract)?
   → No "table locked by migration" during the deploy window?

5. CLIENT IMPACT
   → SPA: does a new deploy break already-loaded clients (chunk hash mismatch → load error)? Versioned assets + graceful reload?

SCORE (weight lower for static/serverless where the platform handles it): 0 = deploy causes hard downtime, 3 = brief downtime, 5 = zero-downtime for code but migration locks, 8 = drained + readiness-gated, 10 = full zero-downtime incl. schema + stateful transitions + client-safe
```

---

## PHASE 14: DEPLOY ORDERING & DEPENDENCY SEQUENCING

> *"Ship the consumer before the provider and you ship a broken release that passed every test in isolation."*

```
1. SERVICE ORDERING
   → Multi-service release: is the deploy order correct? (provider/API before consumer; or backward-compatible either way)
   → Are inter-service contract changes backward-compatible across the deploy window?

2. MIGRATION-CODE ORDERING
   → Does the migration run BEFORE the code that depends on it? (or expand-contract makes order irrelevant — Phase 5)
   → If migration and code are coupled in one release, what's the apply order, and is the in-between state valid?

3. CONFIG / FLAG ORDERING
   → Env var / flag that the new code needs — set BEFORE the code deploys?
   → Webhook/integration re-registration ordered correctly?

4. CRON / SCHEDULED-JOB ORDERING
   → A cron that depends on new code/schema — does it get disabled until the deploy completes?
   → Does a mid-deploy cron run against half-deployed state?

5. CACHE / CDN INVALIDATION ORDERING
   → CDN purge after new assets are live (not before)?
   → Cache stampede risk on invalidation?

SCORE: 0 = consumer-before-provider / migration-after-code with no expand-contract, 3 = ordering manual + fragile, 5 = ordered but in-between state risky, 8 = ordered + backward-compatible window, 10 = order-independent via expand-contract + flags + ordered side-effects
```

---

## PHASE 15: RELEASE OBSERVABILITY

> *"You can't roll back what you can't see go wrong. Observability is the trigger for every safety mechanism above."*

```
1. DEPLOY MARKERS
   → Is each deploy marked in monitoring/logs (deploy event with version + SHA + time)?
   → Can you correlate a metric regression to a specific deploy?

2. RELEASE-SCOPED ALERTING
   → Alerts that fire on post-deploy error/latency spikes (the rollback trigger)?
   → Error-budget burn tracked per release?

3. AUDIT TRAIL
   → Who deployed what, when, with what approval — logged and queryable?
   → Rollbacks logged with reason?

4. PIPELINE OBSERVABILITY
   → Pipeline duration / failure-rate trends tracked? (a slowly-degrading pipeline is a release risk)
   → Flaky deploy steps identified?

5. FEEDBACK LOOP
   → Post-incident: are release failures fed back into new gates/checks?
   → Is there a deploy-success-rate metric?

SCORE: 0 = no deploy visibility, 3 = logs exist but uncorrelated, 5 = deploy markers but no release-scoped alerts, 8 = markers + alerts + audit trail, 10 = markers + error-budget + audited + feedback loop into gates
```

---

## PHASE 16: RELEASE TIME BOMBS

> *"Some releases break not on deploy day, but on a date, a scale threshold, or a credential expiry you forgot about."*

```
1. EXPIRING CREDENTIALS IN THE RELEASE PATH
   → Deploy tokens / signing certs / registry creds with expiry — when do they expire? (expired token = no deploys = no rollback)
   → SSL certs auto-renewed, or manual time bomb?

2. PINNED-BUT-EOL TOOLING
   → Build pinned to an EOL runtime/runner image (ubuntu-20.04 sunset, node EOL)?
   → A pin that will silently stop working when the platform removes the image?

3. ONE-WAY-DOOR CHANGES
   → Changes in this release that cannot be undone later (data deletion, irreversible migration, external state change)?
   → Are they isolated so the rest of the release stays reversible?

4. SCALE-TRIGGERED RELEASE RISK
   → Migration/backfill that's fast on staging data but times out on prod volume (Phase 5 + Phase 10)?
   → Deploy step with a timeout that prod scale will exceed?

5. STALE RUNBOOK / SCRIPT REFERENCES
   → Rollback/deploy scripts referencing buckets/tags/hosts that no longer exist (Phase 6)?
   → Hardcoded URLs/regions that drifted?

SCORE: 0 = expiring creds + EOL tooling + stale rollback refs, 3 = one or two latent bombs, 5 = known but unmitigated, 8 = monitored with renewal, 10 = no latent bombs + expiries tracked + one-way doors isolated + prod-scale verified
```

---

## PHASE 17: ARTIFACT PROVENANCE & SUPPLY-CHAIN SIGNING

> *"If you can't prove the artifact in prod came from your pipeline, an attacker can substitute their own."*

```
1. ARTIFACT IMMUTABILITY & STORAGE
   → Released artifacts immutable + retained (so rollback has a target — ties to Phase 6)?
   → Stored in a registry with content-addressing (digest), not a mutable `latest` tag?

2. PROVENANCE / ATTESTATION
   → Build provenance generated? (SLSA attestation, GitHub artifact attestations, npm provenance)
   → Can you verify the artifact's build came from a specific commit + pipeline run?

3. SIGNING
   → Artifacts signed (cosign, npm provenance, GPG-signed tags/releases)?
   → Signature verified at deploy time before promotion?

4. SBOM & DEPENDENCY PROVENANCE
   → SBOM generated for the release? (know exactly what shipped)
   → Dependencies installed from trusted registries with integrity hashes (ties to Phase 2 lockfile)?

5. TAMPER DETECTION
   → Would a modified artifact be detected before it reaches prod?
   → Subresource Integrity for CDN-served assets?

SCORE: 0 = mutable `latest`, unsigned, no provenance, 3 = immutable digest but unsigned, 5 = signed but unverified at deploy, 8 = signed + verified + immutable, 10 = signed + verified + SLSA provenance + SBOM + tamper-detected
```

---

## PHASE 18: RELEASE RUNBOOK & HUMAN PROCESS

> *"At 3am during an outage, nobody improvises well. The runbook is the difference between a 5-minute recovery and a 5-hour one."*

```
1. RUNBOOK EXISTENCE & CURRENCY
   → Is there a release runbook (how to deploy, how to verify, how to roll back)?
   → Does it match the CURRENT pipeline/infra, or is it stale (FALSIFY: spot-check 3 steps against reality)?

2. ROLLBACK RUNBOOK (the critical one)
   → Step-by-step rollback documented, including the migration decision tree (Phase 5/6)?
   → Tested by someone OTHER than the author following it literally?

3. INCIDENT LINKAGE
   → "If post-deploy verification fails → do X" decision tree present (Phase 9 → Phase 6 linkage)?
   → On-call knows where the runbook is + has access to deploy/rollback controls?

4. OWNERSHIP & APPROVAL
   → Release owner / DRI defined per release?
   → Approval + communication process (who's notified, freeze windows)?

5. AUTOMATION OPPORTUNITY
   → Which manual runbook steps SHOULD be automated (the error-prone ones)?
   → Is tribal knowledge encoded anywhere, or does it live in one person's head (bus factor)?

SCORE: 0 = no runbook / tribal knowledge, 3 = exists but stale, 5 = current but rollback path untested, 8 = current + tested rollback runbook, 10 = current + tested + incident decision tree + DRI + bus-factor-safe
```

---

## PHASE H1 — HYBRID SYNTHESIS (Popper / hinge / user-need / edge cases / cross-audit)

> Runs immediately before VERDICT. It does NOT renumber existing phases — it sits
> between the last domain phase and VERDICT. The token budget freed by Phase 0's
> deterministic gather is REINVESTED here. This phase DEEPENS the earlier phases.

### H1.1 Popper falsification per finding (mandatory)

For every finding (start with `critical`/`high`), try to PROVE it wrong. Each falsification produces a `falsifiable_tests[]` entry in `verdict.json`:

```jsonc
{
  "claim": "The rollback script references s3://builds-prod which no longer exists",
  "test_command": "aws s3 ls s3://builds-prod/ 2>&1 | head; grep -rn 'builds-prod' deploy/",
  "expected": "NoSuchBucket → claim TRUE, rollback is broken",
  "actual": "An error occurred (NoSuchBucket) ...",
  "outcome": "confirmed"
}
```

Outcomes: `confirmed` (test failed to falsify → finding stands), `falsified` (counter-example found → demote to info + record evidence), `inconclusive` (could not run cleanly → keep severity, `confidence: medium`).

**The rule:** every CLAIM (PASS or FAIL) MUST cite ≥3 concrete commands that COULD have falsified it but didn't. Banned phrases (`looks correct`, `should be fine`, `appears to work`) → automatic FAIL.

Common falsification patterns for release findings:

| Claim | Popper test |
|---|---|
| "rollback exists and works" | Read the script end to end; verify every referenced artifact/bucket/tag still exists; if safe, dry-run it in staging |
| "build is reproducible" | `git checkout <tag> && <build>` twice; diff the artifacts (or their digests) |
| "migration is reversible" | Read the down-migration; check for DROP/destructive ops; simulate N-1 code against N schema |
| "CI blocks bad deploys" | Inspect branch protection + workflow for `continue-on-error`, `|| true`, admin-bypass settings |
| "secrets are safe" | grep workflow/scripts for `echo`/`set -x` near secret refs; check fork-PR exposure |
| "changelog is accurate" | `git log <last-tag>..HEAD --oneline` vs the changelog entries — find the gap |
| "zero downtime" | Check for graceful-shutdown handler + readiness gate; absence = downtime claim falsified |

### H1.2 Hinge cross-reference (10× scrutiny)

The RELEASE HINGE POINT (usually the migration or artifact-promotion step) gets:
- 5× more falsification attempts (H1.1)
- 3× more edge-case hunts (H1.4)
- Mandatory full read of the hinge step + its rollback path + everything it depends on.

Output `hinge_findings[]` in `verdict.json` (`finding_id`, `is_load_bearing`, `hinge_reference`, `additional_scrutiny`, `confidence_after_scrutiny`).

### H1.3 User-need verification (`--user-need` quote)

If dispatched with `--user-need="<verbatim>"` (e.g. "last deploy broke prod and we couldn't roll back for 40 minutes"), every finding is evaluated against it. Findings that address the user-need get top priority and are listed first in `user_need_match.findings[]`. If `addressed: false`, the audit MUST score below 90/100. The user's actual problem is the only correct success metric.

### H1.4 Edge case hunting (≥2 per top-5 finding)

The gather checked the pipeline at rest; you must imagine the release in motion:
- "Migration succeeds but deploy step fails → prod is on new schema with old code" — partial-release state
- "Rollback triggered AFTER the migration already ran" — code/schema mismatch
- "Two deploys race / a deploy is cancelled mid-migration" — concurrency
- "Deploy token expires the day of an incident → can't roll back" — credential time bomb
- "Fork PR triggers the workflow → secrets exposed" — supply-chain
- "Prod data volume 100× staging → backfill times out, deploy hangs" — scale
- "CDN serves new index.html but old JS chunks → loaded clients break" — asset versioning

Output `edge_cases[]` in `verdict.json` (`finding_id`, `scenario`, `covered_by_existing_test`, `evidence_gathered`, `fix_includes_coverage`).

### H1.5 Cross-audit synthesis

Re-read sibling summaries (Phase 0.4). For each top-5 finding, check if the same file/step is flagged by secaudit (pipeline secrets), dataaudit (migration), or codeaudit (config drift). If yes → `cross_audit_confirmed: true`, bump severity one level. Write `cross_audit_links[]` in `verdict.json`.

### H1.6 Final verdict.json schema (hybrid v2)

```jsonc
{
  "audit": "release",
  "score": 100,
  "score_raw": "<raw>/<applicable_max>",
  "score_normalized": 100,
  "confidence": "high|medium|low",
  "skill_used": "release",
  "user_need_match": { },        // H1.3
  "falsifiable_tests": [ ],       // H1.1
  "hinge_findings": [ ],          // H1.2
  "issues_found_and_fixed": [
    { "id": "FIX-001", "finding_id": "F-003", "before": "<state>", "after": "<state>", "verification": "<command + output>" }
  ],
  "edge_cases": [ ],              // H1.4
  "cross_audit_links": [ ],       // H1.5
  "evidence_summary_path": "$PROJECT_PATH/audits/.releaseaudit/evidence-summary.json",
  "reversibility_assertion": "Can this release be rolled back in one command, including schema? YES/NO + evidence",
  "confidence_basis": "Why I'm confident. Cite Popper test counts, hinge depth, edge-case coverage, cross-audit confirmations.",
  "banned_phrase_check": "passed (no `looks correct`, `should be fine`, `appears to work`, `streamlined`, `to save time`)",
  "preamble_version": "1.0"
}
```

### H1.7 Score gating (hybrid threshold)

A 100/100 score is BLOCKED unless:
1. ✅ All `critical`/`high` findings fixed OR have a ≥50-word `non_issue_justification` backed by Popper evidence.
2. ✅ All load-bearing (hinge) findings confirmed via Popper falsification.
3. ✅ `user_need_match.addressed = true` with verbatim quote (if `--user-need` given).
4. ✅ ≥3 falsifiable tests cited per phase.
5. ✅ ≥2 edge cases per top-5 finding.
6. ✅ Cross-audit synthesis attempted (array present, may be empty).
7. ✅ `reversibility_assertion` answered with evidence — and if it is NO, the score is capped at 69 (a non-reversible release cannot pass).
8. ✅ `confidence_basis` populated with non-trivial reasoning.

Below threshold → score < 100, the fix-and-reaudit loop kicks in (Phase 22/23), BOUNDED at 5 iterations per the Audit Verification Contract. On iteration 5 if still failing → `confidence: low`, surface as `pending`.

---

## PHASE 19: VERDICT

Score each phase 0-10, weight by severity. Migration and rollback dominate because they decide reversibility — the single question this audit answers.

```
SCORING MATRIX (400 max):
  Phase  1  (CI/CD Pipeline Integrity)      x 2.5  = max 25
  Phase  2  (Build Reproducibility)         x 2.5  = max 25
  Phase  3  (Versioning / Semver)           x 1.5  = max 15
  Phase  4  (Changelog Accuracy)            x 1.5  = max 15
  Phase  5  (DB Migration Safety)           x 4.0  = max 40   ← hinge
  Phase  6  (Rollback Procedure)            x 4.0  = max 40   ← hinge
  Phase  7  (Deployment Strategy)           x 2.5  = max 25
  Phase  8  (Feature-Flag Hygiene)          x 2.0  = max 20
  Phase  9  (Post-Deploy Verification)      x 3.0  = max 30
  Phase 10  (Environment Parity)            x 2.0  = max 20
  Phase 11  (Deploy Gates)                  x 2.5  = max 25
  Phase 12  (Pipeline Secret Handling)      x 2.5  = max 25
  Phase 13  (Zero-Downtime Guarantees)      x 2.0  = max 20
  Phase 14  (Deploy Ordering)               x 1.5  = max 15
  Phase 15  (Release Observability)         x 1.5  = max 15
  Phase 16  (Release Time Bombs)            x 1.5  = max 15
  Phase 17  (Artifact Provenance/Signing)   x 1.0  = max 10
  Phase 18  (Release Runbook / Process)     x 2.0  = max 20
                                            TOTAL  = max 400

NORMALIZE: score = (raw / applicable_max) × 100
  applicable_max excludes phases marked N/A (e.g. Phase 5 if no DB, Phase 8 if no flags
  and risk profile doesn't warrant one). N/A phases are removed from BOTH numerator and denominator.

HARD CAP: if reversibility_assertion == NO (release cannot be rolled back, including schema),
  normalized score is CAPPED at 69 regardless of other phases. Irreversible shipping cannot be "safe".

GRADE:
  90-100: S — Ships safe & reversible. One-command rollback, reproducible builds, expand-contract migrations, gated + verified.
  80-89:  A — Hardened release. Minor gaps reachable only by chained failures.
  70-79:  B — Solid. Reversible, but missing some gates/verification.
  60-69:  C — Risky. Reversible only for code; migrations or rollback are weak spots.
  50-59:  D — Dangerous. Big-bang deploys, untested rollback, or migration coupling.
  <50:    F — Unshippable. Irreversible migrations + no rollback + ungated pipeline. A bad deploy is an outage.
```

---

## PHASE 20: FIX PLAN → FIX EXECUTION → RE-AUDIT

### 20a. FIX PLAN (automatic)

```
Sort: CRITICAL → HIGH → MEDIUM → LOW, prioritized by REVERSIBILITY IMPACT:
  CRITICAL: makes the release irreversible OR can take prod down with no recovery
            (destructive migration + dependent code in one release; no rollback; ungated prod path)
  HIGH:     reversible but recovery is slow/untested (stale rollback runbook, no post-deploy verify)
  MEDIUM:   degrades safety margin (no canary, env drift, stale flags)
  LOW:      hygiene (changelog gaps, missing deploy markers)

Group by blast radius (adding a post-deploy smoke test + auto-rollback link closes several gaps at once).
Dependency order: establish a known-good artifact + rollback BEFORE tightening gates.
Generate fix tasks with file:line specificity → audits/.releaseaudit/fix-plan.{json,md}
```

### 20b. FIX EXECUTION (automatic) — DEPLOY-SAFETY GATE

```
─── SAFETY GATE: DO NO HARM (MANDATORY before EVERY fix) ──────────────
Read `../_shared/AUDIT-VERIFICATION-CONTRACT.md` before ANY fix.

Release fixes are HIGH-STAKES — they touch the pipeline that touches prod. Extra rules:

PRE-FIX:
  a. Read the ENTIRE target file (workflow YAML, deploy script, migration) — not just the line.
  b. NEVER trigger a real prod deploy as part of "verifying" a fix. Use dry-run / staging only.
  c. For migration fixes: NEVER run a destructive migration to "test" it. Reason about it + dry-run on a disposable DB.
  d. CROSS-REFERENCE: changing the pipeline? Grep for everything that triggers/depends on it.

APPLY → POST-FIX VERIFICATION (before commit):
  a. SYNTAX: workflow YAML lints (actionlint / yamllint); shell scripts `bash -n`; SQL parses.
  b. DRY-RUN: pipeline changes validated via `act` / a CI dry-run / staging trigger — NOT a prod deploy.
  c. NON-DESTRUCTION: confirm the fix does not itself introduce an irreversible step or break the rollback path.
  d. If a fix would alter live deploy behavior in a risky way → mark NEEDS_REVIEW (human confirmation required).

IF ANY CHECK FAILS → `git revert HEAD` → log in fix-log.md → NEEDS_REVIEW. Never retry blindly.

FOR EACH FIX (priority order):
  a. Read full file → b. PRE-FIX analysis → c. Document BEFORE state →
  d. Apply fix → e. POST-FIX verification (no prod deploy) → f. If green: commit `release(releaseaudit): FIX-XXX description` →
  g. If red: revert + log + NEEDS_REVIEW → h. Document AFTER state → i. Confirm no regression →
  j. HIGH-RISK fixes (anything touching prod-deploy logic, migrations, secrets, branch protection) ALWAYS require human confirmation.
```

### 20c. RE-AUDIT (automatic)

```
1. CONFIG VALIDITY GATE: all changed pipeline/deploy/migration configs lint + parse clean.
2. REVERSIBILITY RE-CHECK: re-evaluate reversibility_assertion — fixes must not have made it NO.
3. Re-run all FAILING phases. Compare before/after via before-after.md (zero regressions required — R-22).
4. Loop until normalized score ≥ 80 (and reversibility_assertion == YES), or remaining items are NEEDS_REVIEW.
5. BOUNDED at 5 iterations (Audit Verification Contract). On cap: confidence:low + pending + SOS.
```

---

## CROSS-COMMAND BRIDGE

```
/releaseaudit finds a pipeline secret leak     → cross-refs /secaudit (rotate immediately)
/releaseaudit finds a risky migration          → cross-refs /dataaudit (schema/integrity impact)
/releaseaudit finds build config drift         → cross-refs /codeaudit (config drift phase)
/releaseaudit finds no post-deploy smoke test  → cross-refs /debugaudit (runtime verification)

THE QUALITY ARSENAL:
  /codeaudit    → Is the code SOLID?              (preventive)
  /secaudit     → Is it SECURE?                   (detective)
  /dataaudit    → Is the data INTACT?             (detective)
  /releaseaudit → Is shipping SAFE + reversible?  (preventive)

  Together: code that's correct, secure, with intact data, shipped in a way you can undo.
```

---

## COMPLIANCE & CRITICAL ADDENDA (v1.0)

### Quality Arsenal Preamble Compliance

This audit implements contracts defined in `../_shared/QUALITY-ARSENAL-PREAMBLE.md` v1.0:

- ✅ **Gestalt-Popper doctrine** — release hinge point, falsification, evidence chain, adversarial framing
- ✅ **Concurrency lock** — canonical runner `flock` at `audits/.releaseaudit/.runner.lock`
- ✅ **5-iteration cap** — fix-and-reaudit loop bounded at 5 iterations. On cap: NEEDS_REVIEW + SOS.
- ✅ **Scoped invocation flags** — `--url=`, `--files=`, `--scope=`, `--ticket=`, `--no-fix`, `--focus=`
- ✅ **Non-UI context gate** — runs on any shippable target; phases scope per target type; N/A phases excluded from denominator
- ✅ **Output contract verification** — emits verdict.json/md, fix-plan.json/md, progress.json, before-after.md, telemetry.json, fix-log.md
- ✅ **Telegram progress notifications** — start / progress (every 3 phases) / iteration / verdict / abort / sos
- ✅ **Self-telemetry** — telemetry.json at completion (duration, tokens, phases, fixes, model, preamble_version)
- ✅ **Rule-46 compliance** — NO `--quick`/`--streamlined`/`--lightweight`. Narrower scope = `--focus <area>` at FULL depth. Banned-phrase prompts REFUSED.
- ✅ **Score normalization** — raw / applicable_max × 100; reversibility hard-cap at 69 when NO
- ✅ **preamble_version** — `"1.0"` in verdict.json for `/metaudit` compliance scan

### Audit-Specific Critical Addendum — Production Safety

- **NEVER trigger a real production deploy** to "verify" a fix. Dry-run, staging, or reasoning only. A verification step that ships to prod is itself an incident.
- **NEVER run a destructive migration** against a real database to test reversibility. Reason about it + test on a disposable DB.
- **Read-before-write on every pipeline/deploy/migration file** — these are load-bearing; partial edits are dangerous.
- **Self-target gate:** if the target is OmegaOS's own `install.sh` / release path, respect LAW 0 (install parity) — a release fix must keep `./install.sh` reproducible and pass `verify-install.sh`.

### /metaudit Compliance Badge

Run `/metaudit --focus arsenal --scope="releaseaudit only"` to verify against the 11-point preamble checklist. Target: 11/11.

---

## MANDATORY BEFORE/AFTER VERIFICATION

**Read `../_shared/AUDIT-VERIFICATION-CONTRACT.md` before ANY fix execution.**

1. **PRE-FIX BASELINE** — capture current pipeline/deploy/migration state; save to `audits/.releaseaudit/baseline/`.
2. **APPLY FIX** — normal execution (never a real prod deploy).
3. **POST-FIX CHECK** — re-run every baseline check (lint/parse/dry-run). Any PASSED→FAILED → revert.
4. **BREAKAGE SCAN** — grep for changed references across the repo; 0 non-ephemeral hits.
5. **BEFORE/AFTER MATRIX** — `before-after.md` with status table per affected release step.

**An audit that breaks the deploy pipeline is catastrophically worse than no audit.** Do NOT claim "done" without `before-after.md` showing zero regressions AND `reversibility_assertion == YES`.

---

## LAWS

1. **Reversibility is the only real safety.** Every release needs a tested one-command path back to a known-good state.
2. **Code rolls back; migrations often don't.** Expand-contract or you ship an irreversible release.
3. **Green CI proves the checks that exist passed — not that the right checks exist (Popper).**
4. **The pipeline holds the keys to prod.** Least privilege, masked secrets, signed artifacts, or it's a breach.
5. **A build you can't reproduce is a build you don't understand.** Lockfile-pinned, hermetic, SHA-traceable.
6. **"Pipeline green" is not "prod healthy" (First Law).** Verify the deployed runtime, then trust it.
7. **Rehearse the rollback, or discover it's broken during the outage (Popper).**

---

## Dynamic-Workflow Orchestration (v2)

> **This section governs HOW the 20 phases above are EXECUTED — not WHAT they check.**
> Every phase, scoring weight, the /400 matrix, the reversibility hard-cap, the verdict format,
> and the frontmatter remain exactly as defined above. This layer only changes the *execution
> shape*: from one analyst grinding phases 1→18 linearly, to a fan-out of parallel tracks whose
> survivors are adversarially verified and then folded back into the SAME matrix. Doctrine is
> unchanged: Gestalt clarity gate (release hinge point), Popper falsification, evidence chain,
> First Law (runtime over code). If this section ever conflicts with a phase definition, the
> phase definition wins.

### 1. Decompose into INDEPENDENT parallel tracks (fan-out)

The release-safety surface is naturally partitionable: the build pipeline, the schema, the
rollback path, and the secret/gate posture barely touch each other and can be investigated
concurrently. Group the 18 domain phases into file-disjoint tracks and dispatch them via the
**Workflow** tool (one branch per track, all launched together — never one-phase-at-a-time):

| Track | Phases | Evidence surface (mostly disjoint) | Owns |
|---|---|---|---|
| **T1 — Pipeline & Build** | 1, 2, 11 | `.github/workflows/*`, `.gitlab-ci.yml`, `Jenkinsfile`, lockfiles, branch protection | CI integrity, reproducibility, deploy gates |
| **T2 — Schema & State (HINGE)** | 5, 14 | `migrations/`, `prisma/`, `alembic/`, `convex/schema.ts`, backfill scripts | Migration reversibility, deploy ordering |
| **T3 — Reversibility & Rollout** | 6, 7, 13 | rollback script, `vercel.json`/`fly.toml`, deploy/canary config | Rollback path, deploy strategy, zero-downtime |
| **T4 — Secrets & Provenance** | 12, 17 | CI secret refs, deploy-token scope, signing/attestation config | Pipeline secret hygiene, artifact provenance |
| **T5 — Versioning & Comms** | 3, 4 | `package.json`/`Cargo.toml` version, git tags, `CHANGELOG.md` | Semver, changelog-vs-diff |
| **T6 — Verify & Drift** | 9, 10, 15 | post-deploy smoke config, `.env.*`, monitoring/deploy markers | Post-deploy verification, env parity, observability |
| **T7 — Latent Bombs & Runbook** | 8, 16, 18 | flag inventory, expiring creds/EOL pins, runbook docs | Flag hygiene, time bombs, human process |

Rules:
- **Phase 0 / 0b run FIRST and SERIALLY** — the deterministic gather (`evidence-summary.json`) +
  reconnaissance + **release-hinge-point identification** must complete before fan-out, because
  every track reads that shared evidence base and the hinge assignment routes 10× depth. Do NOT
  parallelize Phase 0.
- The track containing the **RELEASE HINGE POINT** (almost always **T2 — Schema & State**, sometimes
  T3's artifact-promotion/rollback step) gets the deepest branch: 5× more falsification attempts and
  3× more edge-case hunts (per H1.2/H1.4), and a mandatory full read of the hinge step + its rollback
  path + everything it depends on.
- **Honor R-SCOPE (one writer per file).** Fan-out branches are READ/ANALYZE only; they emit findings,
  they do NOT edit. All file mutation happens later in Phase 20 (Fix Execution), serialized, so two
  tracks that both flag the same migration never write it concurrently.
- **Synthesis is YOUR job, not a branch's** (R-ORCH) — never paste a track's summary as the verdict.

### 2. Adversarially verify EVERY finding (≥2-of-3 lenses) before it survives

A track's raw finding is a *hypothesis*, never a verdict (R-VERIFY). Before any finding is allowed
into the scoring matrix, subject it to **three independent lenses** and require **≥2 to agree**.
This reuses the existing Phase H1.1 Popper machinery — it does not replace it; it parallelizes and
quorum-gates it:

- **Lens A — REPRODUCE (runtime, First Law):** run the concrete probe that makes the failure
  observable. E.g. read the rollback script end-to-end and confirm every referenced bucket/tag/image
  still exists; dry-run it in staging; `git checkout <tag> && build` twice and diff the digests;
  simulate N-1 code against the N schema; `git log <last-tag>..HEAD` vs the changelog entries.
- **Lens B — REFUTE (try to kill it):** actively hunt the counter-example that makes the finding
  *false* — an expand-contract migration that IS reversible, a `continue-on-error` that is scoped to
  a non-gating job, a "leaked" secret that is actually a public publishable key, a `latest` tag that
  is in fact pinned by digest downstream. If the refutation succeeds, the finding is **killed**.
- **Lens C — CROSS-CHECK (independent angle):** confirm from a second source — a sibling audit
  summary (secaudit on the same pipeline secret, dataaudit on the same migration, codeaudit on the
  same config drift → `cross_audit_confirmed: true`, severity +1), a second config file, or the
  deployed `/version` endpoint vs the git tag.

Quorum rule (maps onto the existing H1.1 outcomes):
- **≥2 lenses confirm** → `outcome: confirmed`, finding **survives** into the matrix at full severity.
- **A successful REFUTE (and <2 confirm)** → `outcome: falsified`, finding is **demoted to info and
  killed** from scoring, with the counter-example recorded as evidence.
- **Only 1 lens runs cleanly / lenses split** → `outcome: inconclusive`, keep the finding but cap its
  `confidence: medium`; the hinge track must escalate to a 3rd clean lens before it can score.
- Every surviving claim still carries **≥3 concrete commands that could have falsified it but didn't**
  (unchanged H1.1 rule). Banned phrases (`looks correct`, `should be fine`, `appears to work`) remain
  an automatic FAIL.

### 3. Synthesize survivors back into THIS audit's EXISTING scoring (unchanged)

After fan-out + verification, **collapse all survivors back into the linear model** — the parallel
shape is an execution detail, the OUTPUT is identical to a linear run:

1. Map each surviving finding to its owning phase and apply that phase's **exact 0-10 rubric**.
2. Apply the **Phase 19 SCORING MATRIX verbatim** — same per-phase multipliers (Migration ×4.0 and
   Rollback ×4.0 still dominate as the hinges), same `max 400`.
3. **N/A handling unchanged** — a track that finds no database (T2) marks Phase 5 N/A and it is
   excluded from BOTH numerator and denominator; same for Phase 8 flags when risk doesn't warrant one.
4. **Normalize unchanged:** `score = (raw / applicable_max) × 100`.
5. **Reversibility hard-cap unchanged:** if `reversibility_assertion == NO`, the normalized score is
   **CAPPED at 69** regardless of how many tracks passed. Parallelism never buys back reversibility.
6. Emit the same v2 `verdict.json` (H1.6 schema) and `verdict.md`. The fan-out adds nothing new to the
   schema except that `falsifiable_tests[]`, `hinge_findings[]`, `edge_cases[]`, and
   `cross_audit_links[]` are now populated per-track and merged. The grade bands (S/A/B/C/D/F),
   the Phase 20 fix pipeline, and the deploy-safety gate are untouched.

### 4. Loop-until-dry for unknown-size discovery

Three of this audit's surfaces are **unbounded** — you cannot know up front how many exist:
**migrations**, **feature flags**, and **pipeline secret references**. For each, run a
**discovery → verify → re-discover** loop instead of a fixed pass, so a release with 3 or 300
migrations is audited to completion:

```
for surface in [migrations(T2), flags(T7), pipeline-secrets(T4)]:
    seen = {}
    repeat:
        batch = enumerate <surface> not in seen        # e.g. each pending migration, each flag key, each secrets.* ref
        for item in batch:
            verify it via the ≥2-of-3 lenses (§2)       # reuse H1.1, do NOT re-grep what Phase 0 already gathered
            record survivors, kill refuted
            seen += item
        # newly-read files often reveal MORE (a migration imports another; a flag gates a second flag;
        # a workflow `uses:` a reusable workflow with its own secrets) — re-enumerate to catch them
    until batch is empty (no new items two iterations running)   # DRY
```

Bound the loop by the **Audit Verification Contract's 5-iteration cap** (shared with the
fix→re-audit loop): if a surface is still surfacing new items at iteration 5, stop, mark the
remainder `NEEDS_REVIEW` in `verdict.json`, and fire the Telegram SOS — never spin indefinitely
(zero tolerance for silent infinite loops, per Preamble §4).

> **Net effect:** same questions, same /400 matrix, same reversibility cap, same verdict — reached
> faster and more rigorously by running disjoint tracks in parallel, killing every finding that
> can't survive 2-of-3 independent lenses, and looping the unbounded surfaces (migrations / flags /
> secrets) until they run dry.

---

*"/releaseaudit v1 — Reproduce. Gate. Migrate-safely. Ship-gradually. Verify. Roll-back. The one question: when this breaks in prod, can you get back to safety in one command? /400."*
