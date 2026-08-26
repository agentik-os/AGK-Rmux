export const meta = {
  name: 'diamond',
  description: 'The workhorse topology, parameterised — fan out over a work list, reduce in code, synthesize once',
  whenToUse: 'Any breadth-first sweep where the units are known up front: one agent per file, per route, per source, per competitor, per document. Pass the list in args; the shape does the rest.',
  phases: [
    { title: 'Work', detail: 'one agent per unit, in parallel' },
    { title: 'Synthesize', detail: 'one agent sees the reduced set' },
  ],
}

// args: {units: [...], task: "what each agent does with its unit", question: "what the
//        synthesis must answer", verify?: true, model?: 'haiku', effort?: 'low'}
// units may be strings or {id, brief} objects.
const UNITS = (args && Array.isArray(args.units) && args.units) || (Array.isArray(args) ? args : [])
const TASK = (args && args.task) || 'Analyse this unit and report what matters about it.'
const QUESTION = (args && args.question) || 'Synthesize the unit reports into one answer.'

if (!UNITS.length) {
  log('no units passed — nothing to fan out over. Pass args: {units: [...], task, question}')
  return { units: 0, error: 'no units' }
}

const label = (u, i) => (typeof u === 'string' ? u : u.id || u.name || `unit-${i + 1}`)
const brief = (u) => (typeof u === 'string' ? u : u.brief || JSON.stringify(u))

const UNIT_REPORT = {
  type: 'object',
  properties: {
    unit: { type: 'string' },
    summary: { type: 'string' },
    points: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          claim: { type: 'string' },
          evidence: { type: 'string', description: 'file:line, URL, or quoted source — never an unsupported assertion' },
          weight: { type: 'string', enum: ['high', 'medium', 'low'] },
        },
        required: ['claim', 'evidence'],
      },
    },
  },
  required: ['unit', 'summary', 'points'],
}

const VERDICT = {
  type: 'object',
  properties: { refuted: { type: 'boolean' }, why: { type: 'string' } },
  required: ['refuted', 'why'],
}

// Per-node tiering: the fan-out is bounded and repetitive, the merge carries the
// judgment. Only override when the caller asked for it (R-GRAPH step 8, R-MODEL).
const nodeOpts = { phase: 'Work', schema: UNIT_REPORT }
if (args && args.model) nodeOpts.model = args.model
if (args && args.effort) nodeOpts.effort = args.effort

log(`${UNITS.length} unit(s) → fan out${args && args.verify ? ' → verify' : ''} → synthesize`)

phase('Work')

// A pipeline, not a barrier: when verification is on, a unit's claims are checked the
// moment that unit lands instead of waiting for the slowest one (R-GRAPH step 1).
const results = await pipeline(
  UNITS,
  (u, _orig, i) =>
    agent(`${TASK}\n\nYour unit: ${brief(u)}\n\nStay inside this unit. Cite evidence for every claim. An empty result is a valid answer.`, {
      ...nodeOpts,
      label: `work:${label(u, i)}`,
    }),
  (report, u, i) => {
    if (!report || !args || !args.verify) return report
    const points = report.points || []
    if (!points.length) return report
    return parallel(
      points.map((p) => () =>
        agent(
          `Try to REFUTE this claim about ${label(u, i)}.\n\nClaim: ${p.claim}\nEvidence offered: ${p.evidence}\n\n` +
            `Check the real source. Refute it if the evidence does not support it. Uncertain means refuted.`,
          { label: `refute:${label(u, i)}`, phase: 'Verify', schema: VERDICT },
        ).then((v) => ({ ...p, refuted: v ? v.refuted : true, why: v ? v.why : 'verifier failed' })),
      ),
    ).then((checked) => ({ ...report, points: checked.filter((p) => !p.refuted), dropped: checked.filter((p) => p.refuted).length }))
  },
)

// The reduce is an edge: plain code, zero model tokens (R-GRAPH step 3).
const reports = results.filter(Boolean)
const missing = UNITS.length - reports.length
const points = reports.flatMap((r) => (r.points || []).map((p) => ({ ...p, unit: r.unit })))
const dropped = reports.reduce((n, r) => n + (r.dropped || 0), 0)

if (missing) log(`${missing} unit(s) returned nothing — the synthesis below does NOT cover them`)
if (dropped) log(`${dropped} claim(s) refuted and removed before synthesis`)

phase('Synthesize')
const answer = await agent(
  `${QUESTION}\n\n` +
    `You are the merge node of a fan-out over ${UNITS.length} unit(s); ${reports.length} reported back.\n\n` +
    JSON.stringify(reports, null, 2) +
    `\n\nUse only what is above. Do not invent a unit that did not report. ` +
    (missing ? `State explicitly that ${missing} unit(s) are missing from this answer.` : ''),
  { label: 'synthesize', phase: 'Synthesize' },
)

return { units: UNITS.length, reported: reports.length, missing, claims: points.length, refuted: dropped, answer }
