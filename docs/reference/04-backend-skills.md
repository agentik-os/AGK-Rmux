# Skills Backend & Database

> Convex, Stripe, Auth, et création de serveurs MCP.

---

## Vue d'ensemble

| Skill | Usage |
|-------|-------|
| `convex-best-practices` | Patterns Convex production-ready |
| `convex-realtime` | Subscriptions, optimistic updates |
| `stripe-best-practices` | Intégrations Stripe |
| `better-auth-best-practices` | Auth TypeScript |
| `mcp-builder` | Création serveurs MCP |

---

## 1. convex-best-practices

**Source:** `~/.agents/skills/convex-best-practices/`

### The Zen of Convex

1. **Convex manages the hard parts** - Caching, real-time sync, consistency
2. **Functions are the API** - Design functions as interface
3. **Schema is truth** - Définir explicitement dans schema.ts
4. **TypeScript everywhere** - Type safety end-to-end
5. **Queries are reactive** - Penser subscriptions, pas requests

### Organisation des fonctions

```typescript
// convex/users.ts - Fonctions par domaine
import { query, mutation } from "./_generated/server";
import { v } from "convex/values";

export const get = query({
  args: { userId: v.id("users") },
  returns: v.union(v.object({
    _id: v.id("users"),
    _creationTime: v.number(),
    name: v.string(),
    email: v.string(),
  }), v.null()),
  handler: async (ctx, args) => {
    return await ctx.db.get(args.userId);
  },
});
```

### Validation args ET returns

```typescript
export const createTask = mutation({
  args: {
    title: v.string(),
    priority: v.union(v.literal("low"), v.literal("medium"), v.literal("high")),
  },
  returns: v.id("tasks"),  // TOUJOURS définir returns
  handler: async (ctx, args) => {
    return await ctx.db.insert("tasks", {
      title: args.title,
      priority: args.priority,
      completed: false,
    });
  },
});
```

### Utiliser les indexes

```typescript
// Schema avec index
export default defineSchema({
  tasks: defineTable({
    userId: v.id("users"),
    status: v.string(),
  })
    .index("by_user", ["userId"])
    .index("by_user_and_status", ["userId", "status"]),
});

// Query avec index
export const getTasksByUser = query({
  args: { userId: v.id("users") },
  handler: async (ctx, args) => {
    return await ctx.db
      .query("tasks")
      .withIndex("by_user", (q) => q.eq("userId", args.userId))
      .order("desc")
      .collect();
  },
});
```

### Error handling avec ConvexError

```typescript
import { ConvexError } from "convex/values";

export const updateTask = mutation({
  args: { taskId: v.id("tasks"), title: v.string() },
  handler: async (ctx, args) => {
    const task = await ctx.db.get(args.taskId);

    if (!task) {
      throw new ConvexError({
        code: "NOT_FOUND",
        message: "Task not found",
      });
    }

    await ctx.db.patch(args.taskId, { title: args.title });
  },
});
```

### Mutations idempotentes

```typescript
// GOOD: Check état avant modification
export const completeTask = mutation({
  args: { taskId: v.id("tasks") },
  handler: async (ctx, args) => {
    const task = await ctx.db.get(args.taskId);

    // Early return si déjà complete (idempotent)
    if (!task || task.status === "completed") {
      return null;
    }

    await ctx.db.patch(args.taskId, {
      status: "completed",
      completedAt: Date.now(),
    });
  },
});
```

### Internal vs Public

```typescript
// Public - exposée aux clients
export const getUser = query({ ... });

// Internal - seulement depuis autres fonctions Convex
export const _updateUserStats = internalMutation({ ... });
```

### Best practices

- JAMAIS `npx convex deploy` sans instruction explicite
- Toujours définir `returns` validator
- Utiliser indexes pour tous les queries qui filtrent
- Mutations idempotentes pour gérer les retries
- Organiser par domaine (users.ts, tasks.ts, etc.)

---

## 2. convex-realtime

**Source:** `~/.agents/skills/convex-realtime/`

### Topics couverts

- Gestion des subscriptions
- Optimistic updates
- Cache behavior
- Paginated queries avec cursors

### Subscriptions React

```typescript
import { useQuery } from "convex/react";
import { api } from "../convex/_generated/api";

function TaskList() {
  // Automatiquement réactif!
  const tasks = useQuery(api.tasks.list);

  if (tasks === undefined) return <Loading />;

  return tasks.map(task => <Task key={task._id} {...task} />);
}
```

### Optimistic Updates

