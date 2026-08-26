#!/usr/bin/env python3
"""Six-Month Identity Challenge — scaffold a 180-day life+identity transformation
workspace with daily / weekly / monthly follow-up artifacts and a state file the
auto-coach loop reads.

Deterministic + stdlib only. It creates editable Markdown/JSON; the COACHING
(the LLM growth loop) is `auto_coach.sh` + the /mindset-os agent. Never
overwrites an existing file unless --force.

Governing doctrine (Mindset {OS}): protect life/health/sleep/stability/
relationships before optimization; a missed day is data, not a verdict;
minimum effective behavior first; fewer completed commitments over many
exciting ones; label claims E1/E2/S/P/C.
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
from pathlib import Path

CHALLENGE_DAYS = 180
MONTHS = 6

# Six monthly identity themes — the arc of the challenge. Editable after scaffold.
MONTH_THEMES = [
    ("Month 1 — Stabilize & Baseline",
     "Protect sleep, health and calm. Capture the honest baseline. Install ONE keystone daily discipline."),
    ("Month 2 — Identity Redesign",
     "Write who you are becoming. Align a written philosophy and the first identity-based habits."),
    ("Month 3 — Discipline & Environment",
     "Compound the disciplines. Redesign environment + associations so the identity is the default."),
    ("Month 4 — Value & Wealth Behavior",
     "Increase marketplace value and wealth behavior (skill, ownership, leverage). Wealth is an outcome, never promised."),
    ("Month 5 — Depth: Mind, Body, Meaning",
     "Deepen mental/emotional fitness, training/recovery, and chosen spiritual practice (labeled S)."),
    ("Month 6 — Integration & Next Season",
     "Make the identity self-sustaining. Review the 180 days. Design the next season from evidence."),
]

START_HERE = """# Six-Month Identity Challenge — Start Here

You are changing your whole life and identity over {days} days, in {months} monthly
seasons, with daily / weekly / monthly follow-up and an AI growth loop.

## The doctrine (non-negotiable)
- Protect life, health, sleep, mental stability, integrity and close relationships FIRST.
- A missed day is DATA, not an identity verdict. You reset, you do not restart.
- Minimum effective behavior first; add complexity only after consistency.
- Fewer completed commitments beat many exciting ones.
- Wealth is an OUTCOME of value + ownership + leverage + time; never promised.
- Claims are labeled E1 (established) / E2 (promising) / S (spiritual) / P (personal) / C (clinical → a professional).

## The follow-up rhythm
- **Daily** — fill `daily/DAY-<n>.md` (2 minutes): state, keystone done?, one win, one friction.
- **Weekly** — fill `weekly/WEEK-<n>.json` (the scorecard) + `weekly/WEEK-<n>.md` review.
- **Monthly** — fill `monthly/MONTH-<n>.md`: theme review, identity delta, next-month design.

## The AI growth loop (auto-coaching)
`omega-mindset coach <workspace>` runs the Mindset {{OS}} master agent over your
latest follow-ups and writes coaching into `coaching/`. Arm the autonomous
cadence with `omega-mindset coach <workspace> --arm` (daily 07:00). Disarmed by
default — nothing runs until you arm it.

## Files
- `IDENTITY_CONSTITUTION.md` — who you are becoming (the anchor).
- `CHALLENGE_PLAN.md` — the {months}-month arc.
- `state.json` — the challenge state the coach reads (start date, day index, cadence).
- `daily/` `weekly/` `monthly/` — your follow-ups.
- `coaching/` — the AI growth log.
""".format(days=CHALLENGE_DAYS, months=MONTHS)

IDENTITY_CONSTITUTION = """# Identity Constitution — who I am becoming

> Rewrite this in your own words. Present tense, identity-based, specific.

## The person I am becoming
I am someone who ...

## My governing philosophy (E-labels where a claim could look like fact)
- ...

## Non-negotiables (protected before any optimization)
- Sleep window: ...
- Health / movement: ...
- Relationships I protect: ...
- Integrity line I never cross: ...

## Keystone daily discipline (the ONE that carries the identity)
- ...

## What I am deliberately NOT doing this season
- ...
"""

DAILY_TEMPLATE = """# Day {n} — {date_hint}

- State (0-10): __   (energy / mood / focus)
- Keystone discipline done? [ ] yes  [ ] no  → if no, the SYSTEM reason (not "I'm lazy"): ...
- One win today: ...
- One friction / what got in the way: ...
- Tomorrow's one thing: ...
"""

WEEKLY_MD = """# Week {n} review

