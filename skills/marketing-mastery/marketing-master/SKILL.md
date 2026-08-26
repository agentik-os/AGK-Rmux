---
name: marketing-master
description: "Run the full Marketing Mastery doctrine on a real project — a 12-Part gap-check that audits positioning, message, funnel, content, paid, pricing, partnerships, selling, retention against the doctrine and corrects the landing + strategy, then ships a 90-day plan. Also fires on EN: 'marketing master', 'run the marketing doctrine', 'full marketing audit', 'marketing gap-check', 'align my marketing', 'marketing mastery', '90-day marketing plan', 'audit my go-to-market'; and FR: 'marketing master', 'audit marketing complet', 'aligner mon marketing', 'gap-check marketing', 'doctrine marketing', 'plan marketing 90 jours', 'audite mon go-to-market'. Use to run all 12 mm-* doctrine skills as one orchestrated, self-correcting pass over a project."
metadata:
  version: 1.0.0
allowed-tools: ["Workflow", "Read", "Write", "Bash"]
---

# /marketing-master — The Marketing Mastery Doctrine Orchestrator

The **top of the marketing doctrine stack**. It runs all **12 Parts** of *Marketing Mastery* on a
**real project** as one gap-checked, self-correcting pass: load the live assets → audit each Part's
alignment against the doctrine → correct the landing + strategy → adversarially verify → loop until
dry → ship a 90-day plan. **Doctrine routes, tactical skills execute.** It runs **one Workflow at the
top level** and applies the 12 `mm-*` doctrines as **lenses** — an agent *reads* each `mm-XX SKILL.md`
as its rubric — because a Workflow sub-agent **cannot nest another Workflow**.

## When to use

**EN:** "marketing master", "run the marketing doctrine", "full marketing audit", "marketing
gap-check", "align my marketing", "90-day marketing plan", "audit my go-to-market".
**FR:** "audit marketing complet", "aligner mon marketing", "gap-check marketing", "doctrine
marketing", "plan marketing 90 jours", "audite mon go-to-market".

Use it for a **full-project** marketing audit + alignment + 90-day plan — when you want the whole
go-to-market measured against the doctrine in one pass, not a single tactic. Run it **AFTER**
`/omg-brand-identity` (the visual + brand direction exists) and `/product-marketing-context` (the
positioning / ICP / messaging dossier `.agents/product-marketing.md` exists). It is the **apex of the
go-to-market layer** in R-MARKETING.

**Do NOT use it for a single tactic.** Need ad copy → `/ads-copy`; a price → `/mk-pricing-strategy`; a
funnel → `/ads-funnel`; one Part's doctrine → the matching `/mm-XX`. `marketing-master` is the
orchestrator that runs *all twelve* — point single-tactic work at its home skill.

## The 12-Part chain (book order = the sequence law)

You run the Parts **in order** because each depends on the one before — the **sequence law** the whole
doctrine obeys:

> **Positionnement → Message → Un canal → Conversion → Mesure → Scaling.**

