#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# OmegaOS — omega stream · the snapshot loop
# ───────────────────────────────────────────────────────────────────────────
#   omega-stream.sh <target> <session> [interval] [lines]
#     target   : an ssh host alias from ~/.ssh/config, OR the literal "local"
#     session  : the source rmux session name on that box
#     interval : poll seconds  (default 3)
#     lines    : scrollback lines to capture (default 120)
#
# Installed at ~/.omega/bin/omega-stream.sh. `omega stream` creates the viewer with
#   rmux new-session -d -s <viewer> "$HOME/.omega/bin/omega-stream.sh <target> <session> <interval> <lines>"
# so THIS LOOP *IS* THE VIEWER SESSION'S COMMAND. Four things follow, each learned
# the hard way — treat them as invariants, not preferences:
#
#  1. SNAPSHOT THE RENDERED SCREEN, never replay raw bytes. `pipe-pane -O | tail -f`
#     renders as garbage: a full-screen TUI emits cursor moves and partial redraws
#     that only mean anything against a live screen buffer. capture-pane returns the
#     RENDERED text. Capture, clear, print, sleep. That is the entire mechanism.
#  2. PULL, never push. The viewer box reaches out; the source ships nothing. A
#     push-based shipper died once and the mirror FROZE while the source kept
#     growing — and a frozen mirror is indistinguishable from a quiet one. Pulling
#     puts the liveness of the stream on the box that can notice it stopped.
#  3. THE LOOP MUST NEVER EXIT. If it exits, the rmux session dies and the operator
#     sees NOTHING — strictly worse than an error on screen. No `set -e`. Errors are
#     RENDERED (banner + failure state), never fatal. It retries forever.
#  4. rmux IS NOT tmux and is NOT on the non-interactive PATH: always the absolute
#     $HOME/.local/bin/rmux. In the remote command that path MUST be written \$HOME
#     so it expands on the REMOTE box — a locally-expanded $HOME once pointed a Linux
#     box at /Users/hacker/... , silently.
#  5. EVERY CAPTURE CARRIES A WALL CLOCK. ssh's ConnectTimeout bounds the HANDSHAKE
#     only. A source that connects and then stops answering (hung rmux server, half
#     open TCP, a wedged box) parks the loop inside the command substitution forever:
#     no new frame, no new banner, and the staleness machinery below never runs
#     because it only runs after capture RETURNS. Measured: 12s of a wedged source
#     produced 0 bytes and 0 frames, leaving a [live] banner over a frozen frame —
#     verbatim the failure invariant 2 exists to prevent. So the capture is bounded,
#     and a bound that fires is a RENDERED state (SOURCE NOT ANSWERING), never fatal.
# ═══════════════════════════════════════════════════════════════════════════

# Deliberately NO `set -e` / `set -u` / `pipefail`: invariant 3. A stray non-zero
# exit or unset var must never take the viewer session down with it.

TARGET="$1"
SESSION="$2"
INTERVAL="${3:-3}"
SCROLLBACK="${4:-120}"          # NOT named LINES: bash owns LINES (terminal height).

RMUX_LOCAL="$HOME/.local/bin/rmux"
# shellcheck disable=SC2016  # NOT expanding here is the whole point: see invariant 4.
RMUX_REMOTE='$HOME/.local/bin/rmux'   # single-quoted: expands on the REMOTE box.

usage() {
    cat <<'EOF'
omega-stream.sh — mirror a live rmux session by snapshotting its rendered screen.

  usage: omega-stream.sh <target> <session> [interval] [lines]

    target    ssh host alias from ~/.ssh/config, or the literal "local"
    session   rmux session name on that box
    interval  poll seconds (default 3)
    lines     scrollback lines to capture (default 120)

  examples:
    omega-stream.sh local BluePrint-OS
    omega-stream.sh matrix MAC-STREAM 3 120
EOF
}

# ── arg validation ────────────────────────────────────────────────────────────
# Bad args are a STARTUP fault, not a runtime one, so this is the ONE place the
# script may stop: invariant 3 governs failures hit while STREAMING, not an argv we
# cannot use at all. It still must not vanish instantly — if we are the command of a
# viewer session, exiting takes the session with us and the operator never gets to
# read why — so the usage is held on screen for a BOUNDED grace first.
# Bounded, never forever: rmux exports RMUX / RMUX_PANE (NOT $TMUX, which reports
# "not in a multiplexer" from inside rmux), but neither can tell "I am the session's
# command" from "a human typed me at a prompt inside a session" — both have them set.
# So an unbounded hold would freeze the terminal of anyone who simply typo'd.
USAGE_GRACE=10
die_usage() {
    printf '%s\n\n' "omega-stream: $1" >&2
    usage >&2
    if [ -t 1 ]; then
        printf '\n[warn]  omega stream · BAD ARGUMENTS · exiting in %ss (ctrl-c to quit now)\n' "$USAGE_GRACE" >&2
        sleep "$USAGE_GRACE"
    fi
    exit 2
}

