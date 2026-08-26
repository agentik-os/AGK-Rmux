# 01 — The Adoption & Onboarding Playbook

Phases 2-3 of the skill. Outputs `caio-enablement/01-Onboarding-Session-Plans.md` and feeds `03-End-User-Training-Curriculum.md` + `04-Validated-Use-Cases-Log.md`.

This reference designs the sessions that move a person from **spectator** to **operator**, audience by audience, and runs the **demo-to-adoption arc**. The thesis underneath it all: a session is not "done" because it was attended — it is done when the person used the system on their **own real work, unaided**, weeks later (mm-11 — adoption is retention, not attendance).

---

## A. First principle — onboarding is activation, not a launch event

The dominant failure mode of an AI rollout is the **big launch**: a town-hall demo, a flurry of excitement, a Slack post, then silence. Excitement is not adoption. Adoption is a person, three weeks later, choosing the system over the old way for a real task.

Borrow the retention frame from mm-11 directly: acquisition (getting people into a session) is worthless if it evaporates. The internal **leaky bucket** is the team you trained who never used it again. So you design every session backwards from the *retained* behaviour — "operator runs their real workflow through the system next Monday" — not from "operator attended training".

The single most important number you instrument here is the **internal aha-moment**: the first time an operator's *own* real task is completed through the system, unaided. mm-11's onboarding-is-chantier-n°1 logic applies: most internal abandonment happens in the first days, before the person feels the value once. Your job is to compress time-to-aha.

> The launch event is for awareness. The aha-moment is for adoption. Spend 10% of your energy on the launch and 90% on getting each person to their first unaided run.

---

## B. The four audiences (source the roster from upstream)

Build the roster from `company-ai-os/02-Role-And-Workflow-Inventory.md` (who does what), `07-Dashboard-Feature-Specs.md` (which feature serves which role), and the discovery dossiers' `metadata.json.index.ai_appetite` (champion / neutral / skeptic per person). Then split into four audiences — each gets a session pitched at *their* job, never a generic deck.

