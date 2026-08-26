# Install

1. Drop the `caio-ai-readiness-assessment/` folder into your skills directory
   (e.g. `~/.omega/skills/` for OmegaOS, `/mnt/skills/user/`, or your Claude Code skills path).
2. Trigger it: "AI readiness assessment", "qualify this client", "go/no-go", "is this company
   AI-ready", "should we take this client", "discovery call" — or in French "évaluation de maturité IA",
   "qualifier ce client", "diagnostic avant pitch", "doit-on prendre ce client".
3. One run = one lead = one `./caio-readiness/` folder (5 artefacts), produced at qualification altitude
   from three inputs only: company context + the qualification call + a public-site scan.
4. The verdict routes the lead:
   - **GO** → hand `Recommended-Engagement.md` to `/market-proposal` for the signed SOW; the engagement
     then begins with `caio-discovery-interview` → `caio-enterprise-workflow-architect`.
   - **NOT-YET** → give the company `Gap-To-Target-Plan.md` + a re-qualify trigger; re-run later in
     `re-qualification` mode.
   - **REDIRECT** → point them to the named alternative (a point SaaS / a data engineer / an internal
     hire / a compliance partner / a single agent via `agentic-systems-builder`).

Structure:
- `SKILL.md` ............ the operating protocol (fence, iron laws, doctrine, boot, 9-dim model, decision tree, pricing, refusals, iron test)
- `references/01-readiness-maturity-rubric.md` ...... the 9-dimension 0-4 rubric with evidence anchors
- `references/02-scoring-and-readiness-index.md` .... weights + scoring method + a full worked example
- `references/03-go-no-go-decision-tree.md` ......... the decision tree, disqualification rules, 4-forces scoring, objection bank
- `references/04-engagement-shaping-and-pricing.md` . engagement shape + indicative pricing on the real grid
- `references/05-qualification-call-protocol.md` .... the 30-minute discovery-call script (diagnostic before pitch)
- `assets/templates/` .. the 5 fixed deliverables (Scorecard, Go-No-Go-Brief, Recommended-Engagement, Gap-To-Target-Plan, metadata.json)
- `platforms/{claude,codex,gemini}.sh` ... platform activation adapters

**The fence:** this is the pre-sign commercial qualification gate, NOT the technical audit. It defers all
detailed technical mapping to E2 (`caio-enterprise-workflow-architect`) and estimates indicative
investment (a price), never ROI (a return).
