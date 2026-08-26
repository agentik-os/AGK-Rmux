---
name: acceptance
description: >
  OmegaOS autonomous browser-acceptance + self-heal gate. The terminal phase of a build:
  Playwright-sweeps EVERY route (200 + render), captures EVERY console error and failed
  network request, and walks the authenticated golden path with a real persisted write —
  then AUTONOMOUSLY fixes whatever it finds (missing route, dead auth bridge, console/network
  error, broken flow) and re-runs, looping until the sweep is fully green or a hard external
  blocker (a missing secret) is hit. "It builds" is never "it works" — this proves it works.
  Use when user says "/omg-acceptance", "acceptance gate", "test everything", "verify the app
  works", "browser e2e", or as the last step of /omg-new-project + /omg-planner builds.
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "Skill"]
domain: qualitygate
read_only: false
triggers: ["omg-acceptance", "acceptance gate", "test everything", "browser e2e", "verify the app works"]
---

# /omg-acceptance — Autonomous browser acceptance + self-heal

The build is NOT done when it compiles — it is done when the running app actually works in
a browser. This gate is a **closed loop**: sweep → on any failure, fix the root cause →
rebuild → sweep again → repeat until green. The agent running it is autonomous (Law L3):
it does not stop at "found a bug", it FIXES the bug and re-verifies.

## The sweep (the falsifiable truth)

`sweep.cjs` (shipped beside this file) drives a real Chromium over the running app and exits
non-zero with a JSON failure report if ANYTHING is wrong:

- **Every route** (crawled from `/` + the explicit `ROUTES`/`PROTECTED_ROUTES` list) returns
  **HTTP < 400** — catches the `/sign-in` 404, any unmatched route, any 500 render throw.
- **Zero console errors** and **zero failed network requests (status ≥ 400)** on every page —
  catches the Clerk `tokens/convex` 404 and the Convex `Unauthenticated` mutation, which are
  invisible to `npm run build`.
- **Secure context** — fails fast if `BASE_URL` is a raw `http://<IP>` (Clerk/WebCrypto auth
  silently dies there). Sweep on `http://127.0.0.1:$PORT` or HTTPS.
- **Authenticated golden path** — logs in programmatically (Clerk testing token, no flaky
  widget clicks), does the core action, and asserts the write **survives a reload** (a real
  persisted backend write, not optimistic-only UI).

Run it (from the project dir, after `npm run build` + `next start` on a port):

```bash
BASE_URL="http://127.0.0.1:$PORT" \
ROUTES="/sign-in,/sign-up" PROTECTED_ROUTES="/chat" GOLDEN_PATH="/chat" \
CLERK_PUBLISHABLE_KEY="$pk" CLERK_SECRET_KEY="$sk" \
TEST_IDENTIFIER="omega+clerk_test@example.com" TEST_PASSWORD="$pw" \
node ~/.omega/skills/acceptance/sweep.cjs
```

Need an e2e user? Create one once via the Clerk Backend API (the `+clerk_test@` convention
accepts the dev verification code `424242`):

```bash
curl -s -X POST https://api.clerk.com/v1/users -H "Authorization: Bearer $sk" \
  -H "Content-Type: application/json" \
  -d '{"email_address":["omega+clerk_test@example.com"],"password":"OmegaE2E!verify2026","skip_password_checks":true}'
```

## The heal loop (the autonomy)

```
build → serve on 127.0.0.1 → run sweep.cjs
  └─ ok:true  → DONE (report green)
  └─ ok:false → for each failure, FIX the ROOT CAUSE (table below) → rebuild → re-sweep
                repeat until ok:true, or MAX_ROUNDS (6) / no-progress (2 rounds, same
                failure set) → STOP and report the blocker honestly (never fake-pass).
```

Round budget and no-progress detection are mandatory — an unfixable failure (a genuinely
missing secret/credential you cannot self-provision) is a legitimate **blocked**, surfaced
with the exact failure, not a silent pass and not an infinite loop. Everything that IS
fixable in-repo, you fix.

### Failure → remediation playbook

| `kind` | Root cause | Autonomous fix |
|---|---|---|
| `route` HTTP 404 | route referenced (nav/`*_URL`/`<Link>`) but no page | create the page (`src/app/<route>/page.tsx`); for Clerk auth use the catch-all `[[...x]]` |
| `route` HTTP 500 | server render throws | read the stack from `next start` logs, fix the throwing component/loader |
| `network` 404 on `…clerk.com/.../tokens/convex` | **no Clerk `convex` JWT template** | create it via Backend API (`aud:"convex"`); see new-project Phase 2d |
| `console` `Unauthenticated` (Convex) | the JWT bridge isn't reaching `ctx.auth` | fix `convex/auth.config.ts` domain/applicationID + the JWT template; redeploy Convex |
| `secure-context` | `BASE_URL` is `http://<IP>` | re-serve on `127.0.0.1`/HTTPS; never validate auth on a raw IP |
| `auth` signIn failed | password strategy off / unverified e2e user | enable the strategy or use code `424242`; recreate the test user |
| `golden` "did not survive reload" | optimistic UI, write never persisted | wire the action to the real mutation; ensure the read query reloads it |
| `golden` "never appeared" | the mutation threw | open the console/network detail for that submit; fix the failing call |

Fix file-disjoint failures in parallel (spawn workers or a `/dynamic` workflow — R-ORCH);
serialize anything sharing a file (R-SCOPE). After each round, **re-run the FULL sweep** —
never trust a local fix without the independent re-verification (R-VERIFY).

## Output

Finish with the final sweep JSON (`ok:true`) inline as evidence (R-CITE), the count of
rounds, and what each round fixed. If blocked, state the single remaining failure and the
exact external thing needed to clear it. A green sweep is the only "done".
