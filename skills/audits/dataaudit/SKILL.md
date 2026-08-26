---
name: dataaudit
description: >
  Forensic data integrity audit v1 (Gestalt-Popper). 21-phase deep analysis of every data
  surface: Schema validation, Migration status, Orphaned records, Referential integrity,
  Data consistency, Type safety (runtime vs schema), Null handling, Duplicate detection,
  Cascade behavior, Backup verification, Query performance, Index coverage, Data lifecycle
  (TTL, archival), PII detection, Seed data separation, Transaction integrity, plus verdict,
  fix plan, fix execution, re-audit, and DB backup safety gate (DESTRUCTIVE audit — verifies
  backup exists before any write operation). Outputs audits/.dataaudit/verdict.json consumed by
  /apiaudit for schema-to-contract validation. Score /320. Preamble v1.0 compliant.
  Audit -> Plan -> Fix -> Re-audit.
  Use when user says "/dataaudit", "data audit", "data integrity check", "schema audit",
  "database audit", "orphaned records", "data consistency".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet"]
domain: data
phases: 21
max_score: 320
read_only: false
triggers: ["data integrity", "schema", "data audit", "database audit", "orphaned records"]
---


<!-- AUDIT-META-V2-INJECTED -->

> ## ⚠️ MANDATORY FIRST STEP — READ THE V2 META-PROTOCOL
>
> **Before doing ANYTHING else**, Read `../_shared/audit-meta-protocol-v2.md`.
>
> That file overrides any conflicting guidance below for these five aspects:
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
> Model context: this audit runs on Opus 4.7 with max effort. There is no time pressure.
> Run every test you claim to have run. Cite verbatim outputs. No exceptions.

---

# /dataaudit v1 — Forensic Data Integrity Audit (Gestalt-Popper)

> *"The other audits ask 'does it work?' I ask 'is the data TRUTHFUL?'"*

---

## DOCTRINE

You are not a database administrator. You are a **data integrity forensic pathologist**. The database is your patient — possibly harboring orphaned records, definitely accumulating inconsistencies, pretending to be healthy because queries return results. Your job is to find every broken reference, every type mismatch, every silent data loss while the application says "saved successfully."

**The 5 Laws of Data Forensics (Gestalt-Popper Synthesis):**
1. **If it saves, it's still corrupt.** A successful INSERT doesn't mean the data is valid, consistent, or referenceable. Data integrity bugs are time bombs — they explode when someone queries a relationship that was silently broken months ago.
2. **Schemas lie (Popper).** A passing migration doesn't mean the schema matches reality. Nullable columns that shouldn't be null. Missing indexes that haven't caused problems YET. FALSIFY every "schema looks good" with actual data analysis.
3. **Every orphan is a broken promise.** That user record without a profile. That order without a customer. That file reference pointing to nothing. Each orphan is a future crash, a wrong calculation, a corrupted report.
4. **Clarity before querying (Gestalt).** Before running any diagnostic query, UNDERSTAND the data model. Read CLAUDE.md, schema files, migration history. Identify the **HINGE TABLES** — the core entities that everything depends on. Users, Orders, Products. THESE tables get every phase at 10x depth.
5. **"No errors in production" means nobody checked (Popper).** FALSIFY every "data is fine" claim by actually counting orphans, checking referential integrity, and verifying type consistency.

**Gestalt Hinge Tables:** Before Phase 1, identify THE tables/collections that are the core of the data model. Users. Products. Transactions. THESE get every phase at maximum depth.

**Popper Data Falsification Categories:**
- **SCHEMA vs REALITY** — Schema says NOT NULL, but nulls exist
- **REFERENCE vs EXISTENCE** — Foreign key points to deleted record
- **TYPE vs VALUE** — Column is string, but stores JSON/numbers/booleans
- **MIGRATION vs STATE** — Migration ran, but data wasn't backfilled
- **DEV vs PROD** — Dev data clean, prod has 3 years of accumulated garbage

---

## SCOPE DETECTION (automatic)

```
EXAMPLES:
  "/dataaudit"
  -> Full 16-phase pipeline. Analyze entire database/data layer.

  "/dataaudit users table"
  -> TARGETED: only user-related tables and their relationships

  "/dataaudit after migration"
  -> POST-MIGRATION: verify data integrity after schema change

  "/dataaudit orphans"
  -> ORPHAN-FOCUSED: find all broken references

  "/dataaudit types"
  -> TYPE-FOCUSED: runtime vs schema type mismatches
```

---

## OUTPUT CONTRACT

```
audits/.dataaudit/
|-- session.log
|-- discovery/
|   |-- schema.json             # Current schema snapshot
|   |-- tables.json             # All tables/collections with counts
|   |-- relationships.json      # All foreign keys / references
|   |-- migrations.json         # Migration history and status
|-- reports/
|   |-- schema-validation.md        # Phase 1
|   |-- migration-status.md         # Phase 2
|   |-- orphaned-records.md         # Phase 3
|   |-- referential-integrity.md    # Phase 4
|   |-- data-consistency.md         # Phase 5
|   |-- type-safety.md              # Phase 6
|   |-- null-handling.md            # Phase 7
|   |-- duplicate-detection.md      # Phase 8
|   |-- cascade-behavior.md         # Phase 9
|   |-- backup-verification.md      # Phase 10
|   |-- query-performance.md        # Phase 11
|   |-- index-coverage.md           # Phase 12
|   |-- data-lifecycle.md           # Phase 13
|   |-- pii-detection.md            # Phase 14
|   |-- seed-data-separation.md     # Phase 15
|   |-- transaction-integrity.md    # Phase 16
|-- verdict.json
|-- verdict.md
|-- fix-plan.json
|-- fix-plan.md
|-- progress.json
|-- fix-log.md
```

---

## PHASE 0 — PROGRAMMATIC GATHER (HYBRID, runs FIRST, before all other phases)

> **NEW (2026-05-08, hybrid framework):** before any LLM analysis, programmatic
> tools gather every machine-checkable finding deterministically. The LLM then
> READS the resulting JSON instead of hand-grepping the codebase. Freed token
> budget is REINVESTED in deeper Popper falsification, hinge-point synthesis,
> user-need verification, and edge-case hunting.

### 0.1 Run the gather script (mandatory, FIRST step)

```bash
~/.omega/lib/audit-runner.sh data "$PROJECT_PATH" \
  --files="$FILES_MODIFIED" \
  --url="$URL" \
  --user-need="$USER_NEED_QUOTE" \
  --hinge="$HINGE_POINT" \
  --ticket="$TICKET_ID"
```

This invokes `~/.omega/lib/audit-gather/data.sh` which runs:
Convex schema parse, Prisma validate + format-check, Drizzle/TypeORM/Mongoose schema scanner, migration directory inventory, SQL file scanner with CREATE TABLE/INDEX counts, SQLite integrity_check, TypeScript type/interface inventory

Output is written to:

```
$PROJECT_PATH/audits/.dataaudit/
├── raw/                    # raw tool outputs (JSON / text per tool)
└── evidence-summary.json   # normalized findings, single source of truth for the LLM
```

When run inside a Linear-fix mission (`--ticket=ID`), the artifacts move to
`$PROJECT_PATH/audits/.linear-fix/<ID>/.dataaudit/` so multiple audits on the same
ticket can cross-reference each other (see 0.5).

### 0.2 evidence-summary.json schema

```jsonc
{
  "audit": "data",
  "tools_run": ["..."],
  "tools_skipped": [{"tool": "...", "reason": "..."}],
  "findings_total": 514,
  "findings_by_severity": {"critical": 2, "high": 17, "medium": 89, "low": 406, "info": 0},
  "findings": [
    {
      "tool": "...",
      "severity": "critical|high|medium|low|info",
      "location": "file:line[:col]",
      "rule": "...",
      "message": "...",
      "suggested_fix": "...",
      "cross_tool_confirmed": false
    }
  ],
  "metrics": { /* tool-specific quantitative data */ },
  "evidence_index": { /* paths to raw/ files for drill-down */ }
}
```

