"""The Tracker owns runtime truth: step states, attempts, reviews, and the
append-only event log. Conversational memory is never project state
(pack rule 2: Tracker owns progress).

Storage: `.stepper/state.json` (atomic tmp+rename writes) plus
`.stepper/events.jsonl` (append-only). The pack suggests SQLite; v1 uses
JSON for a zero-dependency, inspectable, git-diffable store with the same
restart-safety contract - swap to SQLite when multi-worker leases land
(documented divergence, see engine README).
"""

from __future__ import annotations

import json
import os
import uuid
from datetime import datetime, timezone
from pathlib import Path

from .models import Attempt, Event, Review, StepState, StepStatus

STATE_FILE = "state.json"
EVENTS_FILE = "events.jsonl"


def now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


class Tracker:
    def __init__(self, state_dir: Path):
        self.state_dir = state_dir
        self.state_path = state_dir / STATE_FILE
        self.events_path = state_dir / EVENTS_FILE
        self.steps: dict[str, StepState] = {}
        self.attempts: dict[str, Attempt] = {}
        self.reviews: list[Review] = []
        self._load()

    # ── persistence ─────────────────────────────────────────────────────────

    def _load(self) -> None:
        if not self.state_path.is_file():
            return
        data = json.loads(self.state_path.read_text())
        self.steps = {k: StepState(**v) for k, v in data.get("steps", {}).items()}
        self.attempts = {
            k: Attempt(**v) for k, v in data.get("attempts", {}).items()
        }
        self.reviews = [Review(**r) for r in data.get("reviews", [])]

    def save(self) -> None:
        self.state_dir.mkdir(parents=True, exist_ok=True)
        payload = {
            "steps": {k: v.model_dump() for k, v in self.steps.items()},
            "attempts": {k: v.model_dump() for k, v in self.attempts.items()},
            "reviews": [r.model_dump() for r in self.reviews],
        }
        tmp = self.state_path.with_suffix(".json.tmp")
        tmp.write_text(json.dumps(payload, indent=2, sort_keys=True))
        os.replace(tmp, self.state_path)

    def log(self, event: str, step_id: str = "", detail: str = "") -> None:
        self.state_dir.mkdir(parents=True, exist_ok=True)
        record = Event(at=now(), event=event, step_id=step_id, detail=detail)
        with self.events_path.open("a") as f:
            f.write(record.model_dump_json() + "\n")

    def events(self, limit: int = 50) -> list[Event]:
        if not self.events_path.is_file():
            return []
        lines = self.events_path.read_text().splitlines()
        return [Event(**json.loads(line)) for line in lines[-limit:]]

    # ── step state ──────────────────────────────────────────────────────────

    def state_of(self, step_id: str) -> StepState:
        return self.steps.setdefault(step_id, StepState())

    def status_of(self, step_id: str) -> StepStatus:
        return self.state_of(step_id).status

    def set_status(
        self, step_id: str, status: StepStatus, detail: str = ""
    ) -> None:
        self.state_of(step_id).status = status
        self.log(f"STEP_{status.value}", step_id, detail)

    # ── attempts ────────────────────────────────────────────────────────────

    def open_attempt(self, step_id: str, agent_adapter: str = "manual") -> Attempt:
        attempt = Attempt(
            attempt_id=uuid.uuid4().hex[:12],
            step_id=step_id,
            started_at=now(),
            agent_adapter=agent_adapter,
        )
        self.attempts[attempt.attempt_id] = attempt
        self.state_of(step_id).attempts += 1
        return attempt

    def close_attempt(self, step_id: str, status: str, summary: str = "") -> None:
        for attempt in self.attempts.values():
            if attempt.step_id == step_id and attempt.finished_at is None:
                attempt.finished_at = now()
                attempt.status = status
                attempt.summary = summary

    def open_attempts(self) -> list[Attempt]:
        return [a for a in self.attempts.values() if a.finished_at is None]

    # ── reviews ─────────────────────────────────────────────────────────────

    def record_review(
        self, step_id: str, role: str, verdict: str, reviewer: str, notes: str = ""
    ) -> None:
        if not reviewer.strip():
            raise ValueError("a review must name its reviewer")
        self.reviews.append(
            Review(
                step_id=step_id,
                role=role,
                verdict=verdict,
                reviewer=reviewer,
                at=now(),
                notes=notes,
            )
        )
        self.log(
            "REVIEW_PASSED" if verdict == "PASS" else "REVIEW_REQUESTED",
            step_id,
            f"{role}: {verdict} by {reviewer}",
        )

    def passing_review_roles(self, step_id: str) -> set[str]:
        """Roles whose LATEST review of this step is a PASS (a later FAIL
        revokes an earlier PASS)."""
        latest: dict[str, str] = {}
        for review in self.reviews:  # chronological append order
            if review.step_id == step_id:
                latest[review.role] = review.verdict
        return {role for role, verdict in latest.items() if verdict == "PASS"}

    # ── restart safety (pack 04 resume protocol) ────────────────────────────

    def reconcile(self) -> list[str]:
        """On restart: any step left RUNNING/VERIFYING by a dead process gets
        its open attempt marked INTERRUPTED and drops back to FAILED so the
        planner can re-offer it. Committed DONE work is never re-run."""
        recovered: list[str] = []
        for step_id, state in self.steps.items():
            if state.status in {StepStatus.RUNNING, StepStatus.VERIFYING}:
                self.close_attempt(step_id, "INTERRUPTED", "reconciled at restart")
                state.status = StepStatus.FAILED
                self.log("STEP_FAILED", step_id, "interrupted attempt reconciled")
                recovered.append(step_id)
        return recovered
