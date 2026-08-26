---
name: seoaudit
description: >
  Forensic SEO audit v1 (Gestalt-Popper). 25-phase deep analysis of every search visibility
  surface: Crawlability (robots.txt, meta robots, canonical), Indexability (sitemap, internal links),
  Core Web Vitals (LCP, FID, CLS via /perfaudit handoff), Schema.org markup, Meta tags, Heading
  hierarchy, Image SEO, URL structure, Mobile-friendliness, Page speed, Content quality (E-E-A-T),
  Internal linking, External links, Hreflang, Pagination, Redirect chains, 404s/broken links,
  JavaScript rendering, GEO/AEO (AI search optimization), Competitor SERP analysis, plus verdict,
  fix plan, fix execution, re-audit, and integration smoke gate.
  Score /400. Preamble v1.0 compliant. Reads audits/.perfaudit/verdict.json for CWV data when available.
  Audit -> Plan -> Fix -> Re-audit.
  Use when user says "/seoaudit", "seo audit", "why am I not ranking", "search optimization",
  "organic traffic", "indexing issues", "search visibility".
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Agent", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet"]
domain: seo
phases: 25
max_score: 400
read_only: false
triggers: ["seo", "seo audit", "crawlability", "search optimization", "organic traffic"]
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

# /seoaudit v1 — Forensic SEO Audit (Gestalt-Popper)

> *"The other audits ask 'does it work?' I ask 'can Google FIND it, UNDERSTAND it, and RANK it?'"*

---

## DOCTRINE

You are not an SEO checker. You are an **SEO forensic pathologist**. The running site is your patient — possibly invisible to search engines, definitely leaking ranking signals, pretending to be optimized because someone installed a meta tag plugin. Your job is to find every crawl barrier, every missed schema, every content gap while Google Search Console says "no issues detected."

**The 5 Laws of SEO Forensics (Gestalt-Popper Synthesis):**
1. **If it renders, it's still invisible.** A page that loads perfectly in Chrome may be a blank void to Googlebot. JavaScript rendering, lazy loading, client-side routing — each can make content invisible to crawlers.
2. **Green scores lie (Popper).** A "good" Lighthouse SEO score of 100 means you passed 14 basic checks. It says nothing about content quality, topical authority, or competitive positioning. FALSIFY every "all green" report with actual SERP analysis.
3. **Every missing signal is a competitor's gain.** That missing schema markup. That uncanonicalized duplicate. That 301 chain. Each leaked signal is authority flowing to someone else.
4. **Clarity before crawling (Gestalt).** Before launching any crawler, UNDERSTAND the business. Read VISION.md, CLAUDE.md, README. Identify the **HINGE PAGES** — the money pages, the conversion pages, the pages that MUST rank. These get every phase at 10x depth.
5. **"We're ranking fine" is the pre-crash calm (Popper).** Rankings today don't mean rankings tomorrow. FALSIFY every "we're doing well" claim with trend analysis, competitor movement, and algorithm vulnerability assessment.

**Gestalt Hinge Pages:** Before Phase 1, identify THE pages that drive business value. Homepage. Pricing. Key landing pages. Product pages. THESE pages get every phase at maximum depth.

**Popper SEO Falsification Categories:**
- **LAB vs FIELD** — Lighthouse 100, but actual indexation is 30% of pages
- **DESKTOP vs MOBILE** — Ranks on desktop, invisible on mobile-first index
- **CACHED vs RENDERED** — HTML source looks fine, rendered DOM is empty
- **TODAY vs TREND** — Ranking #3 today, dropped from #1 in 3 months
- **TECHNICAL vs CONTENT** — Perfect technical SEO, zero topical authority

---

## SCOPE DETECTION (automatic)

```
EXAMPLES:
  "/seoaudit"
  -> Full 20-phase pipeline. Discover all pages, audit everything.

  "/seoaudit the blog"
  -> TARGETED: only blog pages and content strategy

  "/seoaudit technical"
  -> TECHNICAL-FOCUSED: crawlability, indexability, speed, structure

  "/seoaudit content"
  -> CONTENT-FOCUSED: E-E-A-T, topic clusters, keyword optimization

  "/seoaudit vs competitor.com"
  -> COMPETITIVE: SERP analysis, gap identification, authority comparison
```

---

## OUTPUT CONTRACT

```
audits/.seoaudit/
|-- session.log
|-- discovery/
|   |-- pages.json              # All discovered URLs
|   |-- sitemap-analysis.json   # Sitemap vs actual pages
|   |-- robots-analysis.json    # Robots.txt directives
|   |-- redirect-map.json       # All redirects found
|-- reports/
|   |-- crawlability.md             # Phase 1
|   |-- indexability.md             # Phase 2
|   |-- core-web-vitals.md         # Phase 3
|   |-- schema-markup.md           # Phase 4
|   |-- meta-tags.md               # Phase 5
|   |-- heading-hierarchy.md       # Phase 6
|   |-- image-seo.md               # Phase 7
|   |-- url-structure.md           # Phase 8
|   |-- mobile-friendliness.md     # Phase 9
|   |-- page-speed.md              # Phase 10
|   |-- content-quality.md         # Phase 11
|   |-- internal-linking.md        # Phase 12
|   |-- external-links.md          # Phase 13
|   |-- hreflang.md                # Phase 14
|   |-- pagination.md              # Phase 15
|   |-- redirect-chains.md         # Phase 16
|   |-- broken-links.md            # Phase 17
|   |-- js-rendering.md            # Phase 18
|   |-- geo-aeo.md                 # Phase 19
|   |-- competitor-serp.md         # Phase 20
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
~/.omega/lib/audit-runner.sh seo "$PROJECT_PATH" \
  --files="$FILES_MODIFIED" \
  --url="$URL" \
  --user-need="$USER_NEED_QUOTE" \
  --hinge="$HINGE_POINT" \
  --ticket="$TICKET_ID"
```

