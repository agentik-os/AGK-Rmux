---
description: >
  OmegaOS new-project pipeline — guiding (client/works) → stack pick → AUTO-PROVISION
  external services (Vercel/Convex/GitHub/Clerk/Stripe) from ~/.omega/provisioning →
  scaffold the full stack → wire every key → register the project → chain into
  vision/PRD/planner. Launchable from the TUI Projects menu or by typing /omega-new-project.
argument-hint: "[stack] [category] [name] [group]   e.g. nextstack client acme acme-corp   (all optional → interactive)"
allowed-tools: ["Read", "Write", "Edit", "Bash", "Glob", "Grep", "AskUserQuestion", "Task", "ToolSearch"]
---

# /omega-new-project — Provision-and-scaffold pipeline

Turns "new project" into a fully wired project: external services created and
connected, the stack scaffolded, then vision → PRD → plan. Idempotent and
resumable. Honest about what can and cannot be auto-provisioned.

`$ARGUMENTS` = `[stack] [category] [name]` (positional, all optional). When
invoked from the TUI menu these are pre-filled; when typed, any missing value is
asked interactively. Never block on a value the menu already supplied.

---

## STACKS (registry)

| id | Type | Stack |
|----|------|-------|
| `nextstack` | SaaS product | **Next.js 16 + Convex + Clerk + Stripe + shadcn-chatbot-kit** (App Router, realtime, auth, billing, full chat kit, oklch brand). Optional React Flow. |
| `nextstack-content` | Content / multi-user | Next.js 16 + Convex (docs/comments/reactions schema) + ISR + sitemap. Clerk optional. |
| `nextstack-static` | Marketing / landing / docs | Next.js 16 **static export** (`output: export`) + shadcn + landing template. NO backend. |
| `rust-cli` | CLI / daemon / internal tool | Rust + clap + tokio + serde + anyhow + Makefile + README. |
| `bun-script` | Script / tooling / DOM | Bun + TypeScript + shebang entry. |
| `expo-mobile` | Mobile iOS/Android | Expo + React Native + NativeWind + Expo Router + EAS. Clerk/Stripe optional. |

> The TUI menu reads this table (`NEW_PROJECT_STACKS` in app.rs — keep in sync).
> `$STACK` selects the PROVISION set + SCAFFOLD block. R-STACK doctrine: Rust
> internals, Bun tooling, Next.js clients.

## STACK DISPATCH (what each `$STACK` does)

Phases 2–5 below describe the **`nextstack`** reference flow. For other stacks,
adapt per this matrix (`—` = skip the step):

| $STACK | Provision (Ph2) | Scaffold (Ph3) | Env (Ph4) | Chain (Ph5) | Verify |
|--------|-----------------|----------------|-----------|-------------|--------|
| `nextstack` | Vercel+Convex+Clerk+Stripe+GitHub | create-next-app + shadcn-chatbot-kit + Convex + Clerk + Stripe | .env.local + push Vercel | /vision→/prd→/planner | `npm run build` |
| `nextstack-content` | Vercel+Convex+GitHub (Clerk opt) | create-next-app + shadcn subset + Convex (docs schema) + ISR + sitemap | .env.local + push Vercel | /vision→/prd→/planner | `npm run build` |
| `nextstack-static` | Vercel+GitHub | create-next-app (`output: export`) + shadcn + landing + robots/sitemap; **no** Convex/Clerk/Stripe | .env.example only | /vision (opt); skip /prd /planner | `npm run build` |
| `rust-cli` | GitHub only | `cargo new <name>` + clap/tokio/serde/anyhow + main.rs skeleton + Makefile + README | — (self-contained) | skip /vision; document in README | `cargo build --release` |
| `bun-script` | GitHub only | `bun init` + TypeScript + shebang + bunfig | — | skip /vision; document in README | runs clean |
| `expo-mobile` | GitHub+EAS (Clerk/Stripe opt) | create-expo-app + Expo Router + NativeWind + eas.json | .env.local for API keys (opt) | /vision→/prd→/planner (if product) | `eas build` configured |

Provision honesty (Law L4): a stack needing a service whose token is blank →
PAUSE with the manual link, never silently skip.

---

## PHASE 0 — Resolve inputs (no redundant questions)

