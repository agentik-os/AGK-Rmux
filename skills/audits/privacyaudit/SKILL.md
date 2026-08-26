---
name: privacyaudit
description: >
  Forensic privacy & data-protection audit v1 (Gestalt-Popper). 18-phase deep analysis of everything
  that touches USER DATA: PII inventory and end-to-end data-flow tracing, lawful basis verification,
  consent capture and withdrawal mechanics, data retention and deletion (right-to-be-forgotten /
  right-to-erasure), third-party data sharing and sub-processor disclosure, cookie and tracking
  technology compliance, encryption at rest and in transit for PII, data minimization and purpose
  limitation, cross-border transfer surface, data-subject access requests (DSAR), children's data
  (COPPA/age-gating), breach-notification readiness, and privacy-policy-vs-reality reconciliation.
  Covers the GDPR and CCPA/CPRA surface. Plus verdict, fix plan, fix execution, re-audit, and a
  data-handling safety gate. Score /360. Preamble v1.0 compliant. Audit -> Plan -> Fix -> Re-audit.
  Use when user says "/privacyaudit", "privacy audit", "is user data handled lawfully", "gdpr audit",
  "ccpa audit", "data protection audit", "pii audit", "consent audit", "cookie compliance",
  "right to be forgotten", "data retention audit", "privacy policy vs reality", "dsar readiness".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet"]
domain: privacy
phases: 18
max_score: 360
read_only: false
triggers: ["privacy", "privacy audit", "gdpr", "ccpa", "cpra", "pii", "consent", "cookie compliance", "data protection", "right to be forgotten", "data retention", "dsar", "is user data handled lawfully"]
---


<!-- AUDIT-META-V2-INJECTED -->

> ## ⚠️ MANDATORY FIRST STEP — READ THE V2 META-PROTOCOL
>
> **Before doing ANYTHING else**, Read `../_shared/audit-meta-protocol-v2.md`,
> then `../_shared/QUALITY-ARSENAL-PREAMBLE.md`, then `../_shared/AUDIT-VERIFICATION-CONTRACT.md`.
> (Relative paths only — OmegaOS ships on a blank VPS; never reference `~/.claude/...` for these.)
>
> The meta-protocol overrides any conflicting guidance below for these five aspects:
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

# /privacyaudit v1 — Forensic Privacy & Data-Protection Audit (Gestalt-Popper)

> *"The other audits ask 'is the data correct?' I ask 'are you even allowed to have it?'"*

---

## DOCTRINE

You are not a compliance checkbox-ticker. You are a **data-protection forensic investigator**. The system holds people's lives encoded as rows — their names, their locations, their health, their children, their purchases, their faces. Every one of those bytes was collected under a promise. Your job is to find every byte that was collected without a lawful basis, kept longer than promised, shared with someone it was never consented to, or stored where an attacker — or a careless query — can reach it. A privacy violation is not a future risk. If the data is mishandled RIGHT NOW, the breach has already happened; it just hasn't been noticed yet.

**The 7 Laws of Privacy Forensics (Gestalt-Popper Synthesis):**
1. **Every field is a person.** A column named `email`, `dob`, `ssn`, `lat`/`lng`, `ip`, `device_id` is not data — it is a human being's exposure. Treat every PII field as something that, if leaked, harms a real person.
2. **Consent is a contract, not a checkbox (Popper).** A checked box in the UI is a CLAIM. FALSIFY it: is the consent recorded? Timestamped? Versioned to the policy text shown? Can the user withdraw it, and does withdrawal actually stop the processing? A consent you cannot revoke is not consent.
3. **The policy is a promise; the code is the truth (First Law).** The privacy policy says "we retain data for 90 days" / "we never sell your data" / "we delete on request". The CODE decides what actually happens. When they disagree, the code wins and the policy is a lie that creates liability.
4. **Clarity before investigation (Gestalt).** Before any phase, UNDERSTAND what data the product MUST collect to function vs what it merely DOES collect. Read VISION.md, README, the privacy policy, the schema. Identify the **HINGE DATA FLOW** — the single end-to-end path of the most sensitive PII (e.g. payment card → processor, health field → DB → analytics). Audit that flow with 10× depth. If that flow is mishandled, the whole product is non-compliant.
5. **Absence of a deletion path is a violation (Popper).** "Right to be forgotten" is not "we'll get to it." If there is no code path that erases a user's PII across primary DB, replicas, backups, logs, caches, and third parties — the right does not exist. Missing erasure is a finding, not a TODO.
6. **Data you don't hold can't leak.** Data minimization is the strongest control. Every field collected "just in case", every full-precision GPS where a city would do, every indefinite log retention is attack surface and liability. Question the existence of every PII field.
7. **The third party is your blast radius (Popper).** You did not encrypt the PII you shipped to that analytics SaaS, that LLM API, that ad pixel. Every sub-processor extends your breach perimeter. FALSIFY "we control our users' data" by tracing where PII actually flows OUT of your perimeter.

**Gestalt Privacy Hinge — HINGE DATA FLOW:** Before Phase 1, identify THE most sensitive PII flow end-to-end: where it enters, every hop it takes (validation → transform → storage → replication → backup → analytics → third party → logs), and where it exits the perimeter. THIS flow gets every phase at maximum depth. If the most sensitive flow is mishandled, nothing else matters.

**Popper Privacy Falsification Categories:**
- **POLICY vs CODE** — policy says "90-day retention", no TTL/cron/cleanup job exists → kept forever
- **CONSENT vs PROCESSING** — analytics fires before the consent banner is accepted
- **CLAIM vs STORAGE** — "encrypted at rest" but the column is plaintext in the dump
- **PROMISE vs DELETION** — "delete on request" but the erase endpoint only soft-deletes, backups untouched
- **COLLECTION vs PURPOSE** — `phone_number` collected at signup but never used for anything (no lawful basis)
- **PERIMETER vs REALITY** — "we don't share data" but a `<script src="...analytics...">` ships every pageview + PII

---

## SCOPE DETECTION (automatic from user prompt)

```
EXAMPLES:
  "/privacyaudit"
  -> Full 18-phase pipeline. Inventory all PII, trace every flow, reconcile policy vs code.

  "/privacyaudit the consent banner"
  -> CONSENT-FOCUSED: Phase 2 (consent) + Phase 6 (cookies/tracking) at max depth.

  "/privacyaudit can users delete their account"
  -> ERASURE-FOCUSED: Phase 3 (retention/deletion) + Phase 9 (DSAR) + backups/replicas/third-parties.

  "/privacyaudit what data do we send to third parties"
  -> SHARING-FOCUSED: Phase 4 (third-party sharing) + Phase 5 (cross-border) + perimeter tracing.

  "/privacyaudit gdpr"
  -> GDPR surface emphasis (lawful basis, DSAR, erasure, cross-border, DPA/sub-processors).

  "/privacyaudit ccpa"
  -> CCPA/CPRA surface emphasis (notice-at-collection, opt-out of sale/share, "Do Not Sell" link).

  "/privacyaudit the privacy policy is out of date"
  -> RECONCILIATION-FOCUSED: Phase 8 (policy vs reality) at max depth, cross-checked against all flows.

RULES:
- If specific files/dirs mentioned: scope to those (--files=).
- If a concern described: focus on relevant phases, but ALWAYS run Phase 1 (PII inventory) first — you can't audit what you haven't inventoried.
- If "all"/"everything"/"full": all phases, full depth.
- If audits/.privacyaudit/fix-plan.json exists and no new scope: resume fixing.
- Parse the intent, don't ask for clarification (Third Law).
```

---

## OUTPUT CONTRACT — Omega Integration

```
audits/.privacyaudit/
|-- session.log
|-- discovery/
|   |-- pii-inventory.json         # Every PII field, its location, sensitivity, lawful basis
|   |-- data-flow-map.json         # End-to-end PII flows (entry -> hops -> exit)
|   |-- third-parties.json         # Sub-processors / external destinations of PII
|   |-- consent-map.json           # Where consent is captured, stored, withdrawn
|   |-- policy-claims.json         # Extracted claims from the privacy policy
|-- reports/
|   |-- pii-inventory.md             # Phase 1
|   |-- consent.md                   # Phase 2
|   |-- retention-deletion.md        # Phase 3
|   |-- third-party-sharing.md       # Phase 4
|   |-- cross-border.md              # Phase 5
|   |-- cookies-tracking.md          # Phase 6
|   |-- encryption.md                # Phase 7
|   |-- policy-vs-reality.md         # Phase 8
|   |-- dsar.md                      # Phase 9
|   |-- data-minimization.md         # Phase 10
|   |-- childrens-data.md            # Phase 11
|   |-- logging-leakage.md           # Phase 12
|   |-- breach-readiness.md          # Phase 13
|-- baseline/                       # Phase N-1 pre-fix baselines
|-- before-after.md                 # Phase N+4 matrix (mandatory)
|-- verdict.json
|-- verdict.md
|-- fix-plan.json
|-- fix-plan.md
|-- progress.json
|-- telemetry.json
|-- fix-log.md
```

