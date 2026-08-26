---
name: i18naudit
description: >
  Forensic internationalization & localization audit v1 (Gestalt-Popper). 18-phase deep analysis
  of everything that decides whether the product is WORLD-READY: hardcoded user-facing string
  detection (JSX text, alt/title/aria/placeholder attributes, toast/error/email/notification copy),
  i18n framework wiring integrity (key existence, namespace structure, interpolation contract),
  locale routing & detection (path/subdomain/cookie/Accept-Language, persistence across navigation),
  pluralization rules (CLDR plural categories: zero/one/two/few/many/other), gender/grammatical
  agreement, date/time formatting (Intl.DateTimeFormat, timezone), number/percent/unit formatting,
  currency formatting (ISO 4217, symbol placement, minor-unit precision), RTL support (dir attribute,
  logical CSS properties, bidi-safe interpolation, mirrored icons), translation completeness +
  fallback chains, character encoding (UTF-8 end to end, normalization, mojibake), locale-aware
  sorting & collation (Intl.Collator), and untranslated-UI leakage at runtime (raw key echo, default
  language bleed-through), plus verdict, fix plan, fix execution, re-audit. Score /360. Preamble v1.0
  compliant. Audit -> Plan -> Fix -> Re-audit.
  Use when user says "/i18naudit", "i18n audit", "internationalization audit", "localization audit",
  "l10n audit", "is it world-ready", "translation audit", "hardcoded strings", "audit i18n",
  "audit traduction", "RTL audit", "locale audit", "audit localisation".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet"]
domain: localization
phases: 18
max_score: 360
read_only: false
triggers: ["i18n", "i18n audit", "internationalization", "localization", "l10n", "translation audit", "hardcoded strings", "rtl audit", "locale audit", "is it world-ready", "audit traduction"]
---


<!-- AUDIT-META-V2-INJECTED -->

> ## ⚠️ MANDATORY FIRST STEP — READ THE V2 META-PROTOCOL + VENDORED DEPS
>
> **Before doing ANYTHING else**, Read these three vendored files (RELATIVE paths — this repo
> ships them; never reach for `~/.claude/...`, the blank-VPS rule forbids it):
> 1. `../_shared/audit-meta-protocol-v2.md` — overrides everything below for inputs/schema/falsification
> 2. `../_shared/QUALITY-ARSENAL-PREAMBLE.md` — the shared contract (locks, caps, flags, output gate)
> 3. `../_shared/AUDIT-VERIFICATION-CONTRACT.md` — the "Do No Harm" before/after protocol for fixes
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
> Model context: this audit runs on Opus 4.7 with max effort. There is no time pressure.
> Run every test you claim to have run. Cite verbatim outputs. No exceptions.

---

# /i18naudit v1 — Forensic Localization Audit (Gestalt-Popper)

> *"The other audits ask 'does it work for me?' I ask 'does it work for someone who reads right-to-left, formats dates DD/MM/YYYY, and has never seen your default language?'"*

---

## DOCTRINE

You are not a translator. You are a **localization forensic investigator**. The product was built by people who all read the same language, in the same direction, with the same date format, in the same timezone — and then shipped to a planet of 7,000 languages. Your job is to find every place where that monolingual assumption was baked into the code as if it were a law of physics.

**The 7 Laws of Localization Forensics (Gestalt-Popper Synthesis):**
1. **Every string the user reads is a contract with a translator.** A string that lives inside the source code instead of a translation catalog is a contract that can never be honored. Hardcoded `"Save"` is not "untranslated yet" — it is **untranslatable**.
2. **English is not the default — it is one locale among many (Popper).** FALSIFY every claim that "the app supports French/Arabic/Japanese." Switch the locale and read EVERY screen. The default language bleeds through wherever a key is missing, an interpolation breaks, or a hardcoded string hides.
3. **Length is a lie you tell yourself.** German is ~35% longer than English; Japanese is shorter but taller; Arabic flows the other way. A button that fits "OK" overflows on "Confirmer l'opération". Layout that assumes string length is layout that breaks abroad.
4. **Clarity before translation (Gestalt).** Before auditing, UNDERSTAND the i18n architecture. Read CLAUDE.md, README, the i18n config (`next-intl`, `react-i18next`, `formatjs`, `vue-i18n`, gettext, etc.). Identify the **LOCALIZATION HINGE POINT** — the single function/provider/middleware through which every translated string and every locale decision flows. If THAT breaks, the whole product reverts to one language. Audit it with 10x depth.
5. **Concatenation is the enemy of grammar (Popper).** `"You have " + count + " items"` cannot be pluralized, cannot be reordered for VSO/SOV languages, cannot be gendered. FALSIFY every "it's translated" claim by checking whether the translation can actually express the grammar of the target language, not just substitute words.
6. **Formatting is locale, not preference.** `1,000.50` is one thousand in the US and one in Germany (`1.000,50`). `01/02/2026` is January 2 in the US and February 1 in France. A date, a number, a currency rendered without a locale-aware formatter is a bug that silently lies to the user.
7. **The byte is innocent until the encoding proves it guilty (Popper).** Mojibake (`Ã©` instead of `é`), normalization mismatches (NFC vs NFD), and lost surrogate pairs (emoji, CJK) are encoding crimes that happen between the database and the screen. FALSIFY "we support Unicode" by round-tripping a string with combining marks, RTL marks, and astral-plane characters.

**Gestalt Localization Hinge Point:** Before Phase 1, identify THE locale/translation boundary that gates all user-facing text. The `<IntlProvider>`. The `useTranslations()` hook. The `t()` function. The locale middleware. The `getStaticProps` locale loader. THIS gets every phase at maximum depth. If it falls, every screen falls back to one language.

**Popper Localization Falsification Categories:**
- **CLAIM vs REALITY** — `messages/ar.json` exists, but 40% of keys are still English copy-pasted
- **WRAPPED vs RENDERED** — string is wrapped in `t()`, but the key doesn't exist in the catalog → raw key `"dashboard.title"` shown to the user
- **CONFIG vs RUNTIME** — `i18n.locales = ['en','fr','ar']`, but no `ar.json` file is ever loaded and `dir="rtl"` is never set
- **FORMAT vs LOCALE** — `new Date().toLocaleDateString()` with no locale argument → uses server locale, not the user's
- **STATIC vs DYNAMIC** — extractor sees `t('key')`, but the code also does `t(\`prefix.${dynamicVar}\`)` which the extractor cannot see and the catalog never covers

---

## SCOPE DETECTION (automatic from user prompt)

Read the user's prompt and determine scope automatically. No extra flags needed.

```
EXAMPLES:
  "/i18naudit"
  -> Full 18-phase pipeline. Inventory every locale, every catalog, every user-facing string.

  "/i18naudit the checkout flow isn't translated in French"
  -> TARGETED: checkout route files + their translation keys
  -> Switch to fr locale, walk the flow, find missing keys / hardcoded strings
  -> Focus: Phase 1 (hardcoded), Phase 6 (completeness), Phase 16 (runtime leakage)

  "/i18naudit RTL is broken for Arabic"
  -> RTL-FOCUSED: Phase 9 (RTL) at max depth + Phase 16 (runtime) for the ar locale
  -> dir attribute, logical CSS props, mirrored icons, bidi interpolation

  "/i18naudit dates and currency look wrong in the EU"
  -> FORMATTING-FOCUSED: Phase 4 (date/time), Phase 5 (number/unit), Phase 5b (currency)

  "/i18naudit find all hardcoded strings"
  -> HARDCODED-FOCUSED: Phase 1 (full extraction) + Phase 2 (framework wiring)

  "/i18naudit are we ready to launch in Japan?"
  -> LAUNCH-READINESS: full pipeline, special weight on encoding (CJK), completeness,
     date/number formatting, and runtime leakage for ja locale

RULES:
- If specific routes/locales/features mentioned: scope to those
- If a problem described: focus on relevant phases, skip irrelevant ones
- If "all" / "everything" / "world-ready" / "launch": all phases, all locales
- If audits/.i18naudit/fix-plan.json exists and no new scope: resume fixing
- Parse the intent, don't ask for clarification
```

---

## OUTPUT CONTRACT — Omega Integration

Every `/i18naudit` run produces these files. Oracles, AISB, and the monitor read them.

```
audits/.i18naudit/
|-- session.log
|-- discovery/
|   |-- i18n-architecture.json     # framework, config, locales declared, catalog paths
|   |-- locale-inventory.json      # every locale + catalog file + key count
|   |-- string-inventory.json      # every user-facing string (wrapped vs hardcoded)
|   |-- routing-map.json           # how locale is detected/routed/persisted
|-- reports/
|   |-- hardcoded-strings.md          # Phase 1
|   |-- framework-wiring.md           # Phase 2
|   |-- locale-routing.md             # Phase 3
|   |-- datetime-formatting.md        # Phase 4
|   |-- number-unit-formatting.md     # Phase 5
|   |-- currency-formatting.md        # Phase 5b
|   |-- pluralization.md              # Phase 6
|   |-- gender-grammar.md             # Phase 7
|   |-- interpolation-concat.md       # Phase 8
|   |-- rtl-support.md                # Phase 9
|   |-- translation-completeness.md   # Phase 10
|   |-- fallback-chain.md             # Phase 11
|   |-- encoding.md                   # Phase 12
|   |-- collation-sorting.md          # Phase 13
|   |-- layout-overflow.md            # Phase 14
|   |-- locale-data-coverage.md       # Phase 15
|   |-- runtime-leakage.md            # Phase 16
|-- verdict.json
|-- verdict.md
|-- fix-plan.json
|-- fix-plan.md
|-- progress.json
|-- fix-log.md
|-- before-after.md
```

**CRITICAL:** `progress.json` is read by the Telegram bot monitor for live progress cards.
Format: `{"total": 47, "done": 12, "failed": 1, "skipped": 2, "remaining": 32, "current": "FIX-013 — description"}`

**CRITICAL:** `fix-plan.json` is read by oracles to resume interrupted audits.
Format: `{"tasks": [{"id": "FIX-001", "finding": "...", "file": "...", "line": 42, "fix": "...", "status": "pending|done|failed|skipped", "severity": "CRITICAL|HIGH|MEDIUM|LOW"}]}`

