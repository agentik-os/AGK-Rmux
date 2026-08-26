#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════
# OmegaOS · omega monitor · LAYER TWO, the deep audit team
# ───────────────────────────────────────────────────────────────────────────
#   omega-monitor-audit.sh <target> <session> [agent-budget] [max-dimensions]
#     target         : an ssh host alias from ~/.ssh/config, OR the literal "local"
#     session        : the WATCHED rmux session on that box (its pane names the project)
#     agent-budget   : wall clock in seconds for EACH auditor (default 420)
#     max-dimensions : how many auditors may run in parallel (default 6)
#
# Layer one (omega-monitor.sh) polls every minute and answers ONE question: is this
# session stopped, and if so which kind of stopped. This is layer two, on a much
# slower cadence: what is actually WRONG with the work being produced. The watcher
# dispatches it in the BACKGROUND, so nothing here may block the cheap loop, and
# every agent carries its own wall clock: a hung auditor must never wedge the watcher.
#
# ── THE DIMENSIONS COME FROM THE WATCHED PROJECT'S OWN RULES ─────────────────
# A generic audit finds generic nothing. So the first thing this does is DISCOVER
# what the watched project says about itself (CLAUDE.md, RULES.md, AGENTS.md,
# .cursorrules, .claude/CLAUDE.md) ON THE TARGET BOX, and DERIVE the audit dimensions
# from those rules. A project that mandates install parity gets an install-parity
# auditor; one that mandates design tokens gets a token auditor. Only when the project
# states no rules of its own does the baseline apply:
#     design and token rules · access control and isolation · test reality (does the
#     green mean anything) · corpus and documentation integrity
#
# ── AN AUDIT THAT ALWAYS RETURNS "ALL CLEAN" IS AN AUDIT NOBODY READS ────────
# Every auditor is therefore instructed to be ADVERSARIAL, to RANK its findings, to
# carry `file:line` evidence for each one (R-CITE: an uncited assertion is noise), and
# to state in ONE LINE when a rule genuinely holds, so a clean result is still
# information instead of an empty page.
#
# ── READ-ONLY IS ENFORCED BY THE ALLOWLIST, NOT BY THE PROMPT ────────────────
# The agents get Read, Grep and Glob and nothing else. A prompt that says "do not
# edit" is a request; an allowlist is a boundary. There is deliberately NO
# permission-bypass flag anywhere in this file: an auditor that can write is how an
# audit quietly becomes a second author of the thing it is grading.
#
# Inherits the R-STREAM substrate, same as the watcher: rmux is not tmux and lives at
# the absolute $HOME/.local/bin/rmux, that path must reach a REMOTE shell UNEXPANDED,
# a `#{...}` format stays quoted or the remote shell reads `#` as a comment, host
# coordinates come from ~/.ssh/config and are never hardcoded, and every remote call
# is bounded because ConnectTimeout bounds only the connect.
# ═══════════════════════════════════════════════════════════════════════════

# Single quotes here are load-bearing in two places and shellcheck flags both as
# SC2016: the literal `$HOME/...` paths that must reach a REMOTE shell unexpanded, and
# the backticks inside printf format strings, which are markdown code spans in the
# generated report rather than command substitutions.
# shellcheck disable=SC2016

# No `set -e`: one failing auditor must never abort the report that the other five
# already produced. Failures are WRITTEN INTO the report, which is the only place the
# operator will look for them.
set -u

TARGET="${1:-}"
SESSION="${2:-}"
AGENT_BUDGET="${3:-420}"
MAX_DIMENSIONS="${4:-6}"

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
# shellcheck disable=SC2016  # literal on purpose: it must expand on the REMOTE box.
RMUX_ANY='$HOME/.local/bin/rmux'
CLAUDE_LOCAL="${CLAUDE_BIN:-$HOME/.local/bin/claude}"
[ -x "$CLAUDE_LOCAL" ] || CLAUDE_LOCAL="$(command -v claude 2>/dev/null || echo "$CLAUDE_LOCAL")"
# shellcheck disable=SC2016  # same reason: the remote box resolves its own $HOME.
CLAUDE_REMOTE='$HOME/.local/bin/claude'

