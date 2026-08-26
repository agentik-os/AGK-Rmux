#!/usr/bin/env bash
# stack-new.sh — scaffold the canonical AgentikOS app stack:
#   Next.js (App Router, TS, Tailwind) + Convex + Clerk + Stripe + Stax
#
# Stax is PULLED before every scaffold (R-BLUEPRINT-STACK) and the vendored commit is
# recorded in stax.lock.json, so a build is always traceable to a Stax revision.
#
# Composes the existing OmegaOS tooling instead of reimplementing it:
#   - skills/stax/scripts/stax-sync.sh      → ff-only pull of every Stax checkout
#   - skills/stax/scripts/stax-scaffold.sh  → vendors the panel engine + design system
#
# Refuses to touch an existing app directory (R-DESTRUCT): scaffolding over a live
# project would overwrite hand-written wiring, so it stops instead of guessing.
set -uo pipefail

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
STAX_REPO="$OMEGA_DIR/repos/stax"
STAX_SKILL="$OMEGA_DIR/skills/stax"

c_info(){ printf '\033[36m[stack]\033[0m %s\n' "$*"; }
c_ok(){   printf '\033[32m[stack]\033[0m %s\n' "$*"; }
c_warn(){ printf '\033[33m[stack]\033[0m %s\n' "$*"; }
c_die(){  printf '\033[31m[stack]\033[0m %s\n' "$*" >&2; exit 1; }

# ── args ────────────────────────────────────────────────────────────────────────
APP=""; BLUEPRINT=""; PARENT="$HOME/Station/SideBusiness"; DO_INSTALL=1; WITH_STRIPE=0; FORCE_GATES=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --blueprint) BLUEPRINT="${2:-}"; shift 2;;
    --dir)       PARENT="${2:-}";    shift 2;;
    --stripe)    WITH_STRIPE=1;      shift;;
    --force-gates) FORCE_GATES=1;    shift;;
    --no-install) DO_INSTALL=0;      shift;;
    -h|--help)   sed -n '2,12p' "$0"; exit 0;;
    -*)          c_die "unknown flag: $1";;
    *)           [[ -z "$APP" ]] && APP="$1" || c_die "unexpected arg: $1"; shift;;
  esac
done
[[ -n "$APP" ]] || c_die "usage: stack-new.sh <app-name> [--blueprint <dir>] [--dir <parent>] [--stripe] [--no-install]"

# Slug: lowercase, non-alnum → dash. npm package names refuse anything else.
SLUG="$(printf '%s' "$APP" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]\+/-/g; s/^-//; s/-$//')"
[[ -n "$SLUG" ]] || c_die "app name reduces to an empty slug: $APP"
TARGET="$PARENT/$SLUG"

[[ -e "$TARGET" ]] && c_die "refusing to scaffold over an existing path: $TARGET
Pick another name, or move the existing directory yourself first."

# Case-insensitive collision guard. The slug is lowercased, so `Stax` becomes `stax`:
# on Linux that is a DIFFERENT directory (and we would silently create a confusing
# sibling), on macOS it is the SAME one (and we would overwrite it). Refuse both.
if [[ -d "$PARENT" ]]; then
  CLASH="$(ls -1 "$PARENT" 2>/dev/null | awk -v s="$SLUG" 'tolower($0)==s {print; exit}')"
  [[ -n "$CLASH" ]] && c_die "refusing: \"$PARENT/$CLASH\" already exists and differs from \"$SLUG\" only by case.
On a case-insensitive filesystem this would overwrite it. Pick another name."
fi

