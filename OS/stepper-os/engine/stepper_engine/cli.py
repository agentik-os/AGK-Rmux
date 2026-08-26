"""The `stepper` CLI - the one surface every agent (Claude, Codex, OmegaOS)
drives. Commands return nonzero when validation / verification / release
checks fail, so an agent can branch on the exit code.

Lifecycle:  init -> validate -> plan -> start -> (implement) -> done
`done` runs the deterministic verifier; a failed check leaves the step
FAILED with machine-readable evidence - self-report never reaches DONE.
"""

from __future__ import annotations

import json
from pathlib import Path

import typer
from rich.console import Console
from rich.table import Table

from . import __version__
from .brief import agent_brief
from .graph import StepGraph
from .loader import LoadError, Project, load_project
from .models import StepStatus
from .planner import is_ready, rank, wave
from .reporter import release_check, render_markdown, status_report
from .scaffold import init_project
from .tracker import Tracker
from .verifier import verify_step

app = typer.Typer(no_args_is_help=True, add_completion=False)
console = Console()

PROJECT_OPTION = typer.Option(
    Path("."), "--project", "-p", help="Stepper project root"
)


def _load(project_path: Path) -> tuple[Project, StepGraph, Tracker]:
    try:
        project = load_project(project_path)
        graph = StepGraph(list(project.steps.values()))
    except (LoadError, ValueError) as e:
        console.print(f"[red]✗[/red] {e}")
        raise typer.Exit(code=1)
    return project, graph, Tracker(project.state_dir)


def _require_step(project: Project, step_id: str):
    step = project.steps.get(step_id)
    if step is None:
        console.print(f"[red]✗[/red] unknown step '{step_id}'")
        raise typer.Exit(code=1)
    return step


@app.command()
def version() -> None:
    """Print the engine version."""
    console.print(f"stepper-os {__version__}")


@app.command()
def init(
    project: Path = PROJECT_OPTION,
    name: str = typer.Option("", "--name", help="Project name (default: dir name)"),
) -> None:
    """Scaffold a Stepper project (manifest + spec tree + first step)."""
    root = project.resolve()
    created = init_project(root, name or root.name)
    if created:
        for path in created:
            console.print(f"[green]+[/green] {path.relative_to(root)}")
        console.print("\nNext: `stepper validate` then `stepper plan`.")
    else:
        console.print("Already initialized - nothing created.")


@app.command()
def validate(project: Path = PROJECT_OPTION) -> None:
    """Validate specs (schema, uniqueness, references) and the DAG, then audit
    that each step's Blueprint / Design references resolve to real docs."""
    proj, graph, _ = _load(project)
    console.print(
        f"[green]✓[/green] {len(proj.modules)} modules, {len(proj.epics)} epics, "
        f"{len(proj.slices)} slices, {len(proj.steps)} steps - graph is a DAG"
    )
    from .loader import check_references

    warnings = check_references(proj)
    bp = proj.source_root("blueprint")
    dz = proj.source_root("design")
    console.print(f"  sources: blueprint={bp}/  design={dz}/")
    if warnings:
        console.print(f"[yellow]! {len(warnings)} reference warning(s):[/yellow]")
        for w in warnings:
            console.print(f"    [yellow]- {w}[/yellow]")
    else:
        console.print("[green]✓[/green] every step reference resolves to a real doc")


@app.command()
def status(
    project: Path = PROJECT_OPTION,
    as_json: bool = typer.Option(False, "--json", help="JSON output"),
) -> None:
    """Project status: raw + weighted progress, per-status counts."""
    proj, graph, tracker = _load(project)
    report = status_report(proj, graph, tracker)
    if as_json:
        console.print_json(json.dumps(report.as_dict()))
        return
    console.print(render_markdown(proj, report))


@app.command()
def plan(
    project: Path = PROJECT_OPTION,
    limit: int = typer.Option(10, "--limit", "-n"),
    as_json: bool = typer.Option(False, "--json", help="JSON output"),
) -> None:
    """Ranked READY candidates + the safe execution wave (locks + WIP)."""
    proj, graph, tracker = _load(project)
    candidates = rank(proj, graph, tracker)[:limit]
    current_wave = wave(proj, graph, tracker)
    if as_json:
        console.print_json(
            json.dumps(
                {
                    "candidates": [
                        {
                            "step_id": c.step.step_id,
                            "title": c.step.title,
                            "score": c.score,
                            "components": c.components,
                        }
                        for c in candidates
                    ],
                    "wave": [c.step.step_id for c in current_wave],
                }
            )
        )
        return
    if not candidates:
        console.print("No READY steps. `stepper status` for the blockage picture.")
        return
    table = Table(title="READY candidates (ranked, explainable)")
    table.add_column("step")
    table.add_column("title")
    table.add_column("score", justify="right")
    table.add_column("why")
    wave_ids = {c.step.step_id for c in current_wave}
    for candidate in candidates:
        marker = "▶ " if candidate.step.step_id in wave_ids else "  "
        table.add_row(
            marker + candidate.step.step_id,
            candidate.step.title[:48],
            f"{candidate.score:g}",
            ", ".join(candidate.reasons),
        )
    console.print(table)
    console.print(f"Wave (safe to run now): {', '.join(sorted(wave_ids)) or '—'}")