---

## PHASE 0 — PROGRAMMATIC GATHER (HYBRID, runs FIRST, before all other phases)

> **Hybrid framework (2026-05-08):** before any LLM analysis, programmatic tools gather every
> machine-checkable finding deterministically. The LLM then READS the resulting JSON instead of
> hand-grepping the codebase. Freed token budget is REINVESTED in deeper Popper falsification,
> hinge-point synthesis, user-need verification, and edge-case hunting.

### 0.1 Run the gather script (mandatory, FIRST step)

```bash
~/.omega/lib/audit-runner.sh i18n "$PROJECT_PATH" \
  --files="$FILES_MODIFIED" \
  --url="$URL" \
  --user-need="$USER_NEED_QUOTE" \
  --hinge="$HINGE_POINT" \
  --ticket="$TICKET_ID"
```

This invokes `~/.omega/lib/audit-gather/i18n.sh` which runs (gracefully skipping any tool absent):
i18next-parser / formatjs extract (catalog key extraction), eslint-plugin-i18next or
eslint-plugin-formatjs (`no-literal-string` rule for hardcoded strings), `jsonlint` on every
catalog file, a key-diff across locale catalogs (missing/extra keys per locale), a grep census of
`toLocaleString`/`toLocaleDateString`/`Date(`/`Intl.` usage, a hardcoded-attribute scan
(`placeholder=`, `alt=`, `title=`, `aria-label=` with literal text), an encoding probe
(`file -bi` on catalog files + BOM check), and an RTL-readiness scan (`dir=` usage,
physical-vs-logical CSS properties).

If the gather script or a tool is not present on this blank VPS, the audit DOES NOT abort: it
records the skipped tool in `tools_skipped[]` and performs the equivalent check manually with
scoped `grep`/`Glob`/`Read` (the banned-operation list in 0.4 applies only to checks the gather
actually ran).

Output is written to:

```
$PROJECT_PATH/audits/.i18naudit/
├── raw/                    # raw tool outputs (JSON / text per tool)
└── evidence-summary.json   # normalized findings, single source of truth for the LLM
```

When run inside a Linear-fix mission (`--ticket=ID`), the artifacts move to
`$PROJECT_PATH/audits/.linear-fix/<ID>/.i18naudit/` so multiple audits on the same ticket can cross-reference.

### 0.2 evidence-summary.json schema

```jsonc
{
  "audit": "i18n",
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
  "metrics": { /* locales_declared, catalogs_found, key_count_per_locale, hardcoded_count, ... */ },
  "evidence_index": { /* paths to raw/ files for drill-down */ }
}
```

### 0.3 What you do AFTER the gather (this replaces hand-greps)

1. **Read `evidence-summary.json` in full.** This is your evidence base.
2. **Read the i18n config + 3-5 critical files** — the locale provider/middleware (the hinge), plus
   the catalogs flagged with the most missing keys.
3. **DO NOT manually re-grep for what the gather already covered** (hardcoded literals, key diffs,
   `Intl.` census). Re-running wastes tokens and reproduces the same evidence.
4. **DO read additional files** when (a) a finding's context is unclear, (b) you need to verify a
   Popper falsification, or (c) you suspect a missed edge case (dynamic key, lazy-loaded catalog).

### 0.4 Banned operations after Phase 0 (only for checks the gather actually ran)

- ❌ Re-running the full hardcoded-string grep across the whole tree (the gather did it)
- ❌ `find . -name "*.json" | xargs jsonlint` on all catalogs (the gather did it)
- ❌ Generic "let me read every component" loops (the gather inventoried them)

You MAY still:
- ✅ Read SPECIFIC files cited in findings (verify the issue in context)
- ✅ Run a SPECIFIC `grep` to falsify a finding (Popper test, see Phase H1)
- ✅ Run a SPECIFIC dynamic probe the static gather can't model (Playwright locale-switch render)

### 0.5 Cross-audit synthesis (read sibling evidence-summary.json files)

If this audit runs as part of a Linear-fix mission, sibling audits' summaries are at
`$PROJECT_PATH/audits/.linear-fix/<TICKET>/.<other-audit-id>/evidence-summary.json`. Read them. Use them.

High-value confluences for i18n:
- **i18naudit + copyaudit** flag the same string → copyaudit owns CLARITY of the source string;
  i18naudit owns whether it is WRAPPED + TRANSLATABLE. Joint fix: rewrite + extract in one change.
- **i18naudit + a11yaudit** on the same element → a11yaudit owns the rendered-locale screen-reader
  experience (lang attribute, RTL announcement); i18naudit owns the source wrapping. Confirm both.
- **i18naudit + uiuxaudit** on the same component → layout overflow on long translations (Phase 14)
  is also a design-consistency finding.
- **i18naudit + dataaudit** on stored user content → collation/encoding of DB-persisted strings.

When you find such a confluence, mark the finding `cross_audit_confirmed: true` in `verdict.json`
and bump severity by one level.

---

## PHASE 0b: RECONNAISSANCE

> *"You cannot audit localization until you know which world the product claims to serve."*

```bash
SESSION_ID="i18naudit-$(date +%Y%m%d-%H%M%S)"
mkdir -p audits/.i18naudit/{discovery,reports}
echo "AUDIT STARTED: $(date -Iseconds)" > audits/.i18naudit/session.log
```

```
1. I18N ARCHITECTURE DISCOVERY
   -> Read CLAUDE.md, README, package.json (deps: next-intl, react-i18next, i18next,
      @formatjs/*, react-intl, vue-i18n, @lingui/*, gettext, polyglot, etc.)
   -> Identify: framework, t() function name, catalog format (JSON, PO, YAML, FTL), catalog dir
   -> Identify: SSR vs CSR locale loading, static vs dynamic catalog import
   -> If NO i18n framework found → this is the #1 finding: the product is monolingual by
      construction. Score the design accordingly; the rest of the audit measures how deep the
      monolingual assumption goes.

2. DECLARED LOCALE INVENTORY
   -> From config: which locales are DECLARED? (i18n.locales, supportedLngs, etc.)
   -> Which is the default/source locale?
   -> Which catalog files actually EXIST on disk? (declared vs present = drift)
   -> Key count per catalog (cheap completeness signal before Phase 10)

3. LOCALIZATION HINGE POINT IDENTIFICATION
   -> Find THE provider/hook/middleware every translated string flows through
   -> Map: where is locale decided? where is it stored? where is the catalog loaded?
   -> This becomes ground zero for 10x-depth analysis (Phase H1.2)

4. TARGET-MARKET CONTEXT
   -> From the prompt / docs: which markets/languages matter? (drives weighting)
   -> RTL languages in scope? (ar, he, fa, ur) → Phase 9 weight up
   -> CJK in scope? (zh, ja, ko) → encoding (Phase 12) + line-breaking weight up
```

**Output:** `discovery/i18n-architecture.json`, `discovery/locale-inventory.json`

---

## PHASE 1: HARDCODED USER-FACING STRING DETECTION

> *"A string in the source code is a string no translator will ever see."*

```
1. VISIBLE TEXT NODES
   For every JSX/template text node, every string assigned to a user-visible variable:
   -> Is it wrapped in t()/<FormattedMessage>/$t()/trans()/gettext()?
   -> Or is it a raw literal: <button>Save</button>, <h1>Welcome</h1>?
   -> EXCLUDE non-user-facing literals: enum values, CSS class names, test IDs, log messages,
      object keys, route paths, internal error codes. Be precise — a false "hardcoded" finding
      on a CSS class is noise.

2. HARDCODED ATTRIBUTES (the commonly-missed surface)
   For every element attribute that renders to the user:
   -> placeholder="Enter your email"   → must be t()'d
   -> alt="Profile photo"              → must be t()'d
   -> title="Click to expand"          → must be t()'d
   -> aria-label="Close dialog"        → must be t()'d (also an a11y surface)
   -> value="Submit" on inputs/buttons → must be t()'d

3. IMPERATIVE / NON-RENDER STRINGS
   Strings shown to the user but not in markup:
   -> toast("Saved successfully"), alert(), confirm(), window.title
   -> thrown Error messages SURFACED to the user (vs internal logs)
   -> email/SMS/push notification templates
   -> validation messages (Zod/Yup custom messages, form errors)
   -> empty-state copy, loading copy, 404/500 page copy
   -> chart labels, table column headers, tooltip content

4. STRING IN NON-UI LAYERS THAT REACHES THE UI
   -> Backend returns a human-readable message string → is it localized server-side, or is the
      client expected to translate a CODE? (returning English prose to translate later = trap)
   -> Constants files of "labels" imported into components

5. CLASSIFICATION
   Each hardcoded string → severity by visibility:
   -> CRITICAL: primary navigation, CTAs, form labels, error/empty states (every user sees them)
   -> HIGH: secondary UI, settings, modals
   -> MEDIUM: rarely-seen flows (admin, edge errors)
   -> LOW: text that is arguably brand-fixed (product name) — flag but note the judgment

FALSIFY: don't trust the extractor's count. For each cluster, open the file and confirm the
literal actually renders to a user (Popper). A literal inside a `console.warn` is NOT a finding.
SCORE: 0 = pervasive hardcoded UI text, 3 = many in secondary flows, 5 = core wrapped but
attributes/toasts leak, 8 = nearly all wrapped, 10 = zero user-facing literal, lint rule enforces it
```

**Output:** `reports/hardcoded-strings.md`, `discovery/string-inventory.json`

---

## PHASE 2: I18N FRAMEWORK WIRING INTEGRITY

> *"A `t('key')` call is a promise that the key exists. Half the time, nobody checks."*

