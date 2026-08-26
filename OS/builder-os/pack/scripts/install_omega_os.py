#!/usr/bin/env python3
"""Safe, idempotent installer for Builder {OS} into an Omega OS tree."""

from __future__ import annotations

import argparse
import hashlib
import shutil
import sys
from dataclasses import dataclass
from pathlib import Path


REQUIRED = {
    "skill": "SKILL.md",
    "system_prompt": "references/system-prompt.md",
    "roles": "assets/builder-role-prompts.json",
    "tools": "assets/builder-tools.json",
    "schema": "assets/builder-state.schema.json",
    "manifest": "assets/omega-os.manifest.json",
    "cli": "scripts/builder_os.py",
}

FLAT_FALLBACKS = {
    "skill": ["omega-builder-skill.md", "SKILL.md"],
    "system_prompt": ["system-prompt.md"],
    "roles": ["builder-role-prompts.json"],
    "tools": ["builder-tools.json"],
    "schema": ["builder-state.schema.json"],
    "manifest": ["omega-os.manifest.json"],
    "cli": ["builder_os.py"],
}


class InstallerError(RuntimeError):
    pass


@dataclass(frozen=True)
class CopyItem:
    source: Path
    target: Path


def digest(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def resolve_source_root(explicit: str | None) -> Path:
    if explicit:
        return Path(explicit).expanduser().resolve()
    return Path(__file__).resolve().parent.parent


def resolve_required(source_root: Path) -> tuple[dict[str, Path], bool]:
    structured = all((source_root / rel).is_file() for rel in REQUIRED.values())
    if structured:
        return {key: source_root / rel for key, rel in REQUIRED.items()}, True
    resolved: dict[str, Path] = {}
    for key, candidates in FLAT_FALLBACKS.items():
        match = next((source_root / name for name in candidates if (source_root / name).is_file()), None)
        if match is None:
            raise InstallerError(f"Missing Builder package input for {key} under {source_root}")
        resolved[key] = match
    return resolved, False


def skill_files(source_root: Path) -> list[Path]:
    ignored_names = {"__pycache__", ".DS_Store"}
    result: list[Path] = []
    for path in sorted(source_root.rglob("*")):
        if not path.is_file():
            continue
        if any(part in ignored_names for part in path.parts):
            continue
        if path.suffix in {".pyc", ".pyo"}:
            continue
        result.append(path)
    return result


def build_plan(source_root: Path, omega_root: Path) -> list[CopyItem]:
    required, structured = resolve_required(source_root)
    items: list[CopyItem] = []
    extension_root = omega_root / "extensions" / "builder-os"
    if structured:
        for source in skill_files(source_root):
            items.append(CopyItem(source, extension_root / source.relative_to(source_root)))
    else:
        # A flat download remains installable, but the extension folder receives
        # only canonical entrypoints because reference hierarchy is unavailable.
        for key, source in required.items():
            if key == "system_prompt":
                relative = Path("references/system-prompt.md")
            elif key in {"roles", "tools", "schema", "manifest"}:
                relative = Path("assets") / source.name
            elif key == "cli":
                relative = Path("scripts/builder_os.py")
            else:
                relative = Path("SKILL.md")
            items.append(CopyItem(source, extension_root / relative))

    items.extend(
        [
            CopyItem(required["system_prompt"], omega_root / "prompts" / "system" / "builder-os.md"),
            CopyItem(required["roles"], omega_root / "prompts" / "roles" / "builder-os.json"),
            CopyItem(required["tools"], omega_root / "tools" / "builder-os.json"),
            CopyItem(required["schema"], omega_root / "schemas" / "builder-state.schema.json"),
            CopyItem(required["manifest"], omega_root / "config" / "plugins" / "builder-os.json"),
        ]
    )
    unique: dict[Path, CopyItem] = {}
    for item in items:
        unique[item.target] = item
    return [unique[target] for target in sorted(unique, key=str)]


def classify(item: CopyItem) -> str:
    if not item.target.exists():
        return "CREATE"
    if not item.target.is_file():
        return "CONFLICT"
    return "SAME" if digest(item.source) == digest(item.target) else "CONFLICT"


def install(plan: list[CopyItem], *, apply: bool, force: bool) -> tuple[int, int, int]:
    created = same = conflicts = 0
    for item in plan:
        action = classify(item)
        if action == "CREATE":
            created += 1
        elif action == "SAME":
            same += 1
        else:
            conflicts += 1
        effective = "REPLACE" if action == "CONFLICT" and force else action
        print(f"{effective:8} {item.target}")
        if not apply or action == "SAME":
            continue
        if action == "CONFLICT" and not force:
            continue
        item.target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(item.source, item.target)
    return created, same, conflicts


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Install Builder {OS} into Omega OS")
    parser.add_argument("omega_root", help="Absolute or relative root of the Omega OS installation")
    parser.add_argument("--source", help="Builder package root; defaults to the parent of this script")
    parser.add_argument("--apply", action="store_true", help="Apply the displayed plan; default is dry-run")
    parser.add_argument("--force", action="store_true", help="Replace differing target files; requires --apply")
    return parser


def main() -> int:
    args = build_parser().parse_args()
    if args.force and not args.apply:
        raise InstallerError("--force requires --apply")
    source_root = resolve_source_root(args.source)
    omega_root = Path(args.omega_root).expanduser().resolve()
    if source_root == omega_root or source_root in omega_root.parents:
        raise InstallerError("Omega root must not contain or equal the Builder package source")
    plan = build_plan(source_root, omega_root)
    created, same, conflicts = install(plan, apply=args.apply, force=args.force)
    mode = "APPLY" if args.apply else "DRY-RUN"
    print(f"{mode}: create={created} same={same} conflicts={conflicts}")
    if conflicts and not force_allowed(args.apply, args.force):
        print("Differing existing files were preserved. Re-run with --apply --force only after review.")
        return 1 if args.apply else 0
    return 0


def force_allowed(apply: bool, force: bool) -> bool:
    return apply and force


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except InstallerError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        raise SystemExit(2)
