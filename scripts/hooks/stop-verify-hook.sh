#!/usr/bin/env bash
# stop-verify-hook.sh — THE FINISH GUARD (Stop hook).
#
# Enforces L6 (finish the mission, never stop mid-workflow), R-PLAN (a tracked
# plan or it gets dropped) and L1/L4 (run something before calling it done).
#
# WHY IT BLOCKS INSTEAD OF REMINDING: the original version only `echo`ed a
# reminder and exited 0. A Stop hook that exits 0 is INVISIBLE to the model — its
# stdout goes to the transcript, never into the conversation — so that reminder
# had never once reached an agent. Only exit code 2 (stderr → model, stop
# refused) changes behaviour. Filename kept so every already-installed
# ~/.claude/settings.json inherits the fix with no migration.
#
# Transcript parsing lives in omega_plan_state.py, shared with the SessionStart
# contract hook so the two never drift apart.
#
# CONTRACT
#   stdin  : {"session_id":…, "transcript_path":…, "stop_hook_active":bool}
#   exit 0 : stop allowed (also the fail-open path — a guard must never break a session)
#   exit 2 : stop REFUSED, stderr is handed to the model as its next instruction
#
# LOOP SAFETY (R-LOOP): never blocks when the harness reports stop_hook_active,
# and never more than MAX_BLOCKS times per session — past the ceiling it stops
# nagging and hands control back to the operator.

set -uo pipefail

