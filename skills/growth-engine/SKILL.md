---
name: growth-engine
description: "Growth Engine — a bounded, gated X growth system for @Agentik_os aimed at real follower growth (north star 10k) by being genuinely useful in the right conversations, never by spam automation. A daily Reply Radar finds high-leverage AI-agents/Claude threads via last30days/ScrapeCreators, drafts genuine on-voice replies, passes them through an adversarial anti-spam gate, then (only when ARMED with a valid X session) posts a LOW-CAPPED number of replies + likes via Playwright driving the operator's logged-in session, with human pacing, dedup, and a kill-switch. Use when the user says '/omg-growth-engine', '/growth-engine', 'growth X', 'gagner des followers', 'reply radar', 'engager sur X', 'arm/disarm the growth engine'. NOT a mass like/follow/DM bot (that gets the account suspended and is refused), NOT the content publisher (that is agent-ecosystem-watch): this is the engagement layer, deliberately small-volume and quality-gated."
---

# Growth Engine

Durable follower growth is content plus genuine presence in the right conversations, not
automation X detects and suspends. This engine automates the HARD part (finding the best
threads, drafting excellent replies) and keeps the action layer deliberately small: a few
quality replies and light likes a day from @Agentik_os, gated and human-paced. The 10k
followers goal is the north star of a bounded loop (R-LOOP), measured weekly, never a
"spam until it works" loop.

## Why not full auto like/comment/follow

Mass automated engagement is the number-one pattern X spam-detection nukes. The realistic
outcome is a shadowban or suspension of the brand account, not growth. Bought/botted
followers are dead weight. So this engine refuses mass follow/unfollow, auto-DM, and
spray-reply. It does bounded, genuine, gated engagement only.

## Pipeline

```
cron (daily) → [1] RADAR   find threads → last30days --search x (ScrapeCreators)
             → [2] DRAFT   genuine on-voice replies (claude, sonnet)
             → [3] GATE    adversarial anti-spam/anti-cringe (claude, opus) — default reject
             → [4] QUEUE   dedup vs seen, cap replies + likes
             → [5] EXECUTE (armed + valid session) Playwright → bounded replies + likes, human-paced
             → [6] REPORT  HTML in ~/.omega/artifacts + Telegram alert
```

## Safety rails (non-negotiable)

- `armed` flag (`~/.omega/state/growth-engine/armed`) — kill-switch. Absent = drafts only.
- valid X session at `~/.omega/secrets/x-session.json` (else nothing posts, alert fires).
- gate `keep=true`, fingerprint unseen, under `EG_REPLIES_CAP` (6) and `EG_LIKES_CAP` (15).
- human pacing (22-75s between actions), per-char typing delay, skip already-liked.
- no em/en dash, <=280 chars, no @-spam (enforced in gate + queue, R-NODASH).

## Setup (one-time)

1. Bun runtime with Playwright: `mkdir -p ~/.omega/lib/growth-engine && cd $_ && bun add playwright`,
   then copy the executor: `cp scripts/playwright-engage.mjs ~/.omega/lib/growth-engine/`.
2. X session cookies (drives @Agentik_os): log into x.com, DevTools → Application → Cookies →
   copy `auth_token` and `ct0`, then:
   `bash scripts/x-session-setup.sh <auth_token> <ct0>` → writes ~/.omega/secrets/x-session.json.
3. Verify: `bun ~/.omega/lib/growth-engine/playwright-engage.mjs --check` → `check_ok`.

## Run

```
bash scripts/engage-run.sh                 # full run (drafts unless armed + session)
EG_MOCK=1 bash scripts/engage-run.sh       # fixtures, never posts (smoke test)
```

Env: `EG_REPLIES_CAP` (6), `EG_LIKES_CAP` (15), `EG_DAYS` (2), `EG_ANALYZE_MODEL`
(claude-sonnet-5), `EG_GATE_MODEL` (claude-opus-5), `EG_TOPICS_FILE`.

## Arm / disarm

```
touch ~/.omega/state/growth-engine/armed    # bounded auto-engagement on
rm    ~/.omega/state/growth-engine/armed     # kill-switch: drafts only
```

## Config & state

- `config/engage-topics.txt` — thread-finding queries.
- `~/.omega/state/growth-engine/seen.jsonl` — engaged fingerprints (never engage twice).
- `~/.omega/state/growth-engine/runs/<date>/` — radar, queue, results, screenshots.
- `~/.omega/artifacts/growth-engine-<date>.html` — daily report.

## Requires

- `SCRAPECREATORS_API_KEY` in ~/.omega/secrets/ (thread finding).
- `claude` CLI, `bun` + Playwright + chromium, a valid X session (setup above).
- last30days checkout at ~/.omega/repos/last30days-skill/.