### 0.3 What you do AFTER the gather (this replaces hand-greps)

You now consume `evidence-summary.json` programmatically. You MUST:

1. **Read `evidence-summary.json` in full.** This is your evidence base.
2. **Read 3-5 critical files only** — the ones flagged as load-bearing in
   `~/.omega/state/hinge-points-<ticket>.json` (or computed via
   `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/hinge-analyzer.sh` if no ticket).
3. **DO NOT manually grep the codebase for what the gather already covered.**
   The tools have already exhaustively scanned every file. Re-running grep
   wastes tokens and produces the same evidence.
4. **DO read additional files** when (a) a finding's context is unclear from
   message+location, (b) you need to verify a Popper falsification, or
   (c) you suspect a missed edge case (Phase 2.4 below).

### 0.4 Banned operations after Phase 0

These are now forbidden because the gather already did them. If you catch
yourself about to run one, STOP and read `evidence-summary.json` first:

- ❌ `grep -rn "TODO" .` (the gather scanned for it)
- ❌ `find . -name "*.ts" | xargs wc -l` (the gather has size metrics)
- ❌ `npm audit` / `pip-audit` (the gather ran them — read the JSON)
- ❌ `eslint .` / `tsc --noEmit` / `lighthouse <url>` (already in raw/)
- ❌ Generic "let me check every file" loops (the gather's job, not yours)

You MAY still:
- ✅ Read SPECIFIC files cited in findings (verify the issue)
- ✅ Run a SPECIFIC `grep` to falsify a finding (Popper test, see Phase 2.1)
- ✅ Run a SPECIFIC tool the gather couldn't (e.g. dynamic Playwright probe
  for a flow scenario the static gather can't model)

### 0.5 Cross-audit synthesis (read sibling evidence-summary.json files)

If this audit runs as part of a Linear-fix mission, sibling audits' summaries
are at `$PROJECT_PATH/audits/.linear-fix/<TICKET>/.<other-audit-id>/evidence-summary.json`.
Read them. Use them.

Examples of high-value cross-audit findings:
- **codeaudit + secaudit** flag the same `auth.ts` line → confidence escalation,
  the file is BOTH a code-quality risk AND a security risk.
- **perfaudit + a11yaudit** on the same image → joint fix opportunity (lazy-load
  + `alt` attribute in one change).
- **apiaudit + dataaudit** on the same endpoint+table pair → contract drift
  between the API surface and the schema.
- **debugaudit + flowaudit** report the same broken page → user-flow blocker.

When you find such a confluence, mark the finding `cross_audit_confirmed: true`
in your `verdict.json` and bump severity by one level.

---

## PHASE 0: RECONNAISSANCE

> *"Know the data model before diagnosing its diseases."*

```
1. PROJECT DISCOVERY
   -> Read CLAUDE.md, README, schema files
   -> Identify: database type (SQL/NoSQL/Convex/Supabase/Firebase)
   -> Find: ORM/query layer, migration tool, backup strategy

2. SCHEMA DISCOVERY
   -> Extract complete schema (all tables/collections)
   -> Map all relationships (foreign keys, references, embedded docs)
   -> Document all indexes
   -> Identify schema version and migration state

3. DATA PROFILE
   -> Row/document counts per table/collection
   -> Date ranges (oldest record, newest record)
   -> Null percentages per column
   -> Distinct value counts for key columns
   -> This becomes the "before" for comparison
```

---

## PHASE 1: SCHEMA VALIDATION

> *"The schema is the contract. Violations are breaches."*

```
1. SCHEMA vs CODE
   -> Schema definitions match TypeScript/Python types
   -> All code-referenced columns/fields exist in schema
   -> No schema columns unused by any code path
   -> Enums in schema match enums in code

2. SCHEMA CONSTRAINTS
   -> NOT NULL constraints on required fields
   -> UNIQUE constraints where business logic demands uniqueness
   -> CHECK constraints for valid ranges/formats
   -> DEFAULT values appropriate and consistent

3. SCHEMA NAMING
   -> Consistent naming convention (snake_case vs camelCase)
   -> Table/collection names are plural or singular (not mixed)
   -> Foreign key columns named consistently ({table}_id)
   -> Boolean columns prefixed (is_, has_, can_)

4. CONVEX-SPECIFIC (if applicable)
   -> Schema defined in schema.ts with proper validators
   -> All tables in schema file (no ad-hoc table creation)
   -> Indexes defined for all query patterns
   -> Union types properly discriminated
```

---

## PHASE 2: MIGRATION STATUS

> *"A migration that ran is not a migration that succeeded."*

```
1. MIGRATION COMPLETENESS
   -> All migrations applied in order
   -> No pending migrations
   -> No failed/stuck migrations
   -> Migration history matches schema state

2. DATA BACKFILL
   -> New columns have been backfilled for existing records
   -> Default values applied to historical data
   -> Renamed columns: old data migrated to new columns
   -> Removed columns: dependent code updated

3. MIGRATION SAFETY
   -> Destructive migrations have rollback path
   -> Column drops preceded by deprecation period
   -> Table renames have redirects/aliases
   -> Large data migrations batched (not single transaction)

4. MIGRATION TESTING
   -> Migrations tested against production-like data volume
   -> Migration time estimated for production
   -> Lock contention assessed for large tables
```

---

## PHASE 3: ORPHANED RECORDS (PART OF THE HINGE)

> *"An orphan record is data without a home. It takes up space, causes errors, and nobody claims it."*

```
FOR EVERY relationship in the schema:

1. FORWARD REFERENCES
   -> Every foreign key points to an existing record
   -> Count orphans: records where reference_id exists but referenced record doesn't
   -> Identify: when were orphans created (timestamps)
   -> Root cause: missing cascade delete? Race condition? Bug?

2. BACKWARD REFERENCES
   -> Records that SHOULD have children but don't (empty collections)
   -> Users without profiles, Orders without items, etc.
   -> Are these valid (new records) or data loss (deletions that missed children)?

3. FILE/STORAGE REFERENCES
   -> Every file URL in database points to an existing file
   -> Every stored file has a database record referencing it
   -> Storage bloat: files without references (abandoned uploads)

4. SOFT-DELETE ORPHANS
   -> Soft-deleted parent still referenced by active children
   -> Soft-deleted records not filtered in all queries
   -> Restore path works (undelete doesn't create new orphans)

FALSIFY: For EVERY foreign key/reference field, run a LEFT JOIN/lookup where the referenced record is NULL. Any result > 0 = orphans exist.
```

---

## PHASE 4: REFERENTIAL INTEGRITY (PART OF THE HINGE)

> *"If A references B, B must exist. Always. Without exception. This is not negotiable."*

```
1. FOREIGN KEY ENFORCEMENT
   -> Foreign keys defined at database level (not just application)
   -> All reference columns have FK constraints (or equivalent)
   -> FK constraints match the actual relationships in code

2. CROSS-TABLE CONSISTENCY
   -> Aggregates match detail records (order_total = SUM(items))
   -> Counts match reality (user.post_count = COUNT(posts WHERE user_id))
   -> Statuses consistent (order.status matches latest order_event)

3. TEMPORAL CONSISTENCY
   -> created_at <= updated_at (always)
   -> Child records not created before parent
   -> Sequential records in correct order
   -> No future timestamps (except scheduled events)

4. BUSINESS RULE INTEGRITY
   -> Business invariants hold in actual data
   -> Unique-per-context rules respected (one active subscription per user)
   -> Mutually exclusive states not both true
   -> Required relationships present (every order has at least one item)

FALSIFY: Write queries to check EVERY business invariant. If any returns rows, referential integrity is broken.
```

---

## PHASE 5: DATA CONSISTENCY

> *"Same data, two places, two different values. Which one is right? Nobody knows."*

```
1. DENORMALIZATION CONSISTENCY
   -> Cached/denormalized values match source of truth
   -> user.full_name matches user.first_name + user.last_name
   -> Aggregate fields match computed values
   -> Search indexes match database records

2. CROSS-SERVICE CONSISTENCY
   -> Data in auth provider matches database
   -> Payment provider records match local records
   -> External API data matches cached copies
   -> Email service contacts match user table

3. ENUM CONSISTENCY
   -> Enum values in data match current valid enums
   -> No deprecated enum values still in active records
   -> Case consistency (all lowercase or all uppercase)
   -> No typos in enum values

4. FORMAT CONSISTENCY
   -> Dates in consistent format per column
   -> Phone numbers in consistent format
   -> Emails lowercase and trimmed
   -> URLs with consistent protocol and trailing slash
```

---

## PHASE 6: TYPE SAFETY

> *"The schema says 'number.' The column contains '42', 'forty-two', null, and NaN."*

```
1. RUNTIME vs SCHEMA TYPES
   -> String columns containing JSON (should be JSON type)
   -> Number columns containing strings
   -> Boolean columns containing 0/1 instead of true/false
   -> Date columns containing string dates

2. TYPE COERCION RISKS
   -> Implicit type coercion in queries
   -> JavaScript === vs == issues in data layer
   -> String sorting on numeric columns
   -> Date comparison on string dates

3. VALIDATION GAPS
   -> Input validation matches schema constraints
   -> API accepts data that schema rejects (or vice versa)
   -> Frontend validation matches backend validation
   -> Edge cases: empty string vs null, 0 vs false, [] vs null

4. CONVEX-SPECIFIC (if applicable)
   -> v.string() / v.number() / v.boolean() match actual data
   -> v.optional() only where nulls are actually valid
   -> v.union() discriminators present in data
   -> v.id("table") references valid tables
```

---

## PHASE 7: NULL HANDLING

> *"NULL is not zero. NULL is not empty string. NULL is 'I don't know.' And 'I don't know' spreads."*

```
1. NULL PREVALENCE
   -> Null percentage per column
   -> Columns > 90% null (should these exist?)
   -> Required fields with nulls (data quality issue)
   -> Nullable fields that should be required

2. NULL SEMANTICS
   -> NULL means "not set" vs "not applicable" vs "unknown" — which?
   -> Consistent null handling across queries
   -> NULL in aggregates (COUNT, SUM, AVG behavior)
   -> NULL in comparisons (WHERE x != 'value' excludes NULLs)

3. NULL PROPAGATION
   -> Joins on nullable columns (LEFT JOIN behavior)
   -> Functions receiving null input
   -> UI displaying null values (blank, "N/A", "—")
   -> API returning null vs omitting field

4. DEFAULT vs NULL
   -> New records: default value vs NULL for optional fields
   -> Empty string vs NULL (pick one, be consistent)
   -> Zero vs NULL for numeric fields
   -> Empty array vs NULL for collection fields
```

---

## PHASE 8: DUPLICATE DETECTION

> *"Two records for the same entity. Two different states. One is wrong. Possibly both."*

```
1. EXACT DUPLICATES
   -> Identical rows (minus auto-generated IDs)
   -> Same email, same name, different user IDs
   -> Same order placed twice (idempotency failure)

2. NEAR DUPLICATES
   -> Similar names (fuzzy matching: "John Smith" vs "Jon Smith")
   -> Same entity, different casing ("john@email.com" vs "John@email.com")
   -> Same entity, different formatting ("+1-555-1234" vs "15551234")

3. DUPLICATE PREVENTION
   -> Unique indexes on natural keys
   -> Idempotency keys on write operations
   -> Upsert patterns where appropriate
   -> Deduplication in import/sync pipelines

4. MERGE STRATEGY
   -> If duplicates exist: which record wins?
   -> How to merge (keep newest, keep most complete, manual review)
   -> Dependent records updated after merge
   -> Audit trail of merges
```

---

## PHASE 9: CASCADE BEHAVIOR

> *"Delete a user. Their posts remain. Their comments remain. Their files remain. Congratulations, you have ghost data."*

```
1. DELETE CASCADES
   -> Parent deletion cascades to children appropriately
   -> No orphans created by deletion
   -> Cascade depth: deletion doesn't cascade too broadly
   -> Soft delete cascades to children

2. UPDATE CASCADES
   -> Parent key change cascades to foreign keys
   -> Status changes propagate (deactivate user -> deactivate sessions)
   -> Email change propagates to notification preferences

3. CASCADE TESTING
   -> Each cascade path tested with actual data
   -> Performance of cascade deletions (large trees)
   -> Cascade doesn't trigger infinite loops
   -> Cascade respects soft delete boundaries

4. CASCADE ALTERNATIVES
   -> RESTRICT where deletion should be blocked
   -> SET NULL where orphan is acceptable
   -> SET DEFAULT for fallback values
   -> NO ACTION with application-level handling
```

---

## PHASE 10: BACKUP VERIFICATION

> *"Backups that have never been restored are not backups. They are prayers."*

```
1. BACKUP EXISTENCE
   -> Automated backups configured
   -> Backup frequency appropriate (hourly, daily, weekly)
   -> Point-in-time recovery available
   -> Backups stored in separate location/region

2. BACKUP COMPLETENESS
   -> All tables/collections included
   -> File storage backed up (not just database)
   -> Configuration and secrets backed up (separately, securely)
   -> Migration state included in backup

3. RESTORE TESTING
   -> Last successful restore test (date)
   -> Restore time acceptable for business needs
   -> Restored data matches original (spot check)
   -> Application works against restored database

4. BACKUP SECURITY
   -> Backups encrypted at rest
   -> Access to backups restricted
   -> Backup retention policy defined
   -> Old backups purged on schedule
```

---

## PHASE 11: QUERY PERFORMANCE

```
1. SLOW QUERIES
   -> Queries > 100ms identified
   -> Full table scans detected
   -> Missing index suggestions
   -> N+1 query patterns in code

2. QUERY PATTERNS
   -> SELECT * usage (overfetching)
   -> Large result sets without pagination
   -> Unnecessary JOINs
   -> Subqueries that could be JOINs (or vice versa)

3. WRITE PERFORMANCE
   -> Batch inserts vs single inserts
   -> Transaction scope (not too broad, not too narrow)
   -> Lock contention on hot tables
   -> Write amplification from indexes
```

---

## PHASE 12: INDEX COVERAGE

```
1. MISSING INDEXES
   -> Columns in WHERE clauses without indexes
   -> Columns in JOIN conditions without indexes
   -> Columns in ORDER BY without indexes
   -> Foreign key columns without indexes

2. UNUSED INDEXES
   -> Indexes never used in queries (dead weight)
   -> Duplicate indexes (same columns, different names)
   -> Indexes on low-cardinality columns (boolean, enum with 3 values)

3. INDEX OPTIMIZATION
   -> Composite index column order matches query patterns
   -> Covering indexes for read-heavy queries
   -> Partial indexes for filtered queries
   -> Expression indexes for computed lookups
```

---

## PHASE 13: DATA LIFECYCLE

> *"Data that lives forever costs forever. And eventually, it violates privacy laws."*

```
1. TTL / EXPIRATION
   -> Session data has TTL
   -> Temporary tokens expire
   -> Cache entries have TTL
   -> Logs rotate/expire

2. ARCHIVAL
   -> Historical data archived (not in active tables)
   -> Archive is queryable if needed
   -> Archive doesn't break referential integrity
   -> Migration to archive is automated

3. RETENTION POLICY
   -> Data retention policy documented
   -> Policy implemented in code/database
   -> GDPR/CCPA right-to-deletion supported
   -> Account deletion removes all personal data

4. GROWTH PROJECTIONS
   -> Current growth rate per table
   -> Projected size in 6 months, 1 year
   -> Tables that will need partitioning/sharding
   -> Cost implications of data growth
```

---

## PHASE 14: PII DETECTION

> *"If you're storing it, you're responsible for it. If you don't know you're storing it, you're negligent."*

```
1. PII INVENTORY
   -> All columns containing personal data identified
   -> Email addresses, phone numbers, names, addresses
   -> IP addresses, user agents, device IDs
   -> Financial data (card numbers, bank details)
   -> Health data, biometric data

2. PII PROTECTION
   -> PII encrypted at rest
   -> PII encrypted in transit
   -> PII access logged
   -> PII masked in non-production environments

3. PII IN UNEXPECTED PLACES
   -> Log files containing PII
   -> Error messages containing PII
   -> Search indexes containing PII
   -> Cache entries containing PII
   -> Backup files containing PII (encrypted?)

4. PII COMPLIANCE
   -> Consent recorded for data collection
   -> Data processing purpose documented
   -> Third-party data sharing documented
   -> Right to access / export / delete implemented
```

---

## PHASE 15: SEED DATA SEPARATION

> *"Test data in production. Production data in tests. Both are disasters."*

```
1. SEED vs PRODUCTION
   -> No seed/test data in production database
   -> No "test@example.com" accounts in production
   -> No "Lorem ipsum" content in production
   -> No default passwords in production

2. PRODUCTION DATA SAFETY
   -> Production data not in development databases (or anonymized)
   -> Staging environment uses anonymized production data
   -> No production credentials in test fixtures

3. SEED DATA QUALITY
   -> Seed data covers edge cases
   -> Seed data is realistic (not just "test test test")
   -> Seed data is reproducible (deterministic)
   -> Seed data doesn't conflict with production patterns
```

---

## PHASE 16: TRANSACTION INTEGRITY

> *"Half a transaction is worse than no transaction. It's a lie the database tells itself."*

```
1. TRANSACTION BOUNDARIES
   -> Multi-step operations wrapped in transactions
   -> Payment + order creation atomic
   -> User creation + profile creation atomic
   -> File upload + database record atomic (or compensating)

2. CONCURRENCY SAFETY
   -> Optimistic locking on concurrent updates
   -> No lost updates (last-write-wins without detection)
   -> Counter increments are atomic
   -> Inventory checks + decrements are atomic

3. ERROR RECOVERY
   -> Failed transactions fully rolled back
   -> Partial failures detected and handled
   -> Compensating transactions for distributed operations
   -> Dead letter queue for failed async operations

4. IDEMPOTENCY
   -> Retry-safe operations (same input, same result)
   -> Idempotency keys on payment operations
   -> Webhook handlers idempotent
   -> Import operations handle re-runs gracefully
```

---

## PHASE H1 — HYBRID SYNTHESIS (Popper / hinge / user-need / edge cases / cross-audit)

> **NEW (2026-05-08, hybrid framework, runs immediately before VERDICT):**
> "H1" = Hybrid step 1 of the synthesis layer that pairs with Phase 0's
> programmatic gather. It does NOT renumber existing phases; it sits between
> the audit's last domain phase and the VERDICT phase. Between the per-domain
> phases above and the VERDICT phase, you must run this 5-step synthesis.
> The token budget freed by Phase 0's deterministic gather is REINVESTED here
> — depth of analysis is what increases. This phase does NOT replace any
> earlier phase; it deepens them.

### 2.1 Popper falsification per finding (mandatory)

For every finding in `evidence-summary.json.findings[]` (start with `severity ∈
{critical, high}`, then go down as time/budget allows), try to PROVE the tool
is wrong. Each falsification produces a `falsifiable_tests[]` entry in
`verdict.json`:

```jsonc
{
  "claim": "ts-prune says src/auth/legacy.ts:42 export `signWithOldKey` is unused",
  "test_command": "grep -rn 'signWithOldKey' --include='*.ts' --include='*.tsx' --include='*.test.ts' --include='*.spec.ts' . | grep -v 'src/auth/legacy.ts'",
  "expected": "0 matches → claim TRUE, finding stands",
  "actual": "0 matches found",
  "outcome": "confirmed"
}
```

Outcomes:
- `confirmed` — Popper test FAILED to falsify → finding stands, often promoted
- `falsified` — Popper test produced a counter-example → demote to `info` and
  add `falsified_at: <evidence>` to the finding
- `inconclusive` — test could not run cleanly → keep severity, mark
  `confidence: medium` on this finding

**The rule:** every CLAIM in the audit (PASS or FAIL) MUST cite ≥3 concrete
commands that COULD have falsified it but didn't. Banned phrases (`looks
correct`, `should be fine`, `appears to work`) → automatic FAIL of the audit.

Common falsification patterns by category:

| Tool says | Popper test |
|---|---|
| `unused export` (ts-prune, vulture) | `grep` for the symbol in tests/, integration/, and dynamic imports (e.g. `import("...")`) |
| `unused dependency` (depcheck) | `grep` `package.json` scripts + `find . -type f -exec grep -l <pkg>` |
| `circular dep` (madge) | Read all files in the cycle — is the cycle real or a tooling artifact (e.g. type-only re-export)? |
| `console.error` / `console.warn` (debugaudit) | Reproduce the user flow that triggers it; if not reachable from any UI path, demote to info |
| `npm audit HIGH` (sec) | Check if the vulnerable code path is actually called in your codebase; not all transitive vulns are exploitable |
| `LCP > 2.5s` (perf) | Re-run lighthouse twice; check if it's a one-off (cold cache, network blip) or systematic |
| `axe-core color-contrast` (a11y) | Manually compute the ratio; some palettes hit 4.4 vs 4.5 — fixable in tokens |
| `missing canonical` (seo) | Check if the page is intentionally not canonical (paginated, faceted) before flagging |
| `unauthenticated endpoint` (api) | Read the route handler — is auth enforced via middleware not visible to the static scan? |
| `orphaned record` (data) | Verify the FK is supposed to cascade, or confirm the parent was deleted by an actual user action |

### 2.2 Hinge cross-reference (10× scrutiny on load-bearing findings)

The HINGE TABLES of this audit is the locus of maximum risk/value. Compute or
read:

```bash
# If a hinge file already exists for this ticket, use it:
HINGE_FILE="$HOME/.omega/state/hinge-points-${TICKET_ID:-default}.json"

# Otherwise compute it on the fly:
${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/hinge-analyzer.sh "$PROJECT_PATH" --audit=data --user-need="$USER_NEED_QUOTE" \
  > "$HINGE_FILE"
```

For each finding in `evidence-summary.json`, mark
`is_load_bearing: true` IFF its file matches a hinge entry, then apply 10×
scrutiny to those findings:

- 5× more falsification attempts (Phase 2.1)
- 3× more edge-case hunts (Phase 2.4)
- Mandatory read of the entire hinge file (not just the flagged line)
- Mandatory read of all DIRECT callers of the hinge function/symbol
- Mandatory read of all DIRECT callees from the hinge

Output `hinge_findings[]` in `verdict.json`:

```jsonc
{
  "finding_id": "F-042",
  "is_load_bearing": true,
  "hinge_reference": "<hinge term + file>",
  "additional_scrutiny": "verified all 7 callers, all 4 callees, all 12 tests; reproduced bug in 3 of them",
  "confidence_after_scrutiny": "high"
}
```

### 2.3 User-need verification (`--user-need` quote)

If the audit was dispatched with `--user-need="<verbatim user complaint>"`,
every finding MUST be evaluated against it. The user-need is the ground truth
for what counts as "audit succeeded".

For each finding, ask:
- "If a user reported THIS exact issue verbatim, would this finding be the
  cause?"
- "Does fixing this finding make the user-need quote no longer true?"

Findings that DO NOT relate to user-need:
- Get demoted by one severity level UNLESS they are load-bearing
  (Phase 2.2), in which case they retain severity (load-bearing is
  user-need-independent).
- Are still reported, but flagged `user_need_relevance: "tangential"`.

Findings that DO relate to user-need:
- Get the highest priority in the fix plan
- Are listed first in `user_need_match.findings[]` of `verdict.json`

```jsonc
{
  "user_need_match": {
    "addressed": true,
    "user_need_quote": "<verbatim>",
    "rationale": "Finding F-007 (axe-core: input has no associated label) directly causes the user-reported behavior 'I can't tab to the search field'. Fix removes the cause.",
    "findings": ["F-007", "F-019"],
    "untouched_findings_relevant_to_user_need": []
  }
}
```

If `addressed: false`, the audit MUST score below 90/100 even if all phases
otherwise pass. The user's actual problem is the only correct success metric.

### 2.4 Edge case hunting (mandatory for top findings)

For each top-5 finding (sorted by severity × cross-audit-confirmed × hinge),
generate ≥2 edge cases the tool may have missed. The static analyzer or
runtime probe checked the code at rest; you must imagine motion.

Patterns:
- "If user does X under condition Y..." — concurrency, race, double-submit
- "Tool checked the file at rest, but at runtime + concurrent..." — async/await
  ordering, state mutations, cache races
- "Static scan saw the import, but the dynamic require()..." — code-splitting,
  feature-flag gated, lazy()
- "i18n locale switch..." — strings missing in non-default locale
- "Network flake mid-request..." — partial state, retry idempotency
- "User logs out mid-flow..." — session expiration, token refresh
- "Timezone boundary (DST, UTC midnight)..." — date math
- "Data shape: empty array, null, undefined, single-element..." — boundary

Output `edge_cases[]` in `verdict.json`:

```jsonc
{
  "finding_id": "F-007",
  "scenario": "User pastes a multi-line value into the search field; the
    `<input>` strips newlines silently and submits a partial query",
  "covered_by_existing_test": false,
  "evidence_gathered": "Manual repro on prod URL; screenshot at .data/edge-evidence/F-007.png",
  "fix_includes_coverage": true
}
```

### 2.5 Cross-audit synthesis (re-read sibling summaries from Phase 0.5)

Now that you have your own `verdict.json` draft, do a final pass with sibling
audits' findings open in context:

1. For each of YOUR top-5 findings: is the same file/line/symbol flagged in any
   sibling audit?
2. If yes → escalate confidence (`cross_audit_confirmed: true`), bump severity
   one level (low→medium, medium→high, high→critical, critical stays).
3. For each sibling top-5 finding: is the same file/line/symbol relevant to
   YOUR audit's domain? If yes, add it to YOUR findings as
   `tool: "cross-audit:<sibling-name>"` with proper severity.
4. Write `cross_audit_links[]` in `verdict.json` summarizing the matches.

```jsonc
{
  "cross_audit_links": [
    {
      "this_finding_id": "F-007",
      "sibling_audit": "secaudit",
      "sibling_finding_id": "F-013",
      "shared_location": "src/auth/login.tsx:42",
      "joint_fix_recommended": true
    }
  ]
}
```

### 2.6 Final verdict.json schema (hybrid v2)

```jsonc
{
  "audit": "data",
  "score": 100,
  "score_raw": "<raw>/<denominator>",
  "score_normalized": 100,
  "confidence": "high|medium|low",
  "skill_used": "data",
  "user_need_match": { ... },                   // §2.3
  "falsifiable_tests": [ ... ],                   // §2.1
  "hinge_findings": [ ... ],                      // §2.2
  "issues_found_and_fixed": [
    {
      "id": "FIX-001",
      "finding_id": "F-007",  // back-ref to evidence-summary.json
      "before": "<state>",
      "after": "<state>",
      "verification": "<command + output>"
    }
  ],
  "edge_cases": [ ... ],                          // §2.4
  "cross_audit_links": [ ... ],                   // §2.5
  "evidence_summary_path": "$PROJECT_PATH/audits/.dataaudit/evidence-summary.json",
  "confidence_basis": "Why I'm confident (or not). Cite Popper test counts, hinge scrutiny depth, edge-case coverage, cross-audit confirmations.",
  "banned_phrase_check": "passed (no occurrences of `looks correct`, `should be fine`, `appears to work`, `streamlined`, `to save time`)"
}
```

### 2.7 Score gating (hybrid threshold)

A 100/100 score is now blocked unless:

1. ✅ All `critical` and `high` findings are fixed OR have an explicit
   `non_issue_justification` of ≥50 words backed by Popper evidence.
2. ✅ All load-bearing findings (Phase 2.2) confirmed via Popper falsification.
3. ✅ `user_need_match.addressed = true` with verbatim quote of the user-need.
4. ✅ ≥3 falsifiable tests cited per phase (Phase 2.1).
5. ✅ ≥2 edge cases generated per top-5 finding (Phase 2.4).
6. ✅ Cross-audit synthesis attempted (Phase 2.5) — links may be empty if no
      siblings exist, but the array MUST be present.
7. ✅ `confidence_basis` populated with non-trivial reasoning.

Below threshold → score < 100, fix-and-reaudit loop kicks in (existing R-6 flow
in the FIX EXECUTION / RE-AUDIT phases below). The loop is BOUNDED at 5
iterations per the Audit Verification Contract; on iteration 5 if still failing,
emit `confidence: low` and surface as `pending` in `.done.json`.

---

## PHASE 17: VERDICT

Score each phase 0-10, weight by severity:

```
SCORING MATRIX (320 max):
  Phase  1  (Schema Validation)      x 2.5  = max 25
  Phase  2  (Migration Status)       x 2.0  = max 20
  Phase  3  (Orphaned Records)       x 3.5  = max 35  ** HINGE **
  Phase  4  (Referential Integrity)  x 3.5  = max 35  ** HINGE **
  Phase  5  (Data Consistency)       x 2.5  = max 25
  Phase  6  (Type Safety)           x 2.0  = max 20
  Phase  7  (Null Handling)          x 1.5  = max 15
  Phase  8  (Duplicate Detection)    x 2.0  = max 20
  Phase  9  (Cascade Behavior)       x 2.0  = max 20
  Phase 10  (Backup Verification)    x 2.0  = max 20
  Phase 11  (Query Performance)      x 1.5  = max 15
  Phase 12  (Index Coverage)         x 1.5  = max 15
  Phase 13  (Data Lifecycle)         x 1.5  = max 15
  Phase 14  (PII Detection)         x 2.0  = max 20
  Phase 15  (Seed Data Separation)   x 1.0  = max 10
  Phase 16  (Transaction Integrity)  x 1.0  = max 10
                                     TOTAL  = max 320

NORMALIZE: score = (raw / 320) x 100

GRADE:
  90-100: S — Pristine. Every reference valid, every type correct, every transaction safe.
  80-89:  A — Solid. Minor inconsistencies, no orphans, good practices.
  70-79:  B — Adequate. Some orphans, minor type issues, data mostly trustworthy.
  60-69:  C — Concerning. Significant orphans or inconsistencies, manual fixes needed.
  50-59:  D — Degraded. Data quality unreliable, reports cannot be trusted.
  <50:    F — Corrupt. Referential integrity broken, orphans widespread, data untrustworthy.
```

---

## PHASE 18: FIX PLAN (automatic)

```
Sort: CRITICAL -> HIGH -> MEDIUM -> LOW
Group by impact (one fix may resolve multiple integrity issues)
Priority: referential integrity > orphans > types > consistency > cosmetic
Generate fix tasks with table:column specificity
Save to audits/.dataaudit/fix-plan.json + fix-plan.md
```

---

## PHASE 19: FIX EXECUTION (automatic)

```
Sequential per fix group.

─── SAFETY GATE: DO NO HARM (MANDATORY before EVERY fix) ──────────────

The audit MUST NOT introduce new bugs. A fix that breaks the code is worse than
the original finding. Every fix goes through this gate BEFORE commit.

PRE-FIX ANALYSIS (before writing ANY code):
  a. Read the ENTIRE target file (not just the target line)
  b. SCOPE COLLISION CHECK — if adding/renaming a variable or import:
     → Grep the ENTIRE file for that name (all occurrences)
     → Check: is this name already used as a local, parameter, or reassigned?
     → Check: does this name get shadowed later in the same scope?
     → If collision found → use a different name or fully-qualified reference
  c. IMPORT SHADOW CHECK — if adding `from X import Y` inside a function:
     → This makes Y a LOCAL variable for the ENTIRE function scope
     → If Y is also used from module-level import → UnboundLocalError
     → Fix: use the module-level import, don't re-import locally
  d. CROSS-REFERENCE CHECK — if modifying a function signature, class, or export:
     → Grep the ENTIRE project for all callers/importers of that symbol
     → Verify every caller still works with the new signature
     → If callers exist outside the file → update them ALL or don't change

POST-FIX VERIFICATION (after writing code, BEFORE commit):
  a. SYNTAX CHECK:
     → Python: `python -c "import ast; ast.parse(open('FILE').read())"`
     → JS/TS: `npx tsc --noEmit` or `node -c FILE`
  b. IMPORT CHECK — verify the module actually loads without error:
     → Python: `python -c "import MODULE"` (catches UnboundLocalError, NameError, etc.)
     → JS: `node -e "require('./FILE')"`
  c. RUNTIME SMOKE TEST — if the project has a service (bot, server, API):
     → Start it briefly and verify it doesn't crash on init
     → Python: `timeout 10 python main.py` or systemctl restart + is-active check
     → Node: `timeout 10 node server.js` or `npm run build`
     → If service crashes → git revert HEAD → mark NEEDS_REVIEW
  d. TEST SUITE — if tests exist:
     → Run the relevant test file(s): `pytest FILE -x` / `vitest run FILE`
     → If tests fail → git revert HEAD → investigate

IF ANY POST-FIX CHECK FAILS:
  → `git revert HEAD` immediately
  → Log the failure in audits/.dataaudit/fix-log.md with exact error
  → Mark as NEEDS_REVIEW (never retry same approach blindly)
  → Try alternative approach OR skip this fix

────────────────────────────────────────────────────────────────────────

FOR EACH FIX TASK (in priority order):
  a. Read the ENTIRE target file (full context)
  b. Run PRE-FIX ANALYSIS (scope collision, import shadow, cross-reference)
  c. BACKUP before any data modification
  d. Apply fix (with transaction safety)
  e. Run POST-FIX VERIFICATION (syntax, import, smoke test, tests)
  f. If all green → commit: data(dataaudit): FIX-XXX description
  g. If any red → revert from backup → log → mark NEEDS_REVIEW
  h. Verify: no regression in other tables
```

---

## PHASE 20: RE-AUDIT (automatic)

```
1. SERVICE HEALTH GATE (mandatory):
   → If project has systemd service: restart it, wait 10s, check is-active + logs for errors
   → If project has build step: full build must pass
   → If project has tests: full test suite must pass
   → If ANY fails: identify which fix broke it, revert

2. Re-run all FAILING phases. Compare before/after.
3. Loop until score >= 80 or remaining items are NEEDS_REVIEW.
```

---

## PARALLEL EXECUTION STRATEGY

```
WAVE 1 (discovery -- sequential):
  Phase 0: Reconnaissance

WAVE 2 (schema -- parallel):
  Phase 1: Schema Validation    ]
  Phase 2: Migration Status     ] Schema-level
  Phase 6: Type Safety          ] analysis
  Phase 7: Null Handling        ]

