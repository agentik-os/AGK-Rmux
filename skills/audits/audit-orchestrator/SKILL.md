---
name: audit-orchestrator
description: >
  Intelligent audit orchestrator — detects project type + user intent, recommends
  optimal audits with three bounded modes (Focused/Standard/Forensic). Use when user
  says "/audit", "what should I audit", "full audit", "audit my project",
  "audit fast", "audit deep", "find issues", "improve quality", "production
  ready check", "ship-ready audit". Auto-detects project stack and intent
  keywords (speed, security, design, content, accessibility, full) to pick
  best 1-N audits. Dispatches in parallel waves. Reads results from
  audits/.<audit-id>/verdict.json after each run.
disable-model-invocation: false
---

# /audit-orchestrator — Intelligent Audit Selection + Power Levels

You are the **audit conductor**. Given a user request and a project, pick the
RIGHT audits at the RIGHT power level, dispatch them, and synthesize results.

## How to invoke

```bash
/audit-orchestrator               # interactive: ask user what to audit
/audit-orchestrator full          # run all 23 audits in parallel
/audit-orchestrator quick         # compatibility alias: focused top 5, full phases
/audit-orchestrator standard      # smart selection at Standard level (default)
/audit-orchestrator forensic      # deep Gestalt-Popper on selected audits
/audit-orchestrator security      # secaudit + apiaudit + dataaudit
/audit-orchestrator performance   # perfaudit + seoaudit
/audit-orchestrator design        # uiuxaudit + motionaudit + a11yaudit + copyaudit
```

## The 23 audits in the Quality Arsenal

| Audit | Domain | When to pick |
|---|---|---|
| `/codeaudit` | Code architecture | New codebase, refactor, technical debt |
| `/secaudit` | Security (OWASP) | Pre-prod, payment handling, auth surfaces |
| `/uiuxaudit` | Design quality | Visual consistency, design system audit |
| `/refontaudit` | Dashboard refonte | Senior-level redesign (comme Linear/Vercel) |
| `/flowaudit` | User journeys | Onboarding, conversion drops, dead-ends |
| `/debugaudit` | Runtime bugs | Console errors, broken features, smoke test |
| `/featureaudit` | Completeness | PRD validation, ship-readiness, "what's missing" |
| `/perfaudit` | Core Web Vitals | Slow site, lighthouse improvement |
| `/a11yaudit` | WCAG 2.1 AA | Accessibility, screen readers, contrast |
| `/seoaudit` | Discoverability | Search ranking, GEO/AEO, schema markup |
| `/dataaudit` | Schema integrity | Orphaned records, migrations, RGPD |
| `/apiaudit` | API contracts | Endpoint quality, auth matrix, rate limits |
| `/copyaudit` | Messaging | Claims vs reality, CTA, tone |
| `/dxaudit` | Dev experience | README quality, onboarding new devs |
| `/motionaudit` | Animation design | Transitions, easing, motion brand DNA |
| `/automationaudit` | Cron/scripts | Daemon health, scheduled tasks reliability |
| `/logicaudit` | Architecture | Algorithm efficiency, redundant logic |
| `/i18naudit` | Internationalization | Locale coverage, RTL, hardcoded copy |
| `/retentionaudit` | Product/CPO | Feature opportunities, RICE roadmap (READ-ONLY) |
| `/depaudit` | Supply chain | Dependencies, licenses, lockfiles, SBOM |
| `/privacyaudit` | Privacy | PII, consent, retention, GDPR/CCPA |
| `/releaseaudit` | Release safety | CI/CD, rollback, migrations, provenance |
| `/observabilityaudit` | Observability | Signals, alerts, traces, incident diagnosis |

## The 3 Power Levels

### ⚡ Level 1 — Focused
- Select the top 5 highest-impact audits, but execute every required phase
- Narrow scope with `--focus`; never weaken Plan, Fix, Re-audit, or scoring
- Output: complete `audits/.<audit-id>/verdict.json` + reports
- The legacy `quick` invocation is a compatibility alias for this mode

### 🎯 Level 2 — Standard (30-60 min, DEFAULT)
- Full phases: Audit → Plan → Fix → Re-audit
- Score normalized /100
- Output: complete `audits/.<audit-id>/verdict.json` + reports
- Use case: regular quality cycle, pre-PR validation

### 🔬 Level 3 — Forensic (1-4h per audit)
- Full Gestalt-Popper protocol, all phases extended
- Auto-fix every finding P0/P1/P2
- Re-audit cycles until 100/100 (or 3 cycle cap)
- Output: forensic-grade with falsification proofs + telemetry
- Use case: pre-launch, security/compliance gate, "make it bulletproof"