@app.command()
def show(step_id: str, project: Path = PROJECT_OPTION) -> None:
    """Full spec + runtime state of one step."""
    proj, _, tracker = _load(project)
    step = _require_step(proj, step_id)
    state = tracker.state_of(step_id)
    console.print_json(
        json.dumps(
            {
                "spec": step.model_dump(mode="json"),
                "state": state.model_dump(mode="json"),
            }
        )
    )


@app.command("agent-brief")
def agent_brief_cmd(step_id: str, project: Path = PROJECT_OPTION) -> None:
    """Emit the self-contained markdown brief a coding agent executes."""
    proj, _, _ = _load(project)
    step = _require_step(proj, step_id)
    typer.echo(agent_brief(proj, step))


@app.command()
def start(
    step_id: str,
    project: Path = PROJECT_OPTION,
    agent: str = typer.Option("manual", "--agent", help="Agent adapter name"),
) -> None:
    """Claim a READY step: transition to RUNNING and open an attempt."""
    proj, graph, tracker = _load(project)
    step = _require_step(proj, step_id)
    state = tracker.state_of(step_id)
    if state.status == StepStatus.RUNNING:
        console.print(f"[yellow]![/yellow] {step_id} is already RUNNING")
        raise typer.Exit(code=1)
    if not is_ready(step, graph, tracker):
        missing = [
            dep
            for dep in step.dependencies.hard
            if tracker.status_of(dep) != StepStatus.DONE
        ]
        console.print(
            f"[red]✗[/red] {step_id} is not READY"
            + (f" - unmet hard deps: {', '.join(missing)}" if missing else "")
        )
        raise typer.Exit(code=1)
    attempts = tracker.state_of(step_id).attempts
    max_attempts = proj.manifest.execution.max_fix_attempts
    if attempts >= max_attempts:
        console.print(
            f"[red]✗[/red] {step_id} has burned {attempts}/{max_attempts} attempts - "
            "escalate to a human instead of retrying (bounded repair loop)"
        )
        raise typer.Exit(code=1)
    tracker.open_attempt(step_id, agent)
    tracker.set_status(step_id, StepStatus.RUNNING, f"attempt by {agent}")
    tracker.save()
    console.print(f"[green]▶[/green] {step_id} RUNNING - brief:\n")
    typer.echo(agent_brief(proj, step))


@app.command()
def verify(step_id: str, project: Path = PROJECT_OPTION) -> None:
    """Run the deterministic checks WITHOUT changing step state."""
    proj, _, tracker = _load(project)
    step = _require_step(proj, step_id)
    results = verify_step(step, proj.root, tracker)
    tracker.save()
    _print_results(results)
    if not all(r.passed for r in results):
        raise typer.Exit(code=1)


@app.command()
def done(
    step_id: str,
    project: Path = PROJECT_OPTION,
    summary: str = typer.Option("", "--summary", help="Attempt summary"),
) -> None:
    """Verify then close: DONE only if every check passes (no self-report)."""
    proj, _, tracker = _load(project)
    step = _require_step(proj, step_id)
    tracker.set_status(step_id, StepStatus.VERIFYING)
    results = verify_step(step, proj.root, tracker)
    _print_results(results)
    if all(r.passed for r in results):
        tracker.close_attempt(step_id, "VERIFIED", summary)
        tracker.set_status(step_id, StepStatus.DONE, summary)
        tracker.save()
        console.print(f"[green]✓[/green] {step_id} DONE (verified)")
        return
    tracker.close_attempt(step_id, "FAILED", summary or "verification failed")
    tracker.set_status(step_id, StepStatus.FAILED, "verification failed")
    tracker.save()
    failed = [r for r in results if not r.passed]
    console.print(
        f"[red]✗[/red] {step_id} FAILED verification ({len(failed)} failing check(s)). "
        "Repair against the evidence above, then run `stepper done` again."
    )
    raise typer.Exit(code=1)


@app.command()
def fail(
    step_id: str,
    project: Path = PROJECT_OPTION,
    reason: str = typer.Option("", "--reason"),
) -> None:
    """Mark the current attempt failed (agent gave up / hard error)."""
    proj, _, tracker = _load(project)
    _require_step(proj, step_id)
    tracker.close_attempt(step_id, "FAILED", reason)
    tracker.set_status(step_id, StepStatus.FAILED, reason)
    tracker.save()
    console.print(f"[red]✗[/red] {step_id} FAILED: {reason}")


