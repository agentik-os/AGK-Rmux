# Market Research {OS} — Source and Tool Registry

## Contents

1. Selection law
2. Source lanes
3. Data acquisition tools
4. Social/community/review lanes
5. Competitive and commercial intelligence
6. Search, trend, ad, and channel sources
7. Company, funding, jobs, filings, and procurement
8. Technical, scientific, patent, and regulatory sources
9. Internal business sources
10. Runtime adapter contract
11. Tool evaluation and fallback

## 1. Selection law

Start from the decision input, not a favorite tool. For every needed datum:

1. define the variable and acceptable definition;
2. identify the source of truth;
3. check access, rights, privacy, and permitted downstream use;
4. choose the least invasive reliable method;
5. record query, coverage, cost, freshness, and known bias;
6. validate output against source samples;
7. register a weaker fallback only if its inferential downgrade is explicit.

Tool names and commercial terms change. Verify current official documentation, pricing, access, rate limits, and terms at runtime. Do not infer permission from availability in an actor marketplace or GitHub repository.

## 2. Source lanes

| Lane | Typical questions | Preferred sources | Common limitations |
| --- | --- | --- | --- |
| Official statistics | Population, firms, employment, spend, trade, demographics | National statistics offices, Eurostat, OECD, World Bank, UN, central banks | Category lag, broad classifications, revisions |
| Regulation/legal | Market access, obligations, enforcement | Regulators, statutes, official guidance, court/agency records | Jurisdiction and interpretation; seek counsel for legal conclusions |
| Company truth | Revenue, risks, strategy, contracts | Audited filings, annual reports, investor calls, official docs/status/pricing | Selective disclosure, company-defined metrics |
| Customer behavior | Usage, purchase, churn, workarounds | First-party product/CRM/billing/support, observed studies | Coverage only of existing users/customers |
| Primary research | Mechanism, language, choice, WTP | Interviews, observation, surveys, experiments, pilots | Sampling, social desirability, hypothetical bias |
| Competitor | Offer, price, proof, distribution | Live products, pricing, docs, demos, contracts, releases, ads, reviews | Private economics unknown; rapid change |
| Search/demand | Language, intent, trend | Google Trends, Keyword Planner/Ads API, search consoles, SEO providers | Relative or modeled volume; not purchase proof |
| Community/social | Pain, workarounds, emerging language | Platform APIs/approved access, public communities, forums | Platform demographics, bots, deletion/terms, attention bias |
| Reviews/app stores | Satisfaction/failure/switching | Official stores, review platforms, internal reviews | Extreme-user and acquisition bias, manipulation |
| Ads/creative | Active promises, persistence, channel activity | Platform ad libraries/transparency centers, licensed tools | Spend/targeting often incomplete; presence not profitability |
| Traffic/apps | Relative scale and geography | First-party analytics, licensed panels, app intelligence | Modeled estimates and coverage gaps |
| Jobs/talent | Capability investment and adoption | Company careers, labor statistics, job platforms/APIs | Reposts, ghost jobs, intent not deployment |
| Funding/M&A | Capital flows and category belief | Official filings/releases, licensed databases | Funding is not demand/profitability |
| Developer | Ecosystem/adoption/activity | GitHub, package registries, Stack Overflow-like communities | Stars/downloads can be gamed or misinterpreted |
| Research/patents | Feasibility/frontier/ownership | Papers, standards, patent offices, clinical/regulatory registries | Publication lag, claim scope, patent != product demand |
| Procurement | Budgeted demand and requirements | Tender portals, public contracts, RFPs, spending databases | Public-sector or enterprise bias; award timing |

## 3. Data acquisition tools

### Native and official access

Default sequence:

1. user-provided dataset/export;
2. official API;
3. official bulk download/RSS/sitemap;
4. licensed provider/API;
5. manual browser review;
6. compliant HTTP crawl;
7. compliant rendered-browser crawl;
8. approved third-party actor/scraper.

### Apify