## Smart Selection Algorithm

When user says ambiguous request like "audit my project":

```
1. DETECT PROJECT TYPE
   - Check package.json: React/Next.js/Vue → UI audits relevant
   - Check requirements.txt/pyproject.toml: Python → no motion/uiux
   - Check .convex/ or prisma/: dataaudit relevant
   - Check api/ or routes/: apiaudit relevant
   - Check .github/workflows/: dxaudit + automationaudit
   - No src/ but docs/: feature/copy/seo only (docs project)

2. PARSE INTENT KEYWORDS (English + French)
   - "speed/fast/lent/lenteur" → perfaudit (+ seoaudit if web)
   - "security/sec/vuln/secure/sécurité" → secaudit + apiaudit
   - "design/visual/UI/UX/style" → uiuxaudit + motionaudit
   - "content/copy/messaging/text" → copyaudit
   - "accessibility/a11y/WCAG/handicap" → a11yaudit
   - "API/endpoint/contract" → apiaudit + dataaudit
   - "complete/missing/done/ship-ready" → featureaudit
   - "code/quality/refactor" → codeaudit + logicaudit
   - "retention/features/CPO/sticky" → retentionaudit
   - "data/schema/migration" → dataaudit
   - "automation/cron/scripts" → automationaudit
   - "bug/error/broken/runtime" → debugaudit
   - "redesign/refonte/dashboard" → refontaudit
   - "full/all/everything/complet" → ALL 23 audits

3. PICK POWER LEVEL
   - Default: Standard (Level 2)
   - User mentions "quick/fast/rapide" → Focused (Level 1, full phase depth)
   - User mentions "deep/forensic/production/launch/100" → Forensic (Level 3)

4. CHECK PROJECT MATURITY
   - Empty src/ or fresh scaffold → skip code-focused audits, run featureaudit+copyaudit
   - Mature codebase → all relevant
   - Pre-launch → add secaudit + a11yaudit + perfaudit (the "go-live trio")
```

## Execution Plan Output

Before dispatching, OUTPUT a plan like:

```
🎯 AUDIT PLAN — {project_name}

Detected:
  Stack:     Next.js + Tailwind + Convex
  Maturity:  Production (12 months)
  Intent:    "make sure it's secure before launch"

Recommended (Power Level: Forensic):
  1. /secaudit       (OWASP + payment surfaces — primary)
  2. /apiaudit       (auth matrix + rate limits — secondary)
  3. /dataaudit      (RGPD + orphan records — context for /apiaudit)
  4. /a11yaudit      (legal compliance — go-live blocker)
  5. /perfaudit      (CWV — go-live blocker)

Estimated duration: 4-6h (parallel waves)
Estimated tokens: ~800K
```

### Plan confirmation — session-scoped (Law L3 autonomy)

The confirmation step depends on WHO is running this:

- **Interactive session** (Home / a human is watching the terminal): print
  `Approve? [y/n/customize]` and WAIT for the answer before dispatching.
- **Dispatched session** (spawned by an oracle / worker / AISB — any tmux name
  matching `oracle-*`, `*-linear`, `*-fix-*`, `*-dev-*`, `AISB-*`, or started via
  `dispatch-to-session.sh`): do NOT ask. Pausing for `[y/n]` strands the run
  because nobody is watching (Law L3). Instead, **LOG the decision and proceed
  at the default power level**:

  ```
  Plan confirmation auto-approved (dispatched session — L3 autonomy).
  Power level: {detected default — Standard unless a forensic/go-live keyword was given}.
  Proceeding to dispatch.
  ```

  If the detected scope is genuinely ambiguous with no safe default, write the
  block-file (`~/.omega/state/worker-blocked-<session>.json`) with the candidate
  plan as the fallback and execute that fallback — never idle at a prompt.

## Full Audit Mode

When user says "full audit" / "audit complet" / "tous les audits":

1. Dispatch ALL 23 audits in 3 parallel waves (file-safety partitioned):
   - **Wave 1** (independent inventory, can parallel): codeaudit, logicaudit, dataaudit, apiaudit, seoaudit, featureaudit, retentionaudit, copyaudit, dxaudit, i18naudit, depaudit, privacyaudit
   - **Wave 2** (consumes Wave 1 evidence): secaudit, perfaudit, debugaudit, automationaudit, releaseaudit, observabilityaudit
   - **Wave 3** (experience bundle, after Wave 1): uiuxaudit, refontaudit, motionaudit, a11yaudit, flowaudit
