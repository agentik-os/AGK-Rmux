#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# OmegaOS · omega monitor · the watcher loop
# ───────────────────────────────────────────────────────────────────────────
#   omega-monitor.sh <target> <session> [interval] [lines] [work] [progress] [audit-every]
#     target      : an ssh host alias from ~/.ssh/config, OR the literal "local"
#     session     : the WATCHED rmux session name on that box
#     interval    : poll seconds (default 60)
#     lines       : scrollback lines captured per poll (default 200)
#     work        : ONE argument, a shell command printing ONE integer: how many
#                   units of work are available RIGHT NOW. Quote it.
#     progress    : ONE argument, a shell command printing ONE monotonic integer:
#                   how much work is DONE. Every advance resets the nudge budget.
#     audit-every : dispatch the deep audit team every N polls (0 = never)
#
# NAME COLLISION, READ THIS FIRST. Bare `omega monitor` is the BILLING view
# (crates/omega-core/src/monitor.rs, its own TUI tab) and it keeps working exactly
# as it does today. THIS script is the SESSION monitor and is reached only through
# `omega monitor <session>` / `omega monitor <host>:<session>`. Nothing here may be
# read as a replacement for that command.
#
# Installed at ~/.omega/bin/omega-monitor.sh. `omega monitor <target>` creates the
# watcher with
#   rmux new-session -d -s <viewer> "$HOME/.omega/bin/omega-monitor.sh <target> <session> ..."
# so THIS LOOP *IS* THE WATCHER SESSION'S COMMAND, and it inherits every invariant
# omega-stream.sh paid for. They are invariants, not preferences:
#
#  1. THE LOOP MUST NEVER EXIT. If it exits, the rmux session dies and the operator
#     sees NOTHING, which is strictly worse than an error on screen. No `set -e`.
#     Errors are RENDERED, never fatal. It retries forever.
#  2. PULL, never push. The watcher box reaches out; the watched box ships nothing.
#     A push-based shipper died once and the mirror froze while the source kept
#     growing, and a frozen watcher is indistinguishable from a quiet one.
#  3. EVERY CAPTURE, PROBE AND SEND CARRIES ITS OWN WALL CLOCK. ssh's ConnectTimeout
#     bounds the HANDSHAKE only: a box that takes the connection and then answers
#     nothing parks the loop inside a command substitution forever, so no poll, no
#     classification and no alert ever happen again. A bound that fires is a
#     RENDERED state (SOURCE NOT ANSWERING), never a dead loop.
#  4. `timeout` is GNU coreutils and does NOT exist on stock macOS. Resolve it once
#     at startup and fall back to a watchdog built out of nothing but bash, which
#     signals the capture's PROCESS GROUP: signalling only the direct child leaves
#     a grandchild holding the command substitution's stdout pipe open, and the loop
#     stays frozen even though the wall clock fired (measured: TERM to the pid alone
#     left it stuck past 30s, TERM to the group returned in 3s).
#  5. rmux IS NOT tmux and is NOT on the non-interactive PATH: always the absolute
#     $HOME/.local/bin/rmux. In a remote ssh command that path must reach the REMOTE
#     shell UNEXPANDED, or a locally expanded $HOME points a Linux box at a macOS
#     path, silently. `send-keys` gets its Enter as a SEPARATE call.
#
# ── FOUR STATES, NOT TWO ─────────────────────────────────────────────────────
# A stop has more than one shape and each shape needs a different answer:
#   QUESTION  the agent is asking and will not move  -> needs JUDGEMENT: report to a
#             human, ONCE, and NEVER auto-answer it.
#   STALLED   the turn ended with work still available -> MECHANICAL: nudge it.
#   BLOCKED   nothing is runnable -> NEVER nudge. A nudge here is not persistence,
#             it is manufactured thrash. Report once.
#   WORKING   busy -> say NOTHING. Silence is the correct output most of the time.
#
# The JUDGEMENT is not made here. The pane is piped to `omega monitor classify
# --work <n>`, a PURE Rust function that is unit-testable with zero ssh
# and zero rmux, which is how all four states get proven. The reference watcher put
# this in bash greps that could never be tested; that is the one thing this
# deliberately improves on. This script does the poll, the ssh, the sleep, the send
# and the bookkeeping, and NOTHING ELSE decides what a pane means.
#
# ── THE SEVEN DEFECTS, AND WHERE EACH FIX LIVES ──────────────────────────────
# Every one was a false positive in production, and false positives train the
# operator to ignore alerts, which is the only thing an alerting system has.
#   1 self-echo: text we type ECHOES in the pane and was read back as a signal.
#     ledger_record + scrub_self_echo below: recorded AT SEND TIME, BEFORE sending,
#     excluded BY CONTENT in SHORT SLICES because the pane hard-wraps and a
#     whole-line match fails. The ledger always carries a SENTINEL: `grep -f`
#     against an EMPTY file matches nothing, and a BLANK line in a pattern file
#     matches EVERYTHING, so either one silently blanks the whole stream.
#   2 two signals shared one state variable and ping-ponged forever. ONE LATCH PER
#     SIGNAL here: LATCH_QUESTION, LATCH_BLOCKED, LATCH_EXHAUSTED, and TWO separate
#     failure latches, LATCH_CAPTURE_FAIL and LATCH_CLASSIFY_FAIL. No branch writes
#     another signal's latch. The split is not theoretical: while this file was being
#     tested the two failures shared one variable, a classifier failure overwrote the
#     capture failure's signature, and the very same dead session was reported twice.
#   3 clearing the failure latch on resume made frozen scrollback re-fire. NEITHER
#     failure latch is ever cleared: each holds a SIGNATURE, and a genuinely new
#     failure differs from it and gets through on its own. A recovery is reported as
#     its own event instead, so a flapping source is still visible.
#   4 it matched WORDS where STRUCTURES were meant. No pattern matching happens in
#     this file at all; the classifier owns it (see the Rust side).
#   5 dropping the tail of the capture did not cut the input box. Same answer: the
#     capture is handed over whole and cut STRUCTURALLY in Rust.
#   6 completed task lines re-render constantly and drowned the in-progress
#     transitions. Same answer: Rust.
#   7 counting only RUNNABLE steps reported BLOCKED while steps awaited a sign-off.
#     Awaiting judgement IS work, so an unreadable or absent work probe means ASSUME
#     WORK (never stall silently); see read_work below.
#
# ── THE NUDGE BOUND ──────────────────────────────────────────────────────────
# It bounds the ABSENCE OF PROGRESS, not the work. A flat cap of N restarts stops a
# healthy long run for no reason (a cap of 25 ran out around step 40 of 157). The
# budget resets every time the progress probe's integer ADVANCES, and only N nudges
# that produced NOTHING exhaust it. That is what R-LOOP actually asks for: bounded
# retries on the SAME failure, then a human.
# ═══════════════════════════════════════════════════════════════════════════