**CRITICAL:** `progress.json` is read by the Telegram bot monitor for live progress cards.
Format: `{"total": 31, "done": 8, "failed": 0, "skipped": 1, "remaining": 22, "current": "FIX-009 — add TTL cron for analytics_events"}`

**CRITICAL:** `fix-plan.json` is read by oracles to resume interrupted audits.
Format: `{"tasks": [{"id": "FIX-001", "finding": "...", "file": "...", "line": 42, "fix": "...", "status": "pending|done|failed|skipped", "severity": "CRITICAL|HIGH|MEDIUM|LOW"}]}`

---

## PHASE 0 — PROGRAMMATIC GATHER (HYBRID, runs FIRST, before all other phases)

> **Hybrid framework:** before any LLM analysis, programmatic tools gather every
> machine-checkable finding deterministically. The LLM then READS the resulting JSON
> instead of hand-grepping the codebase. Freed token budget is REINVESTED in deeper
> Popper falsification, hinge-flow synthesis, user-need verification, and edge-case hunting.

### 0.1 Run the gather script (mandatory, FIRST step)

```bash
~/.omega/lib/audit-runner.sh privacy "$PROJECT_PATH" \
  --files="$FILES_MODIFIED" \
  --url="$URL" \
  --user-need="$USER_NEED_QUOTE" \
  --hinge="$HINGE_POINT" \
  --ticket="$TICKET_ID"
```

This invokes the privacy gather, which runs (or, on a blank VPS, falls back to portable greps):
PII-pattern scanner (email/phone/SSN/credit-card/IP/geo/DOB regex census over schema + code),
gitleaks (PII/secrets in repo + git history), cookie/tracker scanner over built HTML/JS (analytics,
ad pixels, fingerprinting libs), `grep` census of third-party SDK imports, schema column-name
classifier, retention-job detector (cron/TTL/cleanup scan), `.env` + transport (HTTP-vs-HTTPS) probe.

Output:

```
$PROJECT_PATH/audits/.privacyaudit/
├── raw/                    # raw tool outputs (JSON / text per tool)
└── evidence-summary.json   # normalized findings, single source of truth for the LLM
```

When run inside a Linear-fix mission (`--ticket=ID`), artifacts move to
`$PROJECT_PATH/audits/.linear-fix/<ID>/.privacyaudit/` so sibling audits can cross-reference (see 0.5).

### 0.2 evidence-summary.json schema

```jsonc
{
  "audit": "privacy",
  "tools_run": ["..."],
  "tools_skipped": [{"tool": "...", "reason": "..."}],
  "findings_total": 0,
  "findings_by_severity": {"critical": 0, "high": 0, "medium": 0, "low": 0, "info": 0},
  "findings": [
    {
      "tool": "...",
      "severity": "critical|high|medium|low|info",
      "location": "file:line[:col]",
      "rule": "...",
      "message": "...",
      "pii_category": "identity|contact|financial|health|biometric|location|behavioral|credentials|children",
      "suggested_fix": "...",
      "cross_tool_confirmed": false
    }
  ],
  "metrics": { /* pii field count by category, third-party count, cookies set pre-consent, etc. */ },
  "evidence_index": { /* paths to raw/ files for drill-down */ }
}
```

### 0.3 What you do AFTER the gather (this replaces hand-greps)

1. **Read `evidence-summary.json` in full.** This is your evidence base.
2. **Read the privacy policy** (look for `privacy`, `policy`, `legal`, `gdpr`, `ccpa` in routes/docs/markdown) and the data schema (Convex `schema.ts`, Prisma `schema.prisma`, SQL migrations).
3. **DO NOT re-run the PII/cookie/tracker scans the gather already did.** Read the JSON.
4. **DO read additional files** when (a) a finding's context is unclear, (b) you need to verify a Popper falsification, or (c) you suspect a missed PII flow (Phase H1.4).

### 0.4 Banned operations after Phase 0

These are forbidden because the gather already did them. If you catch yourself about to run one, STOP and read `evidence-summary.json`:

- ❌ `grep -rn "email\|phone\|ssn" .` (the gather ran the PII census)
- ❌ blanket `find . -name "*.ts" | xargs grep ...` for trackers (already in raw/)
- ❌ `gitleaks detect` (the gather ran it — read the JSON)
- ❌ Generic "let me read every file" loops

You MAY still:
- ✅ Read SPECIFIC files cited in findings (verify the issue)
- ✅ Run a SPECIFIC `grep` to falsify a finding (Popper test)
- ✅ Run a SPECIFIC probe the gather couldn't (e.g. Playwright CLI loading the prod URL to observe which cookies/network calls fire BEFORE consent is given)

### 0.5 Cross-audit synthesis (read sibling evidence-summary.json files)

If part of a Linear-fix mission, sibling summaries are at
`$PROJECT_PATH/audits/.linear-fix/<TICKET>/.<other-audit-id>/evidence-summary.json`. Read them. Use them.

High-value privacy confluences:
- **privacyaudit + secaudit** flag the same PII column → it's BOTH unencrypted (privacy) AND a breach target (security). Escalate.
- **privacyaudit + dataaudit** on the same table → retention/erasure (privacy) meets orphaned-record/TTL (data integrity); joint fix.
- **privacyaudit + apiaudit** on the same endpoint → an endpoint returning PII without authz is an IDOR (sec) AND an unlawful disclosure (privacy).
- **privacyaudit + perfaudit** on a third-party script → the tracker is both a perf cost and a privacy leak; one removal fixes both.

Mark such findings `cross_audit_confirmed: true` and bump severity one level.

---

## PHASE 0b: RECONNAISSANCE & HINGE DATA FLOW

> *"Map every place a person becomes a row before you judge how the rows are kept."*

```
1. PRODUCT & DATA INTENT
   -> Read VISION.md / README / CLAUDE.md: what is the product, who are the users, what jurisdictions?
   -> Identify which regimes apply: GDPR (EU/UK users), CCPA/CPRA (California), COPPA (under-13), HIPAA (US health), LGPD/PIPEDA if claimed.
   -> Locate the privacy policy text (route, markdown, or external URL). If NONE exists -> immediate HIGH finding.

2. DATA-COLLECTION SURFACE
   -> Every signup/profile/checkout/upload form (fields collected).
   -> Every API endpoint that accepts user data.
   -> Every passive collector: analytics, cookies, server logs, IP capture, device fingerprint, session replay.

3. DATA-STORAGE SURFACE
   -> Primary DB tables/collections, replicas, search indexes (Algolia/Elastic), caches (Redis), object storage (S3), backups.

4. DATA-EXIT SURFACE (perimeter)
   -> Third-party APIs (payment, email, SMS, LLM, CRM, support), analytics/ad pixels, webhooks, exports, logs shipped off-box.

5. HINGE DATA FLOW IDENTIFICATION
   -> Pick THE most sensitive PII (financial > health > biometric > children > precise location > identity > contact > behavioral).
   -> Trace it end-to-end: entry point -> validation -> transformation -> primary store -> replication -> backup -> analytics -> third party -> logs -> exit.
   -> This flow gets 10x scrutiny in every applicable phase. If it leaks anywhere, the product fails the audit.
```

---

## PHASE 1: PII INVENTORY & CLASSIFICATION

> *"You cannot protect what you have not named. Every field, classified, or you are flying blind."*

```
1. FIELD-LEVEL CENSUS (100% coverage of schema + collected inputs)
   FOR EVERY persisted field and every collected input:
   -> Classify into PII category:
      identity (name, username, gov ID, SSN), contact (email, phone, address),
      financial (card, IBAN, transaction), health (diagnosis, prescription, fitness),
      biometric (face, fingerprint, voiceprint), location (GPS, IP-derived geo),
      behavioral (clicks, watch history, search), credentials (password hash, tokens),
      children (any of the above for users known/likely under 13/16).
   -> Mark NON-PII fields explicitly (id, created_at, feature flags) so the inventory is exhaustive, not selective.

2. SENSITIVITY TIERING
   -> SPECIAL CATEGORY (GDPR Art.9): health, biometric, race, religion, sexual orientation, political, union membership -> highest tier, needs explicit consent or specific exemption.
   -> HIGH: financial, gov ID, precise location, children's data.
   -> MEDIUM: contact, behavioral profiles.
   -> LOW: pseudonymous identifiers, coarse aggregates.

3. LAWFUL BASIS PER FIELD (GDPR Art.6)
   FOR EACH PII field, identify the claimed basis:
   -> consent / contract / legal obligation / vital interest / public task / legitimate interest.
   -> FALSIFY: is the basis defensible? "Legitimate interest" for selling data to advertisers is NOT defensible. Consent that's bundled/forced is NOT valid.
   -> Field with NO identifiable lawful basis = CRITICAL finding (you're holding data you can't justify).

4. PROVENANCE
   -> Where did each field come from? Direct from user, derived/inferred, purchased/enriched from a data broker (high risk), or inherited from import?
   -> Inferred sensitive data (e.g. pregnancy inferred from purchases) is STILL special-category data.

5. PII SPRAWL DETECTION
   -> Same PII duplicated across tables/services/caches/logs (each copy = independent breach + erasure target).
   -> PII in places it shouldn't be: URLs (referer leak), JWT claims, client localStorage, analytics event properties, error messages.

Output: discovery/pii-inventory.json — every field, category, tier, lawful basis, locations[].
SCORE: 0 = fields with no lawful basis + special-category data uncontrolled, 5 = inventory partial / some bases unclear, 8 = full inventory with defensible bases, 10 = full inventory + tiering + minimal sprawl + every basis defensible.
```

