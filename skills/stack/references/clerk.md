# Clerk plus Convex — the bridge, and the step everyone misses

---

## The step everyone misses

Convex does not trust Clerk by default. It validates a **JWT template** that you must
create by hand:

> Clerk dashboard → **JWT templates** → New template → name it **exactly** `convex`

Without it, every authenticated Convex call fails, and the error blames the token rather
than the missing template. This costs people an afternoon roughly every time.

The template's **Issuer URL** (your Clerk Frontend API URL, e.g.
`https://verb-noun-00.clerk.accounts.dev`) goes into two places:

```bash
# .env.local — for Next.js
CLERK_JWT_ISSUER_DOMAIN=https://verb-noun-00.clerk.accounts.dev

# and on the Convex deployment itself
npx convex env set CLERK_JWT_ISSUER_DOMAIN https://verb-noun-00.clerk.accounts.dev
```

Setting it in only one of the two is the second-most common failure.

---

## The three pieces the scaffold writes

| File | Role |
|---|---|
| `src/middleware.ts` | `clerkMiddleware()` — makes `auth()` available everywhere |
| `src/app/providers.tsx` | `ClerkProvider` wrapping `ConvexProviderWithClerk` |
| `convex/auth.config.ts` | Tells Convex which issuer to trust, `applicationID: "convex"` |

The provider order matters: Clerk **outside**, Convex **inside**. Convex needs Clerk's
`useAuth` to already exist when it mounts.

Wrap the app in `<Providers>` inside `src/app/layout.tsx` — the scaffold writes the
component but leaves the layout to you, because layouts are where design lives.

---

## Tenancy — organizations, not users

For anything sold to a company, the tenant is the **Clerk organization**, not the user.
The scaffolded `requireTenant` reflects that:

```ts
return (identity.org_id as string) ?? (identity.subject as string);
```

Organization when there is one, personal user otherwise. This is what makes a personal
OS and a professional OS share the same schema.

**Never accept a `tenantId` argument from the client.** A client-supplied tenant is one
crafted request away from reading another customer's data. Derive it from the identity,
every time, in every function. The scaffold does this in `convex/items.ts` — keep the
pattern when you add functions.

---

## What `identity` actually contains

`ctx.auth.getUserIdentity()` returns the JWT claims, not a Clerk user object. Fields
like `email` or `name` are present only if the `convex` JWT template maps them. If you
need them in Convex, add them to the template's claims — do not fetch the user from
Clerk's API inside a query.

---

## Roles

Clerk organization roles (`org:admin`, `org:member`) arrive in the token when mapped in
the template. Authorization belongs in the Convex function, not in the component: a
hidden button is not an access control.
