#!/usr/bin/env python3
"""Deterministic state, quality, versioning, and handoff helper for Brainstorm {OS}."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import uuid
from copy import deepcopy
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


VERSION = "3.0.0"
COLLECTIONS = {
    "sources": ("BS-SRC", {"active", "stale", "conflicted", "superseded"}),
    "frames": ("BS-FRM", {"candidate", "surviving", "selected", "rejected", "superseded"}),
    "genomes": ("BS-GEN", {"draft", "active", "selected", "superseded"}),
    "ideas": ("BS-IDEA", {"candidate", "surviving", "selected", "rejected", "parked", "superseded"}),
    "surfaces": ("BS-SRF", {"candidate", "selected", "rejected", "deferred", "experiment-first"}),
    "incubations": ("BS-INC", {"dormant", "triggered", "resurrected", "retired"}),
    "hypotheses": ("BS-HYP", {"untested", "supported", "weakened", "falsified", "research-needed"}),
    "arguments": ("BS-ARG", {"active", "accepted", "rebutted", "superseded"}),
    "tensions": ("BS-TEN", {"open", "resolved", "accepted", "deferred", "blocked"}),
    "decisions": ("BS-DEC", {"locked", "provisional", "experiment-first", "deferred", "rejected", "superseded"}),
    "experiments": ("BS-EXP", {"queued", "running", "passed", "failed", "inconclusive", "cancelled"}),
    "questions": ("BS-QUE", {"open", "resolved", "deferred", "blocked"}),
}
SESSION_STATUSES = {
    "BRAINSTORM IN PROGRESS",
    "BRAINSTORM BLOCKED",
    "BRAINSTORM CONVERGED — HANDOFF READY",
    "BRAINSTORM PARKED",
}
HANDOFF_TARGETS = {"research", "blueprint", "decision", "creative"}
CONFIDENCE = {"low", "medium", "high"}
SURFACE_TYPES = {
    "mobile", "web", "desktop", "multi-surface", "chat", "api-agent",
    "ambient-wearable", "physical-spatial", "human-service", "no-interface",
}


def now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def read(path: Path) -> dict[str, Any]:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise ValueError(f"Session not found: {path}") from exc
    except json.JSONDecodeError as exc:
        raise ValueError(f"Invalid JSON in {path}: {exc}") from exc


def write(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def csv_values(values: list[str] | None) -> list[str]:
    result: list[str] = []
    for value in values or []:
        result.extend(part.strip() for part in value.split(",") if part.strip())
    return list(dict.fromkeys(result))


def new_session(title: str, domain: str, depth: str, project_id: str | None = None) -> dict[str, Any]:
    stamp = now()
    return {
        "schema_version": VERSION,
        "meta": {
            "session_id": str(uuid.uuid4()),
            "project_id": project_id,
            "title": title,
            "domain": domain,
            "depth": depth.upper(),
            "concept_version": "0.1.0",
            "status": "BRAINSTORM IN PROGRESS",
            "current_stage": "recover",
            "created_at": stamp,
            "updated_at": stamp,
        },
        "frame": {
            "idea": "",
            "desired_change": "",
            "actors": [],
            "constraints": [],
            "non_goals": [],
            "success_signals": [],
            "locked_core": [],
            "central_tension": "",
            "highest_impact_unknown": "",
        },
        "founder_dna": {
            "obsessions": [],
            "beliefs": [],
            "taste_markers": [],
            "anti_patterns": [],
            "unfair_insights": [],
            "energy_preferences": [],
            "signature_tension": "",
            "confirmation_status": "unconfirmed",
        },
        "council": {
            "chambers": ["imagination", "evolution", "council"],
            "core_cells": ["expansion", "reality", "adversarial"],
            "specialists": [],
            "independence_preserved": None,
            "cross_examination_completed": False,
        },
        "evolution": {
            "generations": [],
            "current_generation": 0,
            "selection_pressures": [],
            "genetic_diversity_warning": False,
        },
        "surface_lab": {
            "applicability": "unknown",
            "selected_surface_ids": [],
            "primary_surface_id": None,
            "canonical_state_owner": "",
            "role_map": {},
            "multi_surface_rationale": "",
            "next_surface_trigger": "",
        },
        "portfolio": {
            "active_idea_ids": [],
            "coherence_thesis": "",
            "shared_primitives": [],
            "conflicts": [],
        },
        **{name: [] for name in COLLECTIONS},
        "parking_lot": [],
        "rounds": [],
        "lineage": {"snapshots": []},
        "quality": {"latest_audit": None, "history": []},
        "handoff": {"target": None, "readiness": "not-ready", "gaps": [], "last_export": None},
    }


def migrate_data(data: dict[str, Any]) -> dict[str, Any]:
    source_version = data.get("schema_version")
    if source_version == VERSION:
        return data
    if source_version not in {"1.0.0", "2.0.0"}:
        raise ValueError(f"Unsupported schema migration: {source_version!r} → {VERSION}")
    migrated = deepcopy(data)
    stamp = now()
    migrated["schema_version"] = VERSION
    migrated.setdefault("meta", {})
    migrated["meta"].setdefault("session_id", str(uuid.uuid4()))
    migrated["meta"].setdefault("project_id", None)
    migrated["meta"]["updated_at"] = stamp
    migrated.setdefault("frame", {})
    migrated["frame"].setdefault("central_tension", "")
    migrated["frame"].setdefault("highest_impact_unknown", "")
    migrated.setdefault("founder_dna", {
        "obsessions": [], "beliefs": [], "taste_markers": [], "anti_patterns": [],
        "unfair_insights": [], "energy_preferences": [], "signature_tension": "",
        "confirmation_status": "unconfirmed",
    })
    migrated.setdefault("council", {})
    migrated["council"].setdefault("chambers", ["imagination", "evolution", "council"])
    migrated["council"].setdefault("core_cells", ["expansion", "reality", "adversarial"])
    migrated["council"].setdefault("specialists", [])
    migrated["council"].setdefault("independence_preserved", None)
    migrated["council"].setdefault("cross_examination_completed", False)
    migrated.setdefault("evolution", {"generations": [], "current_generation": 0, "selection_pressures": [], "genetic_diversity_warning": False})
    migrated.setdefault("surface_lab", {
        "applicability": "unknown", "selected_surface_ids": [], "primary_surface_id": None,
        "canonical_state_owner": "", "role_map": {}, "multi_surface_rationale": "",
        "next_surface_trigger": "",
    })
    migrated.setdefault("portfolio", {"active_idea_ids": [], "coherence_thesis": "", "shared_primitives": [], "conflicts": []})
    migrated.setdefault("lineage", {"snapshots": []})
    migrated.setdefault("quality", {"latest_audit": None, "history": []})
    migrated.setdefault("handoff", {"target": None, "readiness": "not-ready", "gaps": [], "last_export": None})
    migrated["handoff"].setdefault("last_export", None)
    for name in COLLECTIONS:
        migrated.setdefault(name, [])
        for item in migrated[name]:
            item.setdefault("tags", [])
            item.setdefault("relations", [])
            item.setdefault("updated_at", item.get("created_at", stamp))
    for round_item in migrated.setdefault("rounds", []):
        round_item.setdefault("material", True)
        round_item.setdefault("revisions", [])
    return migrated


def next_id(data: dict[str, Any], collection: str) -> str:
    prefix, _ = COLLECTIONS[collection]
    numbers = []
    pattern = re.compile(rf"^{re.escape(prefix)}-(\d{{3,}})$")
    for item in data.get(collection, []):
        match = pattern.match(str(item.get("id", "")))
        if match:
            numbers.append(int(match.group(1)))
    return f"{prefix}-{max(numbers, default=0) + 1:03d}"


def all_ids(data: dict[str, Any]) -> set[str]:
    return {str(item.get("id")) for name in COLLECTIONS for item in data.get(name, []) if item.get("id")}


def validate(data: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    if data.get("schema_version") != VERSION:
        errors.append(f"schema_version must be {VERSION}; run migrate for legacy sessions")
    meta = data.get("meta")
    if not isinstance(meta, dict):
        errors.append("meta must be an object")
    else:
        if meta.get("status") not in SESSION_STATUSES:
            errors.append(f"invalid session status: {meta.get('status')!r}")
        if not str(meta.get("session_id", "")).strip():
            errors.append("meta.session_id is required")
        if not str(meta.get("title", "")).strip():
            errors.append("meta.title is required")

    seen: set[str] = set()
    pending_relations: list[tuple[str, str]] = []
    for collection, (prefix, statuses) in COLLECTIONS.items():
        items = data.get(collection)
        if not isinstance(items, list):
            errors.append(f"{collection} must be an array")
            continue
        for index, item in enumerate(items):
            where = f"{collection}[{index}]"
            if not isinstance(item, dict):
                errors.append(f"{where} must be an object")
                continue
            item_id = str(item.get("id", ""))
            if not re.fullmatch(rf"{re.escape(prefix)}-\d{{3,}}", item_id):
                errors.append(f"{where}.id must match {prefix}-NNN")
            elif item_id in seen:
                errors.append(f"duplicate id: {item_id}")
            else:
                seen.add(item_id)
            if not str(item.get("statement", "")).strip():
                errors.append(f"{where}.statement is required")
            if item.get("status") not in statuses:
                errors.append(f"{where}.status is invalid: {item.get('status')!r}")
            if item.get("confidence") not in CONFIDENCE:
                errors.append(f"{where}.confidence must be low, medium, or high")
            for relation in item.get("relations", []):
                pending_relations.append((where, str(relation)))
            parent_id = item.get("parent_id")
            if parent_id:
                pending_relations.append((where, str(parent_id)))
            target_id = item.get("target_id")
            if target_id:
                pending_relations.append((where, str(target_id)))

    for where, relation in pending_relations:
        if relation not in seen:
            errors.append(f"{where} references unknown id: {relation}")

    rounds = data.get("rounds")
    if not isinstance(rounds, list):
        errors.append("rounds must be an array")
    else:
        for index, round_item in enumerate(rounds):
            if not isinstance(round_item, dict) or not round_item.get("name") or not round_item.get("delta"):
                errors.append(f"rounds[{index}] requires name and delta")

    for required in ("frame", "founder_dna", "council", "evolution", "surface_lab", "portfolio", "lineage", "quality", "handoff"):
        if not isinstance(data.get(required), dict):
            errors.append(f"{required} must be an object")
    surface_lab = data.get("surface_lab", {})
    if isinstance(surface_lab, dict):
        if surface_lab.get("applicability") not in {"unknown", "applicable", "not-applicable"}:
            errors.append("surface_lab.applicability must be unknown, applicable, or not-applicable")
        surface_ids = {item.get("id") for item in data.get("surfaces", [])}
        for item_id in surface_lab.get("selected_surface_ids", []):
            if item_id not in surface_ids:
                errors.append(f"surface_lab references unknown surface id: {item_id}")
        primary = surface_lab.get("primary_surface_id")
        if primary and primary not in surface_ids:
            errors.append(f"surface_lab.primary_surface_id references unknown id: {primary}")
    portfolio = data.get("portfolio", {})
    if isinstance(portfolio, dict):
        idea_ids = {item.get("id") for item in data.get("ideas", [])}
        for item_id in portfolio.get("active_idea_ids", []):
            if item_id not in idea_ids:
                errors.append(f"portfolio references unknown idea id: {item_id}")
    return errors


def selected_ideas(data: dict[str, Any]) -> list[dict[str, Any]]:
    return [item for item in data.get("ideas", []) if item.get("status") == "selected"]


def critical_open_tensions(data: dict[str, Any]) -> list[dict[str, Any]]:
    return [
        item for item in data.get("tensions", [])
        if item.get("status") in {"open", "blocked"} and "critical" in item.get("tags", [])
    ]


def score_band(value: int, thresholds: tuple[int, int, int, int]) -> int:
    return sum(value >= threshold for threshold in thresholds)


def audit_data(data: dict[str, Any]) -> dict[str, Any]:
    frame = data.get("frame", {})
    founder_dna = data.get("founder_dna", {})
    frames = data.get("frames", [])
    genomes = data.get("genomes", [])
    ideas = data.get("ideas", [])
    surfaces = data.get("surfaces", [])
    incubations = data.get("incubations", [])
    hypotheses = data.get("hypotheses", [])
    arguments = data.get("arguments", [])
    tensions = data.get("tensions", [])
    decisions = data.get("decisions", [])
    experiments = data.get("experiments", [])
    rounds = data.get("rounds", [])
    evolution = data.get("evolution", {})
    surface_lab = data.get("surface_lab", {})
    portfolio = data.get("portfolio", {})
    depth = data.get("meta", {}).get("depth")
    ideation_depth = depth in {"IMAGINATION", "DEEP"}
    selected = selected_ideas(data)
    con_arguments = [item for item in arguments if item.get("polarity") == "con"]
    tagged = lambda *tags: [item for item in arguments if set(tags) & set(item.get("tags", []))]
    hypothesis_falsifiers = [item for item in hypotheses if str(item.get("falsifier", "")).strip()]
    experiment_thresholds = [item for item in experiments if str(item.get("threshold", "")).strip()]
    traceable = [item for name in COLLECTIONS for item in data.get(name, []) if item.get("relations")]
    recombined = [item for item in ideas if item.get("parent_id") or len(item.get("relations", [])) >= 2]
    material_rounds = [item for item in rounds if item.get("material", True)]
    revised_rounds = [item for item in rounds if item.get("revisions")]
    generation_count = len(evolution.get("generations", []))
    multi_locus_mutations = [
        item for item in genomes
        if len([tag for tag in item.get("tags", []) if tag.startswith("locus:")]) >= 2
    ]
    surprise_items = [
        item for item in ideas
        if "anomaly" in item.get("tags", []) or "valuable-surprise" in item.get("tags", [])
    ]
    selected_surfaces = [item for item in surfaces if item.get("status") == "selected"]
    selected_multi = [item for item in selected_surfaces if item.get("surface_type") == "multi-surface"]
    founder_dna_parts = sum(bool(founder_dna.get(field)) for field in (
        "obsessions", "beliefs", "taste_markers", "anti_patterns", "unfair_insights", "signature_tension"
    ))

    intent_parts = sum(bool(frame.get(field)) for field in ("desired_change", "constraints", "non_goals", "locked_core"))
    evidence_ratio = (sum(bool(item.get("provenance")) for item in hypotheses) / len(hypotheses)) if hypotheses else 0
    falsifier_ratio = (len(hypothesis_falsifiers) / len(hypotheses)) if hypotheses else 0
    decision_ratio = (sum(bool(item.get("rationale")) for item in decisions) / len(decisions)) if decisions else 0
    raw_frame_originality = min(4, score_band(len(frames), (1, 3, 5, 8)) + (1 if any("inversion" in item.get("tags", []) for item in frames) else 0))
    raw_evolutionary_depth = min(4, score_band(generation_count, (1, 2, 3, 5)) + (1 if multi_locus_mutations else 0))
    raw_valuable_surprise = min(4, score_band(len(surprise_items), (1, 2, 3, 5)) + (1 if founder_dna_parts >= 3 else 0))
    if surface_lab.get("applicability") == "not-applicable":
        surface_fitness = 4
    elif surface_lab.get("applicability") == "unknown":
        surface_fitness = 2
    else:
        surface_fitness = min(4, (1 if surfaces else 0) + (1 if len({item.get('surface_type') for item in surfaces}) >= 3 else 0) + (1 if selected_surfaces else 0) + (1 if surface_lab.get("primary_surface_id") else 0))
    portfolio_is_single = len(portfolio.get("active_idea_ids", [])) <= 1 and len(selected) <= 1
    portfolio_score = 4 if portfolio_is_single and not any((portfolio.get("coherence_thesis"), portfolio.get("shared_primitives"), portfolio.get("conflicts"))) else min(4, (1 if portfolio.get("active_idea_ids") else 0) + (1 if portfolio.get("coherence_thesis") else 0) + (1 if portfolio.get("shared_primitives") else 0) + (1 if portfolio.get("conflicts") else 0))
    scores = {
        "intent_fidelity": intent_parts,
        "direction_diversity": min(4, score_band(len(ideas), (1, 3, 5, 8)) + (1 if any("inversion" in item.get("tags", []) for item in ideas) else 0)),
        "dissent_quality": min(4, score_band(len(con_arguments), (1, 2, 4, 6)) + (1 if revised_rounds else 0)),
        "evidence_hygiene": min(4, (1 if data.get("sources") else 0) + round(evidence_ratio * 2) + (1 if any(item.get("status") == "research-needed" for item in hypotheses) else 0)),
        "tension_integrity": min(4, score_band(len(tensions), (1, 2, 3, 5)) + (1 if any(item.get("status") != "open" for item in tensions) else 0)),
        "recombination_quality": min(4, score_band(len(recombined), (1, 2, 3, 4))),
        "decision_clarity": min(4, (1 if selected else 0) + round(decision_ratio * 2) + (1 if any(item.get("revisit_trigger") for item in decisions) else 0)),
        "falsifiability": min(4, round(falsifier_ratio * 2) + (1 if experiments else 0) + (1 if experiment_thresholds else 0)),
        "feasibility_realism": min(4, score_band(len(tagged("feasibility", "operations", "scale")), (1, 2, 3, 5))),
        "consequence_depth": min(4, score_band(len(tagged("incentives", "abuse", "second-order", "anti-goal")), (1, 2, 3, 5))),
        "traceability": min(4, score_band(len(traceable), (1, 3, 6, 10))),
        "handoff_readiness": min(4, (2 if selected else 0) + (1 if decisions else 0) + (1 if not critical_open_tensions(data) else 0)),
        "frame_originality": raw_frame_originality if ideation_depth else max(3, raw_frame_originality),
        "evolutionary_depth": raw_evolutionary_depth if ideation_depth else 4,
        "valuable_surprise": raw_valuable_surprise if ideation_depth else max(2, raw_valuable_surprise),
        "surface_fitness": surface_fitness,
        "portfolio_coherence": portfolio_score,
    }
    scores["direction_diversity"] = min(scores["direction_diversity"], 4)
    average = round(sum(scores.values()) / len(scores), 2)
    warnings: list[str] = []
    if len(ideas) > 12:
        warnings.append("More than 12 ideas: cluster before expanding further")
    if len(rounds) > 5 and len(material_rounds) < len(rounds) * 0.6:
        warnings.append("Challenge loop has too many non-material rounds")
    if not con_arguments:
        warnings.append("No explicit causal dissent is recorded")
    if selected and not revised_rounds:
        warnings.append("No material self-revision is recorded before selection")
    if critical_open_tensions(data):
        warnings.append("Critical open tensions block convergence")
    if hypotheses and not hypothesis_falsifiers:
        warnings.append("Hypotheses lack explicit falsifiers")
    if experiments and not experiment_thresholds:
        warnings.append("Experiments lack decision thresholds")
    if ideation_depth and not frames:
        warnings.append("Imagination/Deep mode has no recorded frame fission")
    if depth == "DEEP" and generation_count < 2:
        warnings.append("Deep mode has fewer than two evolutionary generations")
    if surface_lab.get("applicability") == "applicable" and not selected_surfaces:
        warnings.append("Surface Lab is applicable but no surface is selected")
    if selected_multi and (not surface_lab.get("role_map") or not surface_lab.get("canonical_state_owner")):
        warnings.append("Selected multi-surface concept lacks role map or canonical state owner")
    surface_gate = surface_lab.get("applicability") != "applicable" or (
        bool(selected_surfaces) and not (selected_multi and (not surface_lab.get("role_map") or not surface_lab.get("canonical_state_owner")))
    )
    gate = "pass" if average >= 3.0 and not critical_open_tensions(data) and scores["dissent_quality"] >= 3 and scores["decision_clarity"] >= 3 and surface_gate else "fail"
    return {
        "evaluated_at": now(),
        "structural_only": True,
        "scores": scores,
        "average": average,
        "convergence_gate": gate,
        "warnings": warnings,
        "counts": {
            "ideas": len(ideas),
            "selected_ideas": len(selected),
            "con_arguments": len(con_arguments),
            "hypotheses_with_falsifier": len(hypothesis_falsifiers),
            "experiments_with_threshold": len(experiment_thresholds),
            "material_rounds": len(material_rounds),
            "revised_rounds": len(revised_rounds),
            "critical_open_tensions": len(critical_open_tensions(data)),
            "frames": len(frames),
            "genomes": len(genomes),
            "generations": generation_count,
            "valuable_surprise_ideas": len(surprise_items),
            "surface_candidates": len(surfaces),
            "selected_surfaces": len(selected_surfaces),
            "incubated_ideas": len(incubations),
        },
    }


def increment_version(version: str, level: str) -> str:
    match = re.fullmatch(r"(\d+)\.(\d+)\.(\d+)", version)
    if not match:
        raise ValueError(f"Invalid semantic version: {version}")
    major, minor, patch = map(int, match.groups())
    if level == "major":
        return f"{major + 1}.0.0"
    if level == "minor":
        return f"{major}.{minor + 1}.0"
    return f"{major}.{minor}.{patch + 1}"


def canonical_hash(data: dict[str, Any]) -> str:
    copy = deepcopy(data)
    copy.get("meta", {}).pop("updated_at", None)
    payload = json.dumps(copy, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


def markdown_export(data: dict[str, Any]) -> str:
    meta = data["meta"]
    frame = data["frame"]
    lines = [
        f"# {meta['title']} — Brainstorm {{OS}}",
        "",
        f"- Status: `{meta['status']}`",
        f"- Concept version: `{meta['concept_version']}`",
        f"- Mode: `{meta['depth']}`",
        f"- Domain: `{meta['domain']}`",
        "",
        "## Frame",
        "",
        f"**Idea:** {frame.get('idea') or '—'}",
        "",
        f"**Desired change:** {frame.get('desired_change') or '—'}",
        "",
        f"**Central tension:** {frame.get('central_tension') or '—'}",
        "",
    ]
    for label, key in (("Actors", "actors"), ("Constraints", "constraints"), ("Non-goals", "non_goals"), ("Success signals", "success_signals"), ("Locked core", "locked_core")):
        lines.extend([f"### {label}", ""])
        values = frame.get(key, [])
        lines.extend([f"- {value}" for value in values] or ["- —"])
        lines.append("")
    dna = data.get("founder_dna", {})
    lines.extend(["## Founder DNA", "", f"**Signature tension:** {dna.get('signature_tension') or '—'}", ""])
    for label, key in (("Obsessions", "obsessions"), ("Beliefs", "beliefs"), ("Taste markers", "taste_markers"), ("Anti-patterns", "anti_patterns"), ("Unfair insights", "unfair_insights")):
        values = dna.get(key, [])
        lines.extend([f"### {label}", ""] + ([f"- {value}" for value in values] or ["- —"]) + [""])
    for collection in COLLECTIONS:
        lines.extend([f"## {collection.replace('_', ' ').title()}", ""])
        items = data.get(collection, [])
        if not items:
            lines.extend(["—", ""])
            continue
        for item in items:
            suffix = f" · confidence: {item.get('confidence', 'low')}"
            lines.append(f"- `{item['id']}` [{item['status']}]{suffix} — {item['statement']}")
            if item.get("rationale"):
                lines.append(f"  - Rationale: {item['rationale']}")
            if item.get("falsifier"):
                lines.append(f"  - Falsifier: {item['falsifier']}")
            if item.get("threshold"):
                lines.append(f"  - Threshold: {item['threshold']}")
        lines.append("")
    lines.extend(["## Challenge rounds", ""])
    for item in data.get("rounds", []):
        lines.append(f"- Round {item.get('number')}: **{item['name']}** — {item['delta']}")
    if not data.get("rounds"):
        lines.append("—")
    lines.extend(["", "## Evolutionary generations", ""])
    for generation in data.get("evolution", {}).get("generations", []):
        lines.append(f"- G{generation.get('number')} **{generation.get('name')}** · pressure: {generation.get('selection_pressure')} — {generation.get('delta')}")
    if not data.get("evolution", {}).get("generations"):
        lines.append("—")
    lab = data.get("surface_lab", {})
    lines.extend([
        "", "## Surface Lab", "",
        f"- Applicability: `{lab.get('applicability', 'unknown')}`",
        f"- Primary surface: `{lab.get('primary_surface_id') or '—'}`",
        f"- Canonical state owner: {lab.get('canonical_state_owner') or '—'}",
        f"- Next-surface trigger: {lab.get('next_surface_trigger') or '—'}",
        "", "## Quality audit", "", "```json",
        json.dumps(data.get("quality", {}).get("latest_audit"), ensure_ascii=False, indent=2), "```", "",
    ])
    return "\n".join(lines)


def build_handoff(data: dict[str, Any], target: str) -> dict[str, Any]:
    selected = selected_ideas(data)
    common = {
        "contract_version": VERSION,
        "target": target,
        "generated_at": now(),
        "session": {key: data["meta"].get(key) for key in ("session_id", "project_id", "title", "domain", "concept_version", "status")},
        "frame": data["frame"],
        "founder_dna": data.get("founder_dna", {}),
        "frames": data.get("frames", []),
        "genomes": data.get("genomes", []),
        "selected_ideas": selected,
        "strongest_alternatives": [item for item in data.get("ideas", []) if item.get("status") == "surviving"],
        "decisions": data.get("decisions", []),
        "tensions": data.get("tensions", []),
        "sources": data.get("sources", []),
        "quality_audit": data.get("quality", {}).get("latest_audit"),
        "evolution": data.get("evolution", {}),
        "surface_lab": data.get("surface_lab", {}),
        "surfaces": data.get("surfaces", []),
        "portfolio": data.get("portfolio", {}),
        "incubations": data.get("incubations", []),
    }
    if target == "research":
        common.update({
            "hypotheses": data.get("hypotheses", []),
            "questions": data.get("questions", []),
            "experiments": data.get("experiments", []),
            "return_contract": "Map findings to BS-HYP/BS-DEC/BS-IDEA/BS-TEN IDs and run an Evidence Return round",
        })
    elif target == "blueprint":
        common.update({
            "concept_truth": selected[0]["statement"] if selected else None,
            "hypotheses": data.get("hypotheses", []),
            "experiments": data.get("experiments", []),
            "rejected_ideas": [item for item in data.get("ideas", []) if item.get("status") == "rejected"],
            "scope_exclusion": "No screens, complete requirements, APIs, or implementation architecture are defined by this handoff",
        })
    elif target == "decision":
        common.update({"arguments": data.get("arguments", []), "questions": data.get("questions", [])})
    elif target == "creative":
        common.update({
            "territories": [item for item in data.get("ideas", []) if item.get("status") in {"selected", "surviving"}],
            "arguments": data.get("arguments", []),
            "anti_goals": [item for item in data.get("arguments", []) if "anti-goal" in item.get("tags", [])],
        })
    return common


def cmd_init(args: argparse.Namespace) -> int:
    output = Path(args.output)
    if output.exists() and not args.force:
        raise ValueError(f"Refusing to overwrite existing session: {output}; use --force")
    write(output, new_session(args.title, args.domain, args.depth, args.project_id))
    print(output)
    return 0


def cmd_migrate(args: argparse.Namespace) -> int:
    source = Path(args.session)
    data = migrate_data(read(source))
    output = Path(args.output) if args.output else source
    if output != source and output.exists() and not args.force:
        raise ValueError(f"Refusing to overwrite: {output}; use --force")
    write(output, data)
    print(f"migrated to {VERSION}: {output}")
    return 0


def cmd_frame(args: argparse.Namespace) -> int:
    path = Path(args.session)
    data = read(path)
    frame = data["frame"]
    scalar_fields = ("idea", "desired_change", "central_tension", "highest_impact_unknown")
    for field in scalar_fields:
        value = getattr(args, field)
        if value is not None:
            frame[field] = value
    for argument, field in (("actor", "actors"), ("constraint", "constraints"), ("non_goal", "non_goals"), ("success_signal", "success_signals"), ("locked_core", "locked_core")):
        values = csv_values(getattr(args, argument))
        frame[field] = list(dict.fromkeys(frame.get(field, []) + values))
    data["meta"]["updated_at"] = now()
    write(path, data)
    print("frame updated")
    return 0


def cmd_dna(args: argparse.Namespace) -> int:
    path = Path(args.session)
    data = read(path)
    dna = data["founder_dna"]
    for argument, field in (
        ("obsession", "obsessions"),
        ("belief", "beliefs"),
        ("taste_marker", "taste_markers"),
        ("anti_pattern", "anti_patterns"),
        ("unfair_insight", "unfair_insights"),
        ("energy_preference", "energy_preferences"),
    ):
        values = csv_values(getattr(args, argument))
        dna[field] = list(dict.fromkeys(dna.get(field, []) + values))
    if args.signature_tension is not None:
        dna["signature_tension"] = args.signature_tension
    if args.confirmation_status is not None:
        dna["confirmation_status"] = args.confirmation_status
    data["meta"]["updated_at"] = now()
    write(path, data)
    print("founder DNA updated")
    return 0


def cmd_add(args: argparse.Namespace) -> int:
    path = Path(args.session)
    data = read(path)
    _, statuses = COLLECTIONS[args.collection]
    if args.status not in statuses:
        raise ValueError(f"Invalid status for {args.collection}: {args.status}; choose {', '.join(sorted(statuses))}")
    relations = csv_values(args.relates_to)
    known = all_ids(data)
    for relation in relations + ([args.parent_id] if args.parent_id else []) + ([args.target_id] if args.target_id else []):
        if relation not in known:
            raise ValueError(f"Unknown related id: {relation}")
    stamp = now()
    item = {
        "id": next_id(data, args.collection),
        "statement": args.statement,
        "status": args.status,
        "confidence": args.confidence,
        "rationale": args.rationale,
        "provenance": args.provenance,
        "tags": csv_values(args.tag),
        "relations": relations,
        "created_at": stamp,
        "updated_at": stamp,
    }
    optional = (
        "parent_id", "target_id", "polarity", "falsifier", "threshold",
        "revisit_trigger", "resurrection_trigger", "surface_type", "generation",
    )
    for key in optional:
        value = getattr(args, key)
        if value:
            item[key] = value
    data[args.collection].append(item)
    data["meta"]["updated_at"] = stamp
    write(path, data)
    print(item["id"])
    return 0


def parse_role_map(values: list[str] | None) -> dict[str, str]:
    roles: dict[str, str] = {}
    for value in values or []:
        if "=" not in value:
            raise ValueError(f"Surface role must use TYPE=ROLE: {value}")
        key, role = value.split("=", 1)
        key, role = key.strip(), role.strip()
        if not key or not role:
            raise ValueError(f"Surface role must use TYPE=ROLE: {value}")
        roles[key] = role
    return roles


def cmd_surface(args: argparse.Namespace) -> int:
    path = Path(args.session)
    data = read(path)
    if args.surface_type not in SURFACE_TYPES:
        raise ValueError(f"Unknown surface type: {args.surface_type}")
    relations = csv_values(args.relates_to)
    known = all_ids(data)
    for relation in relations:
        if relation not in known:
            raise ValueError(f"Unknown related id: {relation}")
    stamp = now()
    item = {
        "id": next_id(data, "surfaces"),
        "statement": args.statement,
        "status": args.status,
        "confidence": args.confidence,
        "rationale": args.rationale,
        "provenance": args.provenance,
        "tags": csv_values(args.tag),
        "relations": relations,
        "surface_type": args.surface_type,
        "strengths": csv_values(args.strength),
        "costs": csv_values(args.cost),
        "created_at": stamp,
        "updated_at": stamp,
    }
    if args.threshold:
        item["threshold"] = args.threshold
    data["surfaces"].append(item)
    lab = data["surface_lab"]
    lab["applicability"] = "applicable"
    roles = parse_role_map(args.role)
    if roles:
        lab["role_map"].update(roles)
    for field in ("canonical_state_owner", "multi_surface_rationale", "next_surface_trigger"):
        value = getattr(args, field)
        if value is not None:
            lab[field] = value
    if args.status == "selected":
        lab["selected_surface_ids"] = list(dict.fromkeys(lab.get("selected_surface_ids", []) + [item["id"]]))
        if args.primary or not lab.get("primary_surface_id"):
            lab["primary_surface_id"] = item["id"]
    data["meta"]["updated_at"] = stamp
    write(path, data)
    print(item["id"])
    return 0


def cmd_surface_config(args: argparse.Namespace) -> int:
    path = Path(args.session)
    data = read(path)
    lab = data["surface_lab"]
    if args.applicability is not None:
        lab["applicability"] = args.applicability
    roles = parse_role_map(args.role)
    if roles:
        lab["role_map"].update(roles)
    for field in ("canonical_state_owner", "multi_surface_rationale", "next_surface_trigger"):
        value = getattr(args, field)
        if value is not None:
            lab[field] = value
    data["meta"]["updated_at"] = now()
    write(path, data)
    print("surface lab updated")
    return 0


def cmd_evolve(args: argparse.Namespace) -> int:
    path = Path(args.session)
    data = read(path)
    parents = csv_values(args.parent)
    survivors = csv_values(args.survivor)
    extinct = csv_values(args.extinct)
    known = all_ids(data)
    for item_id in parents + survivors + extinct:
        if item_id not in known:
            raise ValueError(f"Unknown evolution id: {item_id}")
    evolution = data["evolution"]
    number = len(evolution["generations"]) + 1
    generation = {
        "number": number,
        "name": args.name,
        "selection_pressure": args.selection_pressure,
        "operators": csv_values(args.operator),
        "parent_ids": parents,
        "survivor_ids": survivors,
        "extinct_ids": extinct,
        "delta": args.delta,
        "at": now(),
    }
    evolution["generations"].append(generation)
    evolution["current_generation"] = number
    if args.selection_pressure not in evolution["selection_pressures"]:
        evolution["selection_pressures"].append(args.selection_pressure)
    evolution["genetic_diversity_warning"] = args.genetic_diversity_warning
    data["meta"]["current_stage"] = "evolve"
    data["meta"]["updated_at"] = now()
    write(path, data)
    print(f"generation {number}")
    return 0


def cmd_portfolio(args: argparse.Namespace) -> int:
    path = Path(args.session)
    data = read(path)
    portfolio = data["portfolio"]
    active = csv_values(args.active_idea)
    known_ideas = {item.get("id") for item in data.get("ideas", [])}
    for item_id in active:
        if item_id not in known_ideas:
            raise ValueError(f"Unknown portfolio idea id: {item_id}")
    portfolio["active_idea_ids"] = list(dict.fromkeys(portfolio.get("active_idea_ids", []) + active))
    if args.coherence_thesis is not None:
        portfolio["coherence_thesis"] = args.coherence_thesis
    portfolio["shared_primitives"] = list(dict.fromkeys(portfolio.get("shared_primitives", []) + csv_values(args.shared_primitive)))
    portfolio["conflicts"] = list(dict.fromkeys(portfolio.get("conflicts", []) + csv_values(args.conflict)))
    data["meta"]["updated_at"] = now()
    write(path, data)
    print("portfolio updated")
    return 0


def cmd_checkpoint(args: argparse.Namespace) -> int:
    path = Path(args.session)
    data = read(path)
    data["rounds"].append({
        "number": len(data["rounds"]) + 1,
        "name": args.name,
        "lens": args.lens,
        "delta": args.delta,
        "material": not args.non_material,
        "revisions": csv_values(args.revision),
        "at": now(),
    })
    data["meta"]["current_stage"] = args.stage
    if args.status:
        if args.status not in SESSION_STATUSES:
            raise ValueError(f"Invalid session status: {args.status}")
        data["meta"]["status"] = args.status
    data["meta"]["updated_at"] = now()
    write(path, data)
    print(f"checkpoint {len(data['rounds'])}")
    return 0


def cmd_audit(args: argparse.Namespace) -> int:
    path = Path(args.session)
    data = read(path)
    result = audit_data(data)
    data["quality"]["latest_audit"] = result
    data["quality"]["history"].append(result)
    data["meta"]["updated_at"] = now()
    write(path, data)
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0 if result["convergence_gate"] == "pass" or not args.require_pass else 1


def cmd_freeze(args: argparse.Namespace) -> int:
    path = Path(args.session)
    data = read(path)
    errors = validate(data)
    if errors:
        raise ValueError("Cannot freeze invalid session: " + "; ".join(errors))
    if not selected_ideas(data):
        raise ValueError("Cannot freeze without a selected idea")
    if critical_open_tensions(data) and not args.allow_open_critical:
        raise ValueError("Cannot freeze with critical open tensions; resolve them or use --allow-open-critical")
    old_version = data["meta"]["concept_version"]
    new_version = increment_version(old_version, args.level)
    snapshot = {
        "from_version": old_version,
        "version": new_version,
        "frozen_at": now(),
        "note": args.note,
        "session_hash": canonical_hash(data),
        "selected_ids": [item["id"] for item in selected_ideas(data)],
        "locked_decision_ids": [item["id"] for item in data.get("decisions", []) if item.get("status") == "locked"],
    }
    data["lineage"]["snapshots"].append(snapshot)
    data["meta"]["concept_version"] = new_version
    data["meta"]["updated_at"] = now()
    if args.converged:
        latest_audit = data.get("quality", {}).get("latest_audit") or audit_data(data)
        if latest_audit.get("convergence_gate") != "pass" and not args.allow_quality_fail:
            raise ValueError("Cannot declare convergence while the quality gate fails; run audit and repair, or use --allow-quality-fail for an explicit governed exception")
        data["meta"]["status"] = "BRAINSTORM CONVERGED — HANDOFF READY"
    write(path, data)
    print(json.dumps(snapshot, ensure_ascii=False, indent=2))
    return 0


def cmd_export(args: argparse.Namespace) -> int:
    data = read(Path(args.session))
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(markdown_export(data), encoding="utf-8")
    print(output)
    return 0


def cmd_handoff(args: argparse.Namespace) -> int:
    path = Path(args.session)
    data = read(path)
    errors = validate(data)
    if errors:
        raise ValueError("Cannot hand off invalid session: " + "; ".join(errors))
    if args.target in {"blueprint", "decision", "creative"} and not selected_ideas(data) and not args.force:
        raise ValueError(f"{args.target} handoff requires a selected idea; use --force only for an explicit partial handoff")
    if args.target == "blueprint" and critical_open_tensions(data) and not args.force:
        raise ValueError("Blueprint handoff is blocked by critical open tensions")
    if args.target == "blueprint" and not data.get("lineage", {}).get("snapshots") and not args.force:
        raise ValueError("Blueprint handoff requires a frozen concept version")
    output = Path(args.output)
    write(output, build_handoff(data, args.target))
    data["handoff"].update({"target": args.target, "readiness": "ready" if not args.force else "partial", "last_export": str(output), "gaps": []})
    data["meta"]["updated_at"] = now()
    write(path, data)
    print(output)
    return 0


def summary(data: dict[str, Any]) -> dict[str, Any]:
    return {
        "title": data.get("meta", {}).get("title"),
        "session_id": data.get("meta", {}).get("session_id"),
        "status": data.get("meta", {}).get("status"),
        "stage": data.get("meta", {}).get("current_stage"),
        "concept_version": data.get("meta", {}).get("concept_version"),
        "counts": {name: len(data.get(name, [])) for name in COLLECTIONS},
        "rounds": len(data.get("rounds", [])),
        "snapshots": len(data.get("lineage", {}).get("snapshots", [])),
        "quality_gate": (data.get("quality", {}).get("latest_audit") or {}).get("convergence_gate"),
        "generations": len(data.get("evolution", {}).get("generations", [])),
        "surface_applicability": data.get("surface_lab", {}).get("applicability"),
        "primary_surface_id": data.get("surface_lab", {}).get("primary_surface_id"),
        "handoff": data.get("handoff", {}),
    }


def cmd_summary(args: argparse.Namespace) -> int:
    print(json.dumps(summary(read(Path(args.session))), ensure_ascii=False, indent=2))
    return 0


def cmd_validate(args: argparse.Namespace) -> int:
    errors = validate(read(Path(args.session)))
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print("Brainstorm session is structurally valid")
    return 0


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    sub = root.add_subparsers(dest="command", required=True)

    init = sub.add_parser("init", help="Create a v3 session JSON file")
    init.add_argument("output")
    init.add_argument("--title", required=True)
    init.add_argument("--domain", default="general")
    init.add_argument("--project-id")
    init.add_argument("--depth", choices=["spark", "imagination", "council", "deep", "red-team", "converge", "audit"], default="council")
    init.add_argument("--force", action="store_true")
    init.set_defaults(func=cmd_init)

    migrate = sub.add_parser("migrate", help="Migrate a v1 or v2 session to v3")
    migrate.add_argument("session")
    migrate.add_argument("--output")
    migrate.add_argument("--force", action="store_true")
    migrate.set_defaults(func=cmd_migrate)

    frame = sub.add_parser("frame", help="Update the session frame")
    frame.add_argument("session")
    frame.add_argument("--idea")
    frame.add_argument("--desired-change")
    frame.add_argument("--central-tension")
    frame.add_argument("--highest-impact-unknown")
    frame.add_argument("--actor", action="append")
    frame.add_argument("--constraint", action="append")
    frame.add_argument("--non-goal", action="append")
    frame.add_argument("--success-signal", action="append")
    frame.add_argument("--locked-core", action="append")
    frame.set_defaults(func=cmd_frame)

    dna = sub.add_parser("dna", help="Update Founder DNA and signature tension")
    dna.add_argument("session")
    dna.add_argument("--obsession", action="append")
    dna.add_argument("--belief", action="append")
    dna.add_argument("--taste-marker", action="append")
    dna.add_argument("--anti-pattern", action="append")
    dna.add_argument("--unfair-insight", action="append")
    dna.add_argument("--energy-preference", action="append")
    dna.add_argument("--signature-tension")
    dna.add_argument("--confirmation-status", choices=["unconfirmed", "partially-confirmed", "confirmed"])
    dna.set_defaults(func=cmd_dna)

    add = sub.add_parser("add", help="Add a typed ledger item")
    add.add_argument("session")
    add.add_argument("collection", choices=sorted(COLLECTIONS))
    add.add_argument("--statement", required=True)
    add.add_argument("--status", required=True)
    add.add_argument("--confidence", choices=sorted(CONFIDENCE), default="low")
    add.add_argument("--rationale", default="")
    add.add_argument("--provenance", default="")
    add.add_argument("--tag", action="append")
    add.add_argument("--relates-to", action="append")
    add.add_argument("--parent-id")
    add.add_argument("--target-id")
    add.add_argument("--polarity", choices=["pro", "con", "mixed"])
    add.add_argument("--falsifier")
    add.add_argument("--threshold")
    add.add_argument("--revisit-trigger")
    add.add_argument("--resurrection-trigger")
    add.add_argument("--surface-type", choices=sorted(SURFACE_TYPES))
    add.add_argument("--generation", type=int)
    add.set_defaults(func=cmd_add)

    surface = sub.add_parser("surface", help="Record and select a product surface candidate")
    surface.add_argument("session")
    surface.add_argument("--type", dest="surface_type", required=True, choices=sorted(SURFACE_TYPES))
    surface.add_argument("--statement", required=True)
    surface.add_argument("--status", choices=sorted(COLLECTIONS["surfaces"][1]), default="candidate")
    surface.add_argument("--confidence", choices=sorted(CONFIDENCE), default="low")
    surface.add_argument("--rationale", default="")
    surface.add_argument("--provenance", default="")
    surface.add_argument("--tag", action="append")
    surface.add_argument("--relates-to", action="append")
    surface.add_argument("--strength", action="append")
    surface.add_argument("--cost", action="append")
    surface.add_argument("--threshold")
    surface.add_argument("--primary", action="store_true")
    surface.add_argument("--role", action="append", help="TYPE=ROLE; repeat for multi-surface")
    surface.add_argument("--canonical-state-owner")
    surface.add_argument("--multi-surface-rationale")
    surface.add_argument("--next-surface-trigger")
    surface.set_defaults(func=cmd_surface)

    surface_config = sub.add_parser("surface-config", help="Configure Surface Lab applicability and multi-surface roles")
    surface_config.add_argument("session")
    surface_config.add_argument("--applicability", choices=["unknown", "applicable", "not-applicable"])
    surface_config.add_argument("--role", action="append", help="TYPE=ROLE")
    surface_config.add_argument("--canonical-state-owner")
    surface_config.add_argument("--multi-surface-rationale")
    surface_config.add_argument("--next-surface-trigger")
    surface_config.set_defaults(func=cmd_surface_config)

    evolve = sub.add_parser("evolve", help="Record an evolutionary generation and its selection pressure")
    evolve.add_argument("session")
    evolve.add_argument("--name", required=True)
    evolve.add_argument("--selection-pressure", required=True)
    evolve.add_argument("--operator", action="append")
    evolve.add_argument("--parent", action="append")
    evolve.add_argument("--survivor", action="append")
    evolve.add_argument("--extinct", action="append")
    evolve.add_argument("--delta", required=True)
    evolve.add_argument("--genetic-diversity-warning", action="store_true")
    evolve.set_defaults(func=cmd_evolve)

    portfolio = sub.add_parser("portfolio", help="Record active ideas and portfolio coherence")
    portfolio.add_argument("session")
    portfolio.add_argument("--active-idea", action="append")
    portfolio.add_argument("--coherence-thesis")
    portfolio.add_argument("--shared-primitive", action="append")
    portfolio.add_argument("--conflict", action="append")
    portfolio.set_defaults(func=cmd_portfolio)

    checkpoint = sub.add_parser("checkpoint", help="Record a challenge-cycle delta")
    checkpoint.add_argument("session")
    checkpoint.add_argument("--name", required=True)
    checkpoint.add_argument("--lens", default="general")
    checkpoint.add_argument("--delta", required=True)
    checkpoint.add_argument("--revision", action="append")
    checkpoint.add_argument("--stage", default="challenge")
    checkpoint.add_argument("--status")
    checkpoint.add_argument("--non-material", action="store_true")
    checkpoint.set_defaults(func=cmd_checkpoint)

    audit = sub.add_parser("audit", help="Compute structural Brainstorm quality gates")
    audit.add_argument("session")
    audit.add_argument("--require-pass", action="store_true")
    audit.set_defaults(func=cmd_audit)

    freeze = sub.add_parser("freeze", help="Version and freeze the selected concept")
    freeze.add_argument("session")
    freeze.add_argument("--level", choices=["patch", "minor", "major"], default="minor")
    freeze.add_argument("--note", default="")
    freeze.add_argument("--converged", action="store_true")
    freeze.add_argument("--allow-open-critical", action="store_true")
    freeze.add_argument("--allow-quality-fail", action="store_true")
    freeze.set_defaults(func=cmd_freeze)

    export = sub.add_parser("export", help="Export a readable Markdown session")
    export.add_argument("session")
    export.add_argument("output")
    export.set_defaults(func=cmd_export)

    handoff = sub.add_parser("handoff", help="Export a structured downstream handoff")
    handoff.add_argument("session")
    handoff.add_argument("target", choices=sorted(HANDOFF_TARGETS))
    handoff.add_argument("output")
    handoff.add_argument("--force", action="store_true")
    handoff.set_defaults(func=cmd_handoff)

    show = sub.add_parser("summary", help="Print a compact session summary")
    show.add_argument("session")
    show.set_defaults(func=cmd_summary)

    check = sub.add_parser("validate", help="Validate a v3 session JSON file")
    check.add_argument("session")
    check.set_defaults(func=cmd_validate)
    return root


def main() -> int:
    try:
        args = parser().parse_args()
        return args.func(args)
    except ValueError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