# Discovery calls are short by nature; only the agents get the long clock.
DISCOVERY_BUDGET=25
DERIVE_BUDGET=180

# The rule files a project may state itself in, in the order they are read.
RULE_FILES="CLAUDE.md RULES.md AGENTS.md .cursorrules .claude/CLAUDE.md"

usage() {
    cat <<'EOF'
omega-monitor-audit.sh: the deep audit team behind `omega monitor`.

  usage: omega-monitor-audit.sh <target> <session> [agent-budget] [max-dimensions]

    target          ssh host alias from ~/.ssh/config, or the literal "local"
    session         the watched rmux session on that box
    agent-budget    per-auditor wall clock in seconds (default 420)
    max-dimensions  parallel auditors (default 6)

  It discovers the WATCHED PROJECT's own rules on the target box, derives the audit
  dimensions from them, runs one read-only adversarial agent per dimension in
  parallel, and writes a timestamped report under ~/.omega/state/monitor/.
EOF
}

die() { printf 'omega-monitor-audit: %s\n\n' "$1" >&2; usage >&2; exit 2; }

[ -n "$TARGET" ]  || die "missing <target>"
[ -n "$SESSION" ] || die "missing <session>"
case "$SESSION" in *\'*) die "session name may not contain a single quote";; esac
case "$TARGET"  in *\'*|-*) die "invalid target '$TARGET'";; esac
case "$AGENT_BUDGET"   in ''|*[!0-9]*) AGENT_BUDGET=420;; esac
case "$MAX_DIMENSIONS" in ''|*[!0-9]*) MAX_DIMENSIONS=6;; esac
[ "$AGENT_BUDGET" -lt 60 ] && AGENT_BUDGET=60
[ "$MAX_DIMENSIONS" -lt 1 ] && MAX_DIMENSIONS=1

LABEL="$TARGET:$SESSION"
SLUG="$(printf '%s' "$LABEL" | tr -c 'A-Za-z0-9._-' '-')"
STATE_DIR="$OMEGA_DIR/state/monitor/$SLUG"
mkdir -p "$STATE_DIR" 2>/dev/null
STAMP="$(date '+%Y%m%d-%H%M%S')"
REPORT="$STATE_DIR/audit-$STAMP.md"

# fd 0 must be OPEN: run_bounded hands the caller's stdin to its child explicitly, and
# the watcher dispatches this script from a background subshell where stdin can be
# anything. A closed fd 0 would fail every agent launch and the report would blame the
# auditors for a plumbing fault.
( : <&0 ) 2>/dev/null || exec 0</dev/null

WORKDIR="$(mktemp -d -t omega-monitor-audit.XXXXXX 2>/dev/null || echo "/tmp/omega-monitor-audit.$$")"
mkdir -p "$WORKDIR" 2>/dev/null
trap 'rm -rf "$WORKDIR"' EXIT

# ── the wall clock ────────────────────────────────────────────────────────────
# Same watchdog as omega-stream.sh and omega-monitor.sh, with the budget as an
# argument because discovery, derivation and the agents want three different clocks.
# `timeout` is GNU coreutils and does not exist on stock macOS, so the bash fallback
# signals the child's PROCESS GROUP: signalling the direct child alone leaves a
# grandchild holding the command substitution's stdout pipe open, and the call never
# returns even though the clock fired.
TIMEOUT_BIN=""
if command -v timeout >/dev/null 2>&1; then TIMEOUT_BIN="timeout"
elif command -v gtimeout >/dev/null 2>&1; then TIMEOUT_BIN="gtimeout"; fi

#
# `<&0` IS LOAD-BEARING here too, and it matters more than in the watcher: every agent
# receives its PROMPT on stdin. Bash sends an asynchronous command's stdin to /dev/null
# unless job control is active, and job control is never active inside a command
# substitution whatever `set -m` says. Measured: without it the child read nothing, so
# on a box using this fallback every auditor would have been launched with an EMPTY
# prompt and would have reported on nothing while looking like it ran.
run_bounded() {   # $1 = seconds, then the command
    local budget="$1"; shift
    if [ -n "$TIMEOUT_BIN" ]; then
        "$TIMEOUT_BIN" -k 5 "$budget" "$@"
        return $?
    fi
    set -m
    "$@" <&0 &
    local pid=$!
    set +m
    (
        sleep "$budget"
        kill -TERM -- "-$pid" 2>/dev/null || kill -TERM "$pid" 2>/dev/null
    ) >/dev/null 2>&1 &
    local watchdog=$!
    wait "$pid"
    local rc=$?
    kill -TERM "$watchdog" >/dev/null 2>&1
    wait "$watchdog" >/dev/null 2>&1
    return $rc
}