| # | Doctrine skill | Role in the chain |
|---|---|---|
| 1 | `mm-01-foundations-2026` | The 2026-2027 map: what changed (channel) vs what never changes (the buying mechanism); the 6-block system. |
| 2 | `mm-02-positioning-category` | Positioning & category (Dunford's 5 components); the bet that contaminates everything downstream. |
| 3 | `mm-03-why-people-buy` | Jobs-to-be-Done, the 4 forces of progress, Schwartz awareness levels, Cialdini. |
| 4 | `mm-04-messaging-copy-offer` | Value prop, copy frameworks (PAS/BAB/AIDA), Hormozi offer, the landing section-by-section. |
| 5 | `mm-05-funnel-channels` | TOFU/MOFU/BOFU, AARRR, owned/earned/paid, product-channel fit, the Bullseye. |
| 6 | `mm-06-content-seo-geo` | Content engine, topical authority, SEO intent, GEO (be cited by the AIs), distribution > production. |
| 7 | `mm-07-paid-ads` | Paid as multiplier; CAC/LTV; creative-is-targeting; systematic creative testing; cold→retargeting. |
| 8 | `mm-08-pricing-monetization` | Value-based pricing, packaging/tiering, price psychology, NRR; price = positioning. |
| 9 | `mm-09-partnerships-network-effects` | The 7 partnership forms, referral loops & viral k, borrowed audience, community as moat. |
| 10 | `mm-10-selling` | Sales-led vs PLG, founder-led sales, the 5-step B2B process, objections, follow-up. |
| 11 | `mm-11-measure-loops-retention` | Retention > monetization > acquisition, the North Star, growth loops, cohorts, the composing system. |
| 12 | `mm-12-novice-to-expert` | The sequence law, the **90-day plan**, the weekly routine, judgment & taste. Owns the plan this skill ships. |

Each Part is a **prerequisite of the next**: positioning precedes message (the message is the verbal
translation of the position), message precedes channel (a channel only amplifies), channel precedes
conversion (you can't optimize a rate on traffic that doesn't exist), conversion precedes serious
measurement, measurement precedes scaling. Auditing out of order produces aligned tactics on a broken
spine — so the gap-check walks Part 1 → 12.

## Inputs it loads (REAL project assets — L1: runtime is truth)

- **Prod URL → Playwright CLI sweep via Bash** (R-TEST + R-BROWSER: scripted Playwright through the
  Bash CLI, **NEVER** an MCP browser tool). Capture the landing **hero, sections, body copy, CTAs, and
  proof elements** — the page as it actually renders is the artifact every Part is measured against.
- **`.agents/product-marketing.md`** — positioning / ICP / audience / messaging, produced by
  `/product-marketing-context`. **If missing, stop and tell the user to run
  `/product-marketing-context` first** — the doctrine has nothing to align against without it.
- **The marketing dossier / strategy docs present in the repo** — any `docs/` go-to-market notes,
  pricing pages, existing `ADS-*.md` / `MARKET-*.md` outputs. Loaded as the current strategy of record.

The audit grades the **real, rendered project**, never an idealized description of it (L1). Code and
docs state intent; the running landing and the live dossier are the truth the gap-check scores.

## The protocol — load → gap-check (12 Parts) → correct → adversarially verify → loop → report

1. **Load.** Playwright-sweep the prod URL (hero/sections/copy/CTAs/proof) and Read
   `.agents/product-marketing.md` + the repo dossier into one project context. Guard: no
   `product-marketing.md` → abort with the instruction to run `/product-marketing-context`.
2. **Gap-check (12 Parts).** Each Part is gap-checked by an agent that **READS the matching
   `mm-XX SKILL.md` as its rubric** and scores the project's **real** alignment on that Part's
   dimensions — returning a strict schema of `{ part, aligned, score, gaps[], corrections[] }` with
   **concrete** gaps (cite the hero line, the missing proof, the absent price anchor) and proposed
   corrections. The agent **reads the doctrine, it never nests a Workflow.**
3. **Correct.** For each gap that carries a correction, an agent **applies or proposes** the fix to the
   landing + strategy — writing a single corrections artifact. It only edits **live files when in
   scope** (R-SCOPE: one writer per file); otherwise it proposes the diff for the operator.
4. **Adversarially verify.** Each correction is checked by **≥2-of-3 skeptics** (R-VERIFY): is the fix
   *faithful* to the doctrine and *real* against the rendered page (Popper — actively try to falsify
   it)? A correction the skeptics can't ratify is dropped back to a residual gap.
5. **Loop until dry.** Re-run the gap-check (bounded, K=2) until **no new gaps** surface or a hard
   blocker (missing secret, out-of-scope file) is hit — the self-correcting pass.
6. **Report.** Render a branded PDF via the OmegaOS pdfgen (**R-PDF**): per-Part alignment, gaps,
   corrections-verified, residual gaps, and the **mm-12 90-day plan**.

## Runnable Workflow script (copy-pasteable)

Run this with the **Workflow tool**. It is the whole pass: Load → GapCheck → Correct → Verify →
Report. **One Workflow at the top level**; the 12 `mm-*` doctrines are applied as **lenses** — each
audit agent `Read`s `skills/marketing-mastery/<skill>/SKILL.md` as its rubric. Honor *no nested
Workflow*: never launch a Workflow from inside an `agent()`. Keep it deterministic — no `Date.now()`,
no `Math.random()`, no shuffling; the Part order is the static `PARTS` array.

Set `PROD_URL` and `PROJECT_DIR` to the real project (or read them from `$ARGUMENTS`).

```javascript
// marketing-master — run all 12 Marketing Mastery Parts on a REAL project as one
// gap-checked, self-correcting pass. ONE Workflow at top level; mm-* are LENSES
// (an agent READS each mm-XX SKILL.md as its rubric — never a nested Workflow).

export const meta = {
  name: "marketing-master",
  description: "Full Marketing Mastery doctrine gap-check + self-correct + 90-day plan over a real project.",
  phases: [
    { title: "Load" },
    { title: "GapCheck" },
    { title: "Correct" },
    { title: "Verify" },
    { title: "Loop" },
    { title: "Report" },
  ],
};

const PROD_URL    = `<<<PASTE THE PROJECT'S PROD URL>>>`;
const PROJECT_DIR = `<<<PASTE THE ABSOLUTE PROJECT DIR>>>`;
const SKILLS_DIR  = `${PROJECT_DIR}/skills/marketing-mastery`; // mm-* live here
const ART_DIR     = `${PROJECT_DIR}/agentic/reports`;          // artifacts go here (R-ENV)

// The 12 Parts in BOOK ORDER = the sequence law. Order is the law; do not reorder.
const PARTS = [
  { n: 1,  skill: "mm-01-foundations-2026",            lens: "fondations / carte du système / GEO" },
  { n: 2,  skill: "mm-02-positioning-category",        lens: "positionnement & catégorie (Dunford)" },
  { n: 3,  skill: "mm-03-why-people-buy",              lens: "JTBD / 4 forces / Schwartz / Cialdini" },
  { n: 4,  skill: "mm-04-messaging-copy-offer",        lens: "message / copy / offre / landing" },
  { n: 5,  skill: "mm-05-funnel-channels",             lens: "funnel TOFU/MOFU/BOFU / AARRR / canaux" },
  { n: 6,  skill: "mm-06-content-seo-geo",             lens: "content engine / SEO / GEO" },
  { n: 7,  skill: "mm-07-paid-ads",                    lens: "paid / CAC-LTV / créatif testing" },
  { n: 8,  skill: "mm-08-pricing-monetization",        lens: "pricing / packaging / NRR" },
  { n: 9,  skill: "mm-09-partnerships-network-effects", lens: "partenariats / referral loops / moat" },
  { n: 10, skill: "mm-10-selling",                     lens: "selling / PLG vs sales-led / closing" },
  { n: 11, skill: "mm-11-measure-loops-retention",     lens: "mesure / growth loops / rétention" },
  { n: 12, skill: "mm-12-novice-to-expert",            lens: "séquence / plan 90 jours / jugement" },
];

// ---- Phase 1: LOAD — the REAL project (L1: runtime is truth) ----------------
// Filesystem + Playwright + shell run INSIDE an agent() (a sub-agent owns Bash/Read/Write).
// The top-level script only orchestrates the declared primitives (agent/parallel/pipeline/meta) —
// it never calls a bare bash()/read()/write() global. The agent does the scripted Playwright sweep
// through the Bash CLI (R-TEST + R-BROWSER) — NEVER an MCP browser tool — and returns the context.
const projectContext = (await agent(
  `You are the LOADER for a Marketing Mastery audit. Assemble the REAL project context. Do EXACTLY:
1. GUARD FIRST — if ${PROJECT_DIR}/.agents/product-marketing.md does NOT exist, return ONLY this one line:
   BLOCKED: .agents/product-marketing.md is missing. Run /product-marketing-context first.
2. Playwright-sweep the prod URL through the Bash CLI (R-TEST + R-BROWSER — scripted Playwright via Bash,
   NEVER an MCP browser tool). Run via your Bash tool:
   cd ${PROJECT_DIR} && ( bun run scripts/pw-capture.ts ${PROD_URL} || ( npx --yes playwright screenshot --full-page ${PROD_URL} ${ART_DIR}/landing.png && npx --yes playwright pdf ${PROD_URL} ${ART_DIR}/landing.pdf ) )
   Capture the rendered hero, sections, body copy, CTAs and proof elements.
3. Read ${PROJECT_DIR}/.agents/product-marketing.md (positioning / ICP / messaging) IN FULL.
4. Read the marketing dossier of record via Bash:
   cd ${PROJECT_DIR} && cat docs/*market*.md docs/*gtm*.md ADS-*.md 2>/dev/null | head -c 40000
Return ONE assembled block, verbatim section contents, NOTHING else:
# PROJECT CONTEXT (real, runtime)
## Prod URL
${PROD_URL}
## Landing (Playwright sweep — hero / sections / copy / CTAs / proof)
<what the page actually renders>
## Positioning / ICP / messaging (.agents/product-marketing.md)
<file contents>
## Marketing dossier / strategy of record
<dossier contents>`,
  { model: "opus" }
)).trim();

// Guard: the dossier MUST exist (produced by /product-marketing-context) — abort cleanly otherwise.
if (projectContext.startsWith("BLOCKED")) {
  return projectContext;
}

// ---- Phase 2: GAPCHECK — 12 Parts, each mm-XX SKILL.md READ as the rubric ----
// pipeline: per Part, an audit agent reads the doctrine and scores the REAL project.
const auditPart = (part) => `You are the Marketing Mastery auditor for PARTIE ${part.n} (${part.lens}).
STEP 1 — Read the doctrine as your rubric: read the file ${SKILLS_DIR}/${part.skill}/SKILL.md IN FULL.
That file's dimensions ARE your scoring rubric — you do NOT invent criteria.
STEP 2 — Audit the REAL project below against that rubric ONLY for this Part's scope. Cite concrete
evidence (the exact hero line, a missing proof block, an absent price anchor, the CTA text).
STEP 3 — Return STRICT JSON, nothing else:
{ "part": ${part.n}, "skill": "${part.skill}", "aligned": <bool>, "score": <0-100>,
  "gaps": [ { "what": "...", "evidence": "<cited from the project>", "severity": "high|med|low" } ],
  "corrections": [ { "target": "landing|strategy", "change": "<concrete, applyable edit>" } ] }
Do NOT nest a Workflow. Read the doctrine; judge the project; return JSON.

PROJECT CONTEXT:
${projectContext}`;

const rawAudits = await pipeline(
  PARTS,
  (part) => agent(auditPart(part), { model: "opus" })
);
const audits = rawAudits.map((r, i) => {
  try { return JSON.parse(r.slice(r.indexOf("{"), r.lastIndexOf("}") + 1)); }
  catch { return { part: PARTS[i].n, skill: PARTS[i].skill, aligned: false, score: 0,
                   gaps: [{ what: "audit JSON unparseable", evidence: r.slice(0, 280), severity: "high" }],
                   corrections: [] }; }
});

// ---- Phase 3: CORRECT — apply/propose each fix to landing + strategy ---------
const openGaps = audits.flatMap((a) =>
  (a.corrections || []).map((c) => ({ part: a.part, skill: a.skill, ...c }))
);
const correctionArtifacts = await parallel(
  openGaps.map((g) => agent(
    `You correct the marketing for PARTIE ${g.part} (${g.skill}). Apply this doctrine-faithful fix to
the ${g.target}: "${g.change}". If the target file is IN YOUR SCOPE, edit it (R-SCOPE: one writer per
file) and report the diff. Otherwise PROPOSE the exact diff/copy for the operator. Be concrete and
faithful to the doctrine rubric — no invented proof, no fake scarcity (honesty gate).

PROJECT CONTEXT:
${projectContext}`,
    { model: "opus" }
  ))
);
const correctionsDoc = openGaps
  .map((g, i) => `## Partie ${g.part} — ${g.skill}\n**Fix:** ${g.change}\n\n${correctionArtifacts[i]}`)
  .join("\n\n");
await agent(
  `Persist the corrections artifact. Via your Bash + Write tools: run \`mkdir -p ${ART_DIR}\`, then write
the CONTENT below VERBATIM to ${ART_DIR}/marketing-master-corrections.md, then reply DONE.

CONTENT:
${correctionsDoc}`,
  { model: "claude-haiku-4-5" }
);

// ---- Phase 4: VERIFY — ≥2-of-3 skeptics per correction (R-VERIFY, Popper) ----
const verifyOne = (g, body) => (reviewer) => agent(
  `You are skeptic #${reviewer} verifying a marketing correction. Try to FALSIFY it (Popper): is it
FAITHFUL to the PARTIE ${g.part} doctrine AND REAL against the rendered landing? Answer strict JSON
{ "ratified": <bool>, "reason": "<one line>" }.

CORRECTION:
${body}

PROJECT CONTEXT:
${projectContext}`,
  { model: reviewer === 1 ? "opus" : reviewer === 2 ? "claude-sonnet-4-6" : "claude-haiku-4-5" }
);
const verdicts = await parallel(openGaps.map(async (g, i) => {
  const panel = await parallel([1, 2, 3].map(verifyOne(g, correctionArtifacts[i])));
  const yes = panel.filter((p) => /"ratified"\s*:\s*true/.test(p)).length;
  return { part: g.part, change: g.change, ratified: yes >= 2, votes: `${yes}/3` };
}));

// ---- Phase 5: LOOP — re-run gapcheck until dry (bounded K=2) -----------------
const K = 2;
let residual = audits.flatMap((a) => (a.gaps || []).map((x) => ({ part: a.part, ...x })));
for (let k = 0; k < K; k++) {
  // Re-audit only the parts that were not aligned. NOTE: corrections are PROPOSED to an artifact
  // (not necessarily applied to the live page), so the re-audit reflects the proposed end-state.
  const reAuditParts = PARTS.filter((p) => audits.find((a) => a.part === p.n && !a.aligned));
  const reAudit = await pipeline(
    reAuditParts,
    (part) => agent(auditPart(part) + `\n\nNOTE: corrections were applied/proposed; re-score and list ONLY the residual gaps that REMAIN open for this Part.`, { model: "opus" })
      .then((raw) => ({ part: part.n, raw }))
  );
  const fresh = reAudit.flatMap(({ part, raw }) => {
    try { return (JSON.parse(raw.slice(raw.indexOf("{"), raw.lastIndexOf("}") + 1)).gaps || []).map((g) => ({ part, ...g })); }
    catch { return [{ part, what: "re-audit JSON unparseable", evidence: raw.slice(0, 200), severity: "high" }]; }
  });
  // Replace ONLY the re-audited parts' residual gaps; keep gaps from parts not re-audited (L4: never silently drop).
  const reAuditedPartNums = new Set(reAuditParts.map((p) => p.n));
  residual = residual.filter((g) => !reAuditedPartNums.has(g.part)).concat(fresh);
  if (fresh.length === 0) break;        // dry
}

// ---- Phase 6: REPORT — branded PDF via the OmegaOS pdfgen (R-PDF) ------------
const plan90 = await agent(
  `Read ${SKILLS_DIR}/mm-12-novice-to-expert/SKILL.md as your rubric and write the project's concrete
90-DAY PLAN (Mois 1 Fondations / Mois 2 Un canal + conversion / Mois 3 Mesure + premier scaling),
grounded in the audit findings below. Return tight markdown.

AUDITS: ${JSON.stringify(audits)}`,
  { model: "opus" }
);

// --template=doc expects a single markdown `body` string (DocData), NOT a sections[] array.
const reportBody = [
  `## Per-Part alignment\n\n` + audits.map((a) =>
    `- Partie ${a.part} — ${a.skill}: ${a.aligned ? "ALIGNED" : "GAP"} (${a.score}/100)`).join("\n"),
  `## Gaps found\n\n` + (audits.flatMap((a) =>
    (a.gaps || []).map((g) => `- P${a.part} [${g.severity}] ${g.what} — ${g.evidence}`)).join("\n") || "Aucun."),
  `## Corrections verified\n\n` + (verdicts.map((v) =>
    `- P${v.part} ${v.ratified ? "RATIFIED" : "REJECTED"} (${v.votes}) — ${v.change}`).join("\n") || "Aucune."),
  `## Residual gaps\n\n` + (residual.map((r) => `- P${r.part} — ${r.what}`).join("\n") || "Aucun."),
  `## 90-day plan (mm-12)\n\n` + plan90,
].join("\n\n");
const reportData = {
  template: "doc",
  title: "Marketing Mastery — Alignment Report",
  subtitle: PROD_URL,
  body: reportBody,
};
const dataPath = `${ART_DIR}/marketing-master-report.json`;
const outPath  = `${ART_DIR}/marketing-master-report.pdf`;
await agent(
  `Render the alignment report PDF. Via your Bash + Write tools, do EXACTLY:
1. \`mkdir -p ${ART_DIR}\` (Bash).
2. Write the REPORT JSON below VERBATIM to ${dataPath} (Write tool).
3. Render via the OmegaOS pdfgen (R-PDF — NEVER hand-roll a generator) with Bash:
   \`omega pdf --template=doc --data=${dataPath} --out=${outPath}\`
4. Reply with: RENDERED ${outPath}

REPORT JSON:
${JSON.stringify(reportData, null, 2)}`,
  { model: "claude-haiku-4-5" }
);

return `Marketing Mastery alignment complete. PDF: ${outPath}\n` +
       audits.map((a) => `P${a.part} ${a.aligned ? "✅" : "⚠️"} ${a.score}/100`).join("  ");
```

**Notes for the running agent**
- `agent()`, `parallel()`, `pipeline()`, `meta()` are the Workflow primitives — in-process sub-agents on
  the current session. **One Workflow at the top level; never nest one inside an `agent()`** — the
  doctrine is applied by `Read`-ing each `mm-XX SKILL.md`, not by spawning a Workflow per Part.
- **Filesystem / shell / Playwright / `omega pdf` run INSIDE an `agent()`** — a sub-agent owns the
  `Bash` / `Read` / `Write` tools. The top-level script orchestrates only the four primitives above; it
  never calls a bare `bash()` / `read()` / `write()` global (those are not Workflow primitives).
- The Playwright sweep goes **through the Bash CLI** (R-TEST + R-BROWSER) — never an MCP browser tool.
- Every PDF goes through **`omega pdf`** (R-PDF) — never a hand-rolled generator.
- Keep it deterministic: no `Date.now()`, no `Math.random()`, no shuffle. The `PARTS` order is the law.
- Respect **R-SCOPE**: only edit a file when it's in your scope; otherwise propose the diff.

## Output — the alignment report (omega pdf --template=doc)

The rendered PDF (R-PDF, `--template=doc`) contains:

- **Per-Part alignment verdict + score** — Partie 1 → 12, each `ALIGNED` / `GAP` with a 0-100 score.
- **Gaps found** — every concrete, evidence-cited gap (the hero line, the missing proof, the absent
  price anchor), tagged by severity.
- **Corrections applied + adversarially verified** — each fix with its **≥2-of-3** skeptic verdict
  (`RATIFIED` / `REJECTED`, vote count).
- **Residual gaps** — what the loop-until-dry pass could not close (or a hard blocker), recorded
  explicitly (L4: never silently dropped).
- **The mm-12 90-day plan** — Mois 1 Fondations → Mois 2 Un canal + conversion → Mois 3 Mesure +
  premier scaling, grounded in the audit findings.

## Notes

- **Doctrine routes, tactics produce.** This orchestrator **never re-implements a tactical skill** — it
  reads the 12 `mm-*` doctrines as lenses and routes corrections to their executing homes
  (`mk-*`, `ads-*`, `ag-seo-*`, `market-*`, …) per the alignment matrix.
- **SSOT:** `docs/marketing-mastery-alignment.md` is the single source of truth for doctrine↔execution
  mapping and the book-order chain; keep this skill's Part list in sync with it.
- **Shape:** mirrors `/llm-council`'s embedded-Workflow pattern — frontmatter with `Workflow` in
  `allowed-tools`, a runnable top-level Workflow script, and the *no-nested-Workflow* constraint made
  explicit (lenses via `Read`, not nested engines).
- **Placement (R-MARKETING):** the **apex of the go-to-market layer**, run after `/omg-brand-identity`
  and `/product-marketing-context`. It is the keystone that runs the entire doctrine in one pass.

--- **Resume:** `/marketing-master` orchestre les 12 Parties de Marketing Mastery sur un projet réel en
une passe gap-check auto-corrigée — un seul Workflow au niveau racine, les `mm-*` lus comme grilles
(jamais de Workflow imbriqué), et un rapport PDF avec le plan 90 jours.
