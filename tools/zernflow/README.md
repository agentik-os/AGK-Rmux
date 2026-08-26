# zernflow — OmegaOS tool

**ZernFlow** is the open-source ManyChat alternative: a visual flow builder for
chatbot automation across Instagram, Facebook, Telegram, Twitter/X, Bluesky and
Reddit — inbox, broadcasts, drip sequences, comment-to-DM, A/B testing. It is a
self-hostable **Next.js + Supabase** app, powered by [Zernio](https://zernio.com)
for OAuth / token refresh / rate limiting / cross-platform messaging.

- **Upstream:** https://github.com/zernio-dev/zernflow
- **Pinned commit:** `78f79294e4a57de3d1b375fe05effdea4429f81c`
- **License:** MIT
- **Same vendor as** `tools/zernio` (the `omega-zernio` publishing CLI).

## Relationship to Zernio (R-ZERNIO vs R-ZERNFLOW)

Two distinct layers over the same Zernio account:

- **`omega-zernio` (R-ZERNIO)** — OUTBOUND publishing & ads: you post a reel /
  thread / ad and it goes out. One-shot, CLI, agent-invoked.
- **ZernFlow (R-ZERNFLOW)** — INBOUND engagement automation: chatbot flows,
  live-chat inbox, drip sequences, comment-to-DM. A standing web app the
  operator runs, not a one-shot CLI call.

## Provisioned backend (Agentik OS)

A **dedicated** Supabase project backs this install (R-PROJ — never shared with
another project):

- **Project ref:** `mbsncijxqvawawpgjbkp` (org `Agentik OS`, eu-west-1)
- **URL:** `https://mbsncijxqvawawpgjbkp.supabase.co`
- **Schema:** 10 migrations applied → 23 tables, RLS on every table.
- **Credentials:** `~/.omega/secrets/zernflow.env` (anon + service_role keys,
  DB password, CRON_SECRET). The account-wide Supabase Management token
  (`sbp_…`) lives separately in `~/.omega/secrets/supabase.env` — different
  scope, never co-located (R-ENV / L0). Nothing secret is tracked in the repo.

## Install (opt-in — NOT auto-run by install.sh)

ZernFlow is a full Next.js app (heavy `npm install`), so — like higgsfield and
browser-use — OmegaOS ships the tool markdown + installer but does **not** clone
or build it on every `install.sh` run. Run it explicitly when you want a local
working tree:

```bash
bash tools/zernflow/install-zernflow.sh
# → clones the pinned commit into ~/.omega/repos/zernflow
#   writes .env from ~/.omega/secrets/zernflow.env
#   runs npm install
```

The Supabase schema is already migrated on the dedicated project above, so a
fresh clone is immediately backed by a live database.

## Run

```bash
cd ~/.omega/repos/zernflow
npm run dev      # local, http://localhost:3000
```

Then in the app **Settings** UI, paste the workspace-level keys ZernFlow expects
at runtime (Zernio API key, optional AI Gateway key).

## Deploy (Vercel — LIVE)

**Live:** https://zernflow-ten.vercel.app

- **Vercel team:** agentik-oss-projects (`team_WrIUaFWi6SG4SUo5yK0NpHjc`)
- **Vercel project:** `zernflow` (`prj_1I8Sg0kVVeYM90khXXgdHF2Cj9Q9`)
- **Token/scope:** `~/.omega/secrets/vercel-agentik-os.env` (R-VERCEL — always
  `--token`, the VPS is headless).
- **Env vars set in Vercel (Production):** `NEXT_PUBLIC_SUPABASE_URL`,
  `NEXT_PUBLIC_SUPABASE_ANON_KEY`, `SUPABASE_SERVICE_ROLE_KEY`, `CRON_SECRET`,
  `NEXT_PUBLIC_APP_URL` — sourced from `~/.omega/secrets/zernflow.env`.

Redeploy after a code or env change:

```bash
source ~/.omega/secrets/vercel-agentik-os.env
cd ~/.omega/repos/zernflow
vercel deploy --prod --yes --scope "$VERCEL_TEAM_ID" --token "$VERCEL_TOKEN"
```

`vercel.json` declares two per-minute crons (`/api/cron/jobs`,
`/api/cron/sequences`) guarded by `CRON_SECRET` (needs a Pro team — accepted on
agentik-oss-projects). Prod-verified 2026-07-09 (R-PROD / L1): `/` and `/login`
→ 200, login page renders with a clean console, `/api/cron/jobs` → 401 without
the secret and `{"processed":0,...}` with it. Remaining golden path is
operator-driven: sign-up → create workspace → connect a channel via Zernio →
build a flow → inbox receives a message.

Note: Vercel could not auto-connect the upstream Git repo (`zernio-dev/zernflow`,
no write access), so deploys are **CLI-driven from `~/.omega/repos/zernflow`**,
not git-push-triggered.

## Files

- `README.md` — this file (SSOT for the tool in the OmegaOS ecosystem).
- `install-zernflow.sh` — idempotent opt-in installer (clone pinned + .env + npm).
