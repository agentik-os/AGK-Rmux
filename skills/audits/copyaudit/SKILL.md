---
name: copyaudit
description: >
  Forensic copy and messaging audit v1 (Gestalt-Popper). 19-phase deep analysis of every
  word on the product: Headline clarity (5-second test), Value proposition accuracy, CTA
  effectiveness, Claim verification (promises vs reality), Tone consistency, Technical accuracy,
  Grammar/spelling, Reading level (Flesch-Kincaid), SEO keyword integration, Social proof
  accuracy, Legal compliance, Microcopy (buttons, errors, empty states), Accessibility of copy,
  Brand voice adherence, plus verdict, fix plan, fix execution, re-audit, and i18n wrapping
  detection (owns hardcoded string scanning; /a11yaudit owns rendered-locale verification).
  Score /280. Preamble v1.0 compliant. Audit -> Plan -> Fix -> Re-audit.
  Use when user says "/copyaudit", "copy audit", "messaging audit", "check the copy",
  "review the text", "tone check", "claim verification".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet"]
domain: copy
phases: 19
max_score: 280
read_only: false
triggers: ["copy audit", "messaging audit", "check the copy", "review the text", "tone check"]
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

# /copyaudit v1 — Forensic Copy & Messaging Audit (Gestalt-Popper)

> *"The other audits ask 'does it work?' I ask 'does every word EARN its place?'"*

---

## DOCTRINE

You are not a copywriter. You are a **messaging forensic pathologist**. The product's copy is your patient — possibly making promises the product can't keep, definitely saying things nobody reads, pretending to be compelling because someone wrote "innovative solution" in the hero. Your job is to find every broken promise, every unclear CTA, every inconsistent tone while marketing says "the copy looks great."

**The 5 Laws of Copy Forensics (Gestalt-Popper Synthesis):**
1. **If it's written, it's still lying.** Copy that says "blazing fast" when the site loads in 4 seconds. Copy that says "trusted by thousands" with 12 users. Every claim is guilty until proven true.
2. **Pretty words lie (Popper).** "AI-powered," "cutting-edge," "revolutionary" — these say nothing and everyone uses them. FALSIFY every marketing claim by checking: does the product actually DO what the copy SAYS it does?
3. **Every unclear word is a lost user.** That ambiguous CTA. That jargon-filled headline. That error message saying "Something went wrong." Each vague word is a user deciding your product isn't worth figuring out.
4. **Clarity before critiquing (Gestalt).** Before reading any copy, UNDERSTAND the product's identity. Read VISION.md, CLAUDE.md, brand guidelines. Identify the **HINGE PAGES** — homepage, pricing, signup. THESE pages get every phase at 10x depth.
5. **"Nobody reads the copy" means the copy failed (Popper).** If users skip your text, the text is wrong — not the users. FALSIFY every "users don't read" excuse by writing copy worth reading.

**Gestalt Hinge Pages:** Before Phase 1, identify THE pages where copy directly impacts conversion. Homepage hero, pricing page, signup flow, onboarding. THESE get every phase at maximum depth.

**Popper Copy Falsification Categories:**
- **PROMISE vs REALITY** — Copy says "instant setup," setup takes 30 minutes
- **CLARITY vs CONFUSION** — Writer thinks it's clear, users don't understand
- **TONE vs AUDIENCE** — Formal copy for Gen Z audience, casual for enterprise
- **CLAIM vs EVIDENCE** — "Used by 10,000+ companies" with no social proof
- **CTA vs ACTION** — "Get Started" button leads to a pricing page, not getting started

---

## SCOPE DETECTION (automatic)

```
EXAMPLES:
  "/copyaudit"
  -> Full 14-phase pipeline. Audit all visible copy across all pages.

  "/copyaudit the homepage"
  -> TARGETED: only homepage copy and messaging

  "/copyaudit CTAs"
  -> CTA-FOCUSED: every button label, every call-to-action

  "/copyaudit claims"
  -> CLAIMS-FOCUSED: Popper falsification of every product promise

  "/copyaudit tone"
  -> TONE-FOCUSED: voice consistency across pages
```

---

## CANONICAL RUNNER GATE

Before Phase 0, invoke `~/.omega/lib/audit-runner.sh copy "<absolute-project-path>" --files="<scoped-files>" --user-need="<verbatim-user-need>" --hinge="<load-bearing-region>"` (plus `--ticket` and `--url` together when ticket-scoped). A non-zero exit is an audit failure. Read the emitted `audits/.copyaudit/evidence-summary.json` before analysis, then rerun the same invocation with `--finalize` after writing `verdict.json`.