MAX_BLOCKS="${OMEGA_FINISH_GUARD_MAX_BLOCKS:-3}"
STATE_DIR="${OMEGA_DIR:-$HOME/.omega}/state/finish-guard"
HOOKS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Escape hatch: a session that legitimately must end early (operator asked for a
# partial answer, a cron probe) exports OMEGA_FINISH_GUARD=off.
[[ "${OMEGA_FINISH_GUARD:-on}" == "off" ]] && exit 0

IN=""
[ ! -t 0 ] && IN=$(cat 2>/dev/null)

command -v python3 >/dev/null 2>&1 || exit 0   # fail-open, no interpreter

# The hook payload travels in an env var, NOT on stdin: the analyzer below is
# fed to python3 through a heredoc, which already occupies stdin.
read -r -d '' OMEGA_GUARD_SRC <<'PY'
import json, os, re, sys, hashlib

sys.path.insert(0, os.environ.get("OMEGA_HOOKS_DIR", ""))
try:
    import omega_plan_state as P
except Exception:
    sys.exit(0)                      # shared module missing → fail open

raw = os.environ.get("OMEGA_HOOK_INPUT", "")
try:
    payload = json.loads(raw) if raw.strip() else {}
except Exception:
    payload = {}
if not isinstance(payload, dict):
    payload = {}

# The harness sets this while it is already re-running the agent because of a
# previous block. Blocking again here is how a hook loops forever.
if payload.get("stop_hook_active"):
    sys.exit(0)

tp = payload.get("transcript_path") or ""
st = P.analyze(tp)
if st is None:
    sys.exit(0)                      # nothing observable → allow

open_items = st["open_items"]

# R-PREFLIGHT — a preflight awaiting approval is a LEGAL stop. Before anything
# with a real blast radius, an interactive session investigates, hands over
# Goal / Blocking questions / Assumptions / Plan, and STOPS without having
# mutated a file. Refusing that stop is a false positive, and a false positive
# is the only fatal defect a guard can have — it teaches everyone to ignore it.
# The exemption is self-limiting BY DESIGN: it is gated on mutations == 0, so
# the instant the session writes a file the guard behaves exactly as before.
# Presenting a plan INSTEAD of executing remains an illegal stop (L6) once work
# has begun. Checked ahead of both branches — a preflight that also opened plan
# tasks is still a preflight.
if st.get("preflight") and st.get("mutations", 0) == 0:
    sys.exit(0)

# Real work, but no tracked plan was ever opened. This is the hole the other two
# checks cannot see: a session that never calls a plan tool has no open tasks to
# inspect, so a multi-part mission can lose its tail tasks silently. Thresholds
# are deliberately conservative — a chat turn or a one-file lookup does 0-2 tool
# calls and must never be blocked. A false positive costs one round-trip in which
# the agent enumerates what it did, which is the exact step (L6.1) being skipped.
MIN_MUTATIONS = P.int_env("OMEGA_FINISH_GUARD_MIN_MUTATIONS", 3)
MIN_TOOL_CALLS = P.int_env("OMEGA_FINISH_GUARD_MIN_TOOLS", 15)
# Two ways this check must NOT fire, both of them false positives that cost the
# session its whole block budget for nothing:
#
#   enumeration        the session recorded the per-ask enumeration R-PLAN is
#                      actually asking for, just in its final message rather
#                      than through a plan tool. The demand is the enumeration,
#                      not the tool that holds it.
#   plan_tools_missing this harness exposes no plan tool at all (the registry
#                      said so, in the transcript). Demanding TaskCreate from a
#                      session that cannot call it is an unwinnable gate; it
#                      refuses every stop until the ceiling and teaches the
#                      operator to ignore the guard, which is the one defect an
#                      alerting system cannot afford (R-MONITOR).
#
# Neither can silently switch the guard off: one requires the agent to produce
# a real enumeration, the other requires hard evidence from the tool registry.
planless_work = (
    (not st["plan_ever"])
    and (not st.get("enumeration"))
    and (not st.get("plan_tools_missing"))
    and (st["mutations"] >= MIN_MUTATIONS or st["tool_calls"] >= MIN_TOOL_CALLS)
)

reason = None
if open_items:
    shown = "\n".join("  - " + str(s)[:110] for s in open_items[:8])
    more = "" if len(open_items) <= 8 else "\n  … and %d more" % (len(open_items) - 8)
    reason = (
        "STOP REFUSED — L6 (finish the mission). %d task(s) in your own tracked plan are "
        "still open:\n%s%s\n\n"
        "Do NOT summarize, do NOT ask whether to continue, do NOT re-report what is already "
        "done. Take the first open task, set it in_progress, finish it, verify it against "
        "runtime (L1), mark it completed, then move to the next one. If a task is genuinely "
        "blocked, finish every other unblocked task first (L4), then say plainly what is "
        "blocked and why. If 3+ open tasks are file-disjoint, fan them out now (R-ORCH) "
        "instead of grinding through them one at a time."
    ) % (len(open_items), shown, more)
else:
    faults = []
    if planless_work:
        faults.append(
            "· NO TRACKED PLAN (R-PLAN). This session ran %d tool call(s) and %d file "
            "mutation(s) without ever opening one. Nothing recorded what the prompt asked "
            "for, so nothing can prove the tail tasks were not dropped — which is exactly "
            "how they get dropped.\n"
            "  Do this now: re-read the ORIGINAL prompt and record ONE item per distinct "
            "ask, in the operator's own order — through a plan tool "
            "(TaskCreate/TodoWrite/update_plan) if this harness exposes one, otherwise as "
            "an explicit checklist or status table in your final message; both count. Mark "
            "the ones you genuinely finished AND verified, then execute every item that is "
            "left. If it turns out everything really is done, the enumeration costs you one "
            "message and proves it." % (st["tool_calls"], st["mutations"])
        )
    if st["edited"] and not st["verified"]:
        faults.append(
            "· NOTHING WAS RUN AFTER THE LAST EDIT (L1/L4). Files were changed and then not "
            "a single command was executed against them.\n"
            "  Run the real check now — the build, the test suite, the script you just wrote, "
            "an HTTP probe of the route you touched — read the actual output, fix what it "
            "reports. A green diff is not a green build."
        )
    if faults:
        reason = "STOP REFUSED — the mission is not finishable yet:\n\n" + "\n\n".join(faults)

if not reason:
    sys.exit(0)

# ── Bounded blocking (R-LOOP) ────────────────────────────────────────────────
state_dir = os.environ.get("OMEGA_STATE_DIR") or "/tmp/omega-finish-guard"
try:
    os.makedirs(state_dir, exist_ok=True)
except Exception:
    sys.exit(0)

sid = payload.get("session_id") or hashlib.sha1(tp.encode()).hexdigest()[:16]
sid = re.sub(r"[^A-Za-z0-9_.-]", "_", str(sid))[:64]
counter = os.path.join(state_dir, sid + ".count")

try:
    n = int(open(counter).read().strip() or "0")
except Exception:
    n = 0

max_blocks = P.int_env("OMEGA_MAX_BLOCKS", 3)
if n >= max_blocks:
    sys.exit(0)      # ceiling reached: stop nagging, hand control back (R-LOOP)

try:
    with open(counter, "w") as fh:
        fh.write(str(n + 1))
except Exception:
    pass

if n + 1 >= max_blocks:
    reason += (
        "\n\n(finish-guard: block %d/%d — the last one for this session. If you truly cannot "
        "finish, say so explicitly and tell the operator what is blocked; do not spin.)"
    ) % (n + 1, max_blocks)

sys.stderr.write(reason + "\n")
sys.exit(2)
PY

OMEGA_HOOK_INPUT="$IN" \
OMEGA_HOOKS_DIR="$HOOKS_DIR" \
OMEGA_STATE_DIR="$STATE_DIR" \
OMEGA_MAX_BLOCKS="$MAX_BLOCKS" \
    python3 -c "$OMEGA_GUARD_SRC"
rc=$?

[[ $rc -eq 2 ]] && exit 2
exit 0