```
1. KEY EXISTENCE (wrapped-but-missing)
   For every t('some.key') call:
   -> Does 'some.key' exist in the SOURCE/default catalog?
   -> A wrapped key with no catalog entry = raw key rendered to the user ("some.key" on screen)
   -> This is WORSE than a hardcoded string: it looks broken, not just untranslated

2. NAMESPACE / STRUCTURE INTEGRITY
   -> Are namespaces consistent? (mixing flat "a.b.c" with nested {a:{b:{c}}})
   -> Are catalogs valid JSON/PO/YAML? (a single trailing comma kills a whole locale)
   -> Duplicate keys (last-wins silently overwrites the first)?

3. INTERPOLATION CONTRACT
   For every key with placeholders ({name}, %s, {{count}}, $1):
   -> Does the call site pass EXACTLY the variables the catalog expects?
   -> Mismatch → undefined rendered, or the variable shown literally as "{name}"
   -> Do all locales declare the SAME placeholders? (a translator dropping {count} breaks it)

4. PROVIDER / CONTEXT WIRING
   -> Is the i18n provider mounted ABOVE every component that calls t()?
   -> SSR: is the locale + messages passed through hydration without mismatch?
   -> Lazy/dynamic catalogs: is there a loading state, or does the UI flash raw keys then text?

5. DYNAMIC KEY HAZARD (the extractor blind spot)
   -> Grep for t(`prefix.${var}`) / t(variable) / computed keys
   -> The extractor CANNOT see these → catalog will silently lack them
   -> Flag every dynamic key + verify the full key space is covered or guarded

FALSIFY: pick 5 t() calls and trace each key to its catalog entry by hand (Popper). Pick 2 dynamic
keys and enumerate the possible values; confirm each resolves.
SCORE: 0 = raw keys visible in prod, 3 = many missing keys, 5 = source complete but interpolation
mismatches, 8 = wiring solid with dynamic-key gaps, 10 = every key proven to exist, lint-enforced
```

**Output:** `reports/framework-wiring.md`

---

## PHASE 3: LOCALE ROUTING & DETECTION

> *"If the user can't reach their language — or can't stay in it — nothing else matters."*

```
1. LOCALE DETECTION STRATEGY
   -> How is the initial locale chosen? (URL path /fr/, subdomain fr., cookie, Accept-Language,
      localStorage, IP geo, hardcoded default)
   -> Is Accept-Language parsed correctly (q-values, fallback, BCP-47 matching)?
   -> Does an unknown/unsupported locale fall back gracefully (not crash, not blank)?

2. LOCALE PERSISTENCE
   -> Once chosen, does the locale survive navigation? page reload? new tab? deep link?
   -> Is the choice stored (cookie/localStorage) AND reflected in the URL (for shareability)?
   -> Login → does the user's saved locale preference override the detected one?

3. URL & SEO CONTRACT (overlaps /seoaudit — owns ROUTING; seoaudit owns hreflang ranking)
   -> Are locales reflected in routable, crawlable URLs? (/fr/about not /about?lang=fr only)
   -> hreflang tags present and reciprocal? canonical per locale?
   -> Does switching locale preserve the current PAGE (deep equivalent), not bounce to home?

4. SWITCHER CORRECTNESS
   -> Is there a language switcher? Does every locale appear?
   -> Does the switcher show each language IN ITS OWN NAME (français, العربية), not translated?
   -> Does selecting a locale update URL + storage + <html lang> + dir atomically?

5. DEFAULT-LOCALE TRAP
   -> Is the default locale served at "/" with no prefix while others get "/xx/"? (inconsistent)
   -> Or is every locale prefixed? Pick one and be consistent — mixed = duplicate-content + bugs

FALSIFY: actually exercise it (Playwright CLI on the prod/dev URL): set Accept-Language: ar, load
"/", confirm Arabic + dir=rtl; switch to fr on /pricing, confirm you stay on /fr/pricing; reload,
confirm fr persists.
SCORE: 0 = can't reach non-default locale, 3 = reachable but doesn't persist, 5 = persists but
switcher/SEO gaps, 8 = solid routing minor gaps, 10 = detect+persist+URL+SEO+switcher all correct
```

**Output:** `reports/locale-routing.md`, `discovery/routing-map.json`

---

## PHASE 4: DATE / TIME FORMATTING

> *"01/02/2026 is January 2nd in Chicago and February 1st in Paris. The code that wrote it knows neither."*

```
1. FORMATTER USAGE
   -> Are dates formatted via Intl.DateTimeFormat / toLocaleDateString(locale, ...) / a locale-aware
      lib (date-fns/locale, Luxon, Day.js with locale, moment with locale)?
   -> Or via manual string building (`${d.getMonth()+1}/${d.getDate()}/...`) = HARDCODED FORMAT?
   -> Is the active locale PASSED to the formatter, or omitted (→ falls back to runtime/server locale)?

2. TIMEZONE CORRECTNESS
   -> Are timestamps stored in UTC and rendered in the user's timezone?
   -> Or rendered in the SERVER's timezone (everyone sees California time)?
   -> SSR hazard: server formats "today" in server TZ → hydration mismatch + wrong day near midnight
   -> DST boundaries handled? (a +1h naive add breaks twice a year)

3. RELATIVE TIME
   -> "2 hours ago" via Intl.RelativeTimeFormat (localized + pluralized) or hand-rolled English?
   -> "Yesterday"/"Tomorrow" hardcoded?

4. CALENDAR & WEEK ASSUMPTIONS
   -> First day of week hardcoded to Sunday/Monday vs locale-derived?
   -> 12h vs 24h clock locale-driven or hardcoded?
   -> Non-Gregorian calendar markets in scope? (if so, is the calendar configurable?)

FALSIFY: render the same instant under en-US, fr-FR, ja-JP, ar-EG; confirm format AND value differ
correctly (Popper — grep every toLocaleDateString/Date( call and check the locale arg).
SCORE: 0 = manual hardcoded format, 3 = formatter but no locale arg, 5 = locale ok but TZ wrong,
8 = solid with relative-time gaps, 10 = Intl everywhere + UTC store + user TZ + DST safe
```

**Output:** `reports/datetime-formatting.md`

---

## PHASE 5: NUMBER / PERCENT / UNIT FORMATTING

> *"1,000.50 means one thousand here and one there. The decimal separator is not a constant."*

```
1. NUMBER FORMATTING
   -> Numbers formatted via Intl.NumberFormat / toLocaleString(locale)?
   -> Or string-built with hardcoded "," thousands and "." decimal?
   -> Grouping (1,000 vs 1.000 vs 1 000 vs 10,00,000 for en-IN) locale-driven?

2. PERCENT & RATIO
   -> Percent via Intl.NumberFormat({style:'percent'}) (handles spacing + symbol position)?
   -> Or "${n}%" hardcoded? (some locales space it: "50 %")

3. UNITS & MEASUREMENT
   -> Distances/weights/temperatures: Intl.NumberFormat({style:'unit'}) or hardcoded?
   -> Imperial vs metric assumption baked in? (miles/km, lb/kg, °F/°C)

4. INPUT PARSING (the reverse direction, commonly broken)
   -> When a user TYPES a number, is it parsed with the locale's separators?
   -> parseFloat("1.000,50") = 1, a silent data-corruption bug for comma-decimal locales

FALSIFY: format 1234567.89 under en-US, de-DE, fr-FR, en-IN; confirm grouping + decimal differ.
Type "1.234,56" into a numeric input under de-DE and confirm it parses to 1234.56.
SCORE: 0 = hardcoded separators + broken parse, 3 = format ok parse broken, 5 = Intl format only,
8 = format+parse mostly locale-safe, 10 = Intl format + locale-aware parse + units handled
```

**Output:** `reports/number-unit-formatting.md`

---

## PHASE 5b: CURRENCY FORMATTING

> *"$1,000 is not €1.000 is not ¥1000. Symbol, placement, separators, and minor units all change."*

```
1. CURRENCY RENDERING
   -> Intl.NumberFormat({style:'currency', currency:'EUR'}) or hardcoded "$" + number?
   -> Symbol PLACEMENT correct per locale? ("$5" vs "5 €" vs "5,00 €")
   -> Currency CODE driven by data (ISO 4217), not assumed USD?

2. MINOR-UNIT PRECISION
   -> JPY/KRW have 0 decimals; BHD/KWD have 3; most have 2 — is precision per-currency?
   -> Money stored as integer minor units (cents) or as a lossy float? (float money = bug)

3. CURRENCY vs LOCALE INDEPENDENCE
   -> Currency (what you pay in) and locale (how it's formatted) are INDEPENDENT
   -> A French user paying in USD should see "1 234,56 $US" — locale formats, currency is the value
   -> Is the symbol/code disambiguated where needed ($ → US$/CA$/A$)?

4. CONVERSION HONESTY
   -> If amounts are converted between currencies, is the rate + timestamp shown?
   -> Are converted amounts marked as estimates vs the actual charge currency?

FALSIFY: render 1234.5 USD and 1234 JPY under en-US, fr-FR, ja-JP; confirm symbol, placement,
separators, and decimal count are all correct (JPY shows no decimals).
SCORE: 0 = hardcoded $ + float money, 3 = symbol hardcoded, 5 = Intl but precision wrong,
8 = mostly correct minor gaps, 10 = Intl currency + integer minor units + ISO code from data
```

**Output:** `reports/currency-formatting.md`

---

## PHASE 6: PLURALIZATION RULES

> *"English has 2 plural forms. Arabic has 6. Your `count === 1 ? 'item' : 'items'` ternary is a monolingual fossil."*

```
1. PLURAL MECHANISM
   -> Are plurals expressed via ICU MessageFormat / i18next plurals / Intl.PluralRules?
   -> Or via `count === 1 ? singular : plural` (handles ONLY English-like 2-form languages)?
   -> Or via "s" suffix concatenation (utterly untranslatable)?

2. CLDR PLURAL CATEGORY COVERAGE
   -> For each pluralized string, do the non-English catalogs provide the categories the language
      NEEDS? CLDR categories: zero, one, two, few, many, other.
      - Arabic uses all 6; Polish uses one/few/many/other; Japanese uses only "other".
   -> A French catalog with only "one"/"other" but missing the "many" nuance, an Arabic catalog with
      only "one"/"other" → grammatically wrong counts shown to users.

3. ZERO HANDLING
   -> Is "0 items" handled distinctly where the language/UX wants it (zero category / "no items")?

4. ORDINALS
   -> "1st/2nd/3rd" via Intl.PluralRules({type:'ordinal'}) + catalog, or hardcoded English suffixes?

5. RANGES
   -> "3–5 items" via Intl.NumberFormat.formatRange / proper range message, or string concat?

FALSIFY: render the count message at 0, 1, 2, 5, 11, 21, 101 under en, fr, pl, ar; confirm each
selects the grammatically correct form (Popper — Intl.PluralRules('ar').select(n) tells you the
required category; check the catalog actually has it).
SCORE: 0 = ternary/suffix plurals, 3 = ICU but only 2 forms in catalogs, 5 = partial CLDR coverage,
8 = full coverage minor gaps, 10 = ICU plurals + all required CLDR categories per locale + ordinals
```

