#!/usr/bin/env python3
"""omega-transcribe — local open-source audio transcription via faster-whisper.

Uses the faster-whisper library (MIT, github.com/SYSTRAN/faster-whisper),
a CTranslate2-based reimplementation of OpenAI Whisper that runs entirely
locally with no API key and no network call during transcription.

Usage:
    omega-transcribe <file> [options]

Options:
    --model     base | small | medium | large-v3   (default: base)
    --language  fr | en | auto | <code>            (default: auto)
    --output    txt | json | srt                   (default: txt)
    --out-file  <path>  write to file instead of stdout
    --device    cpu | cuda | auto                  (default: auto)

Examples:
    omega-transcribe call.mp3
    omega-transcribe interview.wav --model medium --language fr --output srt
    omega-transcribe audio.m4a --output json --out-file transcript.json
"""
import argparse
import json
import os
import sys


def detect_device() -> str:
    try:
        import ctranslate2
        if ctranslate2.get_cuda_device_count() > 0:
            return "cuda"
    except Exception:
        pass
    return "cpu"


def format_srt_time(seconds: float) -> str:
    h = int(seconds // 3600)
    m = int((seconds % 3600) // 60)
    s = int(seconds % 60)
    ms = int((seconds % 1) * 1000)
    return f"{h:02d}:{m:02d}:{s:02d},{ms:03d}"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Local open-source transcription (faster-whisper)"
    )
    parser.add_argument("file", help="Audio file to transcribe")
    parser.add_argument(
        "--model",
        default="base",
        choices=["tiny", "base", "small", "medium", "large-v2", "large-v3"],
        help="Whisper model size (default: base)",
    )
    parser.add_argument(
        "--language",
        default=None,
        help="Language code (fr, en, …) or omit for auto-detect",
    )
    parser.add_argument(
        "--output",
        default="txt",
        choices=["txt", "json", "srt"],
        help="Output format (default: txt)",
    )
    parser.add_argument("--out-file", help="Write output to file (default: stdout)")
    parser.add_argument(
        "--device",
        default="auto",
        choices=["cpu", "cuda", "auto"],
        help="Compute device (default: auto)",
    )
    args = parser.parse_args()

    if not os.path.exists(args.file):
        print(f"ERROR: file not found: {args.file}", file=sys.stderr)
        return 1

    try:
        from faster_whisper import WhisperModel  # type: ignore[import]
    except ImportError:
        print(
            "ERROR: faster-whisper not installed.\n"
            "Run: bash ~/.omega/tools/transcription/install-transcription.sh\n"
            "  or: pip install faster-whisper",
            file=sys.stderr,
        )
        return 1

    device = detect_device() if args.device == "auto" else args.device
    compute = "float16" if device == "cuda" else "int8"

    print(f"[omega-transcribe] model={args.model} device={device} file={args.file}", file=sys.stderr)
    model = WhisperModel(
        args.model,
        device=device,
        compute_type=compute,
        download_root=os.path.expanduser("~/.omega/tools/transcription/models"),
    )

    lang = args.language if args.language and args.language != "auto" else None
    segments_iter, info = model.transcribe(
        args.file,
        language=lang,
        beam_size=5,
        vad_filter=True,
    )
    detected = getattr(info, "language", "?")
    print(f"[omega-transcribe] detected language: {detected}", file=sys.stderr)

    segments = list(segments_iter)

    if args.output == "txt":
        result = " ".join(s.text.strip() for s in segments)
    elif args.output == "srt":
        lines = []
        for i, seg in enumerate(segments, start=1):
            lines.append(str(i))
            lines.append(f"{format_srt_time(seg.start)} --> {format_srt_time(seg.end)}")
            lines.append(seg.text.strip())
            lines.append("")
        result = "\n".join(lines)
    else:
        result = json.dumps(
            {
                "language": detected,
                "segments": [
                    {"start": s.start, "end": s.end, "text": s.text.strip()}
                    for s in segments
                ],
                "full_text": " ".join(s.text.strip() for s in segments),
            },
            ensure_ascii=False,
            indent=2,
        )

    if args.out_file:
        with open(args.out_file, "w", encoding="utf-8") as fh:
            fh.write(result)
        print(f"[omega-transcribe] written to {args.out_file}", file=sys.stderr)
    else:
        print(result)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
