#!/usr/bin/env bash
# Post a message (stdin or $1) to the operator's Telegram, rendered as Telegram
# HTML (bold/italic/underline/strike/code/headers→bold/links/bullets), chunked at
# 4000 chars, with an automatic plain-text fallback if the HTML ever fails to parse.
#
# Secrets stay OUT of the repo: the bot token lives in $NOVA_HOME/.bot (or
# NOVA_BOT_TOKEN / NOVA_BOT_TOKEN_FILE), the chat id in ~/.omega/nova-secrets.env
# as NOVA_CHAT_ID. Nothing here may be hardcoded.
set -uo pipefail
OMEGA="${OMEGA_DIR:-$HOME/.omega}"
LS="${NOVA_HOME:-$HOME/Station/LifeStyle}"
[ -f "$OMEGA/nova-secrets.env" ] && set -a && . "$OMEGA/nova-secrets.env" && set +a
BOT="${NOVA_BOT_TOKEN:-$(cat "${NOVA_BOT_TOKEN_FILE:-$LS/.bot}" 2>/dev/null)}"
CHAT="${NOVA_CHAT_ID:-}"
[ -n "$BOT" ]  || { echo "nova-send: bot token missing — put it in $LS/.bot (or NOVA_BOT_TOKEN in $OMEGA/nova-secrets.env)" >&2; exit 2; }
[ -n "$CHAT" ] || { echo "nova-send: NOVA_CHAT_ID missing — set it in $OMEGA/nova-secrets.env" >&2; exit 2; }
MSG="${1:-$(cat)}"
[ -z "${MSG//[[:space:]]/}" ] && exit 0
python3 - "$BOT" "$CHAT" "$MSG" <<'PY'
import sys, json, re, html, urllib.request, urllib.parse
bot, chat, msg = sys.argv[1], sys.argv[2], sys.argv[3]

def md_to_html(src: str) -> str:
    codes = []
    def stash(h): codes.append(h); return f" \x00{len(codes)-1}\x00 "
    s = re.sub(r"```[a-zA-Z0-9]*\n?([\s\S]*?)```", lambda m: stash("<pre>"+html.escape(m.group(1).rstrip("\n"))+"</pre>"), src)
    s = re.sub(r"`([^`\n]+)`", lambda m: stash("<code>"+html.escape(m.group(1))+"</code>"), s)
    s = html.escape(s)
    s = re.sub(r"\[([^\]\n]+)\]\((https?://[^\s)]+)\)", r'<a href="\2">\1</a>', s)
    s = re.sub(r"^[ \t]*#{1,6}[ \t]+(.+)$", r"<b>\1</b>", s, flags=re.M)   # headers h1-h6 -> bold
    s = re.sub(r"\+\+([^\n+]+)\+\+", r"<u>\1</u>", s)                       # ++underline++
    s = re.sub(r"\*\*([^\n*]+)\*\*", r"<b>\1</b>", s)
    s = re.sub(r"__([^\n_]+)__", r"<b>\1</b>", s)
    s = re.sub(r"~~([^\n~]+)~~", r"<s>\1</s>", s)
    s = re.sub(r"(^|[^*\w])\*([^\n*]+)\*(?!\w)", r"\1<i>\2</i>", s)
    s = re.sub(r"^[ \t]*[-*+][ \t]+", "• ", s, flags=re.M)                  # bullets
    s = re.sub(r"^[ \t]*>[ \t]?(.+)$", r"<blockquote>\1</blockquote>", s, flags=re.M)
    s = re.sub(r"\x00(\d+)\x00", lambda m: codes[int(m.group(1))], s)
    return s

def strip_tags(s): return re.sub(r"<[^>]+>", "", s).replace("&lt;","<").replace("&gt;",">").replace("&amp;","&")

def chunks(text, n=3800):
    out, cur = [], ""
    for para in text.split("\n\n"):
        if len(cur)+len(para)+2 > n:
            if cur: out.append(cur); cur=""
            while len(para) > n: out.append(para[:n]); para=para[n:]
        cur = (cur+"\n\n"+para) if cur else para
    if cur: out.append(cur)
    return out

def send(text, parse=True):
    body = {"chat_id": chat, "text": text, "disable_web_page_preview": "true"}
    if parse: body["parse_mode"] = "HTML"
    data = urllib.parse.urlencode(body).encode()
    r = urllib.request.urlopen(f"https://api.telegram.org/bot{bot}/sendMessage", data=data, timeout=20).read()
    return json.loads(r)

rendered = md_to_html(msg)
for c in chunks(rendered):
    try:
        j = send(c, parse=True)
        if not j.get("ok"): raise RuntimeError(j.get("description","?"))
    except Exception:
        # HTML parse failed → resend that chunk as clean plain text (tags stripped)
        try: send(strip_tags(c), parse=False)
        except Exception as e: sys.stderr.write(f"send fail: {e}\n")
PY