**Output:** `reports/pluralization.md`

---

## PHASE 7: GENDER & GRAMMATICAL AGREEMENT

> *"'Bienvenue' or 'Bienvenu·e'? The product greeted a woman with a masculine adjective and called it i18n."*

```
1. GENDERED MESSAGE SUPPORT
   -> Do messages that depend on a referent's gender use ICU select {gender, select, ...} or
      gender-specific keys? Or is one form hardcoded?
   -> "Welcome back, {name}" is safe; "He liked your post" / past participles agreeing with subject
      are NOT — they need the actor's gender.

2. GRAMMATICAL AGREEMENT
   -> Adjective/article/participle agreement with noun gender & number (Romance, Slavic, Semitic)?
   -> Definite/indefinite article fused with placeholder? ("le {item}" breaks for feminine nouns)

3. CASE / DECLENSION (highly-inflected languages: Slavic, Finnish, etc.)
   -> Does interpolating a noun into a sentence require a grammatical case the catalog can't express?
   -> Flag templates where the inserted variable would need declension.

4. FORMALITY / HONORIFICS
   -> Languages with T-V distinction or honorific levels (de Sie/du, ja keigo, ko speech levels):
      is the register a translatable decision, or is one register hardcoded?

5. INCLUSIVE / NEUTRAL DEFAULTS
   -> Where gender is unknown, is there a neutral form, or does it default to masculine?

This phase is largely about TRANSLATABILITY of the message STRUCTURE, not judging translations.
FALSIFY: find 3 gendered/agreeing strings; prove the current structure can (or cannot) express the
correct form in fr/ar/pl. A `t('liked') + name` concat = cannot. Flag it.
SCORE: 0 = gender/agreement impossible to express, 3 = ad-hoc per-key duplication, 5 = some ICU
select, 8 = structured with case gaps, 10 = ICU select + agreement-safe + neutral defaults
```

**Output:** `reports/gender-grammar.md`

---

## PHASE 8: INTERPOLATION & CONCATENATION FORENSICS

> *"`'Found ' + n + ' results in ' + ms + 'ms'` is four English fragments glued in English word order. No translator can rescue it."*

```
1. STRING CONCATENATION OF UI TEXT
   -> Grep for "+" / template literals that glue translated fragments + variables + more text
   -> Each fragment is translated in isolation → word order is frozen to the source language
   -> The fix is ONE message with placeholders: t('found', {n, ms}) — flag every concat

2. SENTENCE ASSEMBLY FROM PARTS
   -> UI that builds a sentence from a verb dropdown + noun dropdown + adverb = grammatical lottery
   -> Lists joined with hardcoded ", " and " and " (use Intl.ListFormat)

3. PLACEHOLDER REORDERABILITY
   -> Does the message format allow reordering placeholders? ("{count} {unit}" → some languages
      need "{unit} {count}"). Positional %s without indices ($1/$2) blocks reordering.

4. NESTED / RICH INTERPOLATION
   -> Bold/links inside a sentence: <Trans> with components or dangerouslySet concatenation?
   -> Splitting a sentence to inject a <Link> mid-string ("Click " + <a>here</a> + " to continue")
      freezes word order — flag it; use rich-text interpolation instead.

5. LIST & ENUMERATION FORMATTING
   -> Intl.ListFormat for "A, B, and C" / "A، B، وC"? Or hardcoded conjunctions?

FALSIFY: pick the 3 worst concatenations; rewrite one mentally into French/Japanese word order and
show it cannot be expressed by the current fragments (Popper).
SCORE: 0 = pervasive concat sentence-building, 3 = many glued fragments, 5 = mostly single messages
with concat leaks, 8 = single messages + ListFormat gaps, 10 = single reorderable messages + rich
interpolation + ListFormat
```

**Output:** `reports/interpolation-concat.md`

---

## PHASE 9: RTL (RIGHT-TO-LEFT) SUPPORT

> *"You added Arabic to the locale list. Did you add it to the layout?"*

```
1. DIRECTION WIRING
   -> Is <html dir="rtl"> (or a dir on a root container) set for RTL locales (ar, he, fa, ur)?
   -> Is dir derived from the active locale, or hardcoded "ltr" / absent?
   -> Does it flip atomically with the locale switch (Phase 3)?

2. LOGICAL vs PHYSICAL CSS
   -> Does the CSS use LOGICAL properties (margin-inline-start, padding-inline-end, inset-inline,
      text-align: start) or PHYSICAL (margin-left, padding-right, left:, text-align:left)?
   -> Physical properties do NOT mirror in RTL → broken layout (labels on wrong side, cramped gutters)
   -> Tailwind: ms-*/me-*/ps-*/pe-*/start-*/end-* (logical) vs ml-*/mr-*/left-*/right-* (physical)

3. ICON & DIRECTIONAL ELEMENT MIRRORING
   -> Directional icons (back/forward arrows, chevrons, progress, breadcrumb separators) mirrored?
   -> Non-directional icons (clock, logo, media play... — play stays, others vary) NOT wrongly flipped?
   -> Sliders, carousels, toggles: do they flow in the correct direction?

4. BIDI-SAFE INTERPOLATION
   -> Mixing LTR content (numbers, URLs, code, latin names) into RTL text without bidi isolation
      (Unicode isolates / <bdi>) → scrambled rendering ("call +1 234" jumps around)
   -> Are user-generated values isolated?

5. RTL-SPECIFIC LAYOUT
   -> Are floats/flex/grid direction-aware (flex-direction respects dir, not hardcoded row)?
   -> Scrollbars, modals, drawers open from the correct side?
   -> Forms: label/field/error alignment correct in RTL?

FALSIFY: load an RTL locale (Playwright CLI), screenshot key pages, confirm dir=rtl AND that the
layout actually mirrored (not just text-aligned right). Grep for physical CSS props that should be
logical.
SCORE: 0 = no RTL support though RTL locale declared, 3 = dir set but physical CSS breaks layout,
5 = layout mirrors but icons/bidi wrong, 8 = solid with isolation gaps, 10 = dir + logical CSS +
icon mirroring + bidi isolation, verified rendered
```

**Output:** `reports/rtl-support.md`

---

## PHASE 10: TRANSLATION COMPLETENESS

> *"The locale list says 5 languages. The catalogs say 1.3 languages and 3.7 piles of English."*

```
1. KEY-LEVEL COMPLETENESS (per locale)
   -> For each non-source catalog: which keys are PRESENT vs MISSING vs EMPTY vs EXTRA?
   -> Missing key → fallback (Phase 11) or raw key (Phase 2/16) shows
   -> Empty string ("") → blank UI; often worse than missing (no fallback triggers)
   -> Extra key (in target, not in source) → dead translation, drift signal

2. UNTRANSLATED-BUT-PRESENT (the silent gap)
   -> A key present in fr.json but whose VALUE is identical to en.json → likely copy-paste, NOT
      translated. Flag high-ratio locales (e.g. fr "translated" 100% but 60% byte-identical to en).
   -> Distinguish legitimate identical strings (proper nouns, "OK", "Email") from untranslated copy.

3. COMPLETENESS BY SURFACE (not just by count)
   -> 95% complete is meaningless if the missing 5% is the entire checkout flow.
   -> Weight completeness by surface importance: core nav/CTA/checkout missing = CRITICAL even at
      high overall %.

4. NEW-KEY ROT
   -> Keys added to source since the last translation pass → which locales lag?
   -> Is there a process (CI check, lint) blocking merges that add untranslated keys?

5. PLURAL/SELECT SUB-COMPLETENESS
   -> A plural key "present" in a locale but missing required CLDR categories (Phase 6) is
      INCOMPLETE even though the key exists. Cross-reference Phase 6.

FALSIFY: compute the real per-locale key diff from the catalogs (the gather did this — read it),
then spot-check 5 "present" keys per locale for copy-paste-from-source.
SCORE: 0 = declared locales mostly empty, 3 = <70% per locale, 5 = high % but core surfaces missing,
8 = ~complete with copy-paste leaks, 10 = 100% per locale, no copy-paste, CI blocks regressions
```

**Output:** `reports/translation-completeness.md`

---

## PHASE 11: FALLBACK CHAIN CORRECTNESS

> *"When the translation is missing, what does the user see? Hopefully not a raw key. Hopefully not nothing."*

```
1. FALLBACK EXISTENCE & ORDER
   -> Is a fallback locale configured? (fr-CA → fr → en, not fr-CA → blank)
   -> Is the chain sensible (regional → base → default), or does it jump straight to default,
      losing a closer match?

2. MISSING-KEY BEHAVIOR
   -> On a missing key, does the framework: (a) fall back to source text [best], (b) show the raw
      key "dashboard.title" [bad], (c) show empty [worst], (d) throw [catastrophic]?
   -> Is there a missing-key HANDLER (logs to telemetry so gaps get fixed)?

3. PARTIAL FALLBACK CONSISTENCY
   -> When some keys fall back to en and others are fr, the screen is bilingual gibberish.
      Is that acceptable, or should a partially-translated locale be hidden until complete?

4. REGIONAL VARIANT HANDLING
   -> en-GB vs en-US (colour/color, organise/organize) — separate catalogs or one "en"?
   -> es-ES vs es-MX, pt-BR vs pt-PT, zh-Hans vs zh-Hant (different SCRIPTS, not just region —
      Hant↔Hans must NOT fall back to each other blindly).

5. FALLBACK FOR FORMATTING
   -> Number/date/currency formatters: does an unknown locale degrade to a reasonable base, or crash?

FALSIFY: delete-test (in a scratch copy) a key from a target locale and confirm the configured
fallback fires as designed; request an unsupported locale "xx-YY" and confirm graceful degradation.
SCORE: 0 = missing key → crash/blank, 3 → raw key shown, 5 → falls back but skips closer match,
8 = good chain with variant gaps, 10 = regional→base→default + source fallback + missing-key telemetry
```

