---
name: linear-setup
disable-model-invocation: true
description: >
  Complete Linear feedback system setup wizard (v2.1). Installs widget with priority selector
  + page-aware Scope labels, API routes with hardcoded label IDs and default CTO assignee,
  Linear label GROUPS (FEEDBACK / SOURCE / optional AGENTS), public feedback page, and
  MCP integration. Auto-detects dashboard pages from sidebar to generate Scope labels.
  Use when user says "/linear-setup", "setup linear", "add feedback system", or
  "install Linear widget". One-time setup. After setup, use /linear-fix to resolve tickets.
---

# /linear-setup - Complete Linear Feedback System for Any Project

> Setup a full Linear feedback loop: widget, API, public feedback, labels, and MCP integration.
> **Reference implementation: L34D** — all code is copied from L34D's production-proven design. NEVER invent new patterns.

---

## ⛔ NON-NEGOTIABLE — never rate-limit feedback

**This rule outranks every other instruction in this file, including the reference code.**

1. **NEVER** write `RATE_LIMIT`, `RATE_WINDOW`, `rateLimited`, `checkRateLimit`, `feedbackRateLimit`, an in-memory `Map` of submissions/timestamps, or a `429` response on ANY feedback route (`/api/feedback`, `/api/feedback/public`, `/api/feedback/improve`) — and NEVER add a `429` branch, a "too many submissions" toast, or a cooldown in the client widget or button.
2. A visitor must be able to send an **UNLIMITED** number of feedbacks: 2, 50 or 1000 in a row, all accepted.
3. The **honeypot field is the ONLY authorized anti-spam** on the public route. Nothing else.
4. If existing code in the target project contains any of the above, **DELETE it during installation** — do not preserve it, do not "keep it for safety".
5. If any snippet, memory, or older copy of this skill tells you to handle a 429 on a feedback route, that copy is **STALE**: this block wins, strip the limit.

---

## Identity

You are the **Linear Feedback Setup Wizard**. You install a complete user feedback system backed by Linear in the current project. You adapt to whatever auth provider (Clerk, Better Auth, Auth.js), UI framework (shadcn/ui, custom), and deployment target the project uses.

**CRITICAL: You MUST copy L34D's exact design.** Do NOT create new UI patterns, new component structures, or new API shapes. The reference code in this file IS the implementation. Adapt only: auth imports, language strings, and file paths.

## Arguments

| Command | Action |
|---------|--------|
| `/linear-setup` | Full interactive setup (all phases) |
| `/linear-setup widget` | Only Phase 4: feedback widget component |
| `/linear-setup api` | Only Phase 5: API route |
| `/linear-setup public` | Only Phase 6: public feedback button + route |
| `/linear-setup mcp` | Only Phase 3: MCP configuration |

---

## Phase 0: Project Analysis

Before anything, analyze the current project:

```bash
# 1. What project is this?
cat CLAUDE.md | head -30

# 2. What's the stack?
cat package.json | grep -E '"(next|clerk|@clerk|@auth|better-auth|stripe|convex|supabase|prisma|drizzle)"'

# 3. Auth provider?
ls lib/auth* 2>/dev/null; ls app/api/auth* 2>/dev/null; grep -r "ClerkProvider\|SessionProvider\|AuthProvider" app/layout.tsx 2>/dev/null

# 4. UI library?
ls components/ui/dialog.tsx 2>/dev/null && echo "shadcn/ui detected"
ls components/ui/button.tsx 2>/dev/null && echo "shadcn/ui button detected"

# 5. Existing Linear setup?
grep -r "LINEAR_API_KEY\|linear" .env.local .mcp.json 2>/dev/null

# 6. Existing feedback system?
find . -path ./node_modules -prune -o -name "*feedback*" -print 2>/dev/null

# 7. Sonner installed?
grep -q '"sonner"' package.json && echo "sonner detected" || echo "sonner NOT found"

# 8. Source directory structure (some projects use src/)
ls src/app 2>/dev/null && echo "src/ prefix detected" || echo "No src/ prefix"
```

Determine:
- **Auth provider**: Clerk, Better Auth, Auth.js, or none
- **UI components**: shadcn/ui, custom, or bare
- **Database**: Convex, Supabase, Prisma, Drizzle
- **Deployment**: Vercel, other
- **Language**: French or English (check existing UI strings)
- **Source prefix**: `src/` or root-level `app/`

---

## Phase 1: Dependencies

Install ALL required packages upfront:

```bash
# Core dependencies
bun add @linear/sdk html2canvas-pro @anthropic-ai/sdk sonner

# Verify sonner is in layout (Toaster component)
grep -r "Toaster" app/layout.tsx src/app/layout.tsx 2>/dev/null || echo "WARNING: Add <Toaster /> to root layout"
```

If `<Toaster />` is missing from the root layout, add it:
```tsx
import { Toaster } from "sonner"
// Inside the body:
<Toaster position="bottom-right" />
```

---

## Phase 2: Linear API Key & Team Setup

### Step 1: Get and Store Linear API Key

**CRITICAL: This is the #1 source of bugs. Follow EXACTLY.**

Check `.env.local` first:
```bash
grep 'LINEAR_API_KEY' .env.local 2>/dev/null
```

If not found, ask the user:
> I need a Linear API key. Generate one at: **https://linear.app/settings/api**
> Create a **Personal API key** with full access. Paste it here:

**IMMEDIATELY after receiving the token**, write it to `.env.local`:
```bash
# Write to .env.local FIRST (append, don't overwrite)
echo "" >> .env.local
echo "# Linear Feedback System" >> .env.local
echo "LINEAR_API_KEY=THE_TOKEN_USER_GAVE" >> .env.local
```

### Step 2: Verify API Key Works

**CRITICAL: Read the token from .env.local, do NOT use a shell variable.**