2. After all done, generate `audits/SYNTHESIS.md` aggregating scores
3. Score the project: average /100 across all audits + flag any < 80
4. Telegram report with verdict + button to view each detailed report

## State Tracking

Read `audits/SYNTHESIS.md` at start to know what's already done:

```yaml
last_full_audit: 2026-05-13T12:00:00Z
scores:
  codeaudit: 92/A
  secaudit: 88/A
  uiuxaudit: 91/S
  ...
status:
  fresh:    [codeaudit, secaudit]  # < 7 days old
  stale:    [perfaudit]            # 7-30 days old
  expired:  [a11yaudit]            # > 30 days, recommend re-run
```

## Output Convention

ALL audits MUST write to `audits/.<audit-id>/`, where `<audit-id>` is the exact
identifier from `skills/audits/registry.toml`. Never write a machine audit root
at the project root. The audit-orchestrator
+ audit-tracker skills assume this canonical path.

## Anti-patterns

- ❌ Running `/codeaudit` when project has no source code (use /dxaudit instead)
- ❌ Running `/motionaudit` on CLI/library project (it ABORTS automatically)
- ❌ Forensic level on every audit (token waste; use Standard unless go-live)
- ❌ Skipping the plan output (always print the plan). In an INTERACTIVE session also wait for `[y/n/customize]`; in a DISPATCHED session log auto-approval and proceed at default power (Law L3) — never idle at a `[y/n]` prompt nobody is watching
- ❌ Running audits in serial when waves allow parallelism
- ❌ Treating retentionaudit as fix-mode (it's READ-ONLY by design)

## Workflow

```
User: "/audit-orchestrator security"
  ↓
You: parse "security" → secaudit + apiaudit + dataaudit
You: detect project at Standard level (no "deep/forensic" keyword)
You: emit plan markdown
     → interactive session: ask `[y/n/customize]` and wait
     → dispatched session: log auto-approval, proceed at default power (Law L3)
  ↓
User: "y"   (interactive only; dispatched skips straight to dispatch)
  ↓
You: dispatch 3 audits in parallel via tmux work sessions
You: monitor verdict.json files appearing under audits/.<audit-id>/
You: when all 3 done, write audits/SYNTHESIS.md
You: send Telegram report with aggregate score + per-audit links
```

## When to invoke alternative skills

- For a SINGLE specific audit → user types `/codeaudit` directly (not via orchestrator)
- For audit setup / .gitignore / progress dashboard → use `/audit-tracker`
- For oracle dispatch of audit chain → use `/aisb full`

## Dynamic-Workflow Orchestration (v2)

> This section UPGRADES *how* the orchestrator runs — it does NOT change WHAT it
> selects, the 3 power levels, the wave partitioning, the `audits/SYNTHESIS.md`
> contract, or the aggregate `/100` verdict. Same identity, same Gestalt-Popper
> doctrine (hinge-first selection, Popper falsification, runtime > code >
> comments). It just turns this conductor from a linear "decide → dispatch →
> wait → synthesize" loop into a **fan-out → adversarially-verify → synthesize →
> loop-until-dry** workflow. The orchestrator owns no scoring matrix of its own;
> its "verdict" is the aggregate it writes to `SYNTHESIS.md`, and that is
> untouched here.

When this orchestrator runs (any invocation — interactive, `full`, a level, or a
keyword bundle), execute the steps below via the **Workflow tool** instead of
grinding them in series.

### 1. Decompose into INDEPENDENT parallel tracks (fan-out)

The orchestrator's pre-dispatch work splits into file-disjoint, side-effect-free
tracks that have no data dependency on each other. Run them **concurrently** in a
single Workflow fan-out, not one after another:

- **Track A — Stack/signal detection** (§"Smart Selection Algorithm" step 1 +
  preamble §16): read `package.json`, lockfiles, `.convex/`, `prisma/`, `api/`,
  `.github/workflows/`, `tailwind.config.*`, env signals → emit
  `project_signals_detected`.
- **Track B — Intent parsing** (step 2): map the user's EN+FR keywords to the
  candidate audit set + chosen power level.
- **Track C — Maturity probe** (step 4): empty-scaffold vs mature vs pre-launch
  ("go-live trio") classification from git age + src density.
- **Track D — Prior-state read**: load existing `audits/SYNTHESIS.md` +
  `audits/.<audit-id>/verdict.json` freshness (fresh / stale / expired) so
  already-fresh audits are skipped, not re-run.

These tracks are read-only and touch different inputs → safe to parallelize
(R-SCOPE: no shared writer). Join their outputs to compute the recommended audit
list + level. The **dispatch waves themselves stay exactly as defined** in
"Full Audit Mode" (Wave 1 read-only fan-out → Wave 2 consumers like secaudit that
read apiaudit's verdict → Wave 3 UI bundle); each wave is itself a fan-out, and
wave ordering is preserved because Wave 2/3 have real data dependencies on Wave 1
verdicts. Parallelize within a wave; serialize across waves.

### 2. ADVERSARIALLY VERIFY each finding before it reaches SYNTHESIS (>=2-of-3)

The orchestrator does not run forensic phases itself — its "findings" are the
**cross-audit signals it aggregates**: each child audit's top findings, every
`cross_audit_confirmations` elevation, and every score it is about to average
into `SYNTHESIS.md`. Before ANY of these is accepted into the synthesis, subject
it to **three independent lenses and require >=2 to agree**:

- **Lens 1 — Reproduce**: re-read the child audit's own `verdict.json` evidence
  chain (file:line → evidence → blast radius). Does the cited artifact actually
  exist and say what the finding claims? (Runtime > code > comments.)
- **Lens 2 — Refute (Popper)**: actively try to kill it. Is the score an ABORT
  misread as a pass (a 401/403/empty-surface scored as green = ABORT, never a
  PASS — Law L5)? Did the audit hit its 5-iteration cap and leave
  `needs_review`? Is the "finding" an opinion with no falsifiable test?
- **Lens 3 — Cross-check**: does an *independent* audit corroborate it? A
  finding confirmed on the same file:line by a second audit is the existing
  elevation-to-CRITICAL mechanism (preamble §6); a contradiction between two
  audits is itself a finding to surface, not silently averaged away.

A signal that survives <2 lenses is **killed** — dropped from `SYNTHESIS.md` and
recorded under a `rejected_signals` note (with the failing lens) so the kill is
auditable (R-CITE). A delegate audit's own "done" is an input, never the verdict
(R-VERIFY). Synthesis is the orchestrator's own job — never paste a child audit's
summary verbatim as the aggregate truth (R-ORCH).

### 3. Synthesize survivors into the EXISTING aggregate (unchanged)

Fold only the surviving signals back into the orchestrator's existing outputs —
the `audits/SYNTHESIS.md` aggregate, the `average /100 across all audits + flag
any < 80` scoring, and the plan/Telegram report described above. **The synthesis
format, the averaging rule, the grade bands, and the per-audit links are
unchanged.** This section adds a verification stage in front of synthesis; it
does not alter the synthesis itself.

### 4. Loop-until-dry for unknown-size discovery

Audit selection is open-ended: a signal detected in Track A can pull in audits
the keyword bundle never named (e.g. discovering `stripe` →
`/flowaudit --focus=payment`; discovering `prisma/` → `/dataaudit` + `/apiaudit`;
discovering `framer-motion` → `/motionaudit`). Run selection as a **convergence
loop**:

```
candidates = fan-out(Track A..D)         # initial set + level
do:
    run the next eligible wave (fan-out within the wave)
    read the new verdicts → adversarially verify (step 2)
    rescan project_signals_detected from fresh discovery artifacts
    add any newly-relevant audits to the pending waves
until: no new audit surfaces AND no wave has eligible un-run audits   # dry
then: synthesize (step 3)
```

Bound the loop the same way the family already bounds work: honor the 4h
concurrency lock per audit, the 5-iteration fix cap inside each child audit, and
the mission token budget (R-BUDGET — escalate near the cap, never silently
overrun). "Dry" = a full pass adds zero new audits and leaves zero eligible
un-run audits; only then is the aggregate verdict final.

### Net effect

Same audits, same levels, same waves, same `SYNTHESIS.md` verdict — but selected
by parallel discovery, hardened by >=2-of-3 adversarial verification so a single
hallucinated child finding can't poison the aggregate, and exhaustive by
loop-until-dry so a late-discovered stack signal never leaves a relevant audit
unrun.

## Sources

- 23 Quality Arsenal audits declared by `skills/audits/registry.toml`
- Helper docs: `ARSENAL-ORCHESTRATION-PLAYBOOK.md`, `ARSENAL-INTERCONNECTIONS.md`
- Public mirror: https://github.com/agentik-os/quality-arsenal
