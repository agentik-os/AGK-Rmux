/**
 * omega-new-project — HISTORICAL reference implementation (NOT what runs today).
 *
 * The live /omg-new-project (alias /omega-new-project) pipeline is the SKILL at
 * skills/new-project/SKILL.md (installed to ~/.omega/skills/new-project/), which
 * provisions, scaffolds, then hands off to /omg-planner + `omega plan-run`.
 * Nothing in the repo invokes this file. It is kept as a reference for the typed
 * phase-DAG Workflow shape it pioneered (deterministic control flow, parallel
 * fan-out, adversarial 2-of-3 gates, resume via resumeFromRunId) — a
 * delete-candidate once that shape is fully absorbed by the skill + plan-run.
 *
 * SCOPE: this engine owns the MECHANICAL + verifiable phases —
 *   P0 capability probe -> P1 provision (parallel, idempotent) -> GATE-A
 *   -> P2 scaffold (pipeline) -> P3 wire+register -> GATE-B (build truth).
 * The CREATIVE pipeline (vision -> prd -> brand -> deepux -> planner -> execute
 * -> audit/verify) is DELEGATED to the proven skills by the command after this
 * engine returns its handoff (R-KARPATHY: delegate, don't re-implement).
 *
 * NO fs/Node in a Workflow script: every file read/write + shell call is done by
 * the spawned agents (they have Bash/Read/Write). That is also what makes resume
 * idempotent — each provisioner does lookup-then-create and records its own slot
 * in .omega-provision.json, so a cached/replayed agent and a fresh one converge.
 *
 * args = {
 *   name, stack='nextstack', category='works', group='default',
 *   projectDir,                       // absolute, resolved by the command (P-1)
 *   gitEmail,                         // resolved from category (R-FICHE)
 *   dispatched=false,                 // true => never AskUserQuestion; write block-file + fallback (L3)
 *   wantReactFlow=false,
 *   dryRun=false
 * }
 */

export const meta = {
  name: 'omega-new-project',
  description: 'OmegaOS new-project bootstrap engine: probe -> provision (parallel) -> verify -> scaffold -> wire -> build-truth gate. Idempotent, resumable, honest pause.',
  phases: [
    { title: 'P0 Capability', detail: 'live token probes -> CAN_* map (200-only, never env-presence)' },
    { title: 'P1 Provision', detail: '5 file-disjoint provisioners in parallel, lookup-then-create' },
    { title: 'GATE-A', detail: '2-of-3 adversarial verify: every service done-with-id or paused-with-reason, zero silent skip' },
    { title: 'P2 Scaffold', detail: 'pipeline: versions -> create-next-app -> shadcn+kit -> deps -> core files -> brand' },
    { title: 'P3 Wire', detail: 'env.local + .env.example + git identity + register + ecosystem' },
    { title: 'GATE-B', detail: '2-of-3 build-truth: safe-npm-build 0 errors + wiring + registration' },
  ],
}

// args may arrive as an object OR a JSON string (the runtime stringifies it) — parse defensively.
const a = (typeof args === 'string' ? JSON.parse(args || '{}') : (args || {}))
const NAME = a.name
const STACK = a.stack || 'nextstack'
const CATEGORY = a.category || 'works'
const DIR = a.projectDir
const DISPATCHED = !!a.dispatched
const DRY = !!a.dryRun

if (!NAME || !DIR) {
  throw new Error('omega-new-project: args.name and args.projectDir are required (resolve them in P-1 before launching the engine).')
}

const PAUSE_RULE =
  DISPATCHED
    ? 'You are DISPATCHED (non-interactive): NEVER ask the user. If a token is missing/invalid, record status="paused" with a plain reason in .omega-provision.json AND write ~/.aisb/state/worker-blocked-' + '${run}' + '.json {service,question,best_guess,fallback}, then EXECUTE the fallback (record pause + continue). Law L3.'
    : 'You are INTERACTIVE: a pause may surface the dashboard link + collect keys, persisting them immediately to .omega-provision.json.'

const SERVICE_SCHEMA = {
  type: 'object', additionalProperties: false,
  required: ['service', 'status', 'reason'],
  properties: {
    service: { type: 'string' },
    status: { type: 'string', enum: ['done', 'paused', 'failed'] },
    capturedIds: { type: 'object', additionalProperties: true },
    reason: { type: 'string' },
    lastProbe: { type: 'string' },
  },
}