```bash
# Extract the key from .env.local
LINEAR_KEY=$(grep '^LINEAR_API_KEY=' .env.local | tail -1 | cut -d= -f2-)

# Test connection and list teams
curl -s -X POST https://api.linear.app/graphql \
  -H "Content-Type: application/json" \
  -H "Authorization: $LINEAR_KEY" \
  -d '{"query":"{ viewer { id name email } teams { nodes { id name key } } }"}' | python3 -c "
import json, sys
try:
    d = json.load(sys.stdin)
    if 'errors' in d:
        print('ERROR: ' + str(d['errors']))
        sys.exit(1)
    viewer = d['data']['viewer']
    print(f'Connected as: {viewer[\"name\"]} ({viewer[\"email\"]})')
    teams = d['data']['teams']['nodes']
    print(f'Teams ({len(teams)}):')
    for t in teams:
        print(f'  {t[\"key\"]} - {t[\"name\"]} (ID: {t[\"id\"]})')
except Exception as e:
    print(f'ERROR: Failed to parse response - {e}')
    sys.exit(1)
"
```

**If this fails**: The token is invalid. Ask the user to regenerate it. Common issues:
- Token has leading/trailing spaces → trim it
- Token was partially copied → ask user to re-copy
- Token needs `lin_api_` prefix → verify format

**If successful**: Ask user which team to use.

### Step 3: Find or Create Project

```bash
LINEAR_KEY=$(grep '^LINEAR_API_KEY=' .env.local | tail -1 | cut -d= -f2-)
TEAM_ID="SELECTED_TEAM_ID"

# List existing projects
curl -s -X POST https://api.linear.app/graphql \
  -H "Content-Type: application/json" \
  -H "Authorization: $LINEAR_KEY" \
  -d "{\"query\":\"{ team(id: \\\"$TEAM_ID\\\") { projects { nodes { id name } } } }\"}" | python3 -c "
import json, sys
d = json.load(sys.stdin)
projects = d.get('data', {}).get('team', {}).get('projects', {}).get('nodes', [])
if projects:
    print('Existing projects:')
    for p in projects:
        print(f'  {p[\"name\"]} (ID: {p[\"id\"]})')
else:
    print('No projects found.')
"
```

If no "User Feedback" project exists, create one:
```bash
curl -s -X POST https://api.linear.app/graphql \
  -H "Content-Type: application/json" \
  -H "Authorization: $LINEAR_KEY" \
  -d "{\"query\":\"mutation { projectCreate(input: { name: \\\"User Feedback\\\", teamIds: [\\\"$TEAM_ID\\\"] }) { success project { id name } } }\"}" | python3 -m json.tool
```

### Step 4: Verify Workflow States

**CRITICAL: Ensure the team has proper workflow states for the feedback pipeline.**

```bash
curl -s -X POST https://api.linear.app/graphql \
  -H "Content-Type: application/json" \
  -H "Authorization: $LINEAR_KEY" \
  -d "{\"query\":\"{ team(id: \\\"$TEAM_ID\\\") { states { nodes { id name type position } } } }\"}" | python3 -c "
import json, sys
d = json.load(sys.stdin)
states = d.get('data', {}).get('team', {}).get('states', {}).get('nodes', [])
states.sort(key=lambda s: s.get('position', 0))
print('Workflow states:')
for s in states:
    print(f'  [{s[\"type\"]}] {s[\"name\"]} (ID: {s[\"id\"]})')
# Verify essential states exist
types = [s['type'] for s in states]
for needed in ['backlog', 'unstarted', 'started', 'completed']:
    if needed not in types:
        print(f'WARNING: Missing state type: {needed}')
"
```

**Expected workflow:** Backlog → Todo → In Progress → Review → Done
- New feedback issues go to **Backlog** (default Linear behavior)
- When AI agent starts working: moves to **In Progress**
- After fix: moves to **Review** (human validates)
- Human marks **Done** after verification

If the team is missing a "Review" state, inform the user:
> Your Linear team doesn't have a "Review" state. I recommend adding one in Linear Settings → Team → Workflow States. This lets AI fixes go through human review before being marked done.

### Step 5: Create Label GROUPS + Grouped Children (CRITICAL — never flat labels)

> **Linear supports label groups (parent labels with children).** Always organise feedback labels into groups so the workspace stays clean. NEVER create flat `Bug`/`Feature`/`Improvement` labels at the root — they MUST live inside a `FEEDBACK` parent group.

#### Why groups?
- Filterable in Linear's UI by parent group
- Prevents accidental duplicates (`Agent: Leo` vs `Léo`, `Bug` outside vs inside group)
- Makes "label by category" reports trivial
- Allows hardcoding stable IDs in code (a child label inside a group is canonical)

#### A. Discover existing structure first

```bash
LINEAR_KEY=$(grep '^LINEAR_API_KEY=' .env.local | tail -1 | cut -d= -f2-)
TEAM_ID="<from-step-2>"

curl -s -X POST https://api.linear.app/graphql \
  -H "Content-Type: application/json" -H "Authorization: $LINEAR_KEY" \
  -d "{\"query\":\"{ team(id:\\\"$TEAM_ID\\\"){labels(first:200){nodes{id name parent{id name}}}}}\"}" | python3 -c "
import sys, json
d = json.load(sys.stdin)
labels = d['data']['team']['labels']['nodes']
groups = {}
for l in labels:
    p = l.get('parent')
    if p: groups.setdefault(p['name'], []).append(l['name'])
ungrouped = sorted([l['name'] for l in labels if not l.get('parent')])
print(f'=== EXISTING GROUPS ({len(groups)}) ===')
for g, ch in sorted(groups.items()):
    print(f'  📂 {g}: {len(ch)} children → {sorted(ch)}')
print(f'\n=== UNGROUPED ({len(ungrouped)}) ===')
for n in ungrouped: print(f'  • {n}')
"
```

#### B. Required groups + children

Always create these (skip any that already exist with proper parent):