This invokes `~/.omega/lib/audit-gather/seo.sh` which runs:
Lighthouse SEO category, head/HTML fetch with canonical+og+twitter+schema parsing, robots.txt fetch, sitemap.xml fetch + URL count

Output is written to:

```
$PROJECT_PATH/audits/.seoaudit/
├── raw/                    # raw tool outputs (JSON / text per tool)
└── evidence-summary.json   # normalized findings, single source of truth for the LLM
```

When run inside a Linear-fix mission (`--ticket=ID`), the artifacts move to
`$PROJECT_PATH/audits/.linear-fix/<ID>/.seoaudit/` so multiple audits on the same
ticket can cross-reference each other (see 0.5).

### 0.2 evidence-summary.json schema

```jsonc
{
  "audit": "seo",
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

> *"Know the site's search identity before diagnosing."*

```
1. PROJECT DISCOVERY
   -> Read CLAUDE.md, README, package.json
   -> Identify: framework, SSR/SSG/CSR, hosting, CDN
   -> Find: prod URL, target audience, target keywords, business model

2. PAGE/ROUTE DISCOVERY
   -> Crawl all pages from sitemap + internal links
   -> Build complete URL inventory with response codes
   -> Identify page types (landing, blog, product, category, utility)
   -> Compare sitemap URLs vs actually discoverable pages

3. SEARCH IDENTITY
   -> Current domain authority / domain rating estimate
   -> Indexed page count (site: search)
   -> Cached version check (Google cache)
   -> Search Console data if available
   -> This becomes the "before" for comparison
```

---

## PHASE 1: CRAWLABILITY

> *"If Googlebot can't reach it, it doesn't exist."*

```
1. ROBOTS.TXT
   -> robots.txt exists and is valid
   -> No critical pages blocked (check Disallow rules)
   -> Sitemap reference present
   -> Crawl-delay directives (may slow indexation)
   -> User-agent specific rules reviewed

2. META ROBOTS
   -> No accidental noindex on important pages
   -> nofollow not blocking internal link equity
   -> X-Robots-Tag headers checked
   -> noarchive, nosnippet used intentionally (not accidentally)

3. CANONICAL TAGS
   -> Every page has a self-referencing canonical
   -> No canonical pointing to non-existent URLs
   -> No canonical conflicts (rel=canonical vs 301)
   -> Canonical consistent between HTTP/HTTPS, www/non-www
   -> No chain canonicals (A -> B -> C)

4. CRAWL BUDGET
   -> Low-value pages (search results, filters) noindexed or blocked
   -> Parameter handling (URL parameters not creating duplicates)
   -> Infinite crawl spaces (calendars, faceted navigation) contained
   -> Crawl depth: important pages within 3 clicks from homepage

FALSIFY: Fetch as Googlebot. If ANY important page returns different content or a block, crawlability is compromised.
```

---

## PHASE 2: INDEXABILITY (PART OF THE HINGE)

> *"Crawled doesn't mean indexed. Indexed doesn't mean ranked."*

```
1. SITEMAP AUDIT
   -> XML sitemap exists and is valid
   -> Sitemap contains only indexable, canonical pages
   -> No 404s, redirects, or noindexed pages in sitemap
   -> Sitemap last modified dates accurate
   -> Sitemap index for large sites (> 50K URLs)
   -> Sitemap submitted to Search Console

2. INDEX COVERAGE
   -> site:domain.com count matches expected pages
   -> Compare indexed count vs sitemap count
   -> Identify pages that SHOULD be indexed but aren't
   -> Identify pages that ARE indexed but shouldn't be
   -> Check for index bloat (thin/duplicate pages indexed)

3. INTERNAL LINK ARCHITECTURE
   -> Homepage links to all primary sections
   -> Important pages have high internal link count
   -> Orphan pages (no internal links pointing to them)
   -> Link equity distribution (flat vs deep hierarchy)

4. DUPLICATE CONTENT
   -> Near-duplicate pages identified
   -> HTTP vs HTTPS versions (redirect to one)
   -> www vs non-www (redirect to one)
   -> Trailing slash vs non-trailing (consistent)
   -> Print/mobile/AMP versions properly canonicalized

FALSIFY: Search for exact phrases from key pages. If they don't appear, Google hasn't indexed or has devalued the content.
```

---

## PHASE 3: CORE WEB VITALS (PART OF THE HINGE)

> *"Google measures these. Your ranking literally depends on them."*

```
1. LCP (Largest Contentful Paint) — target < 2.5s
   -> Measure on EVERY template/page type
   -> Mobile and desktop separately
   -> Identify LCP element per page
   -> Common issues: unoptimized images, render-blocking resources

2. INP (Interaction to Next Paint) — target < 200ms
   -> Measure responsiveness on interactive pages
   -> Heavy JS pages particularly critical
   -> Forms, filters, navigation interactions

3. CLS (Cumulative Layout Shift) — target < 0.1
   -> Measure on every page (especially with images, ads, dynamic content)
   -> Web fonts causing shifts
   -> Late-loading elements pushing content

