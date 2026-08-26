# Market Research {OS} — Verified Reference Notes

## Scope

These links anchor tool/source behavior and research governance checked during OS design on 2026-08-10. Re-verify current versions, terms, access, and pricing before each engagement. These are references, not blanket authorization.

## Research ethics and survey design

- ICC/ESOMAR International Code 2025: <https://standards.esomar.org/assets/documents/icc-esomar-code-2025.pdf>
- ESOMAR Code hub: <https://esomar.org/icc-esomar-code-of-conduct>
- Pew Research Center, Writing Survey Questions: <https://www.pewresearch.org/writing-survey-questions/>
- U.S. SBA, market research and competitive analysis entry point: <https://www.sba.gov/counseling/plan-your-business/>
- UK ICO, global privacy authorities' statement on data scraping: <https://ico.org.uk/about-the-ico/media-centre/news-and-blogs/2024/10/global-privacy-authorities-issue-follow-up-joint-statement-on-data-scraping-after-industry-engagement/>

The OS derives these controls: systematic/transparent research, participant/data responsibility, instrument pretesting, wording/order bias control, and lawful/privacy-aware scraping.

## Crawling and extraction platforms

- Apify Actors: <https://docs.apify.com/actors>
- Apify Actor/API concepts: <https://docs.apify.com/>
- Crawlee overview: <https://crawlee.dev/>
- Crawlee JavaScript quick start: <https://crawlee.dev/js/docs/quick-start>
- Crawlee Python quick start: <https://crawlee.dev/python/docs/quick-start>
- Crawlee repository: <https://github.com/apify/crawlee>
- Scrapy documentation: <https://docs.scrapy.org/>
- Scrapy architecture: <https://docs.scrapy.org/en/latest/topics/architecture.html>
- Firecrawl API introduction: <https://docs.firecrawl.dev/api-reference/v2-introduction>
- Firecrawl scrape endpoint: <https://docs.firecrawl.dev/api-reference/endpoint/scrape>
- Firecrawl repository: <https://github.com/firecrawl/firecrawl>
- Crawl4AI repository: <https://github.com/unclecode/crawl4AI>
- ScrapFly organization/SDKs: <https://github.com/scrapfly>

Observed design facts: Apify Actors use structured inputs/outputs and cloud runs/storage/schedules; Crawlee offers HTTP/browser crawler classes and queue/runtime abstractions; Scrapy provides a mature spider/scheduler/downloader/pipeline architecture; Firecrawl exposes search/scrape/crawl/map/extract/agent surfaces; open-source alternatives require independent due diligence. None overrides target permissions.

## Social and platform sources

- X API overview: <https://docs.x.com/x-api/introduction>
- X post search: <https://docs.x.com/x-api/posts/search/introduction>
- Reddit Data API Terms: <https://redditinc.com/policies/data-api-terms>
- Reddit Data API support/wiki: <https://support.reddithelp.com/hc/en-us/articles/16160319875092-Reddit-Data-API-Wiki>
- Reddit Developer guidelines: <https://developers.reddit.com/docs/guidelines>
- Meta Ad Library tools: <https://transparency.meta.com/researchtools/ad-library-tools/>
- Meta Content Library/API: <https://transparency.meta.com/researchtools/meta-content-library/>

Runtime implication: use current official access and terms, record retention/deletion/model-use restrictions, and never treat scraping as an automatic workaround for restricted API access.

## Search and ad demand sources

- Google Trends data FAQ/methodology: <https://support.google.com/trends/answer/4365533>
- Compare search terms/topics: <https://support.google.com/trends/answer/17309543>
- Trends public BigQuery dataset: <https://support.google.com/trends/answer/12764470>
- Export/cite Trends: <https://support.google.com/trends/answer/4365538>
- Google Ads Keyword Planner help: <https://support.google.com/google-ads/answer/7337243>
- Google Ads API keyword planning: <https://developers.google.com/google-ads/api/docs/keyword-planning/overview>
- Google Ads Transparency Center: <https://adstransparency.google.com/>

Runtime implication: Trends is normalized/relative/sample-based and not a poll or absolute volume; exact terms/topics and filters matter. Keyword metrics/forecasts and ad transparency are separate evidence classes.

## Official statistical and company data

- World Bank Indicators API: <https://datahelpdesk.worldbank.org/knowledgebase/articles/889392-about-the-indicators-api-documentation>
- Eurostat Statistics API: <https://ec.europa.eu/eurostat/web/user-guides/data-browser/api-data-access/api-getting-started>
- OECD Data Explorer API: <https://www.oecd.org/en/data/insights/data-explainers/2024/09/api.html>
- SEC EDGAR APIs: <https://www.sec.gov/search-filings/edgar-application-programming-interfaces>
- SEC data API root: <https://data.sec.gov/>

Runtime implication: prefer official definitions and programmatic access for macro/firm/filing inputs, while checking update cycles, classifications, revisions, and fair-access rules.
