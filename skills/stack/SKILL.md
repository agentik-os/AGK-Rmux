---
name: stack
description: >
  Build the canonical AgentikOS app stack — Next.js + Convex + Clerk + Stax, with Stripe
  as an opt-in flag — from scratch or from a Blueprint OS blueprint folder. Pulls the
  live Stax checkout before every scaffold (R-BLUEPRINT-STACK), vendors the panel engine,
  wires Convex auth to Clerk, and (with --stripe only) stubs subscriptions with an
  explicit list of the IDs the operator must provide. Use when the user says "/stack", "/omg-stack", "new app",
  "scaffold the stack", "build the app", "start a new OS", "nouveau projet Next Convex",
  "monte la stack", "construis l'app", "build from the blueprint", or names any subset of
  Next.js / Convex / Clerk / Stripe / Stax as the thing to set up. This is the BUILD half
  of the Blueprint OS chain: /blueprint-os designs, /stack builds.
triggers: ["stack","omg-stack","new app","scaffold the stack","build the app","start a new OS","build from blueprint","nouveau projet","monte la stack","construis l'app","next convex clerk","stax app"]
allowed-tools: ["Bash","Read","Write","Edit","Grep","Glob"]
domain: fullstack
read_only: false
argument-hint: "<app-name> [--blueprint <path-to-blueprint-folder>] [--dir <parent-dir>]"
source: OmegaOS — pairs with skills/blueprint-os and skills/stax
---

> **OmegaOS skill — the build half of the Blueprint OS chain.**
> `/blueprint-os` designs an OS. `/stack` turns that design into a running app on the
> canonical stack. Triggers `/stack` and `/omg-stack`.

## The stack, non negotiable (R-BLUEPRINT-STACK)

| Layer | Techno | Role |
|---|---|---|
| **Shell / UI** | **Stax** | The panel grammar. Every OS wears the same navigation |
| Front | Next.js (App Router) | The app |
| Data + realtime | Convex | The single model. Reactive by default |
| Auth | Clerk | Identity, organizations, roles |
| AI | Claude, called from a **Convex action** | Never from the client |

If a design suggests something else, the design changes, not the stack.

**Stripe is opt-in, not part of the default.** Pass `--stripe` when the app actually
charges someone. Most OS products are built and used long before they are sold, and a
billing surface nobody calls is dead code that still demands keys, a webhook endpoint,
and a dashboard setup. Scaffold it when it earns its place.

## Stax is pulled every single time

Before any scaffold, `stack-new.sh` runs the Stax sync — fast-forward only, never
clobbering local commits. The app is always vendored from the **current** `main` of
`github.com/agentik-os/stax`. The commit that was vendored is written into
`stax.lock.json` at the app root, so a build is always traceable to a Stax revision.

This is the rule, not an optimization: an app scaffolded from a stale Stax drifts from
the rest of the family, and the drift is invisible until the panel grammar disagrees.

---

## How to run it

```bash
bash ~/.omega/skills/stack/scripts/stack-new.sh <app-name> [--blueprint <dir>] [--dir <parent>] [--stripe]
```

| Flag | Effect |
|---|---|
| `--blueprint <dir>` | Read a Blueprint OS folder: phase `09-data/schema.ts` becomes the Convex schema, phase `10-stax/panneaux.md` drives the panel registry |
| `--dir <parent>` | Where the app is created. Defaults to `~/Station/SideBusiness` |
| `--stripe` | **Opt-in billing.** Adds the checkout route, the webhook route, `lib/stripe.ts`, the `stripe` dependency, and the Stripe env block |
| `--no-install` | Scaffold the files, skip `npm install` (offline / inspection) |

The script is **idempotent on a fresh dir and refuses to overwrite an existing app** —
rerunning it on a live project is a destructive act, so it stops instead (R-DESTRUCT).

---

## What the script does, in order

