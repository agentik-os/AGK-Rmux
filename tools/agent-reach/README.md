# Agent Reach — internet reach for every agent

[Agent Reach](https://github.com/Panniantong/agent-reach) (MIT) gives an agent the
ability to actually READ the internet instead of coming back with HTML soup or a 403:
X/Twitter, Reddit, YouTube transcripts, Bilibili, Xiaohongshu, RSS, GitHub, a clean
web reader, and AI search.

## How it ships

Installed automatically by `install.sh`, and refreshed by `omega update` (which
re-runs the installer). Nothing to do by hand.

- source, pinned: `~/.omega/repos/agent-reach`
- venv: `~/.omega/tools/agent-reach/.venv`
- CLI: `agent-reach` on PATH
- skill: `~/.omega/skills/agent-reach/` (English locale), linked by `omega sync`

The skill goes into the OmegaOS skills SSOT rather than `~/.claude/skills`, which
the tool would write to by default — that directory is managed, and a flat name
dropped there can be removed by a relink.

## What works with no configuration

Verified on a headless VPS, 2026-07-29: **5 of 15 channels live with zero keys** —
YouTube (info + transcripts), V2EX, RSS/Atom, any web page (via Jina Reader), and
Bilibili search.

Optional keys unlock the rest. They live in `~/.omega/secrets/integrations.env`,
never in this repo:

| Key | Unlocks |
|---|---|
| `EXA_API_KEY` | full-web semantic search (free tier 1000/month) |
| `GROQ_API_KEY` | video/audio transcription (Whisper) |
| `GITHUB_TOKEN` | higher GitHub rate limits |
| `OPENAI_API_KEY` | transcription fallback |

Semantic search also wants `npm install -g mcporter`.

## Security boundary — read before bumping the pin

Reviewed at `b4d52c46` (v1.5.0), 2026-07-29:

- no `sudo` anywhere in the package
- no `curl | sh` executed — the two occurrences are printed advice when Node.js
  is missing, not commands it runs
- no obfuscated payloads, no `eval`/`base64 -d` of remote content
- outbound hosts are the platforms it reads plus the APIs you configure yourself

**One thing to know:** `agent_reach/cookie_extract.py` can read the local browser's
cookie store (Chrome/Firefox/Edge) via `browser-cookie3`. That is its documented
mechanism for login-gated platforms — but it can read the session cookie of *any*
site in that profile. The installer therefore installs the **base package only**;
the `cookies` extra is a deliberate operator decision:

```bash
~/.omega/tools/agent-reach/.venv/bin/pip install browser-cookie3
```

On a headless box there is no browser profile, so this is inert there.

Bumping `PIN` in `install-agent-reach.sh` means re-reviewing the above.