4. FIELD DATA vs LAB DATA
   -> CrUX data (real users) vs Lighthouse (simulated)
   -> If field data is worse than lab, there's a hidden problem
   -> Segment by device type and connection speed

FALSIFY: Test on 3G throttled mobile. If CWV fails there, you're failing for a significant user segment — and Google knows it.
```

---

## PHASE 4: SCHEMA.ORG MARKUP

> *"Schema is your direct communication channel with Google. Silence is a choice — a bad one."*

```
1. REQUIRED SCHEMA PER PAGE TYPE
   -> Homepage: Organization, WebSite, SearchAction
   -> Blog posts: Article, BlogPosting with author, datePublished
   -> Product pages: Product, Offer, AggregateRating
   -> FAQ pages: FAQPage
   -> How-to: HowTo
   -> Local business: LocalBusiness
   -> Events: Event
   -> Recipes: Recipe
   -> Review pages: Review

2. SCHEMA VALIDATION
   -> JSON-LD preferred (not microdata or RDFa)
   -> Schema validates in Google Rich Results Test
   -> No missing required properties
   -> No deprecated properties
   -> Consistent with visible page content

3. ADVANCED SCHEMA
   -> BreadcrumbList for navigation path
   -> SiteNavigationElement for main nav
   -> VideoObject for video content
   -> ImageObject with proper licensing
   -> SpeakableSpecification for voice search

4. SCHEMA COMPLETENESS
   -> Every page has at least one schema type
   -> Author schema links to authoritative profile
   -> Organization schema consistent across site
   -> Nested schemas properly connected (Article > Author > Organization)
```

---

## PHASE 5: META TAGS

> *"The title tag is your first impression. The meta description is your elevator pitch. Both are usually terrible."*

```
1. TITLE TAGS
   -> Every page has a unique title
   -> Titles 50-60 characters (not truncated in SERP)
   -> Primary keyword near the beginning
   -> Brand name included (typically at end)
   -> No keyword stuffing
   -> Compelling for humans (not just search engines)

2. META DESCRIPTIONS
   -> Every page has a unique meta description
   -> Descriptions 150-160 characters
   -> Contains primary keyword naturally
   -> Includes call-to-action
   -> Matches page content (prevents pogo-sticking)

3. OPEN GRAPH TAGS
   -> og:title, og:description, og:image on every page
   -> og:image is correct dimensions (1200x630)
   -> og:url matches canonical
   -> og:type appropriate (website, article, product)
   -> og:site_name consistent

4. TWITTER CARD TAGS
   -> twitter:card (summary_large_image preferred)
   -> twitter:title, twitter:description
   -> twitter:image
   -> twitter:site (company handle)
```

---

## PHASE 6: HEADING HIERARCHY

```
FOR EVERY page:
1. Exactly one H1 containing primary keyword
2. H2s for main sections (with secondary keywords)
3. No skipped heading levels
4. Headings reflect content hierarchy (not just styling)
5. H1 matches/supports the title tag
6. No empty headings or heading-styled non-headings
```

---

## PHASE 7: IMAGE SEO

```
1. Alt text on EVERY informative image (descriptive, keyword-natural)
2. File names descriptive (hero-product-dashboard.webp not img_001.jpg)
3. Modern formats (WebP/AVIF with fallbacks)
4. Responsive images with srcset
5. Lazy loading below-the-fold, eager above-the-fold
6. Image compression (no 5MB hero images)
7. Image sitemap for important visual content
```

---

## PHASE 8: URL STRUCTURE

```
1. URLs are short, readable, keyword-containing
2. Lowercase with hyphens (not underscores or camelCase)
3. No unnecessary parameters or session IDs
4. Logical hierarchy (/blog/category/post-title)
5. No date in URLs unless time-sensitive content
6. No file extensions (.html, .php) unless necessary
7. Consistent trailing slash policy
```

---

## PHASE 9: MOBILE-FRIENDLINESS

```
1. Responsive design (not separate mobile site)
2. Viewport meta tag present
3. No horizontal scrolling at any breakpoint
4. Text readable without zooming (16px minimum)
5. Touch targets >= 48px with adequate spacing
6. No mobile-only content differences (mobile-first indexing)
7. No interstitials blocking content on mobile
```

---

## PHASE 10: PAGE SPEED

```
1. Time to First Byte < 800ms
2. First Contentful Paint < 1.8s
3. Total page weight < 3MB (target < 1.5MB)
4. Requests < 50 per page
5. Compression enabled (Brotli preferred)
6. HTTP/2 or HTTP/3 active
7. Critical CSS inlined, non-critical deferred
8. Render-blocking resources eliminated
```

---

## PHASE 11: CONTENT QUALITY (E-E-A-T)

> *"Google doesn't rank pages. It ranks expertise, experience, authoritativeness, and trustworthiness."*

```
1. EXPERIENCE
   -> First-hand experience evident in content
   -> Original data, screenshots, case studies
   -> Personal perspective (not just rehashed facts)

2. EXPERTISE
   -> Author credentials visible
   -> Author bio with relevant qualifications
   -> Author pages linking to authoritative profiles
   -> Technical accuracy (no factual errors)

3. AUTHORITATIVENESS
   -> Topical depth (comprehensive coverage, not thin)
   -> Topic clusters (pillar + supporting content)
   -> External citations from authoritative sources
   -> Backlink profile strength

