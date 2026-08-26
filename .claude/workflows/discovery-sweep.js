export const meta = {
  name: 'discovery-sweep',
  description: 'Find everything of a given kind when you do not know how many there are — loop until two rounds turn up nothing new',
  whenToUse: 'Unknown-size discovery: bugs, dead code, missing auth checks, untested paths, doctrine violations. Use when a fixed "find the top 10" would silently miss the tail.',
  phases: [
    { title: 'Find', detail: 'multi-modal finders, one search angle each' },
    { title: 'Verify', detail: 'skeptics try to refute each new candidate' },
    { title: 'Report', detail: 'rank what survived' },
  ],
}

// args: a string ("missing auth checks in the API routes"), or
// {hunt, where, angles, maxRounds, dryStreak}.
const HUNT = typeof args === 'string' ? args : (args && args.hunt) || 'defects in this repository'
const WHERE = (args && args.where) || 'this repository'
const MAX_ROUNDS = (args && args.maxRounds) || 6
const DRY_TARGET = (args && args.dryStreak) || 2

// Multi-modal sweep: each finder is blind to what the others surface, because one
// search angle never finds everything (R-GRAPH step 0).
const ANGLES = (args && Array.isArray(args.angles) && args.angles.length
  ? args.angles
  : [
      'by container: walk the directory tree and inspect each module in turn',
      'by content: grep for the patterns and idioms this class of problem leaves behind',
      'by entry point: start at the public surface (routes, CLI, exported API) and follow inward',
      'by history: look at what changed recently and what the diffs left half-done',
    ])

const CANDIDATES = {
  type: 'object',
  properties: {
    items: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          title: { type: 'string' },
          file: { type: 'string' },
          line: { type: 'integer' },
          severity: { type: 'string', enum: ['critical', 'high', 'medium', 'low'] },
          evidence: { type: 'string', description: 'file:line and the quoted code that proves it' },
        },
        required: ['title', 'file', 'evidence'],
      },
    },
  },
  required: ['items'],
}

const VERDICT = {
  type: 'object',
  properties: { refuted: { type: 'boolean' }, why: { type: 'string' } },
  required: ['refuted', 'why'],
}

const key = (item) => `${(item.file || '').trim()}|${(item.title || '').trim().toLowerCase()}`

// seen holds EVERY candidate ever surfaced — including the ones the skeptics killed.
// Dedupe against `confirmed` instead and every rejected item comes back next round,
// the loop never runs dry, and it pays to rediscover the same dead ends (R-GRAPH step 7).
const seen = new Set()
const confirmed = []
let dry = 0
let round = 0

while (dry < DRY_TARGET && round < MAX_ROUNDS) {
  if (budget.total && budget.remaining() < 50000) {
    log(`stopping early: ${Math.round(budget.remaining() / 1000)}k tokens left, below the reserve`)
    break
  }
  round += 1
  phase('Find')

  const known = confirmed.length
    ? `\n\nAlready found, do NOT report these again:\n` + [...seen].slice(0, 120).join('\n')
    : ''

  const rounds = await parallel(
    ANGLES.map((angle, i) => () =>
      agent(
        `Hunt for: ${HUNT}\nIn: ${WHERE}\nSearch angle ${i + 1} (round ${round}) — ${angle}.\n` +
          `Report only what you can prove with file:line evidence. An empty list is a valid answer; ` +
          `do not pad it. Stay on your assigned angle.${known}`,
        { label: `find:r${round}:a${i + 1}`, phase: 'Find', schema: CANDIDATES },
      ),
    ),
  )

  const found = rounds.filter(Boolean).flatMap((r) => r.items || [])
  const fresh = []
  for (const item of found) {
    const k = key(item)
    if (seen.has(k)) continue
    seen.add(k)
    fresh.push(item)
  }

  if (!fresh.length) {
    dry += 1
    log(`round ${round}: nothing new (dry ${dry}/${DRY_TARGET})`)
    continue
  }
  dry = 0
  log(`round ${round}: ${fresh.length} new candidate(s), ${seen.size} seen in total`)

  const judged = await parallel(
    fresh.map((item) => () =>
      parallel(
        ['does-it-reproduce', 'is-the-evidence-real', 'is-it-already-guarded'].map((lens) => () =>
          agent(
            `Try to REFUTE this candidate through one lens: ${lens}.\n\n` +
              `Claim: ${item.title}\nAt: ${item.file}${item.line ? ':' + item.line : ''}\n` +
              `Evidence offered: ${item.evidence}\n\n` +
              `Read the real code. Refute it if it cannot happen, if the guard exists elsewhere, ` +
              `or if the evidence does not say what it claims. Uncertain means refuted.`,
            { label: `refute:${lens}`, phase: 'Verify', schema: VERDICT },
          ),
        ),
      ).then((votes) => {
        const live = votes.filter(Boolean)
        const survived = live.filter((v) => !v.refuted).length
        return { ...item, survived, voters: live.length }
      }),
    ),
  )

  confirmed.push(...judged.filter(Boolean).filter((v) => v.voters > 0 && v.survived * 2 > v.voters))
}

if (round >= MAX_ROUNDS && dry < DRY_TARGET) {
  log(`stopped at the ${MAX_ROUNDS}-round ceiling without going dry — coverage is NOT exhaustive`)
}

const RANK = { critical: 0, high: 1, medium: 2, low: 3 }
confirmed.sort((a, b) => (RANK[a.severity] ?? 9) - (RANK[b.severity] ?? 9))

if (!confirmed.length) {
  return { hunt: HUNT, rounds: round, seen: seen.size, confirmed: [], exhausted: dry >= DRY_TARGET }
}

phase('Report')
const report = await agent(
  `Write the findings report for the hunt: ${HUNT} (in ${WHERE}).\n\n` +
    `These survived adversarial verification:\n` +
    JSON.stringify(confirmed, null, 2) +
    `\n\nRank by what bites first, cite file:line for every claim, and invent nothing that is not ` +
    `in this list. State whether the sweep ran dry (${dry >= DRY_TARGET ? 'yes' : 'NO — it hit the round ceiling'}).`,
  { label: 'report', phase: 'Report' },
)

return {
  hunt: HUNT,
  rounds: round,
  seen: seen.size,
  exhausted: dry >= DRY_TARGET,
  confirmed,
  report,
}
