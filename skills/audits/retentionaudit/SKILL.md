---
name: retentionaudit
description: >
  Forensic retention + opportunity audit v1.1 (Gestalt-Popper, READ-ONLY).
  20-phase parallel deep analysis applying 4 expert frameworks: Hooked (Eyal),
  Jobs-To-Be-Done (Christensen), Power of Moments (Heath), and Fogg B=MAT.
  Inventories user-journey gaps, drop-off forensics, aha-moment latency, hook strength,
  personalization debt, onboarding completeness, empty-states, network effects,
  sales angles, monetization hooks, friction surfaces, reactivation flows, community
  surfaces, discoverability, power-user delight. Outputs RICE-prioritized roadmap with
  Fogg adoption likelihood per top idea.
  Answers: "What would a senior CPO of a $1B SaaS find that we MISSED to satisfy
  users 3x more and drive retention compounding?"
  Score /400. READ-ONLY — proposes, never codes. Hands off to /planner or /implement.
  Use when user says "/retentionaudit", "what would a CPO suggest", "retention audit",
  "feature opportunities", "improve user satisfaction", "make it sticky", "retention hooks",
  "growth opportunities", "what are we missing for retention", "user satisfaction audit",
  "CPO insight", "feature opportunity scan", "retention saas", "audit produit",
  "comment fideliser", "que pourrions-nous oublier", "propose des features".
  Flags: --focus <area> for narrower-but-deep scope (aha | onboarding | monetization |
  reactivation | network | powerUser). Never use --quick (rule 46).
allowed-tools: ["Read", "Bash", "Glob", "Grep", "WebSearch", "WebFetch"]
domain: retention
phases: 20
max_score: 400
read_only: true
triggers: ["retention", "feature opportunities", "make it sticky", "what's missing for retention", "cpo mindset"]
---


<!-- AUDIT-META-V2-INJECTED -->

> ## ⚠️ MANDATORY FIRST STEP — READ THE V2 META-PROTOCOL
>
> **Before doing ANYTHING else**, Read `../_shared/audit-meta-protocol-v2.md`.
>
> That file overrides any conflicting guidance below for these aspects:
> 1. Required CLI inputs (`--user-need`, `--hinge` are MANDATORY since 2026-05-08)
> 2. Required JSON output schema (v2: score + confidence + falsifiable_tests + user_need_match + hinge_findings)
> 3. Popper falsification — every PASS must cite ≥3 concrete commands run with actual output
> 4. Confidence calibration — `high` requires direct verification of every claim
> 5. Banned shortcut phrases — `looks correct`, `should be fine`, `appears to work` = automatic FAIL

---

# /retentionaudit v1.1 — Forensic Retention & Opportunity Discovery (CPO Mindset)

> *"Code/debug/logic ask 'is what exists correct?' Featureaudit asks 'what's missing for completeness?'  /retentionaudit asks 'what would the CPO of a $1B SaaS see that we're blind to — that would make users stay 3x longer?'"*

---

## CONTRACT — READ-ONLY ANALYST

**This skill DOES NOT CODE.** It proposes, ranks, and explains. Implementation is a separate
mission you (the user) explicitly authorize via `/implement retention proposal N°X` or
direct dispatch to MORPHEUS.

**Hard forbidden:**
- Edit / Write any source file (only the report files in `audits/.retentionaudit/reports/`)
- Modify schemas, configs, CSS, copy, or markup
- Push to git, deploy, run migrations
- Add dependencies or run package installers

**The skill ONLY produces:**
- `audits/.retentionaudit/reports/<date>-retention-audit.md` — full forensic report
- `audits/.retentionaudit/reports/<date>-prioritized-roadmap.md` — RICE-scored ideas
- `audits/.retentionaudit/output.json` — machine-readable v2 schema

---

## DOCTRINE

You are a **product prosecutor with a $50M ARR retention obsession**. The product is on
trial for failing to make users stay, succeed, and tell their friends. Every drop-off
is unfinished business. Every empty state is a missed opportunity. Every "set-and-forget"
moment without a retention hook is revenue leakage.

### The 7 Laws of Retention Forensics

1. **Aha-moment is sacred.** The product has ONE moment where the user goes "oh, *now* I get it." Find it. Audit how many users actually reach it. Audit what could prevent them. If the aha-moment is buried at step 7, that's a CONVICTION.

2. **Habits compound, features don't.** A feature used once is forgotten. A trigger → action → variable reward → investment loop creates compulsion. Apply the **Hooked framework** (Eyal): every retention-driving feature must have all 4 elements OR explicitly be a one-shot transactional feature.

3. **Friction is theft.** Every extra click between intent and outcome steals retention. Audit user flows for the **shortest possible path**. If the same user-goal takes 5 clicks here and 2 clicks at the competitor, that's competitive disadvantage.

4. **Empty states are interviews.** When a feature has no data (just-installed, just-onboarded, just-cleaned-out), the empty state is your one shot to teach + invite + commit. Audit every empty state. A blank rectangle is malpractice.

5. **Personalization is retention currency.** Generic feed = anyone could leave. Personalized feed = "this works for me, I can't leave." Audit how much of the experience adapts to the user vs being identical for all.

6. **Sales doesn't stop after signup.** The CPO mindset: every screen is a sales pitch — for upgrade, for invitation, for daily return. Audit screens for missed value-prop reminders, social proof, achievement signals, network effects.

7. **Falsify the retention claim (Popper).** The product implicitly claims "users will stick." DISPROVE this. Find every reason a smart user would churn after week 1 / month 1 / month 3. Score severity. Sort by impact.

### Gestalt Hinge Capability

Before listing opportunities, identify the **HINGE EXPERIENCE** — the ONE thing that
must be world-class for users to stay. For Notion: fast multiplayer block editing.
For Loom: sub-3-second record-to-share. For Linear: keyboard-driven issue ops.

