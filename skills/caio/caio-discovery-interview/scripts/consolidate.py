#!/usr/bin/env python3
"""
consolidate.py — turn N individual discovery bundles into a company-level picture.

Usage:
    python consolidate.py --input <folder-of-zips-and/or-folders> --out <outputs-dir>

Reads every metadata.json it can find (inside .zip bundles or extracted folders),
then writes:
    company-rollup.md    human-readable roll-up for the CAIO
    company-rollup.json  machine-readable aggregate

It only uses metadata.json (the structured index each interview produces), so it
stays fast and privacy-aware: if a person was anonymized, their name is already null.
"""

import argparse
import json
import zipfile
from collections import Counter, defaultdict
from pathlib import Path


def _load_from_zip(path: Path):
    try:
        with zipfile.ZipFile(path) as zf:
            for name in zf.namelist():
                if name.endswith("metadata.json"):
                    return json.loads(zf.read(name).decode("utf-8"))
    except Exception:  # noqa: BLE001
        return None
    return None


def collect(input_dir: Path) -> list[dict]:
    metas = []
    for p in sorted(input_dir.iterdir()):
        if p.is_file() and p.suffix.lower() == ".zip":
            m = _load_from_zip(p)
            if m:
                metas.append(m)
        elif p.is_dir():
            mp = p / "metadata.json"
            if mp.exists():
                try:
                    metas.append(json.loads(mp.read_text(encoding="utf-8")))
                except Exception:  # noqa: BLE001
                    pass
    return metas


def label(meta: dict) -> str:
    person = meta.get("person", {})
    name = " ".join(x for x in [person.get("first_name"), person.get("last_name")] if x)
    pos = person.get("position") or person.get("job_family") or "Unknown"
    return f"{name} ({pos})" if name else pos


def roll_up(metas: list[dict]) -> dict:
    tools, ai_tools, shadow = Counter(), Counter(), Counter()
    ai_appetite, sensitive = Counter(), Counter()
    frictions_by_family = defaultdict(list)
    edges = []  # (from, to) reporting + handoff edges
    feelings = []
    scale = []  # per-person ROI multiplier view
    total_rep_hours = 0.0
    people = []

    for m in metas:
        person = m.get("person", {})
        idx = m.get("index", {})
        fam = person.get("job_family", "OTHER")
        who = label(m)
        people.append({"who": who, "family": fam, "company": person.get("company")})

        for t in idx.get("tools", []) or []:
            if t and not str(t).startswith("{{"):
                tools[t] += 1
        for t in idx.get("ai_tools_today", []) or []:
            if t and not str(t).startswith("{{"):
                ai_tools[t] += 1
        for t in idx.get("shadow_it", []) or []:
            if t and not str(t).startswith("{{"):
                shadow[t] += 1
        for t in idx.get("sensitive_data", []) or []:
            if t and not str(t).startswith("{{"):
                sensitive[t] += 1
        ap = idx.get("ai_appetite")
        if ap and not str(ap).startswith("{{"):
            ai_appetite[ap] += 1
        for f in idx.get("top_frictions", []) or []:
            if f and not str(f).startswith("{{"):
                frictions_by_family[fam].append(f)

        # reporting edge
        rep = person.get("reports_to")
        if rep and not str(rep).startswith("{{"):
            edges.append((who, rep, "reports_to"))
        # handoff edges
        ho = m.get("handoffs", {}) or {}
        for src in ho.get("upstream_from", []) or []:
            if src and not str(src).startswith("{{"):
                edges.append((src, who, "handoff"))
        for dst in ho.get("downstream_to", []) or []:
            if dst and not str(dst).startswith("{{"):
                edges.append((who, dst, "handoff"))

        cur, ideal = idx.get("current_feeling_score"), idx.get("ideal_feeling_score")
        try:
            feelings.append({"who": who, "current": float(cur), "ideal": float(ideal)})
        except (TypeError, ValueError):
            pass
        # repetitive hours + ROI multiplier (rep_hours × people in same role)
        rh = hc = None
        try:
            rh = float(idx.get("repetitive_hours_per_week_est"))
            total_rep_hours += rh
        except (TypeError, ValueError):
            pass
        try:
            hc = float(person.get("role_headcount"))
        except (TypeError, ValueError):
            pass
        if rh is not None:
            scale.append({
                "who": who,
                "rep_hours": rh,
                "role_headcount": hc,
                "team_hours_per_week": round(rh * hc, 1) if hc else None,
            })

    return {
        "people_count": len(metas),
        "people": people,
        "tools_cited": tools.most_common(),
        "ai_tools_in_use": ai_tools.most_common(),
        "shadow_it_in_the_wild": shadow.most_common(),
        "sensitive_data_handled": sensitive.most_common(),
        "ai_appetite": ai_appetite.most_common(),
        "frictions_by_family": {k: v for k, v in frictions_by_family.items()},
        "org_edges": edges,
        "feelings": feelings,
        "scale_roi": scale,
        "repetitive_hours_per_week_total_est": round(total_rep_hours, 1),
    }


