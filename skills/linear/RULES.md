# Linear Feedback-Resolution Protocol (OmegaOS, v2 — Workflow-driven)

> Single source of truth for `/omg-linear`. The `SKILL.md` next to this file is only a launcher;
> this document is the full, non-negotiable protocol. **Read the operator's intent before acting.**
> Don't auto-pilot into ticket-fetching mode.
>
> **Self-contained.** This protocol has NO maintainer-private dependencies — it never reads
> `~/.claude/...`, and it does not use any external `audit-selector.py` or `linear-ticket-gate.sh`
> script. Audits run through the shipped OmegaOS Quality Arsenal (`/omg-audit`); the quality gate is
> enforced inside the Workflow described in §5 against the per-ticket artifacts in §3/§5. A fresh
> `git clone && ./install.sh` ships everything this protocol needs.
>
> Shipped location: `${OMEGA_DIR:-$HOME/.omega}/skills/linear/RULES.md`. The launcher Reads this exact
> path before running. Never execute `/omg-linear` without having Read this file in the current
> session.

---

## §0 — TRIGGER GUARD (activates ONLY when the operator explicitly signals Linear)

At least ONE of these must appear in the operator's prompt (case-insensitive, EN + FR):

### FIX-mode triggers (implement + audit)
- The keyword `linear`, `/linear`, or `/omg-linear`
- A phrase: `fix linear`, `resolve linear`, `linear feedback(s)`, `linear ticket(s)`,
  `linear backlog`, `resolve feedback`, `regler les feedbacks`, `règle les feedbacks`,
  `traite les feedbacks linear`, `corrige les tickets linear`
- An explicit ticket id in Linear format: `ABC-42`, `KOM-7`, `PROJ-221`, etc.
- A `linear.app/...` URL combined with fix/resolve intent

### VERIFY-mode triggers (re-audit, no new code unless a regression is found)
- `verify done linear`, `revérifie linear`, `refais linear`, `audit done linear`,
  `validate done linear`, `recheck linear`, `re-verify linear`
- A Linear **state word** + audit/check intent: `in review`, `in-review`, `in progress`,
  `done linear`, `closed linear`, combined with `check` / `audit` / `verify` / `revérifie`

### CONTEXT RULE
When the current session is already operating on a Linear-tracked project (the project has a Linear
team configured — e.g. a Linear project marker in `.orchestrator/`, or a `LINEAR_API_KEY` plus a
Linear project link in its config), a **state word alone** (`in review`, `in progress`, `todo`,
`backlog`) combined with an action verb (`check`, `audit`, `verify`, `refais`) is **sufficient** to
trigger. The project context already implies Linear.

### Anti-triggers — do NOT auto-invoke on:
- The generic word "feedback" **without any** Linear keyword or state word.
- Generic "ticket" / "backlog" standalone (needs "linear" or a state word).
- General build / refactor / deploy / copywriting missions.
- Status questions ("what's the state of the project?", "show me progress", "quel est l'état ?") —
  these are reporting, not audit requests.
- Any prompt that clearly refers to a non-Linear topic.

**When in doubt, ask the operator once and default to non-Linear.** Never mention Linear in a
response unless the operator mentioned it first or asked for ticket work. If the prompt matches no
trigger above, skip the Linear pipeline entirely.

---

## §1 — SETUP, ENVIRONMENT & API KEY

### Path conventions (de-hardcoded — read once)
| Symbol | Meaning | Default |
|---|---|---|
| `${OMEGA_DIR}` | OmegaOS state/config home (gitignored; holds secrets) | `$HOME/.omega` |
| `${PROJECT_DIR}` | The Linear-tracked project's repo root | the project you were dispatched into |
| `${ARTIFACTS}` | Per-ticket evidence dir inside the project | `${PROJECT_DIR}/.linear-fix/<TICKET_ID>/` |
| `omega` | The OmegaOS CLI (built from source by `install.sh`, on `PATH`) | `~/.local/bin/omega` |

```bash
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
```

Selection, gating, and audit orchestration go through the `omega` audit subcommands plus the
`/omg-<name>audit` skills and the OmegaOS Workflow engine. There is **no** external selector script
and **no** external gate script — both are expressed in this document.

### One-time app setup (`/omg-linear-setup`)
Before `/omg-linear` can resolve tickets, the operator's app needs to be able to file them. A
first-time user runs **`/omg-linear-setup`**, which installs:
- an **in-app feedback widget** (captures a screenshot, the page URL, the clicked element's selector,
  and the browser console at report time),
- the **Linear labels** the pipeline keys off (e.g. `feedback`, plus a review label), and
- the **API route(s)** in their app that receive a widget report and create a Linear issue
  (description + console logs + a screenshot attached as the first comment).

`/omg-linear` itself does not require the widget — it works against any Linear team — but the widget
is what makes end-users able to file the rich tickets this protocol thrives on.

### API key lookup (in order; first hit wins)
```bash
# 1. project .mcp.json (Linear MCP server env), then 2. project .env.local, then 3. ~/.omega
LINEAR_API_KEY="$(python3 -c "import json,sys;print(json.load(open('.mcp.json'))['mcpServers']['linear']['env']['LINEAR_API_KEY'])" 2>/dev/null)"
LINEAR_API_KEY="${LINEAR_API_KEY:-$(grep -m1 '^LINEAR_API_KEY=' .env.local 2>/dev/null | cut -d= -f2-)}"
LINEAR_API_KEY="${LINEAR_API_KEY:-$(grep -m1 '^LINEAR_API_KEY=' "$OMEGA_DIR/credentials/linear.env" 2>/dev/null | cut -d= -f2-)}"
```
If no key resolves → ABORT and tell the operator where to put it. Never guess or hardcode a key.
Secrets live in `${OMEGA_DIR}` (gitignored), never in the repo (R-ENV).

