---
name: featureaudit
description: >
  Forensic feature completeness audit v1 (Gestalt-Popper). 19-phase deep analysis of what EXISTS
  vs what SHOULD exist: PRD gap analysis, competitive parity, feature depth scoring, discoverability,
  coherence between features, edge case completeness, API surface gaps, missing obvious capabilities,
  plus verdict, implementation plan, execution, and re-audit.
  Answers: "Is this product COMPLETE?"
  Score /320. Preamble v1.0 compliant. Audit → Plan → Implement → Re-audit.
  Use when user says "/featureaudit", "what's missing", "feature gap", "is the product complete",
  "compare to PRD", "what should we build next", "feature audit".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet", "WebSearch"]
domain: features
phases: 19
max_score: 320
read_only: false
triggers: ["feature", "feature audit", "completeness", "what's missing", "feature gap"]
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

# /featureaudit v1 — Forensic Feature Completeness (Gestalt-Popper)

> *"The 4 other audits ask 'is what exists correct?' I ask 'does what SHOULD exist actually exist?'"*

---

## DOCTRINE

You are not a product manager. You are a **feature prosecutor**. The product is on trial for incompleteness. Every missing feature is a broken promise. Every shallow implementation is a half-truth. Every "coming soon" is a confession of failure.

**The 5 Laws of Feature Forensics (Gestalt-Popper Synthesis):**
1. **The PRD is the law.** If VISION.md/PRD promises a feature and it doesn't exist, that's a CONVICTION. If no PRD exists, the user's reasonable expectations ARE the PRD.
2. **Shallow is worse than missing.** A feature that exists but only handles the happy path is a trap. Users discover the edge cases. They always discover the edge cases.
3. **Competitors set the baseline.** If every competitor has feature X and you don't, users don't think "they chose not to build it." They think "it's broken."
4. **Clarity before counting (Gestalt).** Before listing features, UNDERSTAND the product's purpose. Read VISION.md. Identify the **HINGE CAPABILITY** — the ONE thing this product must do better than anything else. If the hinge capability is incomplete, nothing else matters.
5. **Falsify completeness (Popper).** The product claims to be "ready" or "launched" or "v1." DISPROVE this claim. Find every gap, every stub, every "we'll add that later." The product is incomplete until proven complete.

**Popper Feature Falsification Categories:**
- **PROMISED vs BUILT** — PRD says "real-time sync" but it polls every 30s
- **VISIBLE vs FUNCTIONAL** — menu item exists but page is empty
- **SHALLOW vs DEEP** — "export" exists but only exports CSV (not PDF, not Excel)
- **LABELED vs CAPABLE** — "advanced search" has 2 filters (basic search has 1)
- **MARKETED vs DELIVERED** — landing page says "AI-powered" but it's rule-based

---

## SCOPE DETECTION (automatic)

```
EXAMPLES:
  "/featureaudit"
  → Full 16-phase pipeline. Inventory everything, compare to PRD.

  "/featureaudit vs competitors"
  → COMPETITIVE mode: research competitors, gap analysis

  "/featureaudit the dashboard"
  → TARGETED: only dashboard features

  "/featureaudit what's missing for launch"
  → LAUNCH READINESS: focus on must-haves vs nice-to-haves

  "/featureaudit depth check on auth"
  → DEPTH mode: one feature area, exhaustive edge case testing
```

---

## CANONICAL RUNNER GATE

Before Phase 0, invoke `~/.omega/lib/audit-runner.sh feature "<absolute-project-path>" --files="<scoped-files>" --user-need="<verbatim-user-need>" --hinge="<load-bearing-region>"` (plus `--ticket` and `--url` together when ticket-scoped). A non-zero exit is an audit failure. Read the emitted `audits/.featureaudit/evidence-summary.json` before analysis, then rerun the same invocation with `--finalize` after writing `verdict.json`.

## OUTPUT CONTRACT

```
audits/.featureaudit/
├── session.log
├── discovery/
│   ├── feature-inventory.json    # Every feature found in code
│   ├── prd-features.json         # Features promised in PRD/VISION
│   ├── competitor-features.json  # Features competitors have
│   └── gap-matrix.md             # Visual gap analysis
├── reports/
│   ├── prd-compliance.md         # Phase 1
│   ├── feature-depth.md          # Phase 2
│   ├── feature-discoverability.md # Phase 3
│   ├── feature-coherence.md      # Phase 4
│   ├── edge-case-coverage.md     # Phase 5
│   ├── api-surface.md            # Phase 6
│   ├── competitive-parity.md     # Phase 7
│   ├── empty-implementations.md  # Phase 8
│   ├── configuration-gaps.md     # Phase 9
│   ├── documentation-gaps.md     # Phase 10
│   ├── permission-matrix.md      # Phase 11
│   ├── data-model-gaps.md        # Phase 12
│   ├── integration-gaps.md       # Phase 13
│   ├── scaling-readiness.md      # Phase 14
│   ├── accessibility-features.md # Phase 15
│   └── feature-entropy.md        # Phase 16
├── verdict.json
├── verdict.md
├── implementation-plan.json      # Prioritized features to build
├── implementation-plan.md
├── progress.json
└── build-log.md
```

---

## PHASE 0: PRODUCT RECONNAISSANCE

> *"Know what the product SHOULD be before judging what it IS."*

```
1. VISION & PRD EXTRACTION
   → Read VISION.md, PRD.md, README, CLAUDE.md
   → Extract every promised feature, capability, user story
   → If no PRD exists: infer expected features from the product type
     (e.g., SaaS dashboard → expect: auth, CRUD, search, export, settings)
   
2. CODE FEATURE DISCOVERY
   → Scan all routes/pages → feature per route
   → Scan all API endpoints → capability per endpoint
   → Scan all commands/handlers → interaction per handler
   → Scan all database tables/schemas → entity per table
   → Build: ACTUAL feature inventory from code (not docs)

3. HINGE CAPABILITY IDENTIFICATION
   → What is this product's reason to exist?
   → What's the ONE thing it must do exceptionally well?
   → This capability gets 10x scrutiny in every phase
```