WAVE 3 (integrity -- parallel):
  Phase 3: Orphaned Records     ]
  Phase 4: Referential Integrity] Core integrity
  Phase 5: Data Consistency     ] checks
  Phase 8: Duplicate Detection  ]

WAVE 4 (operations -- parallel):
  Phase 9: Cascade Behavior      ]
  Phase 10: Backup Verification  ]
  Phase 11: Query Performance    ] Operational
  Phase 12: Index Coverage       ] analysis
  Phase 16: Transaction Integrity]

WAVE 5 (governance -- parallel):
  Phase 13: Data Lifecycle       ]
  Phase 14: PII Detection        ] Governance
  Phase 15: Seed Data Separation ] checks

4 parallel agents per wave = 16 phases in 5 sequential waves.
```

---

## CROSS-COMMAND BRIDGE

```
/dataaudit finds query issues -> references /perfaudit findings
/dataaudit finds schema issues -> references /codeaudit findings
/dataaudit finds PII issues -> references /secaudit findings
/dataaudit finds API data issues -> references /apiaudit findings

THE QUALITY ARSENAL:
  /codeaudit    -> Is the code SOLID?           (preventive)
  /flowaudit    -> Does the experience WORK?    (preventive)
  /uiuxaudit    -> Is the interface BEAUTIFUL?  (preventive)
  /debugaudit   -> What is BROKEN right now?    (detective)
  /featureaudit -> Is the product COMPLETE?     (strategic)
  /perfaudit    -> Is it FAST enough?           (detective)
  /secaudit     -> Is it SECURE?                (detective)
  /a11yaudit    -> Can EVERYONE use it?         (inclusive)
  /seoaudit     -> Can search engines FIND it?  (growth)
  /dataaudit    -> Is the data INTACT?          (integrity)
  /apiaudit     -> Is the API RELIABLE?         (contract)
  /copyaudit    -> Does the copy DELIVER?       (messaging)
  /dxaudit      -> Can devs SHIP efficiently?   (velocity)

  Together: nothing escapes. Every dimension covered.