const VERDICT_SCHEMA = {
  type: 'object', additionalProperties: false,
  required: ['pass', 'evidence', 'problems'],
  properties: {
    pass: { type: 'boolean' },
    evidence: { type: 'array', items: { type: 'string' } },
    problems: { type: 'array', items: { type: 'string' } },
  },
}

const STEP_SCHEMA = {
  type: 'object', additionalProperties: false,
  required: ['ok', 'summary', 'details'],
  properties: {
    ok: { type: 'boolean' },
    summary: { type: 'string' },
    details: { type: 'array', items: { type: 'string' } },
  },
}

log(`omega-new-project engine: ${NAME} (${STACK}/${CATEGORY}) -> ${DIR}${DRY ? ' [DRY-RUN]' : ''}`)

if (DRY) {
  // Dry-run: print the resolved DAG and STOP with zero mutation.
  log('DRY-RUN: phases = P0 Capability -> P1 Provision(5//) -> GATE-A(2of3) -> P2 Scaffold(pipeline) -> P3 Wire -> GATE-B(2of3). No mutation performed.')
  return { dryRun: true, name: NAME, stack: STACK, category: CATEGORY, projectDir: DIR }
}

// ─── P0 Capability map (live probes, 200-only) ───────────────────────────────
phase('P0 Capability')
const capability = await agent(
  `Build the provisioning capability map for project "${NAME}". ` +
  `Source ~/.omega/provisioning/services.env (set -a; source ...; set +a). ` +
  `For EACH service flip CAN_* ON only after a CHEAP LIVE auth probe returning success (never env-presence alone): ` +
  `GitHub = \`gh auth status\` OR a 200 from the GitHub API with $GITHUB_TOKEN; ` +
  `Vercel = GET https://api.vercel.com/v9/projects with Bearer $VERCEL_TOKEN returns 200; ` +
  `Convex = $CONVEX_TEAM_TOKEN present AND \`npx convex --help\` shows the expected flags (verify, do not assume — L1); ` +
  `Stripe = GET https://api.stripe.com/v1/account with $STRIPE_SECRET_KEY returns 200; ` +
  `Clerk = CLERK_PROVISION_MODE (pool: clerk-pool.env has an unused line; else pause — no create-API). ` +
  `A present-but-INVALID token (401/403) => CAN=false => the service will PAUSE, never fail mid-provision. ` +
  `Do NOT print secret values. Return a structured summary of which services are auto-capable vs paused, with the probe result for each.`,
  { schema: STEP_SCHEMA, label: 'capability-probe', phase: 'P0 Capability' }
)
log(`P0: ${capability?.summary || 'capability map built'}`)

// ─── P1 Provision (5 file-disjoint provisioners, parallel) ───────────────────
// R-SCOPE: each provisioner writes ONLY its own slot in .omega-provision.json.
phase('P1 Provision')
const services = ['github', 'vercel', 'convex', 'clerk', 'stripe']
const provisionPrompts = {
  github: `Provision GitHub for "${NAME}" (dir ${DIR}). Idempotent lookup-then-create: \`gh repo view <owner>/${NAME}\` first; if absent \`gh repo create <owner>/${NAME} --private --source="${DIR}" --remote=origin\` (owner = $GITHUB_OWNER or \`gh api user -q .login\`). ${PAUSE_RULE} Record your result ONLY into the "github" key of ${DIR}/.omega-provision.json (create the file/dir if needed; never touch other services' keys).`,
  vercel: `Provision Vercel for "${NAME}". If CAN_VERCEL: GET https://api.vercel.com/v9/projects/${NAME} (Bearer $VERCEL_TOKEN, +?teamId=$VERCEL_TEAM_ID) — if 404 POST https://api.vercel.com/v11/projects {name,framework:nextjs}; then \`vercel link --yes --project ${NAME} --token $VERCEL_TOKEN\` in ${DIR}. Capture the project id. ${PAUSE_RULE} Record ONLY the "vercel" key of ${DIR}/.omega-provision.json.`,
  convex: `Provision Convex for "${NAME}" in ${DIR}. Ensure \`npm i convex\` first. Verify flags with \`npx convex dev --help\` (L1). Lookup an existing deployment; else \`CONVEX_AGENT_MODE=anonymous npx convex dev --once --configure=new --dev-deployment local --project ${NAME}\` (or with $CONVEX_TEAM_TOKEN/team if available). Capture CONVEX_DEPLOYMENT + NEXT_PUBLIC_CONVEX_URL from the .env.local convex writes. ${PAUSE_RULE} Record ONLY the "convex" key of ${DIR}/.omega-provision.json.`,
  clerk: `Provision Clerk for "${NAME}" (NO public create-API). If CLERK_PROVISION_MODE=pool: pop the first unused line of ~/.omega/provisioning/clerk-pool.env, mark it "# USED ${NAME}", capture pk/sk. Else PAUSE. ${PAUSE_RULE} State plainly that pool/pause is a necessity, not a shortcut. Record ONLY the "clerk" key of ${DIR}/.omega-provision.json.`,
  stripe: `Provision Stripe for "${NAME}". If CAN_STRIPE and STRIPE_MODE=single: product-search-before-create (avoid dupes), create a restricted key + a webhook endpoint (placeholder URL https://${NAME}.vercel.app/api/stripe/webhook — note a P3 reconcile will re-point it once the real Vercel URL is known), capture whsec_. connect mode: POST /v1/accounts + onboarding link, surface KYC. ${PAUSE_RULE} Record ONLY the "stripe" key of ${DIR}/.omega-provision.json.`,
}
const provResults = await parallel(
  services.map((s) => () =>
    agent(provisionPrompts[s], { schema: SERVICE_SCHEMA, label: `provision:${s}`, phase: 'P1 Provision' })
  )
)
const provisioned = provResults.filter(Boolean)
const done = provisioned.filter((r) => r.status === 'done').map((r) => r.service)
const paused = provisioned.filter((r) => r.status === 'paused').map((r) => r.service)
const failed = provisioned.filter((r) => r.status === 'failed').map((r) => r.service)
log(`P1: done=[${done}] paused=[${paused}] failed=[${failed}]`)

