---
name: agent-ecosystem-watch
description: "Agent-Ecosystem Watch — a daily autonomous intelligence pass over the Claude and AI-agent ecosystem on X. Reads a topic watchlist via last30days/ScrapeCreators, extracts CONCRETE improvements that make Claude / Claude Code / OmegaOS better (new features, prompting techniques, MCP servers, subagent/orchestration tricks, hooks, eval methods, model updates), scores each for integratability into OmegaOS, writes an HTML report + Telegram alert for the operator to decide, and (only when ARMED, behind an adversarial quality gate + daily cap + dedup) auto-publishes vetted best-practice tweets to @Agentik_os via omega-zernio. Use when the user says '/omg-ecosystem-watch', '/ecosystem-watch', 'veille Claude', 'veille agents IA', 'watch the Claude ecosystem', 'run the ecosystem watch', or wants to inspect/arm/disarm the daily watch. NOT a live web research primitive for one question (that is last30days / deep-research) and NOT a general social scheduler (that is the marketing suite): this is the standing daily radar that feeds OmegaOS improvements and the @Agentik_os best-practice feed. Integration into OmegaOS is NEVER automatic — the report proposes, the operator decides."
---

# Agent-Ecosystem Watch

A standing daily radar on the thing that compounds OmegaOS's edge: how fast you absorb new
Claude and agent techniques. It watches X for concrete improvements, tells you what is worth
wiring into OmegaOS, and shares the best of it on @Agentik_os on autopilot, behind a gate so
your public account never ships a weak or wrong tweet.

Same input feeds three outputs: (1) a decision report for you, (2) a curated best-practice
tweet feed, (3) a growing memory of what the ecosystem shipped. Marginal cost is near zero
once built; the read is a few dollars a month of ScrapeCreators.

---

## Pipeline

```
cron (daily) → [1] READ  watchlist topics → last30days --search x (ScrapeCreators)
             → [2] ANALYZE (claude, sonnet)  extract + score integratable_to_omega 0-10
             → [3] GATE (claude, opus)  adversarial: try to REJECT each candidate tweet
             → [4] PUBLISH (if ARMED, not mock, under cap, unseen)  omega-zernio → @Agentik_os
             → [5] REPORT  self-contained HTML in ~/.omega/artifacts/
             → [6] ALERT   Telegram (Alerts topic) via omega-alert-send.sh
```

Deterministic control (daily cap, dedup persistence, publish side-effects) lives in shell;
judgment (extract, score, draft, refute) lives in the model. The gate is an INDEPENDENT
adversarial pass (R-VERIFY): its only job is to reject, default-reject on doubt.

## Publishing safety rails (non-negotiable)

Publishing to the public @Agentik_os account requires ALL of:

- `armed` flag present (`~/.omega/state/ecosystem-watch/armed`) — the kill-switch. Absent =
  tweets are drafts in the report only. `rm` the flag to stop all publishing instantly.
- a real ScrapeCreators key (mock runs never publish).
- the adversarial gate returned `keep=true` for that tweet.
- the tweet fingerprint is not already in the seen-store (never tweet the same idea twice).
- the daily cap (`EW_DAILY_TWEET_CAP`, default 5) is not exhausted.
- no em/en dash, <= 280 chars, carries its source URL (enforced in shell too, R-NODASH).

Integration of a finding into OmegaOS itself is always a human call: the report proposes
with a 0-10 integratability score, the operator decides.

## Run

```
bash scripts/watch-run.sh                    # full run (reads config/topics.txt)
EW_MOCK=1 bash scripts/watch-run.sh          # fixtures, never publishes (smoke test)
EW_ZERNIO_DRYRUN=1 bash scripts/watch-run.sh # validate zernio post without publishing
```

Env knobs: `EW_DAILY_TWEET_CAP` (5), `EW_DAYS` (2), `EW_ANALYZE_MODEL` (claude-sonnet-5),
`EW_GATE_MODEL` (claude-opus-5), `EW_ZERNIO_PROJECT` (agentik-os), `EW_TOPICS_FILE`.

## Arm / disarm

```
touch ~/.omega/state/ecosystem-watch/armed    # go fully auto (auto-publish)
rm    ~/.omega/state/ecosystem-watch/armed     # kill-switch: drafts only
```

## Config

- `config/topics.txt`   — one X query per line (broad AI-agent ecosystem, Claude-first).
- `config/accounts.txt` — reference builders, an authority hint for scoring.

## State (outside repo, R-ENV)

- `~/.omega/state/ecosystem-watch/seen.jsonl`  — published fingerprints (dedup memory).
- `~/.omega/state/ecosystem-watch/runs/<date>/` — per-run raw pulls, analysis, report.
- `~/.omega/artifacts/ecosystem-watch-<date>.html` — the daily report.
- `~/.omega/logs/ecosystem-watch.log` — run log.

## Cron

```
# 08:00 daily — disarmed by default until the operator arms it
0 8 * * * OMEGA_DIR=$HOME/.omega bash $HOME/.omega/skills/agent-ecosystem-watch/scripts/watch-run.sh
```

## Requires

- `~/.omega/secrets/integrations.env` with `SCRAPECREATORS_API_KEY` (X read) and
  `ZERNIO_API_KEY` (publish). Both already present in this deployment.
- `omega-zernio` with an `agentik-os` profile whose `twitter` account is connected.
- `last30days` skill checkout at `~/.omega/repos/last30days-skill/`.
- `claude` CLI on PATH.
