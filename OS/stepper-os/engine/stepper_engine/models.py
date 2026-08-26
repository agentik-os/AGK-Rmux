"""Immutable spec models (pydantic) + runtime state records.

Specs (manifest / module / epic / slice / step) mirror the pack schemas in
`pack/03_STEP_CONTRACT_SPEC.md` and `pack/11_PROJECT_MANIFEST_EXAMPLE.yaml`.
All spec models allow extra fields so a richer step document (attention,
observability, rollback, ...) loads without loss and without the engine
having to model every documentation-only key.

Runtime state (StepState / Attempt / Event / Review) is what the Tracker
persists - specs never mutate, state does.
"""

from __future__ import annotations

from enum import Enum
from typing import Any

from pydantic import BaseModel, ConfigDict, Field


class StepStatus(str, Enum):
    PENDING = "PENDING"
    READY = "READY"
    RUNNING = "RUNNING"
    VERIFYING = "VERIFYING"
    FAILED = "FAILED"
    BLOCKED = "BLOCKED"
    DONE = "DONE"
    SKIPPED = "SKIPPED"
    SUPERSEDED = "SUPERSEDED"
    STALE = "STALE"


#: Statuses a step can be picked up from (the READY predicate's base set).
SCHEDULABLE = {StepStatus.PENDING, StepStatus.READY, StepStatus.FAILED}
#: Terminal-success statuses that satisfy a hard dependency.
SATISFIES_DEPENDENCY = {StepStatus.DONE, StepStatus.SKIPPED, StepStatus.SUPERSEDED}

PRIORITY_WEIGHT = {"P0": 1000, "P1": 500, "P2": 200, "P3": 50}


class SpecModel(BaseModel):
    """Base for all spec documents: frozen intent, forward-compatible extras."""

    model_config = ConfigDict(extra="allow")


class Dependencies(SpecModel):
    hard: list[str] = Field(default_factory=list)
    soft: list[str] = Field(default_factory=list)


class Risk(SpecModel):
    level: str = "LOW"  # LOW | MEDIUM | HIGH | CRITICAL
    reasons: list[str] = Field(default_factory=list)


class AcceptanceCheck(SpecModel):
    """One deterministic acceptance predicate (pack 05 check types)."""

    type: str  # file_exists | file_absent | grep_present | grep_absent |
    #            command | pytest | review_gate | artifact_exists
    path: str | None = None
    pattern: str | None = None
    command: str | None = None
    role: str | None = None  # review_gate: which review role gates it


class Reference(SpecModel):
    """One upstream reference a step must read. Governs the step: the coding
    agent opens `doc` (resolved against the matching source root) and reads the
    named `sections` and/or `ids`. Used for BOTH Blueprint references (BPH
    requirement/decision/capability ids + doc sections) and Design references
    (flow / surface / screen / state / component ids from the Design Handoff).
    Extra keys are preserved (extra=allow), so a richer ref survives."""

    doc: str = ""  # doc path relative to the source root (blueprint/ or design/)
    sections: list[Any] = Field(default_factory=list)  # section numbers/anchors
    ids: list[str] = Field(default_factory=list)  # BPH/requirement/flow/surface ids
    note: str = ""


class StepSpec(SpecModel):
    step_id: str
    title: str
    module: str
    epic: str = ""
    slice: str = ""
    priority: str = "P2"
    status: StepStatus = StepStatus.PENDING  # initial status only; runtime wins
    weight: float = 1.0
    risk: Risk = Field(default_factory=Risk)
    dependencies: Dependencies = Field(default_factory=Dependencies)
    requirements: list[str] = Field(default_factory=list)
    decisions: list[str] = Field(default_factory=list)
    invariants: list[str] = Field(default_factory=list)
    # Upstream references that GOVERN this step — the coding agent must read
    # them. Blueprint = product truth (WHAT/WHY, from Blueprint OS); Design =
    # UX/UI truth (flows, screens, states, from Design OS). Both are typed so
    # they can be resolved against the project's source roots and validated.
    blueprint_references: list[Reference] = Field(default_factory=list)
    design_references: list[Reference] = Field(default_factory=list)
    locks: list[Any] = Field(default_factory=list)
    context_files: dict[str, list[str]] = Field(default_factory=dict)
    expected_files: dict[str, list[str]] = Field(default_factory=dict)
    implementation_prompt: str = ""
    commands: dict[str, list[str]] = Field(default_factory=dict)
    acceptance_checks: list[AcceptanceCheck] = Field(default_factory=list)
    acceptance_criteria: list[str] = Field(default_factory=list)
    review_roles: list[str] = Field(default_factory=list)
    definition_of_done: list[str] = Field(default_factory=list)

    def lock_keys(self) -> set[str]:
        """Normalized lock identifiers ('domain:experiences', 'path:convex/x')."""
        keys: set[str] = set()
        for lock in self.locks:
            if isinstance(lock, str):
                keys.add(lock)
            elif isinstance(lock, dict):
                for kind, value in lock.items():
                    keys.add(f"{kind}:{value}")
        return keys