if [[ -n "$BLUEPRINT" ]]; then
  [[ -d "$BLUEPRINT" ]] || c_die "blueprint folder not found: $BLUEPRINT"
  c_info "blueprint: $BLUEPRINT"
  # Les gates avant tout scaffold. Construire une app sur un blueprint qui a rate son
  # gate parite produit une demo dont le socle manquant est decouvert a la livraison,
  # ce qui est exactement le mode d'echec que la phase 5 existe pour empecher.
  BP_CHECK="$OMEGA_DIR/skills/blueprint-os/scripts/blueprint-check.sh"
  if [[ -x "$BP_CHECK" ]]; then
    if bash "$BP_CHECK" "$BLUEPRINT" --gates-only --quiet >/dev/null 2>&1; then
      c_ok "les 3 gates du blueprint sont franchis"
    elif [[ "$FORCE_GATES" -eq 1 ]]; then
      c_warn "gates NON franchis, mais --force-gates est passe. Le socle manquant se decouvrira a la livraison."
    else
      c_warn "un ou plusieurs GATES ne sont pas franchis sur ce blueprint:"
      bash "$BP_CHECK" "$BLUEPRINT" --gates-only 2>&1 | sed 's/^/    /'
      c_die "refus de scaffolder. Corriger le blueprint, ou relancer en connaissance de cause avec --force-gates."
    fi
  else
    c_warn "blueprint-check absent — les gates ne sont PAS verifies"
  fi
fi

command -v node >/dev/null 2>&1 || c_die "node is required"
command -v npx  >/dev/null 2>&1 || c_die "npx is required"

# ── 1. pull Stax — the rule, every time ─────────────────────────────────────────
c_info "pulling Stax (ff-only) before scaffolding"
if [[ -x "$STAX_SKILL/scripts/stax-sync.sh" ]]; then
  bash "$STAX_SKILL/scripts/stax-sync.sh" >/dev/null 2>&1 || c_warn "stax-sync reported warnings (non-fatal)"
elif [[ -d "$STAX_REPO/.git" ]]; then
  git -C "$STAX_REPO" fetch origin >/dev/null 2>&1 && git -C "$STAX_REPO" merge --ff-only >/dev/null 2>&1 || c_warn "manual ff-pull had warnings"
else
  c_info "no Stax checkout yet — cloning agentik-os/stax"
  mkdir -p "$OMEGA_DIR/repos"
  git clone https://github.com/agentik-os/stax "$STAX_REPO" >/dev/null 2>&1 || c_die "stax clone failed — check network"
fi
STAX_COMMIT="$(git -C "$STAX_REPO" rev-parse HEAD 2>/dev/null || echo unknown)"
STAX_DATE="$(git -C "$STAX_REPO" log -1 --format=%cI 2>/dev/null || echo unknown)"
c_ok "Stax @ ${STAX_COMMIT:0:12} ($STAX_DATE)"

# ── 2. Next.js ──────────────────────────────────────────────────────────────────
mkdir -p "$PARENT"
c_info "create-next-app → $TARGET"
NEXT_FLAGS=(--typescript --tailwind --eslint --app --src-dir --import-alias "@/*" --use-npm --no-turbopack --yes)
if ! npx --yes create-next-app@latest "$TARGET" "${NEXT_FLAGS[@]}" 2>&1 | tail -5; then
  c_die "create-next-app failed"
fi
[[ -d "$TARGET/src/app" ]] || c_die "create-next-app produced no src/app — aborting"
c_ok "Next.js app created"

cd "$TARGET" || c_die "cannot enter $TARGET"

# ── 3. vendor Stax (reuse the existing scaffolder) ──────────────────────────────
if [[ -x "$STAX_SKILL/scripts/stax-scaffold.sh" ]]; then
  c_info "vendoring the Stax engine + design system"
  bash "$STAX_SKILL/scripts/stax-scaffold.sh" "$TARGET" 2>&1 | sed 's/^/    /' || c_warn "stax-scaffold reported warnings"
else
  c_warn "stax-scaffold.sh not found at $STAX_SKILL — the panel engine was NOT vendored"
fi

cat > stax.lock.json <<JSON
{
  "repo": "github.com/agentik-os/stax",
  "commit": "$STAX_COMMIT",
  "committed_at": "$STAX_DATE",
  "vendored_by": "stack-new.sh",
  "app": "$SLUG"
}
JSON
c_ok "stax.lock.json written (traceable build)"

