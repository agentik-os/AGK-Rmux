#!/usr/bin/env bash
# verify-install.sh — INSTALL PARITY gate (CLAUDE.md Law 0).
#
# Run after ANY OmegaOS improvement. Fails if a fresh
# `git clone … && ./install.sh` would NOT reproduce the current system, or if
# a secret leaked into the repo. Binary changes ship automatically because
# install.sh builds from source; this guards the things that DON'T:
# new assets, uncommitted work, unpushed commits, and secrets.
#
# Usage:  ./scripts/verify-install.sh
# Exit 0 = install is complete and safe. Non-zero = fix before declaring done.

set -u
cd "$(dirname "$0")/.." || exit 2
fail=0
VERIFY_SOURCE_ONLY="${VERIFY_SOURCE_ONLY:-0}"
ok()   { printf '  \033[32m✓\033[0m %s\n' "$1"; }
bad()  { printf '  \033[31m✗ %s\033[0m\n' "$1"; fail=1; }

echo "═══ OmegaOS install-parity check ═══"

# 1. install.sh builds the binary from source (so all Rust features ship).
if grep -q "cargo build --release" install.sh && grep -q "target/release/omega" install.sh; then
  ok "binary built from source (all Rust features ship automatically)"
else
  bad "install.sh does NOT build omega from source"
fi

# 2. Agent prompts (oracle.md v2 etc.) are installed.
if grep -q "agents/\*.md" install.sh; then ok "agents/*.md installed"; else bad "agents/*.md not copied by install.sh"; fi

# 3. Slash commands installed.
if grep -q "omega-\*.md" install.sh; then ok "omega-* slash commands installed"; else bad "omega-* commands not copied"; fi

# 4. Full-scrollback retention persisted in the sourced rmux config, at the
#    deepened 500k default (guards against a regression to the old 100k).
if grep -q "history-limit 500000" config/rmux.conf.omega 2>/dev/null; then ok "rmux history-limit persisted (500000)"; else bad "history-limit not 500000 in rmux.conf.omega"; fi
# 4-bis. The startup history-limit call (omega binary) matches the 500k default.
# (The old Rust telegram bridge was retired — replaced by the Bun bot omega-tg-bot.ts —
# so this now lives only in main.rs.)
if [ "$(grep -ro '"history-limit", "500000"' crates/omega-cli/src/ 2>/dev/null | wc -l)" -ge 1 ]; then ok "omega startup history-limit = 500000 (main.rs)"; else bad "startup history-limit not 500000 in main.rs"; fi

# 4b. Alt+Up/Down scroll bindings use the quoted-string if-shell form. rmux 0.3.1
#     rejects the brace `{ … }` command-list form ("bind-key does not accept a
#     parsed command-list argument") and silently drops the binding — so scroll
#     never works on a fresh install. Guard against the brace form regressing.
if grep -qE "bind-key -n M-Up .*\{ *send-keys" config/rmux.conf.omega 2>/dev/null; then
  bad "rmux Alt+Up uses brace syntax rmux 0.3.1 rejects (use quoted if-shell args)"
elif grep -qE "bind-key -n M-Up +if-shell .*'send-keys -X scroll-up'" config/rmux.conf.omega 2>/dev/null; then
  ok "rmux scroll bindings use rmux-0.3-compatible quoted syntax"
else
  bad "rmux Alt+Up/Down scroll bindings missing from rmux.conf.omega"
fi

# 4b-bis. Alternate-scroll fallback: an altscreen TUI that never requests mouse
#     reporting (Codex's Ctrl+T transcript, less, git log) gets NOTHING from
#     `send -M`, so the wheel must be translated to arrow keys instead.
if grep -qE "bind-key -n WheelUpPane +if-shell" config/rmux.conf.omega 2>/dev/null \
   && grep -qE "bind-key -n WheelDownPane +if-shell" config/rmux.conf.omega 2>/dev/null; then
  ok "rmux alternate-scroll fallback bound (wheel works in altscreen TUIs)"
else
  bad "rmux WheelUp/WheelDownPane alternate-scroll bindings missing from rmux.conf.omega"
fi

# 4b-ter. Modified wheel takes over from applications that capture mouse
#     reporting; both modifiers are required because terminals may consume
#     Shift+wheel before rmux ever receives it.
if grep -qF "bind-key -n S-WheelUpPane copy-mode -e" config/rmux.conf.omega 2>/dev/null \
   && grep -qF "bind-key -n M-WheelUpPane copy-mode -e" config/rmux.conf.omega 2>/dev/null; then
  ok "rmux modified-wheel takeover bindings present (Shift and Alt)"
else
  bad "rmux modified-wheel takeover bindings missing from rmux.conf.omega"
fi

# 4b-quater. Non-modal scroll: the wheel enters copy-mode with no key, and any
#     typed key must cancel the mode AND still reach the pane, otherwise the
#     first character of every sentence typed after a scroll is swallowed.
CM_RESEND=$(grep -cE "^bind-key -T copy-mode .+ 'send-keys -X cancel ; send-keys " config/rmux.conf.omega 2>/dev/null || echo 0)
if [[ "$CM_RESEND" -ge 70 ]] && grep -qF "bind-key -T copy-mode Any 'send-keys -X cancel'" config/rmux.conf.omega 2>/dev/null; then
  ok "rmux non-modal scroll wired ($CM_RESEND keys cancel-and-deliver + Any fallback)"
else
  bad "rmux non-modal scroll bindings missing/incomplete in rmux.conf.omega (got $CM_RESEND cancel-and-deliver keys)"
fi

# 4c. Status-bar clock is localized via `omega clock` (not the rmux server's
#     UTC strftime), so the bar shows the operator's wall time on a headless VPS.
if grep -q "omega clock" config/rmux.conf.omega 2>/dev/null; then ok "rmux status clock localized via omega clock"; else bad "rmux status clock not wired to omega clock"; fi

# 4d. timezone knob documented for the operator in the shipped config template.
if grep -q "^# timezone" config/default.toml 2>/dev/null; then ok "timezone config documented in default.toml"; else bad "timezone knob missing from default.toml"; fi

# 4e. Terminal UX hardening (crash-test findings) persisted in the sourced rmux
#     config: mouse scroll/select, OSC52 clipboard, snappy escape-time, truecolor.
#     These are the defaults rmux gets wrong for an all-day agent session.
RC=config/rmux.conf.omega
while IFS= read -r opt; do
  [ -z "$opt" ] && continue
  if grep -qF "$opt" "$RC" 2>/dev/null; then ok "rmux: '$opt' persisted"; else bad "rmux: '$opt' missing from rmux.conf.omega"; fi
done <<'OPTS'
set -g mouse on
set -g set-clipboard external
set -g allow-passthrough on
set -g escape-time 10
set -sa terminal-features ",*:RGB"
set -g focus-events on
set-environment -g COLORTERM truecolor
set-environment -g FORCE_COLOR 1
OPTS

# 4e-bis. Agent panes must unset inherited NO_COLOR. Cursor starts the rmux
#     daemon with NO_COLOR=1; without this, Claude/Codex splash is grayscale.
if grep -qF 'unset NO_COLOR; export FORCE_COLOR=1 COLORTERM=truecolor' crates/omega-core/src/agents.rs \
   && ! grep -qE 'export NO_COLOR=|NO_COLOR=1 exec' crates/omega-core/src/agents.rs; then
  ok "agent launch unsets inherited NO_COLOR (keeps pane color)"
else
  bad "agent launch does not strip inherited NO_COLOR — Claude/Codex go grayscale"
fi

# 4f. Optional low-latency SSH (mosh) bootstrapped best-effort by install.sh.
#     Predictive local echo + UDP diffs → lag-free typing/streaming on a far VPS.
if grep -q "install_mosh_optional" install.sh; then ok "mosh (low-latency SSH) bootstrapped by install.sh"; else bad "mosh bootstrap step missing from install.sh"; fi

# 4g. System-wide rmux config so EVERY user (root + future accounts) gets the
#     hardened session via /etc/rmux.conf, not just the installing user.
if grep -q "/etc/omega/rmux.conf.omega" install.sh && grep -q "/etc/rmux.conf" install.sh; then ok "system-wide rmux config (/etc/rmux.conf) wired for all users"; else bad "system-wide rmux config step missing from install.sh"; fi

# 4h. UTF-8 locale guaranteed so mosh never degrades + TUI renders correctly,
#     for root + future users (what the Termius mosh -l LANG=… relies on).
if grep -q "ensure_utf8_locale" install.sh; then ok "UTF-8 locale guaranteed by install.sh"; else bad "UTF-8 locale guarantee missing from install.sh"; fi

# 4i. Claude Code set to normal-screen so the full conversation scrolls in the
#     rmux panel (flows into the 500k scrollback) instead of Claude's fullscreen
#     buffer. Written to the shell env file by install.sh for every session.
if grep -q "CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN" install.sh; then ok "Claude normal-screen (full scroll in rmux panel) wired by install.sh"; else bad "CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN not set by install.sh"; fi

# 4j. Omega chrome theme (Monogram model) persisted in the sourced rmux config:
#     dark chrome + cyan accent for focus, dim passives, reserved amber/red
#     alert channel. Sentinels = the state-differentiation load-bearers.
RC=config/rmux.conf.omega
while IFS= read -r opt; do
  [ -z "$opt" ] && continue
  if grep -qF "$opt" "$RC" 2>/dev/null; then ok "rmux theme: '$opt' persisted"; else bad "rmux theme: '$opt' missing from rmux.conf.omega"; fi
done <<'OPTS'
set -g status-style "bg=#16161a,fg=#7f7f7f"
set -g window-status-current-style "fg=#000000,bg=#00ffff,bold"
set -g window-status-bell-style "fg=#000000,bg=#ff6e6e,bold"
set -g pane-active-border-style "fg=#00ffff"
set -g message-style "bg=#ffa500,fg=#000000,bold"
OPTS

# 5. Self-improvement crons scheduled.
if grep -q "omega patrol" install.sh; then ok "patrol/usage crons scheduled"; else bad "crons missing from install.sh"; fi

# 6. NO SECRETS tracked. Telegram-token pattern + hardcoded bot_token + .omega ignored.
if git grep -nIE "[0-9]{9,10}:[A-Za-z0-9_-]{30,}" -- . >/dev/null 2>&1; then
  bad "possible token committed to the repo!"
else
  ok "no telegram-token pattern in tracked files"
fi
if grep -qE "^\.omega/|^\.omega$" .gitignore 2>/dev/null; then ok ".omega/ (secrets) gitignored"; else bad ".omega/ not gitignored"; fi

