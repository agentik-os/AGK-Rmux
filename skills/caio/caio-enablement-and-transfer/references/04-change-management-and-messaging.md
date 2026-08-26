# 04 — Internal Change Management & Messaging

Phase 1 of the skill (and the connective tissue under all of Phase 3). Feeds the announcement and the per-person adoption design across `01-Onboarding-Session-Plans.md`.

The hard truth this reference enforces: **adopting an AI system is a change-management problem, not a marketing problem.** The change-management load is carried by the established frameworks the upstream architect already references — **Kotter 8-step + ADKAR + Prosci** — NOT by marketing copy. mm-04 (messaging) is used only, and lightly, to frame the internal **announcement**; it must not be asked to carry weight it cannot bear. mm-11 (retention) supplies the measurement spine: change that isn't reinforced lapses.

> People don't resist AI because the model is bad. They resist because change threatens competence, control, and identity. You manage that with ADKAR, not with a clever tagline.

---

## A. Why change-management, not copy

A great announcement gets someone to the first session. It does nothing for the operator who attends, feels incompetent fumbling the new tool, gets no reinforcement from their manager, and quietly reverts. That entire failure chain is change-management territory:
- competence threat -> **ADKAR Knowledge + Ability**,
- control/identity threat -> **the frame + HITL** (you stay the decision-maker),
- no reinforcement -> **ADKAR Reinforcement + Kotter step 7**,
- no sponsor -> **Prosci sponsor model + Kotter coalition**.

mm-04 can make the invitation honest and compelling. It cannot install Ability or Reinforcement. Treat copy as the doorway, the frameworks as the building.

---

## B. ADKAR — the per-person engine (Prosci's individual model)

ADKAR is the backbone of the per-audience onboarding design because change happens one person at a time. Each operator must pass all five, in order; the rollout fails at whichever step is weakest.

```
A — AWARENESS    Why this change, why now. (The real pain, from discovery — the Kotter urgency.)
                 Delivered by: the announcement (§D).
D — DESIRE       WIIFM — what THIS person gets, and the fear defused. "Removes the tedium, not you;
                 you stay in control (HITL)." Desire can't be installed by mandate — it's earned by
                 the frame + a credible promise. Skeptics live or die here.
K — KNOWLEDGE    How to use it. Delivered by: the training curriculum + the guided run.
A — ABILITY      Can they actually DO it, unaided, on real work? Knowledge =/= Ability — this is the
                 gap most rollouts miss. Delivered by: the UNAIDED first run (the aha) + the
                 Validated-Use-Cases log. This is the same line the Autonomy-Readiness Gate enforces.
R — REINFORCEMENT What keeps it alive: the adoption tracker, the manager's local reinforcement, the
                 weekly routine, celebrating wins. Without it, the retention curve sags (mm-11).
```

Operational use: for every named person on the roster, mark which ADKAR step they're at. Targeting is then obvious — a skeptic stuck at Desire doesn't need more Knowledge (another tutorial); they need the fear addressed and a personal win. A champion at Reinforcement should be recruited to teach. Diagnose the step, then act on the step.

**The Knowledge->Ability gap is the most important idea in this reference.** Most "we trained everyone" rollouts delivered Knowledge and stopped, then wondered why nobody uses it. Ability requires the person to do the real thing themselves — which is exactly why this skill's adoption proof (the unaided validated use case) and its transfer proof (the unaided extension at the Gate) are both *unaided real motions*, not attendance or quizzes.

---

## C. Kotter 8-step — the org-level sequence

ADKAR moves individuals; Kotter moves the organization. Run the eight steps as the campaign spine:

```
1. Create urgency        — name the real, current pain (from discovery, in their words). Not fear-
                           mongering; the honest cost of the tedious work continuing.
2. Build a coalition      — the executive sponsor + the champions (high-AI-literacy people from
                           discovery ch.7). Adoption with an active sponsor succeeds; without, it stalls.
3. Form a vision          — "operational guardians": the team owns and extends the system. Concrete,
                           not "AI transformation".
4. Enlist a volunteer army — turn champions into internal trainers. They scale you and they make the
                           change peer-led instead of consultant-imposed (which skeptics trust more).
5. Enable action / remove barriers — kill the friction: time carved out for training, access granted,
                           the system actually working (L1), the fear defused.
6. Generate short-term wins — the first validated use cases (ref 01 §F). Visible, real, celebrated.
                           Short-term wins are the fuel; without them, momentum dies by week 3.
7. Sustain acceleration   — don't declare victory early. Keep onboarding the next cohort, keep
                           reinforcing, move toward transfer (Phase 4).
8. Institute the change   — the weekly guardian routine + named ownership make it "how we work now",
                           not a project that ended. This is the bridge to run-and-optimize.
```

Step 6 (short-term wins) and step 8 (institute) are where AI rollouts most often fail: they skip visible early wins (so momentum dies) or they never institutionalize (so it lapses once the consultant leaves). This skill's Validated-Use-Cases log is step 6; the guardian routine + handover are step 8.