All Linear reads/writes use the GraphQL API at `https://api.linear.app/graphql` with header
`Authorization: $LINEAR_API_KEY` (**not** `Bearer`). Screenshot downloads:
`curl -sL -H "Authorization: $LINEAR_API_KEY" <url> -o <file>`.

---

## §2 — INTENT FIRST, THEN THE FOUR MODES

### Understand intent before acting
When the operator mentions Linear: read the actual prompt, check any URLs, match intent to action.

| Operator says | Intent | Action |
|---|---|---|
| "Fix the Linear tickets" | Resolve open issues | Fetch open tickets, fix them via the v2 Workflow |
| "Look at this Linear project [link] and build what's described" | Project = spec/PRD | Read the project as requirements, plan with `/omg-planner` (NOT ticket-by-ticket) |
| "Fix ABC-42" | One specific ticket | Fetch that ticket, fix it |

If unsure what the operator wants, **ask** (interactive sessions only — dispatched workers decide and
proceed per Law L3).

### The four operating modes
Two are detected from prompt keywords; two are detected **per-ticket** by inspecting comment history
and state transitions.

| Mode | Trigger | Fetches | Runs |
|------|---------|---------|------|
| **FIX** (default) | `/omg-linear`, `fix linear`, `regler les feedbacks`, or no keyword | OPEN tickets (`state.type NOT IN [completed, canceled]`) | Full protocol (§3) |
| **VERIFY-DONE** | `verify done linear`, `revérifie linear`, `audit done linear` | DONE tickets (`state.type IN [completed]`) | Audit gate only (§5) scoped to the original fix; re-open on fail |
| **RE-REVIEW** (auto, per-ticket) | non-Done ticket whose **last comment author is the operator** (a human, not the bot) | the single ticket the operator re-opened | Full protocol; the operator's latest comment is the NEW authoritative spec |
| **REGRESSION** (auto, per-ticket) | non-Done ticket with ≥1 prior bot `Fix Verification Report` AND a history showing a previous Done→reopen (or a state revert) | the single ticket whose prior fix failed | Full protocol + deep live root-cause; banned same-files repeat; mandatory "Why prior attempts failed" section |

**Mode priority when several match for one ticket: REGRESSION > RE-REVIEW > VERIFY-DONE > FIX.**
REGRESSION wins because confirmed-failure data outranks the latest-comment check.

Rules:
- If BOTH a FIX and a VERIFY keyword appear in the prompt → ask the operator, do not guess.
- If zero tickets exist for the chosen mode → report empty, do NOT silently fall back.
- RE-REVIEW and REGRESSION are checked **per ticket** regardless of the invocation mode: while running
  FIX, inspect each fetched ticket — if it qualifies, it switches modes for that ticket.
- Echo the detected mode before starting each ticket. Never fall back silently.

### Telling "operator" from "agent" (no person is ever named)
This protocol ships to any user, so it never keys off a specific human's name. Identify the operator
(any human reviewer) vs the agent (this automation) in order:
1. **`user.isMe` on the comment author** — Linear marks whether a comment was authored by the
   API-key's own account (i.e. this bot). `isMe == false` on the last comment = a human wrote it.
2. **Configured reviewers (optional)** — if `${OMEGA_DIR}/skills/linear/config.toml` defines
   `reviewer_identities = [...]`, a last comment whose `name`/`displayName`/`email` matches any entry
   also counts as the operator.
3. **Ambiguous single-user workspace** (bot key == the only human) — fall back to FIX and note the
   assumption in the run log.

> Rationale: the proven design keyed RE-REVIEW off one maintainer's name. OmegaOS keys it off
> "last human comment ≠ the agent's own comment", which generalizes to **any** operator.

### Detecting RE-REVIEW (the operator-rejected-the-fix scenario)
A human operator verifies a previously-resolved ticket, decides it's not good, moves it back out of
the review state, and posts a comment explaining what's still wrong (often with a fresh screenshot).
```graphql
{ issue(id:"UUID"){ state{name type} comments(orderBy:createdAt,last:1){ nodes{ user{name displayName email isMe} body createdAt } } } }
```
If the ticket is NOT Done AND `lastComment.user.isMe == false` → **RE-REVIEW** for that ticket:
- Read the FULL ticket: original description + EVERY comment chronologically + EVERY screenshot.
- Download the operator's latest screenshot(s) and Read each.
- The operator's latest comment becomes the **authoritative new spec**; the original description is
  downgraded to historical context.
- The "What was requested" section of the new Fix-Verification comment MUST quote the operator's
  latest message **verbatim** (not the original ticket text).
- Run the full protocol; the BEFORE state must reflect the CURRENT (supposedly-fixed) page.

