# OmegaOS centralized configuration architecture v3

> **Scope:** authoritative for the `~/.omega/` runtime layout and provider
> topology. For crates, orchestration, agent levels, and channels, see
> [ARCHITECTURE.md](ARCHITECTURE.md).

`~/.omega/` is OmegaOS's persistent home. It contains configuration,
OmegaOS-owned credential copies, registries, installed assets, logs, and state.
Provider CLIs can still require native files outside that directory; those
compatibility paths are coordinated explicitly rather than silently assumed to
be symlinks.

## Runtime layout

```text
~/.omega/
├── OMEGA.md                  universal agent instructions
├── config.toml               general runtime settings
├── providers.toml            typed per-provider settings
├── telegram.toml             Telegram configuration (secret, mode 0600)
├── projects.json             compatibility project-registry projection
├── credentials/
│   ├── <provider>.json       OmegaOS-owned active copies
│   ├── accounts/             named account profiles
│   └── quarantine/           conflicting or invalid Codex copies
├── rules/                    7 Laws plus 52 operational Rules
├── agents/                   shared prompts and 15 Matrix templates
├── skills/                   installed skill catalog
├── telegram-bot/             installed Bun/TypeScript bot runtime
├── inbox/                    operator file deposits
├── state/                    mission ledger, projections, locks, sessions
├── logs/                     operational logs
└── audit/                    audit results
```

The live inventories are authoritative:

```bash
omega rules list
omega agents
omega skills validate --root ~/.omega/skills
```

## Credential topology

- **Claude:** the installer can migrate
  `~/.claude/.credentials.json` into `~/.omega/credentials/claude.json` and
  leave the provider-compatible link.
- **Gemini:** the installer applies the equivalent migration for its OAuth
  credential file when present.
- **Codex:** `CODEX_HOME/auth.json` (default `~/.codex/auth.json`) remains the
  provider-native file. `omega codex-reconcile` validates and reconciles it
  with `~/.omega/credentials/codex.json` under a lock, preserving a valid newer
  copy and quarantining conflicts. Do not replace this protocol with a blind
  symlink or file copy.
- **API-key providers:** keys configured in `providers.toml` are injected only
  into the launched provider process. They must never be committed.

Back up `~/.omega/` with `omega backup`, but remember that a provider-native
home may also participate in its login topology. `omega doctor --deep` and
`omega codex-login-status` are the supported diagnostics.

## Provider catalog

The built-in default provider is Codex. Default models at this revision are:

| Provider | Default model | Authentication |
|---|---|---|
| Claude | `opus` | OAuth or API key |
| Codex | `gpt-5.6` | ChatGPT device auth or API key |
| Gemini | `auto` | Enterprise/API-key Gemini CLI |
| Antigravity | native account default | Google sign-in/keyring |
| GLM | `glm-5.3` | API key |
| OpenRouter | `anthropic/claude-opus-5` | API key |
| Pi | `anthropic/claude-opus-5` | OpenRouter configuration |
| Hermes | `anthropic/claude-opus-5` | API key |
| Kimi | `kimi-for-coding` | OAuth or API key |
| Shell | none | local process |

Run `omega agents`, use the TUI Settings tab, or inspect
`crates/omega-core/src/providers.rs` for the current catalog. Do not copy a
static model list into automation.

`providers.toml` contains typed provider tables, for example:

```toml
[claude]
model = "opus"

[codex]
model = "gpt-5.6"

[gemini]
model = "auto"
```

The global default selected through Telegram `/model` or
`omega config activate <provider> [model]` is mirrored under
`~/.omega/state/active-model.json`. `config.toml:agent_command` remains the
launch authority.

## Safe migration and verification

The installer reconciles Claude credentials with freshness-aware typed code and
delegates Codex conflicts to its flow-aware reconciler. Gemini 0.56+ and
Antigravity retain ownership of their native keyring/hybrid credential stores.
After install or provider login:

```bash
omega sync
omega codex-reconcile --json
omega codex-login-status
omega doctor --deep
```

A topology check is not proof that a credential can authenticate. The provider
login command or a real provider request is the final runtime check. Never print
credential contents while diagnosing the topology.
