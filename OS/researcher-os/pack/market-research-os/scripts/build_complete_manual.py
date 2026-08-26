#!/usr/bin/env python3
"""Compile the progressive Market Research {OS} sources into one review manual."""

from __future__ import annotations

import argparse
import datetime as dt
import os
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
ORDER = [
    "SKILL.md",
    "references/system-prompt.md",
    "references/research-contract.md",
    "references/orchestration-and-gates.md",
    "references/methods-and-frameworks.md",
    "references/source-and-tool-registry.md",
    "references/data-acquisition-and-compliance.md",
    "references/experiments-and-primary-research.md",
    "references/scoring-and-decision.md",
    "references/vertical-playbooks.md",
    "references/response-and-continuation.md",
    "references/omega-os-integration.md",
    "references/agency-service-model.md",
    "references/evidence-source-notes.md",
    "assets/omega-os.manifest.json",
    "assets/market-research-tools.json",
    "assets/market-research-state.schema.json",
    "assets/blueprint-input-manifest.schema.json",
    "assets/market-research-role-prompts.json",
    "assets/research-brief.template.yaml",
    "assets/research-plan.template.yaml",
    "assets/source-preflight.template.yaml",
    "assets/experiment.template.yaml",
    "assets/customer-interview.template.md",
    "assets/survey-questionnaire.template.md",
    "assets/competitor-profile.template.yaml",
    "assets/voc-codebook.template.csv",
    "assets/evidence-ledger.template.csv",
    "assets/decision-scorecard.template.csv",
    "assets/market-model.template.csv",
    "assets/report.template.md",
    "agents/openai.yaml",
    "scripts/market_research_os.py",
    "scripts/install_omega_os.py",
]


def fence_for(path: Path) -> str:
    return {
        ".json": "json", ".yaml": "yaml", ".yml": "yaml", ".py": "python",
        ".csv": "csv",
    }.get(path.suffix.lower(), "")


def render() -> str:
    generated = dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat()
    lines = [
        "# Market Research {OS} — Complete Omega OS Manual",
        "",
        f"Generated: `{generated}`",
        "",
        "This compiled review manual contains the complete installable skill, master system prompt, research contracts, methods, source/scraping governance, experiments, scoring, vertical playbooks, Omega integration, schemas, functions, templates, and deterministic scripts. The installable folder remains the canonical modular source.",
        "",
        "## File inventory",
        "",
    ]
    for relative in ORDER:
        path = ROOT / relative
        lines.append(f"- `{relative}` — {path.stat().st_size} bytes")
    for index, relative in enumerate(ORDER, 1):
        path = ROOT / relative
        content = path.read_text(encoding="utf-8")
        lines.extend(["", f"# Part {index:02d} — `{relative}`", ""])
        if path.suffix.lower() in {".md"}:
            lines.append(content.rstrip())
        else:
            fence = fence_for(path)
            lines.extend([f"```{fence}", content.rstrip(), "```"])
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description="Build the complete Market Research OS manual")
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    output = Path(args.output).expanduser().resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    content = render()
    fd, tmp_name = tempfile.mkstemp(prefix=output.name + ".", suffix=".tmp", dir=output.parent)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            handle.write(content)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(tmp_name, output)
    finally:
        if os.path.exists(tmp_name):
            os.unlink(tmp_name)
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