# Run a shell command string on the box that holds the watched session.
#
# The command string is written with `$HOME` LITERAL: locally `bash -c` expands it to
# this box's home, remotely the REMOTE login shell expands it to the remote home. One
# string, two boxes, and no path from one machine ever leaks onto the other, which is
# the failure that once pointed a Linux box at /Users/... .
on_target() {   # $1 = budget, $2 = command string; stdin is passed through
    local budget="$1" cmd="$2"
    if [ "$TARGET" = "local" ]; then
        run_bounded "$budget" bash -c "$cmd"
    else
        run_bounded "$budget" ssh \
            -o ConnectTimeout=10 \
            -o ServerAliveInterval=5 \
            -o ServerAliveCountMax=2 \
            -o BatchMode=yes "$TARGET" "$cmd"
    fi
}

log() { printf '%s  %s\n' "$(date '+%H:%M:%S')" "$1"; }

# ── 1 · where does the watched session actually live ──────────────────────────
# The project is whatever the watched pane has open, read from rmux rather than
# guessed: guessing a project directory is how an audit ends up grading a different
# repository and reporting confidently about nothing. `#{pane_current_path}` stays
# single-quoted so a remote shell does not read the `#` as a comment.
log "resolving the project directory of $LABEL"
PROJECT_DIR="$(on_target "$DISCOVERY_BUDGET" \
    "$RMUX_ANY display-message -p -t '$SESSION' '#{pane_current_path}'" 2>/dev/null \
    | head -1 | tr -d '\r')"
if [ -z "$PROJECT_DIR" ]; then
    PROJECT_DIR="$(on_target "$DISCOVERY_BUDGET" \
        "$RMUX_ANY list-panes -t '$SESSION' -F '#{pane_current_path}'" 2>/dev/null \
        | head -1 | tr -d '\r')"