# ── 4. Convex ───────────────────────────────────────────────────────────────────
mkdir -p convex
BP_SCHEMA="$BLUEPRINT/09-data/schema.ts"
if [[ -n "$BLUEPRINT" && -f "$BP_SCHEMA" ]]; then
  cp "$BP_SCHEMA" convex/schema.ts
  c_ok "convex/schema.ts taken from the blueprint (phase 09)"
else
  cat > convex/schema.ts <<'TS'
import { defineSchema, defineTable } from "convex/server";
import { v } from "convex/values";

// The canonical AgentikOS data pattern.
// Replace `items` with the blueprint's PRIMITIVE — it is always the first table.
export default defineSchema({
  // 1 — THE PRIMITIVE. Always first. Rename it to the object the product is about.
  items: defineTable({
    tenantId: v.string(),
    title: v.string(),
    createdAt: v.number(),
  }).index("by_tenant", ["tenantId"]),

  // 2 — THE SIGNALS. Mandatory: this is what makes cross-reading possible.
  entries: defineTable({
    tenantId: v.string(),
    actorId: v.string(),
    date: v.string(), // YYYY-MM-DD
    kind: v.string(),
    payload: v.any(),
    createdAt: v.number(),
  })
    .index("by_tenant_date", ["tenantId", "date"])
    .index("by_actor_kind", ["actorId", "kind"]),

  // 3 — THE AI OUTPUT. Always separate from the signals.
  //     Never mix what was observed with what was interpreted.
  syntheses: defineTable({
    tenantId: v.string(),
    scope: v.string(),
    observation: v.string(),
    correlation: v.optional(v.string()), // "nothing notable" is a valid answer
    citations: v.array(v.string()),
    proposals: v.array(v.string()),
    model: v.string(),
    createdAt: v.number(),
  }).index("by_tenant_scope", ["tenantId", "scope"]),
});
TS
  c_ok "convex/schema.ts written (canonical pattern — rename the primitive)"
fi

# Convex gets its OWN tsconfig and the root one excludes it. Without this split
# `next build` typechecks convex/ and dies on ./_generated/server, which does not
# exist before the first `npx convex dev`. Building the interface has no reason to
# wait on a Convex account. Convex typechecks that folder itself on every push.
cat > convex/tsconfig.json <<'JSON'
{
  "compilerOptions": {
    "allowJs": true,
    "strict": true,
    "moduleResolution": "Bundler",
    "jsx": "react-jsx",
    "skipLibCheck": true,
    "allowSyntheticDefaultImports": true,
    "target": "ESNext",
    "lib": ["ES2021", "dom", "dom.iterable"],
    "forceConsistentCasingInFileNames": true,
    "module": "ESNext",
    "isolatedModules": true,
    "noEmit": true
  },
  "include": ["./**/*"],
  "exclude": ["./_generated"]
}
JSON
node -e '
const fs=require("fs"),p="tsconfig.json";
const t=JSON.parse(fs.readFileSync(p,"utf8"));
t.exclude=[...new Set([...(t.exclude||["node_modules"]),"convex"])];
fs.writeFileSync(p,JSON.stringify(t,null,2)+"\n");
' 2>/dev/null || c_warn "tsconfig.json non modifié — exclure convex/ à la main"

cat > convex/auth.config.ts <<'TS'
// Clerk is the identity provider; Convex validates its JWT.
// CLERK_JWT_ISSUER_DOMAIN is your Clerk Frontend API URL, e.g.
//   https://verb-noun-00.clerk.accounts.dev
// It must match the "convex" JWT template you create in the Clerk dashboard.
export default {
  providers: [
    {
      domain: process.env.CLERK_JWT_ISSUER_DOMAIN,
      applicationID: "convex",
    },
  ],
};
TS