def to_md(r: dict) -> str:
    L = []
    L.append("# Company Roll-up — Discovery Interviews\n")
    L.append(f"_{r['people_count']} interview(s) consolidated._\n")

    L.append("## Estimated repetitive time across interviewees")
    L.append(f"- **≈ {r['repetitive_hours_per_week_total_est']} h/week** of repetitive work captured (sum of estimates).\n")

    L.append("## Tools cited (by frequency)")
    L += [f"- {t} ×{n}" for t, n in r["tools_cited"]] or ["- _(none captured)_"]
    L.append("\n## AI tools already in use")
    L += [f"- {t} ×{n}" for t, n in r["ai_tools_in_use"]] or ["- _(none)_"]
    L.append("\n## Shadow IT in the wild (unofficial — watch for risk)")
    L += [f"- {t} ×{n}" for t, n in r["shadow_it_in_the_wild"]] or ["- _(none captured)_"]

    L.append("\n## Sensitive data handled (compliance / what AI must not see)")
    L += [f"- {t} ×{n}" for t, n in r["sensitive_data_handled"]] or ["- _(none captured)_"]

    L.append("\n## AI appetite (who to pilot with first)")
    L += [f"- {a}: {n}" for a, n in r["ai_appetite"]] or ["- _(not captured)_"]

    L.append("\n## ROI multiplier (repetitive time × people in same role)")
    if r["scale_roi"]:
        for s in r["scale_roi"]:
            hc = s["role_headcount"]
            tw = s["team_hours_per_week"]
            tail = f" × {hc:.0f} in role = ~{tw} h/week team-wide" if hc else " (role headcount unknown)"
            L.append(f"- {s['who']}: ~{s['rep_hours']} h/week{tail}")
    else:
        L.append("- _(no estimates captured)_")

    L.append("\n## Frictions by job family")
    if r["frictions_by_family"]:
        for fam, fr in r["frictions_by_family"].items():
            L.append(f"### {fam}")
            L += [f"- {x}" for x in fr]
    else:
        L.append("- _(none captured)_")

    L.append("\n## Reconstructed reporting & handoff map")
    if r["org_edges"]:
        L += [f"- {a} → {b}  _({kind})_" for a, b, kind in r["org_edges"]]
    else:
        L.append("- _(no edges captured — fill reports_to / handoffs in interviews)_")

    L.append("\n## Current → Ideal feeling spread")
    if r["feelings"]:
        for f in r["feelings"]:
            L.append(f"- {f['who']}: {f['current']:.0f} → {f['ideal']:.0f}")
    else:
        L.append("- _(no scores captured)_")

    L.append("\n---\n_Built from metadata.json of each bundle. Re-run as more interviews land._")
    return "\n".join(L) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser(description="Consolidate many discovery bundles.")
    ap.add_argument("--input", required=True, help="Folder with .zip bundles and/or folders.")
    ap.add_argument("--out", required=True, help="Output directory for the roll-up.")
    args = ap.parse_args()

    in_dir = Path(args.input).expanduser().resolve()
    out_dir = Path(args.out).expanduser().resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    if not in_dir.is_dir():
        print(f"[x] Not a folder: {in_dir}")
        return 2

    metas = collect(in_dir)
    if not metas:
        print(f"[x] No metadata.json found in {in_dir}")
        return 1

    r = roll_up(metas)
    (out_dir / "company-rollup.json").write_text(
        json.dumps(r, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    (out_dir / "company-rollup.md").write_text(to_md(r), encoding="utf-8")
    print(f"[ok] Consolidated {r['people_count']} interview(s).")
    print(f"     {out_dir/'company-rollup.md'}")
    print(f"     {out_dir/'company-rollup.json'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