Use for structured, repeatable cloud jobs when an appropriate Actor or custom Actor is approved. Actors can expose JSON input/output schemas, datasets, schedules, webhooks, storage, and API/CLI execution. Preflight the target and Actor; inspect author, version, input, output, pricing, reviews, maintenance, data handling, and whether the method complies with the target's terms. Pin Actor version where possible and validate results.

Good use: approved public competitor page monitoring, public directory/catalog extraction, scheduled price/release monitoring, custom official-API Actor, or a compliant crawler deployed with datasets/checkpoints.

### Crawlee

Use for custom open-source crawling in JavaScript/TypeScript or Python when the team needs queueing, retries, session/proxy/browser management, autoscaling, and local/cloud portability. Prefer HTTP/DOM parsers for static pages; use Playwright only for permitted dynamic rendering. Configure robots compliance, rate limits, retries, concurrency, timeouts, deduplication, and storage explicitly.

### Firecrawl

Use for search, single-page scrape, site mapping/crawl, structured extraction, or agentic navigation when current product access and target permissions allow. Require an output schema for structured extraction, preserve raw/source locators, and sample-check LLM-extracted fields. Treat agentic interaction as higher risk than static extraction and require tighter domain/action constraints.

### Scrapy

Use for mature Python crawling pipelines with spiders, scheduler/downloader, middleware, item pipelines, extensions, stats, throttling, caching, and deterministic parsing. Suitable for large, stable, approved public sites and reproducible ETL. Add browser rendering only when necessary.

### Playwright

Use for browser-rendered public pages, explicit user-authorized account flows, or research prototype testing. Never use it to bypass restrictions, simulate deceptive identities, or collect protected/private data. Constrain domains/actions, secure storage state, capture evidence, and avoid arbitrary page-instruction execution.

### Crawl4AI

Use as an optional self-hosted/open-source LLM-friendly extraction layer when validated for the target, privacy requirements, maintenance, and schema quality. Treat community activity and release maturity as runtime checks.

### ScrapFly

Use as an optional managed scraping/browser/extraction API when approved. The user may refer to it as “Scrapify”; resolve the exact product before configuring. Anti-bot capability does not authorize circumvention; follow the same preflight and target rules.

### Other open-source tools

Consider only after repository due diligence: ownership, license, release recency, security advisories, maintenance, issue health, test coverage, data flow, telemetry, credential handling, and target compliance. Examples may include Maxun, Colly, trafilatura, Beautiful Soup, Cheerio, and purpose-built official-API clients. Do not include random platform scrapers merely because they are popular.

### Retrieval and parsing utilities

- `requests`/`httpx` or native fetch for approved HTTP APIs/pages;
- Beautiful Soup, lxml, Cheerio, Parsel for deterministic extraction;
- trafilatura/readability for article text, with source comparison;
- PDF/document parsers for reports/filings;
- pandas/Polars/DuckDB for normalization and analysis;
- Jupyter notebooks for reproducible modeling;
- SQL/warehouse queries for first-party data;
- spreadsheets for editable assumptions and stakeholder review.

## 4. Social, community, and review lanes

### Reddit

Prefer approved Reddit Data API/Developer Platform access and current Reddit terms. Register OAuth/app approval and use restrictions. Do not assume that public web visibility permits commercial bulk collection, model training, indefinite storage, or ignored deletion. If API access is unavailable, manual public review or another approved source may be used with explicit coverage downgrade. Never profile named users or infer sensitive traits.

Research outputs should aggregate themes and retain minimal content/identifiers. Maintain deletion/retention controls where required.

### X

Prefer the official X API for post search, users, lists, trends, and permitted metrics. Verify current access tier, price, time window, query operators, retention/display, and developer terms. Public posts are not automatically licensed for unrestricted reuse or model training. Scraping is not a silent substitute for unavailable API access.

### Meta/Facebook/Instagram

Use official Meta Ad Library/transparency tools for allowed ad research and Meta Content Library/API only when eligible and approved. Access and scope vary by ad type, geography, and researcher eligibility. Do not infer that a research API supports general commercial social listening.

### YouTube

