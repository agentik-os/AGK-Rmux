# Environment variables — who provides what, and where it lives

---

## The full list

| Variable | Public? | Who provides it | Where to get it |
|---|---|---|---|
| `NEXT_PUBLIC_CONVEX_URL` | public | generated | `npx convex dev` writes it |
| `NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY` | public | operator | Clerk → API keys |
| `CLERK_SECRET_KEY` | **secret** | operator | Clerk → API keys |
| `CLERK_JWT_ISSUER_DOMAIN` | public | operator | Clerk → JWT templates → `convex` → Issuer |
| `STRIPE_SECRET_KEY` | **secret** | operator | Stripe → Developers → API keys |
| `STRIPE_WEBHOOK_SECRET` | **secret** | operator | Stripe → Webhooks → signing secret |
| `STRIPE_PRICE_MONTHLY` | config | **operator must create** | Stripe → Products → price |
| `STRIPE_PRICE_YEARLY` | config | **operator must create** | Stripe → Products → price |
| `NEXT_PUBLIC_APP_URL` | public | you | `http://localhost:3000`, then the real domain |

`NEXT_PUBLIC_` is shipped to the browser. Anything else must never be prefixed with it.
Putting a Stripe secret behind `NEXT_PUBLIC_` publishes it to every visitor, and the
mistake is invisible until someone reads the bundle.

---

## Two places, and only two

| File | Contents | Tracked by git? |
|---|---|---|
| `.env.example` | Names only, with comments | **Yes** — it documents the app |
| `.env.local` | The real values | **Never** |

Plus the mirror: `~/.omega/secrets/<app>.env` holds the real values outside the repo, so
a wiped checkout does not lose them.

A secret in git history is compromised forever, even after the commit is removed, even
in a private repo. If one is ever pushed, the remediation is to **rotate the credential**
first; scrubbing history without rotating fixes nothing.

---

## Convex needs its own copy

Convex functions run on Convex's servers and cannot read `.env.local`. Anything a Convex
function needs must be set on the deployment:

```bash
npx convex env set CLERK_JWT_ISSUER_DOMAIN https://…
npx convex env set ANTHROPIC_API_KEY sk-ant-…
npx convex env list
```

Forgetting this produces a function that works locally in the Next.js route and fails in
the Convex action, with an error that reads like a code bug.

---

## Production

Vercel: set every variable in the project settings, per environment. The build does not
inherit `.env.local`.

Deploys pass the token explicitly (`vercel --prod --token=$VERCEL_TOKEN`) — a headless
machine has no browser for the interactive login, so an untokened deploy just hangs.

Remember that Stripe test and live modes have **different** price IDs and a different
webhook secret. Going live means new IDs, not just a new API key.