[ -z "$TARGET" ]  && die_usage "missing <target>"
[ -z "$SESSION" ] && die_usage "missing <session>"
case "$SESSION" in *\'*) die_usage "session name may not contain a single quote";; esac
case "$TARGET"  in *\'*|-*) die_usage "invalid target '$TARGET'";; esac
case "$INTERVAL"   in ''|*[!0-9]*) die_usage "interval must be a positive integer (got '$INTERVAL')";; esac
case "$SCROLLBACK" in ''|*[!0-9]*) die_usage "lines must be a positive integer (got '$SCROLLBACK')";; esac
[ "$INTERVAL" -lt 1 ]   && INTERVAL=1
[ "$SCROLLBACK" -lt 1 ] && SCROLLBACK=1

LABEL="$TARGET:$SESSION"
ERRFILE="$(mktemp -t omega-stream.XXXXXX 2>/dev/null || echo "/tmp/omega-stream.$$.err")"
trap 'rm -f "$ERRFILE"' EXIT

# ── state ─────────────────────────────────────────────────────────────────────
LAST_FRAME=""       # last frame we successfully captured
LAST_OK=0           # epoch seconds of that capture; 0 == never connected

# ── the wall clock (invariant 5) ──────────────────────────────────────────────
# One capture may take at most this long. Generous on purpose: a healthy capture
# over ssh costs well under a second, so anything past a whole interval plus ten
# seconds is a source that is not answering, not a slow one.
CAPTURE_BUDGET=$(( INTERVAL + 10 ))

# `timeout` is GNU coreutils. It does NOT exist on a stock macOS — where, if it
# exists at all, it is `gtimeout` from brew coreutils — and OmegaOS installs on
# macOS (this script already avoids `mapfile` for the same reason). So: resolve it
# ONCE here, and when neither spelling is present fall back to a watchdog built
# out of nothing but bash. Detecting it per-iteration would pay a PATH lookup
# every tick for an answer that cannot change.
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

