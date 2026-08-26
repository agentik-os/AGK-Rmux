export const meta = {
  name: 'adversarial-review',
  description: 'Review a change across distinct lenses, then try to refute every finding before it ships',
  whenToUse: 'A diff, a branch or a file set needs a review whose findings you can actually trust. Routes on change size, reviews in parallel lenses, kills anything a majority of skeptics can refute.',
  phases: [
    { title: 'Scope', detail: 'size the change, pick the lenses' },
    { title: 'Review', detail: 'one agent per lens' },
    { title: 'Verify', detail: 'independent skeptics try to refute each finding' },
    { title: 'Synthesize', detail: 'rank what survived' },
  ],
}

// args: a string target, or {target, lenses}. Defaults to the working tree.
const TARGET =
  typeof args === 'string'
    ? args
    : (args && args.target) || 'the uncommitted changes in this repository (git diff HEAD, plus untracked files)'

const CLASS = {
  type: 'object',
  properties: {
    size: { type: 'string', enum: ['trivial', 'normal', 'large'] },
    summary: { type: 'string' },
    files: { type: 'array', items: { type: 'string' } },
  },
  required: ['size', 'summary'],
}

const FINDINGS = {
  type: 'object',
  properties: {
    findings: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          title: { type: 'string' },
          file: { type: 'string' },
          line: { type: 'integer' },
          severity: { type: 'string', enum: ['critical', 'high', 'medium', 'low'] },
          evidence: { type: 'string', description: 'file:line plus the quoted code that proves it' },
          failure: { type: 'string', description: 'concrete inputs or state that produce the wrong behaviour' },
        },
        required: ['title', 'file', 'severity', 'evidence', 'failure'],
      },
    },
  },
  required: ['findings'],
}

const VERDICT = {
  type: 'object',
  properties: {
    refuted: { type: 'boolean' },
    why: { type: 'string' },
  },
  required: ['refuted', 'why'],
}

const ALL_LENSES = [
  { key: 'correctness', brief: 'logic errors, off-by-one, wrong branch, unhandled null, broken invariant' },
  { key: 'security', brief: 'injection, authz gaps, secret exposure, unsafe deserialization, path traversal' },
  { key: 'regression', brief: 'behaviour this change silently breaks for existing callers, data or tests' },
  { key: 'concurrency', brief: 'races, shared mutable state, ordering assumptions, partial failure' },
  { key: 'performance', brief: 'N+1 access, unbounded growth, work repeated per item that belongs outside the loop' },
]

// ---- Scope: a Claude-powered classification, a code-level branch (R-GRAPH step 5).
phase('Scope')
const scope = await agent(
  `Inspect ${TARGET}. Report what changed and how big it is.\n` +
    `"trivial" = a few lines, no behaviour change. "normal" = a contained feature or fix. ` +
    `"large" = many files, or it touches auth, data, money or a public contract.\n` +
    `Do not review it yet. Just classify and list the files.`,
  { label: 'scope', phase: 'Scope', schema: CLASS },
)

if (!scope) {
  log('scope agent returned nothing — nothing to review')
  return { findings: [], note: 'scope stage failed' }
}

const requested = args && Array.isArray(args.lenses) ? args.lenses : null
const lensCount = scope.size === 'trivial' ? 1 : scope.size === 'normal' ? 3 : 5
const LENSES = requested
  ? ALL_LENSES.filter((l) => requested.includes(l.key))
  : ALL_LENSES.slice(0, lensCount)
const SKEPTICS = scope.size === 'trivial' ? 1 : 3

log(`${scope.size} change over ${(scope.files || []).length} file(s) — ${LENSES.length} lens(es), ${SKEPTICS} skeptic(s) per finding`)

// ---- Review then verify, as a PIPELINE: a lens verifies its own findings the moment
// it returns, instead of idling behind the slowest lens (R-GRAPH step 1).
const perLens = await pipeline(
  LENSES,
  (lens) =>
    agent(
      `Review ${TARGET} through ONE lens: ${lens.key} — ${lens.brief}.\n` +
        `Context: ${scope.summary}\n` +
        `Report only defects you can prove with file:line evidence. Do not report style, ` +
        `naming or anything outside your lens. An empty list is a valid, respected answer.`,
      { label: `review:${lens.key}`, phase: 'Review', schema: FINDINGS },
    ),
  (review, lens) =>
    review && review.findings && review.findings.length
      ? parallel(
          review.findings.map((f) => () =>
            parallel(
              Array.from({ length: SKEPTICS }, (_unused, i) => () =>
                agent(
                  `Try to REFUTE this claimed defect. You are skeptic ${i + 1} of ${SKEPTICS}.\n\n` +
                    `Claim: ${f.title}\nAt: ${f.file}${f.line ? ':' + f.line : ''}\n` +
                    `Evidence offered: ${f.evidence}\nClaimed failure: ${f.failure}\n\n` +
                    `Read the real code. Refute it if the failure cannot actually happen, if the ` +
                    `guard exists elsewhere, or if the evidence does not say what it claims. ` +
                    `If you are uncertain, refute it — the bar is proof, not plausibility.`,
                  { label: `refute:${lens.key}:${i + 1}`, phase: 'Verify', schema: VERDICT },
                ),
              ),
            ).then((votes) => {
              const live = votes.filter(Boolean)
              const survived = live.filter((v) => !v.refuted).length
              return { ...f, lens: lens.key, survived, voters: live.length, killedBy: live.filter((v) => v.refuted).map((v) => v.why) }
            }),
          ),
        )
      : [],
)

// ---- The reduce is an EDGE: plain code, zero model tokens (R-GRAPH step 3).
const RANK = { critical: 0, high: 1, medium: 2, low: 3 }
const judged = perLens.filter(Boolean).flat().filter(Boolean)
const confirmed = judged
  .filter((f) => f.voters > 0 && f.survived * 2 > f.voters)
  .sort((a, b) => RANK[a.severity] - RANK[b.severity])
const killed = judged.filter((f) => !(f.voters > 0 && f.survived * 2 > f.voters))

log(`${judged.length} raw finding(s) → ${confirmed.length} survived, ${killed.length} refuted`)

if (!confirmed.length) {
  return { scope: scope.summary, lenses: LENSES.map((l) => l.key), confirmed: [], refuted: killed.length }
}

phase('Synthesize')
const report = await agent(
  `Write the review verdict for: ${scope.summary}\n\n` +
    `These findings survived adversarial verification (majority of skeptics failed to refute):\n` +
    JSON.stringify(confirmed, null, 2) +
    `\n\nRank by what bites first. For each: the defect in one sentence, the file:line, and the fix. ` +
    `Do not invent findings that are not in this list. State plainly which lenses were run and which were not.`,
  { label: 'synthesize', phase: 'Synthesize' },
)

return {
  scope: scope.summary,
  lenses: LENSES.map((l) => l.key),
  skepticsPerFinding: SKEPTICS,
  confirmed,
  refutedCount: killed.length,
  report,
}
