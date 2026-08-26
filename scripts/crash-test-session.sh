#!/usr/bin/env bash
# crash-test-session.sh — runtime crash-test of the OmegaOS session layer.
#
# rmux speaks the tmux control protocol, so we drive a THROWAWAY session by its
# CLI and observe real behavior: input, multi-line paste, Unicode width, scroll,
# mouse, clipboard, escape-time, truecolor. No theory — every result is a live
# observation (Law 1: runtime is the only truth).
#
# IMPORTANT — neutralize the environment or you get FALSE failures:
#   • PS1='' before any cursor_x test (a shell prompt contaminates the column)
#   • measure input integrity with `wc -c`, not capture-pane (an 80-col pane
#     WRAPS long lines — that is not input loss)
#   • bracketed-paste markers only appear when the app has enabled DECSET 2004,
#     so we enable it explicitly (cat -v won't, but Claude Code's editor does)
#
# Usage:  ./scripts/crash-test-session.sh        (uses `rmux`; set M=tmux to compare)
# Safe:   dedicated session, killed on exit. Never touches your live sessions.
set +e
M="${M:-rmux}"
S="omega-crashtest-$$"
pass(){ printf '  \033[32mPASS\033[0m %s\n' "$1"; }
fail(){ printf '  \033[31mFAIL\033[0m %s\n' "$1"; FAILS=$((FAILS+1)); }
info(){ printf '  \033[33mINFO\033[0m %s\n' "$1"; }
hdr(){  printf '\n=== %s ===\n' "$1"; }
FAILS=0
cleanup(){ $M kill-session -t "$S" 2>/dev/null; rm -f /tmp/${S}_*.txt; }
trap cleanup EXIT

command -v "$M" >/dev/null || { echo "no '$M' on PATH"; exit 2; }
$M new-session -d -s "$S" -x 80 -y 24 || { echo "could not create session"; exit 2; }
P="$S:0.0"
$M send-keys -t "$P" "PS1='' PS2='' ; clear" Enter; sleep 0.3

# ── A. Bracketed paste (multi-line safety in 2004-aware editors) ──
hdr "A. Bracketed-paste (DECSET 2004)"
$M send-keys -t "$P" "printf '\\033[?2004h'; cat -v" Enter; sleep 0.3
printf 'l1\nl2\nl3' > /tmp/${S}_a.txt
$M load-buffer -b ctA /tmp/${S}_a.txt; $M paste-buffer -b ctA -p -t "$P"; sleep 0.4
$M capture-pane -p -t "$P" | grep -q '200~' \
  && pass "markers present → multi-line paste does NOT submit line-by-line" \
  || fail "no bracketed-paste markers → multi-line paste unsafe"
$M send-keys -t "$P" C-c; sleep 0.2; $M send-keys -t "$P" "PS1='' ; clear" Enter; sleep 0.2

# ── B. Unicode width (cursor accounting) ──
hdr "B. Wide-char width (CJK)"
$M send-keys -t "$P" "printf 'A你好B'" Enter; sleep 0.3
CX=$($M display -p -t "$P" '#{cursor_x}')
[ "$CX" = "6" ] && pass "cursor_x=6 → CJK counted width-2 (no desync)" \
                || fail "cursor_x=$CX ≠ 6 → wide-char width misaccounted"
$M send-keys -t "$P" C-u Enter; sleep 0.2

# ── C. Rapid input integrity (no loss) ──
hdr "C. Rapid keystroke integrity"
BIG=$(printf 'Z%.0s' $(seq 1 500))
$M send-keys -t "$P" "echo -n '$BIG' | wc -c" Enter; sleep 0.4
GOT=$($M capture-pane -p -t "$P" | grep -oE '^[0-9]{2,}$' | tail -1)
[ "${GOT:-0}" = "500" ] && pass "500/500 chars survived" || fail "input loss: wc -c=${GOT:-?}/500"
$M send-keys -t "$P" "clear" Enter; sleep 0.2

# ── D. Scrollback depth ──
hdr "D. Scrollback / history"
$M send-keys -t "$P" "for i in \$(seq 1 120); do echo SB_\$i; done" Enter; sleep 0.6
HB=$($M capture-pane -p -t "$P" -S -100 | grep -c 'SB_')
[ "$HB" -gt 24 ] && pass "scrollback readable beyond one screen ($HB lines)" \
                 || fail "scrollback shallow ($HB lines)"
info "history-limit = $($M show-options -gv history-limit)"

# ── E..I. Option-level checks (what the operator can harden in rmux.conf.omega) ──
hdr "E. Mouse"
[ "$($M show-options -gv mouse)" = "on" ] && pass "mouse on (wheel + drag-select)" \
                                          || fail "mouse off → no wheel scroll / no drag-select"
hdr "F. Clipboard (OSC 52)"
SC=$($M show-options -gv set-clipboard); { [ "$SC" = "on" ] || [ "$SC" = "external" ]; } \
  && pass "set-clipboard=$SC (yank reaches local clipboard over SSH)" \
  || fail "set-clipboard=$SC → copy stuck on server"
info "allow-passthrough = $($M show-options -gv allow-passthrough 2>/dev/null)"
hdr "G. Escape-time (Alt/ESC latency)"
ET=$($M show-options -gv escape-time)
{ [ "${ET:-500}" -le 50 ] 2>/dev/null; } && pass "escape-time ${ET}ms (snappy)" \
                                          || fail "escape-time ${ET}ms → laggy Alt/ESC (want ≤10)"
hdr "H. Color depth"
info "default-terminal = $($M show-options -gv default-terminal 2>/dev/null)"
$M show-options -gv terminal-features 2>/dev/null | grep -qi "RGB" \
  && pass "terminal-features advertises RGB (truecolor)" \
  || fail "no RGB in terminal-features → banded 256-color highlighting"

printf '\n=== done: %s failure(s) ===\n' "$FAILS"
exit $(( FAILS > 0 ? 1 : 0 ))
