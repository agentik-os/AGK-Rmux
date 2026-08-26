#!/usr/bin/env python3
"""Validate and summarize a Mindset {OS} weekly scorecard JSON file."""

from __future__ import annotations

import argparse
import json
import statistics
from pathlib import Path
from typing import Any


def require_mapping(value: Any, path: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ValueError(f"{path} must be an object")
    return value


def require_score(value: Any, path: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise ValueError(f"{path} must be a number from 0 to 10")
    score = float(value)
    if not 0 <= score <= 10:
        raise ValueError(f"{path} must be between 0 and 10")
    return score


def require_nonnegative_int(value: Any, path: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise ValueError(f"{path} must be a non-negative integer")
    return value


def summarize(data: dict[str, Any]) -> dict[str, Any]:
    state = require_mapping(data.get("state"), "state")
    if not state:
        raise ValueError("state must contain at least one score")
    state_scores = {
        key: require_score(value, f"state.{key}") for key, value in state.items()
    }

    execution = require_mapping(data.get("execution", {}), "execution")
    execution_counts: dict[str, int] = {}
    weekly_result_complete = execution.get("weekly_result_complete", False)
    if not isinstance(weekly_result_complete, bool):
        raise ValueError("execution.weekly_result_complete must be true or false")
    for key, value in execution.items():
        if key != "weekly_result_complete":
            execution_counts[key] = require_nonnegative_int(value, f"execution.{key}")

    identity = require_mapping(data.get("identity", {}), "identity")
    identity_counts = {
        key: require_nonnegative_int(value, f"identity.{key}")
        for key, value in identity.items()
    }
    kept = identity_counts.get("promises_kept", 0)
    repaired = identity_counts.get("promises_repaired", 0)
    avoided = identity_counts.get("promises_avoided", 0)
    promise_total = kept + repaired + avoided
    repair_inclusive_rate = None
    if promise_total:
        repair_inclusive_rate = round((kept + repaired) / promise_total, 3)

    rohn = require_mapping(data.get("rohn", {}), "rohn")
    philosophy_reviewed = rohn.get("philosophy_reviewed", False)
    if not isinstance(philosophy_reviewed, bool):
        raise ValueError("rohn.philosophy_reviewed must be true or false")
    rohn_counts = {
        key: require_nonnegative_int(value, f"rohn.{key}")
        for key, value in rohn.items()
        if key != "philosophy_reviewed"
    }

    lowest = min(state_scores, key=state_scores.get)
    highest = max(state_scores, key=state_scores.get)

    return {
        "week_start": data.get("week_start"),
        "state_average": round(statistics.fmean(state_scores.values()), 2),
        "lowest_state_domain": {lowest: state_scores[lowest]},
        "highest_state_domain": {highest: state_scores[highest]},
        "weekly_result_complete": weekly_result_complete,
        "execution_counts": execution_counts,
        "identity_counts": identity_counts,
        "promise_kept_or_repaired_rate": repair_inclusive_rate,
        "rohn_metrics": {
            "philosophy_reviewed": philosophy_reviewed,
            **rohn_counts,
        },
        "interpretation_note": (
            "This summary is descriptive, not a clinical score or a measure of human worth."
        ),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("scorecard", type=Path)
    args = parser.parse_args()

    try:
        raw = json.loads(args.scorecard.read_text(encoding="utf-8"))
        result = summarize(require_mapping(raw, "root"))
    except (OSError, json.JSONDecodeError, ValueError) as exc:
        parser.error(str(exc))

    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
