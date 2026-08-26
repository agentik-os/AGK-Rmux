"""Deterministic verification. The coding agent never certifies its own work
(pack 05): DONE is only reachable through this module's checks.

Check types supported (pack 05 minimum set, v1):
  file_exists / artifact_exists, file_absent, grep_present, grep_absent,
  command / pytest / js_test, review_gate.
Commands run as argv lists (shlex.split) - never shell=True - from the
project root, with a bounded timeout and truncated evidence capture.
"""

from __future__ import annotations

import shlex
import subprocess
from pathlib import Path

from .models import AcceptanceCheck, CheckResult, StepSpec
from .tracker import Tracker

COMMAND_TIMEOUT_SECONDS = 900
EVIDENCE_LINES = 20


def _tail(text: str, lines: int = EVIDENCE_LINES) -> list[str]:
    return text.strip().splitlines()[-lines:]


def run_check(
    check: AcceptanceCheck, root: Path, step: StepSpec, tracker: Tracker
) -> CheckResult:
    kind = check.type
    if kind in {"file_exists", "artifact_exists"}:
        path = root / (check.path or "")
        return CheckResult(
            check=kind,
            path=check.path,
            passed=path.exists(),
            summary=f"{check.path} {'exists' if path.exists() else 'MISSING'}",
        )
    if kind == "file_absent":
        path = root / (check.path or "")
        return CheckResult(
            check=kind,
            path=check.path,
            passed=not path.exists(),
            summary=f"{check.path} {'absent' if not path.exists() else 'PRESENT (should be absent)'}",
        )
    if kind in {"grep_present", "grep_absent"}:
        return _grep_check(check, root)
    if kind in {"command", "pytest", "js_test"}:
        return _command_check(check, root)
    if kind == "review_gate":
        role = check.role or "review"
        passed = role in tracker.passing_review_roles(step.step_id)
        return CheckResult(
            check=kind,
            passed=passed,
            summary=f"review '{role}': {'PASS recorded' if passed else 'no passing review recorded'}",
        )
    return CheckResult(
        check=kind, passed=False, summary=f"unknown check type '{kind}'"
    )


def _grep_check(check: AcceptanceCheck, root: Path) -> CheckResult:
    """grep_present: pattern must appear under path; grep_absent: must not.
    `path` may be a file or a directory (searched recursively)."""
    target = root / (check.path or ".")
    pattern = check.pattern or ""
    hits: list[str] = []
    files = [target] if target.is_file() else [
        p for p in target.rglob("*") if p.is_file()
    ] if target.is_dir() else []
    for path in files:
        try:
            text = path.read_text(errors="ignore")
        except OSError:
            continue
        for i, line in enumerate(text.splitlines(), start=1):
            if pattern in line:
                hits.append(f"{path.relative_to(root)}:{i}: {line.strip()[:120]}")
                if len(hits) >= EVIDENCE_LINES:
                    break
        if len(hits) >= EVIDENCE_LINES:
            break
    found = bool(hits)
    want_present = check.type == "grep_present"
    passed = found if want_present else not found
    return CheckResult(
        check=check.type,
        path=check.path,
        passed=passed,
        summary=(
            f"pattern '{pattern}' "
            + ("found" if found else "not found")
            + f" under {check.path or '.'}"
        ),
        evidence=hits if not passed and not want_present else hits[:3],
    )


def _command_check(check: AcceptanceCheck, root: Path) -> CheckResult:
    command = check.command or ""
    if not command.strip():
        return CheckResult(check=check.type, passed=False, summary="empty command")
    argv = shlex.split(command)
    try:
        proc = subprocess.run(
            argv,
            cwd=root,
            capture_output=True,
            text=True,
            timeout=COMMAND_TIMEOUT_SECONDS,
        )
    except FileNotFoundError:
        return CheckResult(
            check=check.type,
            command=command,
            passed=False,
            summary=f"command not found: {argv[0]}",
        )
    except subprocess.TimeoutExpired:
        return CheckResult(
            check=check.type,
            command=command,
            passed=False,
            summary=f"timeout after {COMMAND_TIMEOUT_SECONDS}s",
        )
    passed = proc.returncode == 0
    return CheckResult(
        check=check.type,
        command=command,
        exit_code=proc.returncode,
        passed=passed,
        summary=f"exit {proc.returncode}",
        evidence=[] if passed else _tail(proc.stdout) + _tail(proc.stderr),
    )


def verify_step(step: StepSpec, root: Path, tracker: Tracker) -> list[CheckResult]:
    """Run every acceptance check plus the implicit review gates. A step that
    declares review_roles is gated on a recorded PASS per role even when no
    explicit review_gate check was written (the pack's reviewer gate)."""
    tracker.log("VERIFY_STARTED", step.step_id)
    results: list[CheckResult] = []
    for check in step.acceptance_checks:
        result = run_check(check, root, step, tracker)
        results.append(result)
        tracker.log(
            "CHECK_PASSED" if result.passed else "CHECK_FAILED",
            step.step_id,
            result.summary,
        )
    explicit_gate_roles = {
        c.role for c in step.acceptance_checks if c.type == "review_gate" and c.role
    }
    for role in step.review_roles:
        if role in explicit_gate_roles:
            continue
        result = run_check(
            AcceptanceCheck(type="review_gate", role=role), root, step, tracker
        )
        results.append(result)
        tracker.log(
            "CHECK_PASSED" if result.passed else "CHECK_FAILED",
            step.step_id,
            result.summary,
        )
    return results
