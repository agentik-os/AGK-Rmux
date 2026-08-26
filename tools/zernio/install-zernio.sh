#!/usr/bin/env bash
# OmegaOS Zernio CLI installer — deploys the pure-Bun omega-zernio CLI.
# Mirrors tools/tts/install-tts.sh. Idempotent: copies the CLI into
# ~/.omega/skills/zernio and writes a launcher at ~/.local/bin/omega-zernio.
# No npm deps, no secrets touched — the key is read at runtime from
# ~/.omega/secrets/integrations.env.
set -uo pipefail

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
SRC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SKILL_DIR="$OMEGA_DIR/skills/zernio"
BIN_DIR="$HOME/.local/bin"
LAUNCHER="$BIN_DIR/omega-zernio"

info() { printf '\033[36m[zernio]\033[0m %s\n' "$*"; }
warn() { printf '\033[33m[zernio]\033[0m %s\n' "$*"; }

mkdir -p "$SKILL_DIR" "$BIN_DIR"

cp -f "$SRC_DIR/cli.ts" "$SKILL_DIR/cli.ts"
cp -f "$SRC_DIR/package.json" "$SKILL_DIR/package.json"
cp -f "$SRC_DIR/README.md" "$SKILL_DIR/README.md"

cat > "$LAUNCHER" <<'EOF'
#!/usr/bin/env bash
exec bun "$HOME/.omega/skills/zernio/cli.ts" "$@"
EOF
chmod +x "$LAUNCHER"
chmod +x "$SRC_DIR/install-zernio.sh" 2>/dev/null || true

if ! command -v bun >/dev/null 2>&1; then
    warn "bun not found on PATH — omega-zernio needs Bun at runtime (/usr/local/bin/bun)"
fi
case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *) warn "$BIN_DIR is not on PATH — add it or call $LAUNCHER directly" ;;
esac

info "installed → $LAUNCHER (runs $SKILL_DIR/cli.ts)"
