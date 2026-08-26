#!/usr/bin/env bash
# install-open-design.sh — opt-in installer for the Open Design daemon.
# Clones upstream (pinned), builds a CLI-baked image so Local CLI mode detects
# the operator's claude/codex on their SUBSCRIPTION, mounts the OmegaOS projects
# so its "Select working directory" can open a real project for a refonte, wires
# git push, starts the Docker daemon, and serves it tailnet-only.
# NOT run by OmegaOS install.sh (heavy Docker dep — ZernFlow/higgsfield boundary).
#
#   OMEGA_SKIP_TS_SERVE=1    skip the tailscale serve step
#   OMEGA_SKIP_OD_LOCALCLI=1 skip baking claude/codex (use the UI's BYOK instead)
#   OMEGA_PROJECTS_ROOT=DIR  projects root to mount (default ~/Station)
#   OPEN_DESIGN_PORT=7456    host port
set -uo pipefail
declare -F ok   >/dev/null 2>&1 || ok()   { printf '  \033[32m[OK]\033[0m %s\n'   "$*"; }
declare -F info >/dev/null 2>&1 || info() { printf '  \033[36m[INFO]\033[0m %s\n' "$*"; }
declare -F warn >/dev/null 2>&1 || warn() { printf '  \033[33m[WARN]\033[0m %s\n' "$*" >&2; }
export GIT_TERMINAL_PROMPT=0

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
REPO_URL="https://github.com/nexu-io/open-design.git"
PIN="${OPEN_DESIGN_PIN:-7d7c56a}"
DST="$OMEGA_DIR/repos/open-design"
DEPLOY="$DST/deploy"
PORT="${OPEN_DESIGN_PORT:-7456}"
PROJECTS_ROOT="${OMEGA_PROJECTS_ROOT:-$HOME/Station}"
UID_GID="$(id -u):$(id -g)"

command -v docker >/dev/null 2>&1 || { warn "docker not found — Open Design needs Docker. Aborting."; exit 0; }

# 1) clone / fast-forward to the pin
mkdir -p "$OMEGA_DIR/repos" "$OMEGA_DIR/secrets"
if [[ -d "$DST/.git" ]]; then
    ( cd "$DST" && git fetch --depth 1 origin "$PIN" 2>/dev/null && git checkout -q "$PIN" 2>/dev/null ) || info "clone present; pin fetch skipped"
else
    git clone --depth 1 "$REPO_URL" "$DST" 2>&1 | tail -1 || { warn "clone failed"; exit 0; }
fi
ok "Open Design source at $DST ($(cd "$DST" && git rev-parse --short HEAD 2>/dev/null))"

# 2) .env with a generated token; view URL = tailnet HTTPS if Tailscale, else localhost
TS_HOST="$(tailscale status --json 2>/dev/null | python3 -c "import json,sys;print(json.load(sys.stdin)['Self']['DNSName'].rstrip('.'))" 2>/dev/null || true)"
if [[ -n "$TS_HOST" ]]; then VIEW="https://$TS_HOST:$PORT"; else VIEW="http://localhost:$PORT"; fi
if [[ ! -f "$DEPLOY/.env" ]]; then
    cp "$DEPLOY/.env.example" "$DEPLOY/.env"
    TOKEN="$(openssl rand -hex 32)"
    sed -i "s|^OD_API_TOKEN=.*|OD_API_TOKEN=$TOKEN|" "$DEPLOY/.env"
    sed -i "s|^OPEN_DESIGN_ALLOWED_ORIGINS=.*|OPEN_DESIGN_ALLOWED_ORIGINS=$VIEW|" "$DEPLOY/.env"
    grep -q '^OPEN_DESIGN_DISABLE_API_AUTH=' "$DEPLOY/.env" \
        && sed -i "s|^OPEN_DESIGN_DISABLE_API_AUTH=.*|OPEN_DESIGN_DISABLE_API_AUTH=1|" "$DEPLOY/.env" \
        || echo "OPEN_DESIGN_DISABLE_API_AUTH=1" >> "$DEPLOY/.env"
    grep '^OD_API_TOKEN=' "$DEPLOY/.env" > "$OMEGA_DIR/secrets/open-design.env" && chmod 600 "$OMEGA_DIR/secrets/open-design.env"
    ok "deploy/.env written (token mirrored to secrets)"
fi

# 3) LOCAL CLI mode: bake claude/codex into the image + run as the host user with
# auth, git and the projects mounted, so the daemon detects the CLIs (subscription)
# and can open/redesign a real OmegaOS project. Opt out: OMEGA_SKIP_OD_LOCALCLI=1.
COMPOSE=(docker compose -f "$DEPLOY/docker-compose.yml")
if [[ "${OMEGA_SKIP_OD_LOCALCLI:-0}" != "1" ]] && command -v claude >/dev/null 2>&1; then
    cat > "$DEPLOY/Dockerfile.omega-agents" <<'DOCKER'