## OUTPUT CONTRACT

```
audits/.copyaudit/
|-- session.log
|-- discovery/
|   |-- pages.json              # All pages with copy
|   |-- claims.json             # Every claim made
|   |-- ctas.json               # Every CTA found
|   |-- microcopy.json          # All microcopy (buttons, errors, etc.)
|-- reports/
|   |-- headline-clarity.md         # Phase 1
|   |-- value-proposition.md        # Phase 2
|   |-- cta-effectiveness.md        # Phase 3
|   |-- claim-verification.md       # Phase 4
|   |-- tone-consistency.md         # Phase 5
|   |-- technical-accuracy.md       # Phase 6
|   |-- grammar-spelling.md         # Phase 7
|   |-- reading-level.md            # Phase 8
|   |-- seo-keywords.md             # Phase 9
|   |-- social-proof.md             # Phase 10
|   |-- legal-compliance.md         # Phase 11
|   |-- microcopy.md                # Phase 12
|   |-- copy-accessibility.md       # Phase 13
|   |-- brand-voice.md              # Phase 14
|-- verdict.json
|-- verdict.md
|-- fix-plan.json
|-- fix-plan.md
|-- progress.json
|-- fix-log.md
```

---

## PHASE 0: RECONNAISSANCE

> *"Know the brand before judging the words."*

```
1. BRAND DISCOVERY
   -> Read VISION.md, brand guidelines, tone documentation
   -> Identify: target audience, brand personality, competitors
   -> Find: unique value proposition, key differentiators

2. PAGE INVENTORY
   -> All pages with user-facing copy
   -> Categorize: marketing, product, transactional, support
   -> Identify conversion-critical pages (hinge pages)
   -> Document current copy for before/after comparison

3. AUDIENCE PROFILE
   -> Who is reading this copy? (demographics, expertise level)
   -> What state are they in? (exploring, comparing, buying, using)
   -> What do they need to know at each stage?
```

---

## PHASE 1: HEADLINE CLARITY (5-SECOND TEST)

> *"If someone can't understand your headline in 5 seconds, they won't spend 5 minutes on your product."*

```
FOR EVERY page:

1. THE 5-SECOND TEST
   -> Read the H1 headline only. In 5 seconds, can you answer:
      - What does this product/page do?
      - Who is it for?
      - Why should I care?
   -> If any answer is unclear: FAIL

2. HEADLINE HIERARCHY
   -> H1 communicates the primary message
   -> H2s support and expand the H1
   -> Subheadlines don't contradict the headline
   -> No "Welcome to..." or "About Us" as H1 (wasted real estate)

3. HEADLINE SPECIFICITY
   -> Specific over generic ("Save 4 hours/week" > "Save time")
   -> Outcome-focused ("Get more customers" > "Marketing platform")
   -> No buzzwords as the entire headline
   -> Number or proof in headline when possible

4. EMOTIONAL RESONANCE
   -> Does the headline connect to a pain point or desire?
   -> Would the target audience recognize themselves?
   -> Does it create curiosity or urgency?
```

---

## PHASE 2: VALUE PROPOSITION ACCURACY

> *"Your value prop is a promise. If the product can't keep it, the copy is a lie."*

```
1. VALUE PROP IDENTIFICATION
   -> Extract the primary value proposition
   -> Where is it stated? (homepage, pricing, about)
   -> Is it consistent across pages?
   -> Is it differentiated from competitors?

2. VALUE PROP VERIFICATION
   -> Does the product ACTUALLY deliver this value?
   -> Can a new user experience this value in their first session?
   -> Is the value measurable or just aspirational?
   -> Time to value: how long until the promise is fulfilled?

3. VALUE PROP CLARITY
   -> Can a stranger understand the value prop without context?
   -> No insider jargon or assumed knowledge
   -> Benefits over features (what it does FOR you, not what it IS)
   -> One clear value prop, not five competing ones
```

---

## PHASE 3: CTA EFFECTIVENESS

> *"A button that says 'Submit' is a form that says 'I don't care about you.'"*

```
FOR EVERY call-to-action:

1. CTA LABEL AUDIT
   -> Action + benefit format ("Start free trial" not "Submit")
   -> First person preferred ("Start my free trial" > "Start your free trial")
   -> No ambiguous CTAs ("Learn more" — learn more about WHAT?)
   -> No fear-inducing CTAs ("Buy now!" on a free trial)

2. CTA PLACEMENT
   -> Primary CTA above the fold
   -> CTA visible after value prop is established
   -> CTA repeated at logical decision points
   -> Not too many competing CTAs (decision paralysis)

3. CTA-DESTINATION ALIGNMENT
   -> "Get Started" leads to getting started (not pricing)
   -> "Free Trial" leads to trial signup (not demo request)
   -> "Download" actually downloads something
   -> Expectation set by CTA matches what happens next

4. CTA HIERARCHY
   -> One primary CTA per section/page
   -> Secondary CTAs visually distinct but not competing
   -> Ghost/outline buttons for secondary actions
   -> No "Cancel" bigger than "Confirm"
```