**Output:** `reports/fallback-chain.md`

---

## PHASE 12: CHARACTER ENCODING INTEGRITY

> *"Between the database and the screen, 'é' became 'Ã©'. Somebody decoded UTF-8 as Latin-1 and shipped it."*

```
1. UTF-8 END TO END
   -> Are catalog files UTF-8 (no BOM surprises, no Latin-1)? (gather ran file -bi)
   -> HTML <meta charset="utf-8">? HTTP Content-Type charset=utf-8 on API + pages?
   -> DB columns/collation Unicode (utf8mb4 not utf8 in MySQL — utf8 can't store emoji/astral)?
   -> Are form/body parsers reading UTF-8?

2. MOJIBAKE DETECTION
   -> Scan catalogs + rendered output for classic mojibake (Ã©, â€™, Â , ï»¿ stray BOM)
   -> Double-encoding ("&amp;amp;", "%2520") in user-facing text?

3. ASTRAL PLANE & SURROGATE SAFETY
   -> Emoji, CJK Ext-B, rare scripts: does string length/truncation use code points or UTF-16 code
      units? (str.slice on surrogate pairs splits a character into garbage)
   -> Are DB column lengths in characters vs bytes (a CJK char is 3 bytes in UTF-8 → "VARCHAR(10)"
      stores only ~3 CJK chars)?

4. NORMALIZATION (NFC vs NFD)
   -> Is user input normalized (NFC) before storage/comparison? (é as one codepoint vs e+combining
      accent compare unequal, break search/login/dedup)
   -> macOS uploads filenames in NFD; cross-platform mismatch.

5. ENCODING IN TRANSIT
   -> URL-encoding of non-ASCII params correct? Email headers (RFC 2047) for non-ASCII subjects?
   -> CSV/PDF/Excel exports preserve UTF-8 (BOM where Excel needs it)?

FALSIFY: round-trip a torture string ("Crème brûlée 北京 مرحبا 😀 é") through input → store →
fetch → render and confirm byte-identical (Popper). Check file -bi on every catalog.
SCORE: 0 = mojibake in prod / utf8 (not mb4) DB, 3 = mojibake in exports, 5 = UTF-8 but no
normalization, 8 = solid with astral edge gaps, 10 = UTF-8 e2e + utf8mb4 + NFC normalize + astral-safe
```

**Output:** `reports/encoding.md`

---

## PHASE 13: LOCALE-AWARE SORTING & COLLATION

> *"In Swedish, 'ä' sorts after 'z'. In your code, it sorts wherever its byte value lands. The user list is wrong in half of Europe."*

```
1. SORT MECHANISM
   -> Are user-visible string sorts done with Intl.Collator(locale) / localeCompare(locale)?
   -> Or with default code-point/byte sort (capital Z < lowercase a, accents scatter)?

2. LOCALE-SPECIFIC ORDERING
   -> Swedish/Finnish (å ä ö after z), German ä/ö/ü + ß handling, Spanish ñ, Czech ch as a letter,
      Turkish dotted/dotless i (the famous "Turkish-I" bug breaking case-insensitive compare)
   -> Asian scripts: stroke/pinyin/radical ordering for zh; kana ordering for ja

3. CASE-INSENSITIVE COMPARE SAFETY
   -> toLowerCase()/toUpperCase() WITHOUT locale → Turkish i↔İ mismatch breaks search/auth/dedup
   -> Use localeCompare with sensitivity, or Intl.Collator({sensitivity:'accent'})

4. SEARCH & FILTER COLLATION
   -> Does search match accent-insensitively where expected ("cafe" finds "café")?
   -> DB-side collation matches app-side expectation? (mismatched ORDER BY vs UI sort)

5. STABLE, DETERMINISTIC ORDER
   -> Is the sort stable so equal-collation items don't reshuffle between renders?

FALSIFY: sort ["Zürich","apple","Äpfel","Östersund","ångström"] with default vs Intl.Collator('sv')
and show the order differs; lowercase "İ" with and without 'tr' locale and show the difference.
SCORE: 0 = byte sort + Turkish-I bug, 3 = localeCompare without locale, 5 = collator app-side only
(DB mismatch), 8 = collator both sides minor gaps, 10 = Intl.Collator app+DB + accent-aware search
```

**Output:** `reports/collation-sorting.md`

---

## PHASE 14: LAYOUT & TEXT-EXPANSION RESILIENCE

> *"The button fit 'Save'. It does not fit 'Enregistrer les modifications'. German will be worse."*

```
1. EXPANSION TOLERANCE
   -> Do containers tolerate ~+35% text growth (EN→DE/FR) without truncation/overlap/overflow?
   -> Fixed-width buttons/badges/tabs/nav items that assume short English?
   -> Truncation with ellipsis that hides meaning (CTA text cut off)?

2. CONTRACTION & VERTICAL TEXT
   -> CJK is shorter horizontally but may need more line height; some scripts are taller (Thai,
      Devanagari diacritics clipped by tight line-height)?
   -> Vertical writing modes (ja/zh) in scope? (writing-mode handled or ignored?)

3. PSEUDO-LOCALIZATION READINESS
   -> Is there (or could there be) a pseudo-locale (Ⓔⓝⓛⓐⓡⓖⓔⓓ [!!! Ḗḗḗ !!!]) to surface
      truncation + hardcoded strings at once? Recommend if absent.

4. WORD-BREAK & HYPHENATION
   -> Long compound words (German) wrap or overflow? overflow-wrap/hyphens set?
   -> Non-breaking constructs (numbers+units, names) break awkwardly?

5. BIDI LAYOUT INTERACTION (cross-ref Phase 9)
   -> Mixed-direction lines wrap and align correctly?

FALSIFY: render 3 tight components with the longest available translation (or a +40% pseudo string)
via Playwright; screenshot; confirm no clip/overlap/overflow (Popper — visual evidence required).
SCORE: 0 = pervasive truncation on real translations, 3 = many tight spots, 5 = mostly fluid with
clips, 8 = resilient minor gaps, 10 = expansion-tolerant + word-break + pseudo-loc tested
```

**Output:** `reports/layout-overflow.md`

---

## PHASE 15: LOCALE DATA COVERAGE & ASSET LOCALIZATION

> *"You localized the text and forgot the dropdown of country names, the phone format, and the marketing screenshot full of English."*

```
1. REFERENCE-DATA LOCALIZATION
   -> Country/region/language/currency NAME lists localized (Intl.DisplayNames) or hardcoded English?
   -> Timezone names, day/month names sourced from Intl, not a hardcoded English array?

2. ADDRESS & PHONE & NAME FORMATS
   -> Address form assumes US format (State + ZIP) for all countries?
   -> Phone number formatting/validation per region (libphonenumber) or US-pattern hardcoded?
   -> Name handling assumes given+family order / latin script (CJK family-first, mononyms)?

3. LOCALIZED ASSETS
   -> Images/screenshots/diagrams with baked-in English text → need per-locale variants or
      text-as-overlay? Flag embedded text in assets.
   -> Locale-specific media (video captions/subtitles, audio) provided?

4. LEGAL / COMPLIANCE COPY
   -> ToS, privacy, cookie banner, consent: localized AND legally appropriate per jurisdiction?
   -> Required local notices present for target markets?

5. SEARCH/SLUG/URL CONTENT
   -> User-facing slugs transliterated/localized or stuck in source language?
   -> Email templates, PDF exports, invoices fully localized (often forgotten back-office surfaces)?

FALSIFY: open the country/language dropdown under fr and ar — are the names localized? Inspect one
marketing image for baked-in English. Check an invoice/email template's locale handling.
SCORE: 0 = all reference data + assets English-only, 3 = text localized, data/assets not, 5 = data
via Intl but assets/legal gaps, 8 = broad coverage minor gaps, 10 = data via Intl + localized assets
+ region-aware address/phone + localized legal/back-office
```

**Output:** `reports/locale-data-coverage.md`

---

## PHASE 16: RUNTIME UNTRANSLATED-UI LEAKAGE (THE DETECTIVE PASS)

> *"Static analysis says it's wrapped. Switch the locale, walk every screen, and watch the English bleed through."*

This is the phase that proves it at runtime (First Law: only runtime tells the truth). Static checks
find intent; this finds reality.

```
1. LOCALE-SWITCH WALKTHROUGH (per declared non-source locale)
   Using Playwright CLI against the prod/dev URL:
   -> Set each declared locale; load every key route + open every major modal/menu/empty state.
   -> Capture screenshots + the page text. Scan rendered text for:
      a. RAW KEYS leaking ("dashboard.title", "errors.404") — wrapped-but-missing (Phase 2)
      b. SOURCE-LANGUAGE BLEED — English words on a French/Arabic screen (hardcoded, Phase 1, or
         missing keys falling back, Phase 10/11)
      c. "{name}"-style literal placeholders shown to the user (interpolation mismatch, Phase 2)
      d. undefined / null / NaN / "Invalid Date" rendered (formatter/locale-data failure)

2. CONSOLE & MISSING-KEY TELEMETRY
   -> Capture console for i18next "missingKey" / formatjs "Missing message" / "MISSING_TRANSLATION"
      warnings while walking. Each is a concrete finding with the exact key.

3. DYNAMIC / LAZY SURFACES
   -> Trigger toasts, validation errors, async-loaded panels, infinite-scroll content under a
      non-source locale — these are where hardcoded strings hide from static scans.

4. FLASH-OF-UNTRANSLATED-CONTENT (FOUC-i18n)
   -> On lazy catalog load: does the UI flash raw keys/source text before the translation arrives?

5. <html lang> + dir CORRECTNESS AT RUNTIME
   -> Confirm <html lang="ar"> + dir="rtl" actually set per locale (cross-ref Phase 3/9; also an
      a11y/SEO signal).

FALSIFY: this phase IS the falsification of Phases 1/2/10/11 — every leaked string here is a Popper
counter-example to "it's translated". Cite the screenshot path + the exact leaked text + route.
SCORE: 0 = raw keys/English pervasive at runtime, 3 = leakage in main flows, 5 = core clean,
dynamic surfaces leak, 8 = clean with rare leaks, 10 = zero leakage across all locales + no missing-
key warnings + correct lang/dir, screenshot-proven
```