# The example functions must match the schema that is actually in place. With a
# blueprint schema, writing a hardcoded `items.ts` produces functions pointing at a
# table that does not exist — it survives `tsc` (no _generated types yet) and only
# blows up on the first `npx convex dev`. So: read the real primitive out of the
# schema and generate against it.
PRIMITIVE="$(grep -oE '^  [a-zA-Z_][a-zA-Z0-9_]*: defineTable' convex/schema.ts | head -1 | sed 's/^  //; s/: defineTable//')"
TABLE_BLOCK="$(sed -n "/^  ${PRIMITIVE}: defineTable/,/^  [a-zA-Z_][a-zA-Z0-9_]*: defineTable/p" convex/schema.ts 2>/dev/null)"
TENANT_FIELD="$(printf '%s' "$TABLE_BLOCK" | grep -oE '(clubId|tenantId|orgId|workspaceId|accountId):' | head -1 | tr -d ':')"
TENANT_INDEX="$(printf '%s' "$TABLE_BLOCK" | grep -oE "\.index\(\"[^\"]+\", \[\"${TENANT_FIELD:-__none__}\"" | head -1 | grep -oE '"[^"]+"' | head -1 | tr -d '"')"

if [[ -n "$PRIMITIVE" && -n "$TENANT_FIELD" && -n "$TENANT_INDEX" ]]; then
  cat > "convex/${PRIMITIVE}.ts" <<TS
import { query } from "./_generated/server";

// Example read against the primitive of this app's schema: \`${PRIMITIVE}\`.
//
// Every function derives the tenant from the authenticated identity.
// NEVER accept a ${TENANT_FIELD} argument from the client — that is one crafted
// request away from reading another tenant's data.
async function requireTenant(ctx: { auth: { getUserIdentity: () => Promise<any> } }) {
  const identity = await ctx.auth.getUserIdentity();
  if (!identity) throw new Error("Not authenticated");
  return (identity.org_id as string) ?? (identity.subject as string);
}

export const list = query({
  args: {},
  handler: async (ctx) => {
    const ${TENANT_FIELD} = await requireTenant(ctx);
    return await ctx.db
      .query("${PRIMITIVE}")
      .withIndex("${TENANT_INDEX}", (q) => q.eq("${TENANT_FIELD}", ${TENANT_FIELD}))
      .order("desc")
      .take(100);
  },
});

// Mutations are deliberately NOT scaffolded here: the required fields of
// \`${PRIMITIVE}\` come from your schema, and guessing them would generate code that
// compiles and then fails at runtime. Write them against the real table.
TS
    c_ok "convex/ auth config + example query on the real primitive: ${PRIMITIVE} (tenant: ${TENANT_FIELD})"
else
  cat > convex/README.md <<'MD'
# Convex functions

No example query was generated: the primitive, its tenant field, or a tenant index
could not be detected in `schema.ts`.

Write your functions against the real schema, and derive the tenant from the
authenticated identity — never from a client argument.
MD
  c_warn "could not detect primitive/tenant in schema.ts — wrote convex/README.md instead of a wrong example"
fi

# ── 5. Clerk bridge ─────────────────────────────────────────────────────────────
mkdir -p src/lib

cat > src/middleware.ts <<'TS'
import { clerkMiddleware } from "@clerk/nextjs/server";
import { NextResponse } from "next/server";

// Clerk is only mounted when its key exists. Without this guard a freshly
// scaffolded app answers 500 on every route before a Clerk account is even
// created, and nothing of the interface can be seen. A first run that fails is
// what makes people abandon a tool halfway through setting it up.
const hasClerk = Boolean(process.env.NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY);

export default hasClerk ? clerkMiddleware() : () => NextResponse.next();

export const config = {
  matcher: [
    // Everything except Next internals and static files, plus every API route.
    "/((?!_next|[^?]*\\.(?:html?|css|js(?!on)|jpe?g|webp|png|gif|svg|ttf|woff2?|ico|csv|docx?|xlsx?|zip|webmanifest)).*)",
    "/(api|trpc)(.*)",
  ],
};
TS

cat > src/app/providers.tsx <<'TSX'
"use client";

