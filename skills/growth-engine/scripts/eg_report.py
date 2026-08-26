#!/usr/bin/env python3
"""argv: radar.json queue.jsonl results.jsonl date armed|disarmed -> self-contained HTML."""
import sys, json, html, os

def strip_dashes(s): return s.replace("—", ", ").replace("–", "-")
def esc(s): return html.escape(str(s or ""))

radar = json.load(open(sys.argv[1]))
queue = [json.loads(l) for l in open(sys.argv[2])] if os.path.exists(sys.argv[2]) else []
results = []
if os.path.exists(sys.argv[3]):
    for l in open(sys.argv[3]):
        try: results.append(json.loads(l))
        except: pass
date = sys.argv[4]; armed = sys.argv[5] == "armed"

posted = {r.get("fingerprint") for r in results if r.get("ok") and r.get("action") == "reply"}
queued_reply_fps = {q["fingerprint"] for q in queue if q.get("type") == "reply"}
n_reply = sum(1 for q in queue if q.get("type") == "reply")
n_like = sum(1 for q in queue if q.get("type") == "like")
did_reply = sum(1 for r in results if r.get("ok") and r.get("action") == "reply")
did_like = sum(1 for r in results if r.get("ok") and r.get("action") == "like")

rows = ""
for o in sorted(radar.get("opportunities", []), key=lambda x: -float(x.get("leverage_score", 0) or 0)):
    fp = o.get("fingerprint")
    if fp in posted: badge = '<span class="pill pub">posted</span>'
    elif fp in queued_reply_fps: badge = '<span class="pill q">queued</span>'
    else: badge = '<span class="pill held">held by gate</span>'
    rows += (f'<tr><td><span class="score">{esc(o.get("leverage_score"))}</span></td>'
             f'<td>{badge}<div class="rt">{esc(strip_dashes(o.get("reply_text","")))}</div>'
             f'<div class="src"><a href="{esc(o.get("target_url","#"))}">@{esc(o.get("author"))} · target</a>'
             f' · {esc(o.get("rationale",""))}</div></td></tr>')
if not rows:
    rows = '<tr><td colspan="2" class="empty">No opportunity surfaced.</td></tr>'

banner = ('<div class="warn">DISARMED or no session: replies are drafts only. Arm with '
          '<code>touch ~/.omega/state/growth-engine/armed</code> and provide the X session.</div>') if not armed or not results else ""

doc = f"""<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1"><title>Growth Engine {esc(date)}</title>
<style>
:root{{--bg:#f6f5f1;--panel:#fffefb;--ink:#1a1a17;--muted:#6b6a63;--line:#e3e1d8;--accent:#2563a8;--pub:#1f7a4d;--pub-bg:#e6f2ea;--q:#b45309;--q-bg:#f8ecd6;--held:#9a938a;--held-bg:#eeece5;--code:#f0eee7;--warn-bg:#f8ecd6}}
@media(prefers-color-scheme:dark){{:root{{--bg:#141311;--panel:#1e1c19;--ink:#ece9e1;--muted:#9c9a90;--line:#302d28;--accent:#5b9bd8;--pub:#5cc98d;--pub-bg:#16281e;--q:#e0a44a;--q-bg:#2a2113;--held:#6f6a61;--held-bg:#232019;--code:#26231f;--warn-bg:#2a2113}}}}
:root[data-theme=dark]{{--bg:#141311;--panel:#1e1c19;--ink:#ece9e1;--muted:#9c9a90;--line:#302d28;--accent:#5b9bd8;--pub:#5cc98d;--pub-bg:#16281e;--q:#e0a44a;--q-bg:#2a2113;--held:#6f6a61;--held-bg:#232019;--code:#26231f;--warn-bg:#2a2113}}
:root[data-theme=light]{{--bg:#f6f5f1;--panel:#fffefb;--ink:#1a1a17;--muted:#6b6a63;--line:#e3e1d8;--accent:#2563a8;--pub:#1f7a4d;--pub-bg:#e6f2ea;--q:#b45309;--q-bg:#f8ecd6;--held:#9a938a;--held-bg:#eeece5;--code:#f0eee7;--warn-bg:#f8ecd6}}
*{{box-sizing:border-box}}body{{margin:0;background:var(--bg);color:var(--ink);font-family:ui-sans-serif,-apple-system,"Segoe UI",Roboto,sans-serif;line-height:1.6}}
.wrap{{max-width:820px;margin:0 auto;padding:48px 22px 90px}}
.kick{{font-size:.72rem;letter-spacing:.16em;text-transform:uppercase;color:var(--accent);font-weight:700;margin:0 0 8px}}
h1{{font-size:2rem;margin:0 0 4px}}.meta{{color:var(--muted);font-size:.9rem}}
h2{{font-size:1.2rem;margin:36px 0 6px}}a{{color:var(--accent)}}
code{{background:var(--code);padding:.1em .4em;border-radius:6px;font-size:.86em;font-family:ui-monospace,Menlo,monospace}}
.warn{{background:var(--warn-bg);border:1px solid var(--line);border-left:4px solid var(--q);border-radius:10px;padding:12px 16px;margin:16px 0;font-size:.92rem}}
.stat{{display:inline-block;margin-right:22px;font-size:.92rem;color:var(--muted)}}.stat b{{color:var(--ink);font-size:1.1rem}}
table{{width:100%;border-collapse:collapse;margin:12px 0;font-size:.93rem}}td{{padding:12px 10px;border-bottom:1px solid var(--line);vertical-align:top}}
.rt{{margin:2px 0}}.src{{font-size:.8rem;color:var(--muted)}}
.score{{display:inline-block;min-width:2ch;text-align:center;font-weight:800;font-variant-numeric:tabular-nums;padding:3px 8px;border-radius:8px;background:var(--held-bg);color:var(--accent)}}
.pill{{font-size:.68rem;font-weight:700;padding:2px 9px;border-radius:999px}}
.pill.pub{{background:var(--pub-bg);color:var(--pub)}}.pill.q{{background:var(--q-bg);color:var(--q)}}.pill.held{{background:var(--held-bg);color:var(--held)}}
.empty{{color:var(--muted);font-style:italic}}
.foot{{color:var(--muted);font-size:.8rem;margin-top:48px;border-top:1px solid var(--line);padding-top:16px}}
</style></head><body><div class="wrap">
<p class="kick">OmegaOS · Growth Engine · @Agentik_os</p>
<h1>Radar du {esc(date)}</h1>
<p class="meta">Croissance par présence génuine, jamais par spam.</p>
{banner}
<div style="margin:18px 0">
<span class="stat"><b>{len(radar.get('opportunities',[]))}</b> opportunités</span>
<span class="stat"><b>{n_reply}</b> replies en queue</span>
<span class="stat"><b>{n_like}</b> likes en queue</span>
<span class="stat"><b>{did_reply}</b> replies postés</span>
<span class="stat"><b>{did_like}</b> likes postés</span>
</div>
<h2>Opportunités (par levier)</h2>
<table><tbody>{rows}</tbody></table>
<div class="foot">Radar via last30days/ScrapeCreators · exécution Playwright bornée sur session @Agentik_os · gate adversarial anti-spam. Plafonds bas par design.</div>
</div></body></html>"""
sys.stdout.write(strip_dashes(doc))