# Deliberately NO `set -e` / `set -u` / `pipefail`: invariant 1. A stray non-zero
# exit or unset var must never take the watcher session down with it.

TARGET="$1"
SESSION="$2"
INTERVAL="${3:-60}"
SCROLLBACK="${4:-200}"          # NOT named LINES: bash owns LINES (terminal height).
WORK_PROBE="${5:-}"
PROGRESS_PROBE="${6:-}"
AUDIT_EVERY="${7:-0}"

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
RMUX_LOCAL="$HOME/.local/bin/rmux"
# shellcheck disable=SC2016  # NOT expanding here is the whole point: invariant 5.
RMUX_REMOTE='$HOME/.local/bin/rmux'   # single-quoted: expands on the REMOTE box.
OMEGA_BIN="${OMEGA_BIN:-$HOME/.local/bin/omega}"
[ -x "$OMEGA_BIN" ] || OMEGA_BIN="$(command -v omega 2>/dev/null || echo "$OMEGA_BIN")"
AUDIT_SCRIPT="${OMEGA_MONITOR_AUDIT:-$OMEGA_DIR/bin/omega-monitor-audit.sh}"

# How many nudges may produce nothing before this needs a human (R-LOOP THRASH_CAP).
NUDGE_BUDGET="${OMEGA_MONITOR_NUDGE_BUDGET:-3}"
# Polls of total silence before ONE liveness line. WORKING says nothing per poll,
# but a watcher that has printed nothing for an hour is indistinguishable from a
# watcher that died, which is the same failure invariant 2 exists to prevent.
HEARTBEAT_EVERY="${OMEGA_MONITOR_HEARTBEAT:-30}"
# Per-agent wall clock handed to the deep audit team.
AUDIT_BUDGET="${OMEGA_MONITOR_AUDIT_BUDGET:-420}"
# Ledger slice length. Short on purpose (defect 1): the pane hard-wraps at the
# terminal width, so a whole-line pattern matches neither half of a wrapped nudge.
LEDGER_SLICE=12
LEDGER_MAX=400                  # slices kept; the ledger must not grow forever.

usage() {
    cat <<'EOF'
omega-monitor.sh: watch a live rmux session, classify what it is doing, and act.

  usage: omega-monitor.sh <target> <session> [interval] [lines] [work] [progress] [audit-every]

    target       ssh host alias from ~/.ssh/config, or the literal "local"
    session      the watched rmux session name on that box
    interval     poll seconds (default 60)
    lines        scrollback lines captured per poll (default 200)
    work         quoted shell command printing ONE integer: work available NOW
                 (> 0 = STALLED and nudgeable, 0 = BLOCKED and never nudged,
                  unreadable or absent = ASSUME WORK, never stall silently)
    progress     quoted shell command printing ONE monotonic integer: work DONE
                 (every advance resets the nudge budget)
    audit-every  dispatch the deep audit team every N polls (0 = never)

  examples:
    omega-monitor.sh local BluePrint-OS
    omega-monitor.sh matrix MAC-STREAM 60 200 'run.py steps --runnable-count' \
        'run.py steps --done-count' 30
EOF
}

