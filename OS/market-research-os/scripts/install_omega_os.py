#!/usr/bin/env python3
"""Safe dry-run-first installer for Market Research {OS} into Omega OS."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import sys
import tempfile
from pathlib import Path


SKILL_ROOT = Path(__file__).resolve().parent.parent


def digest(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def skill_files() -> list[tuple[Path, Path]]:
    mappings: list[tuple[Path, Path]] = []
    for source in sorted(SKILL_ROOT.rglob("*")):
        if not source.is_file():
            continue
        relative = source.relative_to(SKILL_ROOT)
        if "__pycache__" in relative.parts or source.suffix == ".pyc":
            continue
        mappings.append((source, Path("skills/market-research-os") / relative))
    runtime = [
        (SKILL_ROOT / "references/system-prompt.md", Path("prompts/market-research-os/system.md")),
        (SKILL_ROOT / "assets/market-research-role-prompts.json", Path("prompts/market-research-os/roles.json")),
        (SKILL_ROOT / "assets/market-research-tools.json", Path("tools/market-research-os/definitions.json")),
        (SKILL_ROOT / "scripts/market_research_os.py", Path("tools/market-research-os/market_research_os.py")),
        (SKILL_ROOT / "assets/market-research-state.schema.json", Path("schemas/market-research-os/state.schema.json")),
        (SKILL_ROOT / "assets/blueprint-input-manifest.schema.json", Path("schemas/market-research-os/blueprint-handoff.schema.json")),
        (SKILL_ROOT / "assets/omega-os.manifest.json", Path("config/market-research-os.manifest.json")),
    ]
    mappings.extend(runtime)
    return mappings


def classify(source: Path, destination: Path) -> str:
    if not destination.exists():
        return "CREATE"
    if not destination.is_file():
        return "CONFLICT_NON_FILE"
    return "SAME" if digest(source) == digest(destination) else "CONFLICT_DIFFERENT"


def atomic_copy(source: Path, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp_name = tempfile.mkstemp(prefix=destination.name + ".", suffix=".tmp", dir=destination.parent)
    os.close(fd)
    try:
        shutil.copy2(source, tmp_name)
        os.replace(tmp_name, destination)
    finally:
        if os.path.exists(tmp_name):
            os.unlink(tmp_name)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Install Market Research {OS} into Omega OS")
    parser.add_argument("omega_root", help="Absolute or relative Omega OS checkout path")
    parser.add_argument("--apply", action="store_true", help="Apply the reviewed plan")
    parser.add_argument("--force", action="store_true", help="Overwrite reviewed differing files")
    args = parser.parse_args(argv)

    omega_root = Path(args.omega_root).expanduser().resolve()
    if not omega_root.exists() or not omega_root.is_dir():
        print(json.dumps({"ok": False, "error": f"Omega root is not a directory: {omega_root}"}), file=sys.stderr)
        return 2

    actions = []
    for source, relative in skill_files():
        if not source.is_file():
            print(json.dumps({"ok": False, "error": f"Missing package source: {source}"}), file=sys.stderr)
            return 2
        destination = omega_root / relative
        action = classify(source, destination)
        item = {
            "source": str(source),
            "destination": str(destination),
            "relative_destination": str(relative),
            "action": action,
            "applied": False,
        }
        if args.apply:
            if action == "CREATE" or (action == "CONFLICT_DIFFERENT" and args.force):
                atomic_copy(source, destination)
                item["applied"] = True
                item["result"] = "COPIED"
            elif action == "SAME":
                item["result"] = "UNCHANGED"
            else:
                item["result"] = "PRESERVED"
        actions.append(item)

    conflicts = [a for a in actions if a["action"].startswith("CONFLICT")]
    copied = [a for a in actions if a["applied"]]
    payload = {
        "ok": not any(a["action"] == "CONFLICT_NON_FILE" for a in actions),
        "mode": "apply" if args.apply else "dry-run",
        "omega_root": str(omega_root),
        "force": args.force,
        "actions": actions,
        "summary": {
            "total": len(actions),
            "create": sum(a["action"] == "CREATE" for a in actions),
            "same": sum(a["action"] == "SAME" for a in actions),
            "conflicts": len(conflicts),
            "copied": len(copied),
        },
        "next": (
            "Review conflicts; rerun with --apply, adding --force only for explicitly reviewed differing files."
            if not args.apply
            else "Run the Market Research OS verification suite and register the command router."
        ),
    }
    print(json.dumps(payload, ensure_ascii=False, indent=2))
    if args.apply and conflicts and not args.force:
        return 1
    return 0 if payload["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
