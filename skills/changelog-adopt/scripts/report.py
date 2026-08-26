#!/usr/bin/env python3
"""Render a self-contained HTML report for a Claude Changelog Adopt run (R-HTML).

Argv: analysis.json gate.json meta.json
  analysis.json — {"assessments": [...]} from the classifier.
  gate.json     — {"verdicts": [...]} from the gate (or "-" if the gate did not run).
  meta.json     — {"date","latest","last_version","armed","dryrun","dispatched":[fp,...],
                   "new_count"}.
Prints one self-contained HTML document to stdout (inline CSS, no external assets).
"""
import sys, json, html


def load(path, default):
    if path in ("-", "", None):
        return default
    try:
        with open(path) as f:
            return json.load(f)
    except Exception:
        return default


def esc(s):
    return html.escape(str(s if s is not None else ""))


def main():
    analysis = load(sys.argv[1] if len(sys.argv) > 1 else "-", {"assessments": []})
    gate = load(sys.argv[2] if len(sys.argv) > 2 else "-", {"verdicts": []})
    meta = load(sys.argv[3] if len(sys.argv) > 3 else "-", {})

    assessments = analysis.get("assessments", []) or []
    verdicts = {v.get("fingerprint"): v for v in gate.get("verdicts", []) or []}
    dispatched = set(meta.get("dispatched", []) or [])

    date = meta.get("date", "")
    latest = meta.get("latest", "?")
    last_version = meta.get("last_version", "") or "(seed)"
    armed = bool(meta.get("armed"))
    dryrun = bool(meta.get("dryrun"))
    new_count = meta.get("new_count", len(assessments))

    rank = {"high": 0, "medium": 1, "low": 2, "none": 3}
    assessments = sorted(
        assessments,
        key=lambda a: (rank.get(a.get("relevance", "none"), 3), -float(a.get("integratability", 0) or 0)),
    )

    n_high = sum(1 for a in assessments if a.get("relevance") == "high")
    n_med = sum(1 for a in assessments if a.get("relevance") == "medium")
    n_kept = sum(1 for v in verdicts.values() if v.get("keep"))

    rel_color = {"high": "#c0392b", "medium": "#b9770e", "low": "#6b7280", "none": "#9aa0a6"}

    rows = []
    for a in assessments:
        fp = a.get("fingerprint")
        rel = a.get("relevance", "none")
        v = verdicts.get(fp)
        if v is None:
            gate_badge = '<span class="b b-skip">not gated</span>'
        elif v.get("keep"):
            gate_badge = '<span class="b b-keep">gate: KEEP</span>'
        else:
            gate_badge = '<span class="b b-rej">gate: reject</span>'
        disp_badge = ' <span class="b b-disp">dispatched</span>' if fp in dispatched else ""
        scope_badge = "" if a.get("in_scope", True) else ' <span class="b b-core">needs-human (core-rust)</span>'
        gate_reason = esc(v.get("reason")) if v else ""
        rows.append(f"""
      <tr class="rel-{esc(rel)}">
        <td class="ver">{esc(a.get('version'))}</td>
        <td>
          <div class="entry">{esc(a.get('entry'))}</div>
          <div class="meta-line">
            <span class="rel" style="color:{rel_color.get(rel,'#888')}">{esc(rel).upper()}</span>
            · <span class="cat">{esc(a.get('category'))}</span>
            · surface <code>{esc(a.get('surface'))}</code>
            · int <b>{esc(a.get('integratability'))}</b>/10
            {gate_badge}{disp_badge}{scope_badge}
          </div>
          {f'<div class="proposal"><b>Proposal:</b> {esc(a.get("proposal"))}</div>' if a.get('proposal') else ''}
          {f'<div class="reason">gate: {gate_reason}</div>' if gate_reason else ''}
        </td>
      </tr>""")

    status = "ARMED" if armed else "DISARMED"
    if dryrun:
        status += " · DRY-RUN"

    print(f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Claude Changelog Adopt · {esc(date)}</title>
<style>
  :root {{ color-scheme: light dark; }}
  * {{ box-sizing: border-box; }}
  body {{ margin:0; font:15px/1.6 -apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;
         background:#fafafa; color:#1a1a1a; }}
  .wrap {{ max-width:960px; margin:0 auto; padding:28px 20px 64px; }}
  header {{ border-bottom:2px solid #e5e5e5; padding-bottom:16px; margin-bottom:24px; }}
  h1 {{ font-size:22px; margin:0 0 6px; letter-spacing:-.2px; }}
  .sub {{ color:#666; font-size:13px; }}
  .kpis {{ display:flex; flex-wrap:wrap; gap:12px; margin:20px 0 8px; }}
  .kpi {{ background:#fff; border:1px solid #e8e8e8; border-radius:10px; padding:12px 16px; min-width:120px; }}
  .kpi .n {{ font-size:24px; font-weight:700; }}
  .kpi .l {{ font-size:12px; color:#777; text-transform:uppercase; letter-spacing:.4px; }}
  .status {{ display:inline-block; font-size:12px; font-weight:700; padding:3px 10px; border-radius:20px;
             background:#eef; color:#334; border:1px solid #dde; }}
  table {{ width:100%; border-collapse:collapse; margin-top:8px; }}
  td {{ padding:14px 10px; border-bottom:1px solid #ececec; vertical-align:top; }}
  td.ver {{ white-space:nowrap; font-variant-numeric:tabular-nums; color:#888; font-size:13px; width:70px; }}
  .entry {{ font-weight:600; }}
  .meta-line {{ font-size:12.5px; color:#666; margin-top:5px; }}
  .rel {{ font-weight:700; }}
  code {{ background:#f0f0f0; padding:1px 5px; border-radius:4px; font-size:12.5px; }}
  .proposal {{ margin-top:8px; font-size:13.5px; background:#f6f8fa; border-left:3px solid #4a90d9;
               padding:8px 12px; border-radius:0 6px 6px 0; }}
  .reason {{ margin-top:6px; font-size:12.5px; color:#8a6d3b; }}
  .b {{ display:inline-block; font-size:11px; font-weight:700; padding:1px 7px; border-radius:20px; margin-left:4px; }}
  .b-keep {{ background:#e6f4ea; color:#137333; }}
  .b-rej {{ background:#fce8e6; color:#c5221f; }}
  .b-skip {{ background:#f1f3f4; color:#5f6368; }}
  .b-disp {{ background:#e8f0fe; color:#1967d2; }}
  .b-core {{ background:#fef7e0; color:#b06000; }}
  tr.rel-none {{ opacity:.62; }}
  footer {{ margin-top:32px; color:#999; font-size:12px; }}
  @media (prefers-color-scheme: dark) {{
    body {{ background:#161616; color:#e6e6e6; }}
    header {{ border-color:#333; }}
    .sub,.kpi .l,.meta-line {{ color:#9aa0a6; }}
    .kpi {{ background:#1e1e1e; border-color:#333; }}
    code {{ background:#2a2a2a; }}
    .proposal {{ background:#1c2530; border-color:#4a90d9; }}
    td {{ border-color:#2a2a2a; }}
    .status {{ background:#20242e; color:#c7d0e0; border-color:#333; }}
  }}
</style></head><body><div class="wrap">
  <header>
    <h1>Claude Changelog Adopt <span class="status">{esc(status)}</span></h1>
    <div class="sub">{esc(date)} · watching <b>anthropics/claude-code</b> ·
      latest <b>{esc(latest)}</b> · last absorbed <b>{esc(last_version)}</b> ·
      {esc(new_count)} new {'entry' if new_count == 1 else 'entries'}</div>
  </header>
  <div class="kpis">
    <div class="kpi"><div class="n">{new_count}</div><div class="l">new entries</div></div>
    <div class="kpi"><div class="n">{n_high}</div><div class="l">high relevance</div></div>
    <div class="kpi"><div class="n">{n_med}</div><div class="l">medium</div></div>
    <div class="kpi"><div class="n">{n_kept}</div><div class="l">gate-kept</div></div>
    <div class="kpi"><div class="n">{len(dispatched)}</div><div class="l">dispatched</div></div>
  </div>
  <table><tbody>{''.join(rows) if rows else '<tr><td>No new changelog entries this run.</td></tr>'}</tbody></table>
  <footer>OmegaOS · changelog-adopt · proposals are the model's judgment, not verified fact —
    the operator (or an armed + gated oracle dispatch) decides. Arm:
    <code>touch ~/.omega/state/changelog-adopt/armed</code></footer>
</div></body></html>""")


if __name__ == "__main__":
    main()