# ── arg validation ────────────────────────────────────────────────────────────
# Bad args are a STARTUP fault, not a runtime one, so this is the ONE place the
# script may stop: invariant 1 governs failures hit while WATCHING, not an argv we
# cannot use at all. It still must not vanish instantly, since if we are the command
# of a watcher session, exiting takes the session with us and the operator never
# gets to read why, so the usage is held on screen for a BOUNDED grace first.
USAGE_GRACE=10
die_usage() {
    printf '%s\n\n' "omega-monitor: $1" >&2
    usage >&2
    if [ -t 1 ]; then
        printf '\n[warn]  omega monitor · BAD ARGUMENTS · exiting in %ss (ctrl-c to quit now)\n' "$USAGE_GRACE" >&2
        sleep "$USAGE_GRACE"
    fi
    exit 2
}

[ -z "$TARGET" ]  && die_usage "missing <target>"
[ -z "$SESSION" ] && die_usage "missing <session>"
# A single quote in either coordinate would break out of the remote quoting in
# capture/send. Refused, never quoted around: rmux names and ssh aliases are slugs.
case "$SESSION" in *\'*) die_usage "session name may not contain a single quote";; esac
case "$TARGET"  in *\'*|-*) die_usage "invalid target '$TARGET'";; esac
case "$INTERVAL"   in ''|*[!0-9]*) die_usage "interval must be a positive integer (got '$INTERVAL')";; esac
case "$SCROLLBACK" in ''|*[!0-9]*) die_usage "lines must be a positive integer (got '$SCROLLBACK')";; esac
case "$AUDIT_EVERY" in ''|*[!0-9]*) die_usage "audit-every must be an integer (got '$AUDIT_EVERY')";; esac
case "$NUDGE_BUDGET" in ''|*[!0-9]*) NUDGE_BUDGET=3;; esac
case "$HEARTBEAT_EVERY" in ''|*[!0-9]*) HEARTBEAT_EVERY=30;; esac
case "$AUDIT_BUDGET" in ''|*[!0-9]*) AUDIT_BUDGET=420;; esac
[ "$INTERVAL" -lt 5 ]   && INTERVAL=5
[ "$SCROLLBACK" -lt 1 ] && SCROLLBACK=1

# How the operator wrote the coordinate, used for display only. The classifier is
# reached as `omega monitor classify` and needs no coordinate at all: it judges the
# pane it is handed on stdin and nothing else, which is what makes it a pure function.
LABEL="$TARGET:$SESSION"

# ── state on disk ─────────────────────────────────────────────────────────────
# Per-target, because two monitors must never share a ledger: one target's nudges
# would then scrub the other target's pane. The slug maps anything outside the slug
# alphabet to '-' so the path is always creatable.
SLUG="$(printf '%s' "$LABEL" | tr -c 'A-Za-z0-9._-' '-')"
STATE_DIR="$OMEGA_DIR/state/monitor/$SLUG"
mkdir -p "$STATE_DIR" 2>/dev/null
LEDGER="$STATE_DIR/sent-ledger.txt"
EVENT_LOG="$STATE_DIR/events.log"
AUDIT_LOG="$STATE_DIR/audit.log"
AUDIT_LOCK="$STATE_DIR/audit.lock"

# A pattern that can never appear in a captured pane. Its only job is to guarantee
# the ledger is NEVER an empty file and NEVER only blank lines (defect 1).
SENTINEL='OMEGA-MONITOR-LEDGER-SENTINEL-NEVER-IN-A-PANE'

# run_bounded hands the caller's stdin to its child with an explicit `<&0`, so fd 0
# must be OPEN. A session command started with stdin closed (nohup, a cron shim) would
# otherwise fail every bounded call and the loop would report SESSION NOT FOUND for a
# session that is perfectly healthy: a fabricated diagnosis, which is worse than none.
( : <&0 ) 2>/dev/null || exec 0</dev/null

RAW="$(mktemp -t omega-monitor-raw.XXXXXX 2>/dev/null || echo "/tmp/omega-monitor.$$.raw")"
PANE="$(mktemp -t omega-monitor-pane.XXXXXX 2>/dev/null || echo "/tmp/omega-monitor.$$.pane")"
ERRFILE="$(mktemp -t omega-monitor-err.XXXXXX 2>/dev/null || echo "/tmp/omega-monitor.$$.err")"
trap 'rm -f "$RAW" "$PANE" "$ERRFILE"' EXIT

# ── state in memory ───────────────────────────────────────────────────────────
# ONE VARIABLE PER SIGNAL (defect 2). No branch below writes another signal's latch.
LATCH_QUESTION=0        # a question was reported; cleared only when the state leaves QUESTION
LATCH_BLOCKED=0         # blocked was reported;    cleared only when the state leaves BLOCKED
LATCH_EXHAUSTED=0       # "this needs a human" was said; cleared only by real progress
# TWO failure latches, not one, and this split was reproduced live before it was
# written: with a single shared variable the classifier failure OVERWROTE the capture
# failure's signature, so the next identical capture failure no longer matched the
# latch and fired again. That is defect 2 verbatim (failure branch writes it, another
# branch overwrites it, the unchanged failure re-fires forever). Measured: one dead
# session reported SESSION NOT FOUND twice in 30s with nothing having changed.
# Both hold a SIGNATURE and NEITHER is ever cleared on resume (defect 3): clearing on
# resume made frozen scrollback re-fire, and a genuinely new failure differs from the
# signature and gets through on its own.
LATCH_CAPTURE_FAIL=""
LATCH_CLASSIFY_FAIL=""
NUDGES_SPENT=0
LAST_PROGRESS=""        # empty until the progress probe first reads an integer
POLL=0
SAID=0                  # did this poll print anything? drives the liveness line only
SILENT_POLLS=0
FAILED_POLLS=0
LAST_SEEN="unknown"     # last state we successfully read. INFORMATIONAL ONLY: it names
                        # the state in the liveness line and never decides a report,
                        # which is what keeps defect 2 (one variable, two signals) out.

