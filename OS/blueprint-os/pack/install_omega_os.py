#!/usr/bin/env python3
"""Install the portable Blueprint {OS} extension into an Omega OS checkout.

Dry-run is the default. Pass --apply to write. Existing files are preserved
unless --force is explicitly supplied.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import sys
from pathlib import Path


class InstallError(Exception):
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


def package_source(skill_root: Path, script_dir: Path, relative: str) -> Path:
    structured = skill_root / relative
    if structured.is_file():
        return structured
    flat = script_dir / Path(relative).name
    if flat.is_file():
        return flat
    raise InstallError(f"Package source is missing in structured and flat layouts: {relative}")


def mappings(skill_root: Path, script_dir: Path, omega_root: Path, extension_dir: str) -> list[tuple[Path, Path]]:
    extension = omega_root / extension_dir / "blueprint-os"
    result: list[tuple[Path, Path]] = []
    for source_relative, target_relative in [
        ("assets/omega-blueprint-skill.md", "SKILL.md"),
        ("references/system-prompt.md", "references/system-prompt.md"),
        ("references/blueprint-contract.md", "references/blueprint-contract.md"),
        ("references/orchestration-and-gates.md", "references/orchestration-and-gates.md"),
        ("references/response-and-continuation.md", "references/response-and-continuation.md"),
        ("references/functions-and-state.md", "references/functions-and-state.md"),
        ("references/omega-os-integration.md", "references/omega-os-integration.md"),
        ("references/deep-guide.md", "references/deep-guide.md"),
        ("assets/omega-os.manifest.json", "assets/omega-os.manifest.json"),
        ("assets/blueprint-tools.json", "assets/blueprint-tools.json"),
        ("assets/blueprint-state.schema.json", "assets/blueprint-state.schema.json"),
        ("assets/blueprint-role-prompts.json", "assets/blueprint-role-prompts.json"),
        ("scripts/blueprint_os.py", "scripts/blueprint_os.py"),
        ("scripts/install_omega_os.py", "scripts/install_omega_os.py"),
    ]:
        result.append((package_source(skill_root, script_dir, source_relative), extension / target_relative))

    result.extend([
        (package_source(skill_root, script_dir, "references/system-prompt.md"), omega_root / "prompts/system/blueprint-os.md"),
        (package_source(skill_root, script_dir, "assets/blueprint-role-prompts.json"), omega_root / "prompts/roles/blueprint-os.json"),
        (package_source(skill_root, script_dir, "assets/blueprint-tools.json"), omega_root / "tools/blueprint-os.json"),
        (package_source(skill_root, script_dir, "assets/blueprint-state.schema.json"), omega_root / "schemas/blueprint-state.schema.json"),
        (package_source(skill_root, script_dir, "assets/omega-os.manifest.json"), omega_root / "config/plugins/blueprint-os.json"),
    ])
    return result


def install(args: argparse.Namespace) -> int:
    omega_root = safe_root(args.omega_root)
    script_dir = Path(__file__).resolve().parent
    skill_root = script_dir.parent
    planned = mappings(skill_root, script_dir, omega_root, args.extension_dir)

    for source, _ in planned:
        if not source.is_file():
            raise InstallError(f"Package source is missing: {source}")

    conflicts = []
    actions = []
    for source, target in planned:
        if target.exists():
            if target.is_dir():
                conflicts.append(f"Target is a directory: {target}")
                continue
            if digest(source) == digest(target):
                actions.append({"action": "unchanged", "source": str(source), "target": str(target)})
            elif args.force:
                actions.append({"action": "replace", "source": str(source), "target": str(target)})
            else:
                conflicts.append(f"Existing file differs: {target}")
        else:
            actions.append({"action": "create", "source": str(source), "target": str(target)})

    if conflicts:
        raise InstallError("Installation conflicts:\n- " + "\n- ".join(conflicts) + "\nRe-run with --force only after reviewing the exact targets.")

    if args.apply:
        for action in actions:
            if action["action"] == "unchanged":
                continue
            source = Path(action["source"])
            target = Path(action["target"])
            target.parent.mkdir(parents=True, exist_ok=True)
            temporary = target.with_suffix(target.suffix + ".blueprint-os.tmp")
            shutil.copy2(source, temporary)
            temporary.replace(target)

    print(json.dumps({
        "ok": True,
        "mode": "apply" if args.apply else "dry-run",
        "omega_root": str(omega_root),
        "actions": actions,
        "next": "Register config/plugins/blueprint-os.json with the Omega command router, tool registry, and orchestrator if Omega does not auto-discover manifests."
    }, indent=2))
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Install Blueprint {OS} into an Omega OS checkout")
    parser.add_argument("omega_root", help="Existing Omega OS project directory")
    parser.add_argument("--extension-dir", default="extensions", help="Relative extension directory (default: extensions)")
    parser.add_argument("--apply", action="store_true", help="Write the planned files; default is dry-run")
    parser.add_argument("--force", action="store_true", help="Replace only the exact differing target files listed by the dry-run")
    args = parser.parse_args()
    try:
        return install(args)
    except InstallError as exc:
        print(json.dumps({"ok": False, "error": str(exc)}), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
