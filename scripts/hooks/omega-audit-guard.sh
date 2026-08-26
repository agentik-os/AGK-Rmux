#!/usr/bin/env bash
# omega-audit-guard.sh — PreToolUse hook: destructive-operation tripwire.
#
# Autonomous oracles/workers run with bypassPermissions (R-SEC pre-authorized).
# That speed is the point — but an irreversible, out-of-scope mistake (rm -rf
# $HOME, force-push main, disk wipe) has no undo. This guard is the one brake:
# it scans every Bash command BEFORE it runs, journals destructive ops PER PROJECT
# at ~/.omega/audit/<project>/guard.jsonl (next to audit.jsonl), and BLOCKS only
# the small set of truly catastrophic + irrecoverable ones (PreToolUse exit 2 →
# the call is refused and the reason is fed back to the agent). Everything else
# destructive is logged + allowed — visibility, not friction.
#
# Tiers:
#   CATASTROPHIC → block (exit 2) + Telegram alert to the operator.   (irreversible)
#   HIGH         → journal + allow (exit 0).                          (recoverable)
#   (no match)   → silent allow.
#
# NEVER fails a session by accident: set +e, all helpers swallow errors, the only
# non-zero exit is a deliberate catastrophic block. <5ms target.
# Hook stdin (Claude Code): { session_id, tool_name, tool_input }
set +e

INPUT=""
[ ! -t 0 ] && INPUT=$(cat 2>/dev/null | head -c 65536)
[ -z "$INPUT" ] && exit 0