import { ClerkProvider, useAuth } from "@clerk/nextjs";
import { ConvexReactClient } from "convex/react";
import { ConvexProviderWithClerk } from "convex/react-clerk";
import type { ReactNode } from "react";

const convex = new ConvexReactClient(process.env.NEXT_PUBLIC_CONVEX_URL!);

export function Providers({ children }: { children: ReactNode }) {
  return (
    <ClerkProvider publishableKey={process.env.NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY!}>
      <ConvexProviderWithClerk client={convex} useAuth={useAuth}>
        {children}
      </ConvexProviderWithClerk>
    </ClerkProvider>
  );
}
TSX
c_ok "Clerk middleware + Convex/Clerk provider bridge written"

# ── 6. Stripe — OPT-IN ONLY (--stripe) ──────────────────────────────────────────
# Billing is not part of the default stack: most OS products are built and used long
# before they are sold, and an unused Stripe surface is dead code that still needs keys.
# Pass --stripe when the app actually charges someone.
if [[ "$WITH_STRIPE" -eq 1 ]]; then
mkdir -p src/app/api/stripe/checkout src/app/api/stripe/webhook
cat > src/lib/stripe.ts <<'TS'
import Stripe from "stripe";

// No apiVersion pin on purpose: the SDK's own default always matches the installed
// package, while a hardcoded version string goes stale and fails typecheck on the next
// stripe upgrade. Pin it here only if you deliberately need an older API version.
export const stripe = new Stripe(process.env.STRIPE_SECRET_KEY!);

// PLACEHOLDERS — these price IDs do not exist until the operator creates the
// products in the Stripe dashboard. Never guess a price_… : a wrong ID fails at
// checkout, in production, on a real customer. See NEEDS-OPERATOR.md.
export const PRICES = {
  monthly: process.env.STRIPE_PRICE_MONTHLY ?? "price_REPLACE_ME_MONTHLY",
  yearly: process.env.STRIPE_PRICE_YEARLY ?? "price_REPLACE_ME_YEARLY",
} as const;
TS

cat > src/app/api/stripe/checkout/route.ts <<'TS'
import { auth } from "@clerk/nextjs/server";
import { NextResponse } from "next/server";
import { stripe, PRICES } from "@/lib/stripe";

export async function POST(req: Request) {
  const { userId } = await auth();
  if (!userId) return NextResponse.json({ error: "unauthenticated" }, { status: 401 });

  const { plan } = (await req.json()) as { plan?: keyof typeof PRICES };
  const price = PRICES[plan ?? "monthly"];
  if (price.startsWith("price_REPLACE_ME")) {
    return NextResponse.json(
      { error: "Stripe price ID not configured — see NEEDS-OPERATOR.md" },
      { status: 501 },
    );
  }

  const origin = process.env.NEXT_PUBLIC_APP_URL ?? new URL(req.url).origin;
  const session = await stripe.checkout.sessions.create({
    mode: "subscription",
    line_items: [{ price, quantity: 1 }],
    client_reference_id: userId,
    success_url: `${origin}/?checkout=success`,
    cancel_url: `${origin}/?checkout=cancelled`,
  });

  return NextResponse.json({ url: session.url });
}
TS

cat > src/app/api/stripe/webhook/route.ts <<'TS'
import { NextResponse } from "next/server";
import { stripe } from "@/lib/stripe";

// Stripe needs the RAW body to verify the signature — never parse it first.
export async function POST(req: Request) {
  const signature = req.headers.get("stripe-signature");
  if (!signature) return NextResponse.json({ error: "no signature" }, { status: 400 });

  const body = await req.text();
  let event;
  try {
    event = stripe.webhooks.constructEvent(body, signature, process.env.STRIPE_WEBHOOK_SECRET!);
  } catch (err) {
    return NextResponse.json({ error: `signature failed: ${String(err)}` }, { status: 400 });
  }

  switch (event.type) {
    case "checkout.session.completed":
    case "customer.subscription.updated":
    case "customer.subscription.deleted":
      // TODO: persist the subscription state in Convex, keyed by client_reference_id.
      break;
    default:
      break;
  }

  return NextResponse.json({ received: true });
}
TS
c_ok "Stripe checkout + webhook routes written (price IDs = placeholders)"
else
  c_info "Stripe skipped (opt-in — pass --stripe when the app actually charges)"