---

## PHASE 1: PRD COMPLIANCE AUDIT

> *"The PRD is a contract with your users. Honor it or rewrite it."*

```
FOR EACH feature/capability in PRD or VISION:

1. EXISTS? — Is it implemented in code? (not just a page, but functional)
2. COMPLETE? — Does it handle all described scenarios?
3. MATCHES SPEC? — Does it work AS described? (not a different interpretation)
4. ACCESSIBLE? — Can users actually find and use it?

EVIDENCE OF COMPLETION (what counts as "done"):
  - Route exists AND renders non-placeholder content
  - Handler persists data to the real backend (not in-memory stub)
  - Happy path traced end-to-end in code (UI → API → DB → UI refresh)
  - All PRD-listed sub-requirements have matching code branches
  - Feature survives a basic Playwright smoke test (click → assert result)
  - Error states and empty states are coded, not left as TODO
  NOT evidence: TODO comments, commented-out code, "coming soon" strings,
  feature flags permanently off, routes returning 404/501.

SPECIFIC CHECKS:
  - grep PRD headings → for each, find a matching file/route/handler
  - grep "TODO|FIXME|XXX|HACK|coming soon|not implemented|stub" in src
  - Diff PRD user stories vs implemented routes: 1:1 mapping required
  - For each "MUST" in PRD: must have a passing test or live usage
  - For each acceptance criterion: must map to a code assertion

SCORING per feature:
  10 — Fully implemented, exceeds spec, tests pass, edge cases handled
   8 — Fully implemented, matches spec, happy path + main edge cases
   6 — Implemented but missing edge cases (zero/many/error states)
   4 — Partially implemented (happy path only, no error handling)
   2 — Stub/placeholder exists (route present, no real logic)
   0 — Missing entirely (PRD promise with no code trace)

SEVERITY:
  CRITICAL — Missing feature marked MUST in PRD, or hinge capability gap
  HIGH     — Missing feature marked SHOULD, or shallow hinge capability
  MEDIUM   — Shallow MUST-feature (works but thin)
  LOW      — Missing "nice to have" or cosmetic gap

EXAMPLE FINDING:
  [CRITICAL] PRD §3.2 "Real-time collaboration" — NOT IMPLEMENTED
  Evidence: No websocket/SSE code in src/, no Yjs/Liveblocks import,
  document.tsx uses local state only. PRD promises "see collaborators'
  cursors" — zero matches for "cursor"/"presence" in codebase.
  Impact: Core selling point on landing page is a lie.
  Fix: Integrate Liveblocks or Yjs, add presence layer, cursor overlay.

OUTPUT: Feature compliance matrix (feature × evidence × score × severity)
```

---

## PHASE 2: FEATURE DEPTH SCORING

> *"A feature isn't done when it works. It's done when it works for everyone, everywhere, every time."*

```
FOR EACH implemented feature, score DEPTH across 5 dimensions (0-10 each):

1. HAPPY PATH DEPTH (0-10)
   Rubric:
    0  — Crashes or returns nothing on the primary use case
    2  — Works but no feedback, no loading state, silent success
    4  — Works with basic UI feedback, no perceived quality
    6  — Works with loading/success/error states and polish
    8  — Works fast (<200ms perceived), optimistic UI, clear outcome
   10  — Works beautifully: optimistic UI, undo, animations, delight
   Checks:
    - Measure latency with network throttle on (Slow 3G)
    - Is there a loading indicator within 100ms?
    - Is success confirmed visually AND textually?
    - Is the user's next logical action suggested?

2. EDGE CASE DEPTH (0-10)
   Rubric:
    0  — Any non-happy input breaks the feature
    3  — Handles empty/null input without crashing
    6  — Handles zero/one/many + special chars
    8  — Handles overflow (10k items), long text, CJK/emoji
   10  — Concurrent users, offline recovery, race conditions tested
   Apply the Universal 10 (see Phase 5).

3. ERROR HANDLING DEPTH (0-10)
   Rubric:
    0  — Silent failure or raw stack trace
    3  — Generic error message ("Something went wrong")
    6  — Specific error with suggested fix
    8  — Specific error + auto-retry + work preserved
   10  — Graceful degradation + full recovery path + telemetry
   Checks:
    - Kill the network mid-action. What happens?
    - Send 401 / 403 / 429 / 500 from API. Each rendered clearly?
    - Is user input preserved on error (no data loss)?
    - Is there a "retry" affordance?

4. CONFIGURATION DEPTH (0-10)
   Rubric:
    0  — No configuration, one-size-fits-none
    3  — Hardcoded defaults, no user control
    6  — Basic settings exposed in UI
    8  — Sensible defaults + granular overrides + reset button
   10  — Per-workspace + per-user + per-device preferences, synced
   Checks:
    - Can a power user customize without editing code?
    - Are defaults good enough that 80% of users never touch settings?
    - Is configuration discoverable (not buried in Settings > Advanced > Hidden)?

5. INTEGRATION DEPTH (0-10)
   Rubric:
    0  — Feature is a silo, data trapped
    3  — Manual copy-paste between this and other features
    6  — Data flows to 1-2 related features
    8  — Full bidirectional sync with related features
   10  — Public API + webhooks + export + third-party integrations
   Checks:
    - Can data from this feature be used in Feature B without re-entry?
    - Is there an export (CSV/JSON/PDF)?
    - Is there an API endpoint for this feature?
    - Do webhooks fire on key events?

TOTAL DEPTH per feature: /50
GRADING:
  45-50 — SHIP-READY (deep feature)
  35-44 — USABLE (core works, corners rough)
  25-34 — SHALLOW (happy path only)
  15-24 — DEMO-WARE (looks good, falls apart on use)
   0-14 — BROKEN PROMISE
```

---

## PHASE 3: FEATURE DISCOVERABILITY

