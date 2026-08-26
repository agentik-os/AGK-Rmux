#!/usr/bin/env python3
"""Safely install Habit Tracker {OS} into an existing Omega {OS} checkout.

Dry-run is the default. Pass --apply to write. Existing differing files are
preserved unless --force is explicitly supplied after reviewing the dry-run.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import sys
from pathlib import Path


class InstallError(RuntimeError):
    pass


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def safe_root(value: str) -> Path:
    root = Path(value).expanduser().resolve()
    if root == Path("/") or root == Path.home().resolve():
        raise InstallError("Refusing to use a filesystem root or home directory as the Omega OS target")
    if not root.exists() or not root.is_dir():
        raise InstallError(f"Omega OS target must be an existing directory: {root}")
    return root


def package_files(skill_root: Path) -> list[str]:
    files = [
        "SKILL.md",
        "agents/openai.yaml",
        "assets/habit-state.schema.json",
        "assets/tool-contracts.json",
        "assets/omega-os.manifest.json",
        "scripts/habit_os.py",
        "scripts/install_omega_os.py",
        "scripts/test_habit_os.py",
        "references/system-prompt.md",
        "references/behavior-science.md",
        "references/conversation-protocols.md",
        "references/domain-model.md",
        "references/analytics-and-visuals.md",
        "references/safety-and-boundaries.md",
        "references/omega-os-integration.md",
        "references/feature-catalog.md",
        "references/evaluation-suite.md",
    ]
    missing = [relative for relative in files if not (skill_root / relative).is_file()]
    if missing:
        raise InstallError("Package files are missing: " + ", ".join(missing))
    return files


def mappings(skill_root: Path, omega_root: Path, extension_dir: str) -> list[tuple[Path, Path]]:
    extension = omega_root / extension_dir / "habit-tracker-os"
    result = [(skill_root / relative, extension / relative) for relative in package_files(skill_root)]
    result.extend(
        [
            (skill_root / "references/system-prompt.md", omega_root / "prompts/system/habit-tracker-os.md"),
            (skill_root / "assets/tool-contracts.json", omega_root / "tools/habit-tracker-os.json"),
            (skill_root / "assets/habit-state.schema.json", omega_root / "schemas/habit-state.schema.json"),
            (skill_root / "assets/omega-os.manifest.json", omega_root / "config/plugins/habit-tracker-os.json"),
        ]
    )
    return result


def install(args: argparse.Namespace) -> int:
    omega_root = safe_root(args.omega_root)
    skill_root = Path(__file__).resolve().parent.parent
    planned = mappings(skill_root, omega_root, args.extension_dir)
    conflicts: list[str] = []
    actions: list[dict[str, str]] = []
    for source, target in planned:
        if target.exists():
            if target.is_dir():
                conflicts.append(f"Target is a directory: {target}")
            elif digest(source) == digest(target):
                actions.append({"action": "unchanged", "source": str(source), "target": str(target)})
            elif args.force:
                actions.append({"action": "replace", "source": str(source), "target": str(target)})
            else:
                conflicts.append(f"Existing file differs: {target}")
        else:
            actions.append({"action": "create", "source": str(source), "target": str(target)})
    if conflicts:
        raise InstallError(
            "Installation conflicts:\n- "
            + "\n- ".join(conflicts)
            + "\nRe-run with --force only after reviewing these exact targets."
        )
    if args.apply:
        for action in actions:
            if action["action"] == "unchanged":
                continue
            source = Path(action["source"])
            target = Path(action["target"])
            target.parent.mkdir(parents=True, exist_ok=True)
            temporary = target.with_suffix(target.suffix + ".habit-os.tmp")
            shutil.copy2(source, temporary)
            temporary.replace(target)
    print(
        json.dumps(
            {
                "ok": True,
                "mode": "apply" if args.apply else "dry-run",
                "omega_root": str(omega_root),
                "actions": actions,
                "next": "Register config/plugins/habit-tracker-os.json with the Omega router if manifests are not auto-discovered.",
            },
            indent=2,
        )
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Install Habit Tracker {OS} into Omega {OS}")
    parser.add_argument("omega_root", help="Existing Omega OS project directory")
    parser.add_argument("--extension-dir", default="extensions")
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--force", action="store_true")
    args = parser.parse_args()
    try:
        return install(args)
    except InstallError as exc:
        print(json.dumps({"ok": False, "error": str(exc)}), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
