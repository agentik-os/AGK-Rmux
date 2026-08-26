#!/usr/bin/env bash
# omega-plan-mirror.sh — PostToolUse hook on the plan tools.
#
# The tracked plan lives inside the session transcript, which nothing outside
# that session can see. So a mission that stalls at 3 of 7 tasks looks exactly
# like a mission that stalled at 7 of 7: the stuck-oracle alert could say
# "no activity for 25 min" and nothing more.
#
# This mirrors the plan into ~/.omega/state/plan-<session>.json every time a
# plan tool is called, so the patrol, the Telegram alert and the operator can
# see mission progress from the outside without attaching to the session.
#
# Never blocks, never prints to the model. Fail-open everywhere.

set -uo pipefail

[[ "${OMEGA_PLAN_MIRROR:-on}" == "off" ]] && exit 0

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
STATE_DIR="$OMEGA_DIR/state"
HOOKS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

IN=""
[ ! -t 0 ] && IN=$(cat 2>/dev/null)
[ -z "$IN" ] && exit 0

command -v python3 >/dev/null 2>&1 || exit 0

OMEGA_HOOK_INPUT="$IN" \
OMEGA_HOOKS_DIR="$HOOKS_DIR" \
OMEGA_STATE_DIR="$STATE_DIR" \
python3 - <<'PY' 2>/dev/null
import json, os, re, sys, time

sys.path.insert(0, os.environ.get("OMEGA_HOOKS_DIR", ""))
try:
    import omega_plan_state as P
except Exception:
    sys.exit(0)

try:
    payload = json.loads(os.environ.get("OMEGA_HOOK_INPUT") or "{}")
except Exception:
    sys.exit(0)
if not isinstance(payload, dict):
    sys.exit(0)

state = P.analyze(payload.get("transcript_path") or "")
if state is None or not state["plan_ever"]:
    sys.exit(0)

state_dir = os.environ.get("OMEGA_STATE_DIR") or "/tmp"
try:
    os.makedirs(state_dir, exist_ok=True)
except Exception:
    sys.exit(0)

sid = re.sub(r"[^A-Za-z0-9_.-]", "_", str(payload.get("session_id") or "unknown"))[:64]
cwd = payload.get("cwd") or os.getcwd()

out = {
    "session_id": sid,
    "cwd": cwd,
    "project": os.path.basename(cwd.rstrip("/")) or cwd,
    "updated_at": int(time.time()),
    "total": state["total_tasks"],
    "open": len(state["open_items"]),
    "done": max(0, state["total_tasks"] - len(state["open_items"])),
    "open_items": [str(s)[:120] for s in state["open_items"][:12]],
}

path = os.path.join(state_dir, "plan-%s.json" % sid)
tmp = path + ".tmp"
try:
    with open(tmp, "w") as fh:
        json.dump(out, fh)
    os.replace(tmp, path)
except Exception:
    try:
        os.unlink(tmp)
    except Exception:
        pass
PY
exit 0