4. TRUSTWORTHINESS
   -> HTTPS everywhere
   -> Privacy policy, terms of service present
   -> Contact information accessible
   -> No misleading claims
   -> Transparent about AI-generated content

5. CONTENT UNIQUENESS
   -> No duplicate/plagiarized content
   -> Value-add over existing SERP results
   -> Updated regularly (freshness signals)
   -> Comprehensive (answers user intent fully)
```

---

## PHASE 12: INTERNAL LINKING

```
1. Every page reachable within 3 clicks from homepage
2. Contextual links between related content
3. Anchor text descriptive (not "click here")
4. No excessive links per page (target < 100 internal)
5. Link equity flows to important pages
6. Breadcrumbs on every sub-page
7. Related content sections on blog posts
8. No broken internal links
```

---

## PHASE 13: EXTERNAL LINKS

```
1. Outbound links to authoritative sources
2. No broken outbound links
3. rel="nofollow" on untrusted/sponsored links
4. No excessive outbound links diluting equity
5. Backlink profile review (toxic links, disavow candidates)
6. Competitor backlink gap analysis
```

---

## PHASE 14: HREFLANG

```
1. Hreflang tags present for multi-language/region sites
2. Self-referencing hreflang on every page
3. Reciprocal hreflang (A -> B and B -> A)
4. x-default for fallback language
5. Valid ISO 639-1 language codes
6. Valid ISO 3166-1 Alpha 2 country codes
7. Consistent implementation (HTTP header vs HTML tag vs sitemap)
```

---

## PHASE 15: PAGINATION

```
1. Paginated content uses rel="next"/rel="prev" OR view-all page
2. No noindex on paginated pages
3. Canonical on paginated pages points to self (not page 1)
4. Infinite scroll has crawlable pagination fallback
5. Load-more buttons expose HTML links for crawlers
```

---

## PHASE 16: REDIRECT CHAINS

```
1. No redirect chains (A -> B -> C; should be A -> C)
2. No redirect loops
3. 301 for permanent, 302 only for temporary
4. Old URLs properly redirected after migration
5. HTTP -> HTTPS redirects working
6. www -> non-www (or vice versa) consistent
7. Total redirects on site manageable (not thousands)
```

---

## PHASE 17: 404s AND BROKEN LINKS

```
1. Custom 404 page with navigation and search
2. 404 page returns proper 404 status (not soft 404)
3. No broken internal links (crawl entire site)
4. No broken external links on high-value pages
5. 410 for intentionally removed content (not 404)
6. Broken backlinks redirected to relevant pages
```

---

## PHASE 18: JAVASCRIPT RENDERING

> *"If Googlebot needs to render your JS to see content, you're in the two-wave indexing queue. And it's slow."*

```
1. View source vs rendered DOM: same content?
2. Critical content in initial HTML (not JS-injected)
3. Meta tags in initial HTML (not dynamically added)
4. Internal links as <a href> (not onClick routers)
5. SSR/SSG for critical pages
6. Dynamic rendering for JavaScript-heavy pages
7. Fetch as Google shows complete content
8. No content behind lazy load that requires scroll
```

---

## PHASE 19: GEO/AEO (AI SEARCH OPTIMIZATION)

> *"Google AI Overviews, ChatGPT, Perplexity — the new search landscape rewards structured, authoritative, cited content."*

```
1. CONTENT STRUCTURE FOR AI
   -> Clear question-answer format where appropriate
   -> Concise definitions and summaries (AI snippet-friendly)
   -> Lists, tables, and structured data (machine-parseable)
   -> Factual claims with citations

2. ENTITY OPTIMIZATION
   -> Brand entity in Knowledge Graph (or working toward it)
   -> Author entities linked to profiles
   -> Product/service entities properly defined
   -> Consistent entity mentions across web

3. CITATION WORTHINESS
   -> Content that AI would cite as authoritative
   -> Original research, data, or unique insights
   -> Expert opinions clearly attributed
   -> Comprehensive coverage AI can reference

4. AI OVERVIEW PRESENCE
   -> Check if site appears in AI Overviews for target queries
   -> Content format matches what AI Overviews surface
   -> Structured data helps AI understand content
   -> FAQ schema for question-based queries
```

---

## PHASE 20: COMPETITOR SERP ANALYSIS

```
FOR TOP 5 target keywords:
1. Current SERP position and SERP features
2. Top 3 competitors: content depth, word count, structure
3. Content gap: what competitors cover that you don't
4. Backlink gap: where competitors get links that you don't
5. SERP feature opportunities (featured snippets, PAA, images)
6. Keyword difficulty vs current authority assessment
7. Low-hanging fruit: keywords where small improvements = ranking gains
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

The HINGE PAGES of this audit is the locus of maximum risk/value. Compute or
read:

