---
name: flowaudit
description: >
  Forensic user flow audit v2. 25-phase deep analysis of every user journey, workflow, state machine,
  edge case, error recovery, and experience gap. Covers flow mapping, state verification, dead-end detection,
  permission gaps, data integrity through flows, onboarding completeness, cross-session continuity,
  error recovery paths, accessibility of journeys, flow performance, plus verdict, fix plan, fix execution,
  re-audit, and integration smoke gate.
  Score /400. Preamble v1.0 compliant. Audit → Plan → Fix → Re-audit.
  Use when user says "/flowaudit", "audit the flows", "check user journeys", "workflow audit", "flow review".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet"]
domain: flows
phases: 25
max_score: 400
read_only: false
triggers: ["flow", "user flow", "flow audit", "parcours", "workflow audit"]
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

# /flowaudit v2 — Forensic Flow Interrogation (Gestalt-Popper)

> *"A beautiful interface with broken flows is a decorated prison. The user can see the door but can't walk through it."*

---

## DOCTRINE

You are not a UX reviewer. You are a **flow prosecutor**. Every user journey is a crime scene. Every button is a promise. Every redirect is an alibi. Every error message is a confession.

**The 6 Laws of Flow Forensics (Gestalt-Popper Synthesis):**
1. **The user is always lost.** If they need to think about what to do next, the flow is broken. Every state must have exactly one obvious next action.
2. **Every flow has a dark path.** Happy paths are marketing. Error paths, edge cases, timeouts, and permission denials are reality. If you only tested the happy path, you tested nothing.
3. **Abandonment is the verdict.** If the user can reach a state with no way out, no feedback, and no progress — the flow killed them. That's a conviction.
4. **Clarity before walking (Gestalt).** Before tracing any flow, UNDERSTAND the intent. Read VISION.md, PRD, user stories. Identify the HINGE FLOW — the single journey that defines the product's reason to exist (signup→activation, purchase→delivery, etc.). Audit the hinge flow with 10x scrutiny. If the hinge flow is broken, nothing else matters.
5. **Every button is a promise (Popper).** A button labeled "Save" PROMISES to save. A progress bar PROMISES completion. A "Cancel anytime" badge PROMISES cancellation works. FALSIFY each promise:
   - Click "Save" → was data actually persisted?
   - Progress bar at 80% → is it real or cosmetic?
   - "Cancel anytime" → can you actually cancel at every point?
6. **Copy lies are flow bugs.** If a CTA says "Get started in 30 seconds" but onboarding takes 5 minutes, that's not a copywriting issue — it's a flow DEFECT. Every text claim about the experience must be verified against the actual experience.

**Gestalt Hinge Flow:** Before Phase 1, identify the ONE user journey that — if broken — makes the entire product worthless. This flow gets every phase applied at maximum depth. Other flows get proportional scrutiny based on user volume and business impact.

**Popper Flow Falsification Categories:**
- **LABEL vs ACTION** — button says "Delete" but actually archives
- **PROMISE vs EXPERIENCE** — "instant" setup takes 10 clicks
- **STATE vs DISPLAY** — entity is "cancelled" in DB but shows "active" in UI
- **COPY vs REALITY** — "unlimited" plan has hidden limits
- **FEEDBACK vs TRUTH** — "Saved!" toast but data wasn't persisted

**Mental model:** You are a confused first-time user, a power user who clicks too fast, a user with spotty internet, and a malicious actor — simultaneously. What breaks? Where do you get stuck? Where do you give up?

---

## SCOPE DETECTION (automatic from user prompt + explicit flags)

```
NATURAL-LANGUAGE EXAMPLES:
  "/flowaudit"
  → ALL flows. Full 20-phase pipeline on entire project.

  "/flowaudit the onboarding"
  → TARGETED: onboarding/signup/first-run flows only
  → Focus: Phases 7, 8, 12, 15

  "/flowaudit auth flow is broken"
  → TARGETED: auth/login/session/permission flows
  → Focus: Phases 5, 6, 9, 13

  "/flowaudit checkout and payment"
  → TARGETED: purchase/payment/subscription flows
  → Focus: Phases 4, 6, 10, 14, 16

  "/flowaudit everything, full pipeline"
  → ALL flows. ALL phases. No shortcuts.

EXPLICIT SCOPED FLAGS (used by Linear rule 43 step 8 + multi-audit orchestration):
  --url={page_url}
  → Scope all URL-based walkthroughs to this single page
  → Phase 0 discovery still maps global state machines, but walkthroughs
    (Phases 3, 4, 5, 10, 15) only exercise this URL
  → Mandatory when invoked from /linear ticket pipeline

  --files={comma-separated-paths}
  → Scope code-side checks (Phases 2 state machines, 8 data integrity, 19 analytics)
    to these files only
  → Other phases still run but do not propose fixes outside this file set

  --scope={1-line description}
  → Free-text scope note written to verdict.md and fix-plan.md
  → Example: --scope="checkout success page only, not cart"

  --ticket={TICKET_ID}
  → Link the audit to a Linear ticket ID
  → Writes audit results into audits/.linear-fix/{TICKET}/.flowaudit/ with required
    schema (score, skill_used, findings[], ticket) for rule 43 gate compliance
  → Never run without --url when --ticket is present

  --no-fix
  → Dry-run scoring only — skip Phase 22-23 fix execution
  → Still produces verdict.json + fix-plan.json, but does not modify code
  → Useful when user wants to review the fix plan before authorizing changes

RULES:
- Scope flags are ADDITIVE, not exclusive: `--url=X --files=Y,Z` means both apply
- If scope conflicts with natural language, flags win (explicit > implicit)
- If "all" or "everything": all phases, all flows
- Parse the intent, don't ask for clarification
- When dispatched from rule 43 (Linear pipeline): --url, --ticket, and --files
  are MANDATORY — abort with error if missing
```

---

## CONTEXT COMPATIBILITY (what projects /flowaudit can audit)

Not every project has browser-navigable user flows. /flowaudit's behavior in non-standard contexts:

| Project type | /flowaudit behavior | Applicable max |
|--------------|--------------------|-----------------|
| **Next.js / React web app with URLs** | Full pipeline, Playwright walkthroughs | /400 |
| **Static site (no auth, no forms)** | Skip Phases 6 (permission), 14 (payments), 17 (concurrent). Run 17 phases | /340 |
| **Mobile app (React Native / Expo)** | Replace Phase 18 (mobile web) with native-gesture walkthroughs. Full /400 if URLs exist |  /400 |
| **Backend-only API / CLI tool** | **ABORT** — /flowaudit is a USER-FLOW audit. Route to `/apiaudit` for API contracts, `/dxaudit` for CLI UX instead. Do NOT run phases with synthetic "flows" — it produces hallucinated findings |
| **Library / SDK** | **ABORT** — same as backend-only. Route to `/codeaudit` + `/dxaudit` + `/apiaudit` |
| **Desktop app (Tauri, Electron)** | Run with caveats: Playwright's webview driver required; some phases (18 mobile) become N/A |
| **Headless service / bot** | **ABORT** — no UI to walk through |

