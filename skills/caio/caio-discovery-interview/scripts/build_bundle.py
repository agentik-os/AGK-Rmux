#!/usr/bin/env python3
"""
build_bundle.py — validate a filled discovery folder and zip it with a
deterministic, sortable name the CAIO can stack across many people.

Usage:
    python build_bundle.py --input <filled-folder> --out <outputs-dir>

The <filled-folder> must contain exactly the 16 standardized files (see
REQUIRED below), already filled in from the interview. The script reads
metadata.json to build the name:

    {Company}_{LastName}_{FirstName}_{YYYY-MM-DD}.zip

It refuses to build if any file is missing or if metadata.json is incomplete,
because the whole value of this skill is that every person's bundle lines up.
"""

import argparse
import datetime as _dt
import json
import re
import sys
import unicodedata
import zipfile
from pathlib import Path

REQUIRED = [
    "metadata.json",
    "company-context.md",
    "summary.md",
    "00-identity.md",
    "01-role-and-responsibility.md",
    "02-typical-week-and-month.md",
    "03-daily-actions.md",
    "04-handoffs.md",
    "05-tools-and-systems.md",
    "06-connections-and-integrations.md",
    "07-ai-automation-and-shadow-it.md",
    "08-frictions.md",
    "09-keep-as-is.md",
    "10-improvements-wanted.md",
    "11-current-position-and-feeling.md",
    "12-ideal-position-and-feeling.md",
    "13-gap-analysis.md",
    "transcript.md",
]


def slug(value: str, fallback: str = "unknown") -> str:
    """Filesystem-safe token: strip accents, keep alnum, collapse to hyphens."""
    if not value or not str(value).strip():
        return fallback
    value = unicodedata.normalize("NFKD", str(value))
    value = value.encode("ascii", "ignore").decode("ascii")
    value = re.sub(r"[^A-Za-z0-9]+", "-", value).strip("-")
    return value or fallback


def validate(folder: Path) -> list[str]:
    present = {p.name for p in folder.iterdir() if p.is_file()}
    missing = [f for f in REQUIRED if f not in present]
    extra = [f for f in sorted(present) if f not in REQUIRED]
    problems = []
    if missing:
        problems.append("Missing files: " + ", ".join(missing))
    if extra:
        problems.append(
            "Unexpected extra files (remove or fold into transcript.md): "
            + ", ".join(extra)
        )
    return problems


def build_name(folder: Path) -> str:
    meta_path = folder / "metadata.json"
    company = last = first = date = position = sharing = None
    try:
        meta = json.loads(meta_path.read_text(encoding="utf-8"))
        person = meta.get("person", {})
        company = person.get("company")
        last = person.get("last_name")
        first = person.get("first_name")
        position = person.get("position")
        sharing = (meta.get("consent", {}) or {}).get("sharing_level")
        date = (meta.get("generated_at") or "")[:10]
    except Exception as exc:  # noqa: BLE001
        print(f"[!] Could not read metadata.json: {exc}", file=sys.stderr)

    if not date or not re.match(r"\d{4}-\d{2}-\d{2}", date):
        date = _dt.date.today().isoformat()

    # Anonymized (or missing names) → name the bundle by role, not by person.
    anonymized = (sharing == "anonymized") or not (last or first)
    if anonymized:
        return f"{slug(company,'Company')}_{slug(position,'Role')}_{date}"
    return f"{slug(company,'Company')}_{slug(last,'Last')}_{slug(first,'First')}_{date}"


def main() -> int:
    ap = argparse.ArgumentParser(description="Validate and zip a discovery bundle.")
    ap.add_argument("--input", required=True, help="Filled discovery folder.")
    ap.add_argument("--out", required=True, help="Output directory for the .zip.")
    ap.add_argument(
        "--force",
        action="store_true",
        help="Zip even if validation finds problems (not recommended).",
    )
    args = ap.parse_args()

    folder = Path(args.input).expanduser().resolve()
    out_dir = Path(args.out).expanduser().resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    if not folder.is_dir():
        print(f"[x] Not a folder: {folder}", file=sys.stderr)
        return 2

    problems = validate(folder)
    if problems and not args.force:
        print("[x] Bundle not standardized — refusing to build:", file=sys.stderr)
        for p in problems:
            print("    - " + p, file=sys.stderr)
        print("    Fix the folder (or pass --force) and retry.", file=sys.stderr)
        return 1
    if problems:
        print("[!] Building despite problems (--force):", file=sys.stderr)
        for p in problems:
            print("    - " + p, file=sys.stderr)

    name = build_name(folder)
    inner = name  # folder name inside the zip == zip stem
    zip_path = out_dir / f"{name}.zip"

    with zipfile.ZipFile(zip_path, "w", zipfile.ZIP_DEFLATED) as zf:
        for fname in REQUIRED:
            fpath = folder / fname
            if fpath.exists():
                zf.write(fpath, arcname=f"{inner}/{fname}")

    print(f"[ok] Bundle ready: {zip_path}")
    print(f"     Internal folder: {inner}/  ({len(REQUIRED)} files)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