- Theme this week: ...
- Weighted consistency (keystone days / 7): __/7
- Identity evidence (what a camera would have seen that proves the new identity): ...
- Biggest system fix for next week: ...
- Protect-first check (sleep / health / relationships still guarded?): ...
"""

MONTHLY_MD = """# {title}

Focus: {focus}

## Review
- What changed in my identity this month (evidence, not feeling): ...
- What compounded / what stalled: ...
- Energy + wellbeing trend (protect-first): ...
- Wealth behavior (value created, not money promised): ...

## Design next month
- The ONE keystone to carry: ...
- One thing to add (only if consistency held): ...
- One thing to drop: ...
"""


def weekly_scorecard(week: int) -> dict:
    domains = ["sleep", "training", "nutrition", "focus", "mood",
               "discipline", "relationships", "wealth_behavior", "meaning"]
    return {
        "week": week,
        "week_start": "YYYY-MM-DD",
        "keystone_days": 0,
        "states": {d: 0.0 for d in domains},
        "commitments_made": 0,
        "commitments_completed": 0,
        "notes": "",
    }


def build_plan() -> str:
    lines = ["# Six-Month Challenge Plan", "",
             f"{CHALLENGE_DAYS} days · {MONTHS} monthly seasons. Edit freely.", ""]
    for i, (title, focus) in enumerate(MONTH_THEMES, 1):
        lines += [f"## {title}", f"- Days {(i-1)*30+1}–{i*30}", f"- Focus: {focus}",
                  "- Keystone discipline: ...", "- Monthly outcome (evidence): ...", ""]
    return "\n".join(lines)


def scaffold(root: Path, name: str, start: str, force: bool) -> list[str]:
    created: list[str] = []

    def write(rel: str, content: str) -> None:
        p = root / rel
        if p.exists() and not force:
            return
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(content, encoding="utf-8")
        created.append(rel)

    write("00_START_HERE.md", START_HERE)
    write("IDENTITY_CONSTITUTION.md", IDENTITY_CONSTITUTION)
    write("CHALLENGE_PLAN.md", build_plan())

    # daily follow-ups (one file per day; a hint date if a start was given)
    start_date = None
    if start:
        try:
            start_date = dt.date.fromisoformat(start)
        except ValueError:
            start_date = None
    for d in range(1, CHALLENGE_DAYS + 1):
        hint = (start_date + dt.timedelta(days=d - 1)).isoformat() if start_date else "YYYY-MM-DD"
        write(f"daily/DAY-{d:03d}.md", DAILY_TEMPLATE.format(n=d, date_hint=hint))
    for w in range(1, (CHALLENGE_DAYS // 7) + 2):
        write(f"weekly/WEEK-{w:02d}.json", json.dumps(weekly_scorecard(w), indent=2) + "\n")
        write(f"weekly/WEEK-{w:02d}.md", WEEKLY_MD.format(n=w))
    for m, (title, focus) in enumerate(MONTH_THEMES, 1):
        write(f"monthly/MONTH-{m}.md", MONTHLY_MD.format(title=title, focus=focus))
    (root / "coaching").mkdir(parents=True, exist_ok=True)

    state = {
        "kind": "six-month-identity-challenge",
        "name": name,
        "start_date": start or "YYYY-MM-DD",
        "days_total": CHALLENGE_DAYS,
        "months_total": MONTHS,
        "cadence": {"daily": "07:00", "weekly": "Sun 18:00", "monthly": "last-day 18:00"},
        "auto_coach_armed": False,
        "created_utc": dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat(),
    }
    write("state.json", json.dumps(state, indent=2) + "\n")
    return created


def main() -> int:
    ap = argparse.ArgumentParser(description="Scaffold a Six-Month Identity Challenge workspace")
    ap.add_argument("--name", default="Me")
    ap.add_argument("--output", required=True)
    ap.add_argument("--start", default="", help="ISO start date YYYY-MM-DD (optional; dates the daily files)")
    ap.add_argument("--force", action="store_true", help="Replace files created by this script")
    args = ap.parse_args()
    root = Path(args.output).expanduser().resolve()
    created = scaffold(root, args.name, args.start, args.force)
    print(json.dumps({
        "ok": True,
        "workspace": str(root),
        "created": len(created),
        "daily": CHALLENGE_DAYS,
        "weekly": CHALLENGE_DAYS // 7 + 1,
        "monthly": MONTHS,
        "next": "Fill IDENTITY_CONSTITUTION.md, then `omega-mindset coach %s` (add --arm for the daily loop)." % root,
    }, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