// ─── GATE-A: 2-of-3 adversarial provision verifier ───────────────────────────
phase('GATE-A')
const gateALenses = [
  `Lens A (schema validity): read ${DIR}/.omega-provision.json. Is it valid JSON with one slot per service, each carrying status in {done,paused,failed} + a reason? Any malformed/missing slot FAILS.`,
  `Lens B (live-id resolves): for every service marked "done", does its captured id resolve to a REAL live resource (gh repo view / vercel project GET / convex deployment / stripe account)? A "done" with no resolvable id FAILS.`,
  `Lens C (zero silent skip + no leaked secret): is EVERY one of the 5 services present as either done-with-id OR paused-with-recorded-reason — none silently missing? AND confirm no live secret value was written into any git-tracked file (R-ENV: secrets only in .env.local + ~/.omega). A silently-skipped service or a leaked secret FAILS.`,
]
const gateA = await parallel(
  gateALenses.map((p, i) => () =>
    agent(`Adversarially verify the provisioning ledger for "${NAME}". ${p} Default to pass=false if uncertain. Cite evidence (file:line / command output).`,
      { schema: VERDICT_SCHEMA, label: `gateA:lens${i + 1}`, phase: 'GATE-A' })
  )
)
const gateAPass = gateA.filter(Boolean).filter((v) => v.pass).length >= 2
log(`GATE-A: ${gateAPass ? 'PASS' : 'FAIL'} (${gateA.filter(Boolean).filter(v => v.pass).length}/3 lenses)`)
if (!gateAPass) {
  return { aborted: 'GATE-A', reason: 'provisioning ledger not honest/complete', gateA, provisioned }
}

