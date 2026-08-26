#!/usr/bin/env python3
"""Safely install the portable Brainstorm {OS} extension into an Omega OS tree."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path


VERSION = "3.0.0"
SOURCE_ROOT = Path(__file__).resolve().parents[1]
EXCLUDED_NAMES = {"__pycache__", ".git", ".DS_Store"}


def timestamp() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def validate_target(target: Path) -> Path:
    resolved = target.expanduser().resolve()
    if not resolved.exists() or not resolved.is_dir():
        raise ValueError(f"Omega OS target must be an existing directory: {resolved}")
    if resolved == Path(resolved.anchor) or len(resolved.parts) < 3:
        raise ValueError(f"Refusing broad target: {resolved}")
    return resolved


def included_files() -> list[Path]:
    files: list[Path] = []
    for path in SOURCE_ROOT.rglob("*"):
        if not path.is_file():
            continue
        relative = path.relative_to(SOURCE_ROOT)
        if any(part in EXCLUDED_NAMES for part in relative.parts) or path.suffix == ".pyc":
            continue
        files.append(relative)
    return sorted(files)


def file_hash(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def plan(target: Path) -> dict[str, object]:
    destination = target / "extensions" / "brainstorm-os"
    files = included_files()
    return {
        "extension": "brainstorm-os",
        "version": VERSION,
        "source": str(SOURCE_ROOT),
        "destination": str(destination),
        "destination_exists": destination.exists(),
        "file_count": len(files),
        "files": [str(path) for path in files],
    }


def install(target: Path, force: bool) -> dict[str, object]:
    destination = target / "extensions" / "brainstorm-os"
    backup: Path | None = None
    destination.parent.mkdir(parents=True, exist_ok=True)
    if destination.exists():
        if not force:
            raise ValueError(f"Destination exists: {destination}; rerun with --force to create a backup and replace it")
        backup = destination.with_name(f"brainstorm-os.backup-{timestamp()}")
        destination.rename(backup)
    try:
        destination.mkdir(parents=True, exist_ok=False)
        hashes: dict[str, str] = {}
        for relative in included_files():
            source = SOURCE_ROOT / relative
            output = destination / relative
            output.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, output)
            hashes[str(relative)] = file_hash(output)
        receipt = {
            "extension": "brainstorm-os",
            "version": VERSION,
            "installed_at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
            "destination": str(destination),
            "backup": str(backup) if backup else None,
            "hashes": hashes,
        }
        (destination / "installation-receipt.json").write_text(
            json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
        )
        return receipt
    except Exception:
        if destination.exists():
            shutil.rmtree(destination)
        if backup and backup.exists():
            backup.rename(destination)
        raise


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("target", help="Existing Omega OS root directory")
    parser.add_argument("--dry-run", action="store_true", help="Print the exact installation plan without writing")
    parser.add_argument("--force", action="store_true", help="Back up an existing Brainstorm OS extension and replace it")
    args = parser.parse_args()
    try:
        target = validate_target(Path(args.target))
        result = plan(target) if args.dry_run else install(target, args.force)
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 0
    except (OSError, ValueError) as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