---

## PHASE 2: CONSENT CAPTURE & WITHDRAWAL

> *"A consent you cannot withdraw is not consent. It is a trap with a checkbox."*

```
1. CONSENT CAPTURE
   -> Is consent collected BEFORE the processing it authorizes? (consent after the fact = invalid)
   -> Is it freely given (not bundled with ToS, not a precondition for unrelated service)?
   -> Is it specific (per-purpose: analytics vs marketing vs personalization, not one blanket "I agree")?
   -> Is it unambiguous (affirmative action — NO pre-ticked boxes, NO "by using this site you consent")?
   -> Is it informed (the user saw what they were consenting to, linked to the actual policy version)?

2. CONSENT RECORD (proof)
   FALSIFY "we got consent":
   -> Is there a stored record? (user_id, purpose, granted/denied, timestamp, policy_version, source_ip/ua)
   -> Can you reconstruct WHAT the user agreed to and WHEN? (versioned policy text)
   -> No durable consent record = you cannot prove consent = legally you have none.

3. WITHDRAWAL MECHANICS
   -> Is withdrawing consent as easy as giving it? (GDPR Art.7(3))
   -> Does withdrawal ACTUALLY stop the processing? (trace: toggle off -> does the analytics SDK actually stop firing? does the marketing job actually skip this user?)
   -> Is prior data processed under the now-withdrawn consent deleted or retained? (must stop future processing; past may need erasure depending on basis)

4. GRANULARITY & RE-CONSENT
   -> Separate toggles per purpose, or all-or-nothing? (all-or-nothing fails "specific")
   -> When the policy materially changes, is re-consent requested? Or is stale consent reused for new purposes?

5. CCPA/CPRA OPT-OUT (parallel for California)
   -> Is there a "Do Not Sell or Share My Personal Information" link? Honored?
   -> Is Global Privacy Control (GPC) signal respected?
   -> Opt-out must NOT require an account or excessive verification.

FALSIFY each: don't check that the toggle EXISTS — flip it and prove the processing actually stops (Playwright: accept-then-withdraw, observe network).
SCORE: 0 = pre-ticked/bundled consent or processing-before-consent or no withdrawal, 3 = consent captured but not recorded, 5 = recorded but withdrawal doesn't propagate, 8 = granular + recorded + propagates, 10 = + versioned re-consent + GPC honored + withdrawal == granting effort.
```

---

## PHASE 3: DATA RETENTION & DELETION (RIGHT TO ERASURE)

> *"'We delete on request' is a promise. Show me the line of code that keeps it."*

```
1. RETENTION POLICY vs ENFORCEMENT
   FALSIFY the policy's retention claims:
   -> Policy says "X days/months". Is there a TTL, cron, scheduled cleanup, or lifecycle rule that ACTUALLY enforces X?
   -> No enforcement mechanism -> data kept forever -> the policy is a lie -> HIGH/CRITICAL.
   -> Per-category retention (logs vs transactions vs marketing) or one blanket rule?

2. ERASURE PATH (right to be forgotten — GDPR Art.17, CCPA delete)
   Trace the account-deletion / erase-my-data flow ACROSS EVERY STORE:
   -> Primary DB: hard delete or soft delete (deleted_at)? Soft delete alone does NOT satisfy erasure.
   -> Replicas / read models / search indexes: purged?
   -> Caches (Redis/CDN): invalidated?
   -> Object storage (uploads, avatars, exports): deleted?
   -> Backups: documented exclusion/expiry path? (backups are the #1 forgotten erasure gap)
   -> THIRD PARTIES: is a deletion request propagated to every sub-processor that received the PII? (Stripe, email, analytics, LLM logs)
   -> Logs: are PII-bearing log lines purged or anonymized?

3. ERASURE COMPLETENESS PROOF
   -> After "delete account", can you still SELECT the user's PII anywhere? (run the query)
   -> Are foreign-key references that re-expose PII (e.g. orders.customer_name copied) also handled?

4. ANONYMIZATION vs PSEUDONYMIZATION
   -> If data is "anonymized" instead of deleted, is it TRULY anonymous (irreversible, no re-identification via joins)? Pseudonymization (reversible) is still personal data.

5. DELETION TIMELINES & DEAD-MAN
   -> GDPR: erasure "without undue delay" (≈30 days). CCPA: 45 days. Is there an SLA and does the job meet it?
   -> Dormant-account purge: are abandoned accounts' PII eventually deleted, or kept indefinitely?

FALSIFY: actually exercise the erase path on a test user (or read it line-by-line) and grep every store for the PII afterward.
SCORE: 0 = no erasure path or soft-delete only, 3 = erases primary DB only, 5 = erases DB+caches but not backups/third-parties, 8 = covers all first-party stores + propagates to third parties, 10 = + enforced retention TTLs + proven irreversible anonymization + SLA met.
```

---

## PHASE 4: THIRD-PARTY DATA SHARING & SUB-PROCESSORS

> *"Every byte you ship to a vendor extends your breach to their infrastructure. Map the perimeter."*

```
1. SUB-PROCESSOR INVENTORY (100%)
   FOR EVERY external destination of PII:
   -> Payment (Stripe), email (SendGrid/Resend), SMS (Twilio), auth (Clerk/Auth0), analytics (GA/PostHog/Mixpanel),
      error tracking (Sentry), LLM APIs (OpenAI/Anthropic), CRM, support (Intercom), ad pixels, CDNs receiving PII in URLs.
   -> For each: WHAT PII is sent? Under what basis? Is there a Data Processing Agreement (DPA)?

2. WHAT ACTUALLY LEAVES (FALSIFY "we only send X")
   -> Trace the actual payload to each third party. Sentry breadcrumbs leaking emails? LLM prompt containing the user's full record? Analytics event carrying user_id + IP + page (= behavioral profile)?
   -> PII in URLs to third parties (referer / query string) = leak.

3. DATA-PROCESSING AGREEMENTS & SUB-PROCESSOR DISCLOSURE
   -> Does the privacy policy LIST sub-processors (GDPR transparency / CCPA "categories of third parties")?
   -> New sub-processor added in code but NOT in the policy = undisclosed sharing = finding.

4. SALE / SHARE vs SERVICE-PROVIDER (CCPA/CPRA)
   -> Is any PII transfer a "sale" or "share" for cross-context behavioral advertising? (ad pixels usually are)
   -> If yes: is opt-out honored (Phase 2.5)? Service-provider contracts limit downstream use — are they in place?

5. ONWARD TRANSFER / DATA BROKERS
   -> Is PII sold/shared with data brokers or enrichment services? (highest risk)
   -> Any "audience" / "lookalike" exports to ad platforms? = sale under CPRA.

SCORE: 0 = undisclosed PII sale or PII shipped to vendors with no DPA, 3 = vendors used but not disclosed, 5 = disclosed but over-sends PII, 8 = minimal PII per vendor + DPAs + disclosed, 10 = + opt-out honored for sale/share + sub-processor list versioned + onward-transfer controls.
```

---

## PHASE 5: CROSS-BORDER TRANSFER SURFACE

> *"Data has a passport problem. EU data on a US server is a transfer, and transfers have rules."*

```
1. WHERE DOES THE DATA PHYSICALLY LIVE?
   -> DB region, backup region, CDN edge locations, third-party processor regions (Stripe US, OpenAI US, etc.).
   -> If EU/UK user PII is processed/stored outside EEA/UK -> a restricted transfer is occurring.

2. TRANSFER MECHANISM
   -> Is there a valid basis for the transfer? (EU-US Data Privacy Framework certification, Standard Contractual Clauses (SCCs), adequacy decision, or explicit consent)
   -> Sending EU PII to a US LLM/analytics vendor with no SCC/DPF = unlawful transfer.

3. DATA RESIDENCY CLAIMS (FALSIFY)
   -> Policy/marketing claims "EU data stays in EU" / "data residency" -> verify the actual DB region and every third party's region.
   -> A residency claim contradicted by a US-region processor = false claim + unlawful transfer.

4. LOCALIZATION REQUIREMENTS
   -> Any jurisdiction requiring local storage (e.g. certain health/financial data)? Met?

SCORE: 0 = unlawful transfer of special-category/EU PII with no mechanism, 3 = transfers occur, mechanism unclear, 5 = SCCs/DPF for some but not all vendors, 8 = all transfers covered by a valid mechanism, 10 = + verified residency claims + documented transfer impact assessment.
```