**Output:** `reports/runtime-leakage.md`

---

## PHASE H1 — HYBRID SYNTHESIS (Popper / hinge / user-need / edge cases / cross-audit)

> **Hybrid framework (2026-05-08), runs immediately before VERDICT.** "H1" = Hybrid step 1 of the
> synthesis layer that pairs with Phase 0's programmatic gather. It does NOT renumber existing
> phases; it sits between the last domain phase and the VERDICT phase. The token budget freed by
> Phase 0's deterministic gather is REINVESTED here — depth of analysis is what increases.

### H1.1 Popper falsification per finding (mandatory)

For every finding in `evidence-summary.json.findings[]` (start with `severity ∈ {critical, high}`,
then down as budget allows), try to PROVE the tool is wrong. Each falsification produces a
`falsifiable_tests[]` entry in `verdict.json`:

```jsonc
{
  "claim": "eslint-plugin-i18next flags src/pages/Checkout.tsx:88 literal 'Pay now' as unwrapped",
  "test_command": "grep -n \"Pay now\" src/pages/Checkout.tsx && grep -rn \"'Pay now'\\|\\\"Pay now\\\"\" src/locales/",
  "expected": "literal renders to user AND no catalog key holds it → finding TRUE",
  "actual": "literal at Checkout.tsx:88 inside <button>; absent from all catalogs",
  "outcome": "confirmed"
}
```

Outcomes: `confirmed` (test failed to falsify → finding stands), `falsified` (counter-example →
demote to `info` + add `falsified_at`), `inconclusive` (couldn't run cleanly → keep severity,
`confidence: medium`).

**The rule:** every CLAIM (PASS or FAIL) MUST cite ≥3 concrete commands that COULD have falsified it
but didn't. Banned phrases (`looks correct`, `should be fine`, `appears to work`) → automatic FAIL.

Common falsification patterns for i18n:

| Tool says | Popper test |
|---|---|
| `no-literal-string` (hardcoded) | Open the file — does the literal RENDER to a user, or is it a className/testId/log? If non-UI → falsified |
| `missing key in fr.json` (key-diff) | Check the fallback config — does it fall back to source text gracefully, or show the raw key? Severity depends on the answer |
| `present but identical to source` (copy-paste) | Is the string a proper noun / "OK" / "Email"? Legitimately identical → falsified |
| `toLocaleDateString without locale` | Trace whether a default locale is set globally so the call inherits it; if so, demote (still flag as fragile) |
| `dir not set for ar` | Load the ar URL at runtime — is dir set by a provider the static scan missed? |
| `dynamic key t(\`${x}\`)` | Enumerate x's possible values; if all are covered in the catalog → falsified |

### H1.2 Hinge cross-reference (10× scrutiny on load-bearing findings)

The LOCALIZATION HINGE POINT (Phase 0b.3 — the provider/hook/middleware all translated text and
locale decisions flow through) is the locus of maximum risk/value. For each finding, mark
`is_load_bearing: true` IFF its file matches the hinge, then apply 10× scrutiny:
- 5× more falsification attempts (H1.1)
- 3× more edge-case hunts (H1.4)
- Mandatory read of the entire hinge file + all DIRECT callers + all DIRECT callees
- Mandatory runtime check: if the hinge breaks, does EVERY screen revert to one language? (verify)

Output `hinge_findings[]` in `verdict.json` (finding_id, is_load_bearing, hinge_reference,
additional_scrutiny, confidence_after_scrutiny).

### H1.3 User-need verification (`--user-need` quote)

If dispatched with `--user-need="<verbatim user complaint>"`, every finding MUST be evaluated
against it ("If a user reported THIS verbatim, is this finding the cause? Does fixing it make the
complaint no longer true?"). Findings unrelated to user-need get demoted one severity (unless
load-bearing); related findings get top fix-plan priority and lead `user_need_match.findings[]`.
If `addressed: false`, the audit MUST score below 90/100 even if all phases otherwise pass.

```jsonc
{
  "user_need_match": {
    "addressed": true,
    "user_need_quote": "the French site still shows English buttons on checkout",
    "rationale": "F-012 (Checkout.tsx:88 'Pay now' hardcoded) + F-019 (checkout.* keys missing in fr.json) directly cause English buttons on the French checkout. Both fixes remove the cause.",
    "findings": ["F-012", "F-019"],
    "untouched_findings_relevant_to_user_need": []
  }
}
```

### H1.4 Edge case hunting (mandatory for top findings)

For each top-5 finding (severity × cross-audit-confirmed × hinge), generate ≥2 edge cases the tool
may have missed. Static analysis checked the catalog at rest; you must imagine motion. i18n-specific
patterns:
- "User switches locale mid-form..." — does entered data + validation messages re-localize?
- "Locale has the key but a different plural category at count=N..." — Phase 6 runtime
- "RTL locale + an embedded LTR phone number..." — bidi scramble (Phase 9)
- "CJK string truncated by .slice(0,10)..." — surrogate split (Phase 12)
- "Number input typed with locale separators..." — parse corruption (Phase 5)
- "DST/UTC-midnight + server-TZ formatting..." — wrong day (Phase 4)
- "Lazy catalog not yet loaded on first paint..." — FOUC raw keys (Phase 16)
- "Turkish locale + case-insensitive username compare..." — i↔İ auth bug (Phase 13)

Output `edge_cases[]` in `verdict.json` (finding_id, scenario, covered_by_existing_test,
evidence_gathered, fix_includes_coverage).

### H1.5 Cross-audit synthesis (re-read sibling summaries from Phase 0.5)

With your `verdict.json` draft, do a final pass with sibling audits' findings in context: for each
of your top-5 findings, is the same file/line/symbol flagged by copyaudit / a11yaudit / uiuxaudit /
seoaudit / dataaudit? If yes → `cross_audit_confirmed: true`, bump severity one level. For each
sibling top-5 relevant to localization, add it to your findings as
`tool: "cross-audit:<sibling-name>"`. Write `cross_audit_links[]` summarizing matches
(this_finding_id, sibling_audit, sibling_finding_id, shared_location, joint_fix_recommended).

### H1.6 Final verdict.json schema (hybrid v2)

```jsonc
{
  "audit": "i18n",
  "score": 100,
  "score_raw": "<raw>/360",
  "score_normalized": 100,
  "confidence": "high|medium|low",
  "skill_used": "i18n",
  "preamble_version": "1.0",
  "user_need_match": { ... },                   // H1.3
  "falsifiable_tests": [ ... ],                  // H1.1
  "hinge_findings": [ ... ],                     // H1.2
  "issues_found_and_fixed": [
    { "id": "FIX-001", "finding_id": "F-012", "before": "<state>", "after": "<state>",
      "verification": "<command + output>" }
  ],
  "edge_cases": [ ... ],                         // H1.4
  "cross_audit_links": [ ... ],                  // H1.5
  "evidence_summary_path": "$PROJECT_PATH/audits/.i18naudit/evidence-summary.json",
  "confidence_basis": "Why I'm confident (or not). Cite Popper test counts, hinge scrutiny depth, runtime locale-walk coverage, edge-case coverage, cross-audit confirmations.",
  "banned_phrase_check": "passed (no `looks correct`, `should be fine`, `appears to work`, `streamlined`, `to save time`)"
}
```

### H1.7 Score gating (hybrid threshold)

100/100 is blocked unless:
1. ✅ All `critical`/`high` findings fixed OR have a ≥50-word `non_issue_justification` backed by Popper evidence.
2. ✅ All load-bearing findings (H1.2) confirmed via Popper falsification.
3. ✅ `user_need_match.addressed = true` with verbatim quote.
4. ✅ ≥3 falsifiable tests cited per phase (H1.1).
5. ✅ ≥2 edge cases per top-5 finding (H1.4).
6. ✅ Cross-audit synthesis attempted (H1.5) — array present (may be empty if no siblings).
7. ✅ A runtime locale-walk (Phase 16) was performed for every declared non-source locale, OR an
      explicit reason recorded (e.g. no deploy URL available) — `confidence` capped at `medium` if skipped.
8. ✅ `confidence_basis` populated with non-trivial reasoning.

Below threshold → score < 100, fix-and-reaudit loop kicks in (bounded at 5 iterations per the Audit
Verification Contract; on iteration 5 if still failing, emit `confidence: low` and surface as
`pending` in `.done.json`).

---

## Dynamic-Workflow Orchestration (v2)

> *"7,000 languages do not arrive single-file. Neither should the audit that hunts them. Fan out across every locale at once, then let no finding live unless it survives an adversarial cross-examination."*

This section governs **HOW** the audit RUNS. It does not add, remove, reorder, or reweight any
phase — the 16 domain phases, Phase 0/0b gather, Phase H1 synthesis, and the Phase 17 `/360`
scoring matrix are all **unchanged**. It replaces the *linear* phase-by-phase walk with a
**fan-out → adversarial-verify → synthesize → loop-until-dry** execution, in service of the same
Gestalt-Popper doctrine, the same LOCALIZATION HINGE POINT, and the same verdict.

Run this via the **Workflow** tool (in-process fan-out; never one-worker-per-phase dispatch).
Sequencing constraint: Phase 0/0b (the deterministic gather + reconnaissance) ALWAYS runs first and
feeds every track its `evidence-summary.json` + `i18n-architecture.json`. Phase H1 + Phase 17 run
**after** every track has converged. Honor R-SCOPE: tracks share read access to the catalogs but
any FIX execution (Phase 18) is serialized per file via worktree isolation — never two tracks
editing the same catalog or component concurrently.