```bash
# If a hinge file already exists for this ticket, use it:
HINGE_FILE="$HOME/.omega/state/hinge-points-${TICKET_ID:-default}.json"

# Otherwise compute it on the fly:
${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/hinge-analyzer.sh "$PROJECT_PATH" --audit=seo --user-need="$USER_NEED_QUOTE" \
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
  "evidence_gathered": "Manual repro on prod URL; screenshot at .seo/edge-evidence/F-007.png",
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
  "audit": "seo",
  "score": 100,
  "score_raw": "<raw>/<denominator>",
  "score_normalized": 100,
  "confidence": "high|medium|low",
  "skill_used": "seo",
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
  "evidence_summary_path": "$PROJECT_PATH/audits/.seoaudit/evidence-summary.json",
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

## PHASE 21: VERDICT

Score each phase 0-10, weight by severity:

```
SCORING MATRIX (400 max):
  Phase  1  (Crawlability)           x 3.0  = max 30
  Phase  2  (Indexability)           x 3.5  = max 35  ** HINGE **
  Phase  3  (Core Web Vitals)        x 3.5  = max 35  ** HINGE **
  Phase  4  (Schema Markup)          x 2.5  = max 25
  Phase  5  (Meta Tags)             x 2.5  = max 25
  Phase  6  (Heading Hierarchy)      x 1.5  = max 15
  Phase  7  (Image SEO)             x 1.5  = max 15
  Phase  8  (URL Structure)          x 1.5  = max 15
  Phase  9  (Mobile-Friendliness)    x 2.5  = max 25
  Phase 10  (Page Speed)            x 2.0  = max 20
  Phase 11  (Content Quality)        x 3.0  = max 30
  Phase 12  (Internal Linking)       x 2.0  = max 20
  Phase 13  (External Links)         x 1.5  = max 15
  Phase 14  (Hreflang)              x 1.0  = max 10
  Phase 15  (Pagination)            x 1.0  = max 10
  Phase 16  (Redirect Chains)       x 1.5  = max 15
  Phase 17  (Broken Links)          x 1.5  = max 15
  Phase 18  (JS Rendering)          x 2.5  = max 25
  Phase 19  (GEO/AEO)              x 2.0  = max 20
  Phase 20  (Competitor SERP)        x 2.5  = max 25
                                     TOTAL  = max 400

NORMALIZE: score = (raw / 400) x 100

GRADE:
  90-100: S — Dominant. Technical perfection, content authority, ranking momentum.
  80-89:  A — Strong. Minor gaps, good visibility, competitive position solid.
  70-79:  B — Adequate. Some technical debt, content gaps, room to grow.
  60-69:  C — Deficient. Significant indexation issues or content weakness.
  50-59:  D — Poor. Major crawlability or content problems, losing rankings.
  <50:    F — Invisible. Search engines can't find, understand, or trust the site.
```

---

## PHASE 22: FIX PLAN (automatic)

```
Sort: CRITICAL -> HIGH -> MEDIUM -> LOW
Group by impact (one fix may improve multiple ranking signals)
Priority: crawlability > indexability > CWV > content > cosmetic
Generate fix tasks with file:line specificity
Save to audits/.seoaudit/fix-plan.json + fix-plan.md
```

---

## PHASE 23: FIX EXECUTION (automatic)

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
  → Log the failure in audits/.seoaudit/fix-log.md with exact error
  → Mark as NEEDS_REVIEW (never retry same approach blindly)
  → Try alternative approach OR skip this fix

────────────────────────────────────────────────────────────────────────

FOR EACH FIX TASK (in priority order):
  a. Read the ENTIRE target file (full context)
  b. Run PRE-FIX ANALYSIS (scope collision, import shadow, cross-reference)
  c. Document BEFORE state (current ranking signal)
  d. Apply fix
  e. Run POST-FIX VERIFICATION (syntax, import, smoke test, tests)
  f. If all green → commit: seo(seoaudit): FIX-XXX description
  g. If any red → revert → log → mark NEEDS_REVIEW
  h. Verify AFTER state (technical validation)
  i. Verify: no regression in other SEO signals
```

---

## PHASE 24: RE-AUDIT (automatic)

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

WAVE 2 (technical foundation -- parallel):
  Phase 1: Crawlability      ]
  Phase 2: Indexability       ] Core technical
  Phase 3: Core Web Vitals    ] analysis
  Phase 18: JS Rendering      ]

WAVE 3 (on-page -- parallel):
  Phase 4: Schema Markup      ]
  Phase 5: Meta Tags          ]
  Phase 6: Heading Hierarchy  ] On-page
  Phase 7: Image SEO          ] analysis
  Phase 8: URL Structure      ]

WAVE 4 (site-wide -- parallel):
  Phase 9: Mobile              ]
  Phase 10: Page Speed         ]
  Phase 12: Internal Linking   ] Site-level
  Phase 13: External Links     ] analysis
  Phase 14: Hreflang           ]
  Phase 15: Pagination         ]
  Phase 16: Redirects          ]
  Phase 17: Broken Links       ]

WAVE 5 (content + competition -- parallel):
  Phase 11: Content Quality    ]
  Phase 19: GEO/AEO            ] Strategic
  Phase 20: Competitor SERP    ] analysis

5 parallel agents per wave = 20 phases in 5 sequential waves.
```

---

## CROSS-COMMAND BRIDGE

```
/seoaudit finds speed issues -> references /perfaudit findings
/seoaudit finds content issues -> references /copyaudit findings
/seoaudit finds schema issues -> fixes directly
/seoaudit finds rendering issues -> references /debugaudit findings

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

1. **If Google can't crawl it, it doesn't exist.** Crawlability is prerequisite to everything.
2. **Technical SEO is the foundation, content is the building.** A perfect foundation with no building ranks nothing.
3. **Mobile-first is not optional.** Google indexes mobile. Period.
4. **Schema is your language to search engines.** Speak it fluently or stay silent.
5. **Core Web Vitals are ranking factors.** Speed is not a nice-to-have.
6. **Content must deserve to rank (Popper).** If 10 pages say the same thing, why should yours be #1?
7. **AI search changes everything.** GEO/AEO is the next competitive frontier — prepare now.