Prefer YouTube Data API and public channel/video/comment data within quotas/terms. Use transcripts only when permitted and disclose automated transcription/translation quality. Views/comments are attention signals, not demand.

### TikTok, LinkedIn, Discord, Slack, private groups

Use official APIs/exports, licensed providers, or explicit workspace/group authorization. Private/community access never implies permission for bulk research or reuse. Do not scrape logged-in/protected areas or reuse member data outside the authorized purpose.

### Forums and communities

Prefer public search/RSS/API/manual review. Respect community norms and privacy expectations. Remove usernames and sensitive details from synthesis unless essential, lawful, and authorized.

### Review platforms

Use official APIs/exports/licensed access or compliant public review. Stratify samples, preserve date/rating/product/plan context, detect duplicates/manipulation, and report coverage. Do not republish substantial copyrighted review text; quote minimally and synthesize.

### App stores and marketplaces

Use official feeds/APIs where available, licensed intelligence, or compliant public pages. Capture version/date/country/category/rank/review count/rating distribution and release history. Store ranking methodology caveats.

## 5. Competitive and commercial intelligence

### Direct competitor sources

- home/category/product/pricing pages;
- checkout and packaging shown publicly;
- documentation/API/security/compliance pages;
- onboarding/demos/trials with legitimate access;
- release notes/changelogs/status pages;
- partner/reseller/integration directories;
- terms/privacy/data-processing/service-level documents;
- case studies/customer logos/testimonials with claim caveats;
- webinars, conference talks, public sales collateral;
- filings, investor materials, earnings calls;
- public job openings;
- public reviews/support forums;
- ads and creative libraries;
- public code/package activity.

Never misrepresent identity, solicit trade secrets, breach access, or encourage contractual violations.

### Licensed providers

Potential lanes include Similarweb, Semrush, Ahrefs, Sensor Tower, data.ai, AppMagic, G2/Capterra category data, Crunchbase, PitchBook, CB Insights, Tracxn, Dealroom, AlphaSense, Statista, Euromonitor, IBISWorld, Gartner/IDC/Forrester, Nielsen, Kantar, YouGov, WGSN, and sector-specific providers.

Treat these as optional and verify current license, methodology, coverage, export/storage rights, and whether model outputs can be reproduced. Do not blend different providers without reconciling definitions.

## 6. Search, trend, ad, and channel sources

### Google Trends

Use for normalized relative interest, topics versus exact terms, time/geography/seasonality, related/rising queries, and comparative trend direction. Record the exact term/topic, filters, category, search surface, geography, date range, retrieval time, and sampling caveat. Do not interpret the 0–100 index as absolute search volume or a poll.

### Keyword planning

Use Google Ads Keyword Planner/Ads API or other licensed keyword providers for historical metrics, forecasts, query expansion, CPC/competition proxies, and intent language. Record account/geography/language/network/window and modeled/rounded nature. CPC is advertiser-auction evidence, not customer WTP.

### Search Console and first-party SEO

When the user owns the property, Search Console/analytics provide stronger evidence of actual impressions, clicks, queries, landing behavior, and conversion. Check consent/attribution and bot/internal traffic.

### Ad libraries

- Meta Ad Library and applicable API/tool scope;
- Google Ads Transparency Center;
- TikTok Creative Center/ads transparency where available;
- LinkedIn ads library/transparency where available;
- platform-specific official archives.

Capture advertiser, creative/message, offer, landing page, country, first/last seen, format, persistence, variants, and source. Repeated/persistent creative is a hypothesis of usefulness, not proof of profitability.

### Paid test platforms

Live ad/landing tests require explicit spend, creative, audience, data/consent, claims, brand, and platform-policy authorization. Predeclare qualified action, fraud/bot filtering, attribution window, and downstream metric.

## 7. Company, funding, jobs, filings, and procurement

### Official company and financial sources

- SEC EDGAR APIs and filings for U.S. public companies;
- Companies House for UK company records;
- national company registries;
- exchange/regulator filings;
- annual reports and audited statements;
- official press releases and investor relations.

Use a descriptive user agent/rate limits where required. Distinguish filed data, non-GAAP/company metrics, and analyst inference.