### B.1 C-Level / executive sponsor
- **They do not learn buttons.** They learn the value surface and the decisions they own.
- Show: the executive view of the dashboard, the few numbers that prove the system is working, the approvals they personally hold (HITL on sensitive calls).
- Their real job in adoption is **sponsorship** (Prosci sponsor model; Kotter's guiding coalition). A visible, active sponsor is the #1 predictor of adoption; an absent sponsor is the #1 predictor of failure. Make their sponsorship concrete: they reference the system in their own comms, they ask managers about usage in their 1:1s, they protect the time for training.
- Hand the ROI lens forward: the exec value surface is what `caio-run-and-optimize` will measure. You set the baseline; you do not measure it here.

### B.2 Manager / department head
- Show: how their team's workflows now run end-to-end, the **approval queue** (the HITL they or their team own), reading the dashboard for their department.
- Their real job is **local reinforcement** (ADKAR's R): they keep the behaviour alive after you leave — they notice who's using it, they celebrate the first wins, they unblock the laggards. A manager who reverts to "just do it the old way for now" kills adoption in that team.
- Equip them with the manager view of `08-Adoption-Tracker.md` so reinforcement is data-driven, not nagging.

### B.3 Operator / end-user (where adoption is won or lost)
- This is the audience that decides whether the build paid off. Everything else is support for getting these people to daily use.
- Hands-on only. No slides about "AI transformation". The session is: *here is your real task; let's run it through the system together; now run it yourself*.
- Adapt hard to AI-literacy (§C). The skeptic in this group is your highest-leverage convert and your highest-risk churner.

### B.4 Internal technical owner (the future guardian)
- The eng / ops person who will extend the system. Show: the runbook, the code/config pointers (`02-Internal-Documentation-Pack.md`), the Extension Playbook (`05-`), the evolution process.
- Their onboarding overlaps Phase 4 (transfer). Start them early — they need the most runway to reach guardian level, and they become your escalation target after handover.

---

## C. AI-literacy adaptation (champion / neutral / skeptic)

The *content* you must convey is the same; *how* you convey it changes by appetite. Read each person's appetite from discovery ch.7.

| Appetite | Stance | Move | Anti-pattern |
|---|---|---|---|
| **Champion** | Already convinced, often already using shadow AI | Recruit as an internal trainer / change agent (Kotter's "volunteer army"). Give them the teach-the-trainer kit and a name. | Wasting their time on basics; not giving them a role. |
| **Neutral** | Open but busy; "show me it's worth my time" | Show, don't tell. Get them to the aha-moment on their real task in the first session. Quantify the time they personally get back. | A long demo that never touches *their* task. |
| **Skeptic** | Worried (replacement fear), burned before, or protective of their craft | Defuse the fear first (the mm-04 frame, §D). Keep them visibly in control via HITL. Let them feel ONE personal win before asking for daily use. Never argue. | Arguing them into it; dismissing the fear; forcing daily use before the first win. |

The skeptic deserves special care and it is genuine, not a tactic: the replacement fear is rational. The honest answer (the offer's whole premise) is that the system removes the tedious parts of their job, not the person — and HITL means they stay the decision-maker. If that is not true for this person, do not pretend it is; flag it to the sponsor as a real org issue, not a training problem (L2 — researcher, not sycophant).

---

## D. The internal announcement (mm-04, used lightly)

Before any session, the system gets announced internally. This is the ONLY place mm-04 (messaging) carries weight, and it carries it lightly — the heavy change-management load is Kotter + ADKAR + Prosci (see `04-change-management-and-messaging.md`). What mm-04 contributes:

- **Copy channels a desire that already exists; it does not manufacture one** (mm-04 / Schwartz). The desire to stop doing the tedious, soul-draining parts of the job is already in the team. The announcement points that existing desire at the system. You are not selling — you are naming a relief they already want.
- **Reuse the discovery frame, verbatim where possible:** *"We're removing the tedious parts of your work, not removing you. The more honest you were about the boring bits, the better your own week gets."* This is the exact frame the discovery interview used to earn honesty; reusing it keeps the promise consistent end-to-end.
- **"You", not "us"** (mm-04). The announcement is about what the reader gets back (their Friday evening, their lunch break, the dread of the Monday report gone), not about "our AI initiative" or "the company's digital transformation".
- **Clear beats clever; no false urgency** (mm-04). No countdown timers, no manufactured scarcity, no hype. In a team saturated with "AI will change everything" noise, an honest, specific, calm message is the differentiator. A fake one burns the trust you need for adoption.

Announcement structure (one short message from the sponsor, not from the CAIO):
```
1. Why now (the real pain, in their words — from discovery)      [Kotter urgency]
2. What changes for YOU (the tedious part that goes away)        [mm-04 "you", the relief]
3. What does NOT change (you, your judgment, your control/HITL)   [defuse the fear]
4. What we're asking (come to one short hands-on session)         [a small, concrete first step]
5. Who's leading it (the sponsor + the champions, by name)        [Kotter coalition]
```

---

## E. The demo-to-adoption arc (run per operator)

The spine that moves a person from spectator to operator (mm-12 — novice to competent). Six steps; the arc is not done until step 4 happens unaided.

```
1. DEMO        — show the working golden path LIVE. Runtime, not slides (L1). The real system
                 doing a real task the operator recognizes. Credibility = a working run, not a deck.
2. FRAME       — the announcement message, delivered human-to-human: removes the tedium, not you.
3. GUIDED RUN  — the operator does their OWN real task through the system with you beside them.
                 You narrate; they drive. Knowledge transfer happens here (ADKAR-K).
4. UNAIDED RUN — the operator does the same task alone. THIS is the aha and the Knowledge->Ability
                 line (ADKAR-A). Log it in 04-Validated-Use-Cases-Log.md with real evidence.
5. DAILY USE   — the system becomes the default way the task gets done. Tracked in 08-Adoption-Tracker.
6. REINFORCE   — the weekly routine + champions + the manager's local reinforcement (ADKAR-R).
                 This is what keeps the retention curve from sagging (mm-11).
```

Rules of the arc:
- **Never skip step 4.** A rollout that stops at the guided run trained a passenger, not a driver. The single line in the whole engagement that proves adoption is "the operator did their own task unaided and it worked".
- **Use their real task, not a sandbox toy.** The aha only fires on work they actually care about. A demo dataset does not convert anyone.
- **Capture the evidence at step 4.** A screenshot, a recording, the actual output that shipped — this is what goes in the Validated-Use-Cases log and seeds the run-phase ROI re-measure. No evidence = it did not happen (R-CITE).

---

## F. The first validated use cases log (the activation receipt)

`04-Validated-Use-Cases-Log.md` is the activation receipt for the whole engagement. Each entry is one real task, done through the system, accepted by the human, with evidence. It is NOT a list of features — it is a list of *real work the system did in real conditions*.

Each entry carries:
```
- Use case (the operator's real task, in their words)
- Operator (named) + role + AI-literacy at start
- Date + which system component (agent / dashboard feature)
- Aided or UNAIDED (only unaided counts toward the aha)
- Outcome: accepted as-is / edited then accepted / rejected (and why)
- Evidence (link / screenshot / output that shipped)       [R-CITE: no evidence, no entry]
- Time before vs after (operator's own estimate — a baseline for run-and-optimize, not an ROI claim)
```

The log is the bridge to `caio-run-and-optimize`: it is the seed set of real, accepted runs that the run phase will re-measure for ROI. You do NOT compute ROI here (that is the next skill); you capture the receipts honestly so the next skill can.

Discipline: the default `full-adoption-and-transfer` gate requires **>= 5** real, accepted, evidenced runs before adoption is considered proven (engagement-scaled — more for a larger rollout). Fewer than that, and you have demo enthusiasm, not adoption.

---

## G. The adoption-rescue mode (the leaky bucket, head-on)

When you are called into a system that shipped but nobody uses, you are running mm-11's diagnostic on an internal rollout. The reflex (and the wrong move) is "more training". Train harder on a bad fit and you pour more water into a hole.

Run the diagnostic first:
1. **Does the system actually work?** Re-verify the golden path at runtime (L1). A surprising share of "nobody uses it" is "it quietly broke". If broken -> back to `caio-implementation-runbook`; do not re-train.
2. **Did anyone ever reach the aha?** Pull the tracker. If operators never had a first unaided run, this was never adopted — re-run the arc from step 3, do not assume a refresher fixes it.
3. **Does it fit the real work?** Re-read the role inventory + talk to two operators. If the system solves a task that isn't actually their bottleneck, the fit is wrong — that is an architect/implementation problem, not a training one (escalate honestly).
4. **Did the curve collapse after a good start?** If operators used it then stopped, look for friction added later, a change that broke trust, or missing reinforcement (no manager kept it alive). Fix the specific cause; re-onboard the lapsed cohort.

Only after the diagnostic do you re-onboard — and you re-baseline `08-Adoption-Tracker.md` so the recovery is measured, not assumed.

---

## H. Discipline checks for this phase

| Check | Pass = |
|---|---|
| System golden path re-verified green (runtime) before any session | yes |
| Roster built from role inventory + per-person AI-literacy (not guessed) | yes |
| A session plan exists for every audience present (C-Level / Manager / Operator / Technical) | yes |
| Each plan adapted to the audience's AI-literacy (champion/neutral/skeptic) | yes |
| Announcement reuses the discovery frame ("tedium, not you"), "you" not "us", no false urgency | yes |
| The demo-to-adoption arc reaches step 4 (UNAIDED run) for each operator | yes |
| `04-Validated-Use-Cases-Log.md` has >= N real, accepted, evidenced, UNAIDED runs | yes |
| Skeptics handled by defusing fear + a personal win, never by argument | yes |
| Nothing invented — every logged use case traces to real evidence | yes |

If any check fails, the audience was trained but not activated. Re-run the arc before claiming adoption.