fi

# ── 7. env + operator checklist ─────────────────────────────────────────────────
cat > .env.example <<'ENV'
# ── Convex ─────────────────────────────────────────────────────────────────────
# Generated by `npx convex dev` on first run. Not a secret.
NEXT_PUBLIC_CONVEX_URL=

# ── Clerk ──────────────────────────────────────────────────────────────────────
# Clerk dashboard → API keys. The publishable key is public; the secret is not.
NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY=
CLERK_SECRET_KEY=
# Clerk dashboard → JWT templates → new template named exactly "convex".
# This is that template's Issuer URL (your Frontend API URL).
CLERK_JWT_ISSUER_DOMAIN=

# ── App ────────────────────────────────────────────────────────────────────────
NEXT_PUBLIC_APP_URL=http://localhost:3000
ENV

if [[ "$WITH_STRIPE" -eq 1 ]]; then
cat >> .env.example <<'ENV'

# ── Stripe (this app was scaffolded with --stripe) ─────────────────────────────
# Stripe dashboard → Developers → API keys.
STRIPE_SECRET_KEY=
# Stripe dashboard → Developers → Webhooks → your endpoint → signing secret.
STRIPE_WEBHOOK_SECRET=
# Product price IDs — OPERATOR MUST CREATE THESE. Never invent a price_… value.
STRIPE_PRICE_MONTHLY=
STRIPE_PRICE_YEARLY=
ENV
fi

cat > NEEDS-OPERATOR.md <<MD
# What a human must do before this app runs