fi
case "$PROJECT_DIR" in
    /*) : ;;
    *)
        # Never invent one. A report that says plainly "I could not find the project"
        # is worth more than four agents auditing $HOME.
        {
            printf '# omega monitor · deep audit · %s\n\n' "$LABEL"
            printf -- '- generated: %s\n' "$(date '+%Y-%m-%d %H:%M:%S')"
            printf -- '- result: **could not resolve the project directory** of session `%s` on `%s`.\n' "$SESSION" "$TARGET"
            printf -- '- rmux returned: `%s`\n\n' "${PROJECT_DIR:-<nothing>}"
            printf 'No audit was run. Either the session no longer exists, rmux is not at `$HOME/.local/bin/rmux` on that box, or the box did not answer within %ss.\n' "$DISCOVERY_BUDGET"
        } > "$REPORT"
        log "no project directory, wrote $REPORT"
        printf '%s\n' "$REPORT"
        exit 0
        ;;
esac
case "$PROJECT_DIR" in *\'*) log "project directory contains a single quote, refusing to quote around it: $PROJECT_DIR"; exit 2;; esac
log "project directory: $PROJECT_DIR"

# ── 2 · what does the project say about itself ────────────────────────────────
RULES_TXT="$WORKDIR/rules.txt"
: > "$RULES_TXT"
FOUND_RULES=""
for rf in $RULE_FILES; do
    # head -c bounds a pathological file; a 200KB CLAUDE.md would otherwise blow the
    # deriver's context and cost the whole audit for one badly kept repo.
    body="$(on_target "$DISCOVERY_BUDGET" "head -c 20000 '$PROJECT_DIR/$rf' 2>/dev/null" 2>/dev/null)"
    [ -z "$body" ] && continue
    FOUND_RULES="$FOUND_RULES $rf"
    {
        printf '\n===== %s =====\n' "$rf"
        printf '%s\n' "$body"
    } >> "$RULES_TXT"
done
FOUND_RULES="${FOUND_RULES# }"
if [ -n "$FOUND_RULES" ]; then
    log "project rules found: $FOUND_RULES"
else
    log "the project states no rules of its own, falling back to the baseline dimensions"
fi

# ── 3 · derive the dimensions from those rules ────────────────────────────────
# The baseline is the fallback, never the default: a generic checklist finds generic
# nothing, which is precisely why the derivation happens at all.
DIMS="$WORKDIR/dimensions.txt"
: > "$DIMS"
baseline_dimensions() {
    cat > "$DIMS" <<'EOF'
design-and-token-rules|Hardcoded values where the project defines a token, scale or theme, and any visual rule stated somewhere and broken elsewhere.
access-control-and-isolation|Who can read or write what: auth checks, tenant and project isolation, secrets reachable from code, privileges wider than the task needs.
test-reality|Does the green mean anything: tests that assert nothing, mocks standing in for the behaviour under test, skipped or always-true assertions, gates that pass on an exit code alone.
corpus-and-documentation-integrity|Documentation and data that contradict the code: instructions naming tools or paths that no longer exist, stale counts, duplicated sources of truth.
EOF
}

DERIVED_FROM="the baseline (the project states no rules of its own)"
if [ -s "$RULES_TXT" ]; then
    DERIVE_PROMPT="$WORKDIR/derive.prompt"
    {
        printf 'You are deriving the DIMENSIONS of a code audit for one specific project.\n\n'
        printf 'Below is what this project states about itself, taken from %s in %s.\n\n' "$FOUND_RULES" "$PROJECT_DIR"
        printf 'Read it and output the %d most auditable, most project-SPECIFIC dimensions it implies:\n' "$MAX_DIMENSIONS"
        printf 'the rules this project would most plausibly be violating right now, the ones where a\n'
        printf 'violation would be concrete and provable from files rather than a matter of taste.\n\n'
        printf 'OUTPUT CONTRACT, obeyed exactly, nothing else in your reply:\n'
        printf '  one line per dimension, at most %d lines\n' "$MAX_DIMENSIONS"
        printf '  each line is: <kebab-case-slug>|<one sentence naming exactly what to look for>\n'
        printf '  no numbering, no bullets, no headings, no preamble, no closing remark\n'
        printf '  a dimension a generic auditor would also produce is a wasted line: prefer the rules\n'
        printf '  that are unusual, load-bearing, or stated with unusual force in the text below\n\n'
        printf -- '----- the project rules -----\n'
        cat "$RULES_TXT"
    } > "$DERIVE_PROMPT"

    log "deriving audit dimensions from the project's own rules"
    RAW_DIMS="$WORKDIR/dimensions.raw"
    if [ "$TARGET" = "local" ]; then
        DERIVE_CMD="cd '$PROJECT_DIR' && '$CLAUDE_LOCAL' -p"
    else
        DERIVE_CMD="cd '$PROJECT_DIR' && $CLAUDE_REMOTE -p"
    fi
    on_target "$DERIVE_BUDGET" "$DERIVE_CMD" < "$DERIVE_PROMPT" > "$RAW_DIMS" 2>"$WORKDIR/derive.err"
    # Parse defensively: keep only lines that honour the contract, slug the left side,
    # cap the count. A deriver that ignored the contract must degrade to the baseline,
    # never inject a sentence of prose as an audit dimension.
    grep -E '^[A-Za-z0-9][A-Za-z0-9 _-]*\|.+' "$RAW_DIMS" 2>/dev/null \
        | head -n "$MAX_DIMENSIONS" \
        | while IFS='|' read -r slug focus; do
            slug="$(printf '%s' "$slug" | tr '[:upper:]' '[:lower:]' | tr -c 'a-z0-9-' '-' | sed 's/--*/-/g; s/^-//; s/-$//' | cut -c1-40)"
            [ -z "$slug" ] && continue
            [ -z "$focus" ] && continue
            printf '%s|%s\n' "$slug" "$focus"
        done > "$DIMS.tmp" 2>/dev/null
    if [ -s "$DIMS.tmp" ]; then
        mv -f "$DIMS.tmp" "$DIMS"
        DERIVED_FROM="the project's own rules ($FOUND_RULES)"
    else
        rm -f "$DIMS.tmp" 2>/dev/null
        log "the deriver produced nothing usable, falling back to the baseline dimensions"
        baseline_dimensions
        DERIVED_FROM="the baseline (the deriver produced no usable dimension from $FOUND_RULES)"
    fi