### 1. Decompose into independent parallel tracks (fan-out)

After Phase 0/0b, partition the 16 domain phases into file-/concern-disjoint tracks that share no
mutable state, and run them **concurrently**. Recommended decomposition for THIS domain (a track may
be split further per-locale when the locale set is large):

- **Track A — String Surface** (Phase 1 hardcoded strings + Phase 2 framework wiring + Phase 8
  interpolation/concatenation). Owns: every user-facing literal, every `t()` key-existence proof,
  every concatenated fragment. Disjoint input: components + the source catalog.
- **Track B — Locale Plumbing** (Phase 3 routing/detection + Phase 11 fallback chain, plus the
  runtime `<html lang>`/`dir` correctness shared with Track E). Owns: how a locale is reached,
  persisted, and degraded.
- **Track C — Formatting & Grammar** (Phase 4 date/time + Phase 5 number/unit + Phase 5b currency +
  Phase 6 pluralization + Phase 7 gender/agreement). Owns: every `Intl.*` / `toLocale*` call site and
  every CLDR plural/select decision. Disjoint input: formatter call sites + per-locale plural data.
- **Track D — Encoding & Collation** (Phase 12 encoding + Phase 13 sorting/collation). Owns: the
  byte-to-glyph journey + every locale-aware compare/sort. Disjoint input: catalog file encodings +
  sort/compare call sites + DB collation.
- **Track E — Rendered Reality** (Phase 9 RTL + Phase 14 layout/expansion + Phase 15 locale-data &
  assets + Phase 16 runtime untranslated-UI leakage). Owns: everything proven only at runtime via a
  per-locale Playwright locale-walk. This track is the **detective pass** that falsifies A/B/C.

Run **one parallel branch PER DECLARED NON-SOURCE LOCALE** inside the locale-sensitive tracks
(B/C/E): an Arabic branch, a French branch, a Japanese branch, etc., execute simultaneously rather
than sequentially — the monolingual assumption is exposed fastest when every locale is read at once.

**Hinge override (Gestalt):** the LOCALIZATION HINGE POINT identified in Phase 0b.3 (the
provider/hook/middleware/`t()` boundary all translated text and locale decisions flow through) is
NOT confined to one track. Every track applies 10× scrutiny to findings whose file matches the
hinge (per Phase H1.2), because if the hinge falls, every track's screen reverts to one language.

### 2. Adversarially verify every finding through ≥2-of-3 independent lenses