# ── the wall clock (invariants 3 and 4) ───────────────────────────────────────
# Deliberately the SAME watchdog as omega-stream.sh, kept verbatim rather than
# sourced: a session command must be able to bound its own captures without
# depending on a second file being installed beside it.
CAPTURE_BUDGET=$(( INTERVAL + 10 ))

TIMEOUT_CMD=()
if command -v timeout >/dev/null 2>&1; then
    TIMEOUT_CMD=(timeout -k 5 "$CAPTURE_BUDGET")
elif command -v gtimeout >/dev/null 2>&1; then
    TIMEOUT_CMD=(gtimeout -k 5 "$CAPTURE_BUDGET")
fi

# Exit codes that mean "the wall clock fired, the source never answered":
#   124  GNU timeout, budget elapsed        (137 if the child ignored the TERM)
#   143  the bash fallback below (128 + SIGTERM)
timed_out() {
    case "$1" in 124|137|143) return 0;; *) return 1;; esac
}

# Run "$@" under the budget, whichever mechanism we have. Stdin is passed straight
# through, so a caller may redirect a file into it (the classifier reads the pane
# on stdin).
#
#   * `set -m` puts the child in its OWN process group and the watchdog signals that
#     GROUP (`kill -- -PID`), exactly as GNU timeout does. Signalling only the direct
#     child is not enough: a grandchild still holds the command substitution's stdout
#     pipe open, so `$(...)` keeps blocking and the loop stays frozen even though the
#     wall clock fired. Job control is switched straight back off.
#   * `<&0` IS LOAD-BEARING, not decoration. Bash redirects an asynchronous command's
#     stdin to /dev/null unless job control is active, and job control is NEVER active
#     inside a command substitution no matter what `set -m` says, which is exactly
#     where classify() calls this. Measured: without it, `$(run_bounded cat < file)`
#     returned EMPTY while the identical call outside `$( )` returned the file, so on
#     any box using this fallback the classifier would have been handed an empty pane
#     and confidently classified nothing at all.
#   * the watchdog subshell redirects its own stdout, or IT would hold that same pipe
#     open and every call would cost the full budget.
#   * whatever the child printed BEFORE it wedged still reaches us, so a partial frame
#     is never mistaken for a clean one: the loop reads the STATUS, never the output.
run_bounded() {
    if [ ${#TIMEOUT_CMD[@]} -gt 0 ]; then
        "${TIMEOUT_CMD[@]}" "$@"
        return $?
    fi
    set -m
    "$@" <&0 &
    local pid=$!
    set +m
    (
        sleep "$CAPTURE_BUDGET"
        kill -TERM -- "-$pid" 2>/dev/null || kill -TERM "$pid" 2>/dev/null
    ) >/dev/null 2>&1 &
    local watchdog=$!
    wait "$pid"
    local rc=$?
    kill -TERM "$watchdog" >/dev/null 2>&1
    wait "$watchdog" >/dev/null 2>&1
    return $rc
}

CLOCK_NOTE=""
[ ${#TIMEOUT_CMD[@]} -eq 0 ] && CLOCK_NOTE=" · bash watchdog"

# ── the event surface ─────────────────────────────────────────────────────────
# Append-only and timestamped, not a redrawn frame: a monitor's output is a history
# of what it DID, and an operator attaching hours later must be able to read back
# every action instead of one screen of now. Mirrored to $EVENT_LOG so the record
# survives the pane's scrollback.
say() {
    local kind="$1" msg="$2" line
    line="$(date '+%H:%M:%S')  [$kind] $msg"
    printf '%s\n' "$line"
    printf '%s  %s\n' "$(date '+%Y-%m-%d')" "$line" >> "$EVENT_LOG" 2>/dev/null
    SAID=1
}

# Closes a poll: the liveness accounting, then the sleep. Every path through the loop
# ends here, INCLUDING the failure paths, because a watcher that has printed nothing
# for an hour is indistinguishable from a watcher that died, and a latched failure or
# a latched question is exactly when that silence lasts longest.
end_poll() {
    if [ "$SAID" -eq 1 ]; then
        SILENT_POLLS=0
    else
        SILENT_POLLS=$(( SILENT_POLLS + 1 ))
    fi
    if [ "$SILENT_POLLS" -ge "$HEARTBEAT_EVERY" ]; then
        say "alive" "$LABEL · last read: $LAST_SEEN · poll $POLL · nothing new for $SILENT_POLLS polls"
        SILENT_POLLS=0
    fi
    sleep "$INTERVAL"
}

# ── the sent-ledger (defect 1) ────────────────────────────────────────────────
# Everything this watcher types into the watched session is recorded here BEFORE it
# is sent, in SHORT SLICES, and every recorded slice is stripped from the pane before
# the pane is classified. Recording AFTER the send loses the race: the echo can be on
# screen by the next poll.
ledger_seed() {
    printf '%s\n' "$SENTINEL" > "$LEDGER" 2>/dev/null
}

ledger_guard() {
    # An EMPTY ledger is the silent killer: `grep -f` against an empty pattern file
    # matches nothing, and a BLANK line inside one matches every line. Either way the
    # stream is blanked and the monitor goes deaf without saying so.
    [ -s "$LEDGER" ] || ledger_seed
}

ledger_record() {
    # Two `local` statements, not one: in `local a="$1" n=${#a}` the second expansion
    # does not see the first, so n would be 0, the loop below would record NOTHING and
    # the self-echo fix would be silently disabled (shellcheck SC2318).
    local text="$1"
    local i=0 slice
    local n=${#text}
    ledger_guard
    while [ "$i" -lt "$n" ]; do
        slice="${text:$i:$LEDGER_SLICE}"
        i=$(( i + LEDGER_SLICE ))
        # Never record a blank or near-blank slice: as a fixed-string pattern a blank
        # line matches EVERY line of the pane, and a 3-character fragment matches half
        # the screen. Both delete real signal.
        [ "${#slice}" -lt 6 ] && continue
        case "$slice" in *[![:space:]]*) : ;; *) continue ;; esac
        printf '%s\n' "$slice" >> "$LEDGER" 2>/dev/null
    done
    # Prune, dropping any blank line that ever reached the file.
    if [ -s "$LEDGER" ]; then
        grep -v '^[[:space:]]*$' "$LEDGER" 2>/dev/null | tail -n "$LEDGER_MAX" > "$LEDGER.tmp" 2>/dev/null
        if [ -s "$LEDGER.tmp" ]; then mv -f "$LEDGER.tmp" "$LEDGER" 2>/dev/null; else rm -f "$LEDGER.tmp" 2>/dev/null; fi
    fi
    ledger_guard
}

scrub_self_echo() {   # $1 = raw capture, $2 = pane handed to the classifier
    ledger_guard
    grep -v -F -f "$LEDGER" -- "$1" > "$2" 2>/dev/null
    if [ ! -s "$2" ] && [ -s "$1" ]; then
        # The ledger ate the entire capture. That is the blanking failure, and a
        # blanked pane classifies as something the session is not, so use the raw
        # pane for this poll and SAY so rather than fail silently.
        cp -f "$1" "$2" 2>/dev/null
        say "warn" "the sent-ledger removed every line of the capture, classifying the RAW pane this poll (ledger: $LEDGER)"
    fi
}

# ── the capture ───────────────────────────────────────────────────────────────
# stdout -> the pane, stderr -> $ERRFILE (never mixed into the pane), rc -> state.
# Always under the wall clock: a capture that cannot finish must come back as a
# STATE, not as a loop that stopped iterating.
capture() {
    if [ "$TARGET" = "local" ]; then
        run_bounded "$RMUX_LOCAL" capture-pane -p -t "$SESSION" -S -"$SCROLLBACK" 2>"$ERRFILE"
    else
        # $HOME stays literal so it expands on the REMOTE box (invariant 5).
        # -n : stdin from /dev/null, so ssh never swallows the watcher's keystrokes.
        # ServerAlive* : ssh's own liveness probe on an ESTABLISHED connection, so a
        #   half-open TCP is usually ssh's own error (with a real message) before our
        #   wall clock has to guess.
        run_bounded ssh -n \
            -o ConnectTimeout=10 \
            -o ServerAliveInterval=5 \
            -o ServerAliveCountMax=2 \
            -o BatchMode=yes "$TARGET" \
            "$RMUX_REMOTE capture-pane -p -t '$SESSION' -S -$SCROLLBACK" 2>"$ERRFILE"
    fi
}

err_text() {
    tr '\n' ' ' < "$ERRFILE" 2>/dev/null | sed 's/  */ /g; s/ *$//'
}

# ── the probes ────────────────────────────────────────────────────────────────
# Operator-supplied shell commands, run under the same wall clock as everything
# else: a probe that hangs would freeze the watcher exactly like a wedged capture.
# They run on the WATCHER box, because that is where the operator wrote them and
# where the build's state files usually are; a probe that must run elsewhere is
# written as its own ssh command by the operator.
probe_int() {   # $1 = command string; echoes the integer, or nothing
    local out
    [ -z "$1" ] && return 1
    out="$(run_bounded bash -c "$1" 2>/dev/null | tr -d ' \t\r' | head -1)"
    case "$out" in
        ''|*[!0-9]*) return 1 ;;
        *) printf '%s' "$out"; return 0 ;;
    esac
}

# DEFECT 7: the first watcher counted only steps marked RUNNABLE and reported BLOCKED
# while steps sat awaiting a sign-off. Awaiting judgement IS work. So an unreadable,
# non-numeric or absent probe means ASSUME WORK, never stall silently. The assumption
# is normalized HERE, at the input, so the classifier stays a pure function of
# (pane, work) and never has to guess what a missing number meant.
#
# Sets the globals WORK and WORK_NOTE rather than echoing: a command substitution runs
# the function in a SUBSHELL, so anything it sets is lost the moment it returns and the
# caller would silently always see the initial value.
#
# WORK is left EMPTY when nothing could be measured. It is never filled with an
# invented 1: a fabricated measurement crossing the seam is the same class of lie as
# a false positive, and the classifier already reads a missing count as work.
WORK=""
WORK_NOTE=""
read_work() {
    local n
    if n="$(probe_int "$WORK_PROBE")"; then
        WORK="$n"
        WORK_NOTE="work=$n"
    else
        WORK=""
        WORK_NOTE="work not measured, the probe was unreadable or absent, so work is assumed"
    fi
}

# ── the classifier seam ───────────────────────────────────────────────────────
# The pane goes IN on stdin, exactly one of QUESTION / STALLED / BLOCKED / WORKING
# comes OUT. No classification logic lives in this file: put it in bash and it can
# never be unit-tested, which is how the seven defects survived for so long.
#
# $OMEGA_MONITOR_CLASSIFY overrides the command (it receives the pane on stdin and
# the work count as $1). It exists so the acceptance harness can prove THIS loop's
# plumbing (ledger scrub, latches, nudge bound) against real captured panes without
# a built binary, and so an operator can drop in a debug classifier. It is not a
# second implementation of the classifier.
classify() {   # $1 = the MEASURED work count, EMPTY when nothing could be measured
    local out rc
    if [ -n "$OMEGA_MONITOR_CLASSIFY" ]; then
        out="$(run_bounded bash -c "$OMEGA_MONITOR_CLASSIFY" omega-monitor-classify "$1" < "$PANE" 2>"$ERRFILE")"
        rc=$?
    elif [ -n "$1" ]; then
        # `classify` is the TARGET word, not a flag: `omega monitor` with no
        # target is the pre-existing billing view, so the session monitor is
        # selected by a target and `classify` is one of them (beside `list`).
        # --work hands over the count THIS loop already measured, so the probe
        # is not re-run inside the binary: it is usually an ssh round trip, and
        # two runs can disagree across the gap.
        out="$(run_bounded "$OMEGA_BIN" monitor classify --work "$1" < "$PANE" 2>"$ERRFILE")"
        rc=$?
    else
        # No --work at all when the probe could not be read. The classifier reads a
        # missing count as WORK AVAILABLE, which is defect 7's rule ("awaiting a
        # sign-off is work, never stall silently") implemented ONCE, in the pure
        # function that is unit-tested, instead of twice with a number this loop
        # made up.
        out="$(run_bounded "$OMEGA_BIN" monitor classify < "$PANE" 2>"$ERRFILE")"
        rc=$?
    fi
    printf '%s' "$(printf '%s' "$out" | tr -d '\r' | grep -m1 -E '^[A-Za-z]+$' 2>/dev/null | tr '[:lower:]' '[:upper:]')"
    return $rc
}

# ── the nudge ─────────────────────────────────────────────────────────────────
# Two calls, always: rmux send-keys does not submit anything on its own, so the
# Enter is a SEPARATE call. `-l` sends the text LITERALLY, or a word that happens to
# be a key name (Enter, Escape, Space) would be typed as that key instead.
send_nudge() {
    local text="$1" rc_text rc_enter
    # RECORD BEFORE SENDING (defect 1). Recording after the send loses the race, and
    # recording a nudge we failed to send costs nothing but a stricter scrub.
    ledger_record "$text"
    if [ "$TARGET" = "local" ]; then
        run_bounded "$RMUX_LOCAL" send-keys -t "$SESSION" -l "$text" >/dev/null 2>"$ERRFILE"; rc_text=$?
        run_bounded "$RMUX_LOCAL" send-keys -t "$SESSION" Enter      >/dev/null 2>>"$ERRFILE"; rc_enter=$?
    else
        run_bounded ssh -n -o ConnectTimeout=10 -o BatchMode=yes "$TARGET" \
            "$RMUX_REMOTE send-keys -t '$SESSION' -l '$text'" >/dev/null 2>"$ERRFILE"; rc_text=$?
        run_bounded ssh -n -o ConnectTimeout=10 -o BatchMode=yes "$TARGET" \
            "$RMUX_REMOTE send-keys -t '$SESSION' Enter" >/dev/null 2>>"$ERRFILE"; rc_enter=$?
    fi
    [ "$rc_text" -eq 0 ] && [ "$rc_enter" -eq 0 ]
}

# ── the deep audit team ───────────────────────────────────────────────────────
# Dispatched in the BACKGROUND so the cheap loop never blocks on the expensive one,
# and gated by a lock DIRECTORY rather than a pid: the lock also survives a watcher
# restart, which a pid does not, and two audit teams on one project is pure waste.
# A crashed auditor must not disable the layer forever, so a lock older than the
# team's own wall clock plus slack is broken and said out loud.
audit_lock_age() {
    local started now
    started="$(cat "$AUDIT_LOCK/started" 2>/dev/null)"
    case "$started" in ''|*[!0-9]*) printf '%s' 999999; return 0;; esac
    now="$(date +%s)"
    printf '%s' $(( now - started ))
}