```

---

## LAWS

1. **Data is the product.** Code can be rewritten. Data loss is permanent.
2. **Every orphan is a future bug.** Today's orphan is tomorrow's null pointer exception.
3. **Referential integrity is non-negotiable.** If you can't enforce it at the database level, enforce it at the application level. But enforce it.
4. **NULL is not nothing — it's uncertainty.** Handle it explicitly, consistently, and intentionally.
5. **Backups you haven't restored are hopes, not backups (Popper).** Test the restore.
6. **PII is a liability.** Every personal data field is a GDPR/CCPA obligation. Know where it all is.
7. **Transactions are all-or-nothing.** Half a transaction is corruption with extra steps.

---

*"/dataaudit v1 — Validate. Verify. Protect. Every record, every reference, every table. /320."*

---

## COMPLIANCE & CRITICAL ADDENDA (v1.1 — 2026-04-14T10:35:36Z)

### Quality Arsenal Preamble Compliance

This audit implements contracts defined in `../_shared/QUALITY-ARSENAL-PREAMBLE.md` v1.0:

- ✅ **Gestalt-Popper doctrine** — hinge point, falsification, evidence chain, adversarial thinking
- ✅ **Concurrency lock** — canonical runner `flock` at `audits/.dataaudit/.runner.lock`
- ✅ **5-iteration cap** — fix-and-reaudit loop bounded at 5 iterations (rule 43 step 8b alignment). On cap: NEEDS_REVIEW + Telegram SOS. No silent infinite loops.
- ✅ **Scoped invocation flags** — `--url=`, `--files=`, `--scope=`, `--ticket=`, `--no-fix`, `--focus=`
- ✅ **Non-UI context gate** — Non-UI contexts: /dataaudit runs on any project with a DB. N/A for projects without data persistence.
- ✅ **Output contract verification** — emits `audits/.dataaudit/verdict.json`, `verdict.md`, `fix-plan.json`, `fix-plan.md`, `iterations.md`, `progress.json`, `telemetry.json`, `fix-log.md`. Output gate runs at end; missing/malformed files = audit did NOT succeed.
- ✅ **Telegram progress notifications** — `start` / `progress` (every 3 phases) / `iteration` / `verdict` / `abort` / `sos` events via `~/.omega/bin/audit-notify.sh`
- ✅ **Discovery drift check** — on resumed runs, if `audits/.dataaudit/discovery/` > 1h old, re-verify inventory or abort with user-confirm
- ✅ **Self-telemetry** — `audits/.dataaudit/telemetry.json` emitted at completion (duration, tokens, phases, fixes, model, preamble_version)
- ✅ **Deprecation registry** — cross-references checked against `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/DEPRECATED.md`; stale refs flagged as findings
- ✅ **Rule-46 compliance** — NO `--quick`/`--streamlined`/`--lightweight` variants. Narrower scope uses `--focus <area>` flags with FULL phase depth. Orchestrator prompts containing rule-46 banned phrases are REFUSED.
- ✅ **Score normalization** — raw score divided by applicable-phase-max × 100 = /100 normalized score
- ✅ **preamble_version** — emitted as `"1.0"` in verdict.json for `/metaudit` compliance scan

### Audit-Specific Critical Addendum — DB Backup Gate (CRITICAL) + Data Sampling

**DB Backup Gate (MANDATORY before any schema migration fix):**

```
Phase 23 fix gate extension for /dataaudit:

