#!/usr/bin/env python3
"""Render analysis.json -> self-contained HTML report (both themes, R-NODASH safe)."""
import sys, json, html, re, datetime, os

def strip_dashes(s: str) -> str:
    return s.replace("—", ", ").replace("–", "-")

def md_inline(s: str) -> str:
    s = html.escape(s)
    s = re.sub(r"\[([^\]]+)\]\((https?://[^\s)]+)\)", r'<a href="\2">\1</a>', s)
    s = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", s)
    s = re.sub(r"`([^`]+)`", r"<code>\1</code>", s)
    return s

def md_to_html(md: str) -> str:
    out, in_list = [], False
    for raw in md.splitlines():
        line = raw.rstrip()
        if not line.strip():
            if in_list: out.append("</ul>"); in_list = False
            continue
        if line.startswith("### "):
            if in_list: out.append("</ul>"); in_list = False
            out.append(f"<h3>{md_inline(line[4:])}</h3>")
        elif line.startswith("## "):
            if in_list: out.append("</ul>"); in_list = False
            out.append(f"<h2>{md_inline(line[3:])}</h2>")
        elif line.lstrip().startswith(("- ", "* ")):
            if not in_list: out.append("<ul>"); in_list = True
            out.append(f"<li>{md_inline(line.lstrip()[2:])}</li>")
        else:
            if in_list: out.append("</ul>"); in_list = False
            out.append(f"<p>{md_inline(line)}</p>")
    if in_list: out.append("</ul>")
    return "\n".join(out)

def score_class(n):
    try: n = float(n)
    except Exception: return "mid"
    return "hi" if n >= 7 else ("mid" if n >= 4 else "lo")

