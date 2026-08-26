"""Status reporting: raw + weighted progress (pack 04 - never done/total
alone), per-module rollup, critical path remaining, release check."""

from __future__ import annotations

from collections import Counter
from dataclasses import dataclass, field

from .graph import StepGraph
from .loader import Project
from .models import PRIORITY_WEIGHT, StepStatus
from .tracker import Tracker


@dataclass
class StatusReport:
    total_steps: int
    by_status: dict[str, int]
    raw_progress: float
    weighted_progress: float
    modules_total: int
    modules_complete: int
    critical_path_remaining: float
    ready_count: int

    def as_dict(self) -> dict:
        return self.__dict__.copy()


def status_report(project: Project, graph: StepGraph, tracker: Tracker) -> StatusReport:
    statuses = {
        step_id: tracker.status_of(step_id) for step_id in project.steps
    }
    counts = Counter(s.value for s in statuses.values())
    total = len(project.steps)
    done = sum(1 for s in statuses.values() if s == StepStatus.DONE)
    total_weight = sum(s.weight for s in project.steps.values()) or 1.0
    done_weight = sum(
        s.weight
        for s in project.steps.values()
        if statuses[s.step_id] == StepStatus.DONE
    )

    module_steps: dict[str, list[str]] = {}
    for step in project.steps.values():
        module_steps.setdefault(step.module or "(none)", []).append(step.step_id)
    modules_complete = sum(
        1
        for ids in module_steps.values()
        if all(statuses[i] == StepStatus.DONE for i in ids)
    )

    remaining = sum(
        project.steps[n].weight
        for n in graph.critical_path()
        if statuses[n] != StepStatus.DONE
    )
    from .planner import is_ready  # late import to avoid a cycle

    ready = sum(1 for s in project.steps.values() if is_ready(s, graph, tracker))

    return StatusReport(
        total_steps=total,
        by_status=dict(counts),
        raw_progress=round(done / total * 100, 1) if total else 0.0,
        weighted_progress=round(done_weight / total_weight * 100, 1),
        modules_total=len(module_steps),
        modules_complete=modules_complete,
        critical_path_remaining=round(remaining, 1),
        ready_count=ready,
    )


@dataclass
class ReleaseCheck:
    target: str
    passed: bool
    blockers: list[str] = field(default_factory=list)


def release_check(project: Project, tracker: Tracker) -> ReleaseCheck:
    """Release gate: every step at or above the target priority must be DONE
    (P0 target gates P0 only; P1 gates P0+P1; ...). Explicit blockers, exact
    and exhaustive - `stepper release-check` exits nonzero on FAIL."""
    target = project.manifest.release.target
    threshold = PRIORITY_WEIGHT.get(target, PRIORITY_WEIGHT["P0"])
    blockers = [
        f"{step.step_id} [{step.priority}] {step.title}: {tracker.status_of(step.step_id).value}"
        for step in sorted(project.steps.values(), key=lambda s: s.step_id)
        if PRIORITY_WEIGHT.get(step.priority, 0) >= threshold
        and tracker.status_of(step.step_id) != StepStatus.DONE
    ]
    return ReleaseCheck(target=target, passed=not blockers, blockers=blockers)


def render_markdown(project: Project, report: StatusReport) -> str:
    name = project.manifest.project.get("name", project.root.name)
    lines = [
        f"# Stepper status - {name}",
        "",
        f"- Weighted progress: **{report.weighted_progress}%** (raw {report.raw_progress}%)",
        f"- Modules complete: {report.modules_complete}/{report.modules_total}",
        f"- Steps: {report.total_steps} total, READY: {report.ready_count}",
        f"- Critical path remaining: {report.critical_path_remaining} weighted units",
        "",
        "| Status | Count |",
        "|---|---|",
    ]
    for status, count in sorted(report.by_status.items()):
        lines.append(f"| {status} | {count} |")
    return "\n".join(lines) + "\n"