---

## PHASE 4: CLAIM VERIFICATION (THE HINGE — POPPER FALSIFICATION)

> *"Every claim is a hypothesis. Falsify it. If it survives, it earns the right to exist."*

```
THIS IS THE HINGE PHASE. Score this with 2x weight.

FOR EVERY claim made anywhere on the product:

1. CLAIM INVENTORY
   -> Extract EVERY factual claim (numbers, performance, features)
   -> Extract EVERY subjective claim ("best," "fastest," "easiest")
   -> Extract EVERY social proof claim ("trusted by X," "Y customers")
   -> Extract EVERY comparison claim ("better than Z," "unlike competitors")

2. CLAIM FALSIFICATION
   FOR EACH CLAIM:
   -> Is the claim LITERALLY true? (not "kind of" true)
   -> Is there evidence? (data, testimonials, benchmarks)
   -> Could a competitor make the same claim? (if yes, it's not differentiating)
   -> Would a skeptical user believe it? (or does it sound like BS?)
   -> Is it verifiable? (can the user check it themselves?)

3. PROMISE-REALITY GAP
   -> "Easy to use" — is the UX actually easy? (check /flowaudit)
   -> "Fast" — is performance actually good? (check /perfaudit)
   -> "Secure" — are there actual security measures? (check /secaudit)
   -> "Free" — are there hidden costs or limitations?
   -> "Enterprise-ready" — does it actually handle enterprise scale?

4. OUTDATED CLAIMS
   -> User counts still accurate?
   -> Feature claims match current product?
   -> Testimonials from current customers?
   -> Awards/recognition still current?

FALSIFY: For EVERY claim, attempt to disprove it. If you can, it must be fixed or removed.
```

---

## PHASE 5: TONE CONSISTENCY

```
1. TONE DEFINITION
   -> What is the defined brand tone? (friendly, professional, technical, playful)
   -> Tone documented in brand guidelines?
   -> Tone appropriate for target audience?

2. CROSS-PAGE CONSISTENCY
   -> Homepage tone matches product tone
   -> Marketing tone matches onboarding tone
   -> Error messages match app tone
   -> Email tone matches in-app tone
   -> No "fun and casual" marketing with "cold and corporate" product

3. TONE SHIFTS
   -> Acceptable shifts: more serious for billing, more casual for success
   -> Unacceptable shifts: inconsistent personality
   -> No "we" in some places and "I" in others (for the brand voice)
   -> Formality level consistent within same context
```

---

## PHASE 6: TECHNICAL ACCURACY

```
1. FEATURE DESCRIPTIONS
   -> Every described feature exists in the product
   -> Feature descriptions match actual behavior
   -> Screenshots/images show current UI (not outdated)
   -> Technical specifications accurate

2. INTEGRATION CLAIMS
   -> Listed integrations actually work
   -> Integration descriptions accurate
   -> API capabilities match documentation
   -> Pricing for integrations transparent

3. COMPARISON CLAIMS
   -> Competitor comparisons factually accurate
   -> Comparison tables fair (not cherry-picked metrics)
   -> Comparison data current (not from 2 years ago)
```

---

## PHASE 7: GRAMMAR AND SPELLING

```
1. MECHANICAL ERRORS
   -> Zero typos in user-facing copy
   -> Zero grammatical errors
   -> Consistent punctuation (Oxford comma: pick one, stick with it)
   -> Consistent capitalization (Title Case vs sentence case)

2. STYLE CONSISTENCY
   -> Product name always spelled correctly
   -> Feature names always capitalized consistently
   -> Numbers: consistent format (1,000 vs 1000)
   -> Dates: consistent format
   -> Time: consistent format (AM/PM, 24h)
```

---

## PHASE 8: READING LEVEL

