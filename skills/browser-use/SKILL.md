---
name: browser-use
description: >
  Run an agentic browser task from natural language on the Browser Use cloud — the LLM-driven agent navigates an unknown site, extracts data, or fills a form on its own (NOT a local scripted browser). Use when the user says "/omg-browser-use", "/browser-use", "agentic browser task", "browse this site", "extract from a website", "fill out this form", "agentic web automation", or "navigate an unknown site". Boundary R-BROWSER: agentic/unknown-UI/open-ended only; deterministic scripted steps for OUR apps stay in Playwright/acceptance. Requires the paid Browser Use cloud plan + BROWSER_USE_API_KEY.
triggers: ["omg-browser-use","browser-use","agentic browser task","browse this site","extract from a website","fill out this form","agentic web automation","navigate an unknown site"]
allowed-tools: ["Bash","Read"]
domain: automation
read_only: false
argument-hint: "<natural-language browser task>"
source: browser-use/browser-use-sdk (PyPI, official Browser Use cloud SDK)
license: MIT (upstream)
---

> **OmegaOS skill** — wraps browser-use/browser-use-sdk (PyPI, the official Browser Use cloud SDK). Triggers `/omg-browser-use` (and `/browser-use`). Governed by R-BROWSER. **Requires:** the paid Browser Use cloud plan + `BROWSER_USE_API_KEY` in `~/.omega/secrets/integrations.env`.

# Browser Use — agentic cloud browser automation

## What it is

Agentic **cloud** browser automation. You hand the skill a single natural-language task — "find the pricing tiers on this SaaS site and return them", "fill out this contact form with these details", "navigate this dashboard I've never seen and extract the open invoices" — and an LLM-driven agent **runs it on Browser Use's hosted cloud browser**, deciding each click/scroll/type itself, then returns the final answer.

It is **NOT a local browser** on this machine: nothing renders here, no local Chromium is launched. The task is dispatched to `api.browser-use.com`, executed remotely on the operator's paid plan, and only the agent's final text/structured output comes back. This is the OmegaOS arm for *open-ended, unknown-UI* web work — the cases where you cannot pre-script the steps because you don't know the page in advance.

## When to use it vs Playwright (the R-BROWSER boundary)

Two browser surfaces, two jobs — pick by **who decides the steps**:

| | **Playwright / `acceptance`** | **browser-use** (this skill) |
|---|---|---|
| Decides the steps | **You** — deterministic, scripted, known selectors | **The LLM agent** — figures out the UI live |
| Target | **OUR** apps (routes we built, golden path we know) | Sites we **don't control / don't know** |
| Shape | Scripted / repeatable / assertion-based e2e | Open-ended / one-off / "just go do this" |
| Runtime | Local Chromium via Playwright | Browser Use **cloud** browser |
| Use it for | Acceptance gate, route sweep, golden-path proof, regression | Agentic task, scrape-an-unknown-site, fill-an-arbitrary-form, navigate-an-unfamiliar-UI |

Rule of thumb: if you can write the selectors and assertions up front, it's **Playwright/acceptance**. If the only spec you have is a sentence and an unfamiliar page, it's **browser-use**. Never reach here to test an OmegaOS-built app's known flow — that's `acceptance`'s job and it's runtime-verifiable locally; this skill's egress is to a paid third party.

## Setup

The key is **never** in the repo. It lives in `~/.omega/secrets/integrations.env` (mode 600, outside any git tree) as one line:

```
BROWSER_USE_API_KEY=bu_...
```

Get a key at `cloud.browser-use.com/settings`, then append that line to `~/.omega/secrets/integrations.env`. The wrapper sources the file at runtime if `BROWSER_USE_API_KEY` is not already in the environment — you do not export it by hand.

The Python venv lives at `~/.omega/skills/browser-use/.venv`. It is **auto-created on the first run** of the wrapper; `pip install browser-use-sdk` (base package only) happens **lazily** at that point, never at install time.

## Usage

Call the wrapper with the task as a single quoted argument:

```bash
./browser-use "<natural-language task>"
```

Real examples:

```bash
# Extract from an unknown site
./browser-use "Go to https://news.ycombinator.com and return the titles and points of the top 5 stories."

# Navigate an unfamiliar UI and pull structured data
./browser-use "Open https://example-saas.com/pricing, list every plan with its monthly price and the headline feature of each."

# Drive an arbitrary form
./browser-use "Go to https://httpbin.org/forms/post, fill the customer name with 'Ada Lovelace', pick the medium pizza size, and submit — report the confirmation the page shows."
```

The wrapper prints the agent's final output to stdout. Reply to the user in their language; keep it to the answer, not a play-by-play of the run.

## How it works

The `browser-use` bash wrapper, in order:

1. **Resolves the key** — uses `$BROWSER_USE_API_KEY` if already set, else lifts **only** that one variable out of `~/.omega/secrets/integrations.env` (the file is sourced in a subshell, so the other secrets in it never enter the SDK process — least privilege). If still unset it prints how to get/place a key and exits 1. The key value is never echoed or logged.
2. **Ensures the venv** — if `~/.omega/skills/browser-use/.venv` is missing, it creates it (`python3 -m venv`) and runs `pip install --quiet --upgrade pip browser-use-sdk` (base package only — never the `x402` EVM extra). A one-line "bootstrapping venv" notice goes to stderr.
3. **Runs `run.py`** against the cloud API — `run.py` (next to the wrapper) instantiates the official SDK client (which reads `BROWSER_USE_API_KEY` from the environment automatically), runs the single task on `api.browser-use.com`, waits for completion, and prints `result.output`.

The task is always passed as one quoted argv element straight to Python — never `eval`'d, never interpolated into a shell string — so arbitrary task text (quotes, `$`, `;`) is injection-safe.

## Security & boundary

This is a **runtime opt-in**, identical in shape to the higgsfield and gooseworks dependencies (rules R-VISUAL-ID / R-SEC):

- **`install.sh` ships only this markdown + the wrapper + `run.py`.** It **never** pip-installs `browser-use-sdk`, never creates the venv, never provisions a key. The pip install and the venv are created lazily on first real run.
- **The key comes only from the environment / `~/.omega`.** Never the repo, never a tracked file, never a log line, never stdout. `~/.omega/secrets/integrations.env` is gitignored and lives outside the repo.
- **Data egress is to the paid Browser Use cloud** (`api.browser-use.com`). The task text and whatever the agent reads on the target site are sent to a third party under the operator's plan. Treat task contents accordingly; don't pass secrets in the task string.
- **A live agentic run is not runtime-verifiable inside OmegaOS without the operator's key + plan.** The verifiable contract OmegaOS ships is *"the skill resolves via its `/omg-browser-use` alias and the wrapper exists"* — **not** *"the agentic run succeeded"*. Never claim a generated browser result as runtime-verified absent a key.

## Verify

Opt-in live smoke check — gated on the key being set, exactly like the higgsfield/gooseworks skills:

```bash
./browser-use --smoke
```

`--smoke` runs a trivial deterministic task ("Go to https://example.com and return the exact text of the main H1 heading.") on the cloud and prints the result. With no key set it exits 1 with the setup hint and performs no run. Expected green output names the `Example Domain` heading. Without a key — or without a paid plan — there is **no** offline pass: a 401/403 from the cloud is an abort, never a pass (L5).

The install-parity contract (what a fresh `git clone && ./install.sh` must reproduce): this `SKILL.md`, the `browser-use` wrapper, and `run.py` present and the skill resolving via `/omg-browser-use`. The venv, the pip install, and a successful cloud run are explicitly **out of scope** for install-time verification — they are the runtime opt-in.
