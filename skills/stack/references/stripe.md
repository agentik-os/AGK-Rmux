# Stripe — the layer that always needs a human

*Read before promising an operator that payment "works".*

---

## Why this file exists

Every other layer of the stack can be scaffolded. Stripe cannot. Products and prices
live in the operator's account, and their IDs do not exist until a human creates them.

**Never invent a `price_…` or a `prod_…`.** A wrong price ID does not fail at build,
does not fail at typecheck, and does not fail in a test. It fails at checkout, in
production, on a real customer, with money on the line. This is the single most
expensive kind of fabrication in the whole stack.

The scaffold therefore writes named placeholders (`price_REPLACE_ME_MONTHLY`) and the
checkout route returns **501 Not Implemented** while they are still in place. A loud
501 in development beats a silent failure in production.

---

## What the operator must create, in order

| Step | Where | Produces |
|---|---|---|
| 1. Product | Dashboard → Products | `prod_…` |
| 2. Monthly price | On that product | `price_…` → `STRIPE_PRICE_MONTHLY` |
| 3. Yearly price | On that product | `price_…` → `STRIPE_PRICE_YEARLY` |
| 4. API key | Developers → API keys | `sk_…` → `STRIPE_SECRET_KEY` |
| 5. Webhook endpoint | Developers → Webhooks | `whsec_…` → `STRIPE_WEBHOOK_SECRET` |

The webhook endpoint is `<your-domain>/api/stripe/webhook`.

## The three events that actually matter

Subscribing to everything is noise. For a subscription product:

| Event | What it means | What to persist |
|---|---|---|
| `checkout.session.completed` | The customer paid the first time | Link `client_reference_id` (the Clerk user or org) to the Stripe customer |
| `customer.subscription.updated` | Plan change, renewal, or failed payment | The new status and period end |
| `customer.subscription.deleted` | It is over | Revoke access at period end, not instantly |

`invoice.payment_failed` is worth adding once dunning matters. It does not before.

---

## The raw-body trap

Signature verification hashes the **exact bytes Stripe sent**. Parse the body first and
verification fails with a message that blames the secret, sending you to rotate a key
that was never wrong.

```ts
const body = await req.text();          // ✅ raw
const body = await req.json();          // ❌ verification will fail
```

The scaffolded route already does this correctly. Keep it that way.

## Test mode is a different account

Test keys (`sk_test_…`) and live keys (`sk_live_…`) have **separate** products, prices,
customers, and webhook secrets. A `price_…` created in test mode does not exist in live
mode. Migrating to production means recreating the products and swapping every ID, not
just the API key.

Say this out loud when handing over. It is the second most common surprise after the
raw-body trap.

## Testing locally

```bash
stripe login
stripe listen --forward-to localhost:3000/api/stripe/webhook
```

`stripe listen` prints its own `whsec_…` for the session. It is not the dashboard
secret, and it changes each run.

---

## What lands in Convex

The webhook route is a Next.js route, not a Convex function, because Stripe needs the
raw body. It should then call a Convex mutation to persist state. Keep the subscription
record keyed by the tenant, next to the rest of the data, so entitlement checks are a
normal indexed query and not an HTTP call to Stripe on every page load.

Never call the Stripe API to check whether a user may see a page. Read your own table.
