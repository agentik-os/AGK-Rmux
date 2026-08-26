# Market Research {OS} — Start Here

Market Research {OS} is the market-evidence and validation compiler placed before Blueprint {OS}:

`Idea -> Market Research {OS} -> Decision -> Blueprint {OS} -> Stepper {OS} -> Build {OS}`

It does not merely write a market-study narrative. It maintains sources, rights preflights, hypotheses, methods, samples, observations, models, experiments, risks, traceability, gates, a recommendation, and a frozen Blueprint input manifest.

## Pack contents

- `00_MARKET_RESEARCH_OS_COMPLETE_MANUAL.md` — one-file compiled manual containing the full system prompt, contracts, frameworks, functions, schemas, templates, scripts, and Omega integration.
- `market-research-os/` — canonical modular installable skill with progressive references.
- `VALIDATION_REPORT.json` — deterministic QA summary for this release.

Critical rules are always represented in text or tables; Mermaid diagrams are supplemental.

## Primary commands

```text
/market-research <idea>
/market-research scan <idea>
/market-research validate <idea>
/market-research diligence <opportunity>
/market-research deep <market|segment|competitor|price|feature|channel>
/market-research audit
/market-research delta <version-a> <version-b>
/market-research continue
/market-research status
/market-research score
/market-research handoff
```

## Depths

- `SIGNAL` — directional desk research and exact validation plan. It cannot claim full market validation.
- `VALIDATION` — triangulated research, primary customer evidence, and relevant behavioral validation.
- `INVESTMENT_GRADE` — reproducible models, disclosed samples, stronger commercial evidence, independent critic, and governance review.

## Decisions

The only recommendation values are:

- `GO`
- `PIVOT`
- `HOLD`
- `NO-GO`
- `INSUFFICIENT EVIDENCE`

`GO` and `PIVOT` are always bounded by segment, problem/JTBD, promise, geography, business model, channel, stage, conditions, kill criteria, and expiry.

## Omega OS installation

Run a dry run first:

```bash
python3 market-research-os/scripts/install_omega_os.py /absolute/path/to/omega-os
```

After reviewing every destination:

```bash
python3 market-research-os/scripts/install_omega_os.py /absolute/path/to/omega-os --apply
```

Existing differing files are preserved. Use `--force` only after reviewing the exact conflict.

## Deterministic workspace

```bash
python3 market-research-os/scripts/market_research_os.py init ./research-state \
  --project-id my-project \
  --project-name "My Project" \
  --decision "Should this opportunity proceed to Blueprint?" \
  --mode FULL_VALIDATION \
  --depth VALIDATION

python3 market-research-os/scripts/market_research_os.py validate ./research-state
python3 market-research-os/scripts/market_research_os.py status ./research-state
python3 market-research-os/scripts/market_research_os.py score ./research-state
```

The deterministic engine validates structure and machine-checkable rules. It never replaces source verification, method review, domain judgment, or the independent critic.

## Scraping boundary

Apify, Crawlee, Firecrawl, Scrapy, Playwright, Crawl4AI, ScrapFly, official APIs, and licensed data providers are supported as possible adapters. A mandatory source preflight controls whether collection is allowed. Technical access never grants permission, and the OS forbids bypassing authentication, paywalls, CAPTCHAs, bans, rate limits, or platform enforcement.