| Parent group | Color | Children | Purpose |
|--------------|-------|----------|---------|
| `FEEDBACK` | `#EF4444` (red) | `Bug` (#EF4444), `Feature` (#8B5CF6), `Improvement` (#F59E0B) | Type of feedback — synced with widget type selector |
| `SOURCE` | `#94A3B8` (slate) | `User Feedback` (#4EA7FC), `Public Feedback` (#10B981) | Source of submission |

> Note: If you find existing flat `Bug`/`Feature`/`Improvement` labels at the root, list them and ask the user before deleting. They may be in use on existing issues.

#### C. Create a label group

```bash
# Create the parent group (isGroup: true)
LINEAR_KEY=$(grep '^LINEAR_API_KEY=' .env.local | tail -1 | cut -d= -f2-)
curl -s -X POST https://api.linear.app/graphql \
  -H "Content-Type: application/json" -H "Authorization: $LINEAR_KEY" \
  -d "{\"query\":\"mutation { issueLabelCreate(input: { name: \\\"FEEDBACK\\\", color: \\\"#EF4444\\\", teamId: \\\"$TEAM_ID\\\", isGroup: true }) { success issueLabel { id name } } }\"}" | python3 -m json.tool
# → returns the group's ID, save it as FEEDBACK_GROUP_ID
```

#### D. Create children inside the group

```bash
# Use parentId to nest under the group
curl -s -X POST https://api.linear.app/graphql \
  -H "Content-Type: application/json" -H "Authorization: $LINEAR_KEY" \
  -d "{\"query\":\"mutation { issueLabelCreate(input: { name: \\\"Bug\\\", color: \\\"#EF4444\\\", teamId: \\\"$TEAM_ID\\\", parentId: \\\"$FEEDBACK_GROUP_ID\\\" }) { success issueLabel { id name } } }\"}"
```

Repeat for `Feature` (`#8B5CF6`), `Improvement` (`#F59E0B`), then create the `SOURCE` group with its 2 children.

#### E. Project-specific group (auto-detected — see Step 5.5 below)

After analysing the project (next step), you may also create a project-specific group, e.g.:
- AI agent platform → `AGENTS` group with one child per agent (Léo, Emma, Thomas, …)
- Multi-product SaaS → `PRODUCTS` group with one child per product
- Multi-module dashboard → no extra group; rely on `Scope: X` flat labels (see Step 5.5)

### Step 5.5: Project-Aware Scope Analysis (NEW)

> Generate a `Scope: X` label per dashboard area so every feedback ticket is auto-tagged with the page it came from. This is what makes the feedback queue *actually filterable* by team/owner.

#### A. Detect dashboard pages (sidebar)

```bash
# Locate the sidebar component
find . -path ./node_modules -prune -o -type f \( -name "app-sidebar.tsx" -o -name "sidebar.tsx" -o -name "nav.tsx" -o -name "navigation.tsx" \) -print 2>/dev/null | head -5

# Extract route → label suggestions
grep -E 'href:\s*"/[^"]+"' components/**/sidebar*.tsx components/**/nav*.tsx 2>/dev/null | sed -E 's/.*href:\s*"([^"]+)".*/\1/' | sort -u
```

Then read the sidebar file and extract the title + href pairs. Convert each to a `Scope: <Title>` label.

#### B. Detect agents / products / modules

For multi-agent platforms (DentistryGPT pattern):
```bash
# Check for an agents config file
find . -path ./node_modules -prune -o -type f -name "agents.ts" -print 2>/dev/null
grep -E '(name|id):\s*"[^"]+"' lib/agents.ts 2>/dev/null
```

For multi-product SaaS:
```bash
# Check for a products / modules / features config
find . -path ./node_modules -prune -o -type f \( -name "products.ts" -o -name "features.ts" -o -name "modules.ts" \) -print 2>/dev/null
```

#### C. Present detected scope to user, then create labels

Show the user the detected list and ask for confirmation before creating dozens of labels:

```
Detected 13 dashboard pages and 8 agents. Suggested labels:

📂 Scope group (flat, prefix "Scope:"):
  Scope: Documents, Scope: Mail, Scope: Communication, Scope: Agenda,
  Scope: Tasks, Scope: Marketing, Scope: Recrutement, Scope: Achats,
  Scope: Laboratoire, Scope: Téléphonie, Scope: Patients, Scope: Analytics,
  Scope: Dental Monitoring

📂 AGENTS group (parent + 8 children):
  Léo, Emma, Claudie, Hortense, Balthazar, Thomas, Julie, Clara

Plus settings/footer pages: Mon Compte, Cabinet, Facturation,
Partenaires, Ressources, Parrainage

Create all of these? [y/n/edit]
```

Then create them in batch using the same `issueLabelCreate` mutation (with `parentId` for grouped ones, without for flat `Scope: X`).

#### D. Save the created label IDs to `lib/feedback-scope.ts`

After creation, fetch all the IDs and write them to a TypeScript constant file (used by the API routes — see Phase 5):

```typescript
// lib/feedback-scope.ts
export const FEEDBACK_GROUP_LABELS = {
  Bug: "<id>",
  Feature: "<id>",
  Improvement: "<id>",
} as const

export const SOURCE_LABELS = {
  User: "<id>",
  Public: "<id>",
} as const

// If project has agents/products group:
export const AGENTS_GROUP_LABELS = {
  Léo: "<id>", Emma: "<id>", /* ... */
} as const

export function feedbackTypeLabelId(type: string): string | null {
  if (type === "bug") return FEEDBACK_GROUP_LABELS.Bug
  if (type === "feature") return FEEDBACK_GROUP_LABELS.Feature
  if (type === "improvement") return FEEDBACK_GROUP_LABELS.Improvement
  return null  // "question" has no group label
}
```

> **Why hardcoded IDs?** A name-based lookup (`findOrCreateLabel(linear, name)`) can match a stale duplicate label that's NOT in the group. Hardcoding the canonical ID once eliminates that whole class of bug.

### Step 6: Save Configuration to .env.local

```bash
# Append team and project IDs
echo "LINEAR_TEAM_ID=$TEAM_ID" >> .env.local
echo "LINEAR_PROJECT_ID=$PROJECT_ID" >> .env.local
```

Add to `.env.example` (without values):
```bash
grep -q 'LINEAR_API_KEY' .env.example 2>/dev/null || cat >> .env.example << 'EOF'

# Linear Feedback System
LINEAR_API_KEY=
LINEAR_TEAM_ID=
LINEAR_PROJECT_ID=
EOF
```

---

## Phase 3: MCP Configuration

### Step 1: Update .mcp.json

Read the existing `.mcp.json` (or create it) and add the Linear MCP server:

```json
{
  "mcpServers": {
    "linear": {
      "command": "npx",
      "args": ["-y", "@anthropic-ai/linear-mcp-server"],
      "env": {
        "LINEAR_API_KEY": "<actual key from .env.local>"
      }
    }
  }
}
```

**IMPORTANT:** Use the actual key value in .mcp.json (it is gitignored). Check that `.mcp.json` IS in `.gitignore`.

```bash
grep -q '.mcp.json' .gitignore 2>/dev/null || echo '.mcp.json' >> .gitignore
```

### Step 2: Vercel Env Vars (if available)

```bash
if grep -q 'VERCEL_TOKEN' .env.local 2>/dev/null; then
  VERCEL_TOKEN=$(grep '^VERCEL_TOKEN=' .env.local | cut -d= -f2-)
  LINEAR_API_KEY=$(grep '^LINEAR_API_KEY=' .env.local | tail -1 | cut -d= -f2-)
  LINEAR_TEAM_ID=$(grep '^LINEAR_TEAM_ID=' .env.local | tail -1 | cut -d= -f2-)
  LINEAR_PROJECT_ID=$(grep '^LINEAR_PROJECT_ID=' .env.local | tail -1 | cut -d= -f2-)

  vercel env add LINEAR_API_KEY production --token "$VERCEL_TOKEN" <<< "$LINEAR_API_KEY" 2>/dev/null
  vercel env add LINEAR_TEAM_ID production --token "$VERCEL_TOKEN" <<< "$LINEAR_TEAM_ID" 2>/dev/null
  vercel env add LINEAR_PROJECT_ID production --token "$VERCEL_TOKEN" <<< "$LINEAR_PROJECT_ID" 2>/dev/null

  echo "Vercel env vars set."
fi
```

---

## Phase 4: Feedback Widget (Dashboard — Authenticated Users)

### REFERENCE: Copy from L34D

**Target file (write into the detected project):** `<project>/components/dashboard/feedback-widget.tsx`
**Target file:** `components/dashboard/feedback-widget.tsx` (adapt path if project uses `src/`)

**Read the L34D file and copy it exactly.** The only adaptations allowed are:

1. **Language strings** — if French project, translate UI strings (button labels, toast messages, placeholders)
2. **Import paths** — adjust `@/components/ui/...` if project has different UI component paths
3. **Auth-specific imports** — if project doesn't use Clerk, remove Clerk-specific imports from the widget (widget doesn't need auth, only the API route does)

### L34D Widget Architecture (DO NOT DEVIATE)

The widget consists of these exact sections in this exact order:

```
1. Types & Constants
   - FeedbackType = "bug" | "feature" | "improvement" | "question"
   - TargetedElementInfo { selector, tagName, text, screenshot, rect }
   - feedbackTypes array with icons from lucide-react

2. Helper Functions
   - getCssSelector(el) — max 3 levels, filters hover: classes, max 2 classes
   - captureGlobalScreenshot() — html2canvas-pro, scale 0.5, JPEG 0.6, max 1200px, max 1MB
   - captureElementScreenshot(target) — html2canvas-pro, scale 1, JPEG 0.7, max 800px, max 500KB

3. CollapsibleSection component
   - Reusable collapsible with icon, title, badge, chevron rotation
   - Used for both "Targeted Element" and "Page Screenshot" sections

4. Main FeedbackWidget component
   State:
   - dialogOpen, isTargeting, selectedType, description
   - improvedDescription, isImproving, isSubmitting
   - screenshot, targetedElement, previewImage
   Refs:
   - consoleErrorsRef (last 10 console.error calls)
   - isTargetingRef, hoveredRef

   Effects:
   a. Console error capture (intercept console.error, circular buffer of 10)
   b. Targeting mode (overlay + tooltip + mouse tracking + click capture)
   c. Alt key shortcut (starts targeting when not in dialog)

   Handlers:
   a. startTargeting() — sets isTargeting, shows toast
   b. captureAndOpenModal() — captures global screenshot, opens dialog
   c. handleElementSelected() — saves element info, calls captureAndOpenModal
   d. handleTargetingSkip() — calls captureAndOpenModal (no element)
   e. handleImprove() — POST /api/feedback/improve
   f. handleSubmit() — auto-improve if needed, POST /api/feedback
   g. resetForm() — clears all state

   JSX (exact order):
   a. data-feedback-widget wrapper div
   b. Trigger Button (outline, sm, MessageSquarePlus icon + "Feedback" text)
   c. Dialog (max-w-lg, z-[100001], max-h-[85vh], overflow-y-auto)
      - DialogTitle: "Send Feedback"
      - Type selector (Badge components, 4 types)
      - Description label + "Improve with AI" pill button (top-right)
      - Textarea (rows=3, resize-none)
      - AI improved version display (collapsible box, border-primary/20)
      - Targeted Element CollapsibleSection (closed by default, Crosshair icon)
      - Page Screenshot CollapsibleSection (closed by default, Camera icon)
      - Console errors indicator text
      - Submit Button (full width, loading spinner)
   d. Fullscreen Image Preview overlay (z-[100002], bg-black/80)
```

### Critical Implementation Details

| Feature | Spec |
|---------|------|
| **Toast library** | `sonner` — `import { toast } from "sonner"` |
| **Global screenshot** | JPEG 0.6, scale 0.5, max 1200px dimension, max 1MB |
| **Element screenshot** | JPEG 0.7, scale 1.0, max 800px dimension, max 500KB |
| **Console errors** | Intercept `console.error`, keep last 10, circular buffer |
| **html2canvas options** | `allowTaint: false`, `useCORS: true`, `logging: false` |
| **Targeting overlay** | z-index 99999, 3px `var(--primary)` border, `color-mix(in srgb, var(--primary) 8%, transparent)` bg. **NEVER use `hsl(var(--primary))` — breaks oklch values in Tailwind v4** |
| **Targeting tooltip** | z-index 100000, `var(--popover)` bg, `var(--border)` border, shows `<tagName>` |
| **Dialog** | z-index 100001 |
| **Fullscreen preview** | z-index 100002 |
| **Button click** | → targeting mode (NOT dialog open) |
| **Escape key** | → skip targeting → global screenshot → open dialog |
| **Alt key** | → start targeting (secondary shortcut) |
| **data-feedback-widget** | Attribute on wrapper to exclude from targeting |
| **AI improve pill** | Top-right of Description label, rounded-full, border-primary/20 |
| **Auto-improve on submit** | If no improved version exists, auto-call /api/feedback/improve before submit |
| **User Agent** | Sent as `navigator.userAgent` in payload |
| **Page URL** | Sent as `window.location.href` in payload |
| **Pathname** | `usePathname()` from `next/navigation`, sent as `pathname` in payload (used for Scope label) |
| **Priority selector** | 4 buttons (Critique/Haute/Moyenne/Basse), default Moyenne, sent as `priority` in payload |

### Priority Selector (REQUIRED — added v2.1)

The widget MUST include a 4-button priority selector below the type selector and above the Description field:

```tsx
type FeedbackPriority = "critique" | "haute" | "moyenne" | "basse"

const PRIORITIES = [
  { value: "critique", label: "Critique", dotClass: "bg-red-500",    activeClass: "border-red-500/60 bg-red-50 text-red-700 …" },
  { value: "haute",    label: "Haute",    dotClass: "bg-orange-500", activeClass: "border-orange-500/60 bg-orange-50 text-orange-700 …" },
  { value: "moyenne",  label: "Moyenne",  dotClass: "bg-blue-500",   activeClass: "border-blue-500/60 bg-blue-50 text-blue-700 …" },
  { value: "basse",    label: "Basse",    dotClass: "bg-slate-400",  activeClass: "border-slate-400/60 bg-slate-50 text-slate-700 …" },
]

// state: const [feedbackPriority, setFeedbackPriority] = useState<FeedbackPriority>("moyenne")
// JSX: 4 buttons with colored dot + label, same flex pattern as type selector
```

### Pathname Capture (REQUIRED — added v2.1)

```tsx
import { usePathname } from "next/navigation"
// inside component:
const pathname = usePathname()
// in payload: pathname: pathname || window.location.pathname
```

### Add Widget to Dashboard Layout

Find the dashboard header/layout and add:
```tsx
import { FeedbackWidget } from "@/components/dashboard/feedback-widget"
// In the header actions area:
<FeedbackWidget />
```

---

## Phase 5: API Routes (Authenticated)

### 5A: Feedback Route — `/api/feedback/route.ts`

**Target file (write into the detected project):** `<project>/app/api/feedback/route.ts`

Copy L34D's route exactly. The only adaptations:

1. **Auth imports** — adapt to project's auth provider:
   - **Clerk:** `import { auth, currentUser } from "@clerk/nextjs/server"`
   - **Better Auth:** adapt to project's auth helper
   - **None:** skip auth check, use "Anonymous" as user name

2. **Language** — French projects: translate issue body labels

#### L34D API Route Architecture (DO NOT DEVIATE)

```typescript
// Structure:
1. getLinear() — factory with env var check
2. priorityMap — { bug: 2, feature: 3, improvement: 3, question: 4 }
3. emojiMap — { bug: "BUG", feature: "IDEA", improvement: "UP", question: "?" }
4. typeLabelMap — { bug: { name: "Bug", color: "#EF4444" }, ... }
5. findOrCreateLabel() — search ALL labels (filter by name), create if missing
6. uploadToLinear() — base64 → buffer → fileUpload → PUT to pre-signed URL
7. POST handler:
   a. Auth check (Clerk: auth() + currentUser())
   b. Parse body: type, description, improvedDescription, screenshot, pageUrl, userAgent, consoleLogs, targetedElement
   c. Validate description (min 3 chars)
   d. Build issue title: [EMOJI] description[:80]
   e. Build markdown body: Description (original) + AI-improved + Context + Targeted Element + Console Errors
   f. Find/create labels: "Source: User Feedback" + type label
   g. Create issue with labels + optional projectId
   h. Upload global screenshot as COMMENT (not in body)
   i. Upload element screenshot as SEPARATE COMMENT with selector info
   j. Return { success, issueId, identifier }
```

#### Key Rules for API Route

- **NEVER paste base64 in issue body** — always use `linear.fileUpload()`
- **Two separate comments** for screenshots: one for global, one for element
- **Element comment includes selector** in the body text
- **findOrCreateLabel searches ALL scopes** (team + workspace) with `filter: { name: { eq: labelName } }`
- **uploadToLinear validates size** — max 2MB base64 data
- **Linear headers are forwarded** — `uploadData.headers` contains required auth headers for S3
- **Non-blocking screenshot uploads** — if upload fails, issue is still created
- **projectId is optional** — only include if `LINEAR_PROJECT_ID` env var is set
- **NO rate limit on feedback routes** (see [NON-NEGOTIABLE](#-non-negotiable--never-rate-limit-feedback) at the top of this file) — do NOT wrap the feedback POST handlers (auth or public) in `checkRateLimit`/`feedbackRateLimit`. Feedback is low-volume, high-value; a limit blocks a user mid-review-session after only a few submissions. Honeypot (public) handles spam. Never add a 429 path here.

#### NEW in v2.1 — `lib/feedback-scope.ts` helper (REQUIRED)

Create a shared helper file that the route imports. This centralises three things:

```typescript
// 1. Hardcoded label IDs (from Step 5 + 5.5 above)
export const FEEDBACK_GROUP_LABELS = { Bug: "<id>", Feature: "<id>", Improvement: "<id>" } as const
export const SOURCE_LABELS = { User: "<id>", Public: "<id>" } as const

// 2. Pathname → Scope label name mapping (project-specific, generated from Step 5.5)
const PATHNAME_TO_SCOPE: { pattern: RegExp; labelName: string }[] = [
  // SPECIFIC patterns FIRST (e.g. /dashboard/settings/cabinet before /dashboard/settings)
  { pattern: /^\/dashboard\/agents\/leo/, labelName: "Léo" },
  { pattern: /^\/dashboard\/marketing/,    labelName: "Scope: Marketing" },
  // … one entry per dashboard page
]
export function getScopeLabelForPath(pathname: string | null): string | null { /* … */ }
export function pathnameFromUrl(url: string): string | null { /* fallback when client didn't send pathname */ }

// 3. Priority mapping (from widget enum to Linear priority int)
export type FeedbackPriority = "critique" | "haute" | "moyenne" | "basse"
export const PRIORITY_TO_LINEAR: Record<FeedbackPriority, number> = { critique: 1, haute: 2, moyenne: 3, basse: 4 }

// 4. Default assignee (project owner / CTO)
export const DEFAULT_ASSIGNEE_ID = "<linear-user-id>"

// 5. Helper to resolve type label from widget enum
export function feedbackTypeLabelId(type: string): string | null {
  if (type === "bug") return FEEDBACK_GROUP_LABELS.Bug
  if (type === "feature") return FEEDBACK_GROUP_LABELS.Feature
  if (type === "improvement") return FEEDBACK_GROUP_LABELS.Improvement
  return null  // "question" intentionally unmapped
}
```

#### Updated POST handler (v2.1 changes)

Replace the L34D `priorityMap` + `findOrCreateLabel` flow with this:

```typescript
import {
  getScopeLabelForPath, pathnameFromUrl,
  PRIORITY_TO_LINEAR, DEFAULT_ASSIGNEE_ID,
  SOURCE_LABELS, feedbackTypeLabelId,
  type FeedbackPriority,
} from "@/lib/feedback-scope"

// Body now includes priority + pathname:
const { type, description, priority, pathname, pageUrl, /* ... */ } = body

// Resolve priority — user-selected wins over type default
const DEFAULT_TYPE_PRIORITY: Record<string, number> = { bug: 2, feature: 3, improvement: 3, question: 4 }
const resolvedPriority =
  priority && PRIORITY_TO_LINEAR[priority as FeedbackPriority]
    ? PRIORITY_TO_LINEAR[priority as FeedbackPriority]
    : DEFAULT_TYPE_PRIORITY[type] ?? 3

// Resolve scope label from pathname (or pageUrl fallback)
const effectivePath = pathname || pathnameFromUrl(pageUrl)
const scopeLabelName = getScopeLabelForPath(effectivePath)

// Build label IDs — use HARDCODED group IDs, scope still by name (catalogue grows)
const labelIds: string[] = [SOURCE_LABELS.User]
const typeLabelId = feedbackTypeLabelId(type)
if (typeLabelId) labelIds.push(typeLabelId)
if (scopeLabelName) {
  labelIds.push(await findOrCreateLabel(linear, LINEAR_TEAM_ID, scopeLabelName))
}

// Create issue WITH default assignee + user-selected priority
const issuePayload = await linear.createIssue({
  teamId: LINEAR_TEAM_ID,
  projectId: LINEAR_PROJECT_ID,
  title,
  description: markdownDescription,
  priority: resolvedPriority,
  labelIds,
  assigneeId: DEFAULT_ASSIGNEE_ID,  // ← NEW: tickets land in owner's queue
})
```

> **Why default assignee?** Without it, feedback tickets land unassigned and get lost. With it, they show up in the project owner's "Assigned to me" view immediately. Override per-project by changing `DEFAULT_ASSIGNEE_ID` in `feedback-scope.ts`.

### 5B: AI Improve Route — `/api/feedback/improve/route.ts`

**Target file (write into the detected project):** `<project>/app/api/feedback/improve/route.ts`

Copy L34D's route exactly:

```typescript
import { NextRequest, NextResponse } from "next/server"
import Anthropic from "@anthropic-ai/sdk"

const client = new Anthropic()

export async function POST(req: NextRequest) {
  try {
    const { description, type } = await req.json()
    if (!description || typeof description !== "string" || description.trim().length < 5) {
      return NextResponse.json({ improved: description }, { status: 200 })
    }

    const typeLabel =
      type === "bug" ? "bug report"
        : type === "feature" ? "feature request"
        : type === "improvement" ? "improvement suggestion"
        : "question"

    const systemPrompt = `You are a QA assistant that rewrites user feedback into clear, structured ${typeLabel}s that developers can immediately act on.

Output format (plain text, no markdown):
- Line 1: One-sentence summary of the issue/request
- Line 2-3: Steps to reproduce or expected behavior (if applicable)
- Line 4: Expected vs actual result (for bugs) or desired outcome (for features)

Rules:
- Keep the user's original meaning intact
- Be specific and technical where possible
- Remove filler words, keep it concise (2-5 sentences max)
- Write in English
- Do NOT add a title, heading, or bullet points — just flowing text`

    const response = await client.messages.create({
      model: "claude-haiku-4-5-20251001",
      max_tokens: 400,
      system: systemPrompt,
      messages: [{
        role: "user",
        content: `Rewrite this user ${typeLabel} into a clear, actionable description:\n\n"${description.trim()}"`,
      }],
    })

    const text = response.content[0]?.type === "text" ? response.content[0].text : description
    return NextResponse.json({ improved: text.trim() })
  } catch (error) {
    console.error("[Feedback Improve]", error)
    return NextResponse.json({ improved: "" }, { status: 500 })
  }
}
```

**French project adaptation:** Change the system prompt to write in French and use French type labels.

---

## Phase 6: Public Feedback (Optional)

Ask the user:
> Does this project have public/marketing pages where visitors (non-authenticated users) should be able to send feedback?

If YES:

### 6A: Public Feedback Button

**Target file (write into the detected project):** `<project>/components/shared/public-feedback-button.tsx`

Copy L34D's public button exactly. Key differences from dashboard widget:
- **Floating pill button** — `fixed bottom-6 left-6 z-50`, rounded-full, bg-primary
- **Honeypot field** — hidden input for spam protection
- **Posts to `/api/feedback/public`** instead of `/api/feedback`
- **No rate limit, no 429 branch** — feedback routes are never rate-limited (see [NON-NEGOTIABLE](#-non-negotiable--never-rate-limit-feedback) at the top of this file); the honeypot field is the only anti-spam
- **Same targeting/screenshot/AI flow** as dashboard widget

### 6B: Public API Route — `/api/feedback/public/route.ts`

**Target file (write into the detected project):** `<project>/app/api/feedback/public/route.ts`

Copy L34D's route exactly. Key differences from authenticated route:
- **No auth** — no Clerk/auth check
- **IP tracking** — `req.headers.get("x-forwarded-for")`
- **Honeypot check** — if honeypot field is filled, return fake success `{ success: true, issueId: "fake" }`
- **Source label** — `SOURCE_LABELS.Public` (id from `feedback-scope.ts`) instead of `SOURCE_LABELS.User`
- **Context section** includes `Source: Public visitor` and IP address

> **Apply ALL v2.1 features from Phase 5 here too**: import `feedback-scope.ts`, accept `priority` + `pathname` in body, resolve scope label via `getScopeLabelForPath()`, use `feedbackTypeLabelId()`, default `assigneeId: DEFAULT_ASSIGNEE_ID`. The only delta vs the authenticated route is the source label.

### 6C: Add to Marketing Layout

```tsx
import { PublicFeedbackButton } from "@/components/shared/public-feedback-button"
// At the bottom of the marketing layout body:
<PublicFeedbackButton />
```

---

## Phase 7: Vercel Configuration

Check if `vercel.json` exists and add/update function timeout for feedback routes:

```json
{
  "functions": {
    "app/api/feedback/route.ts": { "maxDuration": 30 },
    "app/api/feedback/public/route.ts": { "maxDuration": 30 },
    "app/api/feedback/improve/route.ts": { "maxDuration": 15 }
  }
}
```

Adapt paths if project uses `src/`:
```json
{
  "functions": {
    "src/app/api/feedback/route.ts": { "maxDuration": 30 },
    "src/app/api/feedback/public/route.ts": { "maxDuration": 30 },
    "src/app/api/feedback/improve/route.ts": { "maxDuration": 15 }
  }
}
```

---

## Phase 8: Post-Setup Verification

### Verify All Files Exist

```bash
echo "=== Checking feedback system files ==="

# Components
for f in "components/dashboard/feedback-widget.tsx" "components/shared/public-feedback-button.tsx"; do
  # Try with and without src/ prefix
  if [ -f "$f" ] || [ -f "src/$f" ]; then
    echo "OK: $f"
  else
    echo "MISSING: $f"
  fi
done

# API routes
for f in "app/api/feedback/route.ts" "app/api/feedback/public/route.ts" "app/api/feedback/improve/route.ts"; do
  if [ -f "$f" ] || [ -f "src/$f" ]; then
    echo "OK: $f"
  else
    echo "MISSING: $f"
  fi
done

# Env vars
echo "=== Checking env vars ==="
for var in LINEAR_API_KEY LINEAR_TEAM_ID; do
  grep -q "^${var}=" .env.local && echo "OK: $var" || echo "MISSING: $var"
done
grep -q "^LINEAR_PROJECT_ID=" .env.local && echo "OK: LINEAR_PROJECT_ID (optional)" || echo "INFO: LINEAR_PROJECT_ID not set (optional)"

# Dependencies
echo "=== Checking dependencies ==="
for pkg in "@linear/sdk" "html2canvas-pro" "@anthropic-ai/sdk" "sonner"; do
  grep -q "\"$pkg\"" package.json && echo "OK: $pkg" || echo "MISSING: $pkg"
done

# Sonner Toaster
echo "=== Checking Toaster ==="
grep -r "Toaster" app/layout.tsx src/app/layout.tsx 2>/dev/null | head -1 || echo "WARNING: <Toaster /> not found in root layout"
```

### Test API Connection

```bash
LINEAR_KEY=$(grep '^LINEAR_API_KEY=' .env.local | tail -1 | cut -d= -f2-)
echo "Testing Linear API..."
curl -s -X POST https://api.linear.app/graphql \
  -H "Content-Type: application/json" \
  -H "Authorization: $LINEAR_KEY" \
  -d '{"query":"{ viewer { name } }"}' | python3 -c "
import json, sys
d = json.load(sys.stdin)
if 'data' in d and 'viewer' in d['data']:
    print(f'Connected as: {d[\"data\"][\"viewer\"][\"name\"]}')
else:
    print('ERROR: ' + json.dumps(d))
"
```

### Update Project CLAUDE.md

Add a Linear integration section to the project's CLAUDE.md:

```markdown
## Linear Feedback

| Component | Path |
|-----------|------|
| Dashboard widget | `components/dashboard/feedback-widget.tsx` |
| Public button | `components/shared/public-feedback-button.tsx` |
| Auth API | `app/api/feedback/route.ts` |
| Public API | `app/api/feedback/public/route.ts` |
| AI improve | `app/api/feedback/improve/route.ts` |

**Env vars:** `LINEAR_API_KEY`, `LINEAR_TEAM_ID`, `LINEAR_PROJECT_ID`
**Workflow:** Backlog → In Progress (AI) → Review (human) → Done
```

---

## Post-Setup Message

```
Linear Feedback System installed!

Files created:
  components/dashboard/feedback-widget.tsx  - Dashboard feedback widget
  app/api/feedback/route.ts                 - Authenticated feedback API
  app/api/feedback/improve/route.ts         - AI description improvement
  [if public] components/shared/public-feedback-button.tsx
  [if public] app/api/feedback/public/route.ts

Configuration:
  .env.local          - LINEAR_API_KEY, LINEAR_TEAM_ID, LINEAR_PROJECT_ID
  .mcp.json           - Linear MCP server added
  vercel.json         - maxDuration: 30 for feedback routes
  [if vercel] Vercel env vars set

Linear:
  Team: <team_name>
  Project: User Feedback
  Labels: Source: User Feedback, Source: Public Feedback, Bug, Feature, Improvement

Features (L34D design):
  - Element targeting: click Feedback → crosshair mode → select element → dialog
  - Dual screenshots: global (JPEG 0.6, max 1MB) + element (JPEG 0.7, max 500KB)
  - Console errors: last 10 captured automatically
  - User Agent + Page URL: sent with every submission
  - AI improve: "Improve with AI" button + auto-improve on submit
  - Collapsible sections: targeted element + page screenshot
  - Fullscreen image preview
  - Honeypot anti-spam (public route)
  - IP tracking (public route)
  - Toast feedback via Sonner

Workflow:
  Feedback → Backlog → AI fixes → Review → Human marks Done
```

---

## Rules

1. **Generate from the reference code in this skill** — each component/route below has its full source inline (widget, feedback route, improve route, public button/route). Write those into the target project, adapting ONLY auth provider, language, and src/ paths to the project detected in Phase 0. (Do not depend on any external/private project checkout.)
2. **Token handling** — write to .env.local FIRST, then read back with `grep | cut` — NEVER rely on shell variables persisting
3. **findOrCreateLabel must search ALL scopes** — team + workspace labels
4. **Use linear.fileUpload() for ALL screenshots** — NEVER paste base64 in descriptions
5. **Two separate comments** — global screenshot = one comment, element screenshot = another
6. **allowTaint: false** on html2canvas — prevents SecurityError
7. **Targeting-first flow** — button click → crosshair → element selection → THEN dialog
8. **Escape skips targeting** — opens dialog with global screenshot only
9. **Dialog z-index 100001 > overlay z-index 99999**
10. **data-feedback-widget attribute** — exclude widget from targeting
11. **Default type is "bug"**
12. **Global: JPEG 0.6, scale 0.5, 1200px max, 1MB limit**
13. **Element: JPEG 0.7, scale 1.0, 800px max, 500KB limit**
14. **Auto-improve on submit** — if user didn't manually improve, auto-call improve API
15. **Both descriptions in Linear** — "Description (original)" + "Description (AI-improved)"
16. **AI improve is non-blocking** — if it fails, submit continues
17. **Console error buffer** — max 10 entries, FIFO
18. **Sonner for ALL toasts** — `import { toast } from "sonner"`
19. **Honeypot** — hidden input, if filled return fake success (public route only)
20. **IP tracking** — `x-forwarded-for` header (public route only)
21. **Collapsible sections** — both screenshot sections closed by default
22. **Fullscreen preview** — click on screenshot thumbnail opens z-[100002] overlay
23. **Alt key shortcut** — secondary way to start targeting
24. **Verify .mcp.json is gitignored**
25. **Adapt language** — French projects get French UI strings + French AI prompts
26. **Label GROUPS not flat labels** — `Bug`/`Feature`/`Improvement` MUST live inside a `FEEDBACK` parent group (`isGroup: true` + child `parentId`); same for `User Feedback`/`Public Feedback` inside a `SOURCE` group
27. **Hardcode label IDs in `lib/feedback-scope.ts`** — never look up by name in the API route (race risk if a duplicate is created later); name-based lookup is OK only for `Scope: X` labels which grow over time
28. **Project-aware Scope analysis** — read the sidebar component, generate one `Scope: <PageTitle>` label per route, then add a `PATHNAME_TO_SCOPE` regex map in `feedback-scope.ts`. Specific patterns BEFORE general (e.g. `/dashboard/settings/cabinet` before `/dashboard/settings`)
29. **Detect agent/product groups** — multi-agent platforms get an `AGENTS` group; multi-product SaaS gets a `PRODUCTS` group. Single-product apps: skip, use only Scope labels
30. **Priority selector REQUIRED** — 4 buttons (Critique/Haute/Moyenne/Basse), default Moyenne, sent as `priority` in payload. API translates via `PRIORITY_TO_LINEAR` (1/2/3/4). User-selected wins over type-based default
31. **Default assignee REQUIRED** — every issue gets `assigneeId: DEFAULT_ASSIGNEE_ID` (project owner / CTO Linear user ID). Without this, tickets land unassigned and rot
32. **Pathname capture** — widget uses `usePathname()` from `next/navigation`, sends `pathname` in payload; API maps via `getScopeLabelForPath()` and falls back to `pathnameFromUrl(pageUrl)` if missing
33. **Cleanup duplicates after group creation** — if you find existing flat `Bug`/`Feature` labels at the root, list them and delete via `issueLabelDelete` mutation after confirming nothing references them. Keep the GROUP children only

---

*L34D Reference Design | DentistryGPT label-group hardening | v2.1 | 2026-04-17*

## Changelog

**v2.1 (2026-04-17)** — Label groups, priority selector, scope analysis, default assignee
- Phase 2 Step 5: REWRITTEN — label GROUPS (`FEEDBACK`/`SOURCE`/optional `AGENTS`) instead of flat labels
- Phase 2 NEW Step 5.5: Project-aware scope analysis — auto-detect dashboard pages and agents from sidebar/agents config
- Phase 4 Widget: ADDED 4-button priority selector (Critique/Haute/Moyenne/Basse) + `usePathname()` capture
- Phase 5 API: NEW `lib/feedback-scope.ts` helper centralising hardcoded label IDs, scope mapping, priority translation, default assignee
- Phase 5 API: ADDED user-selected priority override + default `assigneeId` on every issue
- Rules 26-33 added (label groups, hardcoded IDs, scope analysis, priority, assignee, pathname, cleanup)

**v2.0 (2026-03-04)** — L34D reference design baseline
