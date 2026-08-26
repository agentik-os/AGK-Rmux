# Six-Month Identity Challenge (Mindset {OS} extension)

A 180-day, 6-season program to change all of life and identity, with
**daily / weekly / monthly** follow-up and an **AI growth loop**. This is the
OmegaOS operationalization of the Rohn 90-day program, doubled and made
self-tracking.

## Structure (the six seasons)

1. **Month 1 — Stabilize & Baseline** — protect sleep/health/calm, honest
   baseline, ONE keystone daily discipline.
2. **Month 2 — Identity Redesign** — written philosophy + first
   identity-based habits.
3. **Month 3 — Discipline & Environment** — compound disciplines; redesign
   environment + associations so the identity is the default.
4. **Month 4 — Value & Wealth Behavior** — marketplace value, ownership,
   leverage (wealth is an OUTCOME, never promised).
5. **Month 5 — Depth: Mind, Body, Meaning** — mental/emotional fitness,
   training/recovery, chosen spiritual practice (labeled S).
6. **Month 6 — Integration & Next Season** — make it self-sustaining, review
   the 180 days, design the next season from evidence.

## The follow-up rhythm

- **Daily** (`daily/DAY-NNN.md`, ~2 min): state 0-10, keystone done? (if not,
  the SYSTEM reason, never "I'm lazy"), one win, one friction, tomorrow's one
  thing.
- **Weekly** (`weekly/WEEK-NN.json` + `.md`): the scorecard (keystone days,
  per-domain states, commitments made vs completed) + a written review with
  identity evidence and the one system fix.
- **Monthly** (`monthly/MONTH-N.md`): theme review, identity delta (evidence,
  not feeling), wealth behavior, and next-month design.

## The AI growth loop (auto-coaching)

`omega-mindset coach <workspace>` runs the Mindset {OS} master agent (an LLM)
over the latest follow-ups and writes evidence-aware coaching into
`coaching/`, then pushes a short card to Telegram. It auto-selects cadence:
a newly-closed week -> weekly coaching, a new month -> monthly, else a daily
nudge. Each pass: (1) names the identity evidence it can see, (2) diagnoses
the SYSTEM behind any miss (never judges character), (3) gives ONE keystone
adjustment for the next period, (4) runs a protect-first check, (5) ends with
a single doable action.

**Armed on demand only** (OmegaOS autonomous-engine posture): `--arm` installs
a daily 07:00 cron; `--disarm` removes it. Nothing autonomous runs until the
operator arms it. Never clinical/crisis/medication advice — the loop routes to
a professional on any sign of risk (`safety.md`).

## Commands

```bash
omega-mindset challenge --output ~/challenge --start 2026-08-11   # scaffold 180 days
omega-mindset coach ~/challenge                                   # one growth pass now
omega-mindset coach ~/challenge --arm                             # daily loop 07:00
omega-mindset coach ~/challenge --disarm                          # stop the loop
```

The agent (`/mindset-os`) drives the coaching content; the scripts own the
deterministic workspace, the follow-up artifacts, and the cadence.
