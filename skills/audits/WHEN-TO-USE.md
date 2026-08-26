# Quality Arsenal — When To Use Each Audit

23 forensic audits + 2 orchestrators. All share the Gestalt-Popper doctrine: clarity gate →
falsification → hinge-point 10x scrutiny → auto-fix → auto-re-audit. Every score normalizes to /100.

> **Routing rule:** match the user's intent (or the changed files) to a domain below and run that
> audit. Multiple domains matched → run each in parallel (one worker per audit, file-disjoint).
> "full audit" / "audit complet" → run all 23 via the orchestrator. Never paraphrase an audit into
> prose — invoke the real skill (`/<name>` on line 1 of the worker prompt).

## Preventive — architecture & design (run before/while building)

| Audit | Trigger keywords | Answers |
|-------|------------------|---------|
| `codeaudit` | code, code audit, code quality, code review | Is the code SOLID? |
| `flowaudit` | flow, user flow, parcours, journey, navigation | Does the experience WORK? |
| `uiuxaudit` | ux, ui, design audit, audit visuel | Is the interface BEAUTIFUL + consistent? |
| `refontaudit` | refonte, redesign dashboard, comme Linear/Vercel, dashboard pro | How to REDESIGN it to senior level? |
| `featureaudit` | feature audit, completeness, what's missing, PRD gap | Is the product COMPLETE? |
| `a11yaudit` | a11y, accessibility, wcag, keyboard, screen reader | Is it ACCESSIBLE (WCAG 2.1 AA)? |
| `seoaudit` | seo, crawlability, ranking, schema markup, GEO/AEO | Is it DISCOVERABLE? |
| `copyaudit` | copy, messaging, claims, CTA, tone | Is the copy CLEAR + honest? |
| `dxaudit` | dx, developer experience, onboarding, README, setup | Is the DX SMOOTH? (primary for CLI/lib) |
| `motionaudit` | motion, animation, easing, scroll, WebGL | Is the motion PURPOSEFUL? (aborts on non-UI) |
| `automationaudit` | automation, cron, scripts, daemon, scheduled tasks | Is automation RELIABLE? |
| `logicaudit` | logic, optimize, system architecture, make it smarter | Is the logic OPTIMAL? |
| `i18naudit` | i18n, localization, translation, RTL, world-ready | Is it WORLD-READY? |
| `retentionaudit` | retention, feature ideas, make it sticky, CPO mindset | What FEATURES are missing? (READ-ONLY) |
| `depaudit` | dependency, supply chain, lockfile, SBOM, license | Is the supply chain SAFE? |
| `releaseaudit` | release, deploy, CI/CD, rollback, migration safety | Is shipping SAFE + reversible? |

## Detective — runtime & security (run on existing/deployed systems)

| Audit | Trigger keywords | Answers |
|-------|------------------|---------|
| `debugaudit` | find bugs, what's broken, runtime errors, console, chaos | What is BROKEN right now? |
| `perfaudit` | perf, slow, core web vitals, bundle, render, N+1 | Is it FAST enough? |
| `secaudit` | security, owasp, xss, sqli, auth bypass, vulnerab | Is it SECURE? |
| `dataaudit` | data integrity, schema, orphans, migration | Is the data INTACT? |
| `privacyaudit` | privacy, GDPR, PII, consent, retention | Is user data HANDLED LAWFULLY? |
| `apiaudit` | api audit, endpoint, contract, rate limit | Is the API SOLID? |
| `observabilityaudit` | observability, logs, traces, metrics, alerts, SLO | Can we SEE what it does in prod? |

## Orchestrators (meta)

| Tool | Use when |
|------|----------|
| `audit-orchestrator` | Intelligent selection + power levels (quick/standard/forensic) OR "full audit" → routes to the right audits |
| `audit-tracker` | Dashboard of audit freshness + scores across a project (init / stale / latest) |

## Decision flow

1. **One concern** (e.g. "is it secure?") → run the single matching audit.
2. **A code change shipped** → `codeaudit` is the baseline floor; add domain audits the diff touches
   (auth→`secaudit`, ui→`uiuxaudit`, api→`apiaudit`, data→`dataaudit`).
3. **Pre-launch / "audit complet"** → `audit-orchestrator` runs all 23 in parallel, then synthesizes.
4. **Non-UI project** (CLI, library, backend) → `dxaudit` is primary; `uiuxaudit`/`flowaudit`/
   `motionaudit` self-abort.
5. **Read-only ideation** → `retentionaudit` proposes features (RICE-scored), never edits code.

> Tokens are unlimited; never run a "quick/streamlined" variant to save time (NO-TIME-PANIC).
> A 403/401 during an audit is an ABORT, never a PASS.