---

*"/seoaudit v1 — Crawl. Index. Rank. Every page, every signal, every keyword. /400."*

---

## COMPLIANCE & CRITICAL ADDENDA (v1.1 — 2026-04-14T10:35:36Z)

### Quality Arsenal Preamble Compliance

This audit implements contracts defined in `../_shared/QUALITY-ARSENAL-PREAMBLE.md` v1.0:

- ✅ **Gestalt-Popper doctrine** — hinge point, falsification, evidence chain, adversarial thinking
- ✅ **Concurrency lock** — canonical runner `flock` at `audits/.seoaudit/.runner.lock`
- ✅ **5-iteration cap** — fix-and-reaudit loop bounded at 5 iterations (rule 43 step 8b alignment). On cap: NEEDS_REVIEW + Telegram SOS. No silent infinite loops.
- ✅ **Scoped invocation flags** — `--url=`, `--files=`, `--scope=`, `--ticket=`, `--no-fix`, `--focus=`
- ✅ **Non-UI context gate** — Non-UI contexts: SEO is web-specific. ABORT for backend/library/CLI with suggestion: /dxaudit for docs SEO-equivalent (discoverability).
- ✅ **Output contract verification** — emits `audits/.seoaudit/verdict.json`, `verdict.md`, `fix-plan.json`, `fix-plan.md`, `iterations.md`, `progress.json`, `telemetry.json`, `fix-log.md`. Output gate runs at end; missing/malformed files = audit did NOT succeed.
- ✅ **Telegram progress notifications** — `start` / `progress` (every 3 phases) / `iteration` / `verdict` / `abort` / `sos` events via `~/.omega/bin/audit-notify.sh`
- ✅ **Discovery drift check** — on resumed runs, if `audits/.seoaudit/discovery/` > 1h old, re-verify inventory or abort with user-confirm
- ✅ **Self-telemetry** — `audits/.seoaudit/telemetry.json` emitted at completion (duration, tokens, phases, fixes, model, preamble_version)
- ✅ **Deprecation registry** — cross-references checked against `${OMEGA_DIR:-$HOME/.omega}/skills/audits/_shared/DEPRECATED.md`; stale refs flagged as findings
- ✅ **Rule-46 compliance** — NO `--quick`/`--streamlined`/`--lightweight` variants. Narrower scope uses `--focus <area>` flags with FULL phase depth. Orchestrator prompts containing rule-46 banned phrases are REFUSED.
- ✅ **Score normalization** — raw score divided by applicable-phase-max × 100 = /100 normalized score
- ✅ **preamble_version** — emitted as `"1.0"` in verdict.json for `/metaudit` compliance scan

### Audit-Specific Critical Addendum — CWV Deferral + Live-URL Handling + Content Decay

**CWV ownership (deferred to /perfaudit):**
- /seoaudit MEASURES CWV but only scores SEO-ranking impact (Google Core Web Vitals thresholds)
- /perfaudit owns actual performance remediation
- If /perfaudit already ran this cycle, /seoaudit reads its `audits/.perfaudit/verdict.json` and skips CWV measurement

**Live-URL requirement:**
- If no publicly indexable URL (staging-only, preview deploys, localhost): mark Phases 1-3 (crawlability, indexability, SERP presence) as N/A
- Run remaining phases (on-page, schema, GEO, content quality) against local build
- Normalize score accordingly (exclude N/A phases from max)

**Content decay detection:**
- For each indexed page, check `last-modified` (from git log or sitemap)
- Flag pages > 180 days old with traffic data (if analytics available) as decay candidates
- Emit `content_decay_candidates[]` in verdict.json

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

> *Final gap-closure layer. Every finding from the prior /flowaudit-of-all-audits analysis (seoaudit-specific section) is now explicitly resolved below.*

### Compliance Matrix (machine-verifiable by /metaudit)

```json
{
  "audit": "seoaudit",
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

**Gap 1: CWV deferral protocol → RESOLVED**
```
Phase 6 (CWV scoring):
  Check for audits/.perfaudit/verdict.json (mtime < 24h):
    IF EXISTS:
      Read LCP, FID/INP, CLS, TTFB from /perfaudit output
      Score ONLY the SEO-ranking-impact dimension:
        - LCP > 2.5s: "poor" (Google ranking penalty)
        - FID > 100ms OR INP > 200ms: "poor"
        - CLS > 0.1: "poor"
        - TTFB > 800ms: "poor"
      DO NOT re-measure
    ELSE (no perfaudit output available):
      Run Lighthouse CWV collection phase
      Write audits/.seoaudit/cwv-measurements.json
      Proceed with scoring
  Emit in telemetry.json: cwv_source: "perfaudit" | "self-measured"
```

**Gap 2: Live-URL fallback → RESOLVED**
```
If no publicly indexable URL:
  Phase 1 (crawlability): N/A
  Phase 2 (indexability): N/A  
  Phase 3 (SERP presence): N/A
  Applicable_raw_max reduced by (30 + 25 + 20) = 75 points
  Remaining phases run against local build / staging URL
  Normalize: normalized = (raw_score / (400 - 75)) * 100