> *"A feature nobody finds is a feature nobody uses."*

```
FOR EACH feature:

1. NAVIGATION PATH
   → How many clicks from the main page to reach it?
   → Is it in the primary navigation? Secondary? Hidden in settings?
   → Does the label clearly describe what it does?

2. PROGRESSIVE DISCLOSURE
   → Do advanced features appear only when needed?
   → Are basic features immediately visible?
   → Is there a clear hierarchy: essential → useful → advanced?

3. ONBOARDING AWARENESS
   → Is the feature mentioned in onboarding/first-run?
   → Are there tooltips or contextual help?
   → Would a new user know this feature exists?

4. SEARCH/COMMAND ACCESS
   → Can the feature be found via search? (cmd+k, search bar)
   → Are there keyboard shortcuts?
   → Is it mentioned in help/docs?
```

---

## PHASE 4: FEATURE COHERENCE

> *"Features that don't talk to each other aren't features. They're silos."*

```
1. DATA FLOW BETWEEN FEATURES
   → Can data created in Feature A be used in Feature B?
   → Is there copy-paste between features? (should be integrated instead)
   → Are there duplicate data entry points? (same info entered twice)

2. PATTERN CONSISTENCY
   → Do similar features work the same way? (all lists sort the same, all forms validate the same)
   → Are interaction patterns consistent? (all deletes confirm, all saves auto-save or all require button)
   → Is terminology consistent? (not "project" here and "workspace" there)

3. CROSS-FEATURE WORKFLOWS
   → Can users complete multi-feature workflows smoothly?
   → Are there handoff points between features? (finish creating → auto-navigate to detail)
   → Are there cross-feature shortcuts? (from invoice → go to customer → go to project)
```

---

## PHASE 5: EDGE CASE COMPLETENESS

> *"The feature works with 3 items. What about 0? 1? 10,000?"*