FROM ghcr.io/nexu-io/od:latest
USER root
RUN npm i -g @anthropic-ai/claude-code @openai/codex 2>/dev/null || npm i -g @anthropic-ai/claude-code
USER 1001
DOCKER
    ( cd "$DEPLOY" && docker build -f Dockerfile.omega-agents -t od-omega:latest .. 2>&1 | tail -1 )

    # resolved agent HOME (auth + git), owned by the host uid so the daemon (run as
    # that uid) reads creds and writes project files with correct ownership.
    AH="$OMEGA_DIR/open-design-agent-home"; DATA="$OMEGA_DIR/open-design-data"
    mkdir -p "$AH/.claude" "$AH/.codex" "$DATA"
    RC="$(readlink -f "$HOME/.claude/.credentials.json" 2>/dev/null)"; [[ -f "$RC" ]] && cp "$RC" "$AH/.claude/.credentials.json"
    [[ -f "$HOME/.codex/auth.json" ]] && cp "$HOME/.codex/auth.json" "$AH/.codex/auth.json"
    [[ -f "$HOME/.codex/config.toml" ]] && cp "$HOME/.codex/config.toml" "$AH/.codex/config.toml"
    printf '[user]\n\tname = %s\n\temail = %s\n[credential]\n\thelper = store --file=/home/node/.git-credentials\n[safe]\n\tdirectory = *\n' \
        "$(git config --global user.name 2>/dev/null || echo agentik-os)" \
        "$(git config --global user.email 2>/dev/null || echo x@agentik-os.com)" > "$AH/.gitconfig"
    for gc in "$OMEGA_DIR/secrets/agentik-os.git-credentials" "$HOME/.git-credentials"; do
        [[ -f "$gc" ]] && cp "$gc" "$AH/.git-credentials" && break
    done
    chown -R "$UID_GID" "$AH" "$DATA" 2>/dev/null || sudo chown -R "$UID_GID" "$AH" "$DATA" 2>/dev/null || true

    cat > "$DEPLOY/docker-compose.omega.yml" <<YML
services:
  open-design:
    image: od-omega:latest
    read_only: false
    user: "$UID_GID"
    network_mode: host
    environment:
      HOME: /home/node
      OD_BIND_HOST: 127.0.0.1
      OD_PORT: "$PORT"
    volumes:
      - $DATA:/app/.od
      - $AH/.claude:/home/node/.claude
      - $AH/.codex:/home/node/.codex
      - $AH/.gitconfig:/home/node/.gitconfig
      - $AH/.git-credentials:/home/node/.git-credentials
      - $PROJECTS_ROOT:$PROJECTS_ROOT
YML
    COMPOSE+=(-f "$DEPLOY/docker-compose.omega.yml")
    ok "Local CLI mode: claude/codex baked + auth/git mounted; projects at $PROJECTS_ROOT (read-write)"
fi

# 4) pull + up (with the override when local-CLI mode is on)
( cd "$DEPLOY" && docker compose pull 2>&1 | tail -1 && "${COMPOSE[@]}" up -d 2>&1 | tail -1 ) || { warn "docker compose up failed"; exit 0; }

# 5) tailscale serve (tailnet-only)
if [[ "${OMEGA_SKIP_TS_SERVE:-0}" != "1" ]] && command -v tailscale >/dev/null 2>&1; then
    tailscale serve --bg --https=${PORT} "http://127.0.0.1:${PORT}" 2>/dev/null && ok "served tailnet-only at https://${TS_HOST:-localhost}:$PORT"
fi

# 6) install the omega-design CLI
if [[ -f "$(dirname "$0")/../../scripts/omega-design" ]]; then
    cp -f "$(dirname "$0")/../../scripts/omega-design" "$OMEGA_DIR/bin/omega-design"
    chmod +x "$OMEGA_DIR/bin/omega-design"
    ln -sf "$OMEGA_DIR/bin/omega-design" "${INSTALL_DIR:-$HOME/.local/bin}/omega-design" 2>/dev/null || true
fi

# 6b) the /opendesign slash command (Claude): run + link on Telegram.
CMD_DST="$HOME/.claude/commands"; mkdir -p "$CMD_DST"
cat > "$CMD_DST/opendesign.md" <<'MD'
# /opendesign

