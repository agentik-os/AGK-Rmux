# omega-os

One-command installer for **OmegaOS** — the agentic terminal OS (rmux + AI orchestration).

```bash
npx omega-os
```

Shows the OMEGA banner, then:

1. Clones the public repo [`agentik-os/OmegaOS`](https://github.com/agentik-os/OmegaOS)
   (into `~/Station/OmegaOS` if `~/Station` exists, else `./OmegaOS`)
2. Runs its `install.sh` — builds rmux + omega from source, sets up `~/.omega`,
   skills, Atlas + the 15 AISB agents, rules, hooks, mosh, bun (~8 min on a fresh box)

While it builds, the **Matrix rain is interactive**: type to inject bright
glyph-drops into the rain, press **`space`** for a pulse, and watch each install
phase name decode out of the katakana. The OMEGA progress bar stays pinned at the
bottom the whole time. `--plain` disables the animation and shows a simple bar.

```bash
npx omega-os --dir /opt/omega   # custom location
npx omega-os --plain            # disable animation, simple bar only
npx omega-os --no-telegram      # skip the guided Telegram remote-control setup
npx omega-os --help
```

After install:

```bash
exec $SHELL -l
omega doctor    # verify (should be healthy)
omega           # launch the TUI
```

Set up Telegram / LLM providers from the TUI — pressing **Enter** on any panel
opens a guided wizard (no command line needed).

**Requirements:** Linux/macOS, `git`, internet, `sudo` for OS packages.
Zero npm runtime dependencies (Node builtins only). Node ≥ 16.

License: MIT OR Apache-2.0 · © Agentik OS