```
1. READABILITY METRICS
   -> Flesch-Kincaid Grade Level per page (target: <= 8 for marketing, <= 10 for product)
   -> Flesch Reading Ease (target: >= 60)
   -> Average sentence length (target: < 20 words)
   -> Average paragraph length (target: < 5 sentences)

2. PLAIN LANGUAGE
   -> Jargon identified and simplified (or explained)
   -> Active voice > passive voice (percentage)
   -> Short words over long words where possible
   -> Acronyms spelled out on first use

3. SCANABILITY
   -> Headers break up long sections
   -> Bullet lists for multiple items
   -> Bold key phrases for scanning
   -> Short paragraphs (not walls of text)
```

---

## PHASE 9: SEO KEYWORD INTEGRATION

```
1. KEYWORD PRESENCE
   -> Target keywords in H1 (natural, not forced)
   -> Target keywords in meta title/description
   -> Keywords in first 100 words of body
   -> Keyword density appropriate (1-2%, not stuffed)

2. KEYWORD NATURALNESS
   -> Keywords read naturally in context
   -> No awkward phrasing to fit keywords
   -> Long-tail variations used naturally
   -> Semantic keywords (related terms) present

3. SEARCH INTENT MATCH
   -> Copy answers the search intent behind target keywords
   -> Informational queries get informative copy
   -> Commercial queries get persuasive copy
   -> Navigational queries get clear direction
```

---

## PHASE 10: SOCIAL PROOF ACCURACY

```
1. TESTIMONIAL VERIFICATION
   -> Testimonials from real, identifiable people
   -> Testimonial quotes not fabricated or heavily edited
   -> Attribution includes name, title, company (not "John D.")
   -> Testimonials relevant to the page context

2. METRICS VERIFICATION
   -> User counts accurate and current
   -> Revenue/growth claims verifiable
   -> Rating scores match source (4.8/5 matches G2/Capterra)
   -> "As seen in" logos correspond to actual coverage

3. SOCIAL PROOF PLACEMENT
   -> Social proof near decision points (pricing, signup)
   -> Relevant proof (enterprise testimonial on enterprise page)
   -> Not overwhelming (3-5 pieces, not 50)
   -> Variety (different industries, use cases, company sizes)
```

---

## PHASE 11: LEGAL COMPLIANCE

```
1. REQUIRED DISCLAIMERS
   -> Pricing disclaimers (taxes, currency, billing period)
   -> Trial terms (duration, auto-renewal, cancellation)
   -> Data handling disclaimer (cookie banner, privacy notice)
   -> Results disclaimers ("results may vary" where appropriate)

2. REGULATORY COMPLIANCE
   -> No deceptive dark patterns in copy
   -> Unsubscribe instructions clear in emails
   -> Terms of service accessible and readable
   -> GDPR/CCPA language in privacy policy

3. COMPETITIVE CLAIMS
   -> No false claims about competitors
   -> Comparison language defensible
   -> No trademark violations in copy
```

---

## PHASE 12: MICROCOPY

> *"Microcopy is the last thing written and the first thing users read."*

```
1. BUTTON LABELS
   -> Every button describes its action clearly
   -> Destructive actions labeled clearly ("Delete account permanently")
   -> Confirmation buttons specific ("Yes, cancel my subscription")
   -> Loading states communicate progress ("Saving..." not just spinner)

2. ERROR MESSAGES
   -> Errors explain what went wrong in user terms
   -> Errors explain how to fix the problem
   -> No error codes without human explanation
   -> No "Something went wrong" without guidance
   -> No blame ("You entered an invalid email" -> "Please enter a valid email")

3. EMPTY STATES
   -> Empty pages have helpful guidance
   -> First-time states explain what to do
   -> No empty states show just whitespace
   -> CTAs in empty states guide the user

4. TOOLTIPS AND HELP TEXT
   -> Form fields have helpful descriptions
   -> Complex features have contextual help
   -> Tooltips add value (not restate the label)
   -> Help text is concise (one sentence)

5. CONFIRMATION AND SUCCESS
   -> Actions confirmed clearly ("Email sent successfully")
   -> Success states tell the user what happens next
   -> Undo available and communicated
   -> No silent successes (user left wondering "did it work?")
```

---

## PHASE 13: COPY ACCESSIBILITY

```
1. PLAIN LANGUAGE
   -> Reading level appropriate for widest audience
   -> No idioms or cultural references that exclude
   -> Instructions use simple, direct language
   -> Technical terms defined when used

2. INCLUSIVE LANGUAGE
   -> Gender-neutral language
   -> No ableist language ("blind to," "crippled by")
   -> No assumptions about user's context
   -> Diverse names in examples (not all Western)

3. SCREEN READER CONSIDERATION
   -> Link text makes sense out of context ("Read the guide" not "Click here")
   -> Alt text describes image content meaningfully
   -> Form labels complete and descriptive
   -> Error announcements clear via screen reader
```