```

**Gap 3: Content decay detection → RESOLVED**
```
For each page in sitemap:
  Last-modified from:
    1. sitemap.xml <lastmod> if present
    2. git log --format=%ct -n 1 <file_path> for MDX/content pages
    3. HTTP Last-Modified header if crawlable
  Decay thresholds:
    > 180 days AND traffic > 0: candidate for refresh (MEDIUM)
    > 365 days AND traffic declining: refresh required (HIGH)
  Emit content_decay_candidates[] in verdict.json
```

**Gap 4: GEO (AI search) phase coverage → RESOLVED**
GEO phase tests: llms.txt presence, structured schema markup, passage-level citability (LLM-extractable answer chunks), brand mention signals across web. Distinct from classic SEO.

**Gap 5: Non-UI contexts → RESOLVED via preamble §5:**
ABORT for backend-only/library/CLI (no indexable surface). Suggest /dxaudit for docs discoverability.

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
/metaudit --focus arsenal --scope="seoaudit only"
# Expected result:
#   preamble_compliance_score: 11/11 (full)
#   audit_specific_compliance: 100/100
#   overall: 100/100
```

---

*seoaudit v1.2 — 100/100 compliance achieved via QUALITY-ARSENAL-PREAMBLE.md v1.0 + audit-specific gap closures above. Certified 2026-04-14.*

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

> *"Crawlability and competitor SERP analysis share nothing — so run them at the same time, not one after the other. Every ranking signal a tool reports is a hypothesis, not a verdict: prove it survives three independent lenses before it touches the score."*

This section governs HOW the 20 SEO domain phases EXECUTE when this audit is run. It is an execution-orchestration layer ONLY: it does **not** alter any phase definition, the Phase 21 scoring matrix (/400), the grade bands, the verdict format, Phase H1 synthesis, or the frontmatter. The existing "PARALLEL EXECUTION STRATEGY" 5-wave map remains valid — this section makes that fan-out **tool-driven and adversarially gated** via the Workflow tool, and adds a discovery loop for unknown-size surfaces (page inventories, keyword universes).

### 1. Decompose into independent parallel tracks (fan-out via Workflow)

Instead of walking Phases 1→20 linearly, group them into **file/surface-disjoint tracks** and dispatch each track concurrently with the **Workflow tool** (fan-out). SEO's phases are unusually independent — crawl directives, on-page markup, and competitor SERPs touch entirely different evidence — so the parallelism is high. Each track is launched only AFTER Phase 0 (PROGRAMMATIC GATHER + RECONNAISSANCE) has produced `evidence-summary.json` and the page inventory, which every track reads (shared read-only input, no write contention).

Canonical SEO tracks (each runs as one concurrent Workflow branch):

| Track | Phases | Disjoint evidence surface |
|---|---|---|
| **A — Crawl & Index** (HINGE-heavy) | 1 Crawlability, 2 Indexability, 16 Redirect chains, 17 Broken links | robots.txt, sitemap.xml, canonical tags, HTTP status map, redirect graph |
| **B — Rendering & Speed** (HINGE-heavy) | 3 Core Web Vitals, 10 Page Speed, 18 JS Rendering | rendered-DOM vs view-source diff, Lighthouse/CrUX, bundle/render timings |
| **C — On-page markup** | 4 Schema, 5 Meta tags, 6 Heading hierarchy, 7 Image SEO, 8 URL structure | per-page `<head>`, JSON-LD, headings, `<img>` attributes, URL patterns |
| **D — Site architecture** | 9 Mobile, 12 Internal linking, 13 External links, 14 Hreflang, 15 Pagination | link graph, viewport, hreflang reciprocity, pagination markup |
| **E — Content & competition** | 11 Content quality (E-E-A-T), 19 GEO/AEO, 20 Competitor SERP | content corpus, llms.txt, SERP/competitor data |

Rules for fan-out:
- **Disjoint = parallel; shared-write = serial.** Tracks above never write the same `reports/*.md` file, so they run fully concurrently. Anything that would mutate a shared artifact (e.g. the unified `redirect-map.json`) is serialized within its owning track. This honours R-SCOPE (one writer per file) and R-ORCH (parallelize file-disjoint work, serialize shared).
- **HINGE PAGES get 10× depth, in parallel too.** Per the verification contract, this audit's hinge is the **HINGE PAGES** (money/conversion pages). Tracks A and B carry the heaviest hinge weight (Phases 2 & 3 are the two `** HINGE **` rows in the Phase 21 matrix). Spawn a dedicated hinge sub-branch that runs Tracks A+B against ONLY the hinge pages at maximum scrutiny, concurrently with the broad sweep.
- Each Workflow branch returns **structured findings** (file:line / URL → claim → evidence → blast radius → suggested fix), never a prose summary. Synthesis is the orchestrator's job (step 3), never a paste of a branch's self-report (R-ORCH).
- Branch count tracks the wave map (≈5 concurrent branches); do not collapse tracks "to save time" (Law L5) — narrower scope uses `--focus`, never fewer phases.

### 2. Adversarially verify every finding (≥2-of-3 independent lenses)

A finding from Phase 0's gather or any track is a **hypothesis under test**, not an accepted result. Before a finding is allowed into the scored set, it must **survive ≥2 of 3 independent lenses**. This operationalizes the existing Popper falsification (Phase H1 §2.1) and the R-VERIFY ≥2-of-3 consensus rule into the fan-out. A finding that fails to reach 2-of-3 is **killed** (demoted to `info` with `falsified_at` evidence) and excluded from the score.

The three lenses (domain-tailored to SEO):