```typescript
const addTask = useMutation(api.tasks.create)
  .withOptimisticUpdate((localStore, args) => {
    const tasks = localStore.getQuery(api.tasks.list);
    if (tasks !== undefined) {
      localStore.setQuery(api.tasks.list, undefined, [
        ...tasks,
        { _id: "temp", title: args.title, completed: false },
      ]);
    }
  });
```

---

## 3. stripe-best-practices

**Source:** `~/.agents/skills/stripe-best-practices/`

### Topics couverts

- Checkout Sessions
- Subscriptions
- Customer Portal
- Webhooks
- Prix et produits
- Coupons et promotions

### Pattern webhook type

```typescript
// app/api/webhooks/stripe/route.ts
import Stripe from "stripe";

export async function POST(req: Request) {
  const body = await req.text();
  const sig = req.headers.get("stripe-signature")!;

  let event: Stripe.Event;

  try {
    event = stripe.webhooks.constructEvent(
      body,
      sig,
      process.env.STRIPE_WEBHOOK_SECRET!
    );
  } catch (err) {
    return new Response("Webhook Error", { status: 400 });
  }

  switch (event.type) {
    case "checkout.session.completed":
      // Handle successful checkout
      break;
    case "customer.subscription.updated":
      // Handle subscription change
      break;
    case "invoice.payment_failed":
      // Handle failed payment
      break;
  }

  return new Response("OK", { status: 200 });
}
```

---

## 4. better-auth-best-practices

**Source:** `~/.agents/skills/better-auth-best-practices/`

### Qu'est-ce que Better Auth?

Framework d'authentification TypeScript complet.

### Features

- User management
- Organizations
- Webhooks
- Middleware configuration
- Next.js integration

---

## 5. mcp-builder

**Source:** `~/.agents/skills/mcp-builder/`

### Vue d'ensemble

Créer des serveurs MCP (Model Context Protocol) pour permettre aux LLMs d'interagir avec des services externes.

### Stack recommandé

- **Language**: TypeScript (meilleur support SDK)
- **Transport**: Streamable HTTP (serveurs remote), stdio (local)

### Workflow en 4 phases

#### Phase 1: Research and Planning

1. Comprendre le design MCP moderne
2. Étudier la spec MCP: `https://modelcontextprotocol.io/sitemap.xml`
3. Charger la doc SDK:
   - TypeScript: `https://raw.githubusercontent.com/modelcontextprotocol/typescript-sdk/main/README.md`
   - Python: `https://raw.githubusercontent.com/modelcontextprotocol/python-sdk/main/README.md`

#### Phase 2: Implementation

1. Setup structure projet
2. Créer infrastructure partagée:
   - API client avec auth
   - Error handling helpers
   - Pagination support

3. Implémenter tools avec:
   - Input schema (Zod ou Pydantic)
   - Output schema
   - Description claire
   - Async/await pour I/O
   - Annotations (readOnlyHint, destructiveHint, etc.)

#### Phase 3: Review and Test

```bash
# TypeScript
npm run build
npx @modelcontextprotocol/inspector

# Python
python -m py_compile your_server.py
```

#### Phase 4: Create Evaluations

Créer 10 questions d'évaluation complexes pour tester l'efficacité.

### Structure d'un tool

```typescript
server.registerTool({
  name: "github_create_issue",
  description: "Create a new issue in a GitHub repository",
  inputSchema: z.object({
    repo: z.string().describe("Repository name (owner/repo)"),
    title: z.string().describe("Issue title"),
    body: z.string().optional().describe("Issue body content"),
  }),
  outputSchema: z.object({
    number: z.number(),
    url: z.string(),
  }),
  annotations: {
    readOnlyHint: false,
    destructiveHint: false,
    idempotentHint: false,
  },
  handler: async (args) => {
    const result = await github.createIssue(args);
    return {
      content: [{ type: "text", text: JSON.stringify(result) }],
      structuredContent: result,
    };
  },
});
```

### Références

| Resource | URL |
|----------|-----|
| MCP Spec | https://modelcontextprotocol.io |
| TypeScript SDK | https://github.com/modelcontextprotocol/typescript-sdk |
| Python SDK | https://github.com/modelcontextprotocol/python-sdk |

---

## Documentation officielle

| Service | URL |
|---------|-----|
| Convex | https://docs.convex.dev |
| Stripe | https://stripe.com/docs |
| Better Auth | https://better-auth.com |
| MCP | https://modelcontextprotocol.io |

---

*Dernière mise à jour: 2026-01-27*