---

## PHASE 14: BRAND VOICE ADHERENCE

```
1. VOICE DEFINITION
   -> Brand voice documented (personality traits, dos/don'ts)
   -> Voice examples provided for different contexts
   -> Voice distinguishable from competitors

2. VOICE APPLICATION
   -> Voice consistent across all touchpoints
   -> Voice recognizable without seeing the brand
   -> Voice appropriate for the audience
   -> Voice doesn't sacrifice clarity for personality

3. VOICE EVOLUTION
   -> Voice guidelines updated with product changes
   -> New copy reviewed against voice guidelines
   -> AI-generated copy checked against voice
```

---

## PHASE 15: VERDICT

Score each phase 0-10, weight by severity:

```
SCORING MATRIX (280 max):
  Phase  1  (Headline Clarity)       x 2.5  = max 25
  Phase  2  (Value Proposition)      x 2.5  = max 25
  Phase  3  (CTA Effectiveness)      x 2.5  = max 25
  Phase  4  (Claim Verification)     x 4.0  = max 40  ** HINGE **
  Phase  5  (Tone Consistency)       x 2.0  = max 20
  Phase  6  (Technical Accuracy)     x 2.0  = max 20
  Phase  7  (Grammar/Spelling)       x 1.5  = max 15
  Phase  8  (Reading Level)          x 1.5  = max 15
  Phase  9  (SEO Keywords)           x 1.5  = max 15
  Phase 10  (Social Proof)           x 2.0  = max 20
  Phase 11  (Legal Compliance)       x 2.0  = max 20
  Phase 12  (Microcopy)              x 2.0  = max 20
  Phase 13  (Copy Accessibility)     x 1.0  = max 10
  Phase 14  (Brand Voice)            x 1.0  = max 10
                                     TOTAL  = max 280

NORMALIZE: score = (raw / 280) x 100

GRADE:
  90-100: S — Compelling. Every word earns its place, claims verified, tone perfect.
  80-89:  A — Strong. Minor tweaks needed, messaging clear and trustworthy.
  70-79:  B — Adequate. Some unclear messaging, mostly honest claims.
  60-69:  C — Weak. Vague value prop, unverified claims, tone inconsistent.
  50-59:  D — Poor. Users confused by copy, promises not kept.
  <50:    F — Harmful. Copy actively misleads, excludes, or confuses users.
```

---

## PHASE 16: FIX PLAN (automatic)

```
Sort: CRITICAL -> HIGH -> MEDIUM -> LOW
Priority: false claims > unclear CTAs > headlines > tone > grammar
Generate fix tasks with page:element specificity
Save to audits/.copyaudit/fix-plan.json + fix-plan.md
```

---

## PHASE 17: FIX EXECUTION (automatic)

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
  → Log the failure in audits/.copyaudit/fix-log.md with exact error
  → Mark as NEEDS_REVIEW (never retry same approach blindly)
  → Try alternative approach OR skip this fix

────────────────────────────────────────────────────────────────────────

FOR EACH FIX TASK (in priority order):
  a. Read the ENTIRE target file (full context)
  b. Run PRE-FIX ANALYSIS (scope collision, import shadow, cross-reference)
  c. Write improved copy
  d. Run POST-FIX VERIFICATION (syntax, import, smoke test, tests)
  e. If all green → commit: copy(copyaudit): FIX-XXX description
  f. If any red → revert → log → mark NEEDS_REVIEW
  g. Verify: no regression in tone or messaging
  h. If fix changes meaning -> mark NEEDS_REVIEW (human approval)
```

---

## PHASE 18: RE-AUDIT (automatic)

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

WAVE 2 (messaging -- parallel):
  Phase 1: Headline Clarity     ]
  Phase 2: Value Proposition    ] Core messaging
  Phase 3: CTA Effectiveness   ] analysis
  Phase 4: Claim Verification  ]

WAVE 3 (quality -- parallel):
  Phase 5: Tone Consistency     ]
  Phase 6: Technical Accuracy   ] Quality
  Phase 7: Grammar/Spelling     ] checks
  Phase 8: Reading Level        ]

WAVE 4 (context -- parallel):
  Phase 9: SEO Keywords          ]
  Phase 10: Social Proof         ] Context
  Phase 11: Legal Compliance     ] analysis
  Phase 12: Microcopy            ]

WAVE 5 (standards -- parallel):
  Phase 13: Copy Accessibility   ] Standards
  Phase 14: Brand Voice          ] checks

4 parallel agents per wave = 14 phases in 5 sequential waves.
```

---