// ─── P2 Scaffold (pipeline of stages) ────────────────────────────────────────
phase('P2 Scaffold')
const scaffold = await agent(
  `Scaffold the ${STACK} stack for "${NAME}" in ${DIR}. Pipeline, in order: ` +
  `(1) Resolve+LOCK latest versions (Context7 resolve-library-id+query for next/convex/@clerk/nextjs/stripe/tailwindcss if available, else @latest). ` +
  `(2) \`npx create-next-app@latest ${NAME} --ts --app --tailwind --eslint --src-dir --import-alias "@/*" --use-npm --yes\` (from ${DIR}/..). ` +
  `(3) \`npx shadcn@latest init -d\` then add the FULL shadcn-chatbot-kit (every chat component — L5, never a subset). ` +
  `(4) \`npm i convex @clerk/nextjs stripe @stripe/stripe-js\`${a.wantReactFlow ? ' + @xyflow/react' : ''}. ` +
  `(5) Core files: src/app/providers.tsx (ClerkProvider+ConvexProviderWithClerk, degraded-mode-tolerant of paused keys), src/proxy.ts (Next 16 Clerk proxy, tolerant of absent keys), convex/schema.ts + convex/auth.config.ts (Clerk issuer, tolerant), src/app/api/stripe/webhook/route.ts (signature-verified, 503 when paused), a /chat route mounting the kit${a.wantReactFlow ? ', a /flow React Flow route' : ''}. ` +
  `(6) FULL oklch brand scale in globals.css (light/dark/radii/shadows/typography) + BRAND.md. ` +
  `Fix any React 19 / Base UI / TS strictness issues so the tree TYPECHECKS. Return ok + a summary of what was created.`,
  { schema: STEP_SCHEMA, label: 'scaffold', phase: 'P2 Scaffold' }
)
log(`P2: ${scaffold?.summary || 'scaffold complete'}`)
if (scaffold && scaffold.ok === false) {
  return { aborted: 'P2', reason: 'scaffold failed', scaffold }
}

// ─── P3 Wire + register ──────────────────────────────────────────────────────
phase('P3 Wire')
const wire = await agent(
  `Wire + register "${NAME}" (dir ${DIR}, git identity ${a.gitEmail || 'x@agentik-os.com'}). ` +
  `(1) Write ${DIR}/.env.local from the provisioned slots in .omega-provision.json (Convex URL+deployment, Clerk pub+secret, Stripe restricted+whsec+pub) leaving paused ones blank; commit ${DIR}/.env.example with the SAME keys BLANKED. Ensure .env.local is gitignored and .env.example is NOT (add !.env.example). Secrets ONLY in .env.local + ~/.omega — never staged (R-ENV). ` +
  `(2) If a real Vercel URL exists, reconcile the Stripe webhook endpoint to it + re-capture whsec_. Push non-public vars to Vercel with --token (CAN_VERCEL only, R-VERCEL). ` +
  `(3) git config user.email "${a.gitEmail || 'x@agentik-os.com'}"; commit; push to origin if the repo was created. ` +
  `(4) Register in ~/.omega/projects.json (name,path,category,git_email,created_at,oracle_session). ` +
  `Return ok + summary.`,
  { schema: STEP_SCHEMA, label: 'wire-register', phase: 'P3 Wire' }
)
log(`P3: ${wire?.summary || 'wired + registered'}`)

// ─── GATE-B: 2-of-3 build-truth verifier (L1 hard gate) ──────────────────────
phase('GATE-B')
const gateBLenses = [
  `Lens A (build truth): run \`bash ~/.aisb/lib/safe-npm-build.sh\` in ${DIR} (NEVER raw npm run build). It MUST exit 0 with no type errors. Paste the final lines as evidence.`,
  `Lens B (wiring): does ${DIR}/.env.local contain every required key for the non-paused services, and is .env.example committed with them blanked? Is .env.local gitignored (no secret staged)?`,
  `Lens C (registration): does \`gh repo view\` (if created) succeed AND is "${NAME}" present in ~/.omega/projects.json with the correct path+category?`,
]
const gateB = await parallel(
  gateBLenses.map((p, i) => () =>
    agent(`Adversarially verify the scaffold for "${NAME}". ${p} Default to pass=false if uncertain. Cite evidence.`,
      { schema: VERDICT_SCHEMA, label: `gateB:lens${i + 1}`, phase: 'GATE-B' })
  )
)
const gateBPass = gateB.filter(Boolean).filter((v) => v.pass).length >= 2
log(`GATE-B: ${gateBPass ? 'PASS' : 'FAIL'} (${gateB.filter(Boolean).filter(v => v.pass).length}/3 lenses)`)

// ─── Handoff to the creative pipeline (delegated to skills by the command) ────
return {
  name: NAME, stack: STACK, category: CATEGORY, projectDir: DIR,
  provision: { done, paused, failed },
  gates: { GATE_A: gateAPass, GATE_B: gateBPass },
  buildVerified: gateBPass,
  handoff: gateBPass
    ? 'MECHANICAL phases complete + build-verified. Command should now chain the CREATIVE pipeline via Skill: vision -> prd -> brand -> deepux -> planner -> (optional) build -> audit/verify, each behind a light rubric gate.'
    : 'GATE-B did not pass — do NOT chain the creative pipeline until the build is green. Inspect gateB problems.',
  blockers: paused.concat(failed),
}
