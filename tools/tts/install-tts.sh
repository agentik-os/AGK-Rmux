#!/usr/bin/env bash
# OmegaOS TTS bench installer — local voice engines for Nova (CPU-only VPS).
# Idempotent. One engine failing must NOT kill the others (no set -e): each
# engine gets its own uv venv under ~/.omega/tts/venvs/<engine>; the daemon
# reports per-engine availability so a partial install degrades gracefully.
set -uo pipefail

OMEGA_DIR="${OMEGA_DIR:-$HOME/.omega}"
SRC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TTS_DIR="$OMEGA_DIR/tts"
PY="${OMEGA_TTS_PYTHON:-python3}"
FAILED=""
# VibeVoice is pinned: upstream removed its 1.5B TTS code once already (Sept 2025,
# "disabled due to widespread misuse"), so an unpinned clone is not reproducible.
VIBEVOICE_REPO="https://github.com/microsoft/VibeVoice.git"
VIBEVOICE_PIN="94da20d98b2fa7688e9cbfaf7692ddb4954f7600"
# OmniVoice (k2-fsa) — 1B diffusion-LM TTS, 600+ languages, zero-shot cloning.
# Pinned for reproducibility; weights (~2.3 GB) download lazily on first synthesis.
OMNIVOICE_REPO="https://github.com/k2-fsa/OmniVoice.git"
OMNIVOICE_PIN="38e992bc60f85548faeb77e8fa70158ba71deb30"

info() { printf '\033[36m[tts]\033[0m %s\n' "$*"; }
warn() { printf '\033[33m[tts]\033[0m %s\n' "$*"; }

mkdir -p "$TTS_DIR/venvs" "$TTS_DIR/workers" "$TTS_DIR/models/piper" "$TTS_DIR/out" "$TTS_DIR/logs"
# Seed the operator preference ONCE, never overwrite a real choice. A fresh install
# must not silently pull VibeVoice's ~2 GB: it is English-only and opt-in by design.
[[ -f "$TTS_DIR/config.json" ]] || printf '%s\n' '{"disabled": ["vibevoice"]}' > "$TTS_DIR/config.json"
cp -f "$SRC_DIR/ttsd.py" "$TTS_DIR/ttsd.py"
cp -f "$SRC_DIR/workers/"*.py "$SRC_DIR/workers/"*.sh "$TTS_DIR/workers/" 2>/dev/null || cp -f "$SRC_DIR/workers/"*.py "$TTS_DIR/workers/"
# Default bilingual cloning refs (CC BY 4.0, see voices/VOICES.md) — never
# overwrite an operator-customized voice of the same name.
if [[ -d "$SRC_DIR/voices" ]]; then
    mkdir -p "$TTS_DIR/voices"
    for v in "$SRC_DIR/voices/"*; do
        dst="$TTS_DIR/voices/$(basename "$v")"
        [[ -f "$dst" ]] || cp "$v" "$dst"
    done
fi