---

## D. The announcement (mm-04, used lightly — the doorway only)

The one place mm-04 carries weight. Keep it light; it opens the door, the frameworks do the rest. mm-04's contributions:

- **Channel an existing desire; don't manufacture one** (mm-04 / Schwartz: copy canalizes the hopes already in people, it doesn't create them). The desire to be free of the tedious, draining parts of the job already exists in the team. The announcement points it at the system. You are naming a relief they already want — not hyping a tool.
- **Reuse the discovery frame, verbatim:** *"We're removing the tedious parts of your work, not removing you. Nothing here is used to evaluate or replace anyone."* This is the exact promise the discovery interview made to earn honesty; keeping it identical end-to-end is what makes it credible. Breaking it (a layoff dressed as "AI efficiency") detonates the whole engagement's trust — flag that to the sponsor as a red line (L2).
- **"You", not "us"** (mm-04): the message is about what the reader gets back (their evening, the dread gone), never "our AI initiative".
- **Clear beats clever; honest beats urgent** (mm-04): no countdowns, no fake scarcity, no "the future is here" noise. In a team saturated with AI hype, a calm, specific, honest message is the differentiator; a manipulative one burns the trust adoption needs.

Announcement template (from the SPONSOR, not the CAIO — Kotter coalition):
```
Subject: A change to make your <role> week lighter

1. Why now: <the real pain, their words from discovery>.
2. What changes for you: <the specific tedious task that goes away>.
3. What does NOT change: you, your judgment, and your control — you approve anything that matters (HITL).
4. The ask: one short hands-on session (~30 min) where you run your own task through it.
5. Who's leading it: <sponsor name> with <champion names>. Questions to them, any time.
```

What mm-04 must NOT do here (the guardrail): it must not carry resistance management, sponsor alignment, capability-building, or reinforcement. The moment you find yourself trying to "write better copy" to fix low adoption, stop — the problem is almost always an ADKAR step (usually Ability or Reinforcement), not the wording.

---

## E. Resistance management (Prosci)

Resistance is data, not disobedience (L2 — researcher, not sycophant). Prosci's stance: manage resistance by understanding its root, not by overpowering it.

Common roots and the right move:
| Root of resistance | Wrong move | Right move |
|---|---|---|
| Replacement fear | "AI won't take your job, trust me" | The frame + HITL (they stay the decision-maker) + a personal win. If the fear is *true* for this person, say so honestly to the sponsor — don't lie. |
| Competence threat ("I'll look slow/dumb") | Public training that exposes them | Private guided run; let them reach the aha without an audience. |
| Past burn (a failed tool before) | "This one's different" | Show the working golden path at runtime (L1), not promises. A working run beats a reassurance. |
| Craft/identity ("this is the part I'm good at") | Automate it anyway | Only automate the tedium; keep the craft. If the system threatens the part they value, that's a scoping problem to fix, not a resistance to crush. |
| Overload ("no time for this") | Mandate attendance | Remove the barrier (Kotter 5): carve the time with the sponsor; make the first win fast so it pays for itself immediately. |

The skeptic who is heard and kept in control becomes the most credible champion, because their conversion is visible and trusted by other skeptics. The skeptic who is steamrolled becomes the permanent quiet non-user who drags the cohort retention down.

---

## F. mm-11 inside change management — reinforcement is retention

ADKAR's R and Kotter's 7-8 are the same idea mm-11 makes measurable: a change that isn't reinforced lapses, exactly like a customer who isn't retained churns. So the change-management plan is instrumented, not vibes:
- The **adoption tracker** (ref 05) is the reinforcement instrument — it shows who's lapsing before they're gone.
- The **manager** is the reinforcement agent — local, weekly, data-driven (not nagging).
- The **weekly routine** is the institutionalization — change becomes cadence.

mm-11's hierarchy applied to change: **reinforce (retain) before you expand.** Don't roll the system out to the next department while the first cohort is silently lapsing — that's pouring into a leaky bucket. Stabilize adoption, then expand (which is run-and-optimize's job).

---

## G. Discipline checks for this phase

| Check | Pass = |
|---|---|
| Change managed via Kotter + ADKAR + Prosci (not "better copy") | yes |
| Every named person mapped to their current ADKAR step; action targets that step | yes |
| The Knowledge->Ability gap closed by an UNAIDED real motion, not a quiz | yes |
| Active executive sponsor + named champions (Kotter coalition) confirmed | yes |
| Announcement reuses the discovery frame verbatim, "you" not "us", no false urgency | yes |
| The discovery promise ("remove tedium, not the person") is actually true — or the breach is flagged | yes |
| Resistance handled by root cause, never by coercion | yes |
| Reinforcement instrumented via the adoption tracker + manager + weekly routine (mm-11) | yes |

If adoption is low and your instinct is to rewrite the announcement, re-read this reference: it is almost never the copy. Find the failing ADKAR step and fix that.