else
    baseline_dimensions
fi
DIM_COUNT="$(grep -c . "$DIMS" 2>/dev/null || echo 0)"
log "dimensions ($DIM_COUNT), derived from $DERIVED_FROM"

# ── 4 · one read-only adversarial agent per dimension, in PARALLEL ────────────
# Parallel because they are file-disjoint questions about the same tree and running
# them in sequence would multiply the slow layer by six for no gain (R-ORCH).
AGENT_PIDS=""
AGENT_SLUGS=""
INDEX=0
while IFS='|' read -r slug focus; do
    [ -z "$slug" ] && continue
    INDEX=$(( INDEX + 1 ))
    PROMPT="$WORKDIR/agent-$INDEX.prompt"
    OUTF="$WORKDIR/agent-$INDEX.out"
    {
        printf 'You are ONE auditor on a parallel team auditing the project at %s.\n' "$PROJECT_DIR"
        printf 'It is being built RIGHT NOW by an agent in the rmux session %s, so you are grading\n' "$SESSION"
        printf 'live work, not a finished artifact.\n\n'
        printf 'YOUR DIMENSION, and nothing else: %s\n' "$slug"
        printf 'What that means concretely: %s\n\n' "$focus"
        if [ -n "$FOUND_RULES" ]; then
            printf 'This dimension was derived from the project stating its OWN rules in %s.\n' "$FOUND_RULES"
            printf 'Read that file first. The project'"'"'s rules are the standard you audit against,\n'
            printf 'not your own preferences and not a generic best-practice list.\n\n'
        fi
        printf 'YOU ARE READ-ONLY. You have Read, Grep and Glob. Do not attempt to write, edit,\n'
        printf 'run, commit or fix anything. Reporting a defect is the whole job.\n\n'
        printf 'HOW TO REPORT, and this format is the deliverable:\n'
        printf '  * RANK your findings, most severe first. Severity is blast radius plus how silently\n'
        printf '    it fails, never how many lines it touches.\n'
        printf '  * EVERY finding carries `file:line` evidence and the offending text. A finding\n'
        printf '    without a citation is noise and will be dropped.\n'
        printf '  * For each finding: one line naming the defect, one line of evidence, one line on\n'
        printf '    what breaks in practice. No essays.\n'
        printf '  * Be ADVERSARIAL. Try to FALSIFY the claim that this project honours this rule.\n'
        printf '    Look where a violation would be easiest to hide, not where it would be easiest\n'
        printf '    to find. A tidy-looking area is a place you have not looked hard enough yet.\n'
        printf '  * When the rule GENUINELY HOLDS, say so in exactly ONE line, naming the strongest\n'
        printf '    evidence that it holds. An audit that returns "all clean" every time is an audit\n'
        printf '    nobody reads, and an audit that manufactures a finding to look useful is worse.\n'
        printf '    A clean result stated with its evidence is a real result.\n'
        printf '  * Do not repeat this brief back. Start with your findings.\n'
        printf '  * Hard cap: 25 lines.\n\n'
        # WHY THIS PARAGRAPH EXISTS. Only the agent's FINAL message is captured, and the
        # first live run of this script returned three epilogues instead of three audits:
        # every auditor had written its ranked findings mid-run, then closed with a tidy
        # "plan opened and closed, audit delivered above" sign-off. That sign-off WAS the
        # report as far as this script could see, so the findings existed and reached
        # nobody. An audit that reaches nobody is indistinguishable from an audit that
        # found nothing, which is the exact failure this whole layer exists to avoid.
        printf 'WHERE TO PUT IT, and this is not a formatting preference:\n'
        printf '  ONLY YOUR FINAL MESSAGE IS READ. Nothing you print before it is captured.\n'
        printf '  So your LAST message must contain the COMPLETE ranked findings list, in full,\n'
        printf '  with every `file:line` citation written out again.\n'
        printf '  Do NOT end with a summary, a recap, a plan status, a sign-off, or a sentence\n'
        printf '  like "the audit is delivered above": there is no above. Whatever is not in the\n'
        printf '  final message does not exist and the finding is lost.\n'
    } > "$PROMPT"

    if [ "$TARGET" = "local" ]; then
        AGENT_CMD="cd '$PROJECT_DIR' && '$CLAUDE_LOCAL' -p --allowedTools 'Read,Grep,Glob' --disallowedTools 'Write,Edit,MultiEdit,NotebookEdit,Bash,Task,WebFetch,WebSearch'"
    else
        AGENT_CMD="cd '$PROJECT_DIR' && $CLAUDE_REMOTE -p --allowedTools 'Read,Grep,Glob' --disallowedTools 'Write,Edit,MultiEdit,NotebookEdit,Bash,Task,WebFetch,WebSearch'"
    fi

    # Each agent under its OWN wall clock, and backgrounded: a hung auditor is bounded
    # by the clock and, until it fires, must not hold up the five that answered.
    (
        on_target "$AGENT_BUDGET" "$AGENT_CMD" < "$PROMPT" > "$OUTF" 2>"$WORKDIR/agent-$INDEX.err"
        rc=$?
        if [ ! -s "$OUTF" ]; then
            if [ "$rc" -eq 124 ] || [ "$rc" -eq 137 ] || [ "$rc" -eq 143 ]; then
                printf '_This auditor hit its %ss wall clock and produced nothing. The dimension is UNAUDITED, not clean._\n' "$AGENT_BUDGET" > "$OUTF"
            else
                printf '_This auditor failed (exit %s) and produced nothing. The dimension is UNAUDITED, not clean._\n\n```\n%s\n```\n' \
                    "$rc" "$(head -c 600 "$WORKDIR/agent-$INDEX.err" 2>/dev/null)" > "$OUTF"
            fi
        fi
    ) &
    AGENT_PIDS="$AGENT_PIDS $!"
    AGENT_SLUGS="$AGENT_SLUGS $slug"
    log "auditor $INDEX dispatched: $slug (budget ${AGENT_BUDGET}s)"