1. Parse `$ARGUMENTS`: `STACK`, `CATEGORY`, `NAME`, `GROUP` (positional; `GROUP`
   defaults to `default`). For **client** projects `GROUP` is the client's own
   credential set (its own Vercel/Convex/Clerk/Stripe/GitHub accounts) — the TUI
   wizard sets it. Export for Phase 1: `GROUP="${4:-default}"`.
2. **Stack**: if missing or unknown → AskUserQuestion listing the table above.
   Any of the 6 ids is valid; `$STACK` drives provisioning + scaffold below.
3. **Category** (the guiding branch) — if missing, AskUserQuestion: `works`
   (personal/internal) or `client` (client work). Picks the **git identity** (Phase 4).
4. **Resolve the project root from config — NEVER hardcode `~/VibeCoding`** (that
   is only the maintainer's layout; this must work for ANY user):
   ```bash
   PROJECTS_DIR="$(grep -E '^projects_dir' "$HOME/.omega/config.toml" 2>/dev/null | sed 's/.*= *//; s/"//g')"
   if [ -z "$PROJECTS_DIR" ]; then
     for c in VibeCoding projects Projects code dev work; do
       [ -d "$HOME/$c" ] && PROJECTS_DIR="$HOME/$c" && break
     done
   fi
   [ -z "$PROJECTS_DIR" ] && PROJECTS_DIR="$HOME/projects"
   case "$CATEGORY" in
     client|clients) SUB=clients ;;
     personal|life|1-life) SUB=1-life ;;
     *) SUB=work ;;
   esac
   PROJECT_DIR="$PROJECTS_DIR/$SUB/$NAME"
   ```
5. **Name**: if missing → ask. Validate: lowercase, `[a-z0-9-]`, not already a
   dir at `$PROJECT_DIR`, not already in `~/.omega/projects.json`.
6. **Kickoff brief**: if the invocation prompt contains a `--- PROJECT KICKOFF
   BRIEF ---` and/or `--- REFERENCED DOCS ---` block (the TUI wizard appends them
   when the user gives an idea / docs), treat it as the seed — fold it into the
   VISION/PRD (Phase 5) and let it steer scaffold choices. Absent → proceed.
7. **React Flow** (Next.js stacks only): ask "Inclure React Flow ?" →
   `WANT_REACTFLOW`. Skip for rust-cli/bun-script/expo-mobile.
8. Echo the resolved plan in 3 lines and proceed (Law L3 — no gate when the menu
   supplied everything).

---

## PHASE 1 — Load provisioning credentials

```bash
# $GROUP (4th arg, default "default") selects the credential set. CLIENT projects
# get their OWN accounts via groups/<group>.env; "default" = the shared services.env.
GROUP="${GROUP:-default}"
if [ "$GROUP" = "default" ] || [ -z "$GROUP" ]; then
  PROV="$HOME/.omega/provisioning/services.env"
else
  PROV="$HOME/.omega/provisioning/groups/${GROUP}.env"
fi
if [[ -f "$PROV" ]]; then set -a; source "$PROV"; set +a; else
  echo "No $PROV — credential group '$GROUP' is empty; provisioning runs in MANUAL/PAUSE mode."
  echo "Fill it with:  omega provision set $GROUP VERCEL_TOKEN=... CONVEX_TEAM_TOKEN=... ..."
fi
```

> **Guided token entry:** the user fills these tokens without editing any file —
> in the OmegaOS TUI, **Monitor tab → "Set up project provisioning keys" (P)** runs
> a step-by-step wizard (Vercel → Convex → GitHub → Stripe) that writes them safely
> to `services.env` (chmod 600, blanks skipped). If tokens are missing, point the
> user there before falling back to PAUSE mode.

Build a capability map (each ON only if its token is present):

- `CAN_VERCEL`  = `[ -n "$VERCEL_TOKEN" ]`
- `CAN_CONVEX`  = `[ -n "$CONVEX_TEAM_TOKEN" ]`
- `CAN_GITHUB`  = `[ -n "$GITHUB_TOKEN" ]` OR `gh auth status` succeeds
- `CAN_STRIPE`  = `[ -n "$STRIPE_SECRET_KEY" ]`
- Clerk uses `CLERK_PROVISION_MODE` (pool|pause), never a create-API.

Print the capability map so the user sees exactly what will be automated vs
paused. **A blank token is a PAUSE, never a silent skip** (Law L4 / L5).

---

## PHASE 2 — Provision external services

Run each provisioner; record results into `$PROJECT_DIR/.omega-provision.json`
(create the dir first). Each is **idempotent** — re-running detects existing
resources and reuses them. On any hard failure, write the partial state and
continue with the rest (never abort the whole pipeline over one service); list
blockers at the end (Law L4).

### 2a. GitHub repo
- `CAN_GITHUB`: `gh repo create <owner>/<name> --private --source="$PROJECT_DIR" --remote=origin`
  (owner = `$GITHUB_OWNER` or `gh api user -q .login`). If `$GITHUB_TOKEN` set,
  export `GH_TOKEN="$GITHUB_TOKEN"` for the call.
- Else PAUSE: tell the user to `gh auth login`, then re-run (resumable).

### 2b. Vercel project (FULL auto)
- `CAN_VERCEL`: create the project via API and capture its id.
  ```bash
  TEAM_Q=${VERCEL_TEAM_ID:+?teamId=$VERCEL_TEAM_ID}
  curl -s -X POST "https://api.vercel.com/v11/projects$TEAM_Q" \
    -H "Authorization: Bearer $VERCEL_TOKEN" -H "Content-Type: application/json" \
    -d "{\"name\":\"<name>\",\"framework\":\"nextjs\"}"
  ```
  Link the local dir: `vercel link --yes --project <name> --token "$VERCEL_TOKEN" ${VERCEL_TEAM_ID:+--scope $VERCEL_TEAM_ID}` in `$PROJECT_DIR`.
  Env vars are pushed in Phase 3 once their values exist.
- Else PAUSE.

### 2c. Convex deployment (FULL auto)
- `CAN_CONVEX`: provision a dev deployment non-interactively. Run inside
  `$PROJECT_DIR` after `npm i convex`:
  ```bash
  CONVEX_AGENT_MODE=anonymous npx convex dev --once --configure=new \
    --team "$CONVEX_TEAM_SLUG" --project "<name>" 2>&1 || true
  # capture CONVEX_DEPLOYMENT + NEXT_PUBLIC_CONVEX_URL from .env.local convex wrote
  ```
  Use `$CONVEX_TEAM_TOKEN` via `CONVEX_DEPLOY_KEY` env if the non-interactive
  team flag is unavailable in the installed CLI version — verify the CLI's flags
  first (`npx convex dev --help`), never assume (Law L1).
- Else PAUSE: `npx convex dev` interactive.

### 2d. Clerk (NO create-API → pool or pause)
- `CLERK_PROVISION_MODE=pool`: pop the first unused line of
  `~/.omega/provisioning/clerk-pool.env` (`pk|sk|label`), mark it `# USED <name>`,
  capture `CLERK_PUBLISHABLE` + `CLERK_SECRET`. If the pool is empty → fall
  through to pause.
- `pause` (default): open `https://dashboard.clerk.com`, instruct the user to
  create an app + copy the **Publishable** and **Secret** keys, then collect them
  with AskUserQuestion (free-text). Persist immediately so a re-run resumes.
- State the reason plainly: "Clerk has no public app-creation API — this step is
  pool/pause by necessity, not by shortcut."

### 2e. Stripe
- `STRIPE_MODE=single` + `CAN_STRIPE`: reuse the master account. Create a
  restricted key + a webhook endpoint pointing at the (future) Vercel URL +
  starter product/price via the API:
  ```bash
  curl -s https://api.stripe.com/v1/products -u "$STRIPE_SECRET_KEY:" -d name="<name>"
  curl -s https://api.stripe.com/v1/webhook_endpoints -u "$STRIPE_SECRET_KEY:" \
    -d url="https://<name>.vercel.app/api/stripe/webhook" -d "enabled_events[]=checkout.session.completed"
  ```
  Capture the webhook signing secret (`whsec_…`) and a restricted key for the app.
- `STRIPE_MODE=connect` + `CAN_STRIPE`: `POST /v1/accounts` (type=standard),
  capture the connected `acct_…`, generate an onboarding link
  (`/v1/account_links`), and **tell the user KYC onboarding is required before
  live charges** — surface the link, don't pretend it's done.
- Else PAUSE.

---

## PHASE 3 — Scaffold the stack (`nextstack`)

Fetch latest versions first (don't hardcode): if Context7 MCP is available,
`ToolSearch("select:mcp__context7__resolve-library-id")` then query `next`,
`convex`, `@clerk/nextjs`, `stripe`, `tailwindcss`. Otherwise use `@latest`.

```bash
cd "$(dirname "$PROJECT_DIR")"
npx create-next-app@latest "<name>" --ts --app --tailwind --eslint --src-dir --import-alias "@/*" --use-npm --yes
cd "$PROJECT_DIR"
npx shadcn@latest init -d
# Full shadcn chatbot kit — EVERY chat component (Law L5: all of it, not a subset)
npx shadcn@latest add "https://shadcn-chatbot-kit.vercel.app/r/chat.json" -y || \
  npx shadcn@latest add chat message prompt-input markdown -y
npm i convex @clerk/nextjs stripe @stripe/stripe-js
[ "$WANT_REACTFLOW" = yes ] && npm i @xyflow/react
```

Then create:
- `src/app/providers.tsx` — ClerkProvider + ConvexProviderWithClerk wired together.
- `src/middleware.ts` — Clerk middleware.
- `convex/` schema stub + `convex/auth.config.ts` keyed to the Clerk issuer.
- `src/app/api/stripe/webhook/route.ts` — signature-verified handler.
- A `/chat` route mounting the chatbot-kit components end-to-end.
- If `WANT_REACTFLOW`: a `/flow` route with a minimal React Flow canvas.
- **Brand system**: `src/app/globals.css` oklch tokens (full light/dark scale,
  radii, shadows, typography) + `BRAND.md`. Pull a coherent palette; this is the
  "ultra complete" brand foundation the user asked for, not 3 stray variables.

---

## PHASE 4 — Wire keys + register

1. Write `$PROJECT_DIR/.env.local` from the provisioned values:
   ```
   NEXT_PUBLIC_CONVEX_URL=…          CONVEX_DEPLOYMENT=…
   NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY=…   CLERK_SECRET_KEY=…
   STRIPE_SECRET_KEY=…(restricted)   STRIPE_WEBHOOK_SECRET=whsec_…   NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY=…
   ```
   Also write `.env.example` with the same keys blanked (committed).
2. Push the non-public vars to Vercel (`CAN_VERCEL`): `vercel env add <KEY> production` via API/CLI with `--token`.
3. **Git identity by category** (Law R-FICHE): set `git config user.email` —
   client → that client's email; works/AgentikOS → `x@agentik-os.com`. Commit
   the scaffold. Push to origin if the repo was created.
4. **Register** in `~/.omega/projects.json` (append an entry: name, path,
   category, created date) so it shows up in the TUI Projects tab immediately.
5. Verify (Law L1): `npm run build` must pass before declaring scaffold done.

---

## PHASE 5 — Chain into the product pipeline

Do not re-implement vision/PRD/planning — delegate to the proven commands, in
order, scoped to `$PROJECT_DIR`:

1. `/vision` — emotional positioning → `VISION.md`
2. `/prd` — full doc suite from the vision
3. `/planner` — task tracker (`.planner/tracker.json`)

Offer to continue straight into `/build` (execute) or stop here so the user can
review the vision first. In a dispatched (non-interactive) context, proceed to
`/vision` automatically (Law L3).

---

## DONE CRITERIA (grade against this, not vibes — R-RUBRIC)

- [ ] Project dir created under the **correct category** path.
- [ ] Every service either **provisioned** (resource id captured) or **explicitly
      paused** with a recorded reason — none silently skipped.
- [ ] `.env.local` wired; `.env.example` committed; secrets only in
      `$PROJECT_DIR/.env.local` + `~/.omega`, never staged for git.
- [ ] `npm run build` passes.
- [ ] Project registered in `~/.omega/projects.json` (visible in the TUI).
- [ ] `VISION.md` started (pipeline handed off).
- [ ] A `--- Resume:` one-line French recap (R-STYLE).

Report a checklist of provisioned vs paused services + the next command.