class ModuleSpec(SpecModel):
    module_id: str
    name: str = ""
    purpose: str = ""
    depends_on: list[str] = Field(default_factory=list)
    priority: str = "P2"
    risk_level: str = "LOW"


class EpicSpec(SpecModel):
    epic_id: str
    module_id: str = ""
    name: str = ""
    objective: str = ""
    requirements: list[str] = Field(default_factory=list)
    depends_on: list[str] = Field(default_factory=list)


class SliceSpec(SpecModel):
    slice_id: str
    module_id: str = ""
    epic_id: str = ""
    name: str = ""
    user_outcome: str = ""
    depends_on: list[str] = Field(default_factory=list)
    acceptance_tests: list[str] = Field(default_factory=list)


class ExecutionConfig(SpecModel):
    max_parallel_steps: int = 4
    max_active_modules: int = 3
    max_fix_attempts: int = 5


class ReleaseConfig(SpecModel):
    target: str = "P0"  # steps with priority <= target gate the release
    allow_critical_blockers: bool = False


class SourceRef(SpecModel):
    """Where an upstream OS's artifacts live for this project, so a step's
    `doc` references resolve to real files. `root` is the doc directory;
    `handoff` is the frozen handoff JSON (Blueprint handoff / Design Handoff)."""

    root: str = ""
    handoff: str = ""
    version: str = ""


class Sources(SpecModel):
    """The upstream truth Stepper steps reference. Blueprint = product truth
    (Blueprint OS); design = UX/UI truth (Design OS). Both optional, so an
    older project with no design stage still loads."""

    blueprint: SourceRef = Field(default_factory=SourceRef)
    design: SourceRef = Field(default_factory=SourceRef)


class Manifest(SpecModel):
    project: dict[str, Any] = Field(default_factory=dict)
    blueprint: dict[str, Any] = Field(default_factory=dict)
    # Typed upstream sources (Blueprint + Design handoffs + doc roots) so step
    # references resolve and validate. Falls back to the legacy `blueprint`
    # dict's `root` when `sources.blueprint.root` is unset.
    sources: Sources = Field(default_factory=Sources)
    stepper: dict[str, Any] = Field(default_factory=dict)
    execution: ExecutionConfig = Field(default_factory=ExecutionConfig)
    planner: dict[str, Any] = Field(default_factory=dict)
    quality: dict[str, Any] = Field(default_factory=dict)
    agents: dict[str, Any] = Field(default_factory=dict)
    release: ReleaseConfig = Field(default_factory=ReleaseConfig)


# ── Runtime state (Tracker-owned, mutable, persisted) ────────────────────────


class Attempt(BaseModel):
    attempt_id: str
    step_id: str
    started_at: str
    finished_at: str | None = None
    agent_adapter: str = "manual"
    status: str = "RUNNING"  # RUNNING | INTERRUPTED | VERIFIED | FAILED
    summary: str = ""


class Review(BaseModel):
    step_id: str
    role: str
    verdict: str  # PASS | FAIL
    reviewer: str
    at: str
    notes: str = ""


class StepState(BaseModel):
    status: StepStatus = StepStatus.PENDING
    attempts: int = 0
    block_reason: str = ""


class Event(BaseModel):
    at: str
    event: str
    step_id: str = ""
    detail: str = ""


class CheckResult(BaseModel):
    """Machine-readable verifier evidence (pack 05 repair-loop shape)."""

    check: str
    passed: bool
    command: str | None = None
    path: str | None = None
    exit_code: int | None = None
    summary: str = ""
    evidence: list[str] = Field(default_factory=list)