If the hinge experience has friction, NO retention hook will save the product. Fix the
hinge first, every other proposal second.

---

## FRAMEWORKS — Beyond Hooked

`/retentionaudit` v1 leaned on Hooked + RICE. v1.1 stacks **four expert lenses** because each one catches gaps the others miss. Apply ALL four — the union of findings is the audit.

### Lens 1 — Hooked (Eyal): daily habit formation
**Trigger → Action → Variable Reward → Investment.** Catches: feature-not-becoming-habit. Best for daily-use products (Notion, Slack, Linear). See Phase 8.

### Lens 2 — Jobs-To-Be-Done (Christensen): the "hire" question
For each persona, surface 3-5 jobs in this format:
> "When [situation], I want to [job], so I can [outcome]"

Example for a SaaS dashboard:
> "When I open the app on Monday morning, I want to see what changed over the weekend, so I can prioritize my day in 60 seconds."

For each job: does the product serve it well? Or does the user hire a competitor (or a workaround) for that job? Catches: features that exist but don't match real usage contexts. See Phase 7 + Phase 9.

### Lens 3 — Power of Moments (Heath brothers): emotional retention
Users remember **peaks** and **transitions**, not the average. Audit four moment types per user journey:
- **Peaks**: memorable elevations (delight, victory, "wow") — amplify them
- **Pits**: frustrating lows (errors, dead ends, confusion) — fix or remove them
- **Transitions**: key journey points (signup, first action, milestone) — mark them with ceremony
- **Plateaus**: long boring middles (data-loading, repetitive flows) — interrupt them with surprise

Catches: products that are "fine" but never memorable → users churn for the next shiny thing. See Phase 10 + Phase 13.

### Lens 4 — Fogg B=MAT: adoption likelihood
**Behavior = Motivation × Ability × Trigger.** For each proposed feature in Phase 20:
- **M** (0-3): how strongly does the user *want* to do this action?
- **A** (0-3): how *easy* is it to actually do (clicks, cognition, prior steps)?
- **T** (0-3): is there a *clear trigger* in the moment of intent?

Score per idea: M × A × T (0-27). High RICE + low Fogg = idea sounds good but won't be adopted. Both must be high.

### How the four interact

| Lens | Time horizon | Catches | Used in |
|---|---|---|---|
| Hooked | Daily / weekly | Habits not forming | Phase 8 |
| JTBD | Per-task | Wrong job served | Phase 7, 9 |
| Heath | Episodic | Emotionally flat | Phase 10, 13 |
| Fogg | Per-decision | Idea won't be adopted | Phase 20 |

A complete audit applies all four. Skipping a lens leaves a class of retention bug undetected.

---

## SCOPE DETECTION

```
EXAMPLES:
  "/retentionaudit"
  → Full 20-phase pipeline on the entire product

  "/retentionaudit the dashboard"
  → TARGETED: only dashboard + adjacent flows

  "/retentionaudit onboarding"
  → ONBOARDING DEEP: the first-7-days experience

  "/retentionaudit on Causio"
  → SCOPED to specific project

  "/retentionaudit churn week 1"
  → CHURN ZONE: focus on the W1 drop-off cliff

  "/retentionaudit power user delight"
  → POWER USER mode: edge cases, keyboard shortcuts, advanced flows

  "/retentionaudit free → paid conversion"
  → MONETIZATION: the upgrade triggers

  "/retentionaudit --focus aha"
  → FOCUS MODE: full depth on aha-moment, hooked loops, onboarding only
  Valid --focus values: aha | onboarding | monetization | reactivation | network | powerUser
  Note: --focus narrows scope but KEEPS full depth (rule 46 — never use --quick).
```

---

## CANONICAL RUNNER GATE

Before Phase 0, invoke `~/.omega/lib/audit-runner.sh retention "<absolute-project-path>" --files="<scoped-files>" --user-need="<verbatim-user-need>" --hinge="<load-bearing-region>" --no-fix` (plus `--ticket` and `--url` together when ticket-scoped). A non-zero exit is an audit failure. Read the emitted `audits/.retentionaudit/evidence-summary.json` before analysis, then rerun the same invocation with `--finalize` after writing `verdict.json`. This gate does not change the audit's read-only contract.

## OUTPUT CONTRACT

```
audits/.retentionaudit/
├── session.log
├── discovery/
│   ├── user-journey.md          # Phase 2: every screen + transition
│   ├── drop-off-zones.md        # Phase 3: where users likely leave
│   ├── empty-states.md          # Phase 4: every empty state inventoried
│   ├── retention-hooks.md       # Phase 5: existing + missing
│   └── competitor-baseline.md   # Phase 6: market comparison
├── reports/
│   ├── 01-hinge-capability.md   # Phase 1
│   ├── 02-aha-moment.md         # Phase 7
│   ├── 03-hooked-loops.md       # Phase 8
│   ├── 04-personalization.md    # Phase 9
│   ├── 05-onboarding.md         # Phase 10
│   ├── 06-empty-states.md       # Phase 11
│   ├── 07-network-effects.md    # Phase 12
│   ├── 08-sales-angles.md       # Phase 13
│   ├── 09-monetization.md       # Phase 14
│   ├── 10-frictionless.md       # Phase 15
│   ├── 11-reactivation.md       # Phase 16
│   ├── 12-community.md          # Phase 17
│   ├── 13-discoverability.md    # Phase 18
│   ├── 14-power-user.md         # Phase 19
│   ├── 15-prioritized-roadmap.md # Phase 20 — final RICE-ranked
│   └── final-report.md          # Aggregate + verdict
└── output.json                  # v2 schema
```

---

## WAVE EXECUTION (parallel grouping)

The 20 phases run in 5 waves. Phases inside a wave are **independent and run in parallel**; waves run sequentially because each consumes outputs from the previous.