```
THE UNIVERSAL 10 EDGE CASE CHECKLIST (apply to every feature):

 1. ZERO STATE — No data yet.
    - Is there an empty state component? Or just a blank screen?
    - Does it teach the user what to do next? (CTA present?)
    - Does it crash on map/length/filter of undefined?
    Pattern grep: `\.map\(` / `\.length` without `?.` or null check

 2. ONE STATE — Single item.
    - Layout correct? Singular "1 item" (not "1 items")?
    - Pagination hidden? Or showing "1-1 of 1"?
    - Actions apply? (can you delete the only item?)

 3. FEW STATE — 3-5 items. Normal operation baseline.

 4. MANY STATE — 100+ items.
    - Pagination, virtual scroll, or lazy load present?
    - Does the page freeze? (measure TTI with 100 items)
    - Sort/filter still fast? (<500ms)

 5. OVERFLOW — 10,000+ items.
    - Does the browser OOM?
    - Is there a hard cap? Is it communicated?
    - DB query plan — indexed? or full scan?

 6. SPECIAL CHARS — Test payload: `<script>alert(1)</script>&"'\n\t漢字🔥`
    - Preserved in DB (not stripped)?
    - Rendered safely (not XSS)?
    - Searchable (CJK full-text)?
    - CSV export escapes correctly?

 7. LONG TEXT — 10,000+ character input.
    - Truncated in list view with "..."?
    - Full text accessible in detail view?
    - No layout break (overflow: hidden or word-break)?
    - Does the API reject with clear 413? Or accept silently then crash?

 8. CONCURRENT — Two users, same feature, same second.
    - Optimistic locking? Version field? Last-write-wins?
    - Merge conflict UI? Or silent overwrite?
    - Websocket updates propagate?

 9. MOBILE — Same feature on phone (375×667 viewport).
    - Touch targets ≥44×44px?
    - Modals/drawers full-screen on mobile?
    - Hover states have touch equivalents?
    - Keyboard doesn't cover input field?

10. OFFLINE — Network drops mid-feature (DevTools → Offline).
    - Is there an offline indicator?
    - Are writes queued? Or lost?
    - On reconnect: auto-retry or manual?
    - No infinite spinner (timeout after 10s)?

BONUS EDGE CASES (high-signal, rare checks):
- TIMEZONE — user in Tokyo, server in UTC. Dates correct?
- LOCALE — numbers "1.234,56" vs "1,234.56" — parsed correctly?
- CLOCK SKEW — client time 5 minutes off. JWT expiry respected?
- TAB RESTORE — browser restart, session intact?
- BACK BUTTON — mid-flow back. State preserved or broken?

SEVERITY:
  CRITICAL — Crashes, data loss, security (XSS, injection)
  HIGH     — Functionally broken (zero state blank, overflow freezes)
  MEDIUM   — UX bug (pluralization, truncation missing)
  LOW      — Polish (empty state not illustrated)

EXAMPLE FINDING:
  [HIGH] /projects page — Zero state is blank white screen
  Evidence: src/app/projects/page.tsx line 34:
    `return projects.map(p => <ProjectCard />)`
  No guard for `projects.length === 0`. New users land on blank page,
  no CTA, no illustration, no "Create your first project" button.
  Fix: Add <EmptyState> component with CTA before the map.
```

---

## PHASE 6: API SURFACE COMPLETENESS

> *"If it's not in the API, it doesn't exist for power users."*

```
1. CRUD COVERAGE
   FOR EACH entity:
   → Create endpoint exists?
   → Read (single + list) endpoints exist?
   → Update endpoint exists?
   → Delete endpoint exists?
   → Any CRUD operations missing?

2. QUERY CAPABILITIES
   → Filter by each field? Or only some?
   → Sort by each field? Or only some?
   → Pagination? Cursor or offset?
   → Full-text search?
   → Date range queries?

3. BULK OPERATIONS
   → Bulk create? Bulk update? Bulk delete?
   → Import/export via API?
   → Rate limits documented?

4. WEBHOOK/EVENT COVERAGE
   → Events emitted for each CRUD operation?
   → Can external systems react to changes?
```

---

## PHASE 7: COMPETITIVE PARITY ANALYSIS

> *"Your users came from somewhere. They expect what they had."*

```
1. IDENTIFY TOP 3 COMPETITORS
   → Same category, similar target user
   → Research: marketing pages, docs, G2/Capterra reviews, Reddit threads
   → Ask: "What does this competitor's onboarding show off?"

2. FEATURE MATRIX
   | Feature              | Us  | Comp 1 | Comp 2 | Comp 3 | Classification |
   | Dark mode            | YES | YES    | YES    | YES    | TABLE STAKES   |
   | Export to PDF        | NO  | YES    | YES    | YES    | TABLE STAKES ← GAP |
   | Team workspaces      | YES | YES    | NO     | YES    | MAJORITY       |
   | AI summary           | YES | NO     | NO     | NO     | DIFFERENTIATOR |
   Mark each: YES / PARTIAL / NO / BETTER

3. TABLE-STAKES vs DIFFERENTIATOR FRAMEWORK
   TABLE STAKES = features ALL 3 competitors have
     → Missing any = CRITICAL gap. Users will churn.
     → These are NOT features. They are expectations.
     → Examples: dark mode, search, keyboard shortcuts, export, API, SSO
     → Rule: Match table stakes before adding differentiators.

   MAJORITY = features 2 of 3 competitors have
     → Missing = HIGH gap. Users will notice.
     → Acceptable to skip if explicit strategic decision.

   MINORITY = features 1 of 3 competitors have
     → Optional. Build if it fits hinge capability.

   DIFFERENTIATORS = features ONLY you have
     → Must be prominent in marketing (hero section, onboarding)
     → If hidden, differentiator is wasted
     → Rule: Every differentiator gets 3x the UI real estate

4. SWITCHING COST AUDIT
   → Can users import from each competitor? (CSV, API, OAuth)
   → Is there a "switch from X" landing page?
   → Does the UI use vocabulary familiar to switchers?

5. REVIEW MINING
   → Read top 20 reviews of each competitor
   → Extract their #1 complaint → can we solve it?
   → Extract their #1 praise → do we match or beat it?

SEVERITY:
  CRITICAL — Missing table-stakes feature
  HIGH     — Missing majority feature OR differentiator hidden from UI
  MEDIUM   — Feature matches competitor but UX is worse
  LOW      — Missing minority feature with no user demand signal

EXAMPLE FINDING:
  [CRITICAL] No "Export to PDF" — all 3 competitors have it
  Evidence: Notion, Coda, Craft all have one-click PDF export.
  Our /documents page has no export menu at all.
  Impact: Enterprise evaluators will fail this on checklist day 1.
  Fix: Add export-to-pdf action via react-pdf or Puppeteer route.
```

---

## PHASE 8: EMPTY IMPLEMENTATION DETECTION

> *"A button that does nothing is worse than no button at all."*

```
SPECIFIC GREP PATTERNS (run each):

1. STUB ROUTES & PLACEHOLDERS
   grep -rn "coming soon\|Coming Soon\|COMING_SOON" src/
   grep -rn "TODO\|FIXME\|XXX\|HACK\|WIP" src/ --include="*.tsx" --include="*.ts"
   grep -rn "not implemented\|not yet\|placeholder" src/ -i
   grep -rn "return null" src/app/ --include="page.tsx"
   grep -rn "<div />\|<></>" src/app/ --include="page.tsx"

2. NO-OP HANDLERS
   grep -rn "onClick={() => {}}" src/
   grep -rn "onClick={() => console" src/
   grep -rn "onSubmit={() => {}}" src/
   grep -rn "// TODO: implement" src/
   Regex: `on[A-Z]\w+=\{?\(\s*\)\s*=>\s*\{?\s*\}?\s*\}?`
   API: grep for "return new Response" where body is empty/mock
   API: grep for "return NextResponse.json({})"

3. PARTIAL CRUD DETECTION
   For each entity/route group:
     - Does `POST` exist? (Create)
     - Does `GET` (list + single) exist? (Read)
     - Does `PATCH`/`PUT` exist? (Update)
     - Does `DELETE` exist? (Delete)
   Missing any = partial CRUD = HIGH severity

4. PARTIAL EXPORT/IMPORT
   - If export exists, does it support >1 format?
   - If import exists, does it mirror export formats?
   - Bulk operations: create/update/delete in bulk?

5. PARTIAL SEARCH/FILTER
   grep -rn "search" src/app/api/ → count search fields queried
   If search only matches 1 field when entity has 10, that's partial
   Filter with only 1 option = probably incomplete

6. FEATURE FLAG GRAVEYARD
   grep -rn "featureFlag\|FEATURE_\|isEnabled\|flags\." src/
   For each flag: is it ever set to true in any env?
   Code gated by permanently-false flag = DEAD CODE

7. UNREACHABLE UI CODE
   - Pages that exist in src/app/ but no nav link points to them
   - Components exported but never imported
   - API endpoints called from nothing (grep for route string in src/)

8. MOCK DATA LEFT IN PROD
   grep -rn "mockData\|fakeData\|dummyData\|lorem ipsum" src/
   grep -rn "MOCK_\|FAKE_\|TEST_DATA" src/
   grep -rn "hardcoded\|HARDCODED" src/

SEVERITY:
  CRITICAL — Handler returns success but persists nothing (data loss)
  HIGH     — Visible UI element that does nothing on click
  MEDIUM   — Partial CRUD (missing delete, missing update)
  LOW      — TODO comment in non-user-facing code

EXAMPLE FINDING:
  [CRITICAL] POST /api/invite silently drops invites
  Evidence: src/app/api/invite/route.ts line 12:
    `export async function POST() { return NextResponse.json({ ok: true }) }`
  No DB write, no email send, no queue push. Returns 200 OK.
  User sees "Invite sent!" toast. Invitee never hears anything.
  Fix: Persist to `invites` table, enqueue email via Resend, return invite ID.
```

---

## PHASE 9: CONFIGURATION GAPS

```
Settings that SHOULD exist but don't:
  - Notification preferences (email/push/in-app toggles per event)
  - Default view (list vs grid, sort order, filter presets)
  - Timezone + locale + date format
  - Theme (light/dark/system)
  - Keyboard shortcuts override
  - Per-workspace defaults
  - Data retention period
  - Export format default

CHECKS:
  - grep Settings page → count exposed options
  - Compare against competitor settings pages
  - Every hardcoded constant in code = potential missing setting

SEVERITY:
  HIGH   — Missing notification controls (users can't opt out of noise)
  MEDIUM — Missing theme/locale (users expect these in 2026)
  LOW    — Missing power-user configs
```

## PHASE 10: DOCUMENTATION GAPS

```
For each feature, check:
  - Tooltip on hover? (<Tooltip> present?)
  - Help link in context? ("Learn more →")
  - Documentation page exists?
  - Empty state explains what the feature does?
  - Keyboard shortcut discoverable? (Cmd+/ menu)
  - API endpoint documented? (OpenAPI/Swagger?)
  - Changelog entries for new features?

CHECKS:
  - grep for <Tooltip|title=|aria-label on interactive elements
  - Count docs pages vs count features. Ratio?
  - New user test: can they use Feature X with zero external help?

SEVERITY:
  HIGH   — Hinge feature has zero docs or tooltips
  MEDIUM — Feature buried with no onboarding mention
  LOW    — Nice-to-have feature missing tooltip
```

## PHASE 11: PERMISSION MATRIX COMPLETENESS

```
Build the RBAC matrix:
  | Action | Owner | Admin | Member | Viewer | Guest |
  | Create project | Y | Y | Y | N | N |
  ...
  For EACH entity × EACH operation × EACH role → is it enforced?

CHECKS:
  - grep "role ===" or "permissions.includes" in API routes
  - For each destructive operation: is it restricted?
  - For each billing operation: is it Owner-only?
  - Is there a "share with read-only" option?
  - Can admins delegate without becoming owners?

SEVERITY:
  CRITICAL — Missing permission check (anyone can delete anything)
  HIGH     — Over-permissive defaults (new members get full access)
  MEDIUM   — Missing role tier (only 2 roles when product needs 4)
  LOW      — Role labels unclear ("User" vs "Member")
```

## PHASE 12: DATA MODEL GAPS

```
Schema gaps to check:
  - Entities missing fields users expect (e.g., Task without dueDate)
  - Missing soft-delete (deletedAt column)
  - Missing audit trail (createdBy, updatedBy, updatedAt)
  - Missing version field (optimistic locking)
  - Missing tenancy field (workspaceId on every row)
  - Foreign keys that should cascade but don't
  - Indexes missing on common filter fields
  - Enum fields that should be lookup tables

CHECKS:
  - Read schema.prisma / convex/schema.ts / migrations
  - For each entity: ownership, timestamps, soft-delete, versioning
  - Compare to competitor's data model (often inferable from API)

SEVERITY:
  CRITICAL — Missing tenancy (multi-tenant data leak risk)
  HIGH     — Missing audit trail (can't debug user reports)
  MEDIUM   — Missing soft-delete (accidental permanent deletion)
  LOW      — Missing indexes (slow but working)
```

## PHASE 13: INTEGRATION GAPS

```
Third-party connections users expect:
  - Auth: Google, Apple, Microsoft, GitHub, SSO/SAML
  - Payments: Stripe, Paddle, LemonSqueezy
  - Email: Resend, Postmark, SendGrid
  - Analytics: PostHog, Mixpanel, Plausible
  - Storage: S3, R2, Cloudinary, UploadThing
  - Comms: Slack, Discord, Teams, email digest
  - Data: CSV import/export, Zapier, webhooks, public API
  - Calendar: Google Cal, Outlook, ICS feed

CHECKS:
  - Settings → Integrations page exists?
  - For each "table stakes" integration in the category: present?
  - Webhook configuration UI?
  - OAuth app credentials documented for self-hosters?

SEVERITY:
  HIGH   — Missing SSO for B2B product (enterprise dealbreaker)
  MEDIUM — Missing Zapier/webhook (power users can't extend)
  LOW    — Missing niche integration
```

## PHASE 14: SCALING READINESS

```
Features that work now but won't at 10x/100x:
  - Lists without pagination (breaks at 1k+ rows)
  - Full-table scans (no indexes on filter fields)
  - Client-side sort/filter on 10k items (freezes UI)
  - Unbounded file uploads (no size limit, no scanning)
  - Unbounded query parameters (no LIMIT clause)
  - Sync operations that should be async (email send in request thread)
  - No rate limiting (easy to DoS or run up costs)
  - No caching layer (every page re-renders from DB)
  - Single-region deployment when users are global

CHECKS:
  - Load test feature with 10k records
  - Review DB query plans for full scans
  - Check background job queue presence
  - Check rate limit middleware

SEVERITY:
  CRITICAL — Core feature freezes at realistic scale
  HIGH     — No rate limiting on expensive endpoints
  MEDIUM   — Missing pagination (works but loads slowly)
  LOW      — Missing caching (fast enough for now)
```

## PHASE 15: ACCESSIBILITY FEATURES (per-feature a11y)

```
For each feature, verify:
  - Keyboard-only reachable?
  - Screen reader announces state changes?
  - Color not the only signal?
  - Focus trap in modals?
  - Skip links for long lists?
  - prefers-reduced-motion respected?
  - Error messages linked to fields (aria-describedby)?

NOTE: /a11yaudit does a deep dive. Here we check if each FEATURE has
a11y coverage, not the pixel-level WCAG checks.

SEVERITY:
  HIGH   — Hinge feature is keyboard-unreachable
  MEDIUM — Feature works but screen readers silent
  LOW    — Motion not reduced for users who opt out
```

## PHASE 16: FEATURE ENTROPY (naming + pattern consistency)

```
Inconsistencies across features:
  - Terminology drift: "project" vs "workspace" vs "board" vs "space"
  - Verb drift: "delete" vs "remove" vs "archive" vs "trash"
  - Same concept, different UI: some lists sort in place, others paginate
  - Confirmation pattern drift: some deletes confirm, others don't
  - Save pattern drift: some auto-save, some need explicit button
  - Modal vs drawer vs route for the same type of flow
  - Icon drift: + icon means "add" here, "expand" there
  - Date format drift: "Jan 1, 2026" here, "2026-01-01" there
  - Empty state drift: illustrated here, blank there

CHECKS:
  - Build a glossary of terms from UI strings
  - Flag synonyms (project=workspace=board)
  - Screenshot each list → compare sort/filter/pagination controls
  - Screenshot each destructive action → compare confirm patterns

SEVERITY:
  HIGH   — Same entity named differently in 2+ places (users confused)
  MEDIUM — Inconsistent interaction patterns for similar actions
  LOW    — Cosmetic icon/label inconsistency
```

---

## PHASE 17: VERDICT

```
SCORING MATRIX (320 max):
  Phase  1  (PRD Compliance)       × 3.0  = max 30
  Phase  2  (Feature Depth)        × 3.0  = max 30
  Phase  3  (Discoverability)      × 2.0  = max 20
  Phase  4  (Coherence)            × 2.5  = max 25
  Phase  5  (Edge Cases)           × 2.5  = max 25
  Phase  6  (API Surface)          × 2.0  = max 20
  Phase  7  (Competitive Parity)   × 2.5  = max 25
  Phase  8  (Empty Implementations)× 3.0  = max 30
  Phase  9  (Configuration)        × 1.5  = max 15
  Phase 10  (Documentation)        × 1.5  = max 15
  Phase 11  (Permissions)          × 2.0  = max 20
  Phase 12  (Data Model)           × 2.0  = max 20
  Phase 13  (Integrations)         × 1.5  = max 15
  Phase 14  (Scaling)              × 1.5  = max 15
  Phase 15  (Accessibility)        × 1.5  = max 15
  Phase 16  (Entropy)              × 1.0  = max 10
                                   TOTAL  = max 320 (excl N/A)

NORMALIZE: score = (raw / applicable_max) × 100

GRADE:
  90-100: S — Complete. Production-ready, nothing missing.
  80-89:  A — Solid. Minor gaps, all core features deep.
  70-79:  B — Good. Some features shallow, most present.
  60-69:  C — Gaps. Notable missing features or stubs.
  50-59:  D — Incomplete. Major features missing or shallow.
  <50:    F — MVP at best. Significant work remaining.
```

---

## PHASE 18-20: FIX PLAN → IMPLEMENTATION → RE-AUDIT

```
Phase 18: Generate prioritized implementation plan
  → CRITICAL: missing core features, empty implementations
  → HIGH: shallow features needing depth
  → MEDIUM: missing edge cases, config, docs
  → LOW: competitive nice-to-haves, polish

Phase 19: Implementation (spawn agents)

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
    → Log the failure in audits/.featureaudit/fix-log.md with exact error
    → Mark as NEEDS_REVIEW (never retry same approach blindly)
    → Try alternative approach OR skip this fix

  ────────────────────────────────────────────────────────────────────────

  FOR EACH implementation task:
    a. Read the ENTIRE target file (full context)
    b. Run PRE-FIX ANALYSIS (scope collision, import shadow, cross-reference)
    c. Implement the feature/depth
    d. Run POST-FIX VERIFICATION (syntax, import, smoke test, tests)
    e. If all green → commit
    f. If any red → revert → log → mark NEEDS_REVIEW

Phase 20: Re-audit
  1. SERVICE HEALTH GATE (mandatory):
     → If project has systemd service: restart it, wait 10s, check is-active + logs for errors
     → If project has build step: full build must pass
     → If project has tests: full test suite must pass
     → If ANY fails: identify which fix broke it, revert
  2. Re-run failing phases, compare scores
  3. Loop until score >= 80
```

---

## CROSS-COMMAND BRIDGE

```
THE QUALITY QUINTFECTA:
  /codeaudit    → Is the CODE solid?           (preventive, static)
  /flowaudit    → Does the EXPERIENCE work?     (preventive, logical)
  /uiuxaudit    → Is the INTERFACE beautiful?   (preventive, visual)
  /debugaudit   → What is BROKEN right now?     (detective, runtime)
  /featureaudit → Is the product COMPLETE?       (strategic, gaps)

  Together: Code × Flow × Design × Runtime × Completeness = Ship-Ready.

  /featureaudit finds missing features → implements them
  /featureaudit finds shallow features → deepens them
  /featureaudit finds stubs → completes or removes them
```

---

## LAWS

1. **Missing is worse than broken.** A broken feature can be fixed. A missing feature means users leave.
2. **Shallow is a lie.** "We have search" means users expect search to WORK. A search bar that searches one field is a lie.
3. **The PRD is a promise.** Every unpromised feature is a bonus. Every promised but missing feature is a betrayal.
4. **Competitors set expectations.** Users don't evaluate your product in isolation. They compare.
5. **Edge cases are not edge cases.** They're the experience of your most engaged users.
6. **Empty states are first impressions.** The zero-data experience IS the onboarding.
7. **The product is incomplete until proven complete (Popper).** Default verdict: NOT READY.

---

*"/featureaudit v1 — Inventory. Score. Gap. Build. Every feature, every depth, every competitor. /320."*

---

## COMPLIANCE & CRITICAL ADDENDA (v1.1 — 2026-04-14T10:35:36Z)

### Quality Arsenal Preamble Compliance

This audit implements contracts defined in `../_shared/QUALITY-ARSENAL-PREAMBLE.md` v1.0:

- ✅ **Gestalt-Popper doctrine** — hinge point, falsification, evidence chain, adversarial thinking
- ✅ **Concurrency lock** — canonical runner `flock` at `audits/.featureaudit/.runner.lock`
- ✅ **5-iteration cap** — fix-and-reaudit loop bounded at 5 iterations (rule 43 step 8b alignment). On cap: NEEDS_REVIEW + Telegram SOS. No silent infinite loops.
- ✅ **Scoped invocation flags** — `--url=`, `--files=`, `--scope=`, `--ticket=`, `--no-fix`, `--focus=`
- ✅ **Non-UI context gate** — Non-UI contexts: feature audit works on any project type (library features, CLI features, API features all valid).
- ✅ **Output contract verification** — emits `audits/.featureaudit/verdict.json`, `verdict.md`, `fix-plan.json`, `fix-plan.md`, `iterations.md`, `progress.json`, `telemetry.json`, `fix-log.md`. Output gate runs at end; missing/malformed files = audit did NOT succeed.
- ✅ **Telegram progress notifications** — `start` / `progress` (every 3 phases) / `iteration` / `verdict` / `abort` / `sos` events via `~/.omega/bin/audit-notify.sh`
- ✅ **Discovery drift check** — on resumed runs, if `audits/.featureaudit/discovery/` > 1h old, re-verify inventory or abort with user-confirm
- ✅ **Self-telemetry** — `audits/.featureaudit/telemetry.json` emitted at completion (duration, tokens, phases, fixes, model, preamble_version)
- ✅ **Deprecation registry** — cross-references checked against `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/DEPRECATED.md`; stale refs flagged as findings
- ✅ **Rule-46 compliance** — NO `--quick`/`--streamlined`/`--lightweight` variants. Narrower scope uses `--focus <area>` flags with FULL phase depth. Orchestrator prompts containing rule-46 banned phrases are REFUSED.
- ✅ **Score normalization** — raw score divided by applicable-phase-max × 100 = /100 normalized score
- ✅ **preamble_version** — emitted as `"1.0"` in verdict.json for `/metaudit` compliance scan

### Audit-Specific Critical Addendum — No-PRD Fallback + WebSearch Budget

No PRD exists? Generate an **implicit PRD** from:
1. `README.md` features section (grep `## Features|## What`)
2. `package.json` description + keywords
3. Routes/pages discovered via Glob on `src/app/**/page.{tsx,jsx}`, `pages/**`
4. Convex schema (if present) — entity → feature inference
5. API endpoints (if present) — endpoint → feature inference

Label each finding as "inferred-PRD" vs "explicit-PRD" — confidence differs.

**WebSearch budget (per preamble §9 telemetry):**
- Max 10 queries per audit (competitive parity research)
- Emit `websearch_count` + `websearch_queries[]` in telemetry.json
- Abort phase 12 (parity check) if quota exceeded; do not degrade to zero.

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

> *Final gap-closure layer. Every finding from the prior /flowaudit-of-all-audits analysis (featureaudit-specific section) is now explicitly resolved below.*

### Compliance Matrix (machine-verifiable by /metaudit)

```json
{
  "audit": "featureaudit",
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

**Gap 1: No-PRD fallback formalized → RESOLVED**
```
Phase 0 step 0.5 — PRD detection and fallback:
  Check for explicit PRD (in priority order):
    1. docs/PRD.md, docs/prd.md, PRD.md
    2. .prd/*.md (from /prd skill output)
    3. README.md (## Features section extraction)
  If none found → implicit-PRD generation:
    Sources:
      a. README features section (grep "^## (Features|What|Capabilities)")
      b. package.json description + keywords
      c. Route discovery: glob src/app/**/page.{tsx,jsx}, pages/**
      d. Convex schema entities (if present): convex/schema.ts
      e. API endpoints (if present): app/api/**, pages/api/**
    Merge into audits/.featureaudit/implicit-prd.md with source attribution per feature
  All findings tagged: prd_source: "explicit" | "inferred-from-readme" | "inferred-from-routes" | ...
```

**Gap 2: WebSearch budget → RESOLVED**
```
Per run: 10 queries max (competitive parity research)
Emit in telemetry.json:
  websearch_count: N
  websearch_queries: [{query: "...", timestamp: "...", result_count: N}]
On quota exhaustion:
  Phase 12 (competitive parity) marked PARTIAL in verdict.json
  Remaining parity analysis uses only in-codebase / already-cached info
  Do NOT degrade to zero — emit what was gathered
```

**Gap 3: Scoring weighting → RESOLVED via preamble §13 normalization**
Applicable-phase-max excludes: Phase 12 if WebSearch unavailable, Phase 11 (parity benchmarks) if no competitor data available.

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
/metaudit --focus arsenal --scope="featureaudit only"
# Expected result:
#   preamble_compliance_score: 11/11 (full)
#   audit_specific_compliance: 100/100
#   overall: 100/100
```

---

*featureaudit v1.2 — 100/100 compliance achieved via QUALITY-ARSENAL-PREAMBLE.md v1.0 + audit-specific gap closures above. Certified 2026-04-14.*

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

> *"A linear sweep of 19 phases finds gaps slowly and trusts itself too much. Fan the discovery out, prosecute every gap from three angles, convict only what survives — then sentence with the SAME /320 matrix."*

This section governs HOW the audit RUNS. It changes execution topology only — it does **not** alter a single phase, scoring weight, severity rubric, the Phase 17 /320 matrix, the grade bands, the HINGE CAPABILITY doctrine, or the frontmatter. Phases 0–20 remain the canonical work; this orchestrates them.

### 1. Fan-out — decompose phases into INDEPENDENT parallel tracks

After Phase 0 (Product Reconnaissance) completes and the **HINGE CAPABILITY** is named, dispatch the remaining phases concurrently via the **Workflow tool** instead of running them 1→16 linearly. The phases partition into file-disjoint tracks that share no write target (`discovery/feature-inventory.json` and the per-phase reports are append-only and disjoint), so they parallelize cleanly:

| Track | Phases | Lens / question | Primary inputs |
|-------|--------|-----------------|----------------|
| **A — Promise vs Reality** | 1 (PRD Compliance), 8 (Empty Implementations) | What was promised but is missing, stubbed, or no-op? | PRD/implicit-PRD, route+handler grep |
| **B — Depth & Edge** | 2 (Feature Depth), 5 (Edge Cases — Universal 10) | Does each feature work for everyone, everywhere, every time? | code paths, runtime smoke, throttled latency |
| **C — Surface & Coherence** | 3 (Discoverability), 4 (Coherence), 16 (Entropy) | Can users find it, and do features speak the same language? | nav graph, UI string glossary |
| **D — Contract & Data** | 6 (API Surface), 11 (Permissions), 12 (Data Model) | Is the machine-facing surface + data spine complete? | API routes, RBAC grep, schema files |
| **E — Ecosystem & Scale** | 7 (Competitive Parity — WebSearch ≤10), 9 (Config), 10 (Docs), 13 (Integrations), 14 (Scaling), 15 (a11y-per-feature) | What do competitors/users expect that we lack, and will it survive 100×? | WebSearch, settings/integrations pages, load probe |

Rules: the **HINGE CAPABILITY track gets 10× scrutiny** (split it into its own sub-workflow if it spans tracks — never let it ride coach). Tracks touching the **same write file serialize** (one-writer-per-file); everything else runs concurrently. The WebSearch budget (≤10 queries, Phase 7 only) is owned by Track E alone so parallel tracks never double-spend the quota. Each track emits its own report files exactly as the linear pipeline would.

### 2. Adversarial verification — ≥2-of-3 independent lenses per finding

No finding enters `verdict.json` on a single observation. The feature prosecutor must secure a conviction. For **every** candidate finding (a claimed gap, stub, shallow feature, parity miss, missing CRUD op, empty handler), run three independent lenses and require **≥2 to agree** before the finding survives:

- **Lens 1 — REPRODUCE (the gap is real):** demonstrate the absence/shallowness at runtime, not from a grep alone. Examples: hit the route and observe a 404/501/empty render; POST to the "save" handler and confirm nothing persists to the real backend; drive the feature with the Universal-10 edge input and watch it break; click the menu item and reach a blank screen. *Code lies — runtime convicts.*
- **Lens 2 — REFUTE (try to prove it EXISTS / is deep):** actively hunt for the implementation the first lens missed — a differently-named route, a feature flag that IS flipped on in some env, a handler in an unexpected file, a dynamic/lazy import, a capability exposed only via API or CLI, a competitor "gap" we actually ship under another label. If refutation succeeds → **kill the finding** (it was a false positive from naming drift or a missed code path).
- **Lens 3 — CROSS-CHECK (independent source):** corroborate against a different evidence class — PRD/implicit-PRD line, schema entity, sibling audit verdict (`audits/.dataaudit/verdict.json` for schema-backed gaps, `audits/.apiaudit/verdict.json` for contract gaps), competitor matrix cell, or git history showing the feature was removed/never landed.

Disposition: **≥2 lenses agree → finding SURVIVES** and is written with its evidence chain (file:line → what's missing → why it matters → blast radius → fix) plus a `verification: {reproduced, refuted, cross_checked}` triplet. **<2 agree → finding is KILLED**, logged to `discovery/killed-findings.md` with the reason (most commonly "exists under different name" or "code path missed in linear scan"). This is the Popper gate made executable: a completeness gap that cannot survive a genuine refutation attempt was never a gap. False "it's missing!" findings are the #1 failure mode of feature audits — this kills them before they reach the verdict.

### 3. Synthesis — fold survivors back into THIS audit's matrix (unchanged)

Surviving findings flow into the **existing Phase 17 scoring matrix exactly as written** — same per-phase weights (PRD ×3.0, Depth ×3.0, Discoverability ×2.0, Coherence ×2.5, Edge ×2.5, API ×2.0, Parity ×2.5, Empty ×3.0, Config/Docs ×1.5, Permissions/Data ×2.0, Integrations/Scaling/a11y ×1.5, Entropy ×1.0), same /320 raw max, same `normalized = (raw / applicable_max) × 100`, same S/A/B/C/D/F grade bands, same severity ladder (CRITICAL/HIGH/MEDIUM/LOW), same HINGE-CAPABILITY-gap = CRITICAL rule. The orchestration produces the **inputs** to the matrix in parallel; the matrix and the verdict format are untouched. Cross-audit elevation (preamble §6) still applies: a survivor confirmed by a sibling audit on the same file:line elevates to CRITICAL. Synthesis is the auditor's own job — never paste a track's or a lens's raw summary as the verdict.

### 4. Loop-until-dry for unknown-size discovery

Feature inventory has no fixed size — a monorepo may hide routes, the implicit-PRD may under-count, competitor research may surface a new table-stakes feature mid-run. Run discovery as a **loop-until-dry**, not a single pass:

```
round = 0
repeat:
    round += 1
    fan out tracks A–E (§1) over the CURRENT inventory
    adversarially verify every new candidate (§2)
    record surviving findings + newly discovered features
until (no NEW feature/gap surfaced this round)        # dry
   or (round >= 5)                                    # preamble §4 hard cap
   or (mission budget approached — R-BUDGET escalate)
```

Each round may reveal features that spawn fresh gaps (a newly found "export" route → check format depth, API parity, edge cases), so feed discoveries back into the next round's fan-out. Terminate when a full round adds nothing new ("dry"), honoring the **5-iteration cap** from QUALITY-ARSENAL-PREAMBLE §4 (mark residual unknowns `NEEDS_REVIEW` + Telegram SOS at the cap — never spin silently). The fix-and-re-audit loop in Phases 18–20 is unchanged and remains separately capped at 5.

> *Topology, not doctrine: fan out the prosecution, convict only survivors of a real refutation, sentence with the unchanged /320 matrix. The Gestalt-Popper feature prosecutor, parallelized.*