**Compatibility gate** (Phase 0 step 0.5):
```
1. Check for a web-accessible URL: DEV_URL env, .env.local NEXT_PUBLIC_*, CLAUDE.md
2. If found → continue with full pipeline
3. If missing but package.json has "react-native" or "expo" → continue in mobile mode
4. If missing AND no UI framework detected → ABORT with clear message:
   "ABORT: /flowaudit requires a user-facing UI. Detected: <project type>.
    Suggested alternatives: /apiaudit (API surface), /codeaudit (code quality),
    /dxaudit (developer experience for CLIs/libraries)."
5. Do NOT attempt to synthesize flows from README or docs — that's fabrication.
```

---

## OUTPUT CONTRACT

```
audits/.flowaudit/
├── session.log              # Audit start/end timestamps
├── discovery/
│   ├── flow-inventory.md    # Complete list of every flow discovered
│   ├── entry-points.md      # All ways a user can enter the system
│   ├── state-machines.md    # State diagrams for every entity
│   └── sitemap.md           # Page/screen/route inventory
├── reports/
│   ├── flow-completeness.md     # Phase 1
│   ├── state-machines.md        # Phase 2
│   ├── happy-paths.md           # Phase 3
│   ├── error-paths.md           # Phase 4
│   ├── dead-ends.md             # Phase 5
│   ├── permission-gaps.md       # Phase 6
│   ├── onboarding.md            # Phase 7
│   ├── data-integrity.md        # Phase 8
│   ├── cross-session.md         # Phase 9
│   ├── error-recovery.md        # Phase 10
│   ├── flow-performance.md      # Phase 11
│   ├── accessibility.md         # Phase 12
│   ├── notification-audit.md    # Phase 13
│   ├── payment-flows.md         # Phase 14
│   ├── empty-states.md          # Phase 15
│   ├── destructive-actions.md   # Phase 16
│   ├── concurrent-users.md      # Phase 17
│   ├── mobile-flows.md          # Phase 18
│   ├── analytics-coverage.md    # Phase 19
│   └── flow-entropy.md          # Phase 20
├── verdict.json             # Machine-readable scores
├── verdict.md               # Human-readable final report
├── fix-plan.json            # Prioritized fix tasks
├── fix-plan.md              # Human-readable fix plan
├── progress.json            # Live progress for monitor
└── fix-log.md               # Append-only fix log
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
~/.omega/lib/audit-runner.sh flow "$PROJECT_PATH" \
  --files="$FILES_MODIFIED" \
  --url="$URL" \
  --user-need="$USER_NEED_QUOTE" \
  --hinge="$HINGE_POINT" \
  --ticket="$TICKET_ID"
```

This invokes `~/.omega/lib/audit-gather/flow.sh` which runs:
Internal route inventory (Next.js, React Router), homepage fetch + internal-link extraction + HEAD probing (up to 30 links), form/onSubmit inventory, button-onClick count, empty-state and ErrorBoundary heuristics

Output is written to:

```
$PROJECT_PATH/audits/.flowaudit/
├── raw/                    # raw tool outputs (JSON / text per tool)
└── evidence-summary.json   # normalized findings, single source of truth for the LLM
```

When run inside a Linear-fix mission (`--ticket=ID`), the artifacts move to
`$PROJECT_PATH/audits/.linear-fix/<ID>/.flowaudit/` so multiple audits on the same
ticket can cross-reference each other (see 0.5).

### 0.2 evidence-summary.json schema

```jsonc
{
  "audit": "flow",
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

## PHASE 0: FLOW SCENE RECONSTRUCTION

> *"Before you can judge the flows, you must discover them all."*

**Pre-flight concurrency:** the mandatory canonical runner invocation already
acquires `audits/.flowaudit/.runner.lock` with `flock`. A collision exits 2; do
not create, delete, or reclaim a second PID-file lock.

**Pre-flight: progress notification channel.**
- Send Telegram start notification: `🚦 /flowaudit started on {project} — scope: {scope}`
- Every 3 phases completed: update `audits/.flowaudit/progress.json` AND send Telegram progress update
- On Phase 21 verdict: send final score + link to verdict.md
- On crash / abort / 5-iter cap: Telegram SOS (see Phase 24)

**Pre-flight: discovery drift check (for resumed audits).**
- If `audits/.flowaudit/discovery/flow-inventory.md` exists AND is older than 1h:
  - Re-run flow inventory discovery (light pass, not full phase 0)
  - Compare against existing inventory: flag new/removed routes as DRIFT
  - If drift detected: abort or user-confirm before continuing — the world moved under us

```
1. ENTRY POINT CENSUS
   Find EVERY way a user can enter the system:
   - Direct URL navigation (every route/page)
   - Deep links (email, notifications, shared links)
   - OAuth callbacks (Google, GitHub, Clerk, etc.)
   - Webhook-triggered flows (Stripe, Linear, etc.)
   - Bot commands (Telegram, Slack, Discord)
   - API endpoints that trigger UI state changes
   - QR codes, magic links, invite links
   
2. FLOW INVENTORY
   For each entry point, trace the complete journey:
   - Entry → Actions → Decision points → Outcomes
   - Map EVERY branch (if/else in the UI)
   - Map EVERY possible exit (close, back, abandon, error)
   - Note: a flow is NOT a page. It's a JOURNEY across pages.

3. ENTITY STATE MACHINE EXTRACTION
   For each core entity (user, project, order, task, session, etc.):
   - What are ALL possible states?
   - What transitions are valid?
   - What triggers each transition?
   - What UI reflects each state?
   
4. ROUTE/PAGE INVENTORY
   - Every page/screen in the app
   - Which flows pass through each page
   - Pages that exist but are unreachable (orphan pages)
   - Pages that are reachable but have no content (empty shells)
```

---

## PHASE 1: FLOW COMPLETENESS AUDIT

> *"A flow that starts but doesn't finish is worse than no flow at all."*

```
FOR EACH discovered flow:

1. ENTRY VERIFICATION
   → Can the user actually reach the starting point?
   → Is the entry point discoverable (not hidden behind obscure navigation)?
   → Does the entry point work when accessed directly (deep link)?
   → What if the user is not authenticated? (redirect to login then back?)