| Wave | Phases | Mode | Why |
|---|---|---|---|
| **1 — Foundation** | 1, 2 | Sequential | Hinge first, then journey (drop-offs need the journey) |
| **2 — Evidence** | 3, 4, 5 | Parallel | Drop-offs / empty states / existing hooks — no shared state |
| **3 — Outside-in** | 6, 7, 8, 9 | Parallel | Competitor / aha / hooked / personalization — independent lenses |
| **4 — Proposal generation** | 10–19 | Parallel | All proposal-mining phases — no inter-dependencies |
| **5 — Synthesis** | 20 | Sequential | RICE prioritization needs everything before it |

When dispatching this audit via Oracle, the worker prompt should mention: "Run waves 2/3/4 with parallel agent calls (one Agent per phase), then synthesize in wave 5."

---

## EVERY PHASE FALSIFIES (Popper, mandatory)

Per audit-meta-protocol-v2.md, every phase MUST cite ≥3 concrete commands that **could have failed but didn't**. Banned phrases (auto-FAIL): `looks correct`, `should be fine`, `appears to work`, `we probably`, `users likely`, `competitors all`.

For every claim in every phase, you MUST:
1. State the hypothesis ("X drives drop-off at step 4")
2. Name the falsification command ("grep for blocking modal at step-4 component")
3. Run it and paste the output
4. State whether the hypothesis survived or died

If the command was not run, the claim is unsubstantiated and scores 0 for that finding.

---

## 20 PHASES — Forensic Retention Audit

### Phase 1 — Hinge Capability Identification (Gestalt — 50% effort lives here)

**Why this phase gets disproportionate weight:** Per Quality DNA, every skill has a hinge moment. For a retention audit, the hinge is the answer to "what is THE ONE experience that, if it works flawlessly, will keep users coming back regardless of every other gap?" If the hinge experience has friction, NO downstream proposal will save the product. Fix the hinge first.

#### 1.A — Stated hinge (the promise)

Read in this order, capturing direct quotes:

```bash
# Product identity sources
test -f VISION.md && cat VISION.md | head -200
test -f docs/PRD.md && cat docs/PRD.md | head -200
test -f README.md && head -80 README.md
test -f public/index.html && grep -E "<title>|description|og:" public/index.html
# Landing page hero (if web)
find src/app -path "*page.tsx" -maxdepth 4 2>/dev/null | head -5 | xargs grep -l "hero\|landing" 2>/dev/null | head -1 | xargs head -100 2>/dev/null
```

Extract: **the stated hinge sentence** (verbatim quote from copy).

#### 1.B — Observed hinge (what the code prioritizes)

The codebase reveals the *actual* hinge by what gets the most engineering attention. Survey:

```bash
# Lines of code per feature directory (proxy for engineering investment)
find src -type d -maxdepth 3 2>/dev/null | while read d; do
  loc=$(find "$d" -maxdepth 1 -type f \( -name "*.ts" -o -name "*.tsx" \) 2>/dev/null | xargs wc -l 2>/dev/null | tail -1 | awk '{print $1}')
  [[ -n "$loc" && "$loc" != "0" ]] && echo "$loc $d"
done | sort -rn | head -15

# Test coverage concentration (the most-tested feature is what the team cares about)
find . -type f \( -name "*.test.ts" -o -name "*.spec.ts" \) -not -path "*/node_modules/*" 2>/dev/null | xargs grep -l "describe(" 2>/dev/null | head -20

# Recent commit activity per directory (heat map of attention)
git log --since="30 days ago" --name-only --pretty=format: 2>/dev/null | grep -v "^$" | awk -F/ '{print $1"/"$2}' | sort | uniq -c | sort -rn | head -10
```

Extract: **the observed hinge** (the area where the team actually invests).

#### 1.C — Stated vs Observed gap

Compare 1.A and 1.B. **The most damaging retention bug is a divergence.**
- "We are a fast issue tracker" (stated) vs 60% of code in billing module (observed) → MISALIGNED
- "We help solo devs ship" (stated) vs collab features dominate (observed) → MISALIGNED
- "We are the AI assistant for X" (stated) vs AI is 5% of code (observed) → STATED IS LIE

If misaligned: this is the **single most important finding in the entire audit**. Document it loudly.

#### 1.D — Hinge benchmark (quantified friction)

For the stated hinge, measure the actual user cost to achieve it:

```bash
# Click count: count interactive elements between landing and hinge outcome
# (manual trace via Read of route files is fine — count Link/button/form elements crossed)

# Time-to-hinge: if dev server runnable, time the flow
# (otherwise estimate from network tab + render measurements)

# Page weight at hinge route
find src -name "*.tsx" -path "*hinge_route*" | xargs wc -l
```

Record three numbers:
- **Clicks to hinge** (target: ≤3)
- **Seconds to hinge** (target: ≤5s after auth)
- **Required prior steps** (target: 0 — hinge should work for guest if possible)

#### 1.E — Competitor hinge comparison (WebSearch)

Identify top 3 competitors. For each, find the hinge experience and measure the same 3 numbers (from public demos, screencasts, comparison reviews, Reforge case studies).

Build a 3-row table: **us vs competitor1 vs competitor2 vs competitor3**.

#### 1.F — Falsification (Popper, mandatory)

Run these commands. Each must produce output:

1. `find src -name "page.tsx" -newer .git/refs/heads/main 2>/dev/null | head` — recent activity at hinge route
2. `git log --oneline --since="60 days ago" -- <hinge_path> | wc -l` — commits to hinge area
3. Trace one user goal (e.g., "create first issue") through routing files; count actual handlers crossed.

If any of the 3 commands returns 0 lines or contradicts the stated hinge → **VERDICT: BROKEN**.

#### 1.G — Verdict logic

| Condition | Verdict |
|---|---|
| Stated = Observed AND clicks ≤3 AND on-par with competitors | **STRONG** |
| Stated = Observed AND (clicks 4-6 OR ≤1 competitor better) | **WEAK** |
| Stated ≠ Observed (misalignment) | **BROKEN** |
| Clicks >6 OR ≥2 competitors better OR hinge buried in submenu | **BROKEN** |