dispatch_audit() {
    if [ ! -x "$AUDIT_SCRIPT" ]; then
        audit_failure "the audit runner is not installed at $AUDIT_SCRIPT (run install.sh or omega sync)"
        return 0
    fi
    if [ -d "$AUDIT_LOCK" ]; then
        local age; age="$(audit_lock_age)"
        if [ "$age" -gt $(( AUDIT_BUDGET + 120 )) ]; then
            rm -rf "$AUDIT_LOCK" 2>/dev/null
            say "warn" "the previous audit team held its lock for ${age}s (budget ${AUDIT_BUDGET}s), breaking it and dispatching a fresh one"
        else
            say "audit" "the previous audit team is still running (${age}s), skipping this cycle"
            return 0
        fi
    fi
    mkdir "$AUDIT_LOCK" 2>/dev/null || { say "warn" "could not take the audit lock at $AUDIT_LOCK, skipping this cycle"; return 0; }
    date +%s > "$AUDIT_LOCK/started" 2>/dev/null
    # The lock is released by the same subshell that ran the team, so a failing or
    # killed auditor still frees the layer.
    (
        "$AUDIT_SCRIPT" "$TARGET" "$SESSION" "$AUDIT_BUDGET" >> "$AUDIT_LOG" 2>&1
        rm -rf "$AUDIT_LOCK" 2>/dev/null
    ) &
    say "audit" "deep audit team dispatched in the background (per-agent budget ${AUDIT_BUDGET}s), log: $AUDIT_LOG"
}

