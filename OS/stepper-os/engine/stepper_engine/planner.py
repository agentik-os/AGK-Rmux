"""The Planner selects the next best executable work from the fixed graph.
It never invents scope (pack 04). Every candidate carries its explainable
score components - no magic numbers without named parts.

READY predicate (pack 04): schedulable status + every hard dependency
satisfied + not blocked/stale. The Scheduler then builds a wave: READY
candidates that do not contend on locks with RUNNING steps or each other,
bounded by WIP limits.
"""

from __future__ import annotations

from dataclasses import dataclass, field

from .graph import StepGraph
from .loader import Project
from .models import (
    PRIORITY_WEIGHT,
    SATISFIES_DEPENDENCY,
    SCHEDULABLE,
    StepSpec,
    StepStatus,
)
from .tracker import Tracker


@dataclass
class Candidate:
    step: StepSpec
    score: float
    components: dict[str, float] = field(default_factory=dict)
    reasons: list[str] = field(default_factory=list)


def is_ready(step: StepSpec, graph: StepGraph, tracker: Tracker) -> bool:
    if tracker.status_of(step.step_id) not in SCHEDULABLE:
        return False
    return all(
        tracker.status_of(dep) in SATISFIES_DEPENDENCY
        for dep in step.dependencies.hard
    )


def active_lock_keys(project: Project, tracker: Tracker) -> set[str]:
    keys: set[str] = set()
    for step_id, state in tracker.steps.items():
        if state.status in {StepStatus.RUNNING, StepStatus.VERIFYING}:
            spec = project.steps.get(step_id)
            if spec:
                keys |= spec.lock_keys()
    return keys


def active_modules(project: Project, tracker: Tracker) -> set[str]:
    return {
        project.steps[step_id].module
        for step_id, state in tracker.steps.items()
        if state.status in {StepStatus.RUNNING, StepStatus.VERIFYING}
        and step_id in project.steps
    }


def rank(project: Project, graph: StepGraph, tracker: Tracker) -> list[Candidate]:
    """All READY steps, scored with explainable components."""
    modules_in_flight = active_modules(project, tracker)
    candidates: list[Candidate] = []
    for step in project.steps.values():
        if not is_ready(step, graph, tracker):
            continue
        components = {
            "priority": float(PRIORITY_WEIGHT.get(step.priority, 0)),
            "critical_path": graph.critical_path_weight(step.step_id),
            "downstream_unlock": graph.downstream_weight(step.step_id),
            "module_locality": 25.0
            if step.module and step.module in modules_in_flight
            else 0.0,
            "risk_urgency": {"CRITICAL": 40.0, "HIGH": 20.0}.get(
                step.risk.level, 0.0
            ),
        }
        reasons = [f"{k}={v:g}" for k, v in components.items() if v]
        candidates.append(
            Candidate(
                step=step,
                score=sum(components.values()),
                components=components,
                reasons=reasons,
            )
        )
    candidates.sort(key=lambda c: (-c.score, c.step.step_id))
    return candidates


def wave(project: Project, graph: StepGraph, tracker: Tracker) -> list[Candidate]:
    """A safe execution wave: ranked READY candidates minus lock conflicts
    (against RUNNING steps and each other), bounded by WIP limits."""
    execution = project.manifest.execution
    taken_locks = active_lock_keys(project, tracker)
    running = sum(
        1
        for s in tracker.steps.values()
        if s.status in {StepStatus.RUNNING, StepStatus.VERIFYING}
    )
    modules_in_flight = active_modules(project, tracker)

    selected: list[Candidate] = []
    for candidate in rank(project, graph, tracker):
        if running + len(selected) >= execution.max_parallel_steps:
            break
        locks = candidate.step.lock_keys()
        if locks & taken_locks:
            continue
        module = candidate.step.module
        prospective_modules = modules_in_flight | {
            c.step.module for c in selected
        }
        if (
            module
            and module not in prospective_modules
            and len(prospective_modules) >= execution.max_active_modules
        ):
            continue
        selected.append(candidate)
        taken_locks |= locks
    return selected