1. Detect DB provider:
   - Convex: schema.ts + convex/ dir
   - Prisma: prisma/schema.prisma + DATABASE_URL env
   - Drizzle: drizzle.config.ts + schema in src/db/schema.ts
   - Raw SQL: migrations/*.sql + psql/mysql CLI

2. BEFORE any schema migration fix, create snapshot:
   - Convex:    npx convex export --path audits/.dataaudit/snapshot-$(date +%s).zip
   - Prisma PG: pg_dump "$DATABASE_URL" > audits/.dataaudit/snapshot-$(date +%s).sql
   - Prisma SQLite: cp prisma/dev.db audits/.dataaudit/snapshot-$(date +%s).db
   - Raw SQL:  apply DB-specific dump command
   - If snapshot cannot be created → ABORT fix, do NOT proceed

3. Apply schema migration fix

4. Post-fix validation (per preamble §6 output gate + integration smoke §11):
   - Run any existing migration tests
   - Run a representative query against each changed table
   - If ANY post-fix validation fails:
     - Restore snapshot immediately
     - git revert HEAD
     - Mark as NEEDS_REVIEW with snapshot path
     - Telegram SOS

5. Retain snapshots for 7 days, then auto-clean `audits/.dataaudit/snapshot-*`
```

**Data sampling (beyond schema):**
- Per table, sample 100 rows (random if > 100, else all)
- Check: NULL in declared NOT NULL columns, orphan foreign keys, type coercion surprises
- Emit findings as runtime-data integrity issues (distinct from schema-definition issues)

### Scoring addenda

Normalized score = (raw_score / applicable_raw_max) × 100.
Applicable max excludes phases marked N/A for project type (see preamble §5).
`verdict.json.preamble_version = "1.0"` is MANDATORY. Missing field = /metaudit flags as non-compliant.

### /metaudit Compliance Badge

Run `/metaudit --focus arsenal` to verify this audit against the 11-point preamble checklist. Target score: 11/11.

---

*v1.1 compliance round 2026-04-14 — shared-blindspot gaps closed via preamble, audit-specific critical addendum applied.*

---

## § 100/100 COMPLIANCE CERTIFICATE — v1.2 — 2026-04-14T10:43:29Z

> *Final gap-closure layer. Every finding from the prior /flowaudit-of-all-audits analysis (dataaudit-specific section) is now explicitly resolved below.*

### Compliance Matrix (machine-verifiable by /metaudit)

```json
{
  "audit": "dataaudit",
  "preamble_version": "1.0",
  "compliance_version": "1.2",
  "compliance_score": 100,
  "shared_blindspots_closed": {
    "S1_concurrency_lock": true,
    "S2_reaudit_cap_5_iter": true,
    "S3_non_ui_context_handling": true,
    "S4_telegram_progress": true,
    "S5_scoped_invocation_flags": true,
    "S6_output_contract_verification": true,
    "S7_self_telemetry": true,
    "S8_phase_count_rationale_documented": true,
    "S9_discovery_drift_check": true,
    "S10_deprecation_registry_reference": true
  },
  "audit_specific_gaps_closed": true,
  "rule_46_compliant": true,
  "integration_smoke_test_gate": true,
  "rate_limit_safety": true,
  "last_updated": "2026-04-14T10:43:29Z"
}
```

### Gap closures (pre-100 findings → now resolved)

**Gap 1: DB backup retention → RESOLVED**
```
audits/.dataaudit/snapshots/ retention policy:
  - Keep last 5 snapshots always
  - Beyond 5: keep only if < 7 days old
  - Auto-prune older via:
    find audits/.dataaudit/snapshots/ -mtime +7 -type f | sort | head -n -5 | xargs rm -f
  - Cron: runs before each /dataaudit invocation
Snapshot naming: snapshot-{db_type}-{timestamp}.{zip|sql|db}
Snapshot size logged to telemetry.json: snapshot_sizes_bytes[]
```

**Gap 2: Data sampling algorithm → RESOLVED**
```
For each table:
  Count total rows
  Sample strategy:
    < 100 rows: sample all
    100 - 10k rows: sample 100 random (SQL RANDOM() / Convex paginate)
    > 10k rows: sample 100 stratified (first, last, 98 uniform random)
  Per-row validation checks:
    - Declared NOT NULL fields actually non-null
    - Foreign keys reference existing rows (SELECT FROM target WHERE id = <fk>)
    - Type coercion: cast attempts (e.g. column is VARCHAR but contains JSON that fails parse)
    - Enum compliance: if schema declares enum, value must be in set
  Emit findings as DATA_INTEGRITY_RUNTIME distinct from DATA_INTEGRITY_SCHEMA
```

**Gap 3: Pre-fix validation matrix → RESOLVED**
```
Before applying any schema migration fix:
  1. Snapshot created (Gap 1 above)
  2. Dry-run validation:
     - Convex: `npx convex dev --once --skip-push` (schema compile)
     - Prisma: `npx prisma validate`
     - Drizzle: `npx drizzle-kit check`
  3. If dry-run fails: do NOT apply fix, flag as NEEDS_REVIEW
Post-fix:
  4. Run same dry-run on new schema
  5. Run data sampling (Gap 2) on all changed tables
  6. If any data check fails:
     - Restore from snapshot
     - git revert HEAD
     - Mark NEEDS_REVIEW with snapshot path in fix-plan.json
     - Telegram SOS
```

**Gap 4: Non-UI contexts → RESOLVED:**
/dataaudit runs on any project with persistence layer. N/A only for projects without DB.

### Final score calculation

```
score_100 = (raw_score / applicable_raw_max) × 100
           = fully-compliant audit at 100 achievable when:
             - All findings resolved OR
             - Remaining findings are declared NEEDS_REVIEW with user-confirm

After v1.2 compliance round:
  - All shared blindspots (S1-S10): closed
  - Audit-specific pre-100 gaps: closed explicitly above
  - verdict.json.preamble_version = "1.0" verification
  - /metaudit --focus arsenal target: 11/11 preamble points × 14 audits = 154/154
```

### Verification command

```bash
# Run /metaudit to verify this audit's 100/100 claim:
/metaudit --focus arsenal --scope="dataaudit only"
# Expected result:
#   preamble_compliance_score: 11/11 (full)
#   audit_specific_compliance: 100/100
#   overall: 100/100
```

---

*dataaudit v1.2 — 100/100 compliance achieved via QUALITY-ARSENAL-PREAMBLE.md v1.0 + audit-specific gap closures above. Certified 2026-04-14.*

---

## MANDATORY BEFORE/AFTER VERIFICATION (v1.1)

**Read `../_shared/AUDIT-VERIFICATION-CONTRACT.md` before ANY fix execution.**

Every fix MUST follow the "Do No Harm" protocol:

1. **PRE-FIX BASELINE** — grep all references, capture functional state, save to `audits/.<audit-id>/baseline/`.
2. **APPLY FIX** — normal execution.
3. **POST-FIX CHECK** — repeat every baseline check. If any PASSED→FAILED transition occurs, revert immediately.
4. **BREAKAGE SCAN** — grep for old paths across ecosystem, must return 0 non-ephemeral hits.
5. **BEFORE/AFTER MATRIX** — produce `audits/.<audit-id>/before-after.md` with functional status table per affected item.

**An audit that breaks 1 working thing is WORSE than no audit.** Do NOT claim "done" without `before-after.md` showing zero regressions.

---

## Dynamic-Workflow Orchestration (v2)

> *"Sixteen integrity questions, asked at once, each answer cross-examined by three witnesses before it is allowed into the verdict."*

This section governs **HOW** the audit executes when run. It does not add, remove, reorder, or reweight any phase above, nor touch the /320 SCORING MATRIX (Phase 17) or the verdict format — it changes the *execution topology* from linear-sequential to **fan-out → adversarially-verify → synthesize → loop-until-dry**. The Gestalt clarity gate, the HINGE TABLES doctrine, and the 5 Popper falsification categories (SCHEMA-vs-REALITY, REFERENCE-vs-EXISTENCE, TYPE-vs-VALUE, MIGRATION-vs-STATE, DEV-vs-PROD) remain the spine; this only makes them run concurrently and survive cross-examination.

### 1. Decompose into INDEPENDENT parallel tracks (fan-out via the Workflow tool)

After Phase 0 (programmatic gather) + Phase 0 RECON establish the data model and the HINGE TABLES, dispatch the domain phases as **concurrent Workflow tracks** instead of walking them top-to-bottom. The existing PARALLEL EXECUTION STRATEGY waves are the contract — promote each wave-member to its own Workflow branch so file-disjoint phases run truly in parallel:

| Track | Phases (unchanged) | Why independent | Shared input |
|---|---|---|---|
| **T1 — Schema & Types** | 1 Schema Validation, 2 Migration Status, 6 Type Safety, 7 Null Handling | Reads schema files / migration dir / type defs — no row scans | `discovery/schema.json`, `evidence-summary.json` |
| **T2 — Integrity core (HINGE)** | 3 Orphaned Records, 4 Referential Integrity, 5 Data Consistency, 8 Duplicate Detection | Row-level LEFT-JOIN / invariant queries on the HINGE TABLES | `discovery/relationships.json` |
| **T3 — Operations** | 9 Cascade Behavior, 10 Backup Verification, 11 Query Performance, 12 Index Coverage, 16 Transaction Integrity | Operational probes (EXPLAIN, backup config, cascade dry-runs) | `discovery/tables.json` |
| **T4 — Governance** | 13 Data Lifecycle, 14 PII Detection, 15 Seed Data Separation | Column-classification + retention/PII scan | `discovery/schema.json` |

Rules of the fan-out:
- **T2 is the HINGE track** — give it 10× scrutiny budget (per Gestalt Law 4 + Phase 2.2). The HINGE TABLES (Users / Orders / Products / whatever the core entities are) get every Phase-3/4 check at maximum depth; peripheral tables get proportional depth.
- **All tracks are READ-first.** Fan-out is pure discovery + scoring; no write/fix runs inside a parallel branch (R-SCOPE: one writer per file — fixes are serialized in Phase 19, never concurrent).
- Each track writes only its own `reports/*.md` files (already file-disjoint per the OUTPUT CONTRACT), so parallel writes never collide.
- A DESTRUCTIVE audit constraint holds: no track may issue a write/DDL statement — the DB Backup Gate (v1.1 addendum) only opens in the serialized fix phase.

### 2. Adversarially VERIFY every finding (≥2-of-3 independent lenses)

A single LEFT-JOIN returning rows, or a tool finding in `evidence-summary.json`, is a **candidate**, never a verdict (R-VERIFY). Before a finding is admitted to the scoring matrix it must survive **≥2 of these 3 independent lenses**:

1. **REPRODUCE** — re-run the exact diagnostic against the live data a second time (or against a fresh snapshot). An orphan count that changes between runs is a race/soft-delete artifact, not a stable orphan. Cite both counts. *(Deterministic command, never LLM-as-judge — Verification Contract Pattern #3.)*
2. **REFUTE (Popper)** — actively try to make the finding FALSE: is the FK *meant* to cascade (so the "orphan" is expected)? Was the parent removed by a real user deletion (legit) vs a bug? Is the NOT-NULL "violation" actually an `v.optional()` the schema legitimately allows? Use the Phase 2.1 falsification table row for `data`: *"orphaned record → verify the FK is supposed to cascade, or confirm the parent was deleted by an actual user action."* A finding that the refutation kills is **demoted to `info` with `falsified_at:<evidence>`** — do NOT score it as a defect.
3. **CROSS-CHECK** — confirm the same defect from a second, independent vantage: schema-definition vs runtime-sample (the v1.2 `DATA_INTEGRITY_SCHEMA` vs `DATA_INTEGRITY_RUNTIME` split), or a sibling audit's `evidence-summary.json` (Phase 0.5 — e.g. `/apiaudit` flags the same endpoint+table contract drift, or `/secaudit` flags the same PII column). A cross-confirmed finding sets `cross_audit_confirmed: true` and bumps severity one level (Phase 2.5).

**The gate:** a finding entering Phase 17 scoring must record ≥2 passing lenses in its `falsifiable_tests[]` / verification trail. Findings that pass only 1 lens are held as `confidence: medium` and MUST NOT lower a phase score on their own. Findings that survive 0 lenses are **killed** (logged in `fix-log.md` as falsified, excluded from the verdict). This is exactly the Verification Contract's "guilty until proven innocent" applied per-finding, and it preserves Popper rigor under parallelism.

### 3. Synthesize survivors into THIS audit's existing scoring matrix + verdict (UNCHANGED)

Synthesis is the audit's own job — never paste a track's summary as the verdict (R-ORCH). Merge the surviving, ≥2-of-3-verified findings back into the **unchanged** machinery:

- Map each survivor to its originating phase and feed the existing **/320 SCORING MATRIX (Phase 17)** exactly as written — same per-phase 0-10 scoring, same weights (Phase 3 ×3.5, Phase 4 ×3.5 HINGE … Phase 16 ×1.0), same `normalize = raw/320 × 100`, same S/A/B/C/D/F grade bands.
- Emit the **same hybrid v2 `verdict.json`** (Phase 2.6 schema): `falsifiable_tests[]` now carries the 2-of-3 lens outcomes, `hinge_findings[]` carries the T2 deep-scrutiny results, `cross_audit_links[]` carries the CROSS-CHECK confluences, and `score_gating` (Phase 2.7) still blocks 100/100 unless every critical/high is fixed-or-justified and `user_need_match.addressed=true`.
- The fix flow is unchanged and **serial**: Phase 18 FIX PLAN → Phase 19 FIX EXECUTION (with the DB Backup Gate + Do-No-Harm + R-SCOPE one-writer serialization) → Phase 20 RE-AUDIT. Parallelism lives only in discovery/scoring, never in mutation.