1. **Sync Stax** — `stax-sync.sh`, ff-only. Records the commit.
2. **`create-next-app`** — TypeScript, App Router, Tailwind, `src/`.
3. **Vendor Stax** — calls the existing `stax-scaffold.sh` (never reimplemented here):
   `panels-core.ts`, `panels-react.tsx`, `tokens.css`, `stax-ui.css` into `src/stax/`.
4. **Convex** — `convex/schema.ts` (from the blueprint if given, else the canonical
   pattern), `convex/auth.config.ts` wired to Clerk, an example query and mutation.
5. **Clerk** — middleware, provider, and the Convex `ConvexProviderWithClerk` bridge.
6. **Stripe — only with `--stripe`** — checkout route, webhook route, and `lib/stripe.ts`.
   The **price IDs are left as named placeholders** because only the operator can create
   them. Without the flag, none of this is written and `stripe` is not installed.
7. **`.env.example`** — every key, with who provides it and where to get it.
8. **`NEEDS-OPERATOR.md`** — the exact list of what a human must do before the app runs.

Then it prints the list. It never invents a key, never fakes a Stripe ID, and never
claims the app runs before `npm run dev` has actually been executed.

---

## The Stripe IDs — read this before promising anything

*Only relevant when the app was scaffolded with `--stripe`.*

Stripe is the one layer that **cannot** be fully scaffolded. Products and prices live in
the operator's Stripe account, and their IDs (`price_…`, `prod_…`) do not exist until a
human creates them. The scaffold therefore writes placeholders and lists them.

Never guess a `price_…`. A wrong price ID fails at checkout, in production, on a real
customer. See `references/stripe.md` for the full setup path and the webhook events
that actually matter.

---

## Building from a blueprint

When `--blueprint` points at a Blueprint OS folder, the chain is:

| Blueprint phase | Becomes |
|---|---|
| `09-data/schema.ts` | `convex/schema.ts`, verbatim if valid |
| `10-stax/panneaux.md` | The panel registry and the inspector stubs |
| `06-features/features.md` | The `NEEDS-BUILD.md` checklist, layer by layer |
| `08-ia/system-prompts.md` | `convex/ai/` action stubs, one per agent |

**Gate, enforced mechanically.** `stack-new.sh` runs `blueprint-check.sh --gates-only`
before it scaffolds anything, and **refuses** on an unfranchised gate. Building from a
blueprint that failed the parity gate produces a demo whose missing socle is discovered
at delivery, which is exactly the failure mode phase 5 exists to prevent.

`--force-gates` overrides it. Use it knowing what it costs, and it says so out loud.

Run the full check yourself when you want more than the gates:

```bash
bash ~/.omega/skills/blueprint-os/scripts/blueprint-check.sh <blueprint-dir>
```

---

## Rules of conduct

**Pull Stax first, always.** No exception, no "it was synced this morning".

**Never fabricate an ID or a key.** Placeholder plus a line in `NEEDS-OPERATOR.md`.

**Never start a dev server to "verify" (R-TEST)** — except here, where the code is
brand-new and not yet deployed. That is the documented exception: one `npm run dev`,
one HTTP probe, then kill it. Real runtime evidence, not a green install log (L1).

**Secrets never enter the repo (R-ENV / L0).** `.env.local` is gitignored; the
`.env.example` carries names only. Keys live in `~/.omega/secrets/<app>.env`.

**The panel grammar is not optional.** No pages, no modals, no tabs. If the app grows a
second navigation mechanic, it stopped being an AgentikOS app.

---

## Reference files

| File | When |
|---|---|
| `references/convex.md` | Schema, indexes, auth, and calling Claude from an action |
| `references/clerk.md` | The Clerk plus Convex bridge, and the JWT template that trips everyone |
| `references/stripe.md` | Products, prices, webhooks, and the IDs the operator must create |
| `references/env.md` | Every environment variable, who provides it, where it lives |