# A failure that repeats every cycle must be said ONCE (defect 3 applied to the audit
# layer): same signature, same silence.
AUDIT_FAILURE_LATCH=""
audit_failure() {
    [ "$AUDIT_FAILURE_LATCH" = "$1" ] && return 0
    AUDIT_FAILURE_LATCH="$1"
    say "warn" "$1"
}

# ── startup banner ────────────────────────────────────────────────────────────
say "start" "omega monitor · $LABEL · every ${INTERVAL}s · ${SCROLLBACK} lines${CLOCK_NOTE}"
say "start" "classifier: ${OMEGA_MONITOR_CLASSIFY:-$OMEGA_BIN monitor classify --work <n>} · state: $STATE_DIR"
if [ -z "$WORK_PROBE" ]; then
    # Said at startup, not on every poll: without a work probe nothing can tell a
    # stop with work left from a stop with nothing runnable, so BLOCKED can never be
    # reached and every stop is treated as nudgeable. The operator must know that.
    say "start" "no work probe configured, so work is ASSUMED available and BLOCKED can never be distinguished from STALLED"
else
    say "start" "work probe: $WORK_PROBE"
fi
[ -n "$PROGRESS_PROBE" ] && say "start" "progress probe: $PROGRESS_PROBE (every advance resets the nudge budget of $NUDGE_BUDGET)"
[ "$AUDIT_EVERY" -gt 0 ] && say "start" "deep audit team every $AUDIT_EVERY polls"
ledger_guard