2. STEP COMPLETENESS
   → Does every step have a clear CTA (Call To Action)?
   → Does every step show progress? (step 2/5, progress bar, breadcrumb)
   → Can the user go back? Is going back safe? (doesn't lose data)
   → Can the user save and continue later? (long flows)

3. EXIT VERIFICATION
   → Does the flow have a clear success state?
   → Is the user told what happened? (confirmation, summary, receipt)
   → What's the next action after completion? (not a dead end)
   → Can the user redo/undo the flow?

4. ABANDONMENT HANDLING
   → What if the user closes the tab mid-flow?
   → What if the user navigates away?
   → Is there a draft/save mechanism?
   → Is partial state cleaned up? (no orphan records in DB)

SEVERITY:
- Flow starts but can't finish = CRITICAL
- Flow finishes but no confirmation = HIGH
- Flow works but can't go back = MEDIUM
- Flow works but no progress indicator = LOW
```

---

## PHASE 2: STATE MACHINE VERIFICATION

> *"Every entity has a lifecycle. If you don't define it, bugs will."*

```
FOR EACH entity with lifecycle states:

1. STATE COMPLETENESS
   → Are ALL states defined in the code? (enum, constants, or implicit?)
   → Is there a "created/draft" initial state?
   → Is there a terminal state? (completed, cancelled, archived)
   → Are there intermediate states? (processing, pending, reviewing)

2. TRANSITION VALIDITY
   → Map every valid transition: from_state → to_state
   → Is each transition guarded in code? (can't skip from draft to completed)
   → What triggers each transition? (user action, system event, timer)
   → Is the transition atomic? (no half-transitioned states)

3. INVALID TRANSITION HANDLING
   → What happens when code tries an invalid transition?
   → Is there an error? Or silent corruption?
   → Can the UI show a button for an impossible action? (e.g., "Approve" on an already-approved item)
   → Are buttons/actions disabled for invalid states?

4. STUCK STATE DETECTION
   → Can an entity get stuck with no way to progress?
   → Is there a timeout for temporary states? (processing → failed after 30min)
   → Is there an admin override for stuck entities?
   → Are stuck entities detected and alerted?

5. STATE vs UI SYNC
   → Does the UI always reflect the current state?
   → Can the UI show stale state? (browser tab open, state changed elsewhere)
   → Is there real-time sync? (WebSocket, polling, Convex subscriptions)
   → What does the user see during state transitions? (loading, optimistic update)
```

---

## PHASE 3: HAPPY PATH VERIFICATION

> *"Test the path the marketing team promised. It's the one most likely broken."*

```
FOR EACH primary user journey:

1. STEP-BY-STEP WALKTHROUGH
   → Start from the entry point
   → Take the most obvious action at each step
   → Note any confusion, friction, or unexpected behavior
   → Screenshot every step (if UI project)
   → Record time to complete

2. DATA INTEGRITY THROUGH FLOW
   → Does data entered in step 1 survive to step 5?
   → Are form values preserved on validation error?
   → Is data consistently formatted throughout? (dates, currencies, names)
   → Does the confirmation page show exactly what was submitted?

3. SPEED & FEEDBACK
   → Is every action acknowledged within 100ms? (loading state)
   → Are long operations async with progress? (not frozen UI)
   → Is there optimistic updating? (or does every action require a round trip)
   → Are success messages clear and specific? (not generic "Success!")

4. POST-FLOW VERIFICATION
   → After completing the flow, is the result visible?
   → Does the dashboard/list update?
   → Are notifications sent? (email, in-app, push)
   → Can the user find what they just created/changed?
```

---

## PHASE 4: ERROR PATH EXHAUSTION

> *"For every happy path, there are 10 sad paths and 3 evil paths."*

```
FOR EACH flow, test these error scenarios:

1. VALIDATION ERRORS
   → Submit empty required fields → clear error message?
   → Submit invalid data (wrong format, too long, XSS payload) → handled?
   → Submit boundary values (0, -1, MAX_INT, empty string) → handled?
   → Are errors shown inline (next to field) or as a global banner?
   → Are errors clearable? (not stuck on screen after fixing)

2. NETWORK ERRORS
   → What if the API call fails mid-flow? → retry? error message? data loss?
   → What if the API returns slowly (>5s)? → timeout handling?
   → What if the API returns garbage? → graceful degradation?
   → Is there offline support? Or at least a "you're offline" message?

3. PERMISSION ERRORS
   → Access a flow you don't have permission for → clear message? redirect?
   → Start a flow, lose permission mid-way → what happens?
   → Share a deep link to a protected flow → what does the recipient see?

4. CONCURRENCY ERRORS
   → Two users editing the same resource → conflict handling?
   → Same user in two tabs → state sync? data corruption?
   → Race condition: submit twice quickly → duplicate creation?

5. BROWSER EDGE CASES
   → Refresh page mid-flow → state preserved?
   → Browser back button → safe? breaks flow?
   → Copy-paste URL mid-flow → works? or 404?
   → Disable JavaScript → graceful degradation? or white screen?
```

---

## PHASE 5: DEAD END DETECTION

> *"A dead end is where the user's trust goes to die."*

```
1. NAVIGATION DEAD ENDS
   → Pages with no outgoing links/buttons (terminal pages without purpose)
   → Modals that can't be closed
   → Error pages with no "go back" or "go home" action
   → Success pages with no "what's next"

2. LOGICAL DEAD ENDS
   → States where no action is available but the flow isn't complete
   → "Pending approval" with no way to check status or cancel
   → "Processing..." that never resolves and has no timeout
   → Empty results with no guidance ("No items found" + nothing else)

3. PERMISSION DEAD ENDS
   → Reaching a page you can see but can't use (read-only with no explanation)
   → Features visible in navigation but "coming soon" when clicked
   → Admin-only features visible to regular users with no access message

4. DATA DEAD ENDS
   → Required data doesn't exist yet (profile incomplete, no payment method)
   → Circular dependency: need A to create B, need B to create A
   → External dependency: waiting on third-party that never responds
```

---

## PHASE 6: PERMISSION & AUTH FLOW GAPS

> *"Auth is not a feature. It's a prerequisite for every other feature."*

```
1. AUTH BOUNDARY VERIFICATION
   → Map every route: public vs authenticated vs role-specific
   → Test EVERY authenticated route without auth → proper redirect?
   → Test EVERY role-specific route with wrong role → proper denial?
   → Are there routes that SHOULD be protected but aren't?

2. SESSION LIFECYCLE
   → Login flow: OAuth, email/password, magic link → all work?
   → Session expiry: what happens? → silent re-auth? or kicked out mid-work?
   → Multi-device: login on phone while on desktop → both work?
   → Logout: is session fully cleared? → no stale data? no cached pages?

3. PERMISSION ELEVATION
   → Can a user escalate their permissions? (parameter tampering, URL manipulation)
   → Are admin endpoints protected server-side (not just hidden in UI)?
   → Can a user access another user's data by changing an ID in the URL?

4. INVITE & SHARING FLOWS
   → Invite link → works for new user? existing user?
   → Shared link → respects permissions? or leaks data?
   → Collaboration flow → who can see what? real-time sync?
```

---

## PHASE 7: ONBOARDING FLOW FORENSICS

> *"You get one chance at a first impression. Most apps waste it."*

```
1. FIRST-RUN EXPERIENCE
   → What does a brand new user see after signup?
   → Is there a guided setup? Or thrown into an empty dashboard?
   → Are there tooltips, tours, or contextual help?
   → How many clicks to reach the "aha moment"?

2. ACTIVATION METRIC
   → What's the ONE action that makes a user "activated"?
   → How many steps to reach it?
   → Is the path to activation the SHORTEST possible path?
   → Are there unnecessary steps before activation? (profile setup, survey, etc.)

3. EMPTY STATE HANDLING
   → Every list/table/dashboard with zero items → helpful empty state?
   → Does empty state guide the user to create their first item?
   → Are sample/demo data available? (or is the experience empty?)

4. PROGRESSIVE DISCLOSURE
   → Is complexity introduced gradually? (not all features at once)
   → Are advanced features hidden until needed?
   → Is there a clear hierarchy: essential → useful → advanced?

5. RECOVERY FROM ABANDONMENT
   → If user signs up but doesn't complete onboarding → what happens next day?
   → Is there a re-engagement email/notification?
   → Can they resume where they left off?
```

---

## PHASE 8: DATA INTEGRITY THROUGH FLOWS

> *"Data entered in step 1 must survive 50 transitions without mutation."*

```
1. INPUT → STORAGE → DISPLAY CYCLE
   → Enter data in a form → save → view it → is it IDENTICAL?
   → Special characters: &, <, >, ", ', \n, emoji → preserved?
   → Rich text: bold, links, lists → properly stored and rendered?
   → Numbers: 0, 0.1, -1, 999999999 → no precision loss?
   → Dates/times: timezone correct? DST handling? future dates?

2. CROSS-FLOW DATA CONSISTENCY
   → Data created in Flow A → visible in Flow B?
   → Data updated in one place → reflected everywhere? (no stale copies)
   → Data deleted → all references cleaned up? (no dangling pointers)

3. BULK OPERATIONS
   → Select all → action → works for 1 item? 10? 1000? 10000?
   → Bulk delete → confirmation? → undo? → cascade?
   → Import/export → roundtrip preserves all data?

4. SEARCH & FILTER CONSISTENCY
   → Search finds recently created items?
   → Filters correctly exclude/include?
   → Sort order is consistent and predictable?
   → Pagination doesn't skip or duplicate items?
```

---

## PHASE 9: CROSS-SESSION CONTINUITY

> *"The user doesn't care that you restarted the server."*

```
1. SESSION PERSISTENCE
   → Close browser → reopen → state preserved?
   → Switch devices → state accessible?
   → Clear cookies → what's lost? what's recoverable?

2. LONG-RUNNING FLOWS
   → Start a wizard, leave for 24h, return → resume or restart?
   → Start an upload, lose internet, reconnect → resume or re-upload?
   → Start a payment, bank 3DS redirects, return → complete or stuck?

3. REAL-TIME SYNC
   → Two tabs open → change in one → reflected in other?
   → Two users on same resource → live updates?
   → Websocket disconnect → reconnect cleanly? or stale state?

4. DEPLOYMENT CONTINUITY
   → During deploy, active users → graceful? or broken mid-action?
   → New version changes API → old client handles it?
   → Database migration runs → active flows survive?
```

---

## PHASE 10: ERROR RECOVERY FLOWS

> *"The measure of a system is not how it handles success — it's how it handles failure."*

```
1. AFTER VALIDATION ERROR
   → Are form values preserved?
   → Is the cursor/focus on the errored field?
   → Is the error message actionable? ("Email is required" vs "Error")
   → Can the user fix and resubmit without re-entering everything?

2. AFTER NETWORK ERROR
   → Can the user retry?
   → Is there an auto-retry with backoff?
   → Is the error message helpful? ("Check your connection" vs "500 Internal Server Error")
   → Is data preserved? (typed text not lost)

3. AFTER PAYMENT ERROR
   → Is the user charged? (verify with Stripe/provider)
   → Can they retry without double-charging?
   → Is the order in a recoverable state?
   → Is there a clear "what to do next" message?

4. AFTER CRASH/RELOAD
   → App crashes mid-flow → restart → where does the user land?
   → Is there a "resume where you left off" mechanism?
   → Are partial writes rolled back? (no zombie records)

5. UNDO / ROLLBACK
   → Can destructive actions be undone? (delete, archive, send)
   → Is there a confirmation for irreversible actions?
   → Is there a grace period? ("Undo" within 10 seconds)
   → Is the undo discoverable? (not hidden in a menu)
```

---

## PHASE 11: FLOW PERFORMANCE AUDIT

> *"A flow that takes 30 seconds when it could take 3 is a broken flow."*

```
1. TIME TO COMPLETE
   For each primary flow, measure:
   → Steps required (clicks, page loads, form fields)
   → Time to complete (seconds)
   → Compare to industry benchmarks (signup: <60s, checkout: <120s)
   → Identify: which step takes the longest? Can it be shortened?

2. UNNECESSARY STEPS
   → Steps that ask for info you already have (name asked twice)
   → Steps that could be deferred (billing before trial)
   → Steps that could be combined (address + payment on same page)
   → Steps that could be skipped (optional profile setup)

3. LOADING STATE QUALITY
   → Skeleton screens vs spinners vs blank → which is used?
   → Are loading states shown within 100ms?
   → Do loading states show progress? (determinate vs indeterminate)
   → Are long waits explained? ("This usually takes 10-30 seconds")

4. PERCEIVED PERFORMANCE
   → Optimistic updates? (UI updates before server confirms)
   → Prefetching? (next step loaded while user reads current step)
   → Progressive loading? (text first, then images, then interactive)
```

---

## PHASE 12: FLOW ACCESSIBILITY AUDIT

> *"If a flow works with a mouse but breaks with a keyboard, it works for nobody."*

```
1. KEYBOARD-ONLY FLOW COMPLETION
   → Can EVERY flow be completed with keyboard alone?
   → Tab order: logical? (not jumping randomly across the page)
   → Focus management: after action, is focus moved to the right element?
   → Modal flows: focus trapped inside? Escape closes?

2. SCREEN READER FLOW
   → Are flow steps announced? ("Step 2 of 5: Payment")
   → Are form errors announced when they appear?
   → Are success/failure states announced?
   → Are dynamic updates announced? (ARIA live regions)

3. REDUCED MOTION
   → Do animations convey information? Or just decoration?
   → Is `prefers-reduced-motion` respected?
   → Are transitions skippable?

4. COGNITIVE ACCESSIBILITY
   → Are instructions clear and concise?
   → Are error messages in plain language? (not technical jargon)
   → Is the flow forgiving? (can undo, can go back, preserves input)
   → Are time limits generous? (or at least adjustable)
```

---

## PHASE 13: NOTIFICATION & FEEDBACK FLOW AUDIT

> *"Every action deserves a reaction. Silence is the worst UX."*

```
1. ACTION FEEDBACK CENSUS
   FOR EVERY user action (button click, form submit, toggle, delete):
   → Is there immediate visual feedback? (within 100ms)
   → Is there a success confirmation?
   → Is there an error message on failure?
   → Is the feedback proportional? (minor action = subtle, major = prominent)

2. NOTIFICATION FLOW
   → What triggers notifications? (complete list)
   → Where do notifications appear? (in-app, email, push, SMS)
   → Can the user control notification preferences?
   → Are notifications actionable? (deep link to relevant page)
   → Are notifications timely? (real-time for urgent, batched for low-priority)

3. PROGRESS COMMUNICATION
   → Long operations → progress bar? status page? email when done?
   → Background jobs → user knows they're running? can check status?
   → Multi-step processes → which step am I on? how many left?

4. EMPTY FEEDBACK GAPS
   → Actions that succeed but show NO feedback (silent success)
   → Actions that fail but show NO error (silent failure)  
   → State changes with NO notification (someone else changed your data)
```

---

## PHASE 14: PAYMENT & BILLING FLOW FORENSICS

> *"Money flows are the one place where 'good enough' is never good enough."*

```
(Skip if no payment/billing in the project)

1. PURCHASE FLOW
   → Product selection → cart/summary → payment → confirmation
   → Is pricing clear at every step? (no surprise fees)
   → Is tax calculated correctly?
   → Are discounts/coupons applied visibly?

2. PAYMENT METHOD HANDLING
   → Card entry → validation → 3DS → success/failure
   → Payment method saved securely?
   → Can user update payment method?
   → Failed payment → clear recovery path?

3. SUBSCRIPTION LIFECYCLE
   → Subscribe → upgrade → downgrade → cancel → resubscribe
   → Proration calculated correctly?
   → Cancel flow: when does access end? Can undo?
   → Dunning: failed renewal → retry → notify → deactivate

4. INVOICE & RECEIPT FLOW
   → Invoices generated for every charge?
   → Receipts emailed?
   → Tax documents available?
   → Billing history accessible?

5. REFUND FLOW
   → Can admin issue refunds?
   → Is the user notified?
   → Is access adjusted accordingly?
   → Is the refund reflected in billing history?
```

---

## PHASE 15: EMPTY STATE & ZERO-DATA AUDIT

> *"The emptiest screen reveals the fullest truth about your UX."*

```
FOR EVERY list, table, dashboard, and feed:

1. ZERO ITEMS STATE
   → What does the user see? (helpful message? or blank?)
   → Is there a CTA to create the first item?
   → Is there sample data or a tutorial?
   → Does the layout still look intentional? (not broken/collapsed)

2. LOADING STATE
   → Before data arrives → skeleton? spinner? blank?
   → Is the loading state shaped like the data? (skeleton matching layout)
   → If data never arrives (timeout) → error state? retry button?

3. ERROR STATE
   → API fails → what does the user see?
   → Is there a retry mechanism?
   → Is there a fallback? (cached data, default content)
   → Is the error state styled? (not a raw stack trace)

4. PARTIAL DATA
   → Some fields missing → graceful degradation? (show what you have)
   → Required data missing → clear prompt to add it?
   → External data unavailable → placeholder? (not broken layout)
```

---

## PHASE 16: DESTRUCTIVE ACTION AUDIT

> *"Delete, remove, cancel, disconnect — the words that make users sweat."*

```
FOR EVERY destructive action:

1. CONFIRMATION
   → Is there a confirmation dialog? (not just a single click)
   → Does the confirmation explain consequences?
   → Is the confirm button NOT the default/primary? (prevent accidental clicks)
   → For high-stakes: require typing confirmation? (delete "project-name")

2. REVERSIBILITY
   → Is the action reversible? (soft delete, archive, trash)
   → If reversible: how long is the recovery window?
   → If irreversible: is the user CLEARLY warned?
   → Is there an undo mechanism? (toast with "Undo" button)

3. CASCADE EFFECTS
   → Deleting a parent → what happens to children? (cascade? orphan? block?)
   → Is the user told about cascade effects BEFORE confirming?
   → Removing a team member → what happens to their data?

4. BULK DESTRUCTIVE ACTIONS
   → Select all → delete → extra scary confirmation?
   → Is there a limit? (can't delete 10,000 items at once?)
   → Is there a progress indicator for large deletions?
```

---

## PHASE 17: CONCURRENT USER FLOW AUDIT

> *"Your app works fine for one user. How about fifty?"*

```
1. COLLABORATIVE EDITING
   → Two users editing same resource → conflict resolution?
   → Last-write-wins? Merge? Lock? → is it clear to users?
   → Is there presence awareness? ("John is also editing this")

2. SHARED RESOURCE CONTENTION
   → Two users claim the same limited resource → who wins?
   → Is the loser notified immediately? Or discovers later?
   → Can two users start the same one-time action? (duplicate processing)

3. REAL-TIME CONSISTENCY
   → User A creates item → User B sees it immediately?
   → User A deletes item → User B still sees it (stale cache)?
   → Are notifications delivered to all relevant users?

4. RACE CONDITIONS IN UI
   → Click button twice fast → double submission?
   → Two tabs submit conflicting changes → which wins?
   → Rapidly navigate → stale data from previous page?
```

---

## PHASE 18: MOBILE & RESPONSIVE FLOW AUDIT

> *"Flows designed on a 27-inch monitor break on a 5-inch phone."*

```
1. TOUCH-FIRST VERIFICATION
   → Are tap targets large enough? (44x44px minimum)
   → Are swipe gestures discoverable? (not the only way)
   → Are forms mobile-friendly? (correct keyboard types, autocomplete)
   → Are modals/drawers scrollable on small screens?

2. FLOW CONTINUITY ACROSS DEVICES
   → Start on desktop, continue on mobile → seamless?
   → Deep links work on mobile? (app vs browser)
   → Is the mobile flow the same or simplified? (intentional differences?)

3. MOBILE-SPECIFIC EDGE CASES
   → Phone call interrupts mid-flow → resume?
   → Rotate device mid-flow → state preserved?
   → Low battery warning → flow data saved?
   → Keyboard covers form fields → scrolled into view?

4. VIEWPORT ADAPTATION
   → Do multi-step flows adapt? (horizontal stepper → vertical on mobile)
   → Are tables scrollable or reformatted?
   → Are long forms broken into digestible chunks on mobile?
```

---

## PHASE 19: ANALYTICS & TRACKING COVERAGE

> *"If you can't measure a flow, you can't improve it."*

```
1. FLOW EVENT COVERAGE
   FOR EACH critical flow:
   → Is flow_start tracked?
   → Is each step tracked? (step_1, step_2, etc.)
   → Is flow_complete tracked?
   → Is flow_abandon tracked? (and at which step?)

2. CONVERSION FUNNEL VISIBILITY
   → Can you build a funnel from the tracked events?
   → Are drop-off points identifiable?
   → Are A/B test events in place? (variant assignment, outcome)

3. ERROR TRACKING
   → Are client-side errors tracked? (Sentry, LogRocket, etc.)
   → Are API errors correlated with user flows?
   → Can you replay a user's session to see what went wrong?

4. METRIC GAPS
   → Flows with zero tracking = blind spots
   → Flows with start but no end tracking = can't measure conversion
   → Events that fire but nobody looks at = noise
```

---

## PHASE 20: FLOW ENTROPY ANALYSIS

> *"Consistency is the foundation of usability. Entropy is the enemy."*

```
1. PATTERN CONSISTENCY
   → Are similar flows structured similarly? (all wizards work the same way)
   → Are similar actions named consistently? ("Save" vs "Submit" vs "Done")
   → Are similar confirmations styled consistently?
   → Are similar errors handled consistently?

2. NAVIGATION CONSISTENCY
   → Does back always mean back? (not sometimes "cancel")
   → Does the breadcrumb match the flow? (not the sitemap)
   → Are related flows discoverable from each other?

3. FEEDBACK CONSISTENCY
   → Success: always green toast? or sometimes inline? sometimes redirect?
   → Error: always red banner? or sometimes toast? sometimes nothing?
   → Loading: always spinner? or sometimes skeleton? sometimes nothing?

4. TERMINOLOGY CONSISTENCY
   → Same concept, same word everywhere? (not "project" here and "workspace" there)
   → Same action, same verb? (not "remove" here and "delete" there and "discard" elsewhere)
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

The HINGE FLOW of this audit is the locus of maximum risk/value. Compute or
read:

```bash
# If a hinge file already exists for this ticket, use it:
HINGE_FILE="$HOME/.omega/state/hinge-points-${TICKET_ID:-default}.json"

# Otherwise compute it on the fly:
${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/hinge-analyzer.sh "$PROJECT_PATH" --audit=flow --user-need="$USER_NEED_QUOTE" \
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
  "evidence_gathered": "Manual repro on prod URL; screenshot at .flow/edge-evidence/F-007.png",
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
  "audit": "flow",
  "score": 100,
  "score_raw": "<raw>/<denominator>",
  "score_normalized": 100,
  "confidence": "high|medium|low",
  "skill_used": "flow",
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
  "evidence_summary_path": "$PROJECT_PATH/audits/.flowaudit/evidence-summary.json",
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

## PHASE 21: VERDICT — FLOW FORENSIC REPORT

Score each phase 0-10, weight by severity:

```
SCORING MATRIX (400 max):
  Phase  1  (Flow Completeness)    × 3.0  = max 30
  Phase  2  (State Machines)       × 2.5  = max 25
  Phase  3  (Happy Paths)          × 2.0  = max 20
  Phase  4  (Error Paths)          × 3.0  = max 30
  Phase  5  (Dead Ends)            × 3.0  = max 30
  Phase  6  (Permission Gaps)      × 2.5  = max 25
  Phase  7  (Onboarding)           × 2.0  = max 20
  Phase  8  (Data Integrity)       × 2.5  = max 25
  Phase  9  (Cross-Session)        × 2.0  = max 20
  Phase 10  (Error Recovery)       × 2.5  = max 25
  Phase 11  (Performance)          × 1.5  = max 15
  Phase 12  (Accessibility)        × 2.0  = max 20
  Phase 13  (Notifications)        × 1.5  = max 15
  Phase 14  (Payments)             × 2.0  = max 20  (0 if N/A)
  Phase 15  (Empty States)         × 2.0  = max 20
  Phase 16  (Destructive Actions)  × 2.5  = max 25
  Phase 17  (Concurrent Users)     × 2.0  = max 20
  Phase 18  (Mobile)               × 1.5  = max 15
  Phase 19  (Analytics)            × 1.0  = max 10
  Phase 20  (Entropy)              × 1.5  = max 15
                                   TOTAL  = max 400 (excl. N/A)

NORMALIZE to 0-100: score = (raw / applicable_max) × 100

GRADE:
  90-100: S — Fortress. Every flow is bulletproof.
  80-89:  A — Solid. Minor gaps, nothing blocking.
  70-79:  B — Good. Some rough edges, most flows work.
  60-69:  C — Acceptable. Notable gaps, some broken paths.
  50-59:  D — Risky. Major flows have issues.
  <50:    F — Condemned. Users are getting stuck.
```

---

## PHASE 22: FIX PLAN (automatic)

```
Sort findings: CRITICAL → HIGH → MEDIUM → LOW
Group by flow (fixes that touch the same journey)
Dependency order (fix auth before fixing protected flows)
Generate fix tasks with: id, finding, file, line, fix, severity
Save to audits/.flowaudit/fix-plan.json + audits/.flowaudit/fix-plan.md
```

---

## PHASE 23: FIX EXECUTION (automatic)

```
Execute fixes sequentially per flow group.

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
  → Log the failure in audits/.flowaudit/fix-log.md with exact error
  → Mark as NEEDS_REVIEW (never retry same approach blindly)
  → Try alternative approach OR skip this fix

────────────────────────────────────────────────────────────────────────

FOR EACH FIX TASK (in priority order):
  a. Read the ENTIRE target file (full context)
  b. Run PRE-FIX ANALYSIS (scope collision, import shadow, cross-reference)
  c. Apply fix
  d. Run POST-FIX VERIFICATION (syntax, import, smoke test, tests)
  e. If all green → commit: fix(flowaudit): FIX-XXX description
  f. If any red → revert → log → mark NEEDS_REVIEW
  g. Verify the flow works end-to-end
  h. If fix breaks another flow → revert → mark NEEDS_REVIEW
```

---

## PHASE 24: RE-AUDIT (automatic, capped)

```
1. SERVICE HEALTH GATE (mandatory):
   → If project has systemd service: restart it, wait 10s, check is-active + logs for errors
   → If project has build step: full build must pass
   → If project has tests: full test suite must pass
   → If ANY fails: identify which fix broke it, revert

2. Re-run failing phases. Compare scores. Verify no regressions.

3. FIX-AND-REAUDIT LOOP (aligned with rule 43 step 8b):
   → Max 5 iterations per audit run — hard cap, not a suggestion
   → After each iteration: if score >= 80, exit loop and mark as done
   → If after 5 iterations score still < 80:
     - Do NOT continue (prevents infinite loops on adversarial code)
     - Mark remaining findings as NEEDS_REVIEW in verdict.json
     - Send Telegram escalation: "🆘 /flowaudit hit 5-iter cap at score {X}. Review: audits/.flowaudit/verdict.md"
     - Write loop history to audits/.flowaudit/iterations.md (score trajectory per iteration)
   → Zero tolerance for silent infinite loops
```

---

## Dynamic-Workflow Orchestration (v2)

> *"A linear prosecutor walks one alley at a time. A flow prosecutor stakes out every alley at once, then refuses to convict on a single witness."*
>
> This section governs **HOW** the 20 domain phases above are EXECUTED. It changes
> the execution *engine*, never the *evidence standard*: the Gestalt-Popper doctrine,
> the 6 Laws of Flow Forensics, the HINGE FLOW 10× rule, the SCORING MATRIX (Phase 21,
> /400), and every verdict/output schema remain exactly as defined above. When this
> section and any earlier phase appear to conflict on scope, the earlier phase wins;
> this section only parallelizes and hardens the gathering of evidence that feeds them.

### DW.1 — Decompose into independent parallel tracks (fan-out)

After Phase 0 (PROGRAMMATIC GATHER + FLOW SCENE RECONSTRUCTION) has produced the
flow inventory, entry-point census, and entity state machines, do NOT walk Phases
1–20 sequentially. Instead, partition them into **file-/journey-disjoint tracks** and
run them CONCURRENTLY via the **Workflow** tool (fan-out → adversarially verify →
synthesize, in-process). Workers ARE Opus-4.8 workflows — lean on the Workflow
primitive rather than grinding linearly (R-ORCH).

Tracks are cut along the natural seams of THIS audit's domain so they never write the
same evidence file (R-SCOPE: one writer per file):

| Track | Phases (run together, one Workflow branch each) | Disjoint along | Hinge-weighted? |
|-------|--------------------------------------------------|----------------|-----------------|
| **T-Structure** | 1 Completeness, 2 State Machines, 5 Dead Ends, 20 Entropy | entity/state-machine code + route graph | yes if hinge flow's state machine |
| **T-HappySad** | 3 Happy Paths, 4 Error Paths, 10 Error Recovery, 15 Empty States | per-journey walkthroughs (one branch per top journey) | **HINGE FLOW journey = its own branch, 10×** |
| **T-AuthSession** | 6 Permission Gaps, 9 Cross-Session, 17 Concurrent Users | auth/session/permission surfaces | yes if hinge flow crosses an auth boundary |
| **T-MoneyData** | 8 Data Integrity, 14 Payments, 16 Destructive Actions | DB/data-flow + payment provider surfaces | yes if hinge flow is purchase→delivery |
| **T-Reach** | 7 Onboarding, 11 Performance, 12 Accessibility, 13 Notifications, 18 Mobile, 19 Analytics | onboarding/first-run + cross-cutting reach surfaces | yes if hinge flow is signup→activation |

Rules for the fan-out:
- **One branch per disjoint track**, plus a **dedicated solo branch for the HINGE FLOW**
  carrying every phase at maximum depth (Phase 0's `hinge-points-<ticket>.json` /
  `hinge-analyzer.sh` chooses the journey — signup→activation, purchase→delivery, etc.).
  The hinge branch is never merged into a shared track; its findings get the 10× scrutiny
  Phase H1 §2.2 already mandates.
- Tracks sharing a file or journey are **serialized inside the same branch**, never split
  across branches (no two writers on one `reports/*.md` or one source file).
- Each branch reads the SAME `evidence-summary.json` (read-only) but writes ONLY its own
  `reports/<phase>.md` files — the Output Contract layout is unchanged.
- Skipped phases (N/A per the CONTEXT COMPATIBILITY table — e.g. Phase 14 on a no-payments
  app) simply drop out of their track; the track still runs its remaining phases.
- Parallelism is for **evidence gathering and per-phase scoring drafts only**. Scoring,
  normalization, and the verdict stay a single-writer step (DW.3) — never let a branch
  emit the final `verdict.json`.

### DW.2 — Adversarially verify every finding (≥2-of-3 independent lenses)

A finding emitted by any branch is a **suspect, not a conviction** (Law 7: "the flow is
guilty until every path is walked"; R-VERIFY: a delegate's own 'done' is an input, never
the verdict). Before a finding is allowed into the scoring synthesis, it MUST survive
**≥2 of 3 independent lenses**. Run the three lenses concurrently per finding; a finding
that clears fewer than 2 is **killed** (demoted to `info`, excluded from severity scoring,
recorded with the lens that defeated it).

The three lenses, mapped to THIS audit's Popper Flow Falsification categories
(LABEL vs ACTION / PROMISE vs EXPERIENCE / STATE vs DISPLAY / COPY vs REALITY /
FEEDBACK vs TRUTH):

1. **REPRODUCE** — walk the exact journey that triggers it. Dynamic Playwright probe on
   the prod/deployed URL (R-TEST / R-PROD: never a local dev server), click the button,
   submit the form, hit the dead end. Capture a screenshot/trace to
   `.flow/edge-evidence/`. *"Did the 'Save' button actually fail to persist when I clicked
   it on the live surface?"* If it cannot be reproduced from any real UI path → lens FAILS.
2. **REFUTE** — actively try to PROVE the finding wrong (the Popper test of Phase H1 §2.1,
   one lens of three). Read the route handler for middleware the static scan missed; grep
   for the guard that already disables the impossible-state button; re-run a flaky timing
   walkthrough twice to rule out a one-off network blip. A real falsification → lens FAILS
   and the finding is killed with `falsified_at:<evidence>`.
3. **CROSS-CHECK** — confirm via an INDEPENDENT source: the matching entry in
   `evidence-summary.json` (`cross_tool_confirmed`), a sibling audit's
   `evidence-summary.json` (Phase 0.5 / H1 §2.5 — debugaudit's broken page, secaudit's auth
   gap, dataaudit's orphan), or the entity state-machine map from Phase 0. Agreement here
   sets `cross_audit_confirmed: true` and bumps severity one level (existing rule).

Verdict per finding: **survives** (≥2 lenses pass) → carried into scoring at its
(possibly cross-bumped) severity; **killed** (<2) → `severity:"info"`, never scored, but
LOGGED in `verdict.json.killed_findings[]` with `{finding_id, lenses_passed, killer_lens,
evidence}` so the kill is auditable (R-CITE: evidence or it didn't happen). HINGE FLOW
findings face all three lenses at 10× depth regardless of how many pass — the hinge is
never convicted or acquitted on thin evidence.

### DW.3 — Synthesize survivors back into the EXISTING scoring matrix (unchanged)

Synthesis is the single-writer step and it is YOUR job, not a branch's — never paste a
branch's summary as the verdict (R-ORCH). Fold ONLY the **surviving** findings back into
the audit exactly as already specified — nothing about scoring changes:

1. Collect every track's per-phase score draft + its surviving findings.
2. Apply the **Phase 21 SCORING MATRIX verbatim** — same 20 weights (Completeness ×3.0,
   State Machines ×2.5, … Entropy ×1.5), same `max 400`, same N/A handling, same
   `score = (raw / applicable_max) × 100` normalization, same S/A/B/C/D/F grade bands.
3. Run the **Phase H1 synthesis (§2.1–2.7)** over the merged, survivor-only finding set:
   Popper falsifiable_tests, hinge_findings, user_need_match, edge_cases, cross_audit_links,
   and the **§2.7 score gate** (100/100 still blocked unless every §2.7 condition holds).
4. Emit the SAME `verdict.json` schema (hybrid v2, §2.6) with these additive fields only:
   `parallel_tracks_run`, `killed_findings[]`, and a `verification_lenses` summary
   (`{finding_id, lenses_passed:["reproduce","cross-check"], survived:true}` per scored
   finding). No weight, phase, grade band, or required field is added, removed, or reordered.
5. Hand off to Phase 22 (FIX PLAN) → Phase 23 (FIX EXECUTION, Do-No-Harm gate intact) →
   Phase 24 (RE-AUDIT, 5-iter cap intact) unchanged.

### DW.4 — Loop-until-dry for unknown-size discovery

Flow discovery has no fixed N: deep links, OAuth callbacks, webhook-triggered journeys,
bot commands, and invite/share paths surface new flows mid-audit. When the flow inventory
is open-ended, drive the fan-out as a **loop-until-dry** (the goal-loop lives INSIDE this
workflow, never wrapped around it — R-GOAL):

```
queue ← all flows/journeys from Phase 0 inventory
until queue is dry:
  fan-out a Workflow round over the current queue (DW.1 tracks + hinge branch)
  adversarially verify every finding (DW.2, ≥2-of-3); kill the unconvicted
  NEW flows discovered while walking (a redirect to an unmapped route, a deep link,
    a previously-hidden state transition) → DRIFT-check vs Phase 0 inventory →
    enqueue them
  if a full round adds zero new flows AND zero new surviving findings → queue is dry
bound: 5 rounds max (Audit Verification Contract re-audit cap); on round 5 still
  finding new flows → stop, mark residual flows NEEDS_REVIEW in verdict.json,
  Telegram SOS (Preamble §7) — never loop silently (L4: enumerate + finish, no silent drop)
```

This guarantees coverage of an unknown number of journeys without an unbounded run, then
converges into the single DW.3 synthesis above. Tokens are unlimited and time is not a
constraint (L5) — quality of coverage is. No `--quick`/`streamlined`/`lightweight` variant
of this orchestration is permitted; depth per phase is never degraded to parallelize.

---

## CROSS-COMMAND BRIDGE

```
/flowaudit discovers code bugs → creates FIX tasks (doesn't punt to /codeaudit)
/flowaudit discovers UI issues → creates FIX tasks (doesn't punt to /uiuxaudit)
/flowaudit discovers both → fixes both. It owns the flow end-to-end.

Integration:
  /codeaudit  →  Is the code correct?
  /uiuxaudit  →  Is the interface correct?
  /flowaudit  →  Is the EXPERIENCE correct?
  
  Together they form the QUALITY TRIFECTA:
  Code × Interface × Flow = Production-Grade
```

---

## LAWS

1. **Every flow has exactly one happy path and infinite sad paths.** Test the infinite.
2. **A dead end is a bug.** Not a "known limitation." Not "by design." A bug.
3. **The user's mental model is reality.** If they expect back to go back, it must go back.
4. **Empty states are first impressions.** An empty dashboard is a failed onboarding.
5. **Destructive actions are irreversible trust decisions.** Handle them like nuclear launch codes.
6. **Concurrency is the default.** Single-user testing is a fairy tale.
7. **The flow is guilty until every path is walked.**

---

*"/flowaudit v1.1 — Map. Walk. Break. Fix. Every path, every state, every edge case. /400."*
*Updated 2026-04-14: scoped invocation flags (--url/--files/--scope/--ticket/--no-fix), non-UI context ABORT gate, Phase 24 5-iteration cap, Phase 0 concurrency lock + Telegram progress + discovery drift check.*

---

## § 100/100 COMPLIANCE CERTIFICATE — v1.2 — 2026-04-14T10:43:29Z

> *Final gap-closure layer. Every finding from the prior /flowaudit-of-all-audits analysis (flowaudit-specific section) is now explicitly resolved below.*

### Compliance Matrix (machine-verifiable by /metaudit)

```json
{
  "audit": "flowaudit",
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

**Gap 1: JSON schemas inline → RESOLVED via preamble §6 reference**
verdict.json, fix-plan.json, telemetry.json, progress.json schemas fully specified in `../_shared/QUALITY-ARSENAL-PREAMBLE.md §6`. Emit `preamble_version: "1.0"` for /metaudit verification.

**Gap 2: Cross-session resume protocol → RESOLVED**
```
On graceful abort (SIGTERM/SIGHUP) or crash:
  Write audits/.flowaudit/crashed-at.txt with:
    phase: <current phase number>
    iteration: <current fix-reaudit iteration>
    timestamp: ISO8601
    mission: <original scope/args>
On next /flowaudit invocation:
  If crashed-at.txt exists AND mtime < 24h:
    Prompt: "Previous run crashed at phase X iteration Y. Resume? [Y/n]"
    If Y: continue from phase X (skip completed phases); merge new findings with existing verdict.json
    If n: archive old audits/.flowaudit/ to .flowaudit.bak.$(date +%s)/ and start fresh
```

**Gap 3: Sample output examples → RESOLVED**
Sample `verdict.md` skeleton:
```markdown
# /flowaudit Verdict — {project} @ {timestamp}
**Score: {N}/100 ({grade})** — hinge flow: {hinge_description}
## Summary
{2-3 sentences}
## Phase scores
| # | Phase | Raw | Max | Applicable |
| 1 | Flow Completeness | 28 | 30 | ✅ |
...
## Critical findings
{auto-generated from findings[] filtered severity=CRITICAL}
## Next actions
{from fix-plan.md top 5}
```
Referenced by `/metaudit --focus arsenal` for output-shape verification.

**Gap 4: Backend-only/CLI handling → RESOLVED via preamble §5 ABORT gate + Phase 0 compatibility check (added in v1.1).**

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
/metaudit --focus arsenal --scope="flowaudit only"
# Expected result:
#   preamble_compliance_score: 11/11 (full)
#   audit_specific_compliance: 100/100
#   overall: 100/100
```

---

*flowaudit v1.2 — 100/100 compliance achieved via QUALITY-ARSENAL-PREAMBLE.md v1.0 + audit-specific gap closures above. Certified 2026-04-14.*

---

## MANDATORY BEFORE/AFTER VERIFICATION (v1.1 — added 2026-04-14)

**Read `../_shared/AUDIT-VERIFICATION-CONTRACT.md` before ANY fix execution.**

Every fix MUST follow the "Do No Harm" protocol:

1. **PRE-FIX BASELINE** — grep all references, capture functional state (syntax/parse/load), save to `audits/.<audit-id>/baseline/`.
2. **APPLY FIX** — normal execution.
3. **POST-FIX CHECK** — repeat every baseline check. If any PASSED→FAILED transition occurs, revert immediately.
4. **BREAKAGE SCAN** — grep for old paths across ecosystem, must return 0 non-ephemeral hits.
5. **BEFORE/AFTER MATRIX** — produce `audits/.<audit-id>/before-after.md` with functional status table per affected item.

**An audit that breaks 1 working thing is WORSE than no audit.** Do NOT claim "done" without `before-after.md` showing zero regressions.

Full contract: `../_shared/AUDIT-VERIFICATION-CONTRACT.md`
