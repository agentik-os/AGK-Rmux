# Shipped default cloning references

- `nova-fr.wav` — 10 s French female reference, from the CML-TTS corpus
  (speaker 10087, LibriVox-derived, CC BY 4.0 — https://huggingface.co/datasets/ylacombe/cml-tts).
  Measured UTMOS 3.81; OmniVoice clone measured 3.72 / 0% WER on this bench.
- `nova-en.wav` — 10 s English female reference, from the VCTK corpus
  (speaker p329, CC BY 4.0 — https://datashare.ed.ac.uk/handle/10283/3443).
  Measured UTMOS 4.03; OmniVoice clone measured 4.30 / 0% WER.

Each `.txt` is the exact transcription (the omnivoice worker reads it as the
cloning `ref_text` sidecar). Installed to `~/.omega/tts/voices/` without
overwriting operator-customized versions.