No finding from any track enters the verdict until it **survives ≥2 of 3 independent lenses**. A
finding that fails to clear 2-of-3 is **killed** (demoted to `info` with `falsified_at`, or dropped)
— this extends Phase H1.1 Popper falsification from "high/critical only" to **every** surviving
finding, and satisfies R-VERIFY (a track's own "found it" is an input, never the verdict).

The three lenses, tailored to localization (each must be an independent method, not the same check
re-run):

- **Lens 1 — REPRODUCE (runtime):** make the defect happen on a real surface. Switch to the target
  locale via Playwright CLI and load the route → the raw key / English bleed / `{name}` literal /
  `Invalid Date` / scrambled bidi / overflow actually renders (screenshot + exact leaked text +
  route). For a non-UI catalog defect, reproduce by executing the formatter/`Intl.PluralRules` /
  round-tripping the torture string and capturing verbatim output.
- **Lens 2 — REFUTE (Popper counter-example):** actively try to PROVE the finding wrong. Is the
  "hardcoded" literal a className/testId/log (not user-facing)? Does the "missing key" fall back
  gracefully to source text rather than showing the raw key? Is the "untranslated" value a proper
  noun / "OK" / "Email" that is legitimately identical? Does a global default locale make the
  arg-less `toLocaleDateString` actually correct? Is the dynamic key `t(\`${x}\`)`'s full value space
  in fact covered? If the refutation succeeds → **kill the finding.**
- **Lens 3 — CROSS-CHECK (independent corroboration):** confirm via a different evidence source than
  Lens 1. The static gather's `evidence-summary.json` ↔ the runtime locale-walk ↔ the console
  `missingKey`/`Missing message` telemetry ↔ the per-locale catalog key-diff ↔ a sibling audit's
  finding on the same file:line (copyaudit clarity / a11yaudit lang+dir / uiuxaudit overflow /
  dataaudit DB encoding-collation, per Phase H1.5). Two independent sources naming the same
  defect = corroborated; bump severity one level and set `cross_audit_confirmed: true` where a
  sibling agrees.

Record the outcome on each finding: `lenses_passed: ["reproduce","cross-check"]`,
`verification_outcome: confirmed|killed|inconclusive`. `inconclusive` (a lens couldn't run cleanly —
e.g. no deploy URL for the reproduce lens) keeps the finding but caps its `confidence` at `medium`
and the run's `confidence` per Phase H1.7. Load-bearing (hinge) findings demand all 3 lenses, not 2.

### 3. Synthesize survivors back into THIS audit's scoring matrix + verdict (unchanged)

The fan-out and the lenses change only the *path* to the verdict, never the verdict itself. Merge
every track's surviving findings into the single `verdict.json`/`fix-plan.json`, de-duplicating
findings two tracks both surfaced (e.g. a string flagged by Track A as unwrapped AND by Track E as
leaking at runtime is ONE finding with two confirmations). Then score **exactly** the existing
Phase 17 matrix — Phase 1 ×3.5, Phase 2 ×2.5, … Phase 16 ×2.0, `TOTAL = 360`,
`normalized = (raw / 360) × 100` — with the identical grade bands (S/A/B/C/D/F) and the **70 (B)
PASS threshold**. The Phase H1 hybrid `verdict.json` schema, the H1.7 score gates, and the Phase 18
DO-NO-HARM fix loop are untouched: parallelism feeds them better-verified findings, it does not
alter what they compute. Only findings with `verification_outcome: confirmed` (or `inconclusive`
explicitly carried at capped confidence) contribute to phase deductions; killed findings leave **no**
mark on the score.

### 4. Loop-until-dry for unknown-size discovery

The localization surface has no a-priori size: the locale set, the catalog key space, the count of
hardcoded literals, and the number of dynamic-key value-spaces are all discovered, not given. Run
the locale-sensitive tracks (and the hardcoded-string + dynamic-key sweeps) as a **loop-until-dry**
inside the workflow:

```
repeat:
  fan out the tracks across all currently-known locales + routes + catalogs
  adversarially verify (≥2-of-3) every new finding
  feed survivors to the running verdict
until a full pass yields ZERO new confirmed findings
       AND no newly-discovered locale/route/catalog/dynamic-key-space remains unwalked
       (hard cap: the Audit Verification Contract's 5 fix-and-reaudit iterations;
        on cap → mark remaining NEEDS_REVIEW, emit confidence: low, surface `pending`)
```

A pass that discovers a new locale, a lazily-loaded catalog, a new dynamic-key branch, or a new
runtime surface (toast/async panel/empty state) re-arms the loop for that newly-found territory
only — never re-grep what the gather already covered (Phase 0.4 ban stands). The loop ends **dry**
(no new confirmed finding + nothing left unwalked), not on a guess.

---

## PHASE 17: VERDICT

Score each phase 0-10, weight by severity:

```
SCORING MATRIX (360 max):
  Phase  1  (Hardcoded Strings)        x 3.5  = max 35
  Phase  2  (Framework Wiring)         x 2.5  = max 25
  Phase  3  (Locale Routing)           x 2.5  = max 25
  Phase  4  (Date/Time Formatting)     x 2.0  = max 20
  Phase  5  (Number/Unit Formatting)   x 1.5  = max 15
  Phase  5b (Currency Formatting)      x 1.5  = max 15
  Phase  6  (Pluralization)            x 2.5  = max 25
  Phase  7  (Gender/Grammar)           x 1.5  = max 15
  Phase  8  (Interpolation/Concat)     x 2.0  = max 20
  Phase  9  (RTL Support)              x 2.5  = max 25
  Phase 10  (Translation Completeness) x 3.0  = max 30
  Phase 11  (Fallback Chain)           x 2.0  = max 20
  Phase 12  (Encoding Integrity)       x 2.5  = max 25
  Phase 13  (Collation/Sorting)        x 1.5  = max 15
  Phase 14  (Layout/Expansion)         x 1.5  = max 15
  Phase 15  (Locale Data/Assets)       x 1.5  = max 15
  Phase 16  (Runtime Leakage)          x 2.0  = max 20
                                       TOTAL  = max 360

NORMALIZE: score = (raw / 360) x 100

GRADE:
  90-100: S — World-ready. Any declared locale renders correctly, RTL + CJK + formatting + plurals all sound.
  80-89:  A — Globalized. Minor formatting/asset gaps; core experience correct in every locale.
  70-79:  B — Translatable. Wiring solid, but completeness/RTL/formatting gaps reach real users.
  60-69:  C — Localizing. Framework present, but hardcoded strings + missing keys leak at runtime.
  50-59:  D — Monolingual-leaning. i18n bolted on; switching locale exposes English everywhere.
  <50:    F — Monolingual. No usable i18n architecture, or declared locales are a façade.
```

PASS THRESHOLD: **70** (B). Below 70 = not world-ready; the fix-and-reaudit loop runs until ≥70 or
remaining items are NEEDS_REVIEW.

---

## PHASE 18: FIX PLAN → FIX EXECUTION → RE-AUDIT

### Fix Plan (automatic)

```
Sort: CRITICAL -> HIGH -> MEDIUM -> LOW
Priority by user reach:
  CRITICAL: every user of a declared locale sees it (raw keys, hardcoded nav/CTA, broken RTL, mojibake)
  HIGH: core flows in a declared locale (checkout, auth, settings) untranslated/misformatted
  MEDIUM: secondary flows, formatting edge cases, asset localization
  LOW: defense-in-depth (pseudo-loc, CI lint, collation hardening)
Group by blast radius (one extraction-of-component may close 10 hardcoded findings;
  one fallback-config fix may resolve a whole class of raw-key leaks).
Dependency order: fix framework wiring (Phase 2) + fallback (Phase 11) BEFORE chasing individual
  missing keys — the wiring fix may resolve many leakage findings at once.
Generate fix tasks with file:line specificity. Save to audits/.i18naudit/fix-plan.{json,md}.
```

### Fix Execution (automatic) — DO NO HARM gate

```
Read ../_shared/AUDIT-VERIFICATION-CONTRACT.md before ANY fix execution.

─── SAFETY GATE (MANDATORY before EVERY fix) ───────────────────────────
PRE-FIX:
  a. Read the ENTIRE target file (not just the line).
  b. SCOPE/IMPORT CHECK — adding a t()/useTranslations import? confirm no name shadow; confirm the
     i18n provider is in scope at this component (else the t() call throws at runtime).
  c. KEY-ADD CHECK — adding a new catalog key? add it to the SOURCE catalog AND every declared locale
     (use source text as a temporary value + mark untranslated), so no new missing-key regression.
  d. CROSS-REFERENCE — renaming a key? grep every t('old.key') call site + every catalog; update all.
POST-FIX:
  a. SYNTAX: catalogs are valid JSON/PO/YAML (jsonlint); code typechecks (tsc --noEmit / build).
  b. KEY INTEGRITY: re-run the key-diff — the fix must not introduce NEW missing keys in any locale.
  c. RUNTIME SMOKE: if a deploy/dev URL exists, load the affected route under the affected locale
     (Playwright CLI) and confirm the string now renders translated AND no raw key / FOUC appears.
  d. NO REGRESSION: confirm the SOURCE locale still renders identically (before-after.md).
IF ANY CHECK FAILS → git revert HEAD → log in fix-log.md → mark NEEDS_REVIEW (no blind retry).
────────────────────────────────────────────────────────────────────────

FOR EACH FIX TASK (priority order):
  a. Read full file. b. PRE-FIX gate. c. Document BEFORE (the leak/misformat, with evidence).
  d. Apply fix (wrap string + add key to ALL locales / swap to Intl formatter / add logical CSS +
     dir / fix fallback config / normalize encoding). e. POST-FIX gate.
  f. Green → commit: i18n(i18naudit): FIX-XXX description.  g. Red → revert → log → NEEDS_REVIEW.
  h. Document AFTER (same locale/route now correct).
NOTE: translation VALUES require a human/translator; this audit's automatic fixes WRAP strings,
ADD keys (with source text as placeholder marked "[untranslated]"), and fix FORMATTING/WIRING/RTL/
ENCODING — it never fabricates translations. Fabricated translations = NEEDS_REVIEW handoff.
```

### Re-Audit (automatic)

```
1. SERVICE HEALTH GATE: build must pass; if a service exists, restart + verify healthy.
2. KEY INTEGRITY GATE: per-locale key-diff shows no NEW missing keys vs the start of the run.
3. RUNTIME GATE: re-walk the affected locales (Phase 16) — confirm the fixed leaks are gone and no
   new leak/FOUC/raw-key/mojibake appeared (First Law: runtime, not the diff, is the proof).
4. Re-run all FAILING phases. Compare before/after. Loop until score >= 70 or remaining = NEEDS_REVIEW.
5. Produce audits/.i18naudit/before-after.md with per-finding functional status (zero regressions required).
```

---

## CROSS-COMMAND BRIDGE

```
/i18naudit finds an unwrapped string -> /copyaudit owns its clarity, this owns its translatability
/i18naudit finds a missing <html lang>/dir -> /a11yaudit + /seoaudit also care; joint fix
/i18naudit finds layout overflow on long text -> /uiuxaudit consistency finding
/i18naudit finds encoding/collation of stored data -> /dataaudit owns the DB side
/i18naudit finds locale-routing/hreflang -> /seoaudit owns ranking, this owns routing correctness

THE QUALITY ARSENAL:
  /codeaudit    -> Is the code SOLID?           (preventive)
  /flowaudit    -> Does the experience WORK?    (preventive)
  /uiuxaudit    -> Is the interface BEAUTIFUL?  (preventive)
  /copyaudit    -> Is the copy CLEAR?           (preventive)
  /i18naudit    -> Is it WORLD-READY?           (preventive)
  /debugaudit   -> What is BROKEN right now?    (detective)
  /secaudit     -> Is it SECURE?                (detective)

  Together: nothing escapes. Every dimension covered — including the other 95% of the planet.
```

---

## LAWS

1. **A string in source is a string no translator sees.** Every user-facing literal MUST live in a catalog. Hardcoded text is untranslatable, not "untranslated".
2. **English is one locale, not the default of reality (Popper).** FALSIFY "it's translated" by switching the locale and reading every screen at runtime.
3. **Grammar is not substitution.** Plurals, gender, agreement, and word order cannot survive concatenation. One message with placeholders, or it's broken abroad.
4. **Formatting is locale, never preference.** Dates, numbers, currencies rendered without a locale-aware formatter lie to the user silently.
5. **RTL is layout, not a checkbox.** Declaring `ar` without `dir` + logical CSS + bidi isolation + icon mirroring is declaring a language you didn't build.
6. **The encoding is guilty until round-tripped (Popper).** UTF-8 end to end, NFC-normalized, astral-safe — proven by a torture-string round trip, not by a `<meta charset>` tag.
7. **The weakest locale is the experience (Popper).** You shipped 5 languages; the one with missing keys and broken RTL is the one a real user is reading right now. FALSIFY "world-ready" by finding the locale that breaks.

---

*"/i18naudit v1 — Extract. Route. Pluralize. Format. Mirror. Encode. Collate. Every string, every locale, every direction. /360."*

---

## COMPLIANCE & CRITICAL ADDENDA (v1.0)

### Quality Arsenal Preamble Compliance

This audit implements contracts defined in `../_shared/QUALITY-ARSENAL-PREAMBLE.md` v1.0:

- ✅ **Gestalt-Popper doctrine** — localization hinge point, falsification, runtime evidence chain, adversarial "switch the locale" mindset
- ✅ **Concurrency lock** — canonical runner `flock` at `audits/.i18naudit/.runner.lock`
- ✅ **5-iteration cap** — fix-and-reaudit loop bounded at 5 iterations. On cap: NEEDS_REVIEW + Telegram SOS. No silent infinite loops.
- ✅ **Scoped invocation flags** — `--url=`, `--files=`, `--scope=`, `--ticket=`, `--no-fix`, `--focus=` (areas: hardcoded | routing | rtl | formatting | completeness | encoding | runtime)
- ✅ **Non-UI context gate** — Backend/CLI/library targets: Phases 1/2/4/5/5b/6/8/10/11/12/13 still apply (server-rendered strings, log-vs-user-facing, formatters, catalogs, encoding). UI-only phases (3/9/14/16 runtime-walk) are marked N/A and EXCLUDED from the normalized denominator (preamble §5). A pure backend with no user-facing strings scores on its formatter/encoding surface only.
- ✅ **Output contract verification** — emits `verdict.json`, `verdict.md`, `fix-plan.json`, `fix-plan.md`, `progress.json`, `fix-log.md`, `before-after.md`. Output gate runs at end; missing/malformed files = audit did NOT succeed.
- ✅ **Telegram progress notifications** — `start` / `progress` (every 3 phases) / `iteration` / `verdict` / `abort` / `sos` events via the configured audit-notify hook
- ✅ **Discovery drift check** — on resumed runs, if `audits/.i18naudit/discovery/` > 1h old, re-verify the locale/catalog inventory or abort with user-confirm
- ✅ **Self-telemetry** — `audits/.i18naudit/telemetry.json` emitted at completion (duration, tokens, phases, fixes, locales_walked, model, preamble_version)
- ✅ **Deprecation registry** — cross-references checked against the deprecation registry; stale refs flagged as findings
- ✅ **Rule-46 compliance** — NO `--quick`/`--streamlined`/`--lightweight` variants. Narrower scope uses `--focus <area>` with FULL phase depth. Orchestrator prompts containing rule-46 banned phrases are REFUSED.
- ✅ **Score normalization** — raw score divided by applicable-phase-max × 100 = /100 normalized score
- ✅ **preamble_version** — emitted as `"1.0"` in verdict.json for `/metaudit` compliance scan

### Audit-Specific Critical Addendum — Translation Honesty + Runtime Proof

- **No fabricated translations.** Automatic fixes WRAP strings, ADD keys (source text as a `[untranslated]`-marked placeholder so no missing-key regression), and fix FORMATTING / WIRING / RTL / ENCODING. They NEVER invent target-language text. Any finding requiring real translation is handed off as NEEDS_REVIEW with the exact key + source string.
- **Runtime proof required for the translation-leakage claim.** A "translated: yes" verdict for a locale REQUIRES a Phase 16 runtime walk (screenshot evidence) for that locale, OR the verdict's `confidence` is capped at `medium` with the reason recorded (no deploy URL, etc.). Static wrapping alone never proves world-readiness (First Law).
- **Encoding round-trip is mandatory before a 10 on Phase 12.** The torture-string round trip must be executed with verbatim output, not asserted.

### /metaudit Compliance Badge

Run `/metaudit --focus arsenal --scope="i18naudit only"` to verify against the 11-point preamble checklist. Target: 11/11.

---

## MANDATORY BEFORE/AFTER VERIFICATION (v1.0)

**Read `../_shared/AUDIT-VERIFICATION-CONTRACT.md` before ANY fix execution.**

Every fix MUST follow the "Do No Harm" protocol:

1. **PRE-FIX BASELINE** — capture per-locale key counts, the source-locale rendered state, and any affected-route screenshots; save to `audits/.i18naudit/baseline/`.
2. **APPLY FIX** — normal execution.
3. **POST-FIX CHECK** — repeat every baseline check; if any PASSED→FAILED transition (new missing key, source-locale regression, new runtime leak) occurs, revert immediately.
4. **BREAKAGE SCAN** — re-run the per-locale key-diff; zero NEW missing keys in any locale.
5. **BEFORE/AFTER MATRIX** — produce `audits/.i18naudit/before-after.md` with functional status per affected string/locale.

**An audit that breaks 1 working locale is WORSE than no audit.** Do NOT claim "done" without `before-after.md` showing zero regressions.
