# 03 — The Transfer & Extension Curriculum (Novice -> Guardian)

Phases 5-6 of the skill. Outputs `caio-enablement/05-Extension-Playbook.md` and `06-Ownership-Handover-Checklist.md` (the Gate itself is detailed in `05-adoption-measurement-and-autonomy-gate.md`).

This is the **headline of the offer** — principle #3, Internal Mastery, made operational. It carries EQUAL depth to the Phase-3 adoption material; the mastery-transfer half is never a thin afterthought. Its spine is mm-12 (novice -> expert): a sequenced curriculum, a weekly routine, and a path from competence to the judgment that makes a true guardian.

> Adoption makes them users. Transfer makes them owners. The engagement is not done at "they use it" — it is done at "they extend it without you."

---

## A. First principle — the sequence is the lever (mm-12)

mm-12's core claim: the #1 reason a technical founder stalls is not effort, it's the wrong **order of attack**. The same is true for a client team becoming guardians. You do not hand them the whole system and say "you own it now". You sequence them through levels, each a prerequisite for the next, and inverting the order destroys the value.

The transfer sequence (the internal analogue of mm-12's Positioning -> Message -> Channel -> Conversion -> Measure -> Scaling):

```
Level 0  SPECTATOR  :: watched the demo, doesn't trust it yet.            (pre-adoption)
Level 1  OPERATOR   :: uses the system for their own real work, daily.    (Phase 3 goal)
Level 2  READER     :: reads the dashboard, traces a number, knows WHY a result is what it is.
Level 3  EXTENDER   :: performs the three motions (add agent / connect tool / adjust report) — guided.
Level 4  GUARDIAN   :: performs the three motions UNAIDED, and has the JUDGMENT to know WHEN to.
```

Mechanism of the dependency (why the order can't be inverted):
- **Operator precedes Reader**: you can't reason about a dashboard for a workflow you've never run. The number is abstract until you've lived the task behind it.
- **Reader precedes Extender**: you can't safely change a report or an agent whose output you can't interpret. Changing a metric you can't read is changing a button you can't see.
- **Extender precedes Guardian**: the unaided motion (Level 4) is only meaningful after the guided motion (Level 3). And **guardianship is judgment, not motions** — knowing *when* to add an agent vs. when a config change suffices, sensing that an extension will misbehave before shipping it. That judgment is cultivated, not taught (§D).

> Never transfer a level the team hasn't earned the prerequisite for. Skipping to "you own it" before they can read the dashboard hands them a system they'll quietly stop touching.

---

## B. The three extension motions, sized to internal technical level

The whitepaper names exactly three things a self-sufficient team must do: **add an agent, connect a tool, adjust a report**. The Extension Playbook teaches each — and the single most important design rule is that the path is **sized to the team's real technical level** (Iron Law 9), read from the discovery dossiers + the constraint snapshot. The same motion looks completely different for a config-only operator and an internal engineer.

### Motion A — ADD AN AGENT
```
config-only team:
  - Clone an existing agent template via a guided form (pick the source, the task, the HITL approver).
  - Ship to staging. Run the template's built-in acceptance check.
  - You never write code; you assemble from proven parts.
can-edit-prompts:
  - + Adjust the cloned agent's system prompt, its tools, and its guardrails/refusals.
  - + Read the logs to confirm behaviour matches intent (L1 — verify at runtime, not by reading the prompt).
can-write-code:
  - + Follow the runbook's add-agent path end to end (new agent, not a clone).
  - + Escalate a GENUINELY NOVEL complex agent to agentic-systems-builder. Teach the escalation
    as a normal move, not a failure — a guardian knows the boundary of their level and routes past it.
```

### Motion B — CONNECT A NEW TOOL
```
config-only:
  - A guided OAuth / Composio connection. READ-ONLY first, always. Least-privilege scopes.
  - Confirm it appears in the dashboard logs (so it's observable, not invisible).
technical:
  - + A direct API / MCP integration via the runbook's integration pattern.
Always (every level): least-privilege, read-only before write, appears in logs, rollback known,
  secret stored in the client's vault (never the repo, never a doc — R-ENV).
```

### Motion C — ADJUST A REPORT
```
config-only:
  - Change a metric, a threshold, a label, or a schedule via the dashboard config UI.
  - VERIFY the new number against runtime — never trust the new label (L1). A report that says
    "247" must be provably 247, not "the field is now called revenue".
technical:
  - + Edit the report's query / aggregation; re-run; diff against the known-good reference value
    from the documentation pack (02 §B.2).
```

For each motion, the Extension Playbook ships: the exact step-by-step at the team's level, the safety rails (staging, least-privilege, HITL re-check, rollback), the pointer into the code/config docs, and the acceptance check that says "this worked". A motion without an acceptance check is a change you can't verify — refused.

---

## C. The curriculum — guided, then unaided

The teaching pattern for each motion, mirroring the demo-to-adoption arc one level up (mm-12 — competence comes from concentrated repetition, not from reading):

```
1. WATCH    :: the CAIO performs the motion once, narrating the WHY at each step (not just the clicks).
2. GUIDED   :: the client owner performs it with the CAIO beside them. Owner drives; CAIO advises.
3. UNAIDED  :: the client owner performs a DIFFERENT real instance alone, from the docs only.
               This is the Autonomy-Readiness Gate (05 §B). If they had to ask HOW, it doesn't pass
               — and the fix is usually the DOCS, not more coaching (legibility failed, ref 02 §A).
4. TEACH    :: the owner teaches the next person the motion. Teaching is the proof of mastery and it
               seeds the volunteer army that self-replicates after you leave (mm-11 — compounding).
```

The unaided step (3) is non-negotiable and must be on a **real** instance under **real** conditions, not a tutorial sandbox. mm-12's distinction is exact: competence is executing the motion; expertise (guardianship) is the judgment that only the calibration loop builds — and the loop only runs on real consequences.

---

## D. From extender to guardian — cultivating judgment (the mm-12 heart)

This is the doctrinal core of the transfer, and no checklist can replace it (just as mm-12's "from competent to expert" is the one part no executing skill embodies — it is cultivated by repetition).

**Competence** = performing the three motions. **Guardianship** = the judgment to know:
- *when* to add an agent vs. when a simple automation or a config change suffices (don't build an agent for an if-this-then-that — the architect's Iron Law 2, inherited),
- *when* an extension will misbehave before shipping it (sensing a bad change the way mm-12's expert senses a flat hook),
- *when* a request is actually a Class-8 refusal (a sensitive decision no agent should make autonomously) and must stay human,
- *when* to escalate to `agentic-systems-builder` rather than force it at their level.

The mechanism of judgment is the **calibration loop** (mm-12): the guardian extends -> the runtime contradicts or confirms (the agent fails, the report misleads, or it works) -> they adjust their intuition -> the next extension is better. Without the repeated loop on real consequences, intuition stays opinion; with it, it becomes calibrated judgment. This is why the Gate demands a real unaided extension, not a passed quiz: you cannot certify judgment from a tutorial.

You cultivate it deliberately:
- Have the guardian make the *call* (add agent? config change? refuse?) before acting, write down the prediction, then check it against runtime. The gap between prediction and result is where judgment grows.
- Let them feel a small failure safely (in staging) — a recovered mistake teaches more than a guided success.
- Point them at excellence (the existing well-built agents, the documentation standard) so they develop taste for what "good" looks like.

---

## E. The weekly guardian routine (mm-12's weekly routine, internalized)

mm-12 prescribes a non-negotiable weekly marketing routine; the transfer installs its internal analogue — the cadence the client team runs **themselves after the CAIO leaves**. This is what converts a one-time handover into a living capability and what keeps the bucket from leaking (mm-11 — reinforcement sustains retention).

```
WEEKLY GUARDIAN ROUTINE (~1-2h, the team owns it):
- Monday (30 min) — READ THE NUMBERS. The one dashboard view: adoption (are operators still using
  it?), agent health (runs, costs, errors, confidence), the week's biggest friction. Decide ONE
  improvement. (mm-12 — read the numbers, pick one action; mm-11 — watch the retention curve.)
- One improvement session (30-60 min) — make the ONE change via the evolution process (staging ->
  acceptance check -> ship -> update the docs). One change at a time, so you know what moved (L1).
- Friday (15 min) — confirm the change held at runtime; log it. Note anything to escalate.
```

This routine is exactly the handoff surface to `caio-run-and-optimize`'s "1h/week + expand" loop — you install the habit; the run phase measures and scales it. The keep/delegate/automate split (mm-12) tells the team what this routine covers:
- **Keep (the team, judgment):** the decision on the numbers, the HITL approvals, the call on what to extend, the Class-8 refusals.
- **Automate:** the measurement (the dashboard already does it), the agent runs themselves, the reporting.
- **Delegate:** a novel complex agent -> `agentic-systems-builder`; a repeatable company-specific process worth codifying -> `agentik-skill-forge`. Delegating a capability you haven't yet mastered is paying for mediocrity you can't evaluate — so the team delegates *up* (genuinely beyond their level), not *away from learning*.

---

## F. The ownership handover (no bus factor of one)

`06-Ownership-Handover-Checklist.md` makes ownership concrete and removes every single point of failure — including you (Iron Law 8). Transfer is not complete while only the CAIO can fix something.

```
- Named owner per component (every agent, dashboard, integration has a person).      [DONE/owner]
- Backup owner per critical component (bus factor >= 2 on anything that hurts if it breaks).
- ZERO CAIO-only credentials: every key/token rotated to the client's vault; the CAIO's access
  revoked or reduced to advisory (R-ENV — secrets in the client's store, never the CAIO's).
- Escalation path documented (what to handle in-house, what goes to agentic-systems-builder, when).
- The evolution process (ref 02 §B.5) adopted and run at least once by the team.
- The weekly guardian routine (§E) run at least twice WITHOUT the CAIO present.
- HITL ownership: for every sensitive decision, the named human approver is a client employee,
  not the CAIO (Iron Law 7 — autonomy is ownership, not unsupervised agents).
- The Validated-Use-Cases log + Adoption-Tracker handed to the team and to caio-run-and-optimize.
```

The credential rotation is the hardest and most-skipped step, and the most important: a system where the consultant still holds the keys is, by definition, not transferred. If you can still log in as the only admin, the handover is theatre.

---

## G. Discipline checks for this phase

| Check | Pass = |
|---|---|
| The curriculum follows the level sequence (Operator -> Reader -> Extender -> Guardian); no skips | yes |
| Each of the three motions has a path sized to the team's actual technical level | yes |
| Each motion ships safety rails (staging, least-privilege, HITL re-check, rollback) + an acceptance check | yes |
| Each motion taught WATCH -> GUIDED -> UNAIDED -> TEACH, unaided on a REAL instance | yes |
| Judgment cultivated via the calibration loop (predict -> runtime -> adjust), not just motions | yes |
| The weekly guardian routine installed + run >= twice without the CAIO | yes |
| Named owner (+ backup on critical) per component; zero CAIO-only credentials | yes |
| HITL approvers are client employees on every sensitive decision | yes |
| Escalation to agentic-systems-builder / agentik-skill-forge taught as a normal move, not re-implemented | yes |

If the team can perform the motions but has no judgment about WHEN, they are competent, not guardians — keep cultivating before you call transfer complete. The Gate (ref 05) is the objective proof.