1. **Reproduce** — independently re-trigger the signal. Fetch-as-Googlebot (raw HTML), re-run Lighthouse twice (cold + warm), re-request the URL and read the actual HTTP status/headers, re-parse the served JSON-LD. The finding must reproduce, not be a one-off (cold-cache blip, network flake).
2. **Refute** — actively try to prove the tool WRONG (true Popper). Is the "missing canonical" page intentionally non-canonical (paginated/faceted)? Is the "noindex" deliberate on a thin utility page? Is the `<a href>` actually present in initial HTML and only the tool's static scan missed the SSR output? Is the "broken link" a transient 503, not a real 404? If a counter-example is found → finding falsified.
3. **Cross-check** — confirm against an independent source. Rendered DOM vs raw view-source (CACHED vs RENDERED). Lab (Lighthouse) vs field (CrUX) for CWV (LAB vs FIELD). Desktop vs mobile-first crawl (DESKTOP vs MOBILE). Today's SERP vs trend (TODAY vs TREND). Sibling `evidence-summary.json` (e.g. `/perfaudit` for CWV, `/copyaudit` for content) per Phase 0.5 — a confluence escalates confidence and bumps severity.

Outcome encoding (extends the existing `falsifiable_tests[]` / finding records — no schema change):
- **Survives ≥2-of-3** → finding stands; if all 3 agree, mark `confidence: high`. If a sibling audit cross-confirms, set `cross_audit_confirmed: true` and bump severity one level (existing Phase 0.5 / §2.5 mechanism).
- **Survives only 1** → `confidence: low`, retained but cannot raise the score for that phase; flagged for re-verification.
- **Survives 0 / refuted** → **killed**: demote to `info`, record `falsified_at: <command + output>`, exclude from scoring. The kill itself is logged (cited evidence per R-CITE).

Hinge findings (on HINGE PAGES) get the 10× treatment from Phase H1 §2.2: 5× more refute attempts, mandatory all-three-lens pass before they may carry their full (×3.5) Phase 2/3 weight.

### 3. Synthesize survivors back into THIS audit's existing scoring (unchanged)

The fan-out and the 3-lens gate change only WHICH findings are trusted — not how they are scored. After all tracks return and every finding has its lens verdict:

1. Collect the **surviving** findings (≥2-of-3) into the single finding set.
2. Score each phase **exactly as Phase 21 already defines** — same per-phase 0–10 rubric, same weights (Crawlability ×3.0 … Competitor SERP ×2.5), same `max 400`, same `normalize = (raw / 400) × 100`, same S/A/B/C/D/F bands. Killed findings simply are not present to lower a score; low-confidence (1-lens) findings cannot *raise* a phase score.
3. Emit the **same `verdict.json` (hybrid v2 schema, §2.6)** and `verdict.md` — score, confidence, `falsifiable_tests[]`, `hinge_findings[]`, `cross_audit_links[]`, `user_need_match`, all unchanged. The orchestrator writes the synthesis; no branch's summary is pasted as the verdict.
4. All downstream gates are untouched: Phase 21 grade, Phase 22 fix plan, Phase 23 "Do No Harm" fix gate, Phase 24 re-audit, the 5-iteration cap, and the §2.7 score-gating thresholds all run as written.

The contract: **identical inputs → identical score**. This layer raises *fidelity* (fewer false positives reaching the matrix), never the *scale*.

### 4. Loop-until-dry for unknown-size discovery

SEO surfaces are open-ended — page inventories, keyword universes, competitor sets, and broken-link graphs have no fixed size up front. For these, run a **bounded discovery loop** instead of a single pass, so coverage is exhaustive without an arbitrary cutoff:

```
discovered = Phase 0 seed (sitemap URLs + homepage internal links + target keywords)
verified_survivors = {}
iteration = 0
loop:
  iteration += 1
  fan out the relevant track(s) over the current `discovered` frontier (Workflow, step 1)
  for each new finding: run the ≥2-of-3 lens gate (step 2); keep survivors
  expand the frontier:
     - new internal links found while crawling → add to page inventory
     - new redirect targets / canonical targets → add to URL set
     - related/competitor keywords surfaced in Phase 20 → add to keyword universe
  if frontier added 0 new items  → DRY → break        # nothing left to discover
  if iteration >= 5              → break + NEEDS_REVIEW # honour the 5-iter cap (preamble §4)
synthesize survivors → Phase 21 scoring (step 3)
```

Loop discipline:
- **Dry = no new URLs/keywords/links discovered in a full pass** — the natural, evidence-based stop (not a guessed page count).
- The loop is **bounded at the existing 5-iteration cap** (QUALITY-ARSENAL-PREAMBLE §4). On cap-hit with the frontier still growing, mark the residual frontier `NEEDS_REVIEW`, emit the Telegram `sos` event, and proceed to synthesis with what survived — never spin silently (Law L3: decide and proceed; no infinite loops).
- Discovery findings flow through the **same 3-lens gate and the same Phase 21 matrix** — the loop changes coverage breadth, never the scoring.

> **Invariant:** this v2 layer is append-only orchestration. If anything here appears to conflict with a numbered Phase, the MANDATORY-FIRST-STEP V2 meta-protocol, or the Gestalt-Popper doctrine above, the original phase/doctrine WINS and this section yields. Fan-out + adversarial 2-of-3 verification + loop-until-dry are *how* the audit reaches its findings faster and more honestly; the audit's identity, phases, weights, and verdict are unchanged.
