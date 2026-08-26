# Convex — schema, indexes, and calling Claude

---

## The canonical shape

Every AgentikOS schema carries the same three-part spine, whatever the domain:

| Table | Role | Rule |
|---|---|---|
| **the primitive** | The object the product is about | **Always the first table.** If you hesitate between two, the design failed its phase-2 gate |
| `entries` | The signals every module writes | **Mandatory.** This is what makes cross-reading possible |
| `syntheses` | What the AI produced | **Separate from `entries`.** Never mix the observed with the interpreted |

`tenantId` goes on every table from day one if the product will ever be sold.
Retrofitting tenancy into a live database is genuinely painful and the pain is
proportional to how long you waited.

## Indexes

One index per **real** query. Not one per imaginable query.

```ts
.index("by_tenant_date", ["tenantId", "date"])
```

Field order matters: it must match the order you filter in. An index whose prefix you
never constrain is dead weight that still costs writes.

A table that appears in no flow of the design should not exist. If you cannot name the
screen that reads it, delete it.

---

## Function shape

Use the object form. Always validate args — an unvalidated mutation is a public write
endpoint.

```ts
export const create = mutation({
  args: { title: v.string() },
  handler: async (ctx, args) => { /* … */ },
});
```

| Kind | Use for | Can it write? |
|---|---|---|
| `query` | Reads. Reactive, cached | No |
| `mutation` | Writes. Transactional | Yes |
| `action` | Anything touching the outside world (fetch, Claude, Stripe) | Not directly — it calls a mutation |

Actions are **not** transactional and can run twice. Anything an action does to the
outside world should be idempotent or guarded by a record you write first.

---

## Calling Claude

```
Client (Next / Stax)
      │  never a direct model call
      ▼
Convex action  ──►  Claude  ──►  structured output
      │
      ├──► writes to `syntheses`
      └──► optionally opens panels via the action registry
```

The rules that make AI output trustworthy rather than decorative:

- **The model is called from an action, never from the client.** A client-side key is a
  published key.
- **The output is structured and validated before it is written.** Free text into a
  database is a future migration.
- **Every claim carries its citations.** `citations: v.array(v.string())` is in the
  schema for this reason.
- **Empty output is a nominal case, not an error.** "Nothing notable this week" is a
  valid, useful answer. A model forced to always find something will invent something.
- **Level-3 agents write an intention, not an effect.** Execution waits for confirmation.

Use the current model IDs from the `claude-api` skill rather than hardcoding one from
memory — model IDs are the thing that quietly goes stale.

---

## Auth inside a function

```ts
const identity = await ctx.auth.getUserIdentity();
if (!identity) throw new Error("Not authenticated");
```

Derive the tenant from the identity. Never from the arguments. See `clerk.md`.

---

## Local development

```bash
npx convex dev      # watches convex/, pushes, writes NEXT_PUBLIC_CONVEX_URL
npx convex env set KEY value
npx convex logs
```

`npx convex dev` must run alongside `npm run dev` — two terminals. A schema change is
live the moment it is saved, which is the point of Convex and also the reason a bad
index shows up immediately rather than at scale.
