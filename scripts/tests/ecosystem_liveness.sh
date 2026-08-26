#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf -- "$TMP"' EXIT
mkdir -p "$TMP/home/.omega" "$TMP/bin" "$TMP/state"

cat > "$TMP/home/.omega/telegram.toml" <<'EOF'
bot_token = "123:TEST_TOKEN"
allow_user_ids = [1]
EOF
cat > "$TMP/bin/curl" <<'EOF'
#!/usr/bin/env bash
cat >/dev/null
[ "${MOCK_CURL_FAIL:-0}" = "1" ] && exit 22
printf '%s\n' '{"ok":true,"pending_update_count":1}'
EOF
cat > "$TMP/bin/systemctl" <<'EOF'
#!/usr/bin/env bash
if [[ "$*" == *" show "* ]]; then printf '%s\n' 4242; exit 0; fi
if [[ "$*" == *" restart "* ]]; then
  [ "${MOCK_RESTART_FAIL:-0}" = "1" ] && exit 1
  printf '%s\n' "$*" >> "$MOCK_RESTARTS"
  exit 0
fi
exit 0
EOF
cat > "$TMP/bin/ps" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
cat > "$TMP/bin/sleep" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
cat > "$TMP/bin/date" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" == "+%s" ]]; then printf '%s\n' "$MOCK_NOW"; else printf '%s\n' "2026-08-11 12:00:00"; fi
EOF
cat > "$TMP/alert" <<'EOF'
#!/usr/bin/env bash
[ "${MOCK_ALERT_FAIL:-0}" = "1" ] && exit 1
printf '%s\n' "$1" >> "$MOCK_ALERTS"
EOF
chmod +x "$TMP/bin/"* "$TMP/alert"
: > "$TMP/restarts"
: > "$TMP/alerts"

run_probe() {
  local now="$1"
  HOME="$TMP/home" PATH="$TMP/bin:/usr/bin:/bin" MOCK_NOW="$now" \
    MOCK_RESTARTS="$TMP/restarts" MOCK_ALERTS="$TMP/alerts" \
    OMEGA_TG_STATE_DIR="$TMP/state" OMEGA_TG_ALERT_BIN="$TMP/alert" \
    OMEGA_TG_DRAIN_SECONDS=0 OMEGA_TG_COOLDOWN_SECONDS=10 \
    OMEGA_TG_WINDOW_SECONDS=1000 OMEGA_TG_MAX_RESTARTS=3 \
    bash "$ROOT/scripts/omega-atlas-liveness.sh" >/dev/null
}

run_probe_with() {
  local now="$1" curl_fail="$2" restart_fail="$3" alert_fail="$4"
  HOME="$TMP/home" PATH="$TMP/bin:/usr/bin:/bin" MOCK_NOW="$now" \
    MOCK_RESTARTS="$TMP/restarts" MOCK_ALERTS="$TMP/alerts" \
    MOCK_CURL_FAIL="$curl_fail" MOCK_RESTART_FAIL="$restart_fail" \
    MOCK_ALERT_FAIL="$alert_fail" \
    OMEGA_TG_STATE_DIR="$TMP/state" OMEGA_TG_ALERT_BIN="$TMP/alert" \
    OMEGA_TG_DRAIN_SECONDS=0 OMEGA_TG_COOLDOWN_SECONDS=10 \
    OMEGA_TG_WINDOW_SECONDS=1000 OMEGA_TG_MAX_RESTARTS=3 \
    bash "$ROOT/scripts/omega-atlas-liveness.sh" >/dev/null 2>&1
}

run_probe 100
run_probe 111
run_probe 122
run_probe 133
[[ "$(wc -l < "$TMP/restarts")" -eq 3 ]]
grep -q '^OPEN ' "$TMP/state/tg-liveness-circuit-omega-tg-bot"
grep -q 'CIRCUIT OPEN' "$TMP/alerts"

# An open circuit never restarts until the explicit recovery command is used.
run_probe 144
[[ "$(wc -l < "$TMP/restarts")" -eq 3 ]]
HOME="$TMP/home" OMEGA_TG_STATE_DIR="$TMP/state" \
  bash "$ROOT/scripts/omega-atlas-liveness.sh" --reset omega-tg-bot.service >/dev/null
run_probe 200
[[ "$(wc -l < "$TMP/restarts")" -eq 4 ]]

# HTTP/auth transport failure is unhealthy, never a silent zero-pending pass.
restart_count="$(wc -l < "$TMP/restarts")"
if run_probe_with 211 1 0 0; then
  echo "expected probe transport failure" >&2
  exit 1
fi
[[ "$(wc -l < "$TMP/restarts")" -eq "$restart_count" ]]

# A failed restart is surfaced and does not write a cooldown/history receipt.
HOME="$TMP/home" OMEGA_TG_STATE_DIR="$TMP/state" \
  bash "$ROOT/scripts/omega-atlas-liveness.sh" --reset omega-tg-bot.service >/dev/null
if run_probe_with 300 0 1 0; then
  echo "expected restart failure" >&2
  exit 1
fi
[[ ! -s "$TMP/state/tg-liveness-restarts-omega-tg-bot" ]]

# A successful restart with failed operator notification remains an error, but
# the restart receipt is retained so a broken alert channel cannot cause a
# destructive restart loop.
if run_probe_with 311 0 0 1; then
  echo "expected alert delivery failure" >&2
  exit 1
fi
[[ "$(wc -l < "$TMP/state/tg-liveness-restarts-omega-tg-bot")" -eq 1 ]]

# State authority may not be redirected through a symlink.
ln -s "$TMP/state" "$TMP/state-link"
if HOME="$TMP/home" PATH="$TMP/bin:/usr/bin:/bin" \
  OMEGA_TG_STATE_DIR="$TMP/state-link" \
  bash "$ROOT/scripts/omega-atlas-liveness.sh" >/dev/null 2>&1; then
  echo "expected symlinked state directory rejection" >&2
  exit 1
fi

# Config discovery follows OMEGA_DIR rather than silently falling back to
# HOME/.omega, matching the Rust runtime's canonical authority root.
mkdir -p "$TMP/custom-omega/state" "$TMP/empty-home"
cp "$TMP/home/.omega/telegram.toml" "$TMP/custom-omega/telegram.toml"
restart_count="$(wc -l < "$TMP/restarts")"
HOME="$TMP/empty-home" OMEGA_DIR="$TMP/custom-omega" \
  PATH="$TMP/bin:/usr/bin:/bin" MOCK_NOW=400 \
  MOCK_RESTARTS="$TMP/restarts" MOCK_ALERTS="$TMP/alerts" \
  OMEGA_TG_ALERT_BIN="$TMP/alert" OMEGA_TG_DRAIN_SECONDS=0 \
  OMEGA_TG_COOLDOWN_SECONDS=10 OMEGA_TG_WINDOW_SECONDS=1000 \
  OMEGA_TG_MAX_RESTARTS=3 \
  bash "$ROOT/scripts/omega-atlas-liveness.sh" >/dev/null
[[ "$(wc -l < "$TMP/restarts")" -eq $(( restart_count + 1 )) ]]
[[ -s "$TMP/custom-omega/state/tg-liveness-restarts-omega-tg-bot" ]]

echo "ecosystem_liveness: ok"