# ── the loop ──────────────────────────────────────────────────────────────────
while true; do
    POLL=$(( POLL + 1 ))
    SAID=0

    capture > "$RAW"; RC=$?
    ERR="$(err_text)"

    if [ "$RC" -ne 0 ]; then
        # Three causes, and the third is why the wall clock exists. Our own budget
        # firing means the box took the connection and then said nothing, which is
        # NOT "unreachable" (we reached it) and NOT "not found" (nobody answered
        # either way), so it gets its own name instead of a diagnosis we did not earn.
        if timed_out "$RC"; then
            STATE_TXT="SOURCE NOT ANSWERING"
            [ -z "$ERR" ] && ERR="no answer within ${CAPTURE_BUDGET}s (capture killed)"
        elif [ "$TARGET" != "local" ] && [ "$RC" -eq 255 ]; then
            STATE_TXT="HOST UNREACHABLE"
        else
            STATE_TXT="SESSION NOT FOUND"
        fi
        FAILED_POLLS=$(( FAILED_POLLS + 1 ))
        # The signature is never cleared (defect 3): the same failure repeating is not
        # news, a genuinely different one differs from it and gets through on its own.
        SIG="$STATE_TXT|$ERR"
        if [ "$LATCH_CAPTURE_FAIL" != "$SIG" ]; then
            LATCH_CAPTURE_FAIL="$SIG"
            say "warn" "$STATE_TXT · $LABEL · exit $RC · ${ERR:-(no output on stderr)} · retrying every ${INTERVAL}s, this loop never gives up"
        fi
        LAST_SEEN="$STATE_TXT"
        end_poll
        continue
    fi

    if [ "$FAILED_POLLS" -gt 0 ]; then
        # A recovery is its own event and does NOT clear LATCH_CAPTURE_FAIL: clearing
        # a failure latch on resume is exactly what made frozen scrollback re-fire
        # forever (defect 3). This line is how a flapping source stays visible without
        # re-reporting a failure that has not changed.
        say "ok" "$LABEL is answering again after $FAILED_POLLS failed poll(s)"
        FAILED_POLLS=0
    fi

    scrub_self_echo "$RAW" "$PANE"

    read_work

    # The progress probe is read BEFORE acting, so a nudge decided this poll spends
    # the freshly reset budget rather than the stale one.
    if [ -n "$PROGRESS_PROBE" ]; then
        if PROGRESS="$(probe_int "$PROGRESS_PROBE")"; then
            if [ -n "$LAST_PROGRESS" ] && [ "$PROGRESS" -gt "$LAST_PROGRESS" ]; then
                # THE NUDGE BOUND bounds the ABSENCE of progress, not the work: a
                # healthy long run must never run out of budget for being long.
                if [ "$NUDGES_SPENT" -gt 0 ] || [ "$LATCH_EXHAUSTED" -eq 1 ]; then
                    say "budget" "progress advanced $LAST_PROGRESS to $PROGRESS, nudge budget reset to $NUDGE_BUDGET"
                fi
                NUDGES_SPENT=0
                LATCH_EXHAUSTED=0
            fi
            LAST_PROGRESS="$PROGRESS"
        fi
    fi

    STATE="$(classify "$WORK")"; CRC=$?
    CERR="$(err_text)"

    case "$STATE" in
        QUESTION|STALLED|BLOCKED|WORKING) : ;;
        *)
            # An unreadable classifier takes NO action. Reporting a state we did not
            # get would be a fabricated alert, and nudging on garbage is manufactured
            # thrash: the two failures this whole design exists to avoid.
            SIG="$CRC|${STATE:-<empty>}|$CERR"
            if [ "$LATCH_CLASSIFY_FAIL" != "$SIG" ]; then
                LATCH_CLASSIFY_FAIL="$SIG"
                say "warn" "the classifier did not answer with a state (exit $CRC, printed '${STATE:-<empty>}'): ${CERR:-no error output} · taking NO action this poll"
            fi
            LAST_SEEN="classifier unreadable"
            end_poll
            continue
            ;;
    esac
    LAST_SEEN="$STATE"

    # Each latch is cleared by ITS OWN signal leaving, and by nothing else (defect 2:
    # two signals sharing one variable ping-ponged and re-fired forever).
    [ "$STATE" != "QUESTION" ] && LATCH_QUESTION=0
    [ "$STATE" != "BLOCKED" ]  && LATCH_BLOCKED=0

    case "$STATE" in
        QUESTION)
            # NEVER auto-answer. A question is the one state that needs judgement,
            # and a machine that answers it has removed the human from the only
            # decision that was theirs.
            if [ "$LATCH_QUESTION" -eq 0 ]; then
                LATCH_QUESTION=1
                say "question" "$LABEL is asking and will not move until a human answers it · attach with: omega attach $SESSION"
            fi
            ;;
        STALLED)
            if [ "$NUDGES_SPENT" -ge "$NUDGE_BUDGET" ]; then
                if [ "$LATCH_EXHAUSTED" -eq 0 ]; then
                    LATCH_EXHAUSTED=1
                    say "stalled" "$NUDGE_BUDGET nudges produced no progress on $LABEL · this needs a human, no further nudge will be sent until progress advances"
                fi
            else
                # The work count is quoted in the nudge, and annotated when it was
                # ASSUMED rather than measured, so a human reading the pane can tell
                # a real count from the defect-7 fallback.
                NUDGE_TEXT="[omega monitor] the turn ended with work still available ($WORK_NOTE). Continue with the next unfinished item and keep going."
                NUDGES_SPENT=$(( NUDGES_SPENT + 1 ))
                if send_nudge "$NUDGE_TEXT"; then
                    say "stalled" "nudge $NUDGES_SPENT/$NUDGE_BUDGET sent to $LABEL ($WORK_NOTE)"
                else
                    NERR="$(err_text)"
                    say "warn" "nudge $NUDGES_SPENT/$NUDGE_BUDGET could NOT be delivered to $LABEL: ${NERR:-no error output}"
                fi
            fi
            ;;
        BLOCKED)
            # NEVER nudge. Nothing is runnable, so a nudge cannot produce work: it
            # produces thrash, and thrash is what teaches an operator to ignore us.
            if [ "$LATCH_BLOCKED" -eq 0 ]; then
                LATCH_BLOCKED=1
                # The count is quoted from the probe, never asserted as 0: printing a
                # number we did not read is the same class of lie as a false positive.
                say "blocked" "$LABEL stopped with nothing runnable ($WORK_NOTE) · NOT nudging, this needs a human · attach with: omega attach $SESSION"
            fi
            ;;
        WORKING)
            # Say NOTHING. Silence is the correct output most of the time, and a
            # watcher that narrates a healthy build is one the operator mutes.
            : ;;
    esac

    if [ "$AUDIT_EVERY" -gt 0 ] && [ $(( POLL % AUDIT_EVERY )) -eq 0 ]; then
        dispatch_audit
    fi

    end_poll
done