#### 1.H — Output

Write `audits/.retentionaudit/reports/01-hinge-capability.md` containing:
- Stated hinge sentence (verbatim quote, citation)
- Observed hinge area (with LOC table)
- Stated-vs-Observed gap analysis
- Quantified friction (3 numbers)
- Competitor table (3-4 rows)
- Falsification commands run with output
- Verdict + 3-sentence rationale
- The ONE proposal to fix the hinge (P0, before any other proposal in the audit)

**This phase MUST take 30-45 minutes of work. If completed in <15 minutes, it is shallow and the verdict is unreliable.**

### Phase 2 — User Journey Mapping

Trace every screen the user can reach from sign-up to power-user. Map transitions.
Tools: read routing/router files, sitemap, navigation components.

**Output:** mermaid diagram + table of every screen + entry/exit conditions

### Phase 3 — Drop-Off Zone Forensics

For each screen, identify the LIKELY drop-off triggers:
- Form fields with no validation (user gives up)
- Loading states without progress (user thinks broken)
- Empty states without action (user doesn't know what to do)
- Modal pop-ups blocking flow
- Auth walls before value
- Pricing walls before perceived value

**Falsification:** for each suspected drop-off, run `grep` to verify the friction exists.
"Probably broken" without evidence = AUTOMATIC FAIL.

### Phase 4 — Empty State Audit

Find EVERY component that renders when data is empty:
- Dashboards with no projects
- Feeds with no items
- Search with no results
- Lists with 0 entries
- New-user pages

For each: does it (a) explain what should appear, (b) provide an action to make it appear, (c) provide a sample/seed/example?

**Empty state = teaching + inviting + committing.** A blank rectangle is malpractice.

### Phase 5 — Existing Retention Hooks Inventory

Scan for already-existing retention mechanisms:
- Email/notification triggers (Resend, Sendgrid, OneSignal, Postmark)
- In-app notifications
- Streaks / achievements / milestones
- Daily / weekly reports / digests
- Social/share triggers
- Re-engagement campaigns

Don't propose what already exists. Inventory first, then find gaps.

### Phase 6 — Competitor Baseline (WebSearch)

Identify 3-5 competitors. For each, note their top retention mechanisms (publicly visible
on their landing/blog/help docs). Build comparison table.

**Falsification:** if competitor has feature X, document the URL/screenshot evidence.
"Competitors all do X" without evidence = AUTOMATIC FAIL.

### Phase 7 — Aha-Moment Detection

What is the moment a new user goes "oh, NOW I get it"?
- For Slack: first message reply within 5 minutes
- For Dropbox: file synced across 2 devices
- For Linear: issue created in <10 seconds via Cmd-K

Identify the aha-moment for THIS product. Then audit:
- How many steps from signup to aha?
- What % of users likely reach it (estimate from analytics if available)?
- What's the BLOCKING factor for the rest?

### Phase 8 — Hooked Loops Analysis (Eyal Framework)

For each retention-driving feature, check the 4 elements:
1. **Trigger** — external (notification) or internal (boredom, anxiety)
2. **Action** — the simplest behavior anticipating reward
3. **Variable reward** — uncertainty drives compulsion
4. **Investment** — user puts something in (data, time, social) → loads next trigger

Score each feature: 4/4 = strong loop, 3/4 = degraded, ≤2/4 = no loop.

### Phase 9 — Personalization Gap Audit

For each screen/feed/list:
- Is the order user-specific or global?
- Are recommendations based on the user's history?
- Are defaults adapted (timezone, language, recently-used)?
- Is there a "for you" surface separate from "all"?

Score: 0% (identical for all) → 100% (every element personalized).

### Phase 10 — Onboarding Completeness

The first 7 days determine LTV. Audit:
- Welcome modal / tour / checklist?
- First-task guidance?
- Empty-state with a "do X to see Y" CTA?
- Day-1, day-3, day-7 nudges?
- Activation criteria measurable?

Compare to **Reforge / Mode benchmarks** for the product category.

### Phase 11 — Empty State Detail (continuation of Phase 4)

For the top 5 empty states, propose specific copy + CTA + sample data.

### Phase 12 — Network Effects Opportunities

- Can a single user invite a teammate / friend with one click?
- Does the product become MORE valuable when N users join?
- Are there public artifacts (shareable links, embeds, widgets) that bring new users?
- Is there UGC (user-generated content) that compounds?

### Phase 13 — Sales Angles On Every Screen

Every screen is a sales surface. Audit:
- Value prop reminder (1 line, every page)?
- Social proof (logos, counts, testimonials, recent activity)?
- Upgrade prompt (contextual, not nag)?
- Achievement signal (you used X, you're at Y%)?
- Comparison ("you saved X hours this week")?

### Phase 14 — Monetization Hook Audit

If freemium / trial:
- Is the value-gate at the right point (after aha, before commitment)?
- Is the upgrade trigger contextual (limit reached) vs nag (banner)?
- Is the price anchor visible early?
- Is there a clear "team plan" upgrade path?

### Phase 15 — Frictionless Improvements

For the top 5 user goals, count clicks/keystrokes/seconds. Propose specific 1-2 click
reductions:
- Cmd-K for everything
- Inline edit vs modal
- Optimistic updates
- Auto-save (vs Save button)
- Smart defaults (most-used fields prefilled)

### Phase 16 — Reactivation / Win-Back

For users who haven't returned in N days:
- Email cadence (D3, D7, D14, D30, D90)?
- "What you missed" digest?
- Re-engagement coupon / feature unlock?
- "Hey, your team did X without you" social pressure?

### Phase 17 — Community / Social Features

- Public profiles?
- Share-to-X buttons?
- Embeddable widgets?
- Comments / reactions on artifacts?
- Templates / showcases?
- Discord / Slack community link visible?

### Phase 18 — Discoverability Of Existing Features

Are powerful features hidden? Audit:
- Cmd-K palette completeness
- Settings page organization
- "What's new" changelog visibility
- Help center quality
- Empty-state hints to features

### Phase 19 — Power User Delight

For the top 1% of users (heavy use), propose:
- Keyboard shortcuts for top-10 actions
- Bulk operations
- API/CLI/integrations
- Export/backup options
- Advanced filters/views

### Phase 20 — Prioritized Roadmap (RICE × Fogg adoption)

Synthesize ALL ideas from phases 7-19 into a single prioritized list. Two-pass scoring:

#### Pass 1 — RICE (impact / effort ratio)

For each idea, compute RICE:
- **Reach**: % of users this affects (0-100)
- **Impact**: 0.25 (minimal), 0.5 (low), 1 (medium), 2 (high), 3 (massive)
- **Confidence**: 0-100% how sure you are
- **Effort**: person-days to implement

`RICE = (Reach × Impact × Confidence) / Effort`

#### Pass 2 — Fogg B=MAT (adoption likelihood)

For the top-15 RICE ideas, also compute Fogg:
- **M** (Motivation, 0-3): how strongly does the user *want* to do this action in the moment?
- **A** (Ability, 0-3): how *easy* is it (clicks, cognition, prior steps)?
- **T** (Trigger, 0-3): is there a *clear trigger* present at the moment of intent?

`Fogg = M × A × T` (range 0-27)

#### Final ranking

For each idea, compute `priority_score = RICE_normalized × (1 + Fogg/27)` — Fogg acts as a multiplier so a great idea with 0 trigger drops down. **High RICE + low Fogg = sounds good but won't be adopted; flag and propose a trigger fix first.**

Sort top 20 by `priority_score`. For each, document:
- Idea (1 sentence)
- Why it drives retention (2-3 sentences referencing the relevant lens — Hooked / JTBD / Heath / Fogg)
- JTBD statement: "When [situation], I want to [job], so I can [outcome]" (verbatim from persona research where available)
- Heath moment type if applicable (Peak / Pit-fix / Transition / Plateau-break)
- RICE breakdown (R, I, C, E) + RICE score
- Fogg breakdown (M, A, T) + Fogg score
- Implementation outline (high level, NOT code)
- Success metric (what to measure, baseline + target)
- Effort estimate (small <2d / medium 2-7d / large >7d)
- Pre-flight test (how would we falsify this won't work, BEFORE building)

---

## OUTPUT JSON SCHEMA (v2)

Write to `audits/.retentionaudit/output.json`:

```json
{
  "audit": "retention",
  "version": "1.0",
  "schema_version": "v2",
  "round": 1,
  "score": 0,
  "raw_score": "0/400",
  "score_normalized": 0,
  "confidence": "low|medium|high",
  "skill_used": "retentionaudit",
  "ticket": "<scope>",

  "user_need_match": {
    "quote": "<verbatim user-need from --user-need flag>",
    "addressed": true,
    "evidence": "<2-3 paragraphs grounding answer in code+screens>",
    "edge_cases_covered": []
  },

  "hinge_findings": [{
    "location": "<file:line OR url>",
    "concern": "<hinge experience friction>",
    "evidence": "<commands run + outputs>",
    "verdict": "STRONG | WEAK | BROKEN"
  }],

  "drop_off_zones": [{"screen": "...", "trigger": "...", "severity": "P0|P1|P2|P3"}],

  "empty_states_audit": [{"component": "...", "current": "blank|stub|good", "proposal": "..."}],

  "hooked_loops": [{"feature": "...", "trigger": "...", "action": "...", "reward": "...", "investment": "...", "score": "0-4"}],

  "competitor_gaps": [{"competitor": "...", "feature": "...", "url": "...", "we_have": false}],

  "prioritized_ideas": [{
    "rank": 1,
    "name": "<2-5 word idea name>",
    "why_retention": "<which retention mechanism — reference Hooked element / JTBD / Heath moment / network effect>",
    "lens_used": "hooked | jtbd | heath | fogg",
    "jtbd_statement": "When <situation>, I want to <job>, so I can <outcome>",
    "heath_moment_type": "peak | pit_fix | transition | plateau_break | none",
    "user_need_quote": "<persona quote OR verbatim if available>",
    "rice": {"reach": 60, "impact": 2, "confidence": 80, "effort_days": 5, "score": 19.2},
    "fogg": {"motivation": 2, "ability": 3, "trigger": 2, "score": 12},
    "priority_score": 27.7,
    "implementation_outline": "<high-level, NOT code>",
    "success_metric": "<measurable, with baseline + target>",
    "preflight_test": "<how would we falsify this won't work BEFORE building>",
    "anti_pattern": false,
    "anti_pattern_reason": "<if anti_pattern=true, why>",
    "category": "onboarding|aha|loop|personalization|network|sales|monetization|frictionless|reactivation|community|discoverability|powerUser"
  }],

  "falsifiable_tests": [
    {"name": "...", "hypothesis": "...", "command": "...", "expected": "...", "actual": "...", "passed": true|false}
  ],

  "score_breakdown": {
    "phase_1_hinge": 0,
    "phase_2_journey": 0,
    "phase_3_dropoff": 0,
    "phase_4_empty": 0,
    "phase_5_existing": 0,
    "phase_6_competitor": 0,
    "phase_7_aha": 0,
    "phase_8_hooked": 0,
    "phase_9_personalization": 0,
    "phase_10_onboarding": 0,
    "phase_11_empty_detail": 0,
    "phase_12_network": 0,
    "phase_13_sales": 0,
    "phase_14_monetization": 0,
    "phase_15_friction": 0,
    "phase_16_reactivation": 0,
    "phase_17_community": 0,
    "phase_18_discoverability": 0,
    "phase_19_power": 0,
    "phase_20_roadmap": 0
  }
}
```

Score per phase: 0-20. Total /400. Normalize to /100.

---

## SCORING RUBRIC (per phase, 0-20)

- **0-5**: phase skipped or shallow inventory (no evidence)
- **6-10**: phase ran but found <3 actionable insights
- **11-15**: phase ran, 3-7 insights, with evidence + RICE
- **16-19**: phase exhaustive, 8+ insights, falsifiable tests included
- **20**: phase exhaustive, prioritized, with persona quotes + competitor evidence

---

## QUALITY GATES (procedural — run BEFORE reporting done)

These gates are not declarative ("if X, fail"). They are a **pre-flight checklist you run as the last step of the audit**, before writing the final report.

### Pre-flight (run before phase 20 synthesis)

```bash
# Gate 1: NO IMPLEMENTATION (verify no source files were edited)
git status --porcelain | grep -v "^.. \audits/.retentionaudit/" | grep -v "^.. retentionaudit-" | head
# MUST output 0 lines. If any line, the audit violated read-only contract → ABORT.

# Gate 2: EVERY DROP-OFF HAS FILE:LINE
jq '.drop_off_zones[] | select(.evidence == null or .evidence == "")' audits/.retentionaudit/output.json
# MUST output empty. Each drop_off_zone must cite a grep result with file:line.

# Gate 3: EVERY COMPETITOR CLAIM HAS URL
jq '.competitor_gaps[] | select(.url == null or (.url | startswith("http") | not))' audits/.retentionaudit/output.json
# MUST output empty.

# Gate 4: EVERY IDEA HAS RICE
jq '.prioritized_ideas[] | select(.rice == null or .rice.score == null)' audits/.retentionaudit/output.json
# MUST output empty.

# Gate 5: TOP-15 IDEAS HAVE FOGG (B=MAT)
jq '.prioritized_ideas[0:15][] | select(.fogg == null)' audits/.retentionaudit/output.json
# MUST output empty.

# Gate 6: HINGE PHASE HAS 3+ FALSIFICATION COMMANDS
jq '.falsifiable_tests | map(select(.name | contains("hinge") or contains("Phase 1"))) | length' audits/.retentionaudit/output.json
# MUST output >= 3.

# Gate 7: NO BANNED SHORTCUT PHRASES
grep -rEn "looks correct|should be fine|appears to work|we probably|competitors all|users likely" audits/.retentionaudit/reports/
# MUST output 0 lines.
```

If any gate fails → **fix and re-run the gate**. The audit is not done until all 7 gates are green.

### Post-flight (after writing final report)

- [ ] `audits/.retentionaudit/output.json` parses as valid JSON (`jq '.' audits/.retentionaudit/output.json >/dev/null`)
- [ ] `final-report.md` summary references the hinge verdict (STRONG/WEAK/BROKEN)
- [ ] Top 5 ideas in roadmap each have a JTBD statement
- [ ] Score normalized to /100 (`raw_score / 400 * 100`)
- [ ] Confidence is `high` only if every claim cites a falsification command

---

## ANTI-PATTERNS (catch these — they signal a shallow audit)

| Anti-pattern | Why it's wrong | What to do instead |
|---|---|---|
| **Vanity hooks** ("add a daily login streak") | Streaks for non-habit products = manipulation, not retention | Only propose streaks if the core action is naturally daily (Duolingo yes, accounting software no) |
| **Dark-pattern reactivation** ("FOMO email: 'your team did X without you'") | Wins back the click, loses trust long-term | Propose value-first reactivation ("here's what changed and why it matters to you") |
| **Engagement ≠ retention** ("DAU is up 30% from notifications") | Notification-driven DAU is a sugar high; M3 retention is what matters | Score every engagement proposal against M3 retention impact, not DAU |
| **Shallow personalization** ("greet user by name") | First-name greeting is 1990s personalization | Propose adaptive defaults, recommendation surfaces, behavior-based feeds |
| **Empty-state CTAs to nowhere** ("Click 'Get Started' which opens an empty modal") | Worse than no CTA — destroys trust | Every empty-state CTA must lead to *value in one step*, never a setup wizard |
| **Onboarding that teaches the UI** ("Click here to see the menu, this is the dashboard") | Teaches UI = teaches there's nothing valuable yet | Onboarding teaches by **delivering** the first valuable outcome |
| **"Premium" that isn't premium** (free tier with feature flags hiding paid features) | Triggers pricing resentment | Premium is *more*, not *unlocked* |
| **Generic empty states** (single illustration + "No items yet") | A blank rectangle with art | Empty state = teach + invite + commit (3 elements minimum) |
| **Adding features without removing any** | Bloat is a retention killer (Sherlock principle) | For every proposed feature, check: is there a feature this would replace or simplify? |
| **Copying a competitor's hook without their context** ("Slack does threads, we should") | Threads only work because of channels + DM volume | Every imported pattern needs a "does our context match?" check |

When an idea matches an anti-pattern, mark it `anti_pattern: true` in the JSON and either fix or remove it.

---

## ECOSYSTEM INTEGRATION

### Run BEFORE /retentionaudit
- `/vision` — Without product identity, the hinge phase has nothing to measure against. If VISION.md is missing, run `/vision` first.
- `/featureaudit` — Establishes the completeness baseline. /retentionaudit assumes the *promised* features exist; it focuses on *unpromised* opportunities.
- `/flowaudit` — User journeys are inputs to Phase 2. If flows are unmapped, run `/flowaudit` first to seed `audits/.flowaudit/journeys.json`.

### Run AFTER /retentionaudit
- `/planner` — Turn the top 10-15 RICE × Fogg ideas into a sequenced implementation plan with milestones.
- `/implement <proposal-N>` — Authorize one specific proposal for build (the user's explicit go-ahead is required — this skill never auto-implements).
- `/uiuxaudit` — After implementing a proposal, verify the proposed UX is well-designed before shipping.
- `/copyaudit` — Verify proposed empty-state copy, microcopy, CTA wording match brand voice.

### Cross-Command Bridge

```
THE CPO-MINDSET LOOP (every quarter, ideally):
  /featureaudit → "What's MISSING from the PRD?"          (completeness)
  /retentionaudit → "What's MISSING for users to stay?"   (CPO opportunity)
  /planner → "How do we sequence the top ideas?"          (roadmap)
  /implement → "Build proposal N"                         (delivery)
  /retentionaudit (verify) → "Did proposal N move retention?" (after 4-6 weeks)

  Together: featureaudit asks 'is the PRD delivered?'
            retentionaudit asks 'what would the CPO of a $1B product see we missed?'
            One audits past commitments. The other discovers future commitments.
```

### Lessons handoff (SMITH integration)

After every run, append the lessons to the OmegaOS state dir (install-parity: this
ships with the install tree, never under `~/.claude/`). Resolve the path and ensure
the dir exists first:

```bash
# Resolve OMEGA_DIR: $OMEGA_DIR env > config.toml omega_dir key > default ~/.omega
OMEGA_DIR="${OMEGA_DIR:-$(awk -F'=' '/^[[:space:]]*omega_dir[[:space:]]*=/{gsub(/[" ]/,"",$2);print $2}' "$HOME/.omega/config.toml" 2>/dev/null)}"
OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
mkdir -p "$OMEGA_DIR/state"
LESSONS="$OMEGA_DIR/state/smith-lessons-retention.md"   # create on first append
```

Then append a block to `$LESSONS`:

```markdown
## <date> — <project> retention audit lessons
- **Hinge verdict:** STRONG/WEAK/BROKEN
- **Top recurring anti-pattern observed:** <e.g. "shallow personalization across 4/5 audited screens">
- **Top RICE × Fogg idea:** <name> (score: RICE=X, Fogg=Y)
- **What surprised me:** <one insight that wasn't in the playbook>
- **Pattern for the next audit:** <heuristic to apply next time>
```

SMITH reads these and proposes new heuristics, new anti-patterns, and new framework lenses for v1.2+.

---

## RUNTIME COMMANDS — actually run these (Popper)

```bash
# Phase 2: route extraction (Next.js / Remix / etc.)
find . -path ./node_modules -prune -o -type f \( -name "page.tsx" -o -name "route.ts" \) -print | head -50

# Phase 3-4: empty-state grep
grep -rn "isEmpty\|hasNoData\|emptyState\|EmptyState\|no items\|aucun\|nothing yet" src/ --include="*.tsx" --include="*.jsx" | head

# Phase 5: existing notification triggers
grep -rEn "resend|sendgrid|onesignal|postmark|trigger\.dev|sendNotification" src/ | head -10

# Phase 8: hooked-pattern keywords
grep -rEn "streak|achievement|milestone|reward|badge|level" src/ | head

# Phase 12: invite/share
grep -rEn "invite|share|inviteLink|shareUrl|copyLink" src/ | head

# Phase 15: keyboard shortcuts
grep -rEn "useHotkeys|cmdK|cmd\+k|ctrl\+|kbd" src/ | head
```

If any returns 0 lines for a project that should have it, that's a P0 GAP.

---

## Dynamic-Workflow Orchestration (v2)

> *This section governs HOW the 20 phases run. It changes the execution engine, not the audit: every phase, scoring weight (0-20/phase, /400 total), the RICE × Fogg matrix, the 4 lenses (Hooked / JTBD / Heath / Fogg), the 7 Laws, the HINGE EXPERIENCE gate, and the READ-ONLY contract are unchanged. Discovery is still observation-only — fan-out NEVER grants write access.*

The legacy "WAVE EXECUTION" table already buckets phases into 5 sequential waves with parallel phases inside each. This section upgrades that from *manual* parallelism ("the worker prompt should mention…") to a **first-class fan-out via the Workflow tool**, adds **adversarial per-finding verification**, and adds **loop-until-dry discovery** for unbounded surfaces (empty states, drop-offs, screens). When `/retentionaudit` runs, the orchestrator MUST drive it as a dynamic workflow, not a linear sweep.

### 1. Decompose into independent parallel tracks (fan-out)

Map the existing waves onto concurrent Workflow branches. Each branch is a self-contained Agent call with a fresh context, its own scratch directory under `audits/.retentionaudit/discovery/`, and a strict READ-ONLY toolset (`Read`, `Grep`, `Glob`, `Bash` read-only, `WebSearch`, `WebFetch` — never `Edit`/`Write` to source). Branches inside a wave share NO mutable state, so they run truly concurrently; waves remain a sequential barrier because each consumes the prior wave's artifacts.

| Workflow stage | Phases | Fan-out (concurrent branches) | Barrier input | Barrier output |
|---|---|---|---|---|
| **Stage A — Foundation** | 1, 2 | 2 branches: (1) HINGE EXPERIENCE forensic — gets 50% effort, runs alone with 10× scrutiny per the Gestalt gate; (2) user-journey map | product identity sources | `01-hinge-capability.md`, `user-journey.md` |
| **Stage B — Evidence** | 3, 4, 5 | 3 branches: drop-off forensics ‖ empty-state inventory ‖ existing-hook inventory | journey map (Stage A) | drop-off / empty-state / hooks inventories |
| **Stage C — Outside-in** | 6, 7, 8, 9 | 4 branches: competitor baseline (WebSearch) ‖ aha-moment ‖ Hooked-loop scoring ‖ personalization gap | journey + hooks inventory | one report each |
| **Stage D — Proposal mining** | 10–19 | up to 10 branches, one per proposal-mining phase (onboarding, empty-state detail, network, sales, monetization, friction, reactivation, community, discoverability, power-user) — fully file-disjoint, no inter-deps | all Stage A–C artifacts | raw idea pools 02–14 |
| **Stage E — Synthesis** | 20 | sequential (you, the orchestrator) | every surviving finding | `15-prioritized-roadmap.md` + `output.json` |

Dispatch each branch in a stage in a **single fan-out** (all Agent calls for the stage issued together), wait for the barrier, then proceed. Cross-branch contamination (a personalization branch editing the onboarding scratch file) violates **R-SCOPE one-writer-per-file** — each branch owns exactly one discovery file.

### 2. Adversarially verify EVERY finding (≥2-of-3 lenses) before it survives

A finding from a single branch is a **claim, not a fact** (Popper; R-VERIFY). Before any finding enters Stage E synthesis or the score, it must survive **≥2 of 3 independent verification lenses**. A finding that passes <2 lenses is **killed** — dropped from the roadmap and the score (it contributes 0, exactly as "command not run = scores 0 for that finding" already mandates).

| Lens | Question | How (READ-ONLY) | Survives if |
|---|---|---|---|
| **Reproduce** | Does the friction/gap actually exist in the running product surface? | Re-run the phase's falsification command from "EVERY PHASE FALSIFIES" (grep the blocking modal, the missing empty-state branch, the absent invite handler) and paste the output | command returns the predicted evidence with `file:line` |
| **Refute** | Can a counter-search disprove it? (steel-man the product) | Search the codebase / docs for an EXISTING mechanism that already covers the gap (e.g. a hidden `EmptyState` variant, an existing D7 email trigger, a Cmd-K entry) before claiming it's missing | the counter-search comes back empty — the gap is real, not just undiscovered |
| **Cross-check** | Does a second lens or external source independently confirm? | Confirm via a *different* framework lens (e.g. a drop-off flagged by JTBD is also a Heath "pit"; a missing hook flagged by Hooked is also a Fogg low-Trigger) OR a competitor-evidence URL (Phase 6) showing the pattern is table-stakes | ≥1 independent confirmation with citation (lens name OR `http…` URL) |

Record each finding's verdict in `output.json` as `cross_audit_confirmations`-style provenance: which lenses ran, which passed, the evidence string. Findings surviving all 3 lenses are eligible for the higher scoring bands (16-20). A surviving finding with only the Reproduce lens stays — but is capped at the mid band until a second lens confirms. **Killed findings are logged** (in `session.log`) with the lens that refuted them, so the synthesis is auditable and the kill is not silent (R-CITE).

This verification is itself parallelizable: fan out the 3 lenses per finding, require the 2-of-3 quorum, synthesize the verdict yourself — never accept a branch's self-graded "confirmed" as the verdict.

### 3. Synthesize survivors back into THIS audit's existing scoring (unchanged)

Synthesis (Stage E = Phase 20) consumes ONLY the surviving, verified findings and runs the audit's existing machinery verbatim:

- Score each of the 20 phases **0-20** using the existing SCORING RUBRIC; a phase's band is driven by how many of *its* findings survived 2-of-3 (e.g. "16-19: 8+ insights **with falsifiable tests**" now means 8+ *verified* insights). Total **/400**, normalized `/400 × 100 = /100`.
- Run the two-pass **RICE → Fogg B=MAT** scoring exactly as specified; compute `priority_score = RICE_normalized × (1 + Fogg/27)`; sort top 20.
- Apply the ANTI-PATTERNS table (a survivor matching an anti-pattern is flagged `anti_pattern:true` and fixed or dropped — adding adversarial verification does NOT relax the anti-pattern gate).
- Emit the unchanged v2 `output.json` schema and the unchanged report tree under `audits/.retentionaudit/`.
- Run all 7 procedural QUALITY GATES, including **Gate 1 (NO IMPLEMENTATION — `git status --porcelain` must show 0 source edits)**: fan-out branches MUST leave that gate green. If any branch touched a source file, the READ-ONLY contract is violated → ABORT.

The verdict format, the JSON schema, the RICE × Fogg formula, and the report filenames are **byte-for-byte the same** as the linear run. Fan-out + adversarial verification only change *how findings are produced and filtered*, never *how they are scored or reported*.

### 4. Loop-until-dry for unknown-size discovery

Several phases discover an **unbounded** set (Phase 2 screens, Phase 3 drop-off zones, Phase 4 empty states, Phase 18 hidden features). For these, the branch runs a **discovery loop** instead of a single pass:

```
seen = ∅
loop:
  run discovery pass (grep/glob/route-trace) for this surface
  new = results − seen
  for each item in new: adversarially verify (step 2) → keep or kill
  seen ∪= new
  if |new| == 0  → DRY, exit loop
  if iterations >= 5 → record halt_reason:"max_iter" in session.log, exit  # bounded, never infinite
```

This guarantees the audit doesn't stop at the first 5 empty states when there are 30 — it loops until a full pass yields zero new surfaces (dry), bounded by the 5-iteration cap that the shared preamble already mandates (no silent infinite loops). The dry/halt state per surface is recorded so coverage is provable (R-CITE).

### Invariants (do NOT cross)

- **READ-ONLY is absolute.** Fan-out multiplies *observers*, never *writers*. No branch gets `Edit`/`Write` to source; the only writes are this audit's own report files. The skill still "proposes, never codes" and still hands off to `/planner` or `/implement`.
- **Scoring, verdict, schema, phases, frontmatter — untouched.** This section is additive orchestration only.
- **Synthesis is the orchestrator's job.** Never paste a branch's summary as the verdict; merge, de-dupe, and re-rank survivors yourself (R-ORCH).
- **Budget aware.** Fan-out raises peak parallel token draw; respect the mission budget (R-BUDGET) and escalate before overrun rather than truncating phases (Law L5 — no streamlined variant).

---

*"The product is a promise. Retention is keeping it. — /retentionaudit v1.1, May 2026 — Hooked + JTBD + Heath + Fogg, RICE × adoption, READ-ONLY."*
