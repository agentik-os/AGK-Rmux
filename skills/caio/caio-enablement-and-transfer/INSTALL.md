# Install

1. Drop the `caio-enablement-and-transfer/` folder into your skills directory
   (e.g. `~/.omega/skills/`, `/mnt/skills/user/`, or your Claude Code skills path).
2. Trigger it: "AI adoption plan", "onboard the team on the system", "end-user training",
   "internal documentation pack", "knowledge transfer / handover", "autonomy readiness",
   "extension playbook", "add an agent / connect a tool / adjust a report", "nobody uses the
   system" — or in FR: "plan d'adoption IA", "former les utilisateurs", "documentation interne",
   "transfert de compétences", "passation", "playbook d'extension", "personne n'utilise le système".
3. Prerequisite (BLOCKING): the system must already be BUILT and WORKING (golden path green,
   runtime evidence). If not, run `caio-implementation-runbook` first — this skill refuses to
   train on a system that does not work (a leaky bucket).
4. One run produces `./caio-enablement/` with 8 deliverables (+ summary + metadata):
   Phase 3 (Adoption): onboarding plans, internal documentation pack, training curriculum,
   validated-use-cases log. Phase 4 (Transfer): extension playbook, ownership handover,
   the Autonomy-Readiness Gate, the adoption tracker.

Structure:
- `SKILL.md` ........ the operating protocol (boot, readiness gate, 6 phases, the demo-to-adoption
   arc, the three extension motions, the Autonomy-Readiness Gate, refusals, iron test)
- `references/` ..... 5 deep how-to references:
    `01-adoption-onboarding-playbook.md` ........ session design per audience + the demo-to-adoption arc
    `02-internal-documentation-standard.md` ..... what every system needs documented to be transferable
    `03-transfer-extension-curriculum.md` ....... add-agent / connect-tool / adjust-report, novice->guardian
    `04-change-management-and-messaging.md` ..... Kotter + ADKAR + Prosci + the mm-04 internal announcement
    `05-adoption-measurement-and-autonomy-gate.md` adoption NSM + retention curve + the Gate scoring rubric
- `assets/templates/` the 8 standardized client deliverables (fill-in `{{placeholders}}`):
    Onboarding-Session-Plan.md, Internal-Documentation-Pack.md, End-User-Training-Curriculum.md,
    Extension-Playbook.md, Ownership-Handover-Checklist.md, Autonomy-Readiness-Gate.md,
    Adoption-Tracker.md, + metadata.json (machine-readable header for caio-run-and-optimize)
- `platforms/` ..... claude.sh / codex.sh / gemini.sh adapters

Chain: reads the live system + docs from `caio-implementation-runbook`, the role inventory from
`caio-enterprise-workflow-architect`, AI-literacy from the `caio-discovery-interview` dossiers.
Hands a trained, autonomous client to `caio-run-and-optimize`. Delegates skill-codification to
`agentik-skill-forge` and novel-agent builds to `agentic-systems-builder`.