done < "$DIMS"

for p in $AGENT_PIDS; do wait "$p" 2>/dev/null; done
log "all $INDEX auditor(s) returned"

# ── 5 · the report ────────────────────────────────────────────────────────────
{
    printf '# omega monitor · deep audit · %s\n\n' "$LABEL"
    printf -- '- **generated**: %s\n' "$(date '+%Y-%m-%d %H:%M:%S %Z')"
    printf -- '- **watched session**: `%s` on `%s`\n' "$SESSION" "$TARGET"
    printf -- '- **project**: `%s`\n' "$PROJECT_DIR"
    printf -- '- **project rules found**: %s\n' "${FOUND_RULES:-none}"
    printf -- '- **dimensions derived from**: %s\n' "$DERIVED_FROM"
    printf -- '- **auditors**: %s, read-only (Read/Grep/Glob), %ss each, run in parallel\n\n' "$INDEX" "$AGENT_BUDGET"
    printf 'Each auditor was told to be adversarial, to rank its findings, to cite `file:line` for\n'
    printf 'every one, and to state in ONE line when a rule genuinely holds. A section that says a\n'
    printf 'rule holds is a result; a section saying UNAUDITED is a gap, not a pass.\n'

    i=0
    while IFS='|' read -r slug focus; do
        [ -z "$slug" ] && continue
        i=$(( i + 1 ))
        printf '\n---\n\n## %d. %s\n\n' "$i" "$slug"
        printf '_Looking for: %s_\n\n' "$focus"
        if [ -s "$WORKDIR/agent-$i.out" ]; then
            cat "$WORKDIR/agent-$i.out"
        else
            printf '_No output from this auditor. UNAUDITED, not clean._\n'
        fi
        printf '\n'
    done < "$DIMS"
} > "$REPORT" 2>/dev/null

log "report written: $REPORT"
printf '%s\n' "$REPORT"
