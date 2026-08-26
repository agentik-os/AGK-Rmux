#!/usr/bin/env python3
"""Create a safe, editable Markdown workspace for Mindset {OS}."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from textwrap import dedent


def render_files(name: str) -> dict[str, str]:
    scorecard = {
        "week_start": "YYYY-MM-DD",
        "state": {
            "sleep": 0,
            "energy": 0,
            "mental_emotional": 0,
            "meaning_spirituality": 0,
            "relationships": 0,
            "joy_recovery": 0,
        },
        "execution": {
            "weekly_result_complete": False,
            "decisive_actions_completed": 0,
            "deep_work_blocks": 0,
            "value_assets_shipped": 0,
            "sales_or_distribution_actions": 0,
        },
        "identity": {
            "promises_kept": 0,
            "promises_repaired": 0,
            "promises_avoided": 0,
        },
        "rohn": {
            "philosophy_reviewed": False,
            "journal_entries": 0,
            "self_education_sessions": 0,
            "daily_disciplines_kept": 0,
            "repeated_errors_interrupted": 0,
            "marketplace_value_actions": 0,
            "relationship_investments": 0,
            "lifestyle_moments": 0,
        },
        "review": {
            "win": "",
            "bottleneck": "",
            "lesson": "",
            "system_change": "",
            "next_week_result": "",
        },
    }

    return {
        "00_START_HERE.md": dedent(
            f"""\
            # {name}

            ## Current season

            - Season:
            - Why now:
            - Transformation outcome:
            - What this must not cost:
            - Review date:

            ## Operating rule

            Stabilize -> Observe -> Clarify -> Design -> Execute -> Measure -> Learn -> Update.

            ## This week's single result

            - Result:
            - Done means:
            - First action:
            - Calendar block:

            ## Floors

            - Identity:
            - Health/energy:
            - Spiritual/reflection:
            - Value creation:
            - Relationship:
            """
        ),
        "01_IDENTITY_CONSTITUTION.md": dedent(
            """\
            # Identity Constitution

            ## Life direction

            - Life vision:
            - Anti-vision:
            - Why:
            - Anti-why:
            - Beneficiaries:
            - Enough means:

            ## Values and tradeoffs

            | Value | Means | Does not mean | Tradeoff | Repair |
            | --- | --- | --- | --- | --- |
            |  |  |  |  |  |

            ## Roles

            - Human:
            - Spiritual/community:
            - Relationship:
            - Work/craft:
            - Stewardship:

            ## Five identity principles

            1.
            2.
            3.
            4.
            5.

            ## Behavioral standards

            - When ..., I ...

            ## Boundaries and anti-identity

            - I do not:
            """
        ),
        "02_90_DAY_PLAN.md": dedent(
            """\
            # 90-Day Transformation

            ## Outcome contract

            - Baseline:
            - Target and date:
            - Value created:
            - Lead indicators:
            - Lag indicators:
            - Costs/guardrails:
            - Kill or pivot criteria:

            ## WOOP

            - Wish:
            - Outcome:
            - Obstacle:
            - Plan: If ..., then I ...

            ## Phases

            - Week 0 — Design:
            - Weeks 1–2 — Stabilize:
            - Weeks 3–6 — Build evidence:
            - Weeks 7–10 — Leverage:
            - Weeks 11–12 — Consolidate:

            ## Current week

            - Single result:
            - Done means:
            - Three actions:
            - Not Now:
            """
        ),
        "03_DAILY_CARD.md": dedent(
            """\
            # Daily Card — YYYY-MM-DD

            ## Morning

            - State: energy __/10 · stress __/10 · sleep __ hours
            - Identity standard:
            - Decisive action:
            - Done means:
            - Health floor:
            - Spiritual/reflection floor:
            - Relationship floor:
            - Primary obstacle:
            - If-then response:

            ## Evening

            - Finished/shipped:
            - Identity vote:
            - Promise kept/repaired/avoided:
            - Friction:
            - Gratitude/release:
            - Tomorrow's first action:
            """
        ),
        "04_WEEKLY_SCORECARD.json": json.dumps(scorecard, indent=2) + "\n",
        "05_IDENTITY_LEDGER.md": dedent(
            """\
            # Identity Evidence Ledger

            | Date | Situation | Standard | Action | Difficulty 0–10 | Evidence learned | Repair/next |
            | --- | --- | --- | --- | --- | --- | --- |
            |  |  |  |  |  |  |  |
            """
        ),
        "06_DECISION_JOURNAL.md": dedent(
            """\
            # Decision Journal

            ## Decision — YYYY-MM-DD

            - Decision and deadline:
            - Current state/emotion/sleep:
            - Options, including do nothing:
            - Values and criteria:
            - Assumptions and probabilities:
            - Reversibility:
            - Downside and guardrails:
            - Pre-mortem:
            - Choice or next evidence test:
            - Review date:
            - Result and process lesson:
            """
        ),
        "07_NOT_NOW.md": dedent(
            """\
            # Not Now

            Capture ideas without activating them.

            | Idea | Why attractive | Why not now | Review date | Activation condition |
            | --- | --- | --- | --- | --- |
            |  |  |  |  |  |
            """
        ),
        "08_RESET_PROTOCOLS.md": dedent(
            """\
            # Reset Protocols

            ## Sixty seconds

            1. Stop and exhale slowly.
            2. State the facts in one sentence.
            3. Name the next physical action.
            4. Do it for two minutes.

            ## Missed day

            - Cause: capacity / environment / emotion / ambiguity / choice
            - Repair:
            - Floor at next safe cue:
            - One condition to change:

            ## Bad week

            - Protect:
            - Cancel/park:
            - One completion:
            - One support conversation:
            - Seven-day floor:

            ## Personal warning signs

            - Mental/emotional:
            - Physical:
            - Financial:
            - Spiritual:
            - People to contact:
            - Professional/emergency plan:
            """
        ),
        "09_PERSONAL_PHILOSOPHY.md": dedent(
            """\
            # Personal Philosophy Constitution

            ## Preamble

            - Who I am becoming:
            - Whom my life serves:
            - What I will not sacrifice:

            ## Reality commitments

            - I treat facts, uncertainty, feedback, and error by:

            ## Twelve decision laws

            1.
            2.
            3.
            4.
            5.
            6.
            7.
            8.
            9.
            10.
            11.
            12.

            ## Philosophy debugger

            | Trigger | Rule | Action | Consequence | Edit/test | Review date |
            | --- | --- | --- | --- | --- | --- |
            |  |  |  |  |  |  |

            ## Version

            - Version: 0.1
            - Date:
            - Evidence required before revision:
            """
        ),
        "10_WRITTEN_GOALS_LEDGER.md": dedent(
            """\
            # Written Goals Ledger

            ## Becoming

            -

            ## Learning and mastery

            -

            ## Creating and building

            -

            ## Giving and contribution

            -

            ## Owning and financial freedom

            -

            ## Experiencing and lifestyle

            -

            ## Horizon map

            | Goal | 1/3/5/10 years | Why | Person/capability required | Status | Review date |
            | --- | --- | --- | --- | --- | --- |
            |  |  |  |  |  |  |

            ## Active 90-day goal

            - External result:
            - Becoming required:
            - Daily/weekly discipline:
            - Sacrifice required:
            - Sacrifice refused:
            - First action:
            """
        ),
        "11_SIMPLE_DISCIPLINES.md": dedent(
            """\
            # Simple Disciplines and Repeated Errors

            ## Three active disciplines

            | Discipline | Floor | Cue | Evidence | If-then repair |
            | --- | --- | --- | --- | --- |
            |  |  |  |  |  |

            ## Repeated errors in judgment

            | Error | Trigger | Immediate payoff | Delayed cost | Interruption rule |
            | --- | --- | --- | --- | --- |
            |  |  |  |  |  |

            ## Daily check

            - Philosophy principle reviewed:
            - Decisive activity completed:
            - Discipline kept/repaired:
            - Error interrupted/learned:
            """
        ),
        "12_ROHN_JOURNAL.md": dedent(
            """\
            # Jim Rohn–Style Applied Journal

            ## Entry — YYYY-MM-DD

            - Type: LESSON / DECISION / ERROR / WIN / QUESTION / PEOPLE / MONEY / SEASON / LIFESTYLE
            - Situation/source:
            - What struck me:
            - Philosophy or assumption involved:
            - Evidence:
            - Action / principle / question / archive:
            - Review date:

            ## Five-pieces evening review

            - Philosophy — what rule drove me?
            - Attitude — what emotional stance shaped me?
            - Activity — what did I complete?
            - Results — what changed and at what cost?
            - Lifestyle — did I live, relate, rest, serve, or appreciate?
            """
        ),
        "13_ASSOCIATION_INPUT_AUDIT.md": dedent(
            """\
            # Association and Mental-Input Audit

            ## People and communities

            | Person/community | Attention share | Standards normalized | Emotional effect | Trust/reciprocity | Response |
            | --- | --- | --- | --- | --- | --- |
            |  |  |  |  |  | deepen / appreciate / learn / diversify / repair / bound / exit |

            ## Five dominant digital inputs

            | Source | Minutes/week | Philosophy installed | Evidence quality | Action generated | Keep/change |
            | --- | --- | --- | --- | --- | --- |
            |  |  |  |  |  |  |

            ## Relationship actions

            - Deepen/appreciate:
            - Learn/seek mentorship:
            - Repair:
            - Boundary:
            - New environment/community:
            """
        ),
        "14_SEASONS_MAP.md": dedent(
            """\
            # Seasons of Life Map

            | Domain | Winter/Spring/Summer/Autumn | Evidence | Required response | Main risk | 30-day action |
            | --- | --- | --- | --- | --- | --- |
            | Life/meaning |  |  |  |  |  |
            | Business/work |  |  |  |  |  |
            | Health/energy |  |  |  |  |  |
            | Relationships |  |  |  |  |  |
            | Capital/money |  |  |  |  |  |
            | Learning |  |  |  |  |  |
            | Influence/creation |  |  |  |  |  |
            | Travel/experience |  |  |  |  |  |
            | Legacy/service |  |  |  |  |  |

            ## Season response

            - Conserve:
            - Plant:
            - Protect:
            - Harvest:
            - Grieve/release:
            """
        ),
        "15_MARKETPLACE_VALUE.md": dedent(
            """\
            # Marketplace Value and Enterprise Map

            - Valuable problem:
            - Customer/stakeholder:
            - Outcome valued:
            - Rare/trusted capability:
            - Current proof:
            - Distribution behavior:
            - Delivery constraint:
            - Ownership/asset opportunity:
            - Leverage: code / media / systems / capital / team
            - Economics and downside:

            ## 90-day value development

            - Capability to build:
            - Practice:
            - Feedback:
            - Weekly output:
            - Asset to finish:
            - Graduation evidence:
            """
        ),
        "16_FINANCIAL_PHILOSOPHY.md": dedent(
            """\
            # Financial Philosophy

            This is a decision-policy draft, not financial advice.

            ## What money is for

            - Freedom:
            - Protection:
            - Ownership:
            - Contribution:
            - Lifestyle:
            - Enoughness:

            ## Allocation order

            | Purpose | Rule/percentage pending professional context | Account/system | Review cadence |
            | --- | --- | --- | --- |
            | Taxes |  |  |  |
            | Essentials/operations |  |  |  |
            | Reserve/runway |  |  |  |
            | Business reinvestment |  |  |  |
            | Long-term investing |  |  |  |
            | Giving/contribution |  |  |  |
            | Lifestyle |  |  |  |

            ## Guardrails

            - Ruin risk never accepted:
            - Waiting-period threshold:
            - Independent review threshold:
            - Lifestyle-inflation rule:
            - Professional questions:
            """
        ),
        "17_LIFESTYLE_NOW_LATER.md": dedent(
            """\
            # Lifestyle Now / Lifestyle Later

            | Domain | Floor available now | Standard rhythm | Future expansion | Guardrail | People included |
            | --- | --- | --- | --- | --- | --- |
            | Faith/sacred time |  |  |  |  |  |
            | Health/vitality |  |  |  |  |  |
            | Relationships |  |  |  |  |  |
            | Home/hospitality |  |  |  |  |  |
            | Art/music/culture |  |  |  |  |  |
            | Travel/exploration |  |  |  |  |  |
            | Generosity/service |  |  |  |  |  |
            | Rest/play/beauty |  |  |  |  |  |

            ## This week's art of living

            - Moment:
            - Date/time:
            - With whom:
            - Why it matters without productivity:
            """
        ),
        "18_ROHN_90_DAY_PROGRAM.md": dedent(
            """\
            # Jim Rohn 90-Day Program Tracker

            ## Contract

            - One 90-day result:
            - Philosophy rule to revise:
            - Value/wealth discipline:
            - Health/relationship/spiritual discipline:
            - Repeated error to interrupt:
            - Non-sacrifices:

            ## Weekly sequence

            - Week 0 — Baseline and safety:
            - Week 1 — Philosophy inventory:
            - Week 2 — Constitution v0.1:
            - Week 3 — Written goals:
            - Week 4 — Attitude and emotional agency:
            - Week 5 — Simple disciplines:
            - Week 6 — Self-education:
            - Week 7 — Marketplace value:
            - Week 8 — Enterprise and ownership:
            - Week 9 — Financial philosophy:
            - Week 10 — Associations and inputs:
            - Week 11 — Seasons and resilience:
            - Week 12 — Lifestyle and integration:

            ## Final review

            - Result:
            - Person/capability developed:
            - Philosophy updated:
            - Asset/value created:
            - Costs and guardrails:
            - Continue / pivot / stop / scale:
            """
        ),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--name", default="My Mindset {OS}")
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument(
        "--force",
        action="store_true",
        help="Replace files created by this script inside an existing directory.",
    )
    args = parser.parse_args()

    output = args.output.expanduser().resolve()
    if output == Path(output.anchor):
        parser.error("Refusing to use a filesystem root as the output directory")

    output.mkdir(parents=True, exist_ok=True)
    files = render_files(args.name.strip() or "My Mindset {OS}")

    conflicts = [name for name in files if (output / name).exists()]
    if conflicts and not args.force:
        parser.error(
            "Refusing to overwrite existing files: " + ", ".join(sorted(conflicts))
        )

    for name, content in files.items():
        (output / name).write_text(content, encoding="utf-8")

    print(json.dumps({"directory": str(output), "files": sorted(files)}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
