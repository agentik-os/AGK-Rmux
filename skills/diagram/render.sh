#!/usr/bin/env bash
# OmegaOS diagram skill — render diagram-as-code to SVG (+ PNG).
#
#   render.sh <input.(mmd|d2)> [out_basename]
#
# Detects syntax by extension:
#   .mmd / .mermaid  -> @mermaid-js/mermaid-cli  (npx, browserless via --no-sandbox)
#   .d2              -> d2                        (single static binary, lazy-installed)
#
# Renderers install LAZILY at runtime to a user dir (never sudo, never in
# install.sh) — exactly like the higgsfield / browser-use precedent:
#   - d2  -> $HOME/.local/bin       (official installer, --prefix "$HOME/.local")
#   - mmdc-> npx -y                 (cached under ~/.npm)
# After SVG, if rsvg-convert is available, a PNG is also produced from the SVG.
# If NO renderer can be obtained, this prints the source path + clear install
# instructions and exits non-zero (never a silent no-op).
#
# R-CLI: pure CLI, no MCP, no paid API. R-ENV: user-dir installs only.
set -euo pipefail

# ---------- args ----------
if [[ $# -lt 1 ]]; then
  echo "usage: render.sh <input.(mmd|d2)> [out_basename]" >&2
  exit 2
fi
IN="$1"
if [[ ! -f "$IN" ]]; then
  echo "render.sh: input not found: $IN" >&2
  exit 2
fi
IN="$(cd "$(dirname "$IN")" && pwd)/$(basename "$IN")"   # absolutize

EXT="${IN##*.}"
EXT="$(printf '%s' "$EXT" | tr '[:upper:]' '[:lower:]')"

# out_basename: explicit arg, else input path minus its extension
if [[ $# -ge 2 && -n "${2:-}" ]]; then
  OUT="$2"
else
  OUT="${IN%.*}"
fi
case "$OUT" in
  /*) : ;;                                   # already absolute
  *)  OUT="$(pwd)/$OUT" ;;
esac
mkdir -p "$(dirname "$OUT")"
SVG="$OUT.svg"
PNG="$OUT.png"

export PATH="$HOME/.local/bin:$PATH"

note() { printf '\033[36m[diagram]\033[0m %s\n' "$*" >&2; }
err()  { printf '\033[31m[diagram]\033[0m %s\n' "$*" >&2; }

# ---------- d2 lazy install ----------
ensure_d2() {
  if command -v d2 >/dev/null 2>&1; then return 0; fi
  note "d2 not found — installing the single static binary to \$HOME/.local (no sudo)…"
  mkdir -p "$HOME/.local/bin"
  # Official installer; PREFIX/--prefix keeps it in the user dir, never /usr/local.
  if curl -fsSL https://d2lang.com/install.sh | sh -s -- --prefix "$HOME/.local" >&2; then
    :
  else
    err "d2 official installer failed (network?)."
  fi
  hash -r 2>/dev/null || true
  command -v d2 >/dev/null 2>&1
}

# ---------- mermaid puppeteer config (browserless) ----------
MMDC_CFG=""
write_mmdc_cfg() {
  MMDC_CFG="$(mktemp -t mmdc-puppeteer-XXXXXX.json)"
  cat > "$MMDC_CFG" <<'JSON'
{ "args": ["--no-sandbox", "--disable-setuid-sandbox", "--disable-gpu", "--disable-dev-shm-usage"] }
JSON
}

render_mermaid() {
  if ! command -v npx >/dev/null 2>&1; then
    err "npx (Node.js) not found — cannot run mermaid-cli."
    err "Install Node.js, or convert the diagram to .d2 and re-run."
    return 1
  fi
  write_mmdc_cfg
  note "rendering Mermaid via npx @mermaid-js/mermaid-cli (first run downloads the CLI + Chromium)…"
  # -p: puppeteer config (no-sandbox). -b transparent keeps slides clean.
  # Optional MERMAID CONFIG (-c) and THEME (-t) from the environment: callers that need
  # SVG-native labels (htmlLabels:false, so the PNG keeps its box text) or a custom theme
  # set MMDC_CONFIG=<file.json> / MMDC_THEME=<name>. Absent → default Mermaid rendering.
  local extra=()
  [ -n "${MMDC_CONFIG:-}" ] && [ -f "$MMDC_CONFIG" ] && extra+=(-c "$MMDC_CONFIG")
  [ -n "${MMDC_THEME:-}" ] && extra+=(-t "$MMDC_THEME")
  npx -y @mermaid-js/mermaid-cli -i "$IN" -o "$SVG" -p "$MMDC_CFG" "${extra[@]}" -b transparent >&2
}

render_d2() {
  if ! ensure_d2; then
    err "Could not obtain the d2 renderer."
    err "Manual install:  curl -fsSL https://d2lang.com/install.sh | sh -s -- --prefix \"\$HOME/.local\""
    err "Then ensure \$HOME/.local/bin is on PATH and re-run, or convert to .mmd."
    return 1
  fi
  note "rendering D2 via $(command -v d2)…"
  # --pad keeps a small margin; theme/vars come from the .d2 source itself.
  d2 --pad 20 "$IN" "$SVG" >&2
}

# ---------- dispatch ----------
case "$EXT" in
  mmd|mermaid) render_mermaid ;;
  d2)          render_d2 ;;
  *)
    err "unknown extension '.$EXT' — use .mmd (Mermaid) or .d2 (D2)."
    exit 2
    ;;
esac

# ---------- verify SVG ----------
if [[ ! -s "$SVG" ]]; then
  err "render produced no SVG (see errors above). Source kept at: $IN"
  exit 1
fi
if ! grep -q "<svg" "$SVG" 2>/dev/null; then
  err "output does not look like SVG. Source kept at: $IN"
  exit 1
fi

# ---------- SVG -> PNG (rsvg-convert if present) ----------
if command -v rsvg-convert >/dev/null 2>&1; then
  # 2x scale for crisp slides / retina.
  if rsvg-convert -z 2 -f png -o "$PNG" "$SVG" >/dev/null 2>&1; then
    note "PNG written."
  else
    err "rsvg-convert failed; SVG is still valid."
    PNG=""
  fi
else
  note "rsvg-convert not found — SVG only (install librsvg2-bin for PNG)."
  PNG=""
fi

# ---------- report ----------
echo "SVG: $SVG"
[[ -n "$PNG" && -s "$PNG" ]] && echo "PNG: $PNG"
echo "SRC: $IN"