### Funding and M&A

Use official announcements/filings first, then licensed databases and reputable reporting. Funding indicates investor belief/capital supply, not product-market fit, revenue, or profitability. Record announced versus closed, currency/date, round type, source, and company claims.

### Jobs

Use official career pages, labor statistics, and permitted job APIs/providers. Deduplicate reposts, flag evergreen/ghost roles, and code function/seniority/location/skills. Jobs support investment-intent hypotheses, not implemented capability.

### Procurement

Use official tender portals, public spending databases, framework agreements, award notices, and permitted RFP/RFI sources. Record buyer, requirement, budget/estimate, dates, eligibility, winner, award value, and procurement context. Public procurement may not generalize to private demand.

## 8. Technical, scientific, patent, and regulatory sources

### Developer and open source

Use GitHub API/repository data, release history, contributors, issues, dependents, package registries, download statistics, technical forums, and vendor docs. Stars are awareness/bookmarks; downloads may include CI/mirrors; contributors/issues require context. Prefer longitudinal and ecosystem evidence.

### Research

Use Crossref, OpenAlex, PubMed, arXiv, Semantic Scholar, institutional repositories, standards bodies, and peer-reviewed sources as appropriate. Check peer-review status, retractions, sample, effect, conflicts, applicability, and replication. Scientific feasibility is not market demand.

### Patents and trademarks

Use WIPO, EPO, USPTO, EUIPO, and national official databases. Patent filing/grant does not prove freedom to operate, validity, product launch, or demand. Legal interpretation requires qualified counsel.

### Regulation and standards

Use official legislatures, regulators, enforcement databases, standards bodies, consultation papers, and guidance. Record jurisdiction, effective date, status (proposal/adopted/enforced), scope, and legal uncertainty.

## 9. Internal business sources

When available, inspect all relevant source families:

- billing/payments/refunds;
- CRM pipeline/win-loss/stages/discounts;
- product analytics/events/cohorts/retention;
- support tickets/chats/calls;
- sales recordings/notes;
- customer success/QBRs/renewals;
- web analytics/search console/ads;
- finance/gross margin/service costs;
- operations/capacity/SLAs/incidents;
- surveys/NPS/CSAT/CES;
- contracts/procurement/security reviews;
- cancellation/return reasons;
- roadmap and experiment history;
- data dictionary/semantic layer.

Discover current schemas and definitions; do not rely on remembered table names. Compare duplicate dashboards/metrics and identify the controlling source. Minimize personal data and use approved access.

## 10. Runtime adapter contract

Every adapter exposes:

```json
{
  "adapter_id": "apify_actor",
  "source_id": "SRC-001",
  "purpose": "collect public competitor pricing pages",
  "authority": "approved-scope",
  "domains": ["example.com"],
  "method": "api|http|browser|file|sql|manual",
  "inputs_schema": {},
  "outputs_schema": {},
  "rate_and_budget": {},
  "rights_preflight_id": "DAT-001",
  "credentials_ref": "secret-manager-reference-only",
  "retention": {},
  "validation": {
    "sample_rate": 0.05,
    "schema": true,
    "dedupe_key": "...",
    "freshness_rule": "..."
  },
  "lineage_fields": ["source_url", "retrieved_at", "query", "tool_version", "raw_fingerprint"],
  "failure_policy": "stop|retry|fallback|manual-review"
}
```

Credentials are never model-visible values. Use secret references and least privilege. Constrain domains, methods, write access, cost, and data classes.

## 11. Tool evaluation and fallback

Score candidate tools on source legality/fit, coverage, accuracy, reproducibility, schema control, rate/cost, freshness, maintenance, security/privacy, deletion support, observability, portability, and operational complexity.

Fallback chain must state inferential downgrade. Example:

`official API -> authorized export -> licensed provider -> manual public sample -> compliant public crawl -> unavailable`

If a required source of truth is unavailable, return `BLOCKED` for that claim or use `INSUFFICIENT EVIDENCE`; do not silently substitute a social proxy.