@app.command()
def block(
    step_id: str,
    project: Path = PROJECT_OPTION,
    reason: str = typer.Option(..., "--reason", help="Why it is blocked"),
) -> None:
    """Block a step on an external decision/dependency."""
    proj, _, tracker = _load(project)
    _require_step(proj, step_id)
    state = tracker.state_of(step_id)
    state.status = StepStatus.BLOCKED
    state.block_reason = reason
    tracker.log("STEP_BLOCKED", step_id, reason)
    tracker.save()
    console.print(f"[yellow]■[/yellow] {step_id} BLOCKED: {reason}")


@app.command()
def unblock(step_id: str, project: Path = PROJECT_OPTION) -> None:
    """Lift a block: the step returns to PENDING (planner re-offers it)."""
    proj, _, tracker = _load(project)
    _require_step(proj, step_id)
    state = tracker.state_of(step_id)
    if state.status != StepStatus.BLOCKED:
        console.print(f"[yellow]![/yellow] {step_id} is not BLOCKED")
        raise typer.Exit(code=1)
    state.status = StepStatus.PENDING
    state.block_reason = ""
    tracker.log("STEP_PENDING", step_id, "unblocked")
    tracker.save()
    console.print(f"[green]✓[/green] {step_id} unblocked")


@app.command()
def review(
    step_id: str,
    role: str,
    verdict: str,
    project: Path = PROJECT_OPTION,
    by: str = typer.Option(..., "--by", help="Reviewer name (required)"),
    notes: str = typer.Option("", "--notes"),
) -> None:
    """Record a review verdict (PASS|FAIL) for a role gate."""
    verdict = verdict.upper()
    if verdict not in {"PASS", "FAIL"}:
        console.print("[red]✗[/red] verdict must be PASS or FAIL")
        raise typer.Exit(code=1)
    proj, _, tracker = _load(project)
    _require_step(proj, step_id)
    tracker.record_review(step_id, role, verdict, by, notes)
    tracker.save()
    console.print(f"[green]✓[/green] review recorded: {step_id} {role}={verdict} by {by}")


@app.command()
def resume(project: Path = PROJECT_OPTION) -> None:
    """Restart-safety reconcile: interrupted RUNNING/VERIFYING attempts drop
    back to FAILED so the planner re-offers them (pack 04 resume protocol)."""
    proj, graph, tracker = _load(project)
    recovered = tracker.reconcile()
    tracker.save()
    if recovered:
        console.print(
            f"[yellow]![/yellow] reconciled interrupted steps: {', '.join(recovered)}"
        )
    else:
        console.print("[green]✓[/green] nothing to reconcile")
    report = status_report(proj, graph, tracker)
    console.print(
        f"READY: {report.ready_count} - continue with `stepper plan`."
    )


@app.command("release-check")
def release_check_cmd(
    project: Path = PROJECT_OPTION,
    as_json: bool = typer.Option(False, "--json", help="JSON output"),
) -> None:
    """Release gate: PASS only when every step at the target priority is DONE."""
    proj, _, tracker = _load(project)
    result = release_check(proj, tracker)
    if as_json:
        console.print_json(
            json.dumps(
                {
                    "target": result.target,
                    "passed": result.passed,
                    "blockers": result.blockers,
                }
            )
        )
    elif result.passed:
        console.print(f"[green]✓ RELEASE PASS[/green] (target {result.target})")
    else:
        console.print(
            f"[red]✗ RELEASE FAIL[/red] (target {result.target}) - "
            f"{len(result.blockers)} blocker(s):"
        )
        for blocker in result.blockers:
            console.print(f"  - {blocker}")
    if not result.passed:
        raise typer.Exit(code=1)


@app.command()
def report(
    project: Path = PROJECT_OPTION,
    out: Path = typer.Option(None, "--out", help="Write markdown to a file"),
) -> None:
    """Markdown status report (weighted + raw, per-status table)."""
    proj, graph, tracker = _load(project)
    markdown = render_markdown(proj, status_report(proj, graph, tracker))
    if out:
        out.write_text(markdown)
        console.print(f"[green]✓[/green] report written to {out}")
    else:
        typer.echo(markdown)


@app.command()
def events(
    project: Path = PROJECT_OPTION,
    limit: int = typer.Option(30, "--limit", "-n"),
) -> None:
    """Tail of the append-only event log."""
    proj, _, tracker = _load(project)
    for event in tracker.events(limit):
        console.print(f"{event.at}  {event.event:<16} {event.step_id}  {event.detail}")


def _print_results(results) -> None:
    for result in results:
        mark = "[green]✓[/green]" if result.passed else "[red]✗[/red]"
        label = result.command or result.path or ""
        console.print(f"  {mark} {result.check} {label} - {result.summary}")
        for line in result.evidence:
            console.print(f"      [dim]{line}[/dim]")


if __name__ == "__main__":
    app()