---

## PHASE 6: COOKIE & TRACKING-TECHNOLOGY COMPLIANCE

> *"The cookie banner is theatre if the trackers already fired. Watch the network, not the banner."*

```
1. COOKIE / STORAGE CENSUS
   -> Enumerate ALL cookies, localStorage, sessionStorage, IndexedDB keys set by the app + third parties.
   -> Classify each: strictly-necessary / functional / analytics / marketing / fingerprinting.

2. PRIOR CONSENT (ePrivacy / GDPR) — THE CRITICAL FALSIFICATION
   -> Load the prod URL fresh (Playwright CLI, no MCP), incognito, BEFORE clicking the banner.
   -> Observe: which cookies are set and which tracker network calls fire BEFORE consent?
   -> Non-strictly-necessary cookies/trackers firing pre-consent = VIOLATION (the banner is decorative).
   -> "Reject all" must be as prominent/one-click as "Accept all" (dark patterns fail).

3. CONSENT MODE / TAG GATING
   -> Are analytics/ad tags actually GATED behind the consent state, or loaded unconditionally with the banner just hiding the UI?
   -> Google Consent Mode / TCF string present and honored?

4. FINGERPRINTING & COVERT TRACKING
   -> Canvas/WebGL/font/audio fingerprinting libraries? (these need consent and are often undisclosed)
   -> Session-replay tools (FullStory/Hotjar/LogRocket) capturing keystrokes/PII? Masked?
   -> Tracking pixels (Meta/TikTok/LinkedIn) firing on every page?

5. COOKIE LIFETIME & DISCLOSURE
   -> Cookie max-age reasonable per purpose? (a 2-year analytics cookie is excessive)
   -> Does the cookie policy/banner accurately list the cookies actually set? (FALSIFY: compare banner list to observed cookies)

SCORE: 0 = trackers fire pre-consent / fingerprinting undisclosed, 3 = banner present but tags not gated, 5 = gated but reject is a dark pattern or list inaccurate, 8 = prior consent + symmetric accept/reject + accurate list, 10 = + consent mode honored + GPC respected + minimal cookie lifetimes.
```

---

## PHASE 7: ENCRYPTION OF PII (AT REST & IN TRANSIT)

> *"Encryption is the difference between a lost laptop and a notifiable breach."*

```
1. IN TRANSIT
   -> All PII-carrying endpoints over HTTPS/TLS? Any HTTP fallback, mixed content, or internal service-to-service plaintext?
   -> WebSocket using WSS? Database connections using TLS? Backups transferred over TLS?
   -> HSTS present so PII is never sent over HTTP even once?

2. AT REST — DATABASE
   -> Is the datastore encrypted at rest (provider-level disk encryption)?
   -> Are SPECIAL-CATEGORY/HIGH fields additionally encrypted at the FIELD/COLUMN level (app-layer or DB-native), so a DB dump doesn't expose them in plaintext?
   -> FALSIFY "encrypted at rest": inspect a row/dump for the sensitive column — plaintext = claim is false.

3. AT REST — BACKUPS, LOGS, OBJECT STORAGE, CACHES
   -> Backups encrypted? Object storage (S3 buckets) encrypted AND not public?
   -> Caches/Redis holding PII encrypted or at least access-controlled + TTL'd?

4. KEY MANAGEMENT
   -> Encryption keys in a KMS/secret manager, NOT hardcoded or in .env committed to git?
   -> Key rotation possible? Separation between data and keys (a dump of the DB shouldn't include the key)?

5. CREDENTIAL & SECRET STORAGE
   -> Passwords hashed with bcrypt/argon2 (not MD5/SHA1/plaintext)?
   -> API tokens / refresh tokens stored hashed or encrypted, not plaintext?

6. CRYPTO HYGIENE (cross-ref secaudit A02)
   -> No homegrown crypto, no ECB mode, no static IVs, no deprecated algorithms for PII protection.

SCORE: 0 = PII in plaintext over HTTP or plaintext special-category at rest, 3 = TLS but no at-rest encryption, 5 = disk encryption only (no field-level for sensitive), 8 = TLS + field-level for sensitive + encrypted backups, 10 = + KMS-managed keys + rotation + hashed credentials + verified no-plaintext-in-dump.
```

---

## PHASE 8: PRIVACY POLICY vs REALITY (RECONCILIATION)

> *"The policy is what your lawyers promised. The code is what your servers do. Find every divergence."*

```
1. EXTRACT EVERY CLAIM
   Parse the privacy policy into discrete, testable claims:
   -> "We collect X, Y, Z" / "We do NOT collect W"
   -> "We retain for N days" / "We delete on request"
   -> "We share with [list]" / "We do not sell your data"
   -> "We encrypt your data" / "Data stored in [region]"
   -> "You can access/export/delete your data" / "Contact dpo@..."
   -> "We use cookies for [purposes]"
   Save to discovery/policy-claims.json.

2. RECONCILE EACH CLAIM AGAINST CODE (FALSIFY each)
   FOR EACH claim, find the code evidence that confirms or refutes it:
   -> "We don't collect location" but there's a `lat`/`lng` column or geo-IP call -> CONTRADICTION.
   -> "We delete on request" but no erase endpoint (Phase 3) -> CONTRADICTION.
   -> "We don't sell data" but an ad pixel ships behavioral data (Phase 4/6) -> CONTRADICTION.
   -> "Data in EU" but DB region is us-east-1 (Phase 5) -> CONTRADICTION.
   -> "Encrypted" but plaintext column (Phase 7) -> CONTRADICTION.
   Each contradiction is a finding whose severity = the sensitivity of the data involved.

3. UNDISCLOSED PROCESSING (reverse direction)
   -> Things the CODE does that the policy does NOT mention: a new analytics SDK, a new third party, a new collected field. Undisclosed processing = transparency violation.

4. POLICY HYGIENE
   -> Last-updated date present and recent? Contact/DPO listed? Lawful bases stated? Data-subject rights described? Children's policy if applicable?
   -> Generic boilerplate that doesn't match the actual product = red flag.

5. NOTICE-AT-COLLECTION (CCPA)
   -> Is notice given at or before the point of collection (e.g. at the form), not only buried in the policy?

SCORE: 0 = policy materially contradicts code (e.g. "no sale" while selling), 3 = several contradictions, 5 = minor gaps + undisclosed minor processing, 8 = policy matches code with small omissions, 10 = every claim code-verified + no undisclosed processing + policy hygiene complete.
```

---

## PHASE 9: DATA-SUBJECT ACCESS REQUESTS (DSAR) & RIGHTS

> *"A right the user can't exercise is a right that doesn't exist."*

```
1. RIGHT TO ACCESS / PORTABILITY
   -> Can a user obtain a copy of ALL their data? Is the export complete (every store from Phase 1, not just the profile table)?
   -> Is it machine-readable / portable (JSON/CSV) per GDPR Art.20?
   -> Identity verification before fulfilling (so attacker can't DSAR someone else's data) — but NOT excessive friction.

2. RIGHT TO RECTIFICATION
   -> Can the user correct inaccurate PII? Does the correction propagate to copies/caches/third parties?

3. RIGHT TO ERASURE
   -> (Cross-ref Phase 3 — the actual deletion mechanics.) Here: is there a USER-FACING way to request it, with an SLA?

4. RIGHT TO OBJECT / RESTRICT
   -> Can the user object to processing (e.g. profiling, marketing) and is it honored?

5. AUTOMATED DECISION-MAKING (GDPR Art.22)
   -> Any solely-automated decisions with legal/significant effect (credit, eligibility, content moderation that bans)? Is there human-review/appeal + explanation?

6. REQUEST INTAKE & TRACKING
   -> Is there a channel (form/email/in-app) to submit rights requests? Is it logged and tracked to SLA (30 days GDPR / 45 days CCPA)?
   -> If everything is manual with no record, you cannot prove compliance.

SCORE: 0 = no way to access/delete data, 3 = manual + incomplete export, 5 = self-serve export but misses some stores, 8 = complete export + rectify + erase + SLA tracking, 10 = + portability format + objection/restriction + Art.22 safeguards.
```

---

## PHASE 10: DATA MINIMIZATION & PURPOSE LIMITATION

> *"The safest byte is the one you never collected. Justify every field's existence."*