Bring up Open Design and give the operator its link.
Run in order, then report: `omega-design up`, `omega-design serve`,
`omega-design telegram` (pushes the link to Telegram), `omega-design open`.
Then tell the operator the URL (`omega-design url`), the SSH tunnel if there is
no Tailscale (`omega-design ssh`), and that projects are under "Recent projects"
or `omega-design import <project>`.
MD
sed 's/^# \/opendesign/# \/omg-opendesign/' "$CMD_DST/opendesign.md" > "$CMD_DST/omg-opendesign.md"
ok "/opendesign command installed"

# 7) verify
i=0; until curl -sf -o /dev/null "http://127.0.0.1:${PORT}/api/health" || [ $i -ge 30 ]; do sleep 2; i=$((i+1)); done
if curl -sf -o /dev/null "http://127.0.0.1:${PORT}/api/health"; then
    ok "Open Design healthy → view: $VIEW"

    # 7a) MCP — design research + project creation from a Claude/Codex chat. The
    # daemon's MCP server runs inside the container; agents reach it via
    # `docker exec`. Claude gets a user-scoped server; Codex needs its own
    # config.toml (it does not read Claude's). Opt out: OMEGA_SKIP_OD_MCP=1.
    if [[ "${OMEGA_SKIP_OD_MCP:-0}" != "1" ]]; then
        MCP_ARGS=(docker exec -i open-design node /app/apps/daemon/dist/cli.js mcp --daemon-url "http://127.0.0.1:${PORT}")
        if command -v claude >/dev/null 2>&1; then
            claude mcp remove open-design --scope user 2>/dev/null || true
            claude mcp add --scope user open-design -- "${MCP_ARGS[@]}" >/dev/null 2>&1 && ok "Claude MCP wired (open-design)"
        fi
        CXCFG="$HOME/.codex/config.toml"
        if [[ -f "$CXCFG" ]] && ! grep -q "\[mcp_servers.open-design\]" "$CXCFG"; then
            printf '\n[mcp_servers.open-design]\ncommand = "docker"\nargs = ["exec", "-i", "open-design", "node", "/app/apps/daemon/dist/cli.js", "mcp", "--daemon-url", "http://127.0.0.1:%s"]\n' "$PORT" >> "$CXCFG"
            ok "Codex MCP wired (~/.codex/config.toml)"
        fi
    fi

    # 7b) populate the UI with every OmegaOS project (the web folder-picker can't
    # reach server paths, so we register them server-side). IDEMPOTENT: skip a
    # baseDir already imported, so re-running on every update never duplicates.
    EXISTING="$(curl -s "http://127.0.0.1:${PORT}/api/projects" 2>/dev/null | python3 -c "import json,sys
try:
    d=json.load(sys.stdin); it=d.get('projects',d) if isinstance(d,dict) else d
    print('\n'.join((p.get('metadata') or {}).get('baseDir','') for p in it))
except Exception: pass" 2>/dev/null)"
    N=0
    while IFS= read -r p; do [[ -n "$p" ]] || continue
        printf '%s\n' "$EXISTING" | grep -qxF "$p" && continue
        curl -s -X POST "http://127.0.0.1:${PORT}/api/import/folder" -H "Content-Type: application/json" -d "{\"baseDir\":\"$p\"}" >/dev/null 2>&1 && N=$((N+1))
    done < <(find "$PROJECTS_ROOT" -maxdepth 3 -name .git -type d 2>/dev/null | sed 's#/.git##' | sort -u)
    ok "Imported $N new OmegaOS project(s) into Open Design (existing skipped)"
    info "Refonte: open $VIEW → pick a project. Composio/connector keys: omega-design composio <KEY> (server-side; the tailnet UI is loopback-only by design)."

    # 7c) SETTINGS: persist all the info (link + connections + commands) where the
    # operator can always find it, and surface it in `omega guide`.
    "$OMEGA_DIR/bin/omega-design" settings > "$OMEGA_DIR/open-design.info" 2>/dev/null || true
    GS="$OMEGA_DIR/GETTING-STARTED.md"
    if [[ -f "$GS" ]] && ! grep -q "## Open Design" "$GS" 2>/dev/null; then
        {
          echo ""
          echo "## Open Design (design engine)"
          echo ""
          echo "- View: **$VIEW** (tailnet; SSH tunnel if no Tailscale: \`omega-design ssh\`)"
          echo "- Runs on your subscription (claude + codex, Local CLI); drives from chat via MCP (\`/opendesign\`)."
          echo "- Full settings: \`omega-design settings\` (also saved at \`~/.omega/open-design.info\`)."
          echo "- Redesign a project: \"Recent projects\" on the Home, or \`omega-design import <project>\`."
        } >> "$GS"
        ok "Open Design added to GETTING-STARTED.md (omega guide) + ~/.omega/open-design.info"
    else
        ok "Open Design settings written → ~/.omega/open-design.info"
    fi
else
    warn "Open Design did not report healthy yet (check: docker logs open-design)"
fi
