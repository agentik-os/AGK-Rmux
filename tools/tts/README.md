# omega-ttsd — TTS voice bench

One local gateway in front of every voice engine on the box, so Nova (and any
omega tool) synthesizes speech with a single `POST /tts` whatever the engine.
The operator picks the reply mode (text / voice / both) and the engine from
Nova's Telegram menu (`/menu` → 🔊 Voix) and can A/B them with 🧪 test notes.

## Engines

| id | What | Why it's here |
|---|---|---|
| `pocket` | [Kyutai Pocket TTS](https://github.com/kyutai-labs/pocket-tts), 100M, `french_24l` | Real-time on CPU, native French — the live-call candidate |
| `chatterbox` | [Resemble Chatterbox](https://github.com/resemble-ai/chatterbox) multilingual | Beats ElevenLabs in blind tests; slow on CPU → async notes |
| `kokoro` | [Kokoro 82M](https://github.com/hexgrad/kokoro), voice `ff_siwis` | Light/fast baseline |
| `piper` | [Piper](https://github.com/OHF-Voice/piper1-gpl), `fr_FR-siwis-medium` | Fastest; fillers + fallback |
| `omnivoice` | [k2-fsa OmniVoice](https://github.com/k2-fsa/OmniVoice), 1B diffusion-LM | 600+ languages, zero-shot voice cloning; best local French measured (0% WER at 16 steps) and faster than real-time warm (bf16 worker) |
| `elevenlabs` | [ElevenLabs API](https://elevenlabs.io), `eleven_multilingual_v2` | Hosted reference; needs `ELEVENLABS_API_KEY` in `provisioning/services.env` |

GPU-bound or non-commercial engines (Voxtral, XTTS, Fish, F5…) are deliberately
excluded — this VPS has no GPU.

## Architecture

- `ttsd.py` — stdlib-only HTTP daemon on `127.0.0.1:8765` (systemd user unit
  `omega-ttsd`). `GET /engines` (status), `POST /tts {engine,text}` → OGG/Opus.
- `workers/*.py` — one persistent subprocess per local engine (own uv venv under
  `~/.omega/tts/venvs/`, model kept in RAM, lazy-spawned). Protocol: one JSON
  per line on stdin/stdout; fd1 is dup'ed for the protocol so library progress
  bars can't corrupt it (they go to `~/.omega/tts/logs/<engine>.log`).
- `install-tts.sh` — idempotent, per-engine fault-tolerant; wired into
  `install.sh` (Law 0).

ElevenLabs voice/model overrides: `~/.omega/tts/elevenlabs.json`
(`{"voice_id": "...", "model_id": "..."}`).