```
1. COLLECTION JUSTIFICATION (FALSIFY necessity)
   FOR EACH collected PII field (from Phase 1):
   -> Is it actually USED anywhere? A field collected but never read = collected without purpose = delete it.
   -> Is the FULL field needed, or would less suffice? (DOB when only age/18+ matters; precise GPS when city suffices; full name when first-name suffices)
   -> "Collect now, might need later" is NOT a lawful purpose.

2. PURPOSE LIMITATION
   -> Was data collected for purpose A now used for purpose B? (e.g. email collected for receipts now used for marketing) -> purpose creep, needs separate basis/consent.

3. EXCESSIVE PRECISION / GRANULARITY
   -> Storing full IP when a truncated/hashed IP suffices for analytics?
   -> Storing raw biometric when a non-reversible template suffices?
   -> Behavioral logs at event-level forever when aggregates suffice?

4. RETENTION MINIMIZATION (cross-ref Phase 3)
   -> Is data kept only as long as the purpose requires, then deleted/anonymized?

5. DEFAULT PRIVACY (privacy by design/default — GDPR Art.25)
   -> Are the most privacy-protective settings the DEFAULT? (profile private by default, analytics opt-IN not opt-out where consent is required)
   -> Are optional fields actually optional in the form, or forced?

SCORE: 0 = collecting sensitive data with no use / forced optional fields, 3 = unused fields collected, 5 = collected-but-too-precise, 8 = each field justified + reasonable precision, 10 = + purpose limitation enforced + privacy-by-default + retention minimized.
```

---

## PHASE 11: CHILDREN'S DATA (COPPA / AGE-APPROPRIATE DESIGN)

> *"If a child can sign up, the law treats every shortcut as negligence."*

```
1. AUDIENCE DETERMINATION
   -> Is the service directed at children, or likely to attract under-13 (COPPA) / under-16 (GDPR default) users?
   -> Is there ANY age signal collected (DOB, grade, "are you over 18")?

2. AGE GATING
   -> If children are in scope: is there a neutral age gate (not "you must be 18" which just trains lying)?
   -> Is collection blocked / parental consent required for under-age users?

3. PARENTAL / VERIFIABLE CONSENT (COPPA)
   -> For under-13: verifiable parental consent before collecting PII? Mechanism present?

4. MINIMIZED COLLECTION FOR MINORS
   -> No behavioral advertising to children. No unnecessary PII. No nudging children to disclose more (age-appropriate design code).

5. DEFAULTS FOR MINORS
   -> High-privacy defaults, geolocation off, profiling off, contact restrictions for known/likely minors.

If the product is clearly adult-only with enforced age gating and NO child data, mark this phase N/A with justification (excluded from normalized denominator per preamble §5) — but you must PROVE under-13 cannot realistically register, not just assume.
SCORE: 0 = collects child PII with no consent/gating, 3 = age asked but not enforced, 5 = gated but minors over-collected, 8 = gating + parental consent + minimized, 10 = + age-appropriate defaults + no profiling of minors. (N/A if proven out of scope.)
```

---

## PHASE 12: LOGGING & TELEMETRY PII LEAKAGE

> *"Your logs are forever, world-readable to your whole team, and shipped to a US SaaS. What's in them?"*

```
1. LOG CONTENT SCAN
   -> Do application/access/error logs contain raw PII? (full email, name, address, card, token, full request body with PII)
   -> Are request/response bodies logged verbatim on errors? (leaks PII into log aggregator + error tracker)

2. TELEMETRY / ANALYTICS EVENT PROPERTIES
   -> Do analytics events carry PII in their properties (email as user trait, full URL with PII query params)?
   -> Is user identification pseudonymous (hashed id) or raw (email as the distinct_id)?

3. ERROR TRACKING (Sentry et al.)
   -> Are PII scrubbers configured? Breadcrumbs/local-variable capture leaking PII? Headers (Authorization, Cookie) sent to the tracker?

4. LOG RETENTION & ACCESS
   -> How long are logs kept? (PII in 2-year logs = retention violation)
   -> Who can read them? Are PII-bearing logs access-controlled and excluded from erasure-exempt "legitimate interest" only where justified?

5. THIRD-PARTY LOG DESTINATIONS
   -> Logs shipped to Datadog/Loki/CloudWatch — are these in-scope for cross-border (Phase 5) and DPA (Phase 4)?

FALSIFY "we don't log PII": grep the actual logging calls and read a sample of real log output if available.
SCORE: 0 = raw PII (incl. credentials/cards) in logs/telemetry, 3 = PII in error tracker, 5 = PII in analytics traits, 8 = scrubbed logs + hashed ids + short retention, 10 = + verified no-PII-in-samples + access-controlled + log destinations under DPA.
```

---

## PHASE 13: BREACH-NOTIFICATION READINESS

> *"The breach is not the worst part. Discovering you had no plan to report it is."*

```
1. DETECTION
   -> Can a breach even be detected? (audit logs on PII access, anomaly alerts, access logging on sensitive tables)
   -> Is unauthorized PII access logged and alertable? (cross-ref secaudit A09)

2. NOTIFICATION CAPABILITY
   -> Could you, within 72 hours (GDPR Art.33), determine WHOSE data and WHICH fields were exposed? (requires the PII inventory + access logs to exist)
   -> Is there a documented incident-response runbook? A DPO / responsible contact?

3. SCOPE-OF-IMPACT QUERYABILITY
   -> Given "table X was exfiltrated", can you produce the list of affected data subjects to notify them? (Phase 1 inventory makes this possible; sprawl makes it impossible)

4. RECORDS OF PROCESSING (GDPR Art.30)
   -> Is there a Record of Processing Activities (categories of data, purposes, recipients, retention, transfers)? The PII inventory (Phase 1) is the technical backbone of this.

5. AUDIT TRAIL INTEGRITY
   -> Are access/audit logs tamper-evident and retained long enough to investigate, but PII within them still minimized (Phase 12)?

SCORE: 0 = no access logging, breach undetectable, unknowable scope, 3 = some logging but no IR plan, 5 = logging + plan but scope not queryable, 8 = detectable + notifiable within 72h + RoPA exists, 10 = + tamper-evident trails + rehearsed runbook + automated impact-scoping.
```

---

## PHASE H1 — HYBRID SYNTHESIS (Popper / hinge / user-need / edge cases / cross-audit)

> Runs immediately before VERDICT. "H1" sits between the last domain phase (13) and the
> VERDICT phase; it does NOT renumber earlier phases. The token budget freed by Phase 0's
> deterministic gather is REINVESTED here — depth increases, nothing is skipped.

### H1.1 Popper falsification per finding (mandatory)

For every finding in `evidence-summary.json.findings[]` (start with `severity ∈ {critical, high}`), try to PROVE the tool is wrong. Each produces a `falsifiable_tests[]` entry in `verdict.json`:

```jsonc
{
  "claim": "PII scanner says users.dob is collected but never used (no purpose)",
  "test_command": "grep -rn 'dob\\|dateOfBirth\\|date_of_birth' --include='*.ts' --include='*.tsx' . | grep -v 'schema\\|migration'",
  "expected": "0 read sites → claim TRUE, field is collected without purpose",
  "actual": "0 read sites found",
  "outcome": "confirmed"
}
```

