# CAIO Enablement & Transfer

> Drive adoption, then transfer mastery — so the client's teams OWN and EXTEND the AI system instead of depending on the CAIO. Your teams are not spectators of the system; they are its operational guardians.

> Adoption is retention, not attendance. Transfer is ability, not a watched demo.

Built by [Agentik OS](https://agentik-os.com). The Phase 3 (Adoption) + Phase 4 (Transfer-to-autonomy) layer of the Chief AI Officer On Demand accompaniment chain. Composes with [caio-implementation-runbook](https://skills.agentik-os.com/caio-implementation-runbook), [caio-enterprise-workflow-architect](https://skills.agentik-os.com/caio-enterprise-workflow-architect), [caio-discovery-interview](https://skills.agentik-os.com/caio-discovery-interview), [caio-run-and-optimize](https://skills.agentik-os.com/caio-run-and-optimize), [agentik-skill-forge](https://skills.agentik-os.com/agentik-skill-forge), [agentic-systems-builder](https://skills.agentik-os.com/agentic-systems-builder), [creator-media-engine](https://skills.agentik-os.com/creator-media-engine).

---

## What it produces

`caio-enablement/` directory with 8 deliverables (+ summary + metadata):

**Phase 3 — Adoption**
1. **01-Onboarding-Session-Plans.md** :: per-audience session plans (C-Level / Manager / Operator / Technical owner), each adapted to the person's AI-literacy.
2. **02-Internal-Documentation-Pack.md** :: system map + how-each-dashboard-works + per-agent runbooks + commented/accessible code+config pointers + the evolution process. The thing that makes the system transferable instead of a black box.
3. **03-End-User-Training-Curriculum.md** :: the role-by-role training path (Knowledge), feeding the unaided first run (Ability).
4. **04-Validated-Use-Cases-Log.md** :: the log of first use cases proven in real conditions — the operator's own task done unaided, with evidence.

**Phase 4 — Transfer to autonomy**
5. **05-Extension-Playbook.md** :: how to ADD AN AGENT, CONNECT A NEW TOOL, ADJUST A REPORT — step-by-step, sized to the team's real technical level (config-only / can-edit-prompts / can-write-code).
6. **06-Ownership-Handover-Checklist.md** :: named owners per component, credential rotation (no CAIO-only keys), escalation path, the weekly guardian routine.
7. **07-Autonomy-Readiness-Gate.md** :: the objective gate — the team adds an agent / connects a tool / fixes a report UNAIDED, under real conditions, before transfer is called complete.
8. **08-Adoption-Tracker.md** :: who uses what, how often (by cohort) — the usage baseline that feeds the run phase.

Plus `00-Enablement-Summary.md` (1-page status the sponsor reads) and `metadata.json` (machine-readable header for `caio-run-and-optimize`).

## When to use

After `caio-implementation-runbook` has BUILT a working system (golden path green in production) and before `caio-run-and-optimize` measures ROI long-term. Use it to:
- onboard every role on a freshly built Company AI OS,
- train end-users until they actually use it for real work,
- document the system so it stops being a black box,
- teach the client's own team to extend it,
- prove autonomy with an objective gate,
- or rescue a system that shipped but nobody uses (`adoption-rescue` mode).

## The 5 enablement modes

| Mode | Duration | Output |
|---|---|---|
| `champion-enablement` | ~1 week | 1-3 internal champions trained to guardian level + teach-the-trainer kit |
| `role-onboarding` | 1-2 weeks | One role: onboarding + curriculum + >= 1 validated use case |
| `full-adoption-and-transfer` | 3-8 weeks | Complete `caio-enablement/` (8 files) + a passed Autonomy-Readiness Gate |
| `transfer-only` | 1-2 weeks | Extension Playbook + Ownership handover + the Gate (adoption already proven) |
| `adoption-rescue` | 1-3 weeks | Leaky-bucket diagnostic + re-onboarding + re-baselined tracker |

## The chain position

```
caio-ai-readiness-assessment   (pre-sign go/no-go)
   -> /market-proposal (signed SOW)
caio-discovery-interview        (Phase 1 — per-person dossiers, incl. AI-literacy ch.7)
caio-enterprise-workflow-architect (Phase 1 — company-ai-os/ blueprint + role inventory)
caio-implementation-runbook     (Phase 2 — build the system + internal docs)
caio-enablement-and-transfer    (Phase 3 adoption + Phase 4 transfer)   <-- THIS SKILL
caio-run-and-optimize           (Phase 5 — measure ROI, optimize, expand) -> loops to architect
```

**Reads:** the live system + internal docs from `caio-implementation-runbook`; the role/workflow inventory + dashboard/agent specs from `caio-enterprise-workflow-architect`; each person's AI-literacy/appetite from the `caio-discovery-interview` dossiers (chapter 7).

**Hands-to:** `caio-run-and-optimize` — a trained, autonomous client, with the Adoption-Tracker as the usage baseline and the Gate result as the self-extend signal.

**Delegates (never re-implements):** `agentik-skill-forge` (codify a repeatable company-specific skill, e.g. "monthly-close skill"), `agentic-systems-builder` (build a genuinely novel complex agent beyond the team's level), `creator-media-engine` (public case study, with consent).

## Composes with

`caio-implementation-runbook`, `caio-enterprise-workflow-architect`, `caio-discovery-interview`, `caio-run-and-optimize`, `agentik-skill-forge`, `agentic-systems-builder`, `creator-media-engine`.

## Doctrine grounding (Marketing Mastery)

- **mm-12 (novice to expert)** — the offer's "Internal Mastery" principle IS mm-12's novice->expert thesis. The transfer curriculum mirrors the sequence and the weekly routine, taking the team from spectator -> operator -> extender -> guardian, with judgment cultivated by the calibration loop.
- **mm-11 (measure / loops / retention)** — adoption-as-retention. An adopted, self-sufficient team is the anti-churn moat — the opposite of the dependency the offer refuses. The adoption NSM, the retention curve by cohort, the aha-moment, and "don't expand before the bucket holds" are applied internally.
- **mm-04 (messaging / copy / offer)** — used lightly, for the internal announcement only: reuse the discovery frame "we remove the tedious parts, not the person", channel a desire that already exists, speak to "you" not "us". The change-management load is carried by Kotter 8-step + ADKAR + Prosci, not mm-04.

## What it refuses

- Training on a system whose golden path is not green (leaky bucket).
- "Adoption" claimed from attendance / seats / demo enthusiasm.
- "Transfer complete" on a quiz or a watched demo (Knowledge is not Ability).
- Invented adoption numbers.
- CAIO-only credentials or a bus-factor-of-one left behind.
- Removing HITL on sensitive decisions in the name of "autonomy".
- An undocumented "magic" component the team can't touch.
- Re-implementing the build / codify layer (delegates it).

## Installation

```bash
bash <(curl -sL https://skills.agentik-os.com/install) caio-enablement-and-transfer
```

Then in Claude Code:

```
/caio-enablement-and-transfer
```

## Iron Test (90 days post-handover)

1. Do trained operators still use the system for real work (NSM > 0, cohort retention flat/up)?
2. Did the team complete all three extension motions UNAIDED, real conditions (the Gate)?
3. Did the team run the weekly guardian routine without the CAIO present?
4. Named owners + zero CAIO-only credentials (bus factor > 1)?
5. Did the Validated-Use-Cases log keep filling with real accepted runs?

4+ of 5 pass = enablement + transfer worked. 12-month test: the team adds a NEW agent the CAIO never specified and adoption holds = operational guardians, self-compounding, CAIO-independent.

## License

MIT.

---

*Version 1.0.0 :: adoption is retention, transfer is ability — operational guardians, not spectators.*