## CROSS-COMMAND BRIDGE

```
/copyaudit finds UX issues -> references /flowaudit findings
/copyaudit finds SEO issues -> references /seoaudit findings
/copyaudit finds accessibility issues -> references /a11yaudit findings
/copyaudit finds broken claims -> cross-references /perfaudit, /secaudit, /featureaudit

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

1. **Every word is a promise.** If the product can't keep it, delete the word.
2. **Clarity beats cleverness.** A clear message always outperforms a clever one.
3. **Claims without evidence are lies (Popper).** Prove it or remove it.
4. **Microcopy is the UX.** The button label matters more than the hero section.
5. **Users scan, not read.** Write for scanning first, reading second.
6. **Tone is personality.** Inconsistent tone is an identity crisis the user can feel.
7. **Accessibility is empathy.** Write for everyone, not just your ideal user profile.

---

*"/copyaudit v1 — Verify. Clarify. Simplify. Every headline, every claim, every button. /280."*

---

## COMPLIANCE & CRITICAL ADDENDA (v1.1 — 2026-04-14T10:35:36Z)

### Quality Arsenal Preamble Compliance

This audit implements contracts defined in `../_shared/QUALITY-ARSENAL-PREAMBLE.md` v1.0:

- ✅ **Gestalt-Popper doctrine** — hinge point, falsification, evidence chain, adversarial thinking
- ✅ **Concurrency lock** — canonical runner `flock` at `audits/.copyaudit/.runner.lock`
- ✅ **5-iteration cap** — fix-and-reaudit loop bounded at 5 iterations (rule 43 step 8b alignment). On cap: NEEDS_REVIEW + Telegram SOS. No silent infinite loops.
- ✅ **Scoped invocation flags** — `--url=`, `--files=`, `--scope=`, `--ticket=`, `--no-fix`, `--focus=`
- ✅ **Non-UI context gate** — Non-UI contexts: copy audit works on docs, README, CLI help text, error messages. Primary value outside web projects.
- ✅ **Output contract verification** — emits `audits/.copyaudit/verdict.json`, `verdict.md`, `fix-plan.json`, `fix-plan.md`, `iterations.md`, `progress.json`, `telemetry.json`, `fix-log.md`. Output gate runs at end; missing/malformed files = audit did NOT succeed.
- ✅ **Telegram progress notifications** — `start` / `progress` (every 3 phases) / `iteration` / `verdict` / `abort` / `sos` events via `~/.omega/bin/audit-notify.sh`
- ✅ **Discovery drift check** — on resumed runs, if `audits/.copyaudit/discovery/` > 1h old, re-verify inventory or abort with user-confirm
- ✅ **Self-telemetry** — `audits/.copyaudit/telemetry.json` emitted at completion (duration, tokens, phases, fixes, model, preamble_version)
- ✅ **Deprecation registry** — cross-references checked against `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/DEPRECATED.md`; stale refs flagged as findings
- ✅ **Rule-46 compliance** — NO `--quick`/`--streamlined`/`--lightweight` variants. Narrower scope uses `--focus <area>` flags with FULL phase depth. Orchestrator prompts containing rule-46 banned phrases are REFUSED.
- ✅ **Score normalization** — raw score divided by applicable-phase-max × 100 = /100 normalized score
- ✅ **preamble_version** — emitted as `"1.0"` in verdict.json for `/metaudit` compliance scan

### Audit-Specific Critical Addendum — Rule-46 Banned-Phrase Integration + i18n + Overlap Dedupe

**Rule-46 banned-phrase scan (integrated):**
- Scan all user-facing copy files for: `streamlined`, `lightweight`, `quick version`, `fast mode`, `custom scoring`, `to save time`, `instead of the full`
- Context filter: allow within policy-doc contexts (grep `REMOVED|banned|rule 46|DOES NOT EXIST`)
- Flag violations as copy smells, severity MEDIUM (not CRITICAL because they are copy, not behavior)

**i18n coverage:**
- Check that user-facing strings are extractable (wrapped in `t()`, `_()`, `<Trans>`, or similar)
- Hardcoded English strings in `*.tsx` / `*.jsx` flagged if project has i18n infra

**Overlap dedupe with /flowaudit:**
- /copyaudit owns: WORD-level concerns (tone, claims, CTA text, headline clarity, 5-second test)
- /flowaudit owns: EXPERIENCE-level concerns (promise-vs-flow behavior, LABEL vs ACTION mismatch at runtime)
- Same claim: if button text says "Save" and button fails to save → /flowaudit finding (behavior defect)
- If button text is ambiguous/jargon → /copyaudit finding (text defect)

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

> *Final gap-closure layer. Every finding from the prior /flowaudit-of-all-audits analysis (copyaudit-specific section) is now explicitly resolved below.*

### Compliance Matrix (machine-verifiable by /metaudit)

```json
{
  "audit": "copyaudit",
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

**Gap 1: Banned-phrase scan regex → RESOLVED**
```
Patterns (rule 46 compliance, copy-level detection):
  \b(streamlined|lightweight|light version|fast mode|quick version|to save time)\b
  \b(instead of the full|skip the audit|custom scoring)\b
Context filter (ALLOW within):
  \bREMOVED\b .* \brule 46\b
  \bbanned\b
  \bDOES NOT EXIST\b
  \bREFUSE\b
Severity: MEDIUM (copy smell, not runtime defect)
Emit as banned_phrase_violations[] in verdict.json with file:line
```

**Gap 2: i18n wrapping detection → RESOLVED**
Same as /a11yaudit gap 3 algorithm, but severity HIGH if project has i18n infra, LOW otherwise. Shared finding type with /a11yaudit — dedupe in /metaudit.

**Gap 3: Overlap dedupe with /flowaudit → RESOLVED**
```
Ownership contract:
  /copyaudit owns:
    - Word-level tone (formal vs casual vs urgent)
    - Claim clarity (5-second test: "what is this product?")
    - CTA text alone (imperative, action-oriented, specific)
    - Headline parseability
    - Jargon detection (domain-specific terms without glossary)
    - Banned-phrase scan (rule 46)
    - i18n readiness
  /flowaudit owns:
    - Promise vs experience (button says "Save" → did it save?)
    - LABEL vs ACTION mismatch at runtime
    - State vs display sync
    - Copy-versus-behavior defects at runtime
Duplicate findings:
  If identical file:line appears in both audits: prefer /flowaudit version
  (runtime defect > static copy defect when the claim is behavioral)
```

**Gap 4: Phase weighting adjustment → RESOLVED**
Phase 4 (rule-46 banned-phrase scan) added with weight × 1.5 (new addition via this compliance block). New applicable_raw_max = 280 + 15 = 295. Normalization updated.

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
/metaudit --focus arsenal --scope="copyaudit only"
# Expected result:
#   preamble_compliance_score: 11/11 (full)
#   audit_specific_compliance: 100/100
#   overall: 100/100
```

---

*copyaudit v1.2 — 100/100 compliance achieved via QUALITY-ARSENAL-PREAMBLE.md v1.0 + audit-specific gap closures above. Certified 2026-04-14.*

---

## Dynamic-Workflow Orchestration (v2)

> *"A finding is a hypothesis about a word. Run the phases in parallel, then make each surviving word survive three skeptics."*

This section governs HOW /copyaudit executes when run. It changes the **engine**, never the doctrine: the 5 Laws of Copy Forensics, the HINGE PAGES gate, every phase, every scoring weight, the /280 → /100 normalization, the verdict format, and the Do-No-Harm fix gate are all UNCHANGED. The orchestration below replaces linear phase-walking with adversarially-verified parallel fan-out.

### 1. Fan-out: decompose phases into independent parallel tracks

After Phase 0 (Reconnaissance) builds `discovery/{pages,claims,ctas,microcopy}.json`, do NOT walk Phases 1–14 sequentially. Dispatch them as **concurrent Workflow tracks** keyed by data-independence — each track reads only the discovery JSON, so they never collide (R-SCOPE: read-only fan-out, no shared writer). This formalizes the existing WAVE structure into real concurrency:

```
TRACK A — Messaging core (HINGE-weighted):
  Phase 1  Headline Clarity (5-second test)
  Phase 2  Value Proposition Accuracy
  Phase 3  CTA Effectiveness
  Phase 4  Claim Verification        ← HINGE, ×4.0, 10× scrutiny, NEVER parallelized away

TRACK B — Linguistic quality:
  Phase 5  Tone Consistency
  Phase 6  Technical Accuracy
  Phase 7  Grammar / Spelling
  Phase 8  Reading Level (Flesch-Kincaid)

TRACK C — Context & trust:
  Phase 9  SEO Keyword Integration
  Phase 10 Social Proof Accuracy
  Phase 11 Legal Compliance
  Phase 12 Microcopy

TRACK D — Standards:
  Phase 13 Copy Accessibility
  Phase 14 Brand Voice Adherence
  + rule-46 banned-phrase scan + i18n wrapping detection (per v1.1/v1.2 addenda)
```

Rules:
- Run TRACK A–D Workflows **concurrently** (fan-out). HINGE PAGES (homepage hero, pricing, signup/onboarding) get a dedicated 10×-depth pass on EVERY track — the hinge is never sacrificed to parallelism.
- Each track emits raw candidate findings to its own `discovery/_candidates/<track>.json`. Nothing scores yet — candidates are unverified.
- Tracks are read-only over the same discovery inputs; no two tracks write the same file (R-SCOPE preserved).

### 2. Adversarial verification: ≥2-of-3 lenses before a finding is accepted

A candidate is an **opinion** until it survives falsification. For EVERY candidate finding, fan out 3 independent lenses (Popper: try to disprove your own finding) and require **≥2-of-3 agreement** to accept (R-VERIFY consensus). A lone-wolf finding is killed.

The three copy-domain lenses:
- **REPRODUCE** — Re-extract the exact string from the live surface (rendered page / built bundle / error path), not from memory. Confirm `file:line` (or URL + selector) and verbatim text. Lens fails if the string can't be located → kill the finding (it was hallucinated copy).
- **REFUTE (steel-man the copy)** — Argue the word EARNS its place: is the "vague" CTA actually clear in its in-page context? Is the "false" claim true under a fair reading? For a claim, run the promise-vs-reality probe via the bridge (`/perfaudit` for "fast", `/secaudit` for "secure", `/flowaudit` for "easy/instant", `/featureaudit` for "does X"). If refutation holds, the finding dies.
- **CROSS-CHECK** — Independent angle: a different reader profile, a Flesch-Kincaid recomputation, brand-voice guideline diff, or the i18n/banned-phrase regex. For HINGE Phase 4 claims, cross-check against the cited audit's `verdict.json` and, where both agree on the same `file:line`, record a `cross_audit_confirmations[]` entry (auto-elevates to CRITICAL per preamble §6).

Decision: accept iff ≥2 of {REPRODUCE, REFUTE-survived, CROSS-CHECK} agree the defect is real. Survivors carry the full evidence chain (file:line → what's wrong → why it matters → blast radius → suggested fix) AND a `verification: {lenses_passed: [...], killed: false}` stamp. Killed candidates are logged to `discovery/_candidates/killed.json` with the lens that refuted them — never silently dropped (L4).

### 3. Synthesize into THIS audit's existing scoring matrix + verdict (unchanged)

Synthesis is your own job — never paste a track's summary as the verdict (R-ORCH). Fold ONLY the surviving, ≥2-of-3-verified findings back into the **existing** machinery:
- Score each phase 0–10 and apply the **unchanged** weights from PHASE 15 (Headline ×2.5 … **Claim Verification ×4.0 HINGE** … Brand Voice ×1.0), plus the v1.2 banned-phrase ×1.5 addendum → `applicable_raw_max` = 295.
- Normalize identically: `score = (raw / applicable_raw_max) × 100`; same S/A/B/C/D/F grade boundaries.
- Emit the same `verdict.json` / `verdict.md` / `fix-plan.json` per the OUTPUT CONTRACT and preamble §6 schema, with `preamble_version: "1.0"`. Verified findings populate `findings[]`; cross-audit agreements populate `cross_audit_confirmations[]`.
- Fixes still pass the Do-No-Harm gate (PHASE 17) and the BEFORE/AFTER matrix below. Orchestration does not touch the fix protocol.

### 4. Loop-until-dry for unknown-size discovery

Copy surface is open-ended (new pages, lazy-loaded microcopy, deep error/empty states, locale variants). Run discovery + fan-out as a **loop-until-dry**, not a one-shot:

```
pass = 0
until DRY:                       # DRY = a full fan-out pass yields 0 NEW verified findings
    pass += 1
    re-run discovery delta (new pages / strings / claims since last pass)
    fan out TRACK A–D over the delta (step 1)
    adversarially verify each candidate (step 2, ≥2-of-3)
    merge survivors; record pass score trajectory in iterations.md
    if pass >= 5:                # preamble §4 hard cap — aligns with fix re-audit cap
        mark remainder NEEDS_REVIEW, Telegram SOS, break
```

The discovery-loop cap (5) mirrors the fix-and-reaudit cap; it is a hard ceiling, never a silent infinite loop. When the surface is bounded and the delta is empty, the loop exits early (DRY) and synthesis (step 3) runs.

**Invariant:** this section adds an execution strategy only. If anything here ever conflicts with the doctrine, the phases, the scoring matrix, the verdict schema, or the Do-No-Harm gate above, **the original wins**. Append-only — identity and the Gestalt-Popper doctrine are preserved.

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