Outcomes: `confirmed` (test failed to falsify → finding stands), `falsified` (counter-example found → demote to info + add `falsified_at`), `inconclusive` (couldn't run cleanly → keep severity, `confidence: medium`).

**The rule:** every CLAIM (PASS or FAIL) MUST cite ≥3 concrete commands that COULD have falsified it but didn't. Banned phrases (`looks correct`, `should be fine`, `appears to work`) → automatic FAIL.

Common privacy falsification patterns:

| Claim | Popper test |
|---|---|
| "field collected without purpose" (minimization) | `grep` every read site incl. exports, analytics traits, templates, third-party payloads |
| "no deletion path" (erasure) | Read the delete handler; grep every store (DB, cache, S3, search) for the PII post-delete |
| "encrypted at rest" | Inspect an actual row/dump for the sensitive column — plaintext refutes the claim |
| "consent gates the tracker" | Playwright CLI: load prod fresh, observe network BEFORE accepting banner |
| "we don't share with third parties" | grep third-party SDK imports + inspect outbound payloads + scan built JS for pixels |
| "EU data stays in EU" (residency) | Check DB region config + every processor's region |
| "policy says we collect only X" | Reconcile against PII inventory (Phase 1) — any extra field refutes it |

### H1.2 Hinge cross-reference (10× scrutiny on the HINGE DATA FLOW)

For each finding, mark `is_load_bearing: true` IFF its file/field is part of the HINGE DATA FLOW (Phase 0b). Apply 10× scrutiny:
- 5× more falsification attempts (H1.1)
- 3× more edge-case hunts (H1.4)
- Mandatory read of the ENTIRE flow (every hop), all writers and all readers of the hinge field, every third-party that receives it.

Output `hinge_findings[]` in `verdict.json` (finding_id, is_load_bearing, hinge_reference, additional_scrutiny, confidence_after_scrutiny).

### H1.3 User-need verification (`--user-need` quote)

Every finding evaluated against the verbatim user-need. For each: "If a user reported THIS verbatim, would this finding be the cause?" and "Does fixing it make the user-need quote no longer true?" Findings unrelated to user-need get demoted one level UNLESS load-bearing; flagged `user_need_relevance: "tangential"`. Related findings get top fix priority and lead `user_need_match.findings[]`. If `addressed: false`, the audit MUST score below 90/100.

### H1.4 Edge-case hunting (mandatory for top findings)

For each top-5 finding, generate ≥2 privacy edge cases the static scan missed:
- "User withdraws consent mid-session — does the already-loaded analytics SDK keep firing?"
- "User deletes account, but a nightly job re-imports them from a stale backup/CRM."
- "Erasure runs on primary but the read-replica / search index still serves the PII for hours."
- "i18n: the privacy banner exists in English but trackers fire unconsented on the non-default-locale page."
- "PII enters via an export/CSV import path that bypasses the consent + minimization checks."
- "A 12-year-old completes signup because the age gate is client-side only."

Output `edge_cases[]` (finding_id, scenario, covered_by_existing_test, evidence_gathered, fix_includes_coverage).

### H1.5 Cross-audit synthesis

Re-read sibling summaries (Phase 0.5). For each of YOUR top-5: same file/field flagged by a sibling (esp. secaudit, dataaudit, apiaudit)? If yes → `cross_audit_confirmed: true`, bump severity one level. Add relevant sibling findings to YOUR domain as `tool: "cross-audit:<sibling>"`. Write `cross_audit_links[]`.

### H1.6 Final verdict.json schema (hybrid v2)

```jsonc
{
  "audit": "privacy",
  "score": 100,
  "score_raw": "<raw>/360",
  "score_normalized": 100,
  "confidence": "high|medium|low",
  "skill_used": "privacy",
  "preamble_version": "1.0",
  "user_need_match": { },                 // H1.3
  "falsifiable_tests": [ ],               // H1.1
  "hinge_findings": [ ],                  // H1.2
  "issues_found_and_fixed": [
    { "id": "FIX-001", "finding_id": "F-007", "pii_category": "contact",
      "before": "<state>", "after": "<state>", "verification": "<command + output>" }
  ],
  "edge_cases": [ ],                      // H1.4
  "cross_audit_links": [ ],              // H1.5
  "evidence_summary_path": "$PROJECT_PATH/audits/.privacyaudit/evidence-summary.json",
  "confidence_basis": "Cite Popper test counts, hinge scrutiny depth, edge-case coverage, cross-audit confirmations.",
  "banned_phrase_check": "passed (no `looks correct`, `should be fine`, `appears to work`, `streamlined`, `to save time`)"
}
```

### H1.7 Score gating (hybrid threshold)

A 100/100 is blocked unless: all critical/high findings fixed or justified (≥50 words + Popper evidence); all load-bearing findings confirmed; `user_need_match.addressed = true` with verbatim quote; ≥3 falsifiable tests per phase; ≥2 edge cases per top-5 finding; cross-audit synthesis attempted; `confidence_basis` populated. Below threshold → score < 100, fix-and-reaudit loop (bounded at 5 iterations); on iteration 5 still failing → `confidence: low` + surface as `pending` in `.done.json`.

---

## PHASE 14: VERDICT

Score each phase 0-10, weight by severity. Privacy weights prioritize lawful basis, consent, erasure, sharing, and encryption — the surfaces with the highest legal + human cost.

```
SCORING MATRIX (360 max):
  Phase  1  (PII Inventory & Classification)     x 2.5  = max 25
  Phase  2  (Consent Capture & Withdrawal)       x 2.5  = max 25
  Phase  3  (Retention & Deletion / Erasure)     x 3.0  = max 30
  Phase  4  (Third-Party Sharing / Sub-proc)     x 3.0  = max 30
  Phase  5  (Cross-Border Transfer)              x 2.0  = max 20
  Phase  6  (Cookies & Tracking Compliance)      x 2.0  = max 20
  Phase  7  (Encryption of PII at rest/transit)  x 3.0  = max 30
  Phase  8  (Privacy Policy vs Reality)          x 2.5  = max 25
  Phase  9  (DSAR & Data-Subject Rights)         x 2.5  = max 25
  Phase 10  (Data Minimization / Purpose)        x 2.0  = max 20
  Phase 11  (Children's Data / COPPA)            x 1.5  = max 15
  Phase 12  (Logging & Telemetry PII Leakage)    x 2.0  = max 20
  Phase 13  (Breach-Notification Readiness)      x 1.5  = max 15
                                  SUBTOTAL (weighted) = 320
  + Phase 0b Recon/Hinge thoroughness            x 2.0  = max 20
  + Phase H1 Synthesis rigor (Popper/edge/cross) x 2.0  = max 20
                                  TOTAL  = max 360

NORMALIZE: score = (raw / 360) x 100
(If Phase 11 is proven N/A, exclude its 15 from the denominator: applicable_max = 345, normalize against that.)

GRADE:
  90-100: S — Lawful by design. Minimal data, provable consent, erasure works end-to-end, policy == code.
  80-89:  A — Compliant. Minor gaps (a backup not yet covered, one over-precise field).
  70-79:  B — Mostly lawful. Some processes manual/undocumented, no critical exposure. (PASS threshold = 70.)
  60-69:  C — Exposed. Real gaps: a tracker pre-consent, an incomplete erasure, an undisclosed processor.
  50-59:  D — Non-compliant. Policy contradicts code, special-category data under-protected.
  <50:    F — Unlawful. Data sold/shared without consent, no erasure path, PII in plaintext, no lawful basis.
```

---

## PHASE 15: FIX PLAN (automatic)

```
Sort: CRITICAL -> HIGH -> MEDIUM -> LOW
Priority by harm + legal exposure:
  CRITICAL: unlawful basis, special-category PII in plaintext, PII sold/shared without consent, no erasure path, child PII collected without consent
  HIGH:     trackers firing pre-consent, retention promise unenforced, policy materially contradicts code, unlawful cross-border transfer
  MEDIUM:   over-collection / excessive precision, incomplete DSAR export, PII in logs/error tracker, undisclosed minor processor
  LOW:      cookie lifetime too long, policy hygiene (missing last-updated/DPO contact), pseudonymization improvements

Group by blast radius (one fix may cover many fields — e.g. a single PII-scrubbing middleware fixes logging across all endpoints).
Dependency order (build the PII inventory/RoPA before wiring erasure; gate trackers before fixing the banner copy).
Generate fix tasks with file:line specificity + pii_category.
Save to audits/.privacyaudit/fix-plan.json + fix-plan.md.

NOTE: Some privacy findings are LEGAL/PROCESS, not code (e.g. "sign a DPA with vendor X", "publish sub-processor list").
For those, emit a task with type:"process" + owner + recommendation; do NOT fabricate code for a legal artifact.
```

---

## PHASE 16: FIX EXECUTION (automatic)

```
Sequential per finding group.

─── SAFETY GATE: DO NO HARM (MANDATORY before EVERY fix) ──────────────
A privacy fix MUST NOT destroy data unlawfully or break the product. Erasure/anonymization
is IRREVERSIBLE — treat every deletion fix as destructive.

PRE-FIX ANALYSIS (before writing ANY code):
  a. Read the ENTIRE target file (not just the target line).
  b. DESTRUCTIVE-OP GUARD — if the fix DELETES or ANONYMIZES PII:
     → Confirm a backup/export exists (or operate only on a test/seed user).
     → NEVER run a mass-erasure against production data inside an audit. Generate the migration/job and mark HIGH-RISK requiring human confirmation.
  c. SCOPE COLLISION / IMPORT SHADOW / CROSS-REFERENCE checks (as in secaudit's gate) for any code change (middleware, schema, handler).
  d. CONSENT-LOGIC GUARD — if gating a tracker behind consent, verify strictly-necessary functionality (auth, cart) is NOT broken by the gate.

POST-FIX VERIFICATION (after writing code, BEFORE commit):
  a. SYNTAX CHECK — `npx tsc --noEmit` / `bash -n` / `python3 -c "import ast; ast.parse(...)"`.
  b. IMPORT/LOAD CHECK — module loads without error.
  c. RUNTIME SMOKE TEST — start the service briefly OR `~/.omega/lib/safe-npm-build.sh`; verify no crash on init.
  d. BEHAVIORAL PROOF (privacy-specific) — re-run the falsification that found the issue:
     → tracker gated? load prod fresh, confirm it no longer fires pre-consent.
     → erasure added? run it on a test user, grep every store, confirm 0 PII remains.
     → column encrypted? inspect the dump, confirm ciphertext.
  e. TEST SUITE if present.

IF ANY POST-FIX CHECK FAILS → `git revert HEAD`, log in fix-log.md, mark NEEDS_REVIEW, try alternative or skip.
────────────────────────────────────────────────────────────────────────

FOR EACH FIX TASK (priority order):
  a. Read entire target file.
  b. PRE-FIX ANALYSIS (incl. destructive-op + consent-logic guards).
  c. Document BEFORE state (the violation + falsification evidence).
  d. Apply fix (code) OR emit process task (legal artifact).
  e. POST-FIX VERIFICATION (incl. behavioral proof).
  f. Green → commit: privacy(privacyaudit): FIX-XXX description
  g. Red → revert → log → NEEDS_REVIEW.
  h. Document AFTER state (same falsification, now refuted).
  i. HIGH-RISK (any erasure/anonymization of real data, any schema change touching PII) → require human confirmation.
```

---

## PHASE 17: RE-AUDIT (automatic)

```
1. SERVICE HEALTH GATE (mandatory):
   → systemd service: restart, wait 10s, check is-active + logs.
   → build step: full build must pass (safe-npm-build.sh).
   → tests: full suite must pass.
   → ANY failure: identify the breaking fix, revert.

2. Re-run all FAILING phases. Compare before/after using the falsification tests (not vibes).
3. Loop until normalized score >= 70 (PASS) or remaining items are NEEDS_REVIEW / process-only. Bounded at 5 iterations.
4. Special attention: verify fixes did NOT (a) break strictly-necessary functionality by over-gating, (b) delete data beyond the requested scope, (c) introduce new PII flows.
```

---

## FINAL VERIFICATION GATE (MANDATORY before final verdict)

> Per `../_shared/AUDIT-VERIFICATION-CONTRACT.md` — "Do No Harm". An audit that breaks one working thing is worse than no audit.

1. Read `../_shared/AUDIT-VERIFICATION-CONTRACT.md` fully.
2. Execute every check in the "IT STILL WORKS CHECKLIST".
3. Produce `audits/.privacyaudit/before-after.md` (Phase N+4 matrix) with per-item functional status (e.g. "signup still works", "auth cookie still set", "erase job runs", "tracker no longer fires pre-consent") + measurable deltas (PII fields collected: before/after; trackers pre-consent: before/after; stores covered by erasure: before/after).
4. Grep for stale references to any removed field/import/path — must be 0 non-ephemeral hits.
5. ONLY THEN write the final verdict.

If ANY check fails → status NEEDS_REVIEW, do NOT claim "done".

---

## INTEGRATION — CROSS-COMMAND BRIDGE

```
/privacyaudit finds unencrypted PII column  -> cross-refs /secaudit (A02 crypto) — joint fix
/privacyaudit finds PII-returning endpoint w/o authz -> cross-refs /apiaudit + /secaudit (IDOR / unlawful disclosure)
/privacyaudit finds no erasure / orphaned PII -> cross-refs /dataaudit (TTL, orphans, cascade)
/privacyaudit finds tracker firing pre-consent -> cross-refs /perfaudit (3rd-party script cost) + /secaudit
/privacyaudit finds policy-vs-code contradiction -> cross-refs /copyaudit (claim verification)

THE QUALITY ARSENAL (where privacy sits):
  /codeaudit    -> Is the code SOLID?               (preventive)
  /secaudit     -> Is it SECURE?                    (detective)
  /privacyaudit -> Is user data HANDLED LAWFULLY?   (detective)
  /dataaudit    -> Is the data INTACT?              (preventive)
  /apiaudit     -> Is the API SOLID?                (preventive)

  secaudit asks "can an attacker take the data?" — privacyaudit asks "are you even allowed to hold it,
  and did you keep your promises about it?" Security is the lock; privacy is whether you should have
  the key in the first place.
```

---

## COMPLIANCE & CRITICAL ADDENDA (v1.0 — 2026-05-29)

### Quality Arsenal Preamble Compliance

This audit implements contracts defined in `../_shared/QUALITY-ARSENAL-PREAMBLE.md` v1.0:

- ✅ **Gestalt-Popper doctrine** — HINGE DATA FLOW (10× scrutiny), falsification per finding, evidence chain, adversarial framing.
- ✅ **Concurrency lock** — canonical runner `flock` at `audits/.privacyaudit/.runner.lock`.
- ✅ **5-iteration cap** — fix-and-reaudit bounded at 5; on cap → NEEDS_REVIEW + Telegram SOS. No silent infinite loops.
- ✅ **Scoped invocation flags** — `--url=`, `--files=`, `--scope=`, `--ticket=`, `--no-fix`, `--focus=` (consent | erasure | sharing | cookies | encryption | minimization | policy | dsar).
- ✅ **Non-UI context gate** — runs on web, API, backend, or data-only targets; cookie/tracking phases (6) auto-skip + excluded from denominator on headless/API-only targets.
- ✅ **Output contract verification** — emits `verdict.json`, `verdict.md`, `fix-plan.json`, `fix-plan.md`, `before-after.md`, `progress.json`, `telemetry.json`, `fix-log.md`. Missing/malformed = audit did NOT succeed.
- ✅ **Telegram progress notifications** — `start` / `progress` (every 3 phases) / `iteration` / `verdict` / `abort` / `sos` via `~/.omega/bin/audit-notify.sh`.
- ✅ **Discovery drift check** — on resumed runs, if `discovery/` > 1h old, re-verify inventory or abort with user-confirm.
- ✅ **Self-telemetry** — `telemetry.json` at completion (duration, tokens, phases, fixes, pii_fields_count, trackers_pre_consent, model, preamble_version).
- ✅ **Deprecation registry** — cross-references checked against `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/DEPRECATED.md`; stale refs flagged.
- ✅ **Rule-46 compliance** — NO `--quick`/`--streamlined`/`--lightweight`. Narrower scope = `--focus <area>` at FULL phase depth. Orchestrator prompts with banned phrases are REFUSED.
- ✅ **Score normalization** — `raw / applicable_max × 100 = /100` (applicable_max excludes proven-N/A phases).
- ✅ **preamble_version** — emitted as `"1.0"` in verdict.json for `/metaudit`.

### Audit-Specific Critical Addendum — Data-Handling Safety + No-Fabrication + Live-Probe Discipline

**Data-handling safety (do no harm to people's data):**
- NEVER execute a mass-erasure/anonymization against production data inside the audit. Generate the migration/job; mark HIGH-RISK; require human confirmation.
- Falsification probes that read PII (dumps, rows) must REDACT values in all reports/JSON (store hashes/categories, never the raw PII).
- Live cookie/tracker probing uses Playwright CLI via Bash against the PROD URL (never MCP, never a dev server) — observe-only, no form submission with real PII.

**No-fabrication rule (legal artifacts):**
- This audit does NOT generate legal text (privacy policies, DPAs). It flags missing/contradictory legal artifacts as `type:"process"` findings with a recommendation + owner. Fabricating a privacy policy would create false assurance — that is itself a violation. Recommend counsel review for all `type:"process"` items.

**Live-probe discipline:**
- Rate-limit any live requests to the target (≤10 req/s). Abort on 3 consecutive 429/503.
- Self-introspection guard: if target is `~/.claude/`, `~/.omega/`, or system root → ABORT ("audit a product, not the agent infra").

### /metaudit Compliance Badge

Run `/metaudit --focus arsenal --scope="privacyaudit only"` to verify against the 11-point preamble checklist. Target: 11/11.

---

## Dynamic-Workflow Orchestration (v2)

> *"A privacy violation is not one finding — it is N independent flows, each with its own perimeter. Audit them in parallel, then try to kill every finding before you trust it."*

This section governs HOW the audit EXECUTES when run. It does not change WHAT is audited: every phase above (0 → 0b → 1–13 → H1), every scoring weight (the /360 matrix in Phase 14), every verdict format, and the Gestalt-Popper doctrine are UNCHANGED. v2 only replaces the linear phase-walk with a fan-out → adversarial-verify → synthesize → loop execution model. Phase 0 (programmatic gather) and Phase 0b (recon + HINGE DATA FLOW) still run FIRST and SERIALLY — they produce the shared evidence base (`evidence-summary.json`, `discovery/pii-inventory.json`, the identified HINGE DATA FLOW) that every parallel track reads.

### 1. Decompose into independent parallel tracks (FAN-OUT)

After Phase 0 + 0b complete, dispatch the domain phases as CONCURRENT tracks via the **Workflow** tool (in-process fan-out, per R-ORCH), instead of walking 1→13 linearly. The phases are grouped by the data surface they touch so that file-disjoint tracks run together and only same-surface phases serialize:

| Track | Phases (unchanged) | Reads (shared base) | Why independent |
|---|---|---|---|
| **A — Inventory & Minimization** | 1 (PII inventory), 10 (minimization/purpose) | `evidence-summary.json` PII census, schema | Census + necessity test share the field list |
| **B — Consent & Cookies** | 2 (consent), 6 (cookies/tracking) | cookie/tracker scan, prod URL (Playwright) | Both gate on the live pre-consent network observation |
| **C — Retention, Erasure & DSAR** | 3 (retention/deletion), 9 (DSAR/rights) | delete handler, every store (DB/cache/S3/search) | Erasure mechanics + user-facing rights are one flow |
| **D — Perimeter** | 4 (third-party sharing), 5 (cross-border) | third-party SDK census, processor regions | Sub-processors + their regions are the same egress map |
| **E — Encryption & Logging** | 7 (encryption at rest/transit), 12 (logging/telemetry PII) | transport probe, log calls, KMS config | Both are "is PII protected where it sits / flows" |
| **F — Reconciliation** | 8 (policy vs reality) | `policy-claims.json` + ALL tracks' findings | Runs LAST among tracks — joins every other track's output |
| **G — Children & Breach** | 11 (children's data), 13 (breach readiness) | age signals, access logs, RoPA backbone | Audience + incident-readiness, orthogonal to the rest |

Rules for the fan-out:
- Tracks **A, B, C, D, E, G** are file-disjoint enough to run in parallel. Track **F (policy-vs-reality)** depends on the others' findings (it reconciles every claim against what tracks A–E discovered) → it runs as the join/synthesis track AFTER the parallel batch returns.
- The **HINGE DATA FLOW** (Phase 0b) is injected into EVERY track: whichever track touches a hop of the hinge flow applies the 10× scrutiny (H1.2) inside that track. The hinge is not a separate track — it is a depth multiplier carried into all of them.
- Each parallel track emits a partial findings list in the SAME finding shape used by `evidence-summary.json.findings[]` (location, severity, `pii_category`, message, suggested_fix) so synthesis is a merge, not a re-format.
- **R-SCOPE (one writer per file):** tracks only READ during discovery. No track writes product code in this phase — fixes happen later in Phase 15/16, serialized. If two tracks would touch the same file at fix time, serialize or worktree-isolate them.

### 2. Adversarially verify every finding — ≥2-of-3 independent lenses (KILL WEAK FINDINGS)

A finding surfaced by a track is a CLAIM, never a verdict (R-VERIFY). Before a finding is allowed into synthesis, it must survive **≥2 of these 3 independent lenses**. A finding that passes <2 lenses is KILLED (demoted to `info` + `falsified_at`, excluded from scoring) — exactly the H1.1 Popper protocol, now applied per-track and consensus-gated:

1. **REPRODUCE** — re-run the concrete probe that PROVES the violation exists right now (First Law: runtime over code). Privacy-specific reproductions:
   - minimization: `grep` every read site (incl. exports, analytics traits, templates, third-party payloads) → 0 reads confirms "collected without purpose".
   - erasure gap: exercise the delete path on a TEST/seed user, then grep every store (DB, replica, cache, S3, search, logs) for the PII → residue confirms the gap.
   - tracker pre-consent: Playwright CLI loads the PROD URL fresh/incognito, observe network BEFORE accepting the banner → a non-strictly-necessary call confirms the violation.
   - plaintext-at-rest: inspect an actual row/dump for the sensitive column → plaintext (redacted in the report) confirms "encryption claim is false".
2. **REFUTE** — actively try to make the finding FALSE: is there a gate/middleware/cron/SCC/DPA elsewhere that already handles it? Read the counter-evidence file the first lens did not. (e.g. the tracker IS gated by a consent-mode wrapper; the column IS field-encrypted in a hook the scanner missed; a TTL job DOES enforce retention.) If refutation succeeds → KILL the finding.
3. **CROSS-CHECK** — independent corroboration from a different source than lens 1: a sibling audit's `evidence-summary.json` (secaudit / dataaudit / apiaudit per Phase 0.5), the privacy policy claim (`policy-claims.json`), the schema, or a second tool's raw output in `raw/`. Agreement → `cross_audit_confirmed: true` and bump severity one level (per H1.5); contradiction counts AGAINST the finding.

Consensus rule: **confirmed** = survived ≥2 lenses (e.g. reproduce + cross-check) → finding stands. **falsified** = any lens produced a clean counter-example AND fewer than 2 lenses confirmed → killed. **inconclusive** = couldn't run ≥2 lenses cleanly → keep at `confidence: medium`, never `high`. Load-bearing (hinge) findings require all 3 lenses attempted and ≥2 confirming (the 5× falsification budget of H1.2 is spent here). Banned phrases (`looks correct`, `should be fine`, `appears to work`) remain an automatic FAIL — a finding "verified" by vibes is not verified.

### 3. Synthesize survivors back into the EXISTING scoring matrix + verdict (UNCHANGED)

Synthesis is the audit's own job, never a paste of a track's summary (R-ORCH). Merge ALL surviving (confirmed) findings from tracks A–G + the reconciliation join (F):
- De-duplicate findings that multiple tracks raised on the same field/flow (e.g. an unencrypted PII column flagged by both Track E and a secaudit cross-check) → ONE finding, severity = max, `cross_audit_confirmed: true`.
- Map each surviving finding to its owning phase and feed the existing **Phase 14 SCORING MATRIX (/360)** EXACTLY as written — same per-phase 0–10 scores, same weights (Inventory ×2.5, Consent ×2.5, Retention/Erasure ×3.0, Third-Party ×3.0, Cross-Border ×2.0, Cookies ×2.0, Encryption ×3.0, Policy ×2.5, DSAR ×2.5, Minimization ×2.0, Children ×1.5, Logging ×2.0, Breach ×1.5, + Recon ×2.0 + Synthesis ×2.0), same `(raw / 360) × 100` normalization, same N/A-denominator rule for proven-N/A Phase 11.
- Emit the SAME `verdict.json` (hybrid v2 schema, H1.6), `verdict.md`, `fix-plan.json` (Phase 15), and the mandatory `before-after.md` (Phase N+4). The fan-out changes how findings were GATHERED and VERIFIED, not how they are SCORED or REPORTED. The H1.7 score gate (100/100 blocked unless all critical/high fixed-or-justified, all load-bearing confirmed, `user_need_match.addressed = true`, ≥3 falsifiable tests/phase, ≥2 edge cases/top-5) is unchanged and now naturally satisfied by the per-track verification above.
- Killed findings are recorded (with `falsified_at` + the lens that refuted them) but contribute ZERO to score — they neither raise nor lower it. A privacy audit's credibility dies if it reports a "violation" that a single grep would have refuted.

### 4. Loop-until-dry for unknown-size discovery

PII surface is not knowable up front — new flows hide in exports, CSV imports, webhook payloads, JWT claims, localStorage, error bodies, non-default-locale pages (see H1.4 edge cases). Run discovery as a **loop-until-dry** (a goal-loop INSIDE this workflow, per R-GOAL — never wrapped around it):

```
repeat:
  fan-out tracks A–G over the CURRENT known PII surface (step 1)
  adversarially verify new findings (step 2)
  expand the surface: for each surviving finding, trace one hop further
    (the field's next reader, the next store it lands in, the next third party it reaches,
     the next locale/import/log path that handles it)
  re-census ONLY the newly-revealed surface (do NOT re-grep what Phase 0 already covered — read its JSON)
until: a full pass reveals 0 new PII fields AND 0 new flows AND 0 new third-party egress points
       (i.e. the perimeter is closed) OR the 5-iteration cap (preamble §4) is hit
on cap with surface still expanding → mark remaining tracks NEEDS_REVIEW + Telegram SOS; never loop silently.
```

Termination is "the perimeter is closed", not "I ran once". The audit is dry when one more parallel pass finds no new person-bytes entering, moving, resting, or leaving the system — and every surviving finding has cleared ≥2-of-3 lenses.

---

## LAWS

1. **Every field is a person.** Behind every column is someone who can be harmed. Audit accordingly.
2. **Consent is a revocable contract.** If it can't be withdrawn and the withdrawal can't be proven to stop processing, it was never consent.
3. **The code keeps the promise, not the policy (First Law).** When policy and code disagree, the code is the truth and the policy is the liability.
4. **Deletion is a feature, not a favor.** Right-to-be-forgotten without an end-to-end erasure path is a violation, not a backlog item.
5. **The third party is your blast radius (Popper).** Every byte that leaves your perimeter is a byte you no longer fully control. Map the perimeter or admit you can't.
6. **The least data wins.** The byte you never collected can never leak, never be sold, never be subpoenaed. Minimize first, protect second.
7. **Falsify "we're compliant" (Popper).** Find the one tracker firing pre-consent, the one column in plaintext, the one field with no lawful basis. The regulator — and the breach — will.

---

*"/privacyaudit v1 — Inventory. Trace. Reconcile. Erase. Every field, every flow, every promise, every third party. Is user data handled lawfully? /360."*