# Run "$@" under the budget, whichever mechanism we have.
#
# The fallback exists so a Mac without coreutils gets the SAME honesty as this
# box, instead of silently keeping the frozen-mirror bug. Three details make it
# work inside `$(capture)`, each one measured rather than assumed:
#   * `set -m` puts the capture in its OWN process group, and the watchdog
#     signals that group (`kill -- -PID`), which is exactly what GNU timeout
#     does. Signalling only the direct child is not enough: a grandchild still
#     holds the command substitution's stdout pipe open, so `$(capture)` keeps
#     blocking and the mirror stays frozen even though the wall clock fired.
#     Measured: TERM to the pid alone left the loop stuck past 30s; TERM to the
#     group returned in 3s. Job control is switched straight back off, and the
#     shell prints no job notices in either state.
#   * the watchdog subshell redirects its own stdout, or IT would hold that same
#     pipe open and every capture would cost the full budget.
#   * whatever the child printed BEFORE it wedged still reaches us, so a partial
#     frame is never mistaken for a clean one — the non-zero status is what the
#     loop reads, never the output.
run_bounded() {
    if [ ${#TIMEOUT_CMD[@]} -gt 0 ]; then
        "${TIMEOUT_CMD[@]}" "$@"
        return $?
    fi
    set -m
    "$@" &
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

# Which mechanism is holding the clock, said out loud in the banner when it is
# the fallback: the operator should never have to guess whether a mirror on this
# box is bounded. Empty wherever a timeout binary exists (every Linux box, and
# any Mac with coreutils), so the common banner stays clean.
CLOCK_NOTE=""
[ ${#TIMEOUT_CMD[@]} -eq 0 ] && CLOCK_NOTE=" · bash watchdog"

# ── geometry ──────────────────────────────────────────────────────────────────
# Re-read every iteration: the viewer resizes the moment the operator attaches.
term_rows() {
    local r
    r="$(tput lines 2>/dev/null)"
    case "$r" in ''|*[!0-9]*) r=40;; esac
    [ "$r" -lt 8 ] && r=8
    printf '%s' "$r"
}
term_cols() {
    local c
    c="$(tput cols 2>/dev/null)"
    case "$c" in ''|*[!0-9]*) c=200;; esac
    [ "$c" -lt 20 ] && c=20
    printf '%s' "$c"
}

# Truncate to the viewer width so nothing wraps. A wrapped line costs a second row
# and the screen scrolls, which loses the TOP of the frame — worse than losing the
# right edge. Only when stdout is a tty: piped output (tests, logs) stays complete.
fit() {
    local s="$1"
    if [ "$IS_TTY" = 1 ] && [ "${#s}" -gt "$COLS" ]; then
        printf '%s' "${s:0:$COLS}"
    else
        printf '%s' "$s"
    fi
}

# ── the capture ───────────────────────────────────────────────────────────────
# stdout -> the frame, stderr -> $ERRFILE (never mixed into the frame), rc -> state.
# Always under the wall clock (invariant 5): a capture that cannot finish must
# come back as a STATE, not as a loop that stopped iterating.
capture() {
    if [ "$TARGET" = "local" ]; then
        run_bounded "$RMUX_LOCAL" capture-pane -p -t "$SESSION" -S -"$SCROLLBACK" 2>"$ERRFILE"
    else
        # \$HOME is written so it expands on the REMOTE box (invariant 4).
        # -n : stdin from /dev/null, so ssh never swallows the viewer's keystrokes.
        # ServerAlive* : ssh's own liveness probe on an ESTABLISHED connection —
        #   5s apart, gone after 2 unanswered — so a half-open TCP is usually ssh's
        #   own error (with a real message) before our wall clock has to guess.
        run_bounded ssh -n \
            -o ConnectTimeout=10 \
            -o ServerAliveInterval=5 \
            -o ServerAliveCountMax=2 \
            -o BatchMode=yes "$TARGET" \
            "$RMUX_REMOTE capture-pane -p -t '$SESSION' -S -$SCROLLBACK" 2>"$ERRFILE"
    fi
}

# ── the render ────────────────────────────────────────────────────────────────
# line 1        : status banner
# lines 2..N-1  : the last (rows-2) lines of the frame  (row N stays free: no scroll)
render() {
    local banner="$1" extra="$2" body="$3"
    local avail=$(( ROWS - 2 ))
    [ -n "$extra" ] && avail=$(( avail - 1 ))
    [ "$avail" -lt 1 ] && avail=1

    # Split $body into lines without mapfile: OmegaOS also installs on macOS, whose
    # /bin/bash is 3.2 and has no mapfile.
    local -a lines=()
    local ln
    if [ -n "$body" ]; then
        while IFS= read -r ln || [ -n "$ln" ]; do lines+=("$ln"); done <<< "$body"
    fi

    local n=${#lines[@]}
    local start=$(( n - avail ))
    [ "$start" -lt 0 ] && start=0

    [ "$IS_TTY" = 1 ] && printf '\033[H\033[2J\033[3J'   # home + clear + drop scrollback

    fit "$banner"; printf '\n'
    [ -n "$extra" ] && { fit "$extra"; printf '\n'; }
    local l
    for l in "${lines[@]:$start:$avail}"; do fit "$l"; printf '\n'; done
}

# ── the loop ──────────────────────────────────────────────────────────────────
while true; do
    ROWS="$(term_rows)"
    COLS="$(term_cols)"
    IS_TTY=0; [ -t 1 ] && IS_TTY=1

    OUT="$(capture)"; RC=$?
    ERR="$(tr '\n' ' ' < "$ERRFILE" 2>/dev/null | sed 's/  */ /g; s/ *$//')"
    NOW="$(date +%s)"

    if [ "$RC" -eq 0 ]; then
        # A capture that SUCCEEDS but is unchanged is a QUIET session, not a dead
        # one. Liveness is "the last capture succeeded", never "the content moved".
        LAST_FRAME="$OUT"
        LAST_OK="$NOW"
        render "[live]  omega stream · $LABEL · $(date +%H:%M:%S) · every ${INTERVAL}s${CLOCK_NOTE}" "" "$OUT"
    else
        # Three causes, and the third is why the wall clock exists. Our own budget
        # firing means the box took the connection and then said nothing — which is
        # NOT "unreachable" (we reached it) and NOT "not found" (nobody answered
        # either way), so it gets its own name instead of being filed under a
        # diagnosis we did not earn.
        if timed_out "$RC"; then
            STATE="SOURCE NOT ANSWERING"
            [ -z "$ERR" ] && ERR="no answer within ${CAPTURE_BUDGET}s (capture killed)"
        elif [ "$TARGET" != "local" ] && [ "$RC" -eq 255 ]; then
            # ssh's own 255 means we never reached the box.
            STATE="HOST UNREACHABLE"
        else
            # Any other non-zero: the command RAN and rmux could not find the session.
            STATE="SESSION NOT FOUND"
        fi

        if [ "$LAST_OK" -gt 0 ]; then
            # SOURCE GONE: we HAD frames and stopped getting them. Keep the last good
            # frame on screen, but label it stale with its age — a frozen mirror must
            # never be able to pass for a quiet one.
            AGE=$(( NOW - LAST_OK ))
            render \
                "[warn]  omega stream · $LABEL · $STATE · last good frame ${AGE}s ago" \
                "------- SOURCE GONE · frame below is STALE (${AGE}s old) · ${ERR:-no error text} -------" \
                "$LAST_FRAME"
        else
            render \
                "[warn]  omega stream · $LABEL · $STATE · never connected" \
                "" \
                "$(printf 'omega stream could not read %s\n\n  state : %s\n  exit  : %s\n  error : %s\n\nretrying every %ss — this loop never gives up.' \
                     "$LABEL" "$STATE" "$RC" "${ERR:-(no output on stderr)}" "$INTERVAL")"
        fi
    fi

    sleep "$INTERVAL"
done
