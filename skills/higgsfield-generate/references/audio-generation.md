# Audio generation

Higgsfield generates audio — **voiceover / text-to-speech, voice cloning, sound, and dubbing** — alongside images and video. This is audio **output** (generation). It is NOT the `--audio` reference input used for Seedance lipsync / soundtrack matching; for that audio-as-input flow see `media-inputs.md`.

> Image + video + **audio** now live under one Higgsfield stack — you can produce a full piece of content (visual + narration + localization) without leaving the CLI. Governed by R-VISUAL-ID: paid plan, `higgsfield auth login`, runtime-installed CLI; OmegaOS ships only this markdown and never auto-installs the CLI, and a live audio render is not runtime-verifiable without operator credentials.

## Discovery (do this first)

Audio models appear in the unfiltered model list. Two of them (`dubbing`, `voice_change`) are typed `video` because they rewrite a video's voice track — classify by task intent, not by the `type` field:

```bash
higgsfield model list --json | jq '.[] | select(.type=="audio") | {display_name, job_set_type}'
higgsfield model get <job_set_type> --json    # exact params, enums, defaults
```

## Models & parameters (runtime-verified, CLI v0.1.40)

### Voiceover / text-to-speech

**`seed_audio` — Seed Audio 1.0** (default voiceover)

| Param | Type | Required | Default / values |
|---|---|---|---|
| `prompt` | string | yes | the text to speak |
| `format` | enum | no | `wav` (also `mp3`, `pcm`, `ogg_opus`) |
| `sample_rate` | enum | no | `24000` (also 8000/16000/32000/44100/48000) |
| `speech_rate` | int | no | `0` |
| `pitch_rate` | int | no | `0` |
| `loudness_rate` | int | no | `0` |
| `medias` / `speaker` | array / object | no | optional reference / speaker selection |

```bash
higgsfield generate create seed_audio --prompt "Your brand, amplified." --format mp3 --sample_rate 48000 --wait
```

**`text2speech_v2` — multi-engine TTS**

| Param | Type | Required | Values |
|---|---|---|---|
| `model` | enum | yes | `elevenlabs`, `minimax`, `seed_speech`, `vibe_voice`, `cozy_voice` |
| `prompt` | string | yes | the text to speak |
| `voice_id` | string | yes | id from the Higgsfield voice library / a cloned voice |
| `voice_type` | enum | yes | `preset` or `element` (custom) |

**ElevenLabs is an engine here**, selected with `--model elevenlabs` — there is no standalone `elevenlabs` job_set_type. The announcement's "ElevenLabs" voiceover maps to this flag.

```bash
higgsfield generate create text2speech_v2 --model elevenlabs --voice_type preset --voice_id <voice_id> --prompt "..." --wait
```

**`inworld_text_to_speech`** — `--prompt` (req), `--voice` (req).

### Sound & music

- **`mirelo_text_to_audio`** — sound / SFX / ambience from text: `--prompt` (req), `--duration` (req, number).
- **`sonilo_music`** — Sonilo Music. Run `higgsfield model get sonilo_music` for its params before use.

### Dubbing — localize a finished video

**`dubbing`** (typed `video`): `--video <path-or-id>` (req) + `--target_language <code>` (req).

Enumerated `target_language` codes in CLI v0.1.40 (**18**): `eng, cmn, fra, hin, ita, jpn, kor, por, rus, tur, spa, deu, ara, pol, ind, fil, swe, fin`. Higgsfield markets "50+ languages"; the above is the in-CLI subset today — re-check with `model get dubbing` as it expands.

```bash
higgsfield generate create dubbing --video ./ad.mp4 --target_language jpn --wait
```

### Voice cloning / transfer

**`voice_change`** (typed `video`): `--video <path-or-id>` (req), `--voice_id` (req), `--voice_type preset|element` (default `preset`). Swaps the speaking voice in a video for a consistent brand voice.

```bash
higgsfield generate create voice_change --video ./ad.mp4 --voice_type preset --voice_id <voice_id> --wait
```

### Transcription

**`speech2text`** — `--audio <path-or-id>` (req). Audio → text (captions / verbatim).

## Notes & honest boundaries

- **Voice ids:** no `voices list` command exists in v0.1.40. Get a `voice_id` from the Higgsfield voice library / app (preset) or a cloned **element** voice. Don't fabricate one.
- **Media flags:** the `input_video` / `input_audio` schema fields are supplied via the standard `--video` / `--audio` flags (path auto-uploaded, or an upload/job id). Confirm with `model get` if a flag is rejected.
- **Not runtime-verified here:** a live audio render (it spends credits on the operator's paid plan). The capability above is verified from `higgsfield model list` + `higgsfield model get` schemas, not a submitted job.