# 7. Working tree clean → a fresh clone is complete.
if [ -z "$(git status --porcelain)" ]; then ok "working tree clean (fresh clone = complete)"; else bad "uncommitted changes — a fresh install would NOT get them"; fi

# 8. Local branch in sync with origin (latest pushed).
git fetch -q origin 2>/dev/null || true
if [ -n "$(git rev-parse @ 2>/dev/null)" ] && [ "$(git rev-parse @ 2>/dev/null)" = "$(git rev-parse '@{u}' 2>/dev/null)" ]; then
  ok "local in sync with origin (latest pushed)"
else
  bad "local commits not pushed to origin"
fi

# 8b. npm channel parity: the npx installer (installer/package.json) version must
#     match what the registry actually serves — a version bump without
#     `npm publish` ships nobody anything, and the npm channel otherwise sits
#     entirely outside this gate. Skips gracefully offline / without node+npm.
if command -v npm >/dev/null 2>&1 && command -v node >/dev/null 2>&1; then
  REPO_NPM_VER="$(node -p 'require("./installer/package.json").version' 2>/dev/null)"
  REG_NPM_VER="$(timeout 15 npm view omega-os version 2>/dev/null)"
  if [ -z "$REG_NPM_VER" ]; then
    ok "npm registry unreachable (offline?) — installer version-parity skipped (repo: ${REPO_NPM_VER:-?})"
  elif [ "$REPO_NPM_VER" = "$REG_NPM_VER" ]; then
    ok "npx installer version matches the npm registry (omega-os $REG_NPM_VER)"
  else
    bad "installer/package.json $REPO_NPM_VER != published omega-os $REG_NPM_VER — npm publish (or revert the bump)"
  fi
else
  ok "npm/node absent — installer version-parity check skipped"
fi

# 9. Self-containment assets (reset-survival): persistent service, hooks, identity.
if [ -f telegram-bot/omega-tg-bot.ts ] && [ -f scripts/omega-tg-up.sh ] && grep -q "omega-tg-bot.ts" install.sh && grep -q "omega-tg-up" install.sh; then ok "Telegram command bot shipped + wired (omega-tg-bot + omega-tg-up + systemd service)"; else bad "Telegram command bot (omega-tg-bot.ts / omega-tg-up.sh) not shipped/wired in install.sh"; fi
if [ -f telegram-bot/inbox-bot.ts ] && [ -f scripts/inbox-bot-up.sh ] && grep -q "inbox-bot.ts" install.sh && grep -q "inbox-bot-up" install.sh && grep -q "omega-inbox-bot.service" install.sh; then ok "Deposit bot shipped + wired (inbox-bot + inbox-bot-up + omega-inbox-bot.service)"; else bad "Deposit bot (inbox-bot.ts / inbox-bot-up.sh) not shipped/wired in install.sh"; fi
# A deposit must reach EVERY session that needs it. ~/.omega lives under a 0700
# home, so a second Unix account (a paired session on the same box) cannot read
# it at all — the bot mirrors each deposit into a group-shared box instead, and
# holds credential-looking files back so a shared box never becomes a key leak.
if grep -q "function fanout" telegram-bot/inbox-bot.ts && grep -q "SECRETISH" telegram-bot/inbox-bot.ts; then ok "deposit fan-out present (second session reachable, credentials held back)"; else bad "deposit fan-out missing — a deposit reaches only the bot owner's home"; fi
# Named boxes: an untagged deposit must reach EVERY box (the operator must never
# have to remember a tag for a file to be reachable), and the boxes must be hard
# links so N boxes cost one file's disk, not N.
if grep -q "function boxesFor" telegram-bot/inbox-bot.ts && grep -q "linkSync" telegram-bot/inbox-bot.ts; then ok "deposit boxes wired (untagged → all boxes, hard-linked)"; else bad "deposit named boxes missing/not hard-linked in inbox-bot.ts"; fi
# Mission Control (separate Docker web UI) is retired. Telegram command bot stays.
if [ -f scripts/omega-mc-up.sh ]; then
  bad "retired omega-mc-up.sh still in the repo"
elif grep -qE 'git clone.*agentik-telegram' install.sh; then
  bad "install.sh still clones the retired Mission Control repo"
elif grep -qE 'cp -f .*omega-mc-up' install.sh; then
  bad "install.sh still copies the retired Mission Control launcher"
else
  ok "OmegaMC retired (no launcher, no clone, no copy — Telegram command bot remains)"