def main():
    # argv: analysis.json [published.jsonl] [armed|-] [mock|-]
    data = json.load(open(sys.argv[1]))
    if len(sys.argv) > 2 and sys.argv[2] not in ("", "-") and os.path.exists(sys.argv[2]):
        data["_published"] = [json.loads(l) for l in open(sys.argv[2]) if l.strip()]
    data["_armed"] = len(sys.argv) > 3 and sys.argv[3] == "armed"
    data["_mock"] = len(sys.argv) > 4 and sys.argv[4] == "mock"
    date = data.get("date") or datetime.date.today().isoformat()
    digest = md_to_html(data.get("digest_md", "_No digest._"))
    imps = data.get("improvements", [])
    cands = data.get("candidates", [])
    published = data.get("_published", [])
    armed = data.get("_armed", False)
    mock = data.get("_mock", False)

    rows = ""
    for i in sorted(imps, key=lambda x: -float(x.get("integratable_to_omega", 0) or 0)):
        sc = i.get("integratable_to_omega", 0)
        rows += (f'<tr><td><span class="score {score_class(sc)}">{html.escape(str(sc))}</span></td>'
                 f'<td><strong>{md_inline(i.get("title",""))}</strong><div class="why">{md_inline(i.get("why_it_matters",""))}</div>'
                 f'<div class="src"><a href="{html.escape(i.get("source_url","#"))}">source</a>'
                 f' · @{html.escape(i.get("author",""))}</div></td></tr>')
    if not rows:
        rows = '<tr><td colspan="2" class="empty">No integratable improvement surfaced today.</td></tr>'

    tw = ""
    pub_fps = {p.get("fingerprint") for p in published}
    for c in cands:
        state = "published" if c.get("fingerprint") in pub_fps else "draft"
        badge = ('<span class="pill pub">published</span>' if state == "published"
                 else '<span class="pill draft">draft</span>')
        tw += (f'<div class="tweet">{badge}<div class="tt">{md_inline(c.get("text",""))}</div></div>')
    if not tw:
        tw = '<div class="empty">No tweet cleared the gate today.</div>'

    banner = ""
    if mock:
        banner = '<div class="warn">MOCK run: X read used fixtures, nothing published.</div>'
    elif not armed:
        banner = '<div class="warn">DISARMED: tweets are drafts only. Arm with <code>touch ~/.omega/state/ecosystem-watch/armed</code> to auto-publish.</div>'

    doc = f"""<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Ecosystem Watch {html.escape(date)}</title>
<style>
:root{{--bg:#f6f5f1;--panel:#fffefb;--ink:#1a1a17;--muted:#6b6a63;--line:#e3e1d8;--accent:#c2410c;--hi:#1f7a4d;--hi-bg:#e6f2ea;--mid:#b45309;--mid-bg:#f8ecd6;--lo:#9a938a;--lo-bg:#eeece5;--code:#f0eee7;--warn-bg:#f8ecd6}}
@media(prefers-color-scheme:dark){{:root{{--bg:#141311;--panel:#1e1c19;--ink:#ece9e1;--muted:#9c9a90;--line:#302d28;--accent:#f97347;--hi:#5cc98d;--hi-bg:#16281e;--mid:#e0a44a;--mid-bg:#2a2113;--lo:#6f6a61;--lo-bg:#232019;--code:#26231f;--warn-bg:#2a2113}}}}
:root[data-theme=dark]{{--bg:#141311;--panel:#1e1c19;--ink:#ece9e1;--muted:#9c9a90;--line:#302d28;--accent:#f97347;--hi:#5cc98d;--hi-bg:#16281e;--mid:#e0a44a;--mid-bg:#2a2113;--lo:#6f6a61;--lo-bg:#232019;--code:#26231f;--warn-bg:#2a2113}}
:root[data-theme=light]{{--bg:#f6f5f1;--panel:#fffefb;--ink:#1a1a17;--muted:#6b6a63;--line:#e3e1d8;--accent:#c2410c;--hi:#1f7a4d;--hi-bg:#e6f2ea;--mid:#b45309;--mid-bg:#f8ecd6;--lo:#9a938a;--lo-bg:#eeece5;--code:#f0eee7;--warn-bg:#f8ecd6}}
*{{box-sizing:border-box}}body{{margin:0;background:var(--bg);color:var(--ink);font-family:ui-sans-serif,-apple-system,"Segoe UI",Roboto,sans-serif;line-height:1.6}}
.wrap{{max-width:820px;margin:0 auto;padding:48px 22px 90px}}
.kick{{font-size:.72rem;letter-spacing:.16em;text-transform:uppercase;color:var(--accent);font-weight:700;margin:0 0 8px}}
h1{{font-size:2rem;margin:0 0 4px;letter-spacing:-.02em}}.meta{{color:var(--muted);font-size:.85rem}}
h2{{font-size:1.2rem;margin:40px 0 6px}}h3{{font-size:1rem;margin:20px 0 2px}}
p{{margin:10px 0;max-width:66ch}}a{{color:var(--accent)}}
code{{background:var(--code);padding:.1em .4em;border-radius:6px;font-size:.86em;font-family:ui-monospace,Menlo,monospace}}
.warn{{background:var(--warn-bg);border:1px solid var(--line);border-left:4px solid var(--mid);border-radius:10px;padding:12px 16px;margin:18px 0;font-size:.92rem}}
table{{width:100%;border-collapse:collapse;margin:12px 0;font-size:.93rem}}td{{padding:12px 10px;border-bottom:1px solid var(--line);vertical-align:top}}
.why{{color:var(--muted);font-size:.9rem;margin:4px 0}}.src{{font-size:.8rem;color:var(--muted)}}
.score{{display:inline-block;min-width:2ch;text-align:center;font-weight:800;font-variant-numeric:tabular-nums;padding:3px 8px;border-radius:8px}}
.score.hi{{background:var(--hi-bg);color:var(--hi)}}.score.mid{{background:var(--mid-bg);color:var(--mid)}}.score.lo{{background:var(--lo-bg);color:var(--lo)}}
.tweet{{background:var(--panel);border:1px solid var(--line);border-radius:12px;padding:14px 16px;margin:12px 0}}
.tt{{margin-top:6px}}.pill{{font-size:.68rem;font-weight:700;padding:2px 9px;border-radius:999px}}
.pill.pub{{background:var(--hi-bg);color:var(--hi)}}.pill.draft{{background:var(--mid-bg);color:var(--mid)}}
.empty{{color:var(--muted);font-style:italic;padding:14px 0}}
.foot{{color:var(--muted);font-size:.8rem;margin-top:50px;border-top:1px solid var(--line);padding-top:16px}}
</style></head><body><div class="wrap">
<p class="kick">OmegaOS · Agent-Ecosystem Watch</p>
<h1>Veille du {html.escape(date)}</h1>
<p class="meta">{len(imps)} ameliorations · {len(cands)} tweets · {len(published)} publies</p>
{banner}
<h2>Digest</h2>{digest}
<h2>Ameliorations (triees par integrabilite OmegaOS)</h2>
<table><tbody>{rows}</tbody></table>
<h2>Tweets</h2>{tw}
<div class="foot">Genere par agent-ecosystem-watch · lecture X via last30days/ScrapeCreators · publication via omega-zernio (@Agentik_os). Aucune integration OmegaOS sans validation operateur.</div>
</div></body></html>"""
    sys.stdout.write(strip_dashes(doc))

if __name__ == "__main__":
    main()
