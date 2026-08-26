# Provider compatibility

OmegaOS validates installed agent CLIs with `omega doctor`. Reinstall or update
an existing provider with:

```bash
omega install <provider> --force
```

## Supported contracts

| Provider | Minimum tested CLI | Omega launch contract |
|---|---:|---|
| Claude Code | 2.1.219 | interactive TTY, `--permission-mode auto` |
| Codex | 0.147.0 | `--sandbox workspace-write --ask-for-approval never` (never pair `--sandbox` with `--approve-for-me`) |
| Gemini CLI | 0.31.0 | `--prompt-interactive --yolo`, Enterprise/API-key accounts |
| Antigravity (`agy`) | 1.1.8 | native Google auth, prompt-interactive |
| Pi / OpenRouter | 0.84.3 | explicit provider/model, `--approve` (no tool-yolo on official CLI) |
| Hermes | 0.20.0 | Home TUI: `hermes chat --yolo` with `HERMES_HOME`. `install.sh` installs the CLI. `omega sync` wires `~/.hermes` (SOUL, AGENTS.md, skills, `/omegaos` bundle). Never `-q` in a pane. Never `dispatch --agent hermes`. |
| Kimi Code | 0.38.0 | `--auto` on interactive and `--prompt` launches |
| GLM | Claude Code 2.1.219 | Claude adapter pointed at Z.AI Anthropic endpoint |

Current catalog defaults are `gpt-5.6` for Codex, `auto` for Gemini CLI,
`glm-5.3` for direct GLM, and `anthropic/claude-opus-5` for OpenRouter-backed
Pi/Hermes sessions. Account-scoped products may resolve an alias to a different
eligible model; use the provider's own model-list command to verify the actual
selection.

Omega-managed Codex sessions default `codex.bypass_hook_trust = true` so
installer-managed hooks cannot block a detached pane. This applies to every
enabled Codex hook; set it to `false` if you keep third-party hooks that must be
reviewed interactively.

## Google migration

Google stopped serving Gemini CLI requests for free, AI Pro, and AI Ultra
individual accounts on June 18, 2026. Those users should install Antigravity:

```bash
omega install antigravity
agy                         # authenticate once
omega config activate antigravity
```

Gemini CLI remains supported for Gemini Code Assist Standard/Enterprise and
paid Gemini or Enterprise Agent Platform API keys. Gemini and Antigravity keep
OAuth credentials in their native hybrid/keyring stores; OmegaOS does not copy
or symlink those credentials.

## Selecting a provider

```bash
omega config activate codex gpt-5.6
omega config activate claude opus
omega config activate antigravity       # native account default
omega new home --agent hermes
omega dispatch MyProject "mission" --agent codex
```

Hermes is Home. `./install.sh` installs the Hermes CLI when missing, then
`omega sync` creates `~/.hermes` even before the first `hermes chat`. The
messaging gateway (`hermes gateway`) is a user service managed by
`omega hermes-gateway {install,setup,start,status}`. It must not share a
Telegram bot token with Omega Atlas. Cloud (Cursor Cloud Agent) is the
Cursor-side writer for OmegaOS itself — it is not `omega dispatch`. See
`docs/ADR-lab-three-backends.md`.

The active global selection is mirrored in
`~/.omega/state/active-model.json`. A mission-level `--agent` override takes
precedence without changing the global default.

## Upgrade checks

After updating OmegaOS:

```bash
omega reconcile
omega doctor
omega doctor --deep
```

`doctor` checks the executable version before a detached pane can fail on an
obsolete flag. `--deep` additionally performs provider authentication probes
where supported.