# Operator preference: engines listed in config.json "disabled" are neither
# served by the daemon nor (re)installed here — multi-GB venvs stay deleted.
is_disabled() { python3 -c "
import json,sys
try: sys.exit(0 if '$1' in (json.load(open('$TTS_DIR/config.json')).get('disabled') or []) else 1)
except Exception: sys.exit(1)
"; }

# uv makes the venv builds fast and reproducible; fall back to pip+venv.
UV="$(command -v uv || true)"
mkvenv() { # mkvenv <name> <pip-args…>
    local name="$1"; shift
    local venv="$TTS_DIR/venvs/$name"
    if [[ -n "$UV" ]]; then
        [[ -x "$venv/bin/python" ]] || "$UV" venv "$venv" --python "$PY" >/dev/null 2>&1
        "$UV" pip install --python "$venv/bin/python" --quiet "$@"
    else
        [[ -x "$venv/bin/python" ]] || "$PY" -m venv "$venv"
        "$venv/bin/pip" install --quiet --upgrade pip && "$venv/bin/pip" install --quiet "$@"
    fi
}

# ffmpeg (OGG/Opus conversion) + espeak-ng (kokoro phonemizer) are the only
# system deps; skip silently when sudo isn't available non-interactively.
if command -v apt-get >/dev/null 2>&1 && sudo -n true 2>/dev/null; then
    command -v ffmpeg >/dev/null 2>&1 || sudo apt-get install -y -qq ffmpeg >/dev/null 2>&1
    command -v espeak-ng >/dev/null 2>&1 || sudo apt-get install -y -qq espeak-ng >/dev/null 2>&1
fi

if is_disabled pocket; then info "pocket disabled by config — skipped"; else
    info "pocket (Kyutai Pocket TTS — CPU real-time, French)…"
    mkvenv pocket pocket-tts scipy || { warn "pocket install failed"; FAILED="$FAILED pocket"; }
fi

if is_disabled kokoro; then info "kokoro disabled by config — skipped"; else
    info "kokoro (Kokoro 82M — light, French ff_siwis)…"
    mkvenv kokoro kokoro soundfile || { warn "kokoro install failed"; FAILED="$FAILED kokoro"; }
fi

if is_disabled piper; then info "piper disabled by config — skipped"; else
    info "piper (Piper — fastest, fr_FR-siwis)…"
    if mkvenv piper piper-tts; then
        "$TTS_DIR/venvs/piper/bin/python" -m piper.download_voices --data-dir "$TTS_DIR/models/piper" fr_FR-siwis-medium >/dev/null 2>&1 \
            || warn "piper voice download failed (will retry lazily at first use)"
    else
        warn "piper install failed"; FAILED="$FAILED piper"
    fi
fi

if is_disabled chatterbox; then info "chatterbox disabled by config — skipped"; else
    info "chatterbox (Resemble — top quality, heavy; CPU torch)…"
    mkvenv chatterbox chatterbox-tts --extra-index-url https://download.pytorch.org/whl/cpu \
        || { warn "chatterbox install failed"; FAILED="$FAILED chatterbox"; }
fi

# VibeVoice-Realtime-0.5B (Microsoft) — ENGLISH long-form / podcast. DISABLED BY
# DEFAULT in the shipped config: it pulls ~2 GB of weights plus CPU torch, and its
# French is measured unusable (53.8% word error rate round-tripped through Whisper,
# vs 5.1% for piper/kokoro), so it must never become a default French voice.
# Enable deliberately: remove "vibevoice" from ~/.omega/tts/config.json "disabled".
if is_disabled vibevoice; then info "vibevoice disabled by config — skipped"; else
    info "vibevoice (Microsoft VibeVoice-Realtime-0.5B — English long-form; ~2 GB)…"
    if [[ -d "$TTS_DIR/vibevoice/.git" ]]; then
        git -C "$TTS_DIR/vibevoice" fetch --depth 1 origin "$VIBEVOICE_PIN" >/dev/null 2>&1 \
            && git -C "$TTS_DIR/vibevoice" checkout -q "$VIBEVOICE_PIN" 2>/dev/null || true
    else
        git clone -q "$VIBEVOICE_REPO" "$TTS_DIR/vibevoice" >/dev/null 2>&1 \
            && git -C "$TTS_DIR/vibevoice" checkout -q "$VIBEVOICE_PIN" 2>/dev/null || true
    fi
    if [[ -f "$TTS_DIR/vibevoice/pyproject.toml" ]]; then
        # torch CPU wheels first so the default CUDA build (multi-GB, useless here) is never pulled.
        if mkvenv vibevoice torch --index-url https://download.pytorch.org/whl/cpu \
           && mkvenv vibevoice --editable "$TTS_DIR/vibevoice[streamingtts]" soundfile; then
            info "vibevoice installed — weights download lazily on first synthesis"
        else
            warn "vibevoice install failed"; FAILED="$FAILED vibevoice"
        fi
    else
        warn "vibevoice clone failed (network?) — will retry on the next run"; FAILED="$FAILED vibevoice"
    fi
fi

# OmniVoice (k2-fsa) — best local French measured on this bench (0-6% Whisper WER,
# homophone-level) AND faster than real-time warm on CPU (~2.5 s for 6 s of audio,
# bf16 worker on avx512_bf16; the fp16 CLI default is ~10x slower — keep bf16).
# Venv ~2 GB (CPU torch) + 2.3 GB weights lazily on first synthesis; worker holds
# ~5 GB RSS once loaded. numba>=0.60 is preinstalled: the resolver otherwise picks
# an ancient numba (0.53) whose llvmlite cannot build on python>=3.10.
if is_disabled omnivoice; then info "omnivoice disabled by config — skipped"; else
    info "omnivoice (k2-fsa OmniVoice 1B — 600+ languages, voice cloning; ~2 GB)…"
    if [[ -d "$TTS_DIR/omnivoice/.git" ]]; then
        git -C "$TTS_DIR/omnivoice" fetch --depth 1 origin "$OMNIVOICE_PIN" >/dev/null 2>&1 \
            && git -C "$TTS_DIR/omnivoice" checkout -q "$OMNIVOICE_PIN" 2>/dev/null || true
    else
        git clone -q "$OMNIVOICE_REPO" "$TTS_DIR/omnivoice" >/dev/null 2>&1 \
            && git -C "$TTS_DIR/omnivoice" checkout -q "$OMNIVOICE_PIN" 2>/dev/null || true
    fi
    if [[ -f "$TTS_DIR/omnivoice/pyproject.toml" ]]; then
        # torch CPU wheels first so the default CUDA build (multi-GB, useless here) is never pulled.
        if mkvenv omnivoice torch torchaudio --index-url https://download.pytorch.org/whl/cpu \
           && mkvenv omnivoice 'numba>=0.60' \
           && mkvenv omnivoice --editable "$TTS_DIR/omnivoice"; then
            info "omnivoice installed — weights download lazily on first synthesis"
        else
            warn "omnivoice install failed"; FAILED="$FAILED omnivoice"
        fi
    else
        warn "omnivoice clone failed (network?) — will retry on the next run"; FAILED="$FAILED omnivoice"
    fi
fi

# Judge venv — faster-whisper (ref transcription + WER gate) and jiwer. Small
# (~100 MB, CPU int8); the omnivoice cloning flow reads ref transcriptions from
# .txt sidecars that judge_wer.py produces, and every voice delivered to the
# operator is gated on MOS+WER (the UTMOS weights download lazily on first
# judge_mos.py run — 411 MB, bench-only, not needed at install).
if is_disabled judge; then info "judge disabled by config — skipped"; else
    info "judge (faster-whisper WER + UTMOS MOS quality gates)…"
    mkvenv judge faster-whisper jiwer || { warn "judge install failed"; FAILED="$FAILED judge"; }
fi

# systemd user service (Linux). The daemon is stdlib-only → system python3.
if command -v systemctl >/dev/null 2>&1; then
    SD_DIR="$HOME/.config/systemd/user"; mkdir -p "$SD_DIR"
    cat > "$SD_DIR/omega-ttsd.service" <<EOF
[Unit]
Description=OmegaOS TTS gateway daemon (voice engines for Nova)
After=network-online.target

[Service]
Type=simple
Environment=OMEGA_DIR=%h/.omega
ExecStart=$(command -v python3) %h/.omega/tts/ttsd.py
Restart=always
RestartSec=3

[Install]
WantedBy=default.target
EOF
    systemctl --user daemon-reload 2>/dev/null || true
    systemctl --user enable omega-ttsd.service 2>/dev/null || true
    systemctl --user restart omega-ttsd.service 2>/dev/null || true
fi

if [[ -n "$FAILED" ]]; then
    warn "done with failures:$FAILED (daemon still serves the engines that installed)"
    exit 1
fi
info "done — daemon on 127.0.0.1:8765 (GET /engines to check status)"
