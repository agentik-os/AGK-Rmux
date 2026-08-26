"""Load + validate a Stepper project from disk.

Layout (created by `stepper init`, mirrored from the pack's recommended tree):

    <project>/
    ├── stepper.yaml            the manifest (pack 11)
    └── stepper/
        ├── modules/*.yaml      one ModuleSpec per file
        ├── epics/*.yaml        one EpicSpec per file
        ├── slices/*.yaml       one SliceSpec per file
        └── steps/*.yaml        one StepSpec per file

Validation: schema (pydantic), id uniqueness, and referential integrity
(step -> module/epic/slice, hard deps -> existing steps). Graph acyclicity
is `graph.StepGraph`'s job - the loader only guarantees well-formed specs.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path

import yaml

from .models import EpicSpec, Manifest, ModuleSpec, SliceSpec, StepSpec

MANIFEST_NAME = "stepper.yaml"
SPEC_ROOT = "stepper"


class LoadError(Exception):
    """A spec failed schema or integrity validation."""


@dataclass
class Project:
    root: Path
    manifest: Manifest
    modules: dict[str, ModuleSpec] = field(default_factory=dict)
    epics: dict[str, EpicSpec] = field(default_factory=dict)
    slices: dict[str, SliceSpec] = field(default_factory=dict)
    steps: dict[str, StepSpec] = field(default_factory=dict)

    @property
    def state_dir(self) -> Path:
        return self.root / ".stepper"

    def source_root(self, kind: str) -> str:
        """The doc root for an upstream source ('blueprint' | 'design'), for
        resolving a step's references. Prefers `sources.<kind>.root`, falls
        back to the legacy `blueprint.root` for blueprint, else a sane default
        ('blueprint' / 'design')."""
        src = getattr(self.manifest.sources, kind, None)
        if src is not None and src.root:
            return src.root
        if kind == "blueprint":
            legacy = self.manifest.blueprint.get("root")
            if legacy:
                return str(legacy)
        return kind

    def source_handoff(self, kind: str) -> Path | None:
        """Absolute path to an upstream handoff JSON if configured + present."""
        src = getattr(self.manifest.sources, kind, None)
        if src is None or not src.handoff:
            return None
        p = (self.root / src.handoff).resolve()
        return p if p.is_file() else None


def _read_yaml(path: Path) -> dict:
    try:
        data = yaml.safe_load(path.read_text())
    except yaml.YAMLError as e:
        raise LoadError(f"{path}: invalid YAML: {e}") from e
    if not isinstance(data, dict):
        raise LoadError(f"{path}: expected a mapping at top level")
    return data


def _load_dir(root: Path, sub: str, model, id_field: str) -> dict:
    out: dict = {}
    directory = root / SPEC_ROOT / sub
    if not directory.is_dir():
        return out
    for path in sorted(directory.glob("*.yaml")) + sorted(directory.glob("*.yml")):
        data = _read_yaml(path)
        try:
            spec = model(**data)
        except Exception as e:  # pydantic ValidationError, kept broad for one message
            raise LoadError(f"{path}: {e}") from e
        spec_id = getattr(spec, id_field)
        if spec_id in out:
            raise LoadError(f"{path}: duplicate {id_field} '{spec_id}'")
        out[spec_id] = spec
    return out


def load_project(root: Path) -> Project:
    root = root.resolve()
    manifest_path = root / MANIFEST_NAME
    if not manifest_path.is_file():
        raise LoadError(
            f"{manifest_path} not found - not a Stepper project (run `stepper init`)"
        )
    manifest = Manifest(**_read_yaml(manifest_path))

    project = Project(
        root=root,
        manifest=manifest,
        modules=_load_dir(root, "modules", ModuleSpec, "module_id"),
        epics=_load_dir(root, "epics", EpicSpec, "epic_id"),
        slices=_load_dir(root, "slices", SliceSpec, "slice_id"),
        steps=_load_dir(root, "steps", StepSpec, "step_id"),
    )
    _check_integrity(project)
    return project


def _check_integrity(project: Project) -> None:
    problems: list[str] = []
    for step in project.steps.values():
        if project.modules and step.module and step.module not in project.modules:
            problems.append(f"{step.step_id}: unknown module '{step.module}'")
        if project.epics and step.epic and step.epic not in project.epics:
            problems.append(f"{step.step_id}: unknown epic '{step.epic}'")
        if project.slices and step.slice and step.slice not in project.slices:
            problems.append(f"{step.step_id}: unknown slice '{step.slice}'")
        for dep in step.dependencies.hard + step.dependencies.soft:
            if dep not in project.steps:
                problems.append(f"{step.step_id}: unknown dependency '{dep}'")
    for epic in project.epics.values():
        if project.modules and epic.module_id and epic.module_id not in project.modules:
            problems.append(f"{epic.epic_id}: unknown module '{epic.module_id}'")
    if problems:
        raise LoadError("integrity errors:\n  " + "\n  ".join(problems))


def check_references(project: Project) -> list[str]:
    """Non-fatal reference audit: does each step's Blueprint / Design reference
    resolve to a real doc under its source root? And when the project declares
    a Design source, do UI-touching steps actually cite a Design reference?
    Returns warnings (the CLI surfaces them; they never fail the load), so a
    step that governs code with the WRONG or MISSING upstream doc is caught."""
    warnings: list[str] = []
    has_design = bool(
        project.manifest.sources.design.root
        or project.manifest.sources.design.handoff
    )

    def audit(kind: str, refs) -> None:
        root = project.root / project.source_root(kind)
        for ref in refs:
            if ref.doc and not ref.doc.startswith("/"):
                target = (root / ref.doc)
                if not target.exists():
                    warnings.append(
                        f"{step.step_id}: {kind} reference '{ref.doc}' not found "
                        f"under {project.source_root(kind)}/"
                    )

    ui_markers = ("apps/", "components/", "app/", "ui/", "screen", "page", "convex/")
    for step in project.steps.values():
        audit("blueprint", step.blueprint_references)
        audit("design", step.design_references)
        # A step that creates/modifies UI files but cites no Design reference,
        # while the project HAS a design stage, is very likely missing its
        # design docs — surface it.
        if has_design and not step.design_references:
            touched = [
                p
                for group in step.expected_files.values()
                for p in group
            ]
            if any(m in p.lower() for p in touched for m in ui_markers):
                warnings.append(
                    f"{step.step_id}: touches UI files but has no design_references "
                    f"(the project declares a Design source)"
                )
    return warnings