### Detecting REGRESSION (the prior fix was a false positive)
A previously-"fixed" ticket was moved back out of review because the fix did NOT actually work, or the
feature is not live. The prior verification report claimed success but runtime disagreed — a Law-L1
violation. Detection: the ticket is NOT Done, its comments contain ≥1 prior `## Fix Verification
Report` (or `Status: RESOLVED`) authored by the bot, AND its state history shows a previous
Done/Completed transition (or a revert after the bot's comment).

REGRESSION is the most serious mode — treat prior attempts as **confirmed failures**, not starting
points:
1. Read EVERY prior Fix-Verification comment chronologically; extract commit hash, files changed,
   claimed root cause, claimed fix.
2. List the failed attempts explicitly in the working brief (the "PRIOR FAILED ATTEMPTS" block, §5.1).
3. **Forbid repeating the same approach.** If a prior fix touched `src/foo.ts:42` with approach X, the
   new attempt MUST either touch a different layer, apply a fundamentally different technique, or prove
   the prior fix was correct in code but the issue is environmental (deploy, cache, flag, env var,
   build artifact) — with evidence.
4. **Diagnose with LIVE runtime evidence FIRST** (Law L1): real prod logs, real screenshots NOW, real
   network requests, real console state — hypothesis-driven, not pattern-matched.
5. The new comment MUST include a **"Why prior attempts failed"** section quoting each prior report and
   explaining the false-positive cause. Without this section, the gate fails.

---

## §3 — THE PROTOCOL (per ticket; 10/10 standard, non-negotiable)

Every step emits artifacts to `${ARTIFACTS}` = `${PROJECT_DIR}/.linear-fix/<TICKET_ID>/`. Create it
first: `mkdir -p "${PROJECT_DIR}/.linear-fix/<TICKET_ID>"`.

```
1.   DEEP ANALYSIS      Download every screenshot from comments, Read each, parse the full
                        description + ALL comments chronologically; resolve the per-ticket mode.
1.5. AUTH PRE-FLIGHT    Prove the page is reachable in an authenticated state; capture the HTTP
                        status. Non-200 → ABORT (never "pass").
2.   BEFORE CAPTURE     Authenticate → navigate → screenshot + console BEFORE any code change.
3.   IMPLEMENT FIX      Surgical change based on the complete analysis. The build MUST pass.
4.   AFTER CAPTURE      Same auth, same URL, same viewport → screenshot + console AFTER the fix.
4.1. MULTI-STEP AFTER   If the ticket describes a workflow / conversation / N-phase flow, capture
                        after-step-1.png … after-step-N.png — one screenshot per stage.
5.   SELF-VERIFY        The 5 mandatory questions, all YES (covering every stage if multi-step).
6.   STRICT COMMENT     The exact template (§6): Before/After (all stages) + verify URL + console diff;
                        save the identical body to ${ARTIFACTS}/linear-comment.md.
7.   COMMENT GATE       Confirm the comment has every required section before audits.
8.   /omg-audit GATE     omega audit select → run each selected /omg-<name>audit, 100/100 each,
                        + adversarial dual-pass + intent verification (§5).
8b.  FIX-AND-REAUDIT    Any audit < 100 → fix every finding, re-run only the failing audits (≤5 iter).
9.   MOVE TO IN REVIEW  Move to the neutral review state (§7). NEVER Done — a human marks Done.
```

### Step 1 — DEEP ANALYSIS (not just the title)
For EACH ticket extract and analyze ALL of:
1. **Title** — the short summary.
2. **Description** — the FULL text with exact instructions.
3. **Screenshots** in comments (from the feedback widget AND/OR a human): parse EVERY comment body for
   `![...](https://uploads.linear.app/...)` and any image attachment URL, download EACH
   (`curl -sL -H "Authorization: $LINEAR_API_KEY" <url> -o "${ARTIFACTS}/comments/<id>.jpg"`), Read EACH
   (page shown, element highlighted, current state, position, what's wrong).
4. **Console logs** from the description (the feedback widget captures them).
5. **Metadata** — page URL, CSS selector, element text, user agent.
6. **Comments** — read EVERY comment oldest→newest, with author + timestamp. The latest comment can
   fully override the original spec.
7. **Last-comment author / state check** — drives mode selection (RE-REVIEW / REGRESSION, §2). Echo:
   `Ticket <ID>: lastComment by <author> (isMe=<bool>) at <ts> → mode = <FIX|RE-REVIEW|REGRESSION>`.

**FORBIDDEN:** reading only the title; skipping screenshot download/analysis; ignoring element
positioning; skipping the last-comment/state check.

### Step 1.5 — AUTH PRE-FLIGHT (mandatory, before BEFORE capture)
Most real tickets live on protected routes. Prove the worker can actually reach the page
**authenticated** before capturing BEFORE. Capture the literal HTTP status to
`${ARTIFACTS}/preflight-status.txt` (trimmed; must be exactly `200`).

**HTTP status → verdict map (HARDCODED, NO EXCEPTIONS):**

| HTTP status | Verdict | Action |
|---|---|---|
| 200 | OK | Continue to Step 2 |
| 301 / 302 | OK | Follow the redirect, then continue |
| 401 | **ABORT** | Auth failure — fix auth, do NOT advance the ticket |
| 403 | **ABORT** | Forbidden — NEVER interpret as "pass" |
| 404 | **ABORT** | Wrong URL — re-read the ticket, do NOT guess |
| 5xx | **ABORT** | Server broken — report an infrastructure issue |

**FORBIDDEN INTERPRETATIONS (automatic failure / cheating):**
- "403 means the page is protected so the fix works" → FALSE.
- "401 is expected, marking as PASS" → FALSE.
- "I couldn't reach the page so I assumed the fix is fine" → FALSE; ABORT instead.

If pre-flight fails (non-200), post a comment "Auth pre-flight failed (`<status>`) — investigation
required" and **leave the ticket in its current state**. Do NOT fix or advance it. (A 403/401/blocked
surface is an ABORT, never a PASS — Law L5, L1.)

### Step 2 + 4 — BEFORE / AFTER browser capture (auth + console)
Capture real runtime, not assumptions (Law L1). Use the project's deployed / prod URL — never spin up
a local dev server to test (R-TEST); the only exception is brand-new code not yet deployed. Browser
automation goes through the **Playwright CLI via Bash** (R-CLI), never an MCP browser tool.

- **Authentication:** for protected routes, establish an authenticated session through the app's
  documented test-auth path (a test/seed account, a backend-minted session, or the operator's
  configured auth helper). Use the **same** auth, URL, and viewport for BEFORE and AFTER so the only
  variable is your fix. Never hardcode a maintainer-specific account or credential path — resolve it
  from the project's own config / `${OMEGA_DIR}` credentials.
- **Save:** BEFORE screenshot → `${ARTIFACTS}/before.jpg` (< 2h old at gate time); AFTER →
  `${ARTIFACTS}/after.jpg` (< 2h old).
- **Console capture at every step** (console errors are the #1 invisible regression):

  | Step | Capture | Save to |
  |---|---|---|
  | Pre-flight | `performance.getEntries()` + console | `${ARTIFACTS}/preflight-console.json` |
  | BEFORE | full console (errors + warnings + logs) | `${ARTIFACTS}/before-console.json` |
  | AFTER | full console | `${ARTIFACTS}/after-console.json` |
  | Each audit | console during audit navigation | inside the audit's verdict.json |

- **Console regression rule:** if AFTER has MORE errors than BEFORE → the fix introduced a regression →
  fix it before proceeding. **Zero tolerance for new console errors.**

### Step 3 — IMPLEMENT FIX (surgical)
Fix the **root cause**, not the symptom (Law L2). Touch only the files the root cause requires
(R-KARPATHY — surgical changes; every changed line traces to the ticket). The build must pass (use the
project's own build; in OmegaOS internals prefer the omega-native build over an ad-hoc `npm run build`).
Commit with a conventional message that carries the ticket id: `fix: [TICKET-ID] <one-line summary>`
(this lets verification trace the commit to the ticket via `git log --oneline | grep`).

### Step 4.1 — MULTI-STEP AFTER (mandatory when the ticket is a flow)
A single screenshot hides regressions that only appear mid-flow (is the agent reply visible? is form
step 2 reachable? did the confirmation modal render?). If the description, title, or operator's latest
comment matches any of: `workflow`, `agent`, `chat`, `conversation`, `étape`, `phase`, `wizard`,
`onboarding`, `then/ensuite/puis/enfin`, `first/finally`, `multi-page`, `multi-étape`, or a described
sequence ("I do X, then Y, then Z…") — you MUST produce `after-step-1.png … after-step-N.png`, one
screenshot per stage, plus a Stage Map: `step-N | action | expected | observed | PASS|FAIL`. A single
static state keeps one `after.jpg`. A multi-step ticket with only ONE AFTER screenshot does NOT pass
the comment gate; the comment embeds each stage inline under an `### After State (stages)` section
ordered by N.

### Step 5 — SELF-VERIFY (all five YES; if any is NO, go back and fix)
1. Does the AFTER screenshot show the EXACT change requested (and address the operator's latest comment
   if RE-REVIEW)?
2. Would the user who submitted this feedback be satisfied?
3. Are there ZERO new console errors vs BEFORE?
4. Did I address EVERYTHING in the description, not just the title (and did the build pass / is it live)?
5. Are there zero visual regressions on the same page — and, for a multi-step flow, does EVERY stage
   satisfy the spec?

### Step 6.5 — SAVE COMMENT TO THE CANONICAL PATH (mandatory)
Immediately after posting the Linear comment, save the EXACT posted body to
`${ARTIFACTS}/linear-comment.md` (exact filename — this is the ONLY path the §7 gate reads). Post **one
consolidated comment** containing ALL required sections; splitting into multiple comments breaks
verification (the gate reads one file). If the save fails (disk full, wrong path), treat it as a
BLOCKER: report `failed` with a pending action "linear-comment.md save failed", not `done_clean`.

---

## §4 — DISPATCH & PARALLELISM (file-disjoint = parallel, overlap = serial)

The v2 engine (§5) parallelizes via the OmegaOS Workflow primitive. The invariant is **one writer per
file** (R-SCOPE):

```
WRONG:  two branches editing the same component simultaneously → lost writes, flaky builds
WRONG:  three tickets on three different files serialized for no reason → 3× slower than needed
RIGHT:  pack tickets with DISJOINT file footprints → run those in parallel; overlapping tickets
        wait for the next wave; vague/broad tickets (no identifiable footprint) run alone.
```

How to pack:
1. For each pending ticket, infer its probable file footprint from the title + description (explicit
   paths, component names, route fragments) — grep the project to confirm. Classify `narrow` (confident
   footprint) or `broad` (vague / project-wide).
2. Greedy-pack up to N `narrow` tickets whose footprints are pairwise disjoint into one parallel wave.
   If no narrow candidates are eligible, run one `broad` ticket alone (sequential fallback).
3. Respect priority within a wave (P1 before P3, as long as footprints stay disjoint).

Stay sequential (wave size 1) for: refactors touching shared utilities, dependency upgrades
(`package.json`/lockfile conflicts), workers needing the same dev port, or any ticket whose description
says "affects many files / project-wide".

**Isolation discipline:** if a wave has >1 ticket, each branch operates in its own isolated working copy
(`git worktree add` or equivalent). Two parallel branches sharing one `.next/`, `node_modules/`, or git
index = corruption.

---

## §5 — THE v2 WORKFLOW ENGINE (fan out → adversarially verify → synthesize)

> This is the headline change from the old purely-sequential, one-ticket-at-a-time loop.

`/omg-linear` runs as an **OmegaOS Dynamic Workflow** (the `Workflow` primitive — put `/dynamic` on
line 1 of the dispatched work to opt in; see Rule R-ORCH: Workflow is the primary orchestration
primitive, above Agent and `omega spawn-worker`). The Workflow fans the eligible ticket set out in
parallel, then synthesizes — instead of grinding tickets one by one.

```
            STAGE 0 — INGEST                     fetch tickets (§9) + comments + history;
                                                 per ticket resolve mode (§2), download every
                                                 screenshot, write the brief + artifacts (§5.1)
                          │
            STAGE 1 — PARTITION (§4)             group tickets into file-disjoint waves
                          │  fan-out (one Workflow branch per ticket)
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
   TICKET BRANCH     TICKET BRANCH   …  TICKET BRANCH    triage → fix → capture → /omg-audit gate
        │                 │                 │            → adversarial verify → intent verify
        └─────────────────┼─────────────────┘
                          │  join
            STAGE 2 — SYNTHESIZE (§5, step 6)    consolidated /omg-audit on the UNION of all
                                                 resolved-ticket diffs, 100/100 gate; catches
                                                 cross-ticket regressions invisible per-ticket
                          │
            STAGE 3 — REPORT (§8)                X → review, Y reopened, Z escalated, with links
```

### Workflow stages (per ticket, as parallel branches across the file-disjoint wave)
1. **Triage branch** — fetch full context (§3 Step 1), resolve the per-ticket mode, run AUTH PRE-FLIGHT
   (§3 Step 1.5), capture BEFORE (§3 Step 2) before any code.
2. **Fix branch** — surgical edit (§3 Step 3), build passes, AFTER capture (§3 Steps 4 / 4.1). Runs in
   an isolated working copy when the wave is parallel (§4).
3. **Audit branch — the `/omg-audit` gate.** Select the audits for THIS ticket's mission and changed
   files, then run each as the real skill, scoped strictly to the ticket (modified files + ticket URL
   only — no project-wide scan, no "while we're here" expansion):

   ```bash
   OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
   FILES="$(cd "$PROJECT_DIR" && git diff --name-only HEAD~1 HEAD | tr '\n' ' ')"

   omega audit select "<ticket summary>" \
     > "$ARTIFACTS/audits-selected.json"     # → the relevant subset of the Quality-Arsenal audits
   # (omega audit select takes ONLY the mission string; per-audit scope flags like
   #  --files/--dir/--url go on the individual /omg-<name>audit calls below.)
   # then, for each audit id returned (run them in parallel):
   #   /omg-codeaudit   --files="$FILES" --url="$PAGE_URL" --ticket="$TICKET_ID" --user-need="$USER_QUOTE" --scope=ticket-only
   #   /omg-debugaudit  …      (or: omega audit run debugaudit --dir "$PROJECT_DIR")
   #   /omg-<name>audit …      each writes its verdict (e.g. audits/.<name>audit/verdict.json)
   ```

   - `omega audit select` is mission-aware: `codeaudit`, `logicaudit`, and `debugaudit` are the
     always-on baseline; it adds the relevant rest (UI/UX changes → `uiuxaudit` + `a11yaudit` +
     `motionaudit`; auth/security → `secaudit` + `apiaudit` + `dataaudit`; API/data tickets →
     `apiaudit`/`dataaudit`; perf complaints → `perfaudit`; public pages → `seoaudit`/`copyaudit`;
     etc.). Generous-by-default: prefer one audit too many over one too few. If `omega audit select` is
     unavailable, fall back to this change-driven mapping and always include the three baseline audits.
   - Each audit persists a verdict with the strict schema
     `{ "score": 100, "max": 100, "skill_used": "<name>audit", "findings": [] }` — `score` is the
     integer 100 (not "100", not 99); `skill_used` must equal the audit name (anti-cheat: proves the
     real skill ran).
   - Run each selected audit as the **real** `/omg-<name>audit` skill (or `omega audit run`). NEVER
     paraphrase a forensic protocol into prose (R-AUDIT) and NEVER hand-roll a custom scorer.
   - **Gate threshold for Linear = 100/100 on EVERY selected audit**, strict. This overrides the softer
     default `pass_threshold` (70) in `${OMEGA_DIR}/config.toml`: a reviewed ticket carries a higher bar
     than a routine end-of-mission audit. **99 is a FAIL — no partial credit.**
   - **Scope (HARD LIMITS):** `files_modified` = `git diff --name-only HEAD~1 HEAD`; `page_url` = the
     ticket URL; `element_selector` = the ticket's selector; `viewport` = from the user agent. FORBIDDEN:
     scanning files not in `files_modified`, testing pages not in `page_url`, proposing fixes outside the
     ticket scope, touching unrelated working code.
   - **Fix-and-reaudit loop (§3 step 8b):** any audit < 100 → read every finding, fix each, rebuild,
     re-run ONLY the failing audits, loop (max 5 iterations). Still failing after 5 → escalate; do NOT
     advance to In Review. Append each iteration to the comment:
     ```markdown
     ### Audit Iteration {N}
     | Audit | Score | Findings fixed |
     |-------|-------|----------------|
     | codeaudit | {X} → {Y}/100 | {fixes} |
     ```
4. **Adversarial-verify branch** — a challenger tries to **falsify** the "resolved" claim with ≥5
   concrete attack attempts (Popper, Rule R-VERIFY / R-30, ≥2-of-3 consensus); the verdict must be
   `confirmed` (record it to `${ARTIFACTS}/adversarial.json`). Then **Intent Verification** — read
   `user_quote` (verbatim from the description, or the operator's latest comment in RE-REVIEW/REGRESSION),
   `measured_behavior` (live AFTER screenshot + console + network + DB state), `change_summary`
   (`git diff --stat`), and answer all 5 in writing in `${ARTIFACTS}/intent-verification.md` (and the
   comment); all must be YES, Q4 may be N/A:
   - **Q1.** Does the AFTER screenshot show the user's described element/state correctly?
   - **Q2.** Does the described workflow now work end-to-end on the live URL (Playwright trace)?
   - **Q3.** Are the values/labels/copy EXACTLY what the user described (not paraphrased)?
   - **Q4.** If the user described a specific edge case, was it tested? (N/A if none.)
   - **Q5.** Is the fix LIVE on prod (prod URL returns 200 AND deploy timestamp > commit timestamp)?

   The audits answer "is the code correct?"; Intent Verification answers "did we solve the operator's
   problem?". **Both must pass.** A 100/100 audit set with a failed intent check is a gate FAIL — this
   catches the most insidious failure mode (code-correct but user-need-mismatched).
5. **Synthesize (you, the orchestrator)** — reconcile the branches yourself. **Never paste a branch's
   summary as the verdict** (a delegate's "done" is an input, never the decision — R-ORCH / R-VERIFY).
   Run the §7 gate. If everything passes → advance the ticket to In Review (§7). If anything failed →
   loop the ticket back to the Fix branch with the findings inlined, or escalate after the iteration cap.
6. **Consolidated cross-ticket audit (STAGE 2 — once at join; BLOCKS `done_clean`).** Per-ticket audits
   are scoped narrowly and cannot catch regressions from the **combined diff** of many fixes — two
   narrow fixes can each be perfect alone yet conflict when both ship (e.g. ticket A edits `page.tsx`,
   ticket B edits `Button.tsx`; each passes alone but the combined diff introduces a layout regression).
   After all branches join, run the Quality Arsenal on the **union of all resolved-ticket diffs**:
   ```bash
   CUMULATIVE_FILES="<union of each resolved ticket's git diff --name-only>"
   CUMULATIVE_TEXT="<concat of each resolved ticket's id + title + description>"
   omega audit select "$CUMULATIVE_TEXT"
   # → fan out each selected /omg-<name>audit in parallel on the union (pass --files="$CUMULATIVE_FILES"
   #   --dir="$PROJECT_DIR" to each audit skill), 100/100 gate + adversarial confirm.
   ```
   On failure, dispatch one fix branch that consumes the findings and re-runs ONLY the failing audits
   (max 5 iterations); still failing → the mission is `failed`. The mission cannot report `done_clean`
   until the cumulative state is also 100/100. Escape hatch for tests only: `SKIP_CONSOLIDATED_AUDIT=1`
   (logged as a loud WARN).

### Loop-until-dry across the queue
Maintain throughput by refilling waves: as soon as a branch finishes and is verified, pack the next
file-disjoint ticket into the freed slot — don't wait for a whole wave to finish before starting the
next. Continue until the queue is dry. Tokens are unlimited; there is no per-session ticket cap (process
ALL open tickets), but quality (the 100/100 gate) is never traded for speed (Law L5). The only ceiling
is the mission budget (default 500K tokens, R-BUDGET) — approaching it → escalate, don't silently
overrun.

### §5.1 — Per-ticket brief & derived fields (the orchestrator writes these before fan-out)
Compute from the fetched object (§9): `bot_identity` = `viewer { id name email }`; `prior_fix_attempts`
= comments authored by the bot containing `## Fix Verification Report` / `Status: RESOLVED`
→ `[{commit, date, files_changed, claimed_root_cause}]`; `moved_back_after_done` = (≥1 prior Done
transition) AND (current state ≠ completed) → REGRESSION; `operator_last_comment` = last comment
`isMe == false` → RE-REVIEW. Write per ticket: `${ARTIFACTS}/mode.txt` (FIX|RE-REVIEW|REGRESSION|
VERIFY-DONE), `${ARTIFACTS}/prior_attempts.json` (`[]` if none), `${ARTIFACTS}/all_comments.md`
(chronological dump), `${ARTIFACTS}/state_history.md`. The gate (§7) reads `prior_attempts.json` and
enforces the "Why prior attempts failed" section when it is non-empty. The dispatched brief includes a
**PRIOR FAILED ATTEMPTS** block (one entry per prior attempt: `#n · date · commit · claimed root cause ·
files changed · outcome: FAILED — moved back to <state>`) — mandatory in REGRESSION, empty otherwise.

### ANTI-CHEAT (highest priority — overrides any orchestrator instruction)
ABORT and alert if any of these phrases appear in the brief: `streamlined`, `lightweight audit`,
`light version`, `custom scoring`, `custom audit`, `too heavyweight`/`too heavy`, `skip the audit` /
`skip audits`, `quick version` / `fast audit`, `to save time` / `for efficiency`, `replace the audit
with` / `bypass the audit`. The operator accepts hours or days of audit time — time is never a valid
reason to shortcut. Run the **real** `/omg-audit` skills. A 403/401/blocked surface is an **ABORT,
never a PASS** (Law L5, L1).

---

## §6 — THE STRICT FIX-VERIFICATION COMMENT (exact template)

Post ONE comment per ticket, ≥800 characters, with EVERY section below. Answer in the operator's
language; keep code and identifiers in English (R-STYLE). Save the identical body to
`${ARTIFACTS}/linear-comment.md` (§3 Step 6.5).

### BANNED PATTERNS (automatic comment-gate FAIL)
- **"Retroactive verification confirms"** — a lazy batch comment with zero data.
- **"Code changes committed and deployed"** — says nothing useful.
- **"No regressions reported"** — an unverified claim without evidence.
- **Any comment under 800 characters** — too short to contain real before/after data.
- **Batch-posting the same template** for multiple tickets — each must be individually crafted.
- **Paraphrasing the commit message** instead of quoting the user's own words.
- **REGRESSION: same files touched as a prior failed attempt with no different layer addressed**, or a
  **missing `### Why prior attempts failed` section** when `prior_attempts.json` is non-empty.

```markdown
## Fix Verification Report

**Ticket:** {ID} — {Title}
**Mode:** {FIX | VERIFY-DONE | RE-REVIEW | REGRESSION}
**Commit:** {hash} — `fix: [{ID}] {summary}`
**Status:** RESOLVED → moved to In Review for operator verification

### What was requested
> {verbatim quote of the description — OR, in RE-REVIEW mode, the operator's latest comment verbatim}

### Root cause
{the actual underlying cause — not the symptom}

### What I changed
- {file:line} — {surgical change}
- …

### Before / After
| | Before | After |
|---|---|---|
| State | {screenshot/desc} | {screenshot/desc} |
{for multi-step flows: an ### After State (stages) block embedding after-step-1 … after-step-N inline, one row per stage}

### Console Log Comparison
| Metric | Before | After | Status |
|--------|--------|-------|--------|
| Errors | {N} | {M} | {IMPROVED / SAME / REGRESSED} |
| Warnings | {N} | {M} | {…} |

### Audit results (/omg-audit gate — 100/100 each)
| Audit | Score | Status |
|-------|-------|--------|
| codeaudit | 100/100 | PASS |
| debugaudit | 100/100 | PASS |
| {…each selected audit…} | 100/100 | PASS |
Adversarial dual-pass: confirmed. Intent verification (Q1–Q5): all YES.

### Verify
{clickable prod URL to the exact page/element} — Device: {viewport from user agent}

### Self-verification checklist
- [x] Addressed the exact problem described
- [x] AFTER proves the fix at the exact element/page
- [x] Zero new console errors
- [x] Build passed / live on prod
- [x] Every stage captured (multi-step)
```

> The `### Audit results` and the intent line are appended **after** §5 by the orchestrator — the
> worker writing the base comment leaves them as `(awaiting audit)`.

REGRESSION mode adds a mandatory **`### Why prior attempts failed`** section (a table quoting each prior
report by commit hash + the false-positive cause, then "This attempt is different because: {runtime
evidence}") — without it the comment gate fails. If you cannot articulate it honestly, the fix is not
ready (First Law).

VERIFY-DONE mode uses the same shape, headlined **`## Re-Verification Report`**: on PASS, leave the
ticket Done; on FAIL, post the failures and re-open the ticket (set state to In Progress) for a future
FIX run.

### GOOD vs BAD reference (calibration)
**BAD (2/10 — NEVER do this):**
```
## Fix Verification Report (Retroactive)
**Ticket:** ABC-32 · **Commit:** 8322cf4
### Verification
Retroactive verification confirms: code changes committed and deployed; build passes; no regressions reported.
### Status: RESOLVED
```
Why it's bad: no user quote, no before/after, no URL, no details. Useless for the operator.

**GOOD (10/10 — the standard):** quotes the user verbatim ("these three buttons do the same thing, not
what they say"), states the precise before ("3 buttons all open the same file picker, console 0") vs
after ("Any file → file picker, Take photo → camera, From phone → QR; console 0"), names the exact
change (`src/app/(client)/start/[token]/documents/page.tsx:145-180 — distinct onClick per button`),
gives a clickable verify URL + device, and a self-check. Verifiable in 30 seconds.

---

## §7 — NEUTRAL IN-REVIEW HANDOFF + THE BLOCKING QUALITY GATE (the agent NEVER self-marks Done)

When the gate passes, move the ticket to a **neutral review state for the human operator to check** —
this is the agent's terminal action. **A human marks Done; the agent never self-marks Done.**

### The blocking quality gate (the orchestrator runs it before moving the ticket)
This is the orchestrator's own check, expressed against the `${ARTIFACTS}` files — it calls NO private
external script. The ticket does NOT advance unless ALL hold (any failure → leave the ticket in its
current state, redo the missing pieces, re-run the gate):

1. `${ARTIFACTS}/before.jpg` exists, mtime < 2h.
2. `${ARTIFACTS}/after.jpg` exists, mtime < 2h (OR `after-step-*.png` when a multi-step trigger matched, §3 Step 4.1).
3. `${ARTIFACTS}/preflight-status.txt` contains exactly `200` (§3 Step 1.5).
4. For **each** selected audit `<name>` (baseline `codeaudit`/`logicaudit`/`debugaudit` always present):
   its verdict has `score == 100` AND `skill_used == "<name>audit"` (the real skill, not a custom version).
5. `${ARTIFACTS}/adversarial.json` verdict == `confirmed` (§5 step 4).
6. `${ARTIFACTS}/intent-verification.md` exists and every question is YES (or N/A for Q4) (§5 step 4).
7. The Linear comment body and the saved `linear-comment.md` contain ALL required sections (§6): grep
   for `### Before / After` (or `### After State (stages)`), `### Console Log Comparison`,
   `### Audit results`, `### Verify` with a clickable URL, `### Self-verification checklist` with `[x]`
   items. In REGRESSION mode also `### Why prior attempts failed`.
8. `git log --oneline -n 50` contains a commit matching `fix: [<TICKET_ID>]`.

The gate enforces three criteria — **comment completeness** (check 7), **audit chain = 100/100 with the
real skills + adversarial confirmed** (checks 4–5), and **Intent Verification all-YES** (check 6). If
ANY fails → reject and redo. Never move to review without all three; never mark Done.

### Move to the review state (NEVER Done)
Discover the target state dynamically (NEVER hardcode a `stateId`):
```graphql
{ team(id:"TEAM_UUID"){ states{ nodes{ id name type } } } }
```
Choose, in order:
1. any state named **`In Review`** (type `started`) — the operator's standard review state, else
2. **`Omega Review`** — OmegaOS's neutral review state, auto-created on demand when the team has none
   (color `#10B981`, type `started`), else
3. if neither exists and `Omega Review` cannot be created → ABORT and tell the operator: "no review
   state found; ticket left in its current state."

Then:
```graphql
mutation { issueUpdate(id:"ISSUE_UUID", input:{ stateId:"REVIEW_STATE_UUID" }){ success } }
```

**FORBIDDEN:** moving a ticket to `Done` / `Completed` / `Closed`, or hardcoding any Done `stateId`.
That is an automatic failure. The review state is the boundary: the agent does the fix + the 100/100
gate + the comment; **you (the operator)** do the final manual check and mark Done. Every ticket the
pipeline finishes MUST be in the review state before the mission reports done — tickets left in Todo /
In Progress / Backlog at mission end = a quality-gate violation.

---

## §8 — DONE CONTRACT & REPORTING

A `/omg-linear` mission is **done_clean** only when BOTH:
- (a) every fetched ticket is either moved to the neutral review state (`In Review` / `Omega Review`)
  WITH a complete Fix-Verification comment and a green 100/100 audit set, OR explicitly escalated WITH a
  failure comment (audit < 100 after 5 iterations, adversarial voided, or intent not all-YES) — the
  failure is recorded, never silently dropped (Law L4); AND
- (b) the consolidated cross-ticket audit (§5 step 6) passed 100/100 on the cumulative diff.

Tickets without one of those terminal markers → status **pending**. Report a final checklist:
`X advanced to review, Y reopened (VERIFY-DONE failures), Z escalated`, each with its Linear link. End a
substantial run with a one-line recap (`--- **Resume:** …` in the operator's language, R-STYLE).

**Every claim in the report carries evidence** (R-CITE): a commit hash, an audit verdict path, a
screenshot file, or a Linear comment URL. Uncited assertions are rejected.

---

## §9 — LINEAR API REFERENCE (fetch with full context)

Workers MUST consume the entire returned object — description AND every comment AND every state
transition. The `history` field powers REGRESSION detection; `comments.user.isMe` powers RE-REVIEW.

```graphql
{
  issues(filter: { state: { type: { nin: ["completed", "canceled"] } } }) {   # VERIFY-DONE: nin → in:["completed"]
    nodes {
      id identifier title description priority createdAt updatedAt
      state { id name type }
      labels { nodes { name } }
      comments(orderBy: createdAt) { nodes { id body createdAt user { name displayName email isMe } } }
      history(orderBy: createdAt) {
        nodes { createdAt fromState { id name type } toState { id name type } actor { name displayName email } }
      }
    }
  }
}
```

Post a comment: `mutation { commentCreate(input:{ issueId:"UUID", body:"BODY" }){ success } }`.
Reopen (VERIFY-DONE fail): `mutation { issueUpdate(id:"UUID", input:{ stateId:"IN_PROGRESS_UUID" }){ success } }`
(query `workflowStates` first for the team's In-Progress id). Embed before/after images by uploading the
PNGs via the `fileUpload` mutation and putting `![before](url)` / `![after](url)` in the comment (skip
for pure backend/data tickets with no visual component).

---

## §10 — WHAT THIS PROTOCOL KEEPS FROM THE PROVEN DESIGN

- The **four-mode model** (FIX / VERIFY-DONE / RE-REVIEW / REGRESSION) with per-ticket auto-detection.
- The **strict Fix-Verification comment template** (§6) — verbatim quote, Before/After, console diff,
  banned-pattern list, audit table, verify URL, self-check, and the canonical `linear-comment.md` save.
- **BEFORE/AFTER evidence captures** + auth pre-flight with the HTTP-status verdict map; multi-step
  flows captured stage-by-stage.
- The **100/100 quality gate** + adversarial dual-pass + 5-question Intent Verification, with the
  concrete §7 gate checklist.
- The **trigger-guard** (never mention Linear unless the operator did) and intent-first reading.

What it changes for OmegaOS:
- De-hardcoded — no `~/.claude/...`, no private `audit-selector.py` / `linear-ticket-gate.sh`. All paths
  go through `${OMEGA_DIR}` and the project's own config.
- Audits run through the shipped **`/omg-audit`** Quality Arsenal (`omega audit select` + the real
  `/omg-<name>audit` skills), replacing the old dynamic selector script.
- The **v2 Workflow engine** (§5) fans tickets out in parallel, adversarially verifies, then synthesizes
  a consolidated cross-ticket audit — replacing the old sequential one-ticket-at-a-time loop.
- **Neutral review wording** — `In Review` / `Omega Review` for "the operator", with no person named; a
  human marks Done, the agent never self-marks Done.

---

*"Download. Capture before. Fix. Capture after. Compare. Strict template. Audit to 100. Verify intent.
Quality gate. Move to review — a human marks Done."*