TOOL=$(printf '%s' "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null)
[ "$TOOL" = "Bash" ] || exit 0   # only shell commands can be destructive here
CMD=$(printf '%s' "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)
[ -z "$CMD" ] && exit 0

# ── Classify ─────────────────────────────────────────────────────────────────
# Hot-path note: the benign case (the vast majority of Bash calls) does only sed
# + a few greps below, then exits. Project resolution (python3) runs ONLY once a
# destructive op is detected, so the guard adds ~no latency to normal commands.
# CATASTROPHIC: irreversible, no reflog/undo, out-of-scope. Patterns are tight so
# no legitimate dev command matches (a scoped `rm -rf ./build` is HIGH, not this).
SEV=""; REASON=""
# Match a NORMALIZED copy: strip git/tag *message* values (-m/--message/-F "…" or '…')
# so a commit that merely DESCRIBES a destructive op (this very guard's commit did)
# is never mistaken for one. The message text is data, never executed.
SCAN=$(printf '%s' "$CMD" | sed -E "s/(-m|--message|-F)[[:space:]]+\"[^\"]*\"/\1 X/g; s/(-m|--message|-F)[[:space:]]+'[^']*'/\1 X/g")
g() { printf '%s' "$SCAN" | grep -Eq "$1"; }   # match the normalized command

# A destructive verb only counts at a COMMAND boundary — start, or after ; & | ( ) { } ` or
# newline — optionally via sudo/doas. This ignores the same token inside an echo'd string or
# prose ( echo "rm -rf /" → the rm sits after a quote, not a boundary ) while still catching
# `sudo rm` and `cmd && rm`. rmf also swallows an optional opening quote so `rm -rf "$HOME"` hits.
BND='(^|[;&|(){}`])[[:space:]]*(sudo[[:space:]]+|doas[[:space:]]+)?'
rmf="${BND}"'rm[[:space:]]+(-[a-zA-Z]*[rf][a-zA-Z]*[[:space:]]+)+["'"'"'`]?'
# A target token ENDS (TERM) at whitespace, a quote, end-of-line, or a shell separator —
# crucially NOT another path char. So a HOME/root target is catastrophic only when it IS
# the root itself (rm -rf /, /*, ~, ~/, $HOME, /home, /home/<user>) — never `~/.local/bin/x`
# or `~/.omega/state/f` (a path UNDER home: ~ then / then more → no TERM → not flagged).
Q="'"; SEP="[[:space:]\"${Q}\`;|&)]"; TERM="(${SEP}|\$)"

if   g "${rmf}(/\*?${TERM}|(~|\\\$HOME|/home(/[^/[:space:]\"${Q}\`]+)?)/?${TERM})"; then
    SEV="catastrophic"; REASON="rm -rf targeting / or \$HOME — wipes the home/root tree, no undo"
elif g "${BND}"'git[[:space:]]+push[^|;&]*(--force([^-]|$)|[[:space:]]-f([[:space:]]|$))' \
  && g '(\bmain\b|\bmaster\b|HEAD:.*(main|master)|\+.*(main|master))'; then
    SEV="catastrophic"; REASON="force-push to main/master — rewrites shared history"
elif g "${BND}"'(mkfs|wipefs)|dd[[:space:]][^|;]*of=/dev/(sd|nvme|vd|disk)|>[[:space:]]*/dev/(sd|nvme|vd)'; then
    SEV="catastrophic"; REASON="disk-level write (mkfs/dd/wipefs to a block device) — destroys a filesystem"
elif g ':\(\)[[:space:]]*\{[[:space:]]*:\|:'; then
    SEV="catastrophic"; REASON="fork bomb"
# HIGH: destructive but recoverable (reflog / re-clone / scoped). Journal, allow.
elif g "${rmf}/" && ! g "${rmf}/tmp/"; then
    SEV="high"; REASON="rm -rf on an absolute path"   # /tmp = sanctioned scratch (R-ENV), skipped
elif g "${BND}"'git[[:space:]]+push[^|;&]*(--force([^-]|$)|[[:space:]]-f([[:space:]]|$))'; then
    SEV="high"; REASON="force-push (non-main branch)"
elif g "${BND}"'git[[:space:]]+reset[[:space:]]+--hard|git[[:space:]]+clean[[:space:]]+-[a-z]*f[a-z]*d|git[[:space:]]+clean[[:space:]]+-[a-z]*d[a-z]*f'; then
    SEV="high"; REASON="git reset --hard / clean -fd (discards working changes — reflog only)"
elif g "${BND}"'chmod[[:space:]]+-R[[:space:]]+777|chown[[:space:]]+-R[^|;]+[[:space:]]/([[:space:]]|$)'; then
    SEV="high"; REASON="recursive chmod 777 / chown on a root path"
elif printf '%s' "$SCAN" | grep -Eiq 'drop[[:space:]]+(database|schema|table)\b|truncate[[:space:]]+table\b'; then
    SEV="high"; REASON="SQL DROP/TRUNCATE — irreversible data loss if on prod"
fi
[ -z "$SEV" ] && exit 0   # nothing destructive — silent allow (hot path ends here)

# ── Resolve the owning project (only now — a destructive op was found) ───────
# Prefer a projects.json path that prefixes $PWD, else the git toplevel basename,
# else the dispatched session's project, else "global".
PROJECTS_FILE="$HOME/.omega/projects.json"
PROJECT=""
if [ -f "$PROJECTS_FILE" ]; then
    PROJECT=$(PWD_NOW="$PWD" python3 - "$PROJECTS_FILE" <<'PY' 2>/dev/null
import json,os,sys
pwd=os.environ.get("PWD_NOW","")
try:
    best=("",-1)
    for p in json.load(open(sys.argv[1])).get("projects",[]):
        path=p.get("path","")
        if path and (pwd==path or pwd.startswith(path.rstrip("/")+"/")):
            if len(path)>best[1]: best=(p.get("name",""),len(path))
    if best[0]: print(best[0])
except Exception: pass
PY
)
fi
if [ -z "$PROJECT" ]; then
    TOP=$(git rev-parse --show-toplevel 2>/dev/null)
    [ -n "$TOP" ] && PROJECT=$(basename "$TOP")
fi
if [ -z "$PROJECT" ]; then
    S="${RMUX_SESSION:-$(rmux display-message -p '#S' 2>/dev/null)}"
    case "$S" in oracle-*) PROJECT=$(printf '%s' "$S" | sed -E 's/^oracle-//; s/-[0-9]+$//');; esac
fi
[ -z "$PROJECT" ] && PROJECT="global"

# ── Journal (per project) ────────────────────────────────────────────────────
AUDIT_DIR="$HOME/.omega/audit/$PROJECT"
mkdir -p "$AUDIT_DIR" 2>/dev/null
TS=$(date -u +%Y-%m-%dT%H:%M:%S.%3NZ)
SESSION="${RMUX_SESSION:-$(rmux display-message -p '#S' 2>/dev/null)}"; [ -z "$SESSION" ] && SESSION="local"
ACTION=$([ "$SEV" = "catastrophic" ] && echo blocked || echo allowed)
jq -nc --arg ts "$TS" --arg sev "$SEV" --arg act "$ACTION" --arg sess "$SESSION" \
       --arg reason "$REASON" --arg cmd "${CMD:0:500}" \
   '{ts:$ts,event:"guard",severity:$sev,action:$act,session:$sess,reason:$reason,command:$cmd}' \
   >> "$AUDIT_DIR/guard.jsonl" 2>/dev/null

# ── Catastrophic → alert (via the canonical Alerts funnel) + block ───────────
if [ "$SEV" = "catastrophic" ]; then
    # Route through the ONE alert funnel (omega-alert-send.sh): posts to the dedicated
    # Alerts topic, auto-recreates it if deleted, DM fallback — NEVER the Atlas topic.
    SENDER="$HOME/.omega/bin/omega-alert-send.sh"
    if [ -x "$SENDER" ]; then
        esc() { printf '%s' "$1" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g'; }
        MSG="‖ <b>Guard — opération bloquée</b>
projet <b>$(esc "$PROJECT")</b> · <i>$(esc "$SESSION")</i>
$(esc "$REASON")
<code>$(esc "${CMD:0:300}")</code>"
        bash "$SENDER" "$MSG" >/dev/null 2>&1 &
    fi
    # PreToolUse: exit 2 refuses the tool call; stderr is fed back to the agent.
    printf '%s\n' "BLOCKED by OmegaOS audit-guard: $REASON. This op is irreversible and out-of-scope — if truly intended, run it yourself outside the agent, or narrow it to a project-scoped path." 1>&2
    exit 2
fi

exit 0   # HIGH journaled, allowed
