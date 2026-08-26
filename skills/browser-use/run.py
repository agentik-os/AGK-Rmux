#!/usr/bin/env python3
"""Run one natural-language task on the Browser Use cloud and print the result.

Cloud API: https://api.browser-use.com (official Browser Use cloud SDK,
PyPI package "browser-use-sdk"). Authentication is via the BROWSER_USE_API_KEY
environment variable, which the SDK client reads automatically — this script
NEVER reads, prints, or hardcodes the key value. The key is provided by the
`browser-use` wrapper from ~/.omega/secrets/integrations.env.

The task text is taken from argv (a single quoted element) and passed verbatim
to the cloud agent; no shell eval, no interpolation.
"""
import asyncio
import re
import sys

# Documented v3 cloud entrypoint. The live cloud docs demonstrate the async
# client for Python (from browser_use_sdk.v3 import AsyncBrowserUse); we drive
# it synchronously via asyncio.run() so this stays a one-shot CLI.
from browser_use_sdk.v3 import AsyncBrowserUse


async def _run(task: str) -> str:
    # AsyncBrowserUse() reads BROWSER_USE_API_KEY from the environment.
    client = AsyncBrowserUse()
    # client.run() creates a cloud session, polls until completion, returns it.
    result = await client.run(task)
    # result.output is the agent's final answer (a string, or a structured
    # object if a schema was passed — we pass none, so it is the final text).
    return result.output


def main() -> int:
    task = " ".join(sys.argv[1:]).strip()
    if not task:
        print('usage: run.py "<natural-language browser task>"', file=sys.stderr)
        return 2
    try:
        output = asyncio.run(_run(task))
    except Exception as exc:  # noqa: BLE001 — surface a concise, key-free error
        # Defensive: redact anything resembling a bu_ key, in case a verbose
        # SDK error echoes auth material.
        msg = re.sub(r"bu_[A-Za-z0-9]+", "bu_***", str(exc))
        print(f"browser-use: cloud run failed: {msg}", file=sys.stderr)
        return 1
    print(output)
    return 0


if __name__ == "__main__":
    sys.exit(main())