### 4. Loop-until-dry for unknown-size discovery

Orphans, duplicates, type-coercion surprises, and PII columns are **unbounded-size discovery** — you cannot know up front how many exist. Run each discovery track as a **loop-until-dry** rather than a fixed single pass:

```
for each discovery track (orphans / duplicates / type-mismatches / PII / invariant-violations):
    repeat:
        fan out the diagnostic across all in-scope tables (HINGE TABLES first, at 10× depth)
        collect new candidate findings
        adversarially verify each (≥2-of-3 lenses, §2)
        feed survivors to the matrix; kill the falsified
    until: a full pass yields ZERO new surviving findings   (dry)
         OR the 5-iteration cap (preamble §4 / Verification Contract) is hit
on cap reached: mark remaining candidates NEEDS_REVIEW, Telegram SOS, emit confidence:low — never loop silently
```

The loop is **bounded** (hard cap 5, per the shared preamble and the 5-iteration fix-and-reaudit cap) and **token-budgeted** (R-BUDGET, default 500K) — approaching either ceiling escalates rather than overruns. "Dry" = a complete fan-out pass across every in-scope table surfaces no new finding that survives §2. Only then is the integrity surface considered fully explored and the verdict allowed to finalize.

> **Invariant:** this v2 orchestration changes only execution topology (parallel discovery + adversarial admission + bounded loop). Every phase, weight, the /320 matrix, the verdict schema, the DESTRUCTIVE backup gate, and the Gestalt-Popper doctrine above remain authoritative and unmodified.