Scaffolded by \`/stack\` on the canonical AgentikOS stack.
Stax vendored at \`${STAX_COMMIT:0:12}\`.

Nothing below can be done by an agent: every item needs an account, a dashboard,
or a decision. The app builds without them; it does not *work* without them.

## 1. Convex

\`\`\`bash
npx convex dev
\`\`\`

Creates the deployment and writes \`NEXT_PUBLIC_CONVEX_URL\` into \`.env.local\`.
Then set the Clerk issuer on the Convex deployment:

\`\`\`bash
npx convex env set CLERK_JWT_ISSUER_DOMAIN <your-clerk-frontend-api-url>
\`\`\`

## 2. Clerk

1. Create the application at dashboard.clerk.com
2. Copy \`NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY\` and \`CLERK_SECRET_KEY\`
3. **JWT templates → New template → name it exactly \`convex\`** (this is the step
   everyone misses; without it Convex rejects every authenticated call)
4. Copy that template's Issuer URL into \`CLERK_JWT_ISSUER_DOMAIN\`

## 3. Secrets location

Real values go in \`.env.local\` (gitignored) and are mirrored to
\`~/.omega/secrets/${SLUG}.env\`. They never enter the repo.

## 4. First run

\`\`\`bash
npm run dev        # Next.js
npx convex dev     # Convex, in a second terminal
\`\`\`

## 5. Expected typecheck errors before step 1

Until \`npx convex dev\` has run once, \`convex/_generated/\` does not exist, so
\`tsc --noEmit\` reports:

\`\`\`
convex/items.ts: Cannot find module './_generated/server'
convex/items.ts: Parameter 'ctx' implicitly has an 'any' type
\`\`\`

This is normal, not a broken scaffold. Convex generates that folder from your schema on
its first run, and the errors disappear. Anything *else* in the typecheck output is real.
MD

if [[ "$WITH_STRIPE" -eq 1 ]]; then
cat >> NEEDS-OPERATOR.md <<'MD'

## 6. Stripe — the IDs (scaffolded with `--stripe`)

The scaffold cannot create these. Products and prices live in your Stripe account
and their IDs do not exist until you make them.

1. Stripe dashboard → Products → create the product
2. Add a **monthly** price and a **yearly** price
3. Copy each `price_…` into `STRIPE_PRICE_MONTHLY` / `STRIPE_PRICE_YEARLY`
4. Developers → API keys → `STRIPE_SECRET_KEY`
5. Developers → Webhooks → add endpoint `<your-domain>/api/stripe/webhook`,
   subscribe to `checkout.session.completed`, `customer.subscription.updated`,
   `customer.subscription.deleted`, then copy the signing secret into
   `STRIPE_WEBHOOK_SECRET`

Until these are set, `/api/stripe/checkout` returns **501** on purpose rather than
failing on a fake ID.
MD
fi
c_ok ".env.example + NEEDS-OPERATOR.md written"

# ── 8. deps ─────────────────────────────────────────────────────────────────────
grep -q '^\.env\.local$\|^\.env\*' .gitignore 2>/dev/null || printf '\n.env.local\n.env*.local\n' >> .gitignore

DEPS=(convex @clerk/nextjs)
[[ "$WITH_STRIPE" -eq 1 ]] && DEPS+=(stripe)
if [[ "$DO_INSTALL" -eq 1 ]]; then
  c_info "installing ${DEPS[*]} (this takes a minute)"
  npm install "${DEPS[@]}" 2>&1 | tail -3 || c_warn "npm install reported warnings"
  c_ok "dependencies installed"
else
  c_warn "--no-install: run  npm install ${DEPS[*]}  yourself"
fi

# ── 9. blueprint carry-over ─────────────────────────────────────────────────────
if [[ -n "$BLUEPRINT" ]]; then
  mkdir -p docs/blueprint
  for f in "$BLUEPRINT"/*-blueprint.md "$BLUEPRINT"/blueprint.json; do
    [[ -f "$f" ]] && cp "$f" docs/blueprint/ 2>/dev/null
  done
  [[ -f "$BLUEPRINT/10-stax/panneaux.md" ]] && cp "$BLUEPRINT/10-stax/panneaux.md" docs/blueprint/ 2>/dev/null
  [[ -f "$BLUEPRINT/06-features/features.md" ]] && cp "$BLUEPRINT/06-features/features.md" docs/blueprint/ 2>/dev/null
  c_ok "blueprint carried into docs/blueprint/ (panels + features drive the build)"

  # Write the build back into the blueprint. Without this the blueprint never knows
  # it was built, so nothing can later detect that the two have drifted apart.
  if [[ -f "$BLUEPRINT/blueprint.json" ]] && command -v python3 >/dev/null 2>&1; then
    python3 - "$BLUEPRINT/blueprint.json" "$TARGET" "$STAX_COMMIT" "$(date -Is)" <<'PY' && \
      c_ok "build recorded back into blueprint.json (path, stax commit, timestamp)"
import json,sys,hashlib,os
p,target,commit,ts = sys.argv[1:5]
d = json.load(open(p))
# Fingerprint the schema that was actually shipped: if the blueprint's phase 09
# changes later, the app is provably out of date and a rebuild is owed.
sha = None
sp = os.path.join(os.path.dirname(p), "09-data", "schema.ts")
if os.path.exists(sp):
    sha = hashlib.sha256(open(sp,"rb").read()).hexdigest()[:12]
d["build"] = {"construit": True, "chemin_app": target, "stax_commit": commit,
              "construit_le": ts, "schema_sha": sha}
json.dump(d, open(p,"w"), indent=2, ensure_ascii=False)
PY
  fi
fi

git init -q 2>/dev/null || true

printf '\n'
c_ok "app scaffolded: $TARGET"
c_info "Stax commit: $STAX_COMMIT"
if [[ "$WITH_STRIPE" -eq 1 ]]; then
  c_warn "READ NEEDS-OPERATOR.md — Convex, Clerk and the Stripe price IDs need a human."
else
  c_warn "READ NEEDS-OPERATOR.md — Convex and Clerk need a human. (Stripe: rerun with --stripe when the app charges.)"
fi
