"""Agent brief compiler: turns one StepSpec into the self-contained markdown
brief a coding agent (Claude, Codex, ...) executes. This is the pack's
ContextCompiler surface - the /stepper-os skill and the Codex prompt both
consume `stepper agent-brief <step>`."""

from __future__ import annotations

from .loader import Project
from .models import Reference, StepSpec


def _as_ref(data: dict) -> Reference:
    try:
        return Reference(**data)
    except Exception:
        return Reference(doc=str(data))


def _ref_line(ref: Reference, root: str) -> str:
    """One reference line, with the doc resolved against its source root so the
    agent knows the exact file to open, plus the sections/ids that govern."""
    doc = ref.doc
    shown = f"{root.rstrip('/')}/{doc}" if root and doc and not doc.startswith("/") else doc
    parts = [shown or "(no doc)"]
    if ref.sections:
        parts.append("sections " + ", ".join(str(s) for s in ref.sections))
    if ref.ids:
        parts.append("ids " + ", ".join(ref.ids))
    if ref.note:
        parts.append(f"— {ref.note}")
    return "  ".join(parts)


def _section(title: str, lines: list[str]) -> list[str]:
    if not lines:
        return []
    return [f"## {title}", *lines, ""]


def _bullets(items: list) -> list[str]:
    return [f"- {item}" for item in items]


def agent_brief(project: Project, step: StepSpec) -> str:
    extras = step.model_extra or {}
    out: list[str] = [
        f"# Step brief - {step.step_id}: {step.title}",
        "",
        f"Project: {project.manifest.project.get('name', project.root.name)}",
        f"Module: {step.module}   Epic: {step.epic}   Slice: {step.slice}",
        f"Priority: {step.priority}   Risk: {step.risk.level}"
        + (f" ({', '.join(step.risk.reasons)})" if step.risk.reasons else ""),
        "",
    ]

    objective = extras.get("objective")
    if isinstance(objective, dict):
        out += _section(
            "Objective",
            [objective.get("concise", ""), "", f"Outcome: {objective.get('outcome', '')}"],
        )
    why = extras.get("why")
    if isinstance(why, list):
        out += _section("Why", _bullets(why))

    # Blueprint references — product truth (WHAT/WHY). Typed field first, with
    # the legacy `extras` list as a fallback so old steps still render.
    bp_root = project.source_root("blueprint")
    bp_refs = list(step.blueprint_references)
    if not bp_refs and isinstance(extras.get("blueprint_references"), list):
        bp_refs = [
            r if isinstance(r, dict) else {"doc": str(r)}
            for r in extras["blueprint_references"]
        ]
        bp_refs = [_as_ref(r) for r in bp_refs]
    if bp_refs:
        out += _section(
            "Blueprint references — product truth, read them (they govern this step)",
            [_ref_line(r, bp_root) for r in bp_refs],
        )

    # Design references — UX/UI truth (flows, screens, states) from Design OS.
    dz_root = project.source_root("design")
    if step.design_references:
        out += _section(
            "Design references — UX/UI truth from Design OS, read them",
            [_ref_line(r, dz_root) for r in step.design_references],
        )

    out += _section("Requirements", _bullets(step.requirements))
    out += _section("Decisions that bind", _bullets(step.decisions))
    out += _section("Invariants (never violate)", _bullets(step.invariants))

    attention = extras.get("attention")
    if isinstance(attention, dict):
        for key, label in [
            ("critical", "Attention - critical"),
            ("watch_for", "Watch for"),
            ("do_not", "Do NOT"),
        ]:
            values = attention.get(key)
            if isinstance(values, list):
                out += _section(label, _bullets(values))
    forbidden = extras.get("forbidden_changes")
    if isinstance(forbidden, list):
        out += _section("Forbidden changes", _bullets(forbidden))

    if step.context_files:
        read = step.context_files.get("read", [])
        optional = step.context_files.get("optional", [])
        out += _section(
            "Context files",
            _bullets(read) + [f"- (optional) {p}" for p in optional],
        )
    if step.expected_files:
        create = step.expected_files.get("create", [])
        modify = step.expected_files.get("modify", [])
        out += _section(
            "Expected files",
            [f"- create: {p}" for p in create] + [f"- modify: {p}" for p in modify],
        )

    if step.implementation_prompt.strip():
        out += _section("Implementation prompt", [step.implementation_prompt.strip()])

    tests = extras.get("tests_required")
    if isinstance(tests, dict):
        lines = []
        for kind, items in tests.items():
            if isinstance(items, list):
                lines += [f"- [{kind}] {t}" for t in items]
        out += _section("Tests required", lines)

    if step.commands:
        lines = []
        for kind, commands in step.commands.items():
            lines += [f"- {kind}: `{c}`" for c in commands]
        out += _section("Commands (run them - output is the evidence)", lines)

    if step.acceptance_checks:
        out += _section(
            "Acceptance checks (the verifier runs these - DONE is gated on them)",
            _bullets(
                f"{c.type}: {c.command or c.path or c.role or ''}"
                + (f" pattern='{c.pattern}'" if c.pattern else "")
                for c in step.acceptance_checks
            ),
        )
    out += _section("Acceptance criteria", _bullets(step.acceptance_criteria))
    if step.review_roles:
        out += _section(
            "Review gates",
            _bullets(
                f"{role} (record with `stepper review {step.step_id} {role} PASS --by <name>`)"
                for role in step.review_roles
            ),
        )
    out += _section("Definition of done", _bullets(step.definition_of_done))
    out += [
        "## Protocol",
        "- One step, one contract: do not widen scope.",
        "- Read every context file and blueprint reference before editing.",
        "- Run the commands; never claim PASS from inspection alone.",
        f"- When implementation is complete run: `stepper done {step.step_id}`",
        "  (the verifier decides - self-report never moves state to DONE).",
        f"- On failure: repair against the evidence, then `stepper done {step.step_id}` again.",
        "",
    ]
    return "\n".join(out)