fi
if ls scripts/hooks/*.sh >/dev/null 2>&1 && grep -q "scripts/hooks" install.sh; then ok "tracking + verify hooks shipped + installed"; else bad "hooks not shipped/wired"; fi
if [ -f agents/identity/SOUL.template.md ] && grep -q "SOUL.template" install.sh; then ok "SOUL identity template shipped + installed"; else bad "SOUL template not shipped/wired"; fi
if grep -q "usage --check" install.sh; then ok "native billing cron (omega usage --check) scheduled"; else bad "native billing cron missing from install.sh"; fi

# Phase 1 /duo assets: prove the source bridge is executable and the installer
# copies both shipped assets, restores executable mode, and exposes the bridge
# on PATH. These are separate assertions so a partial install cannot look green.
if [ -x tools/duo/bin/omega-duo ] && [ -f skills/duo/SKILL.md ] \
  && grep -qF 'if [[ -x "$DUO_SRC/bin/omega-duo" && -f "$DUO_SKILL_SRC/SKILL.md" ]]' install.sh \
  && grep -qF 'cp -f "$DUO_SRC/bin/omega-duo" "$DUO_SKILL_DST/bin/omega-duo"' install.sh \
  && grep -qF 'cp -f "$DUO_SKILL_SRC/SKILL.md" "$DUO_SKILL_DST/SKILL.md"' install.sh; then
  ok "/duo bridge + skill sources shipped and copied"
else
  bad "/duo bridge/skill source or installer copy wiring missing"
fi
if grep -qF 'chmod +x "$DUO_SKILL_DST/bin/omega-duo"' install.sh \
  && grep -qF 'ln -sf "$DUO_SKILL_DST/bin/omega-duo" "$HOME/.local/bin/omega-duo"' install.sh; then
  ok "omega-duo installed executable and linked onto PATH"
else
  bad "omega-duo executable/PATH wiring missing from install.sh"
fi

# `omega monitor <target>` assets. Three SEPARATE assertions, because a monitor
# that installs its watcher but not its audit runner, or its scripts but not its
# skill, is a half feature that would still look green under one combined check.
# The watcher is useless without the report directory, so that is asserted too:
# the first audit run must never race on a missing path.
if [ -f scripts/omega-monitor.sh ] && [ -f scripts/omega-monitor-audit.sh ] \
  && grep -qF 'for MON in omega-monitor omega-monitor-audit; do' install.sh; then
  ok "monitor watcher + audit runner shipped and copied by install.sh"
else
  bad "monitor watcher/audit runner source or installer copy wiring missing"
fi
if grep -qF 'mkdir -p "$OMEGA_DIR/state/monitor"' install.sh; then
  ok "monitor audit report directory created by install.sh"
else
  bad "monitor audit report directory (state/monitor) missing from install.sh"
fi
if [ -f skills/monitor/SKILL.md ] \
  && grep -qF 'MONITOR_SKILLS=(monitor)' install.sh \
  && grep -qF 'for NSK in "${MONITOR_SKILLS[@]}"; do' install.sh; then
  ok "/monitor skill shipped + wired (/monitor + /omg-monitor)"
else
  bad "/monitor skill not shipped/wired in install.sh"
fi

# Codex credential topology is resolved by omega-core, where JSON validity and
# last_refresh can be compared atomically. The installer must honor a nonempty
# CODEX_HOME, must not make the old fixed .pre-omega duplicate choice, and must
# provision the owner-only quarantine before invoking the Rust reconciler.
if grep -qF 'if [[ -n "${CODEX_HOME:-}" ]]' install.sh \
  && grep -qF 'CODEX_NATIVE_DIR="$CODEX_HOME"' install.sh \
  && grep -qF 'CODEX_NATIVE_AUTH="$CODEX_NATIVE_DIR/auth.json"' install.sh; then
  ok "installer honors nonempty CODEX_HOME for native Codex auth"
else
  bad "installer does not honor nonempty CODEX_HOME"
fi
if ! grep -qE 'migrate_creds[[:space:]]+"codex"' install.sh \
  && grep -qF 'OMEGA_DIR="$OMEGA_DIR" CODEX_HOME="$CODEX_NATIVE_DIR"' install.sh \
  && grep -qF '"$INSTALL_DIR/omega" codex-reconcile --json' install.sh \
  && grep -qF '#[command(name = "codex-reconcile")]' crates/omega-cli/src/main.rs \
  && grep -qF 'pub fn reconcile_codex(&self)' crates/omega-core/src/credentials.rs \
  && grep -qF 'codex_login::reconcile_on_startup' crates/omega-cli/src/main.rs; then
  ok "Codex split reconciliation uses the authoritative omega-core command"
else
  bad "unsafe shell-owned Codex credential reconciliation remains"
fi
if grep -qF '#[command(name = "codex-login-abort")]' crates/omega-cli/src/main.rs \
  && grep -qF 'pub fn abort(pid: u32)' crates/omega-core/src/codex_login.rs \
  && grep -qF 'abort_requires_owned_pid_and_settles_after_exit_sentinel' crates/omega-core/src/codex_login.rs; then
  ok "Codex login abort is PID-bound, sentinel-settled, and runtime-tested"
else
  bad "safe Codex login abort path is incomplete"
fi
if grep -qF 'acct:codexabort:' telegram-bot/omega-tg-bot.ts \
  && grep -qF 'codex-login-abort' telegram-bot/omega-tg-bot.ts; then
  ok "Telegram Codex Cancel uses the explicit abort callback"
else
  bad "Telegram Codex Cancel does not use the explicit abort callback"
fi
if grep -qF 'mkdir -p "$OMEGA_DIR/credentials/accounts" "$OMEGA_DIR/credentials/quarantine"' install.sh \
  && grep -qF 'chmod 700 "$OMEGA_DIR/credentials"' install.sh \
  && grep -qF '    "$OMEGA_DIR/credentials/accounts" \' install.sh \
  && grep -qF '    "$OMEGA_DIR/credentials/quarantine"' install.sh; then
  ok "credential store + quarantine provisioned owner-only"
else
  bad "owner-only credential quarantine provisioning missing"
fi
if grep -qF 'install_bun_required' install.sh \
  && grep -qF 'OmegaOS will not install an unusable /duo bridge' install.sh \
  && grep -qF 'Bun disappeared before /duo installation' install.sh; then
  ok "Bun is a required, fail-closed runtime for /duo"
else
  bad "fresh installs can still ship /duo without its Bun runtime"
fi
if grep -qF 'pub async fn probe_codex_auth()' crates/omega-core/src/doctor.rs \
  && grep -qF 'doctor_checks(&config, deep)' crates/omega-cli/src/main.rs; then
  ok "live Codex auth probing is gated behind explicit doctor --deep"
else
  bad "live Codex auth probe is not explicitly gated"
fi

# The bridge's deterministic fake-agent suite is the real runtime gate. Verify
# both its exit status and the named invariants; a shortened suite is not green.
if command -v bun >/dev/null 2>&1; then
  DUO_SELFTEST_OUTPUT="$(tools/duo/bin/omega-duo --self-test 2>&1)"
  DUO_SELFTEST_RC=$?
  DUO_INVARIANTS_OK=1
  while IFS= read -r invariant; do
    [ -z "$invariant" ] && continue
    if ! grep -qF "ok   $invariant" <<<"$DUO_SELFTEST_OUTPUT"; then
      bad "omega-duo self-test omitted invariant: $invariant"
      DUO_INVARIANTS_OK=0
    fi
  done <<'DUO_INVARIANTS'
attempt zero uses native read-only argv
transient provider failure never enters guarded degraded mode
injected watcher failure is the real fail-closed result
Git metadata mutation outside HEAD and index is detected
native read-only mode ignores unrelated ignored-artifact churn
blocked Claude plan read uses disclosed guarded degradation
native read-only guard supports detached HEAD
exit-zero preflight auth diagnostic is normalized to failed JSON and process status
prefixed task echoes cannot trigger auth, quota fallback, or exhaustion
Claude sandbox availability and no-permission prose cannot authorize degradation
transport failure classes independently veto sandbox degradation
exact local Bubblewrap bootstrap denial authorizes guarded degradation
generic repository-read prose cannot authorize degradation
DUO_INVARIANTS
  if [[ "$DUO_SELFTEST_RC" -eq 0 && "$DUO_INVARIANTS_OK" -eq 1 ]]; then
    ok "omega-duo source self-test passed with every required invariant"
  else
    bad "omega-duo source self-test failed or incomplete"
  fi
elif [ "$VERIFY_SOURCE_ONLY" = "1" ] && ! command -v bun >/dev/null 2>&1; then
  ok "Bun runtime self-test skipped in VERIFY_SOURCE_ONLY mode (installer wiring checked above)"
else
  bad "Bun absent: omega-duo runtime gate cannot execute"
fi

# 10. Orchestration engine (Gate+Driver+Guardian) + planner skill shipped.
if [ -f crates/omega-core/src/executor.rs ] && [ -f crates/omega-core/src/guardian.rs ]; then ok "orchestration engine (executor + guardian) in source"; else bad "engine source (executor/guardian) missing"; fi
if grep -qE "PlanRun|PlanStatus|plan-run|plan_run" crates/omega-cli/src/main.rs; then ok "omega plan-run/plan-status CLI present"; else bad "engine CLI commands missing from main.rs"; fi
if [ -f skills/planner/SKILL.md ] && [ -f skills/planner/fallback/plan.ts ] && grep -q "omg-planner" install.sh; then ok "/omg-planner skill + Bun fallback shipped + installed"; else bad "planner skill not shipped/wired in install.sh"; fi
if [ -f skills/agentic-engineering-lab/SKILL.md ] && grep -q "skills/agentic-engineering-lab" install.sh && grep -q "omg-lab" install.sh && grep -q "pub mod lab" crates/omega-core/src/lib.rs; then ok "AGK Lab skill + rust module shipped + wired"; else bad "agentic-engineering-lab skill or lab module not wired"; fi
if [ -f skills/new-project/SKILL.md ] && grep -q "skills/new-project" install.sh && grep -q "omg-new-project" install.sh; then ok "/omg-new-project end-to-end skill shipped + installed"; else bad "new-project skill not shipped/wired in install.sh"; fi
# Linear feedback-resolution skill + one-time setup wizard shipped + wired,
# self-contained (RULES.md present, no maintainer-private audit-selector dep).
if [ -f skills/linear/SKILL.md ] && [ -f skills/linear/RULES.md ] && grep -q "skills/linear" install.sh && grep -q "omg-linear" install.sh; then ok "/omg-linear skill (+ RULES.md) shipped + installed"; else bad "linear skill not shipped/wired in install.sh"; fi
if [ -f skills/linear-setup/SKILL.md ] && grep -q "skills/linear-setup" install.sh && grep -q "omg-linear-setup" install.sh; then ok "/omg-linear-setup wizard shipped + installed"; else bad "linear-setup skill not shipped/wired in install.sh"; fi
if [ -f skills/10x/SKILL.md ] && [ -f skills/pitch/SKILL.md ] && [ -f skills/ghost/SKILL.md ] && grep -q "for TSK in 10x pitch ghost" install.sh; then ok "thinking micro-skills (/10x /pitch /ghost) shipped + wired"; else bad "thinking micro-skills missing or not wired in install.sh"; fi
if [ -f skills/watch/SKILL.md ] && [ -f skills/watch/scripts/watch.py ] && [ -f skills/watch/LICENSE ] && grep -q 'WATCH_SRC="$OMEGA_SRC/skills/watch"' install.sh && grep -q "omg-watch" install.sh; then ok "watch video-analysis skill (/watch /omg-watch) shipped + wired"; else bad "watch skill missing or not wired in install.sh"; fi
if [ -f skills/artifact-design/SKILL.md ] && grep -q 'ARTD_SRC="$OMEGA_SRC/skills/artifact-design"' install.sh && grep -q "omg-artifact-design" install.sh; then ok "artifact-design skill (/artifact-design /omg-artifact-design) shipped + wired"; else bad "artifact-design skill missing or not wired in install.sh"; fi
if [ -f skills/secrets-vault/SKILL.md ] && [ -f skills/secrets-vault/bin/vault-restore.sh ] && grep -q 'SV_SRC="$OMEGA_SRC/skills/secrets-vault"' install.sh && grep -q "omg-secrets-vault" install.sh; then ok "secrets-vault skill (/secrets-vault /omg-secrets-vault) shipped + wired"; else bad "secrets-vault skill missing or not wired in install.sh"; fi
# The wrapper has no .sh extension, so the generic chmod patterns miss it — assert the
# explicit chmod is present, else a fresh install ships it non-executable.
if [ -f skills/pixelrag/SKILL.md ] && [ -x skills/pixelrag/pixelrag ] && grep -q 'PXR_SRC="$OMEGA_SRC/skills/pixelrag"' install.sh && grep -q 'chmod +x "$PXR_DST/pixelrag"' install.sh && grep -q "omg-pixelrag" install.sh; then ok "pixelrag skill (/pixelrag /omg-pixelrag) shipped + wired"; else bad "pixelrag skill missing or not wired in install.sh"; fi
# Linear skill must not LEAK a maintainer-private path (it may mention them in
# negative "does NOT read ~/.claude" prose; only a real ~/.claude/ or /home/hacker
# path token is a leak). Check for the home-dir literal specifically.
if ! grep -q "/home/hacker" skills/linear/RULES.md skills/linear/SKILL.md; then ok "linear skill has no hardcoded maintainer path"; else bad "linear skill leaks /home/hacker"; fi
# Pipeline self-containment: OmegaOS ships its OWN /omg-vision + /omg-prd, so a
# fresh install does not depend on the user's personal /vision /prd. The
# new-project skill must delegate to the /omg-* versions, not the bare ones.
if [ -f skills/vision/SKILL.md ] && [ -f skills/prd/SKILL.md ] && grep -q "omg-\$psk" install.sh; then ok "/omg-vision + /omg-prd shipped (pipeline self-contained)"; else bad "vision/prd not shipped as /omg-* (fresh install pipeline would break)"; fi
if [ -f skills/product-development-system/SKILL.md ] && grep -q "product-development-system" install.sh; then ok "product-development-system skill shipped + wired"; else bad "product-development-system missing from install.sh (fresh install would not get it)"; fi
# AISB doctrine: 15 Matrix agents, quality kernel, shipped shared protocol,
# no retired numeric IDs in live prompts, no maintainer-home leaks.
if [ -f agents/aisb/_quality-kernel.md ] && [ -f agents/aisb/trinity.md ] && [ -f agents/aisb/protocols/shared-protocol.md ]; then
  ok "AISB quality kernel + Trinity + shared-protocol shipped"
else
  bad "AISB quality kernel, Trinity prompt, or shared-protocol.md missing"
fi
if grep -q "15 MATRIX MANAGERS" agents/aisb-atlas.md && grep -q "trinity" agents/aisb-atlas.md && grep -q "15 Matrix manager agents" telegram-bot/omega-tg-bot.ts; then
  ok "Atlas + Telegram advertise 15 Matrix agents including Trinity"
else
  bad "Atlas/Telegram still advertise a stale Matrix roster (need 15 + trinity)"
fi
if grep -q "aisb/protocols" install.sh && grep -q "_quality-kernel.md" telegram-bot/omega-tg-bot.ts; then
  ok "install.sh copies AISB protocols; Telegram Atlas/oracle prepend the quality kernel"
else
  bad "install.sh or Telegram bot missing quality-kernel / protocols wiring"
fi
if grep -q "fn provider_harness_block" crates/omega-core/src/rules.rs \
   && grep -q "fn policy_context_for_agent" crates/omega-core/src/orchestration.rs \
   && grep -q 'opencode' crates/omega-core/src/orchestration.rs \
   && grep -q 'link_policy_kernel' crates/omega-cli/src/main.rs \
   && grep -q 'OpenCode' crates/omega-cli/src/main.rs \
   && grep -q 'OMEGAOS-KERNEL:START' crates/omega-core/src/hermes_sync.rs \
   && grep -q 'sync_hermes_home' crates/omega-cli/src/main.rs \
   && grep -q 'AISB protocols synced' crates/omega-cli/src/main.rs \
   && grep -q 'provider: Option<String>' crates/omega-cli/src/main.rs; then
  ok "provider harness overlay + OpenCode/Hermes/AISB-protocol sync wiring present"
else
  bad "rules are not mapped per-harness (Claude/Codex/OpenCode/Hermes) or omega sync misses OpenCode/Hermes"
fi
if grep -q 'omega install hermes' install.sh \
   && grep -q -- '--skip-setup' crates/omega-core/src/agents.rs \
   && grep -q 'HERMES_HOME=' crates/omega-core/src/agents.rs \
   && grep -q '.hermes/bin' crates/omega-core/src/agents.rs \
   && grep -q 'pub fn sync_hermes_home' crates/omega-core/src/hermes_sync.rs \
   && grep -q 'skill-bundles' crates/omega-core/src/hermes_sync.rs \
   && grep -q 'hermes-gateway' crates/omega-cli/src/main.rs \
   && grep -q 'hermes-gateway install' install.sh \
   && grep -q 'fn hermes_gateway_check' crates/omega-core/src/doctor.rs \
   && grep -q 'telegram_collision' crates/omega-core/src/hermes_gateway.rs; then
  ok "Hermes Home + messaging gateway are installed, synced, and isolated from Atlas Telegram"
else
  bad "Hermes install/sync/gateway wiring missing from install.sh or omega-core"
fi

if [ -f scripts/agk ] \
   && grep -q 'omega agk provision' install.sh \
   && grep -q 'scripts/agk' install.sh \
   && grep -q 'scope.env' install.sh \
   && grep -q 'enum AgkScope' crates/omega-core/src/tenancy.rs \
   && grep -q 'hermes profile create' crates/omega-core/src/tenancy.rs \
   && grep -q 'fn sandbox_bind_args' crates/omega-core/src/tenancy.rs \
   && grep -q 'fn agk_scope_check' crates/omega-core/src/doctor.rs \
   && grep -q 'Commands::Agk' crates/omega-cli/src/main.rs \
   && grep -q 'provision_all' crates/omega-cli/src/main.rs \
   && grep -q 'hermes_launch_home' crates/omega-core/src/agents.rs \
   && [ -f docs/SCOPE-TENANCY.md ] \
   && [ -f docs/TOPOLOGY.md ] \
   && [ -f docs/DISCORD-SURFACE.md ] \
   && grep -q 'agk topology init' install.sh \
   && grep -q 'topology.sample.yaml' install.sh \
   && grep -q 'fn load_resolved' crates/omega-core/src/topology.rs \
   && grep -q 'fn discord_bots' crates/omega-core/src/surfaces.rs \
   && grep -q 'fn team_for_os' crates/omega-core/src/specialists.rs; then
  ok "AGK four-scope tenancy + official Hermes profiles + bwrap sibling-hide are wired"
else
  bad "AGK scope provision/switch/isolation missing from install.sh or omega-core"
fi
if grep -q '"R-PLANE"' crates/omega-core/src/rules.rs \
   && grep -q '"R-AGK"' crates/omega-core/src/rules.rs \
   && grep -q '"R-LAYER"' crates/omega-core/src/rules.rs \
   && grep -q '"R-TOPO"' crates/omega-core/src/rules.rs \
   && grep -q '"R-SURFACE"' crates/omega-core/src/rules.rs \
   && grep -q '## AGK layers' crates/omega-core/src/rules.rs \
   && grep -qE 'canonical_dirs|infrastructure' crates/omega-core/src/tenancy.rs; then
  ok "AGK doctrine (R-PLANE/R-AGK/R-LAYER/R-TOPO/R-SURFACE + AGK layers kernel + canonical workspace trees)"
else
  bad "AGK rules or canonical workspace seed missing from rules.rs / tenancy.rs"
fi
aisb_dead_ids=$(grep -RlnE '\*\*R-(18|19|21|28|35)\*\*|owns R-(18|19|21|28|35)' agents/aisb --include='*.md' | grep -v '_quality-kernel.md' | grep -v 'protocols/shared-protocol.md' | grep -v 'CLAUDE.md' || true)
if [ -z "$aisb_dead_ids" ]; then
  ok "AISB prompts do not own retired R-18/R-19/R-21/R-28/R-35 IDs"
else
  bad "AISB prompts still teach retired numeric rule IDs: $aisb_dead_ids"
fi
if ! grep -qE '/home/hacker|/VibeCoding/' agents/aisb/*.md skills/brand-identity/SKILL.md skills/audits/_shared/AUDIT-VERIFICATION-CONTRACT.md; then
  ok "shipped AISB + brand-identity + audit contract have no maintainer-home leaks"
else
  bad "shipped AISB/skills still leak /home/hacker or /VibeCoding/"
fi
if grep -qE '^[0-9]\. .*/omg-vision' skills/new-project/SKILL.md && grep -qE '^[0-9]\. .*/omg-prd' skills/new-project/SKILL.md; then ok "new-project pipeline delegates to /omg-vision + /omg-prd"; else bad "new-project still calls bare /vision or /prd"; fi
# OmegaOS slash commands are /omg-* namespaced (no collision with other commands).
if ! grep -q '"\$OMG_CMD_DST/planner.md"' install.sh && ! grep -q '/planner.md"' install.sh; then ok "no bare /planner stub (uses /omg-planner — no collision)"; else bad "install.sh still writes a bare /planner stub (collides)"; fi
# Every OmegaOS command exposed as /omg-* (canonical) AND /omega-* (legacy alias — non-breaking).
if grep -q 'omg-${bn#omega-}' install.sh && grep -q 'omg-dynamic' install.sh; then ok "/omg-* aliases generated for all OmegaOS commands (legacy /omega-* kept)"; else bad "/omg-* alias loop missing from install.sh"; fi
# R-GRAPH parity: the graph doctrine is only real if all three halves ship — the
# compiled rule, the saved topologies, and the install step that copies them.
# A saved graph that a fresh clone does not get is a graph nobody can run (Law 0).
wf_count=$(ls .claude/workflows/*.js 2>/dev/null | wc -l)
if [ "$wf_count" -gt 0 ] && grep -q 'CLAUDE_WF_DST' install.sh; then ok "saved workflow graphs shipped ($wf_count in .claude/workflows/ → ~/.claude/workflows/)"; else bad "workflow graphs not shipped by install.sh (R-GRAPH step 9 broken on a fresh clone)"; fi
if [ -x scripts/check-workflows.sh ] && ./scripts/check-workflows.sh >/dev/null 2>&1; then ok "every saved workflow graph parses (check-workflows.sh)"; else bad "a saved workflow graph fails the syntax gate — run ./scripts/check-workflows.sh"; fi
if grep -q '"R-GRAPH"' crates/omega-core/src/rules.rs && grep -q 'pipeline()' .claude/commands/dynamic.md; then ok "R-GRAPH compiled + /dynamic carries the graph playbook"; else bad "R-GRAPH missing from the registry or /dynamic lacks the playbook"; fi
# R-PREFLIGHT parity: the preflight is doctrine ONLY if all three halves ship —
# the compiled rule, the prompt-time trigger that fires while the decision can
# still be made, and the finish-guard exemption that lets a zero-mutation
# preflight stop. A rule whose pause its own Stop hook refuses is worse than no
# rule at all: it trains the agent to ignore the guard (Law 0 + R-MONITOR).
if grep -q '"R-PREFLIGHT"' crates/omega-core/src/rules.rs \
   && grep -q 'R-PREFLIGHT' scripts/hooks/omega-prompt-scan.sh \
   && grep -q 'preflight' scripts/hooks/stop-verify-hook.sh \
   && grep -q 'is_preflight' scripts/hooks/omega_plan_state.py \
   && grep -q 'PREFLIGHT' scripts/hooks/omega-session-contract.sh; then
    ok "R-PREFLIGHT compiled + prompt-scan trigger + finish-guard exemption + session contract"
else
    bad "R-PREFLIGHT incomplete — the rule, the prompt-scan branch, the finish-guard exemption and the session contract must all ship together"
fi

# Oracle lifecycle contract parity (Law 0). Split deliberately into two halves,
# because they can be proven at different times.
#
# (a) STRUCTURAL, always enforced. The three doctrine asset classes must be
#     carried into ~/.omega by DIRECTORY or by GLOB, never by literal filename.
#     That property is the whole reason a rule, a docs page or an agent prompt
#     written today reaches a fresh install with no install.sh edit at all.
#     Naming assets one by one is the failure mode this guards: the copy block
#     keeps passing while the newest file is silently left behind.
LC_STRUCT_OK=1
grep -qF 'cp "$OMEGA_SRC/rules"/*.md "$OMEGA_DIR/rules/"' install.sh \
  || { bad "install.sh does not copy rules/*.md by glob (a NEW rule file would not reach a fresh install)"; LC_STRUCT_OK=0; }
grep -qF 'cp -r "$OMEGA_SRC/docs/"* "$OMEGA_DIR/docs/"' install.sh \
  || { bad "install.sh does not copy the docs/ tree wholesale (a NEW docs page would not reach a fresh install)"; LC_STRUCT_OK=0; }
grep -qF 'cp -r "$OMEGA_SRC/agents/"* "$AGENTS_DIR/"' install.sh \
  || { bad "install.sh does not copy the agents/ tree wholesale (a NEW agent prompt would not reach a fresh install)"; LC_STRUCT_OK=0; }
grep -qF '"$INSTALL_DIR/omega" rules export' install.sh \
  || { bad "install.sh never runs 'omega rules export' (code-registered rules would never reach ~/.omega/rules)"; LC_STRUCT_OK=0; }
[ "$LC_STRUCT_OK" = "1" ] && ok "doctrine assets carried generically (rules/*.md glob + docs/ tree + agents/ tree + omega rules export): a NEW rule/doc/agent ships with no install.sh edit"

# (b) CONTENT, honest about what it cannot see. Did the lifecycle assets
#     actually LAND in ~/.omega? The rule and its docs page may be written on a
#     concurrent branch, and the check must then SKIP and print why: a check
#     that cannot see its subject must never report a pass, because a vacuous
#     pass is exactly how a missing asset ships.
#     The rule half asserts ONE EXACT ID, R-ORACLE-LEDGER, at both ends. The id
#     is what `omega rules export` writes into the exported basename
#     (R-ORACLE-LEDGER-the-oracle-ledger-and-a-close-that-leave.md), so the same
#     literal is checkable in the source registry and in ~/.omega/rules. Two
#     ways this check was previously vacuous, both fixed here: a source pattern
#     that no registered id can match (the rule ships as R-ORACLE-LEDGER, never
#     as *LIFECYCLE*) makes the whole half unreachable and prints "rule files:
#     0" while passing, and a destination CONTENT grep for the word "lifecycle"
#     is satisfied by any unrelated rule that merely mentions it. Match the id,
#     and match it on the basename.
LC_HOME="${OMEGA_DIR:-$HOME/.omega}"
LC_RULE_ID="R-ORACLE-LEDGER"
LC_RULE_SRC=$(grep -cE "id:[[:space:]]*\"$LC_RULE_ID\"" crates/omega-core/src/rules.rs 2>/dev/null)
LC_DOC_SRC=$(find docs -iname '*lifecycle*.md' 2>/dev/null | wc -l)
if [ "$LC_RULE_SRC" -eq 0 ] && [ "$LC_DOC_SRC" -eq 0 ]; then
  ok "oracle lifecycle contract assets absent from this branch (0 $LC_RULE_ID registrations in rules.rs, 0 docs/*lifecycle*.md): landing check SKIPPED, nothing to verify yet"
elif [ "$VERIFY_SOURCE_ONLY" = "1" ]; then
  ok "installed lifecycle landing check skipped in VERIFY_SOURCE_ONLY mode"
elif [ ! -d "$LC_HOME" ]; then
  ok "$LC_HOME absent (OmegaOS never installed here): lifecycle landing check SKIPPED (sources present: $LC_RULE_SRC $LC_RULE_ID registration(s), $LC_DOC_SRC docs pages)"
else
  LC_LANDED=1
  LC_DOC_DST=$(find "$LC_HOME/docs" -iname '*lifecycle*.md' 2>/dev/null | wc -l)
  LC_RULE_DST=$(find "$LC_HOME/rules" -maxdepth 1 -type f \
                  \( -name "$LC_RULE_ID.md" -o -name "$LC_RULE_ID-*.md" \) 2>/dev/null | wc -l)
  if [ "$LC_DOC_SRC" -gt 0 ] && [ "$LC_DOC_DST" -eq 0 ]; then
    bad "lifecycle docs page(s) in the repo but none under $LC_HOME/docs (re-run install.sh, or the docs copy step regressed)"
    LC_LANDED=0
  fi
  if [ "$LC_RULE_SRC" -gt 0 ] && [ "$LC_RULE_DST" -eq 0 ]; then
    bad "$LC_RULE_ID registered in rules.rs but no $LC_RULE_ID*.md exported to $LC_HOME/rules (re-run 'omega rules export')"
    LC_LANDED=0
  fi
  [ "$LC_LANDED" = "1" ] && ok "oracle lifecycle contract landed in $LC_HOME (docs pages: $LC_DOC_DST, $LC_RULE_ID files: $LC_RULE_DST)"
fi
# Companion tools + skills (SST multi-LLM) shipped and sourced by install.sh.
# HONEST contract: install.sh DEFERS this pack by default (opt-in
# OMEGA_WITH_COMPANION=1) — the gate asserts the opt-in branch exists; it must
# never claim a default install provisions it.
if [ -f scripts/install-companion-tools.sh ] && grep -q "install-companion-tools.sh" install.sh && grep -q "OMEGA_WITH_COMPANION" install.sh; then ok "companion tools (planning-with-files/higgsfield/claude-mem/superpowers/mempalace/remotion) available OPT-IN (deferred by default — OMEGA_WITH_COMPANION=1)"; else bad "companion-tools installer not shipped/wired (opt-in branch) in install.sh"; fi
# Third-party skill collections (obra/superpowers + garrytan/gstack): shipped,
# OPT-IN (OMEGA_WITH_THIRD_PARTY=1). Skip still wins if both are set. Pins are
# full 40-hex SHAs (checked below).
if [ -f scripts/install-third-party-skills.sh ] && grep -q "install-third-party-skills.sh" install.sh && grep -q "OMEGA_WITH_THIRD_PARTY" install.sh && grep -q "OMEGA_SKIP_THIRD_PARTY" install.sh; then ok "third-party skill collections (superpowers + gstack) OPT-IN (OMEGA_WITH_THIRD_PARTY=1)"; else bad "third-party skills installer not shipped/wired (opt-in branch) in install.sh"; fi
if grep -qE 'SUPERPOWERS_PIN:-[0-9a-f]{40}' scripts/install-third-party-skills.sh && grep -qE 'GSTACK_PIN:-[0-9a-f]{40}' scripts/install-third-party-skills.sh; then ok "third-party pins are full 40-hex SHAs (reproducible)"; else bad "third-party pins are not full 40-hex SHAs (reproducibility broken)"; fi
if [ -f docs/third-party-skills.md ] && grep -q "third-party-skills" docs/third-party-skills.md; then ok "third-party skills doctrine shipped (docs/third-party-skills.md)"; else bad "docs/third-party-skills.md missing"; fi
# Browser engine for the Quality Arsenal audits (uiux/flow/a11y/perf, browser-tester)
# + CDP/DevTools automation. HONEST contract: install.sh defers the ~150MB
# Chromium + apt deps by default (opt-in OMEGA_WITH_BROWSER=1) — a default
# install has NO chromium, and this gate must say so instead of "provisioned".
if grep -q "playwright install chromium" install.sh && grep -q "OMEGA_WITH_BROWSER" install.sh && grep -q "OMEGA_SKIP_BROWSER" install.sh; then ok "Playwright + Chromium (CDP/DevTools) available OPT-IN (deferred by default — OMEGA_WITH_BROWSER=1)"; else bad "install.sh browser-stack opt-in branch missing — browser audits unreachable on a fresh box"; fi
# Nova personal-assistant layer vendored + wired as an OPT-IN (OMEGA_WITH_NOVA=1
# — needs its own bot token), with a secrets-stripped app-catalogue sample. The
# live ~/.omega/nova-secrets.env / nova-apps.json stay local-only state.
NOVA_MISS=""
for f in scripts/nova-report.sh scripts/nova-send.sh scripts/nova-godmode.sh scripts/nova-composio-connect.sh agents/companion.md config/nova-apps.sample.json; do
  [ -f "$f" ] || NOVA_MISS="$NOVA_MISS $f"
done
if [ -z "$NOVA_MISS" ] && grep -q "OMEGA_WITH_NOVA" install.sh && grep -q "OMEGA-CRON-NOVA-GODMODE-v1" install.sh; then
  ok "Nova layer shipped + wired OPT-IN (4 scripts + companion.md + apps sample + crons — OMEGA_WITH_NOVA=1)"
else
  bad "Nova layer not shipped/wired in install.sh (missing:${NOVA_MISS:- install.sh block})"
fi
# Nova vendored files must not embed operator-specific ids (a hardcoded chat-id
# default or the maintainer's name) — they read ~/.omega/nova-secrets.env at
# runtime. (Token patterns are already covered by the global secret gate.)
if grep -qE 'NOVA_CHAT_ID:-[0-9]|Gareth|nova-face-3angles' scripts/nova-report.sh scripts/nova-send.sh scripts/nova-godmode.sh scripts/nova-composio-connect.sh scripts/nova-self-improve.sh scripts/nova-notify.sh agents/companion.md config/nova-apps.sample.json config/nova-anatomy.template.md 2>/dev/null; then
  bad "Nova vendored files embed an operator id/name/Soul (must come from nova-secrets.env)"
else
  ok "Nova vendored files clean (operator ids/secrets read from nova-secrets.env at runtime)"
fi
# Agentik-Skills mirror step: cloning the SSOT library is not enough — install.sh
# must mirror its <skill>/SKILL.md dirs into ~/.omega/skills/ or a fresh box
# silently lacks the Motion/design families that live only there.
if grep -q "Agentik-Skills mirrored" install.sh; then ok "Agentik-Skills → ~/.omega/skills mirror step present in install.sh"; else bad "Agentik-Skills mirror step missing from install.sh (cloned skills never reach ~/.omega/skills)"; fi
if grep -q "post-mirror skill catalog reconciliation" install.sh && grep -q "post-mirror provider sync" install.sh; then
  ok "skill Atlas/RAG and provider links are reconciled after the private library mirror"
else
  bad "post-mirror catalog/provider reconciliation missing (late skills stay stale)"
fi
if [ "$(grep -A28 "Copy agents from repo if available" crates/omega-cli/src/main.rs | grep -c 'else if entry.file_name().to_string_lossy().ends_with(".md")')" -eq 1 ] \
  && grep -A28 "Copy agents from repo if available" crates/omega-cli/src/main.rs | grep -q 'std::fs::copy(entry.path(), &dst)'; then
  ok "omega sync copies top-level agent markdown instead of silently skipping it"
else
  bad "omega sync does not reliably copy top-level agent markdown"
fi
if grep -q "Native skills synced to" crates/omega-cli/src/main.rs \
  && grep -q "externally installed skills" crates/omega-cli/src/main.rs; then
  ok "omega sync reconciles native skills without deleting external additions"
else
  bad "omega sync does not reconcile the native skill tree safely"
fi
if grep -q 'home.join(".agents").join("skills")' crates/omega-cli/src/main.rs \
  && grep -q 'CODEX_SESSIONSTART_SKILLS' crates/omega-cli/src/main.rs \
  && grep -q 'agentic-engineering-lab' crates/omega-cli/src/main.rs; then
  ok "Codex SessionStart skills are allowlisted (no 90-skill dump)"
else
  bad "Codex SessionStart skill allowlist missing"
fi
# Quality Arsenal audit SKILLS shipped + wired (the registry is 23 audits; the
# skill dirs must match so a fresh install can actually run them — excludes the
# 3 non-audit dirs _shared / audit-orchestrator / audit-tracker).
N_AUDITS=$(find skills/audits -mindepth 1 -maxdepth 1 -type d \
  ! -name _shared ! -name audit-orchestrator ! -name audit-tracker 2>/dev/null | wc -l)
if [ "$N_AUDITS" -ge 23 ] && grep -q "skills/audits" install.sh; then ok "Quality Arsenal audit skills shipped + wired ($N_AUDITS)"; else bad "audit skills missing or not wired in install.sh ($N_AUDITS dirs, need >=23)"; fi
# Design skills (generative UI/UX) shipped + wired.
N_DESIGN=$(ls -d skills/design/*/ 2>/dev/null | wc -l)
if [ "$N_DESIGN" -ge 8 ] && grep -q "skills/design" install.sh; then ok "Design skills shipped + wired ($N_DESIGN)"; else bad "design skills missing or not wired in install.sh ($N_DESIGN dirs, need >=8)"; fi
# design-intelligence pack (vendored UX-process / interaction / AI-product /
# prompt-arch / alignment / agent-orchestration / eval skills) shipped + wired,
# routed by the R-DESIGN doctrine rule.
N_DI=$(ls -d skills/design-intelligence/*/ 2>/dev/null | wc -l)
if [ "$N_DI" -ge 120 ] && grep -qE 'DI_SRC=.*skills/design-intelligence' install.sh; then ok "design-intelligence pack shipped + wired ($N_DI skills)"; else bad "design-intelligence pack missing or not wired in install.sh ($N_DI dirs, need >=120)"; fi
if grep -q '"R-DESIGN"' crates/omega-core/src/rules.rs; then ok "R-DESIGN router rule present in the registry (rules.rs SSOT)"; else bad "R-DESIGN router rule missing from rules.rs"; fi
# Design skills must not leak the maintainer's identity/paths.
if grep -rqE 'Gareth|/home/hacker' skills/design/ 2>/dev/null; then bad "design skills leak Gareth/home path"; else ok "design skills clean (no Gareth/home leak)"; fi
# PDF generator shipped + wired (all branded PDF output depends on it).
if [ -d tools/pdfgen ] && [ -f skills/pdfgen/SKILL.md ] && grep -q "tools/pdfgen" install.sh; then
  ok "pdfgen engine + discoverable skill contract shipped and wired in install.sh"
else
  bad "pdfgen engine or discoverable skill contract missing"
fi
if grep -q "OMEGA_TELEGRAM_CHAT_ID is required" tools/pdfgen/bin/pdfgen.ts \
  && ! grep -qF 'execFileSync(telegramSh, ["file", "' tools/pdfgen/bin/pdfgen.ts; then
  ok "raw pdfgen Telegram delivery requires an explicit allow-listed destination"
else
  bad "raw pdfgen Telegram delivery contains an implicit or hardcoded destination"
fi
# OAuth fallback helper shipped + wired (non-interactive token refresh).
if [ -f docs/reference/oauth/claude-oauth.sh ] && grep -q "claude-oauth.sh" install.sh; then ok "OAuth helper shipped + wired in install.sh"; else bad "oauth helper not shipped/wired in install.sh"; fi
# Reference docs tree shipped + wired (`omega docs` + $HOME Claude sessions read it).
if [ -d docs ] && [ -f docs/ARCHITECTURE.md ] && grep -q "OMEGA_SRC/docs" install.sh; then ok "reference docs shipped + wired in install.sh"; else bad "docs tree not shipped/wired in install.sh"; fi

# Marketing Machine (R-MARKETING) — the WHOLE payload must ship, not just the
# CLIs: `omega marketing capabilities/status/next` reads capabilities.toml, and
# a fresh box has no checkout to fall back on. Credentials are deliberately NOT
# part of the payload (keys stay in ~/.omega/secrets/integrations.env).
# projects.tsv and AUDIT.md are operator-specific marketing data, gitignored on
# purpose ("never ship to the public OmegaOS repo") — requiring them here made
# the parity gate unpassable from a clean clone. install.sh copies them only
# when present, so they are checked as OPTIONAL, never as shipped payload.
mm_missing=""
for f in capabilities.toml scaffold.sh README.md growth/GROWTH-KIT.md growth/OUTLIER-ENGINE.md; do
  [ -f "tools/marketing-machine/$f" ] || mm_missing="$mm_missing $f"
done
if [ -n "$mm_missing" ]; then bad "marketing-machine payload incomplete:$mm_missing"; else ok "marketing-machine payload complete (registry + scaffolder + growth kit)"; fi
# The operator-private files must stay gitignored AND stay optional in install.sh.
if grep -qF '/tools/marketing-machine/projects.tsv' .gitignore \
  && grep -qF '/tools/marketing-machine/AUDIT.md' .gitignore \
  && grep -q '\[\[ -f "\$MM_SRC/\$_mm_f" \]\] && cp -f' install.sh; then
  ok "marketing-machine operator data gitignored + copied only when present"
else bad "marketing-machine operator data (projects.tsv/AUDIT.md) not gitignored or copied unconditionally"; fi
if grep -q 'MM_DST="\$OMEGA_DIR/marketing-machine"' install.sh && grep -q 'cp -f "\$MM_SRC/\$_mm_f"' install.sh; then
  ok "marketing-machine installed to ~/.omega by install.sh (checkout-independent)"
else bad "marketing-machine payload not copied into ~/.omega by install.sh"; fi
if grep -q 'marketing-machine' crates/omega-core/src/marketing.rs && grep -q '\.join("marketing-machine")' crates/omega-core/src/marketing.rs; then
  ok "capabilities registry resolves the installed ~/.omega copy (no checkout needed)"
else bad "capabilities_toml_path has no ~/.omega fallback — breaks on a checkout-less box"; fi
if grep -rqE 'API_KEY *= *["'"'"'][A-Za-z0-9_-]{16,}' tools/marketing-machine/ 2>/dev/null; then
  bad "marketing-machine payload contains a hardcoded credential"
else ok "marketing-machine payload carries NO credentials (keys stay in secrets/)"; fi

# Prebuilt-release freshness gate: a fresh clone must NEVER be handed a release
# artifact older than the source it was cloned from (2026-07-23: v0.1.8 from
# 12 June was installed over a main carrying the July menu — same version
# string, so nothing looked wrong).
if grep -q 'published_at' install.sh && grep -q 'is newer than release' install.sh; then
  ok "prebuilt freshness gate present (never installs a release older than the source)"
else bad "install.sh can still install a stale prebuilt over newer source"; fi
# The source checkout must exist before the prebuilt selector runs, otherwise
# the freshness/provenance checks are vacuous on the documented curl|bash path.
source_authority_line=$(grep -n -m1 '^OMEGA_SOURCE_REV=' install.sh | cut -d: -f1)
prebuilt_call_line=$(grep -n -m1 '^maybe_install_prebuilt$' install.sh | cut -d: -f1)
if [ -n "$source_authority_line" ] && [ -n "$prebuilt_call_line" ] \
  && [ "$source_authority_line" -lt "$prebuilt_call_line" ]; then
  ok "prebuilt selection is bound to a source checkout resolved first"
else
  bad "prebuilt selection runs before the source authority is resolved"
fi
if grep -qF 'prebuilt_build_info_matches_source "$tmp/BUILD-INFO.json" "$triple"' install.sh \
  && grep -qF 'and (.omega_commit == $omega_commit)' install.sh \
  && grep -qF 'and (.cargo_lock_sha256 == $cargo_lock_sha256)' install.sh \
  && grep -qF 'and (.target == $target)' install.sh \
  && grep -qF 'startswith($rmux_rev)' install.sh; then
  ok "prebuilt BUILD-INFO is bound to exact source/rmux/lock/target provenance"
else
  bad "prebuilt BUILD-INFO is present but not bound to the exact source authority"
fi
if grep -qF 'status --porcelain --untracked-files=no' install.sh \
  && grep -qF 'Tracked source changes present' install.sh; then
  ok "tracked local source changes force the source-build fallback"
else
  bad "a prebuilt can still replace tracked local source changes"
fi
if grep -qF 'chmod 700 "$OMEGA_DIR/state"' install.sh \
  && grep -qF '.installed-build-info.json.new' install.sh \
  && grep -qF 'mv -f "$OMEGA_DIR/state/.installed-build-info.json.new"' install.sh; then
  ok "installed provenance is published atomically inside an owner-only state directory"
else
  bad "installed provenance publication is non-atomic or state permissions are too broad"
fi
# Telegram liveness watchdog must cover the AGENT bots, not just the master: they
# run the same poll-loop core, so watching omega-tg-bot.service alone left every
# project bot able to go "alive but deaf" with nothing to recover it.
if [ -f scripts/omega-atlas-liveness.sh ] && grep -q 'agent-bots.json' scripts/omega-atlas-liveness.sh \
   && grep -q 'omega-tg-agent-%s.service' scripts/omega-atlas-liveness.sh; then
  ok "telegram liveness watchdog covers master + agent bots"
else bad "liveness watchdog watches the master only — agent bots can go deaf unrecovered"; fi
# Binary replacement must survive a RUNNING omega (patrol/bot/TUI hold the inode).
if grep -q 'install_binary()' install.sh && ! grep -qE '^\s*cp target/release/(omega|rmux) ' install.sh; then
  ok "binaries installed atomically (survives Text file busy on update)"
else bad "install.sh still cp's over a possibly-running binary (ETXTBSY aborts the install)"; fi

# 10b. No SKILL.md leaks a maintainer-private ~/.claude or /home/hacker path that
#      install.sh does NOT create. OmegaOS ships on a blank VPS: an audit skill that
#      shells out to `~/.claude/lib/hinge-analyzer.sh` or reads `~/.claude/DEPRECATED.md`
#      silently breaks on a fresh clone. install.sh only provisions ~/.claude/commands/,
#      ~/.claude/workflows/, ~/.claude/settings.json, and ~/.claude/.credentials.json — anything
#      else under ~/.claude/ (lib/data/agents/resources/rules/projects/…) or under /home/hacker/
#      is a leak. Documented blank-VPS warnings ("never reference ~/.claude/…") are prose, not leaks.
leaks=$(grep -rhnE '(~/\.claude/|/home/hacker/)[A-Za-z0-9_./-]+' skills/audits/*/SKILL.md skills/audits/_shared/* 2>/dev/null \
  | grep -vE '[~]/[.]claude/(commands/|workflows/|settings[.]json|[.]credentials[.]json)' \
  | grep -vE '[~]/[.]claude/[.][.][.]' \
  | grep -vE '(blank-VPS|never reference|never reach|forbids|ships them|shipped INSIDE|vendored next)')
if [ -z "$leaks" ]; then
  ok "no SKILL.md references an unshipped ~/.claude or /home/hacker path"
else
  bad "SKILL.md files reference ~/.claude/home paths install.sh does NOT create (blank-VPS clone breaks):"
  printf '%s\n' "$leaks" | sed 's/^/      /'
fi

# 10c. Quality Arsenal RUNTIME parity. The audit SKILLs hard-code the hybrid
#      orchestrator by absolute path under the SINGLE home: ~/.omega/lib/audit-runner.sh
#      + audit-gather/ + safe-npm-build.sh + ~/.omega/bin/audit-notify.sh. This gate
#      proves both halves: (a) the runtime is VENDORED in _shared/, and (b) install.sh
#      SHIPS it to ~/.omega. Either half missing = audits break on a fresh clone (Law 0).
#      Also asserts the dual ~/.aisb home is gone (consolidated into ~/.omega).
miss=""
[ -f skills/audits/_shared/audit-runner.sh ]   || miss="$miss audit-runner.sh"
[ -d skills/audits/_shared/audit-gather ]      || miss="$miss audit-gather/"
[ -f skills/audits/_shared/audit-notify.sh ]   || miss="$miss audit-notify.sh"
[ -f skills/audits/_shared/safe-npm-build.sh ] || miss="$miss safe-npm-build.sh"
if [ -n "$miss" ]; then bad "audit runtime not vendored in _shared:$miss"; else ok "audit runtime vendored in _shared (runner + gather + notify + safe-npm-build)"; fi
if grep -q 'OMEGA_DIR/lib/audit-runner\.sh' install.sh && grep -q 'OMEGA_DIR/bin/audit-notify\.sh' install.sh; then
  ok "install.sh ships the ~/.omega audit runtime (skills' absolute paths resolve)"
else
  bad "install.sh does NOT install the ~/.omega audit runtime — audit SKILLs break on fresh clone"
fi
if [ -f skills/audits/registry.toml ] \
  && grep -q 'skills/audits/registry.toml' crates/omega-core/src/audit.rs \
  && grep -q -- '--finalize' skills/audits/_shared/audit-runner.sh; then
  ok "audit registry is canonical and the runner has a fail-closed finalization gate"
else
  bad "audit registry/runner contract drifted (TOML SSOT or --finalize gate missing)"
fi
if scripts/tests/test_audit_runner.sh >/dev/null 2>&1; then
  ok "audit runner rejects incomplete/failed evidence and enforces the score threshold"
else
  bad "audit runner behavioral contract failed"
fi
missing_hinge=""
for audit_skill in skills/audits/*/SKILL.md; do
  if grep -q 'audit-runner\.sh' "$audit_skill" && ! grep -q -- '--hinge=' "$audit_skill"; then
    missing_hinge="$missing_hinge $audit_skill"
  fi
done
if [ -z "$missing_hinge" ]; then
  ok "every hybrid audit passes the mandatory hinge contract to the runner"
else
  bad "hybrid audit runner invocation missing --hinge:$missing_hinge"
fi
# 10c-bis. No skill or agent may reference the retired ~/.aisb dual-home (consolidated → ~/.omega).
if grep -rqE '/\.aisb/' skills/ agents/ 2>/dev/null; then
  bad "a skill/agent still references the retired ~/.aisb home (should be ~/.omega):"; grep -rlE '/\.aisb/' skills/ agents/ | sed 's/^/      /'
else
  ok "no skill/agent references the retired ~/.aisb home (single ~/.omega home)"
fi

# ── Behavioral gates (runtime truth, not text-greps) ─────────────────────────
# The checks above grep install.sh for the right STRINGS; these prove the system
# actually builds and the shipped config actually parses. Skip with VERIFY_FAST=1
# (the fast curator path); CI and the L0 gate run the real thing.
if [ "${VERIFY_FAST:-0}" != "1" ] && command -v cargo >/dev/null 2>&1; then
  if cargo check --workspace --locked >/dev/null 2>&1; then
    ok "workspace compiles against the committed Cargo.lock (reproducible)"
  else
    bad "cargo check --locked failed — lockfile out of sync or code broken"
  fi
  if cargo test -p omega-core --lib shipped_default_toml_deserializes >/dev/null 2>&1; then
    ok "shipped config/default.toml deserializes into OmegaConfig"
  else
    bad "config/default.toml does not deserialize — fresh install would discard it"
  fi
else
  ok "behavioral build/deserialize gates skipped (VERIFY_FAST or no cargo)"
fi

# 10c. Anti-abandon hook payload — every hook file exists AND install.sh copies
# its extension AND registers it. install.sh used to copy only *.sh, which
# silently dropped the shared parser both finish hooks import; they fail open on
# import error, so the enforcement vanished with nothing to see. Assert the whole
# chain so that class of bug is caught here instead of in production.
HOOK_PARITY_OK=1
for f in stop-verify-hook.sh omega-session-contract.sh omega-prompt-scan.sh omega-plan-mirror.sh omega-audit-guard.sh track-tool-use.sh omega_plan_state.py; do
  [ -f "scripts/hooks/$f" ] || { bad "hook payload missing from repo: scripts/hooks/$f"; HOOK_PARITY_OK=0; }
done
grep -q 'scripts/hooks/"\*\.py' install.sh || { bad "install.sh does not copy scripts/hooks/*.py (shared parser would not ship)"; HOOK_PARITY_OK=0; }
for marker in stop-verify-hook omega-session-contract omega-prompt-scan omega-plan-mirror; do
  grep -q "$marker" install.sh || { bad "install.sh never registers $marker"; HOOK_PARITY_OK=0; }
done
[ "$HOOK_PARITY_OK" = "1" ] && ok "anti-abandon hooks shipped, copied (*.sh + *.py) and registered by install.sh"
if [ "$(grep -c '\.hooks.PreToolUse' install.sh)" -ge 2 ] \
  && [ "$(grep -c '\.hooks.PostToolUse' install.sh)" -ge 2 ] \
  && grep -q -- '--dangerously-bypass-hook-trust' crates/omega-core/src/agents.rs; then
  ok "Claude and Codex both receive audit/tracker hooks; detached Codex trust is explicit"
else
  bad "Codex hook enforcement is not in parity with Claude"
fi

# 10b. Post-update coherence: an install must reconcile itself, and the
# staleness check must have something true to compare against.
#
# The defect this pins: `install.sh` used to install a binary and record
# NOTHING about which commit it was, so `auto-update.json` kept naming whatever
# the nightly cron last installed. The staleness check compares HEAD to that
# field, so a stale record reports a stale binary as CURRENT — measured on the
# source box 2026-08-05, five days and thirty commits adrift while `omega
# doctor` printed "all systems healthy". Both halves have to stay wired or the
# drift becomes invisible again.
RECONCILE_OK=1
grep -q 'omega" reconcile' install.sh || { bad "install.sh never runs 'omega reconcile' — an install would leave the doctrine, the installed-commit record and finished workers untouched"; RECONCILE_OK=0; }
grep -q 'fn cmd_reconcile' crates/omega-cli/src/main.rs || { bad "omega reconcile is gone from the CLI"; RECONCILE_OK=0; }
grep -q 'record_installed' crates/omega-cli/src/main.rs || { bad "omega update --record-installed is gone — installs would stop recording their provenance"; RECONCILE_OK=0; }
grep -q 'fn binary_provenance' crates/omega-core/src/doctor.rs || { bad "doctor lost its binary-provenance check — a stale binary would read as healthy again"; RECONCILE_OK=0; }
grep -q 'installed_head_mismatch' crates/omega-core/src/auto_update.rs || { bad "auto-update no longer owes an install when HEAD moved past the installed commit — a machine that authors commits would never rebuild"; RECONCILE_OK=0; }
grep -q 'spawn_reconcile_session' crates/omega-cli/src/main.rs || { bad "a successful auto-update no longer launches the reconcile session"; RECONCILE_OK=0; }
grep -q 'binary_stale' crates/omega-cli/src/main.rs || { bad "the interactive 'omega update' no longer notices a stale binary — only the cron would, and the two must agree"; RECONCILE_OK=0; }
grep -q 'fn doctrine_fingerprint' crates/omega-core/src/rules.rs || { bad "the doctrine fingerprint is gone — the session check would fall back to crying wolf on every doctrine-neutral rebuild"; RECONCILE_OK=0; }
grep -q 'doctrine_changed_at' crates/omega-cli/src/main.rs || { bad "reconcile no longer measures sessions against the last DOCTRINE change"; RECONCILE_OK=0; }
grep -q 'fn cmd_graph_run' crates/omega-cli/src/main.rs || { bad "omega graph run is gone — graph_executor::advance would have no driver again, and an untouched executor is an untested one"; RECONCILE_OK=0; }
grep -q 'evaluate_gate' crates/omega-cli/src/main.rs || { bad "the graph driver no longer consults the risk gate before dispatch (R-DESTRUCT)"; RECONCILE_OK=0; }
[ "$RECONCILE_OK" = "1" ] && ok "post-update reconcile wired (install.sh + cron), provenance recorded and checked"

# 11. New self-healing assets shipped + wired (token-refresh, shared-credential, scrollback alias).
if [ -f scripts/omega-token-refresh.sh ] && grep -q "omega-token-refresh.sh" install.sh; then ok "token-refresh helper shipped + wired in install.sh"; else bad "token-refresh helper not shipped/wired in install.sh"; fi
if grep -q "OMEGA-CRON-TOKEN-REFRESH-v1" install.sh; then ok "token-refresh cron scheduled by install.sh"; else bad "token-refresh cron not in install.sh"; fi
if [ -f scripts/omega-self-heal.sh ] && grep -q "omega-self-heal.sh" install.sh && grep -q "OMEGA-CRON-SELFHEAL-v1" install.sh; then ok "self-heal daemon shipped + wired + cron scheduled"; else bad "self-heal daemon (omega-self-heal.sh) not shipped/wired in install.sh"; fi
if grep -q "OMEGA_CREDENTIALS_LINK" install.sh; then ok "shared-credential override (OMEGA_CREDENTIALS_LINK) wired in install.sh"; else bad "shared-credential override not in install.sh"; fi
if grep -q "alias om=" install.sh && grep -q "CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN" install.sh; then ok "shell integration (om alias + scrollback) wired in install.sh"; else bad "shell integration not in install.sh"; fi
if [ -f docs/GETTING-STARTED.md ] && grep -q "GETTING-STARTED.md" install.sh; then ok "getting-started guide shipped + installed to ~/.omega"; else bad "getting-started guide (docs/GETTING-STARTED.md) not shipped/wired in install.sh"; fi
if [ -f GUIDE.md ] && grep -q 'OMEGA_SRC/GUIDE.md' install.sh; then ok "GUIDE.md operator manual shipped + installed to ~/.omega"; else bad "GUIDE.md not shipped/wired in install.sh"; fi
if grep -q "Commands::Guide" crates/omega-cli/src/main.rs && grep -q "GETTING-STARTED.md" crates/omega-cli/src/main.rs; then ok "omega guide command wired (embedded fallback)"; else bad "omega guide command missing from CLI"; fi
if grep -q "agent-bot resurrect" telegram-bot/omega-tg-bot.ts; then ok "agent-bot units resurrected at bot startup (reinstall-safe)"; else bad "agent-bot resurrection loop missing from bot startup"; fi
# Daily update check + auto-apply: the cron, the CLI path it calls, and the
# opt-out must ship together — a cron pointing at a missing flag updates nobody.
if grep -q "OMEGA-CRON-AUTO-UPDATE-v1" install.sh && grep -q "cmd_update_auto" crates/omega-cli/src/main.rs; then ok "daily update check + auto-apply scheduled by install.sh"; else bad "daily auto-update cron/CLI path not wired"; fi
if grep -q "auto_update" crates/omega-core/src/config.rs && grep -q "auto_update" crates/omega-cli/src/main.rs; then ok "auto-update opt-out wired (omega config set auto_update off|check|apply)"; else bad "auto_update config switch missing"; fi
# Mission patterns: the shape block must reach BOTH the oracle prompt and the
# worker brief. Wired in one place only, an oracle would dispatch workers that
# do not know when to stop.
if grep -q "mission_patterns::orchestration_block" crates/omega-core/src/oracle_lifecycle.rs && grep -q "mission_patterns::orchestration_block" crates/omega-cli/src/main.rs; then ok "mission-pattern orchestration injected into oracle prompts + worker briefs"; else bad "mission patterns not wired into oracle/worker prompts"; fi
# The registry must never be writable from a poisoned load — this is the guard
# that stopped the suite erasing a user's project list.
if grep -q "poisoned" crates/omega-core/src/project_manager.rs; then ok "project registry protected against overwrite-after-failed-load"; else bad "projects.json data-loss guard missing"; fi
# Agent Reach: installer shipped AND actually invoked by install.sh (a shipped
# script nobody calls is the install-parity failure this gate exists to catch).
if [[ -f tools/agent-reach/install-agent-reach.sh ]] && grep -q "install-agent-reach.sh" install.sh; then ok "Agent Reach shipped + installed by install.sh (internet reach for every agent)"; else bad "Agent Reach not shipped/wired in install.sh"; fi
# The pin must be a real commit, not a branch — an unpinned external repo is an
# unreviewed one on every future install.
if grep -qE '^PIN="[0-9a-f]{40}"' tools/agent-reach/install-agent-reach.sh; then ok "Agent Reach pinned to a reviewed commit"; else bad "Agent Reach pin missing or not a full sha"; fi
if grep -q 'skip_metadata' tools/agent-reach/install-agent-reach.sh \
  && grep -q 'skills validate --root "$SKILL_DST"' tools/agent-reach/install-agent-reach.sh; then
  ok "Agent Reach frontmatter is normalized and schema-validated before provider sync"
else
  bad "Agent Reach can publish upstream-only metadata that breaks omega sync"
fi
if grep -q 'mirror_owned_dir "$sk_dir"' install.sh \
  && grep -q 'rsync -a --delete "$src/" "$dst/"' install.sh \
  && grep -q 'rsync -a --delete --exclude=node_modules' install.sh \
  && grep -q 'skills validate --root "$sk_dir"' install.sh \
  && grep -q 'SKMIRROR_REJECTED' install.sh; then
  ok "skill mirrors prune stale files and reject invalid external protocols before publication"
else
  bad "skill mirrors can retain stale or schema-invalid protocols across reinstall"
fi

# omega-gateway (app API daemon): unit shipped, source-build wiring, AND the
# deterministic prebuilt archive. Grepping install.sh alone used to report a
# false pass while every prebuilt install silently omitted the daemon.
if [ -f config/omega-gateway.service ] && grep -q "crates/omega-gateway" Cargo.toml \
  && grep -q "omega-gatewayd" install.sh && grep -q "omega-gateway.service" install.sh \
  && grep -q "release/omega-gatewayd" .github/workflows/release.yml \
  && grep -q "'omega-gatewayd'" .github/workflows/release.yml \
  && grep -q '\$tmp/omega-gatewayd' install.sh; then
  ok "omega-gateway daemon shipped in source + prebuilt lanes"
else
  bad "omega-gateway daemon missing from source/prebuilt/install wiring"
fi

# The configured runtime default and fresh-install payload must agree.
if grep -q 'default_agent_command().*codex\|agent_command = "codex"' config/default.toml \
  && grep -q 'omega" install codex' install.sh; then
  ok "fresh install provisions the default Codex runtime"
else
  bad "Codex is the default but install.sh does not provision it"
fi

# Provider template is non-secret, installed owner-only, and visible on a fresh
# box instead of materializing only after the first UI mutation.
if [ -f config/providers.default.toml ] \
  && grep -q 'config/providers.default.toml' install.sh \
  && grep -q 'chmod 600 "\$OMEGA_DIR/providers.toml"' install.sh; then
  ok "providers.toml template shipped + installed owner-only"
else
  bad "provider config template missing or not installed owner-only"
fi

# Reproducible Rust version: CI, rust-toolchain.toml, and source installer must
# name the same compiler floor.
if grep -q 'channel = "1.97.1"' rust-toolchain.toml \
  && grep -q 'rust_required="1.97.1"' install.sh \
  && grep -q 'toolchain: 1.97.1' .github/workflows/ci.yml; then
  ok "Rust toolchain pinned consistently (1.97.1)"
else
  bad "Rust toolchain pin drift between CI, installer, and rust-toolchain.toml"
fi

# ── OS suite parity: every INTEGRATED OS (a payload beyond its scaffold) must
# have EACH of its four surfaces wired into install.sh, else a fresh install
# silently ships a half-installed OS. An integrated OS = OS/<slug>/ with a
# pack/ or engine/ dir. Surfaces checked: the OS payload block, the bin-wrapper
# loop, the codex-prompt loop (all generic), plus the per-OS Claude skill
# (BSK loop, or a dedicated block for stepper-os/books-os). The skill name is
# the slug, except researcher-os→market-research-os and books-os→alexandria.
if [ -d OS ]; then
  os_parity_ok=1
  # generic surface blocks
  grep -q 'OSS_SRC="\$OMEGA_SRC/OS"' install.sh && grep -q 'cp -rf "\$OSS_SRC/\." "\$OSS_DST/"' install.sh || { bad "OS suite: payload block (OS/. → ~/.omega/os) missing in install.sh"; os_parity_ok=0; }
  grep -q 'for _os_bin in "\$OSS_SRC"/\*/bin/\*' install.sh || { bad "OS suite: bin-wrapper loop missing in install.sh"; os_parity_ok=0; }
  grep -q 'for _os_cx in "\$OSS_SRC"/\*/commands/codex-\*.md' install.sh || { bad "OS suite: codex-prompt loop missing in install.sh"; os_parity_ok=0; }
  bsk_line="$(grep -oE 'for BSK in [^;]+; do' install.sh | head -1)"
  for _os in OS/*/; do
    slug="$(basename "$_os")"
    # integrated = carries a real payload beyond README/MASTER/ledger
    { [ -d "$_os/pack" ] || [ -d "$_os/engine" ]; } || continue
    # bin wrapper + codex prompt present in the repo (so the generic loops ship them)
    ls "$_os"bin/* >/dev/null 2>&1 || { bad "OS suite: $slug has no bin/ wrapper"; os_parity_ok=0; }
    ls "$_os"commands/codex-*.md >/dev/null 2>&1 || { bad "OS suite: $slug has no commands/codex-*.md"; os_parity_ok=0; }
    # the Claude skill install path
    case "$slug" in
      stepper-os) grep -q 'STEPPER_SKILL_DST' install.sh || { bad "OS suite: stepper-os skill block missing in install.sh"; os_parity_ok=0; } ;;
      books-os)   grep -q 'skills/alexandria' install.sh || { bad "OS suite: books-os (alexandria) skill block missing"; os_parity_ok=0; } ;;
      researcher-os) case "$bsk_line" in *market-research-os*) : ;; *) bad "OS suite: researcher-os skill (market-research-os) not in BSK loop"; os_parity_ok=0 ;; esac ;;
      ideation-os)   case "$bsk_line" in *brainstorm-os*) : ;; *) bad "OS suite: ideation-os skill (brainstorm-os) not in BSK loop"; os_parity_ok=0 ;; esac ;;
      habits-os)     case "$bsk_line" in *habit-tracker-os*) : ;; *) bad "OS suite: habits-os skill (habit-tracker-os) not in BSK loop"; os_parity_ok=0 ;; esac ;;
      designer-os)   case "$bsk_line" in *design-os*) : ;; *) bad "OS suite: designer-os skill (design-os) not in BSK loop"; os_parity_ok=0 ;; esac ;;
      storytelling-os) case "$bsk_line" in *storyteller-os*) : ;; *) bad "OS suite: storytelling-os skill (storyteller-os) not in BSK loop"; os_parity_ok=0 ;; esac ;;
      *) case "$bsk_line" in *"$slug"*) : ;; *) bad "OS suite: $slug skill not in the BSK install loop"; os_parity_ok=0 ;; esac ;;
    esac
  done
  [ "$os_parity_ok" -eq 1 ] && ok "OS suite parity: every integrated OS has payload + bin + codex + skill wired in install.sh"
fi

# Every product in the TUI registry must have a real root command on both
# provider surfaces. Exercise the command generator against an isolated HOME;
# a text marker alone cannot prove that the expected files are emitted.
if [ -x scripts/install-os-commands.sh ] \
  && grep -q 'scripts/install-os-commands.sh' install.sh; then
  os_command_test_home="$(mktemp -d)"
  os_command_catalog="$(scripts/install-os-commands.sh --list)"
  os_command_products="$(printf '%s\n' "$os_command_catalog" | sed '/^$/d' | wc -l)"
  if [ "$os_command_products" -ne 24 ]; then
    bad "OS commands: expected 24 products, got $os_command_products"
    os_commands_ok=0
  else
    os_commands_ok=1
  fi
  while IFS='|' read -r os_slug skill_slug aliases; do
    [ -n "$os_slug" ] || continue
    mkdir -p "$os_command_test_home/.omega/skills/$skill_slug"
    printf '%s\n' '---' "name: $skill_slug" 'description: install parity fixture' '---' \
      > "$os_command_test_home/.omega/skills/$skill_slug/SKILL.md"
  done <<EOF
$os_command_catalog
EOF
  if HOME="$os_command_test_home" OMEGA_DIR="$os_command_test_home/.omega" \
      scripts/install-os-commands.sh >/dev/null; then
    while IFS='|' read -r os_slug skill_slug aliases; do
      [ -n "$os_slug" ] || continue
      IFS=',' read -r -a alias_list <<< "$aliases"
      for command_name in "${alias_list[@]}"; do
        for exposed_name in "$command_name" "omg-$command_name"; do
          [ -f "$os_command_test_home/.claude/commands/$exposed_name.md" ] \
            || { bad "OS commands: Claude /$exposed_name missing for $os_slug"; os_commands_ok=0; }
          [ -f "$os_command_test_home/.codex/prompts/$exposed_name.md" ] \
            || { bad "OS commands: Codex /$exposed_name missing for $os_slug"; os_commands_ok=0; }
        done
      done
    done <<EOF
$os_command_catalog
EOF
  else
    bad "OS command generator failed in isolated HOME"
    os_commands_ok=0
  fi
  [ "$os_commands_ok" -eq 1 ] \
    && ok "OS root commands: 24 products and every alias generated for Claude + Codex"
else
  bad "OS root command installer missing or not invoked by install.sh"
fi

if command -v bun >/dev/null 2>&1 \
  && bun scripts/verify-os-suite.ts >/dev/null; then
  ok "OS documentation parity: 24 complete manifests and checksum inventories"
elif [ "$VERIFY_SOURCE_ONLY" = "1" ] && ! command -v bun >/dev/null 2>&1; then
  ok "OS documentation runtime verifier skipped in VERIFY_SOURCE_ONLY mode (Bun unavailable)"
else
  bad "OS documentation parity failed (README/README_FR/MANIFEST/integration/skill/checksums)"
fi

echo "═══════════════════════════════════"
if [ "$fail" -eq 0 ]; then
  printf '\033[32mINSTALL PARITY OK — a fresh install reproduces this system.\033[0m\n'
else
  printf '\033[31mINSTALL PARITY FAILED — fix the above before declaring done (Law 0).\033[0m\n'
fi
exit "$fail"
