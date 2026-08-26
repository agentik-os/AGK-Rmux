use crate::providers::ProvidersConfig;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// A provider launch split into non-secret shell text and structured process
/// environment. Credentials must travel through rmux's environment field,
/// never through command text (which can be exposed in argv or scrollback).
#[derive(Clone, PartialEq, Eq)]
pub struct AgentLaunch {
    command: String,
    environment: Vec<(String, String)>,
}

impl AgentLaunch {
    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn environment(&self) -> impl Iterator<Item = (&str, &str)> {
        self.environment
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }

    pub fn into_parts(self) -> (String, Vec<(String, String)>) {
        (self.command, self.environment)
    }
}

impl std::fmt::Debug for AgentLaunch {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AgentLaunch")
            .field("command", &"<redacted>")
            .field("command_bytes", &self.command.len())
            .field(
                "environment_keys",
                &self
                    .environment
                    .iter()
                    .map(|(key, _)| key)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

/// Options for launching an agent — used by the Master AISB and other
/// callers that need to inject a hidden system prompt or continue an
/// existing conversation.
#[derive(Debug, Clone, Default)]
pub struct LaunchOptions {
    /// Path to a markdown file injected via `--append-system-prompt-file`.
    /// The contents are hidden from the chat (not shown as a user message).
    pub system_prompt_file: Option<String>,
    /// Resume the most recent conversation in the agent's CWD (Claude `--continue`).
    pub resume_conversation: bool,

    // ── Claude-only smart features (2026-w20+) ────────────────────────
    // Other providers (Gemini, Antigravity, Codex, GLM, Pi, Hermes) ignore these
    // fields silently because their CLIs don't have equivalents. We
    // pass them only when Agent::Claude.
    /// `/goal` condition (v2.1.139+) — Claude auto-loops until this
    /// is met. Injected as the first slash command in the initial
    /// prompt. Example: "all tests in tests/auth pass and lint is clean".
    pub goal_condition: Option<String>,

    /// `--effort low|medium|high|xhigh|max` — model reasoning depth.
    /// We map (see dispatch.rs): SIMPLE→high, MEDIUM→xhigh, COMPLEX→xhigh, EPIC→max.
    pub effort: Option<String>,

    /// `--model <name>` — explicit model pin (e.g. "claude-opus-4-8"). When
    /// set, we emit `--model <name>` so the spawned session never silently
    /// drifts onto the CLI's default model. Claude-only.
    pub model: Option<String>,

    /// Requested turn cap for headless execution. Claude Code only supports
    /// this with `--print`, so interactive rmux launches deliberately omit it.
    pub max_turns: Option<u32>,

    /// Requested spend cap for headless execution. Interactive rmux launches
    /// deliberately omit this print-only Claude flag.
    pub max_budget_usd: Option<f32>,

    /// `--name <name>` — deterministic session label for resume.
    pub session_name: Option<String>,

    // ── Interactive-safe Claude flags (Lane A only) ────────────────────
    // These all keep the TTY attachable (rmux pane). Emitted ONLY when
    // set, in the Agent::Claude arm. Headless-only flags (stream-json /
    // --print / --input-format / --include-partial-messages) are NOT here —
    // they live on Lane B (omega-gateway/chat_driver.rs) where there is no
    // human attach.
    /// `--session-id <uuid>` — deterministic session id for resume/dedupe.
    /// Must be a valid UUID; we generate+persist one per oracle in ~/.omega/state.
    pub session_id: Option<String>,
    /// `--fork-session` — on resume, fork to a NEW id instead of mutating the
    /// original (use with resume_conversation/--continue). Interactive-safe.
    pub fork_session: bool,
    /// `--permission-mode <mode>` — per-role: `plan` (oracle planning),
    /// `acceptEdits`, `auto`, `manual`, or `dontAsk`. `bypassPermissions` is
    /// accepted only with the explicit provider high-risk opt-in.
    pub permission_mode: Option<String>,
    /// `--allowedTools` — comma/space list (e.g. "Bash(git *) Edit Read").
    pub allowed_tools: Option<String>,
    /// `--disallowedTools` — comma/space deny list.
    pub disallowed_tools: Option<String>,
    /// `--mcp-config <path...>` — JSON file(s) wiring OmegaOS tools as MCP
    /// servers. Binaries resolve under ~/.omega/bin; config under ~/.omega.
    pub mcp_config: Option<Vec<String>>,
    /// `--strict-mcp-config` — ONLY use servers from mcp_config, ignore
    /// user/project .mcp.json. Pair with mcp_config for hermetic worker roles.
    pub strict_mcp_config: bool,
    /// `--debug-file <path>` — debug log to ~/.omega/state/<session>.debug.log
    /// (implicitly enables debug mode). Interactive-safe (writes a file, keeps TTY).
    pub debug_file: Option<String>,
    /// `--verbose` — override verbose config. Interactive-safe on Lane A.
    pub verbose: bool,
    /// `--exclude-dynamic-system-prompt-sections` — move per-machine sections
    /// out of the system prompt into the first user message for cross-session
    /// prompt-cache reuse. Ignored when --system-prompt is set; we use
    /// --append-system-prompt-file so the default prompt still applies → SAFE.
    pub exclude_dynamic_prompt_sections: bool,
    /// `--brief` — enable the SendUserMessage agent→user tool. Interactive-safe;
    /// useful for oracles to push a structured note to the human.
    pub brief: bool,
    /// `--bare` — minimal mode (skip hooks/LSP/plugin-sync/CLAUDE.md auto-discovery,
    /// sets CLAUDE_CODE_SIMPLE=1). Interactive-safe BUT changes auth to API-key-only
    /// and disables CLAUDE.md autodiscovery → only for hermetic/perf worker roles,
    /// never the default oracle. Off by default.
    pub bare: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Agent {
    Claude,
    Codex,
    Gemini,
    Antigravity,
    Pi,
    OpenRouter,
    Hermes,
    Glm,
    Kimi,
    Shell,
}

impl Agent {
    pub fn all() -> &'static [Agent] {
        &[
            Agent::Claude,
            Agent::Codex,
            Agent::Gemini,
            Agent::Antigravity,
            Agent::Pi,
            Agent::OpenRouter,
            Agent::Hermes,
            Agent::Glm,
            Agent::Kimi,
            Agent::Shell,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Agent::Claude => "claude",
            Agent::Codex => "codex",
            Agent::Gemini => "gemini",
            Agent::Antigravity => "antigravity",
            Agent::Pi => "pi",
            Agent::OpenRouter => "openrouter",
            Agent::Hermes => "hermes",
            Agent::Glm => "glm",
            Agent::Kimi => "kimi",
            Agent::Shell => "shell",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Agent::Claude => "Claude Code (Anthropic)",
            Agent::Codex => "Codex (OpenAI)",
            Agent::Gemini => "Gemini (Google)",
            Agent::Antigravity => "Antigravity (Google)",
            Agent::Pi => "Pi (earendil-works)",
            Agent::OpenRouter => "OpenRouter",
            Agent::Hermes => "Hermes (Nous Research)",
            Agent::Glm => "GLM (Z.AI / Zhipu)",
            Agent::Kimi => "Kimi (Moonshot AI)",
            Agent::Shell => "Plain shell",
        }
    }

    /// Executable used by this adapter. GLM intentionally shares Claude Code;
    /// Antigravity's product/provider name differs from its `agy` binary.
    pub fn binary_name(&self) -> &'static str {
        match self {
            Agent::Claude | Agent::Glm => "claude",
            Agent::Codex => "codex",
            Agent::Gemini => "gemini",
            Agent::Antigravity => "agy",
            Agent::Pi | Agent::OpenRouter => "pi",
            Agent::Hermes => "hermes",
            Agent::Kimi => "kimi",
            Agent::Shell => "bash",
        }
    }

    /// Writers that may own a detached worker or an oracle mission.
    /// Hermes is Home (`omega new --agent hermes`) and is never a writer.
    pub fn is_writer(self) -> bool {
        matches!(self, Agent::Claude | Agent::Codex | Agent::Glm)
    }

    /// Map a configured Home/shell provider onto a writer. Used when the
    /// global `agent_command` is Hermes but a worker/oracle still needs a
    /// coding agent.
    pub fn writer_or_codex(self) -> Self {
        if self.is_writer() {
            self
        } else {
            Agent::Codex
        }
    }

    pub fn from_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Some(Agent::Claude),
            "codex" => Some(Agent::Codex),
            "gemini" => Some(Agent::Gemini),
            "antigravity" | "agy" => Some(Agent::Antigravity),
            "pi" => Some(Agent::Pi),
            "openrouter" => Some(Agent::OpenRouter),
            "hermes" => Some(Agent::Hermes),
            "glm" => Some(Agent::Glm),
            "kimi" => Some(Agent::Kimi),
            "shell" | "bash" => Some(Agent::Shell),
            _ => None,
        }
    }

    /// Official one-line installer command for this agent, or None if it
    /// comes pre-installed / not installable via a script.
    pub fn install_command(&self) -> Option<&'static str> {
        match self {
            // GLM (Z.AI/Zhipu) has NO standalone CLI. The official method is to run
            // Claude Code pointed at Z.AI's Anthropic-compatible endpoint. So
            // "installing GLM" = ensuring the Claude Code binary is present; the
            // redirect happens at launch (see launch_command_with) + the key.
            // npm-package CLIs: use npm when present, else fall back to bun
            // (OmegaOS installs bun; reference it by absolute path so a
            // non-interactive `bash -c` pane finds it even when ~/.bun/bin isn't
            // on PATH). A node-less box was why these "Enter → install" actions
            // appeared to do nothing.
            Agent::Glm => Some(
                "if command -v npm >/dev/null 2>&1; then mkdir -p \"$HOME/.npm-global\" && npm install -g --prefix \"$HOME/.npm-global\" @anthropic-ai/claude-code; elif [ -x \"$HOME/.bun/bin/bun\" ]; then \"$HOME/.bun/bin/bun\" add -g @anthropic-ai/claude-code; else echo 'Need Node.js or bun first (run: curl -fsSL https://bun.sh/install | bash)'; exit 1; fi",
            ),
            Agent::Claude => Some(
                "T=$(mktemp) || exit $?; curl -fsSL https://claude.ai/install.sh -o \"$T\" && bash \"$T\"; R=$?; rm -f \"$T\"; exit $R",
            ),
            // Official standalone installer (same shape as Claude's above), NOT
            // `npm i -g @openai/codex`: the npm build lacks the managed standalone
            // package at ~/.codex/packages/standalone that `codex remote-control`
            // requires, so an npm-installed Codex cannot be driven from the phone.
            Agent::Codex => Some(
                "T=$(mktemp) || exit $?; curl -fsSL https://chatgpt.com/codex/install.sh -o \"$T\" && CODEX_NON_INTERACTIVE=1 sh \"$T\"; R=$?; rm -f \"$T\"; exit $R",
            ),
            Agent::Gemini => Some(
                "if command -v npm >/dev/null 2>&1; then mkdir -p \"$HOME/.npm-global\" && npm install -g --prefix \"$HOME/.npm-global\" @google/gemini-cli; elif [ -x \"$HOME/.bun/bin/bun\" ]; then \"$HOME/.bun/bin/bun\" add -g @google/gemini-cli; else echo 'Need Node.js or bun first (run: curl -fsSL https://bun.sh/install | bash)'; exit 1; fi",
            ),
            Agent::Antigravity => Some(
                "T=$(mktemp) || exit $?; curl -fsSL https://antigravity.google/cli/install.sh -o \"$T\" && bash \"$T\"; R=$?; rm -f \"$T\"; exit $R",
            ),
            // Pi: install the npm package directly (the curl|sh installer runs a
            // TTY animation that fails in a non-interactive pane → `pi` never landed).
            Agent::Pi => Some(
                "if command -v npm >/dev/null 2>&1; then mkdir -p \"$HOME/.npm-global\" && npm install -g --prefix \"$HOME/.npm-global\" @earendil-works/pi-coding-agent; elif [ -x \"$HOME/.bun/bin/bun\" ]; then \"$HOME/.bun/bin/bun\" add -g @earendil-works/pi-coding-agent; else echo 'Need Node.js or bun first (run: curl -fsSL https://bun.sh/install | bash)'; exit 1; fi",
            ),
            Agent::OpenRouter => Some(
                "if command -v npm >/dev/null 2>&1; then mkdir -p \"$HOME/.npm-global\" && npm install -g --prefix \"$HOME/.npm-global\" @earendil-works/pi-coding-agent; elif [ -x \"$HOME/.bun/bin/bun\" ]; then \"$HOME/.bun/bin/bun\" add -g @earendil-works/pi-coding-agent; else echo 'Need Node.js or bun first (run: curl -fsSL https://bun.sh/install | bash)'; exit 1; fi",
            ),
            Agent::Hermes => Some(
                "T=$(mktemp) || exit $?; curl -fsSL https://hermes-agent.nousresearch.com/install.sh -o \"$T\" && CI=1 bash \"$T\" --skip-setup --skip-browser --skip-computer-use --non-interactive; R=$?; rm -f \"$T\"; exit $R",
            ),
            Agent::Kimi => Some(
                "T=$(mktemp) && curl -fsSL https://code.kimi.com/kimi-code/install.sh -o \"$T\" && bash \"$T\"; R=$?; rm -f \"$T\"; exit $R",
            ),
            Agent::Shell => None,
        }
    }

    /// Best-effort uninstall command for an agent. Documents how to
    /// remove the binary from the user's PATH. Not all agents have a
    /// turnkey uninstaller, so this is informational + best-effort.
    pub fn uninstall_command(&self) -> Option<&'static str> {
        match self {
            // Binary ONLY. NEVER `rm -rf ~/.claude` — that directory holds the
            // user's agents, slash commands, settings.json and credentials (and
            // OmegaOS itself installs into it). Uninstalling the CLI must not wipe
            // the user's whole config. Config can be removed by hand if desired.
            Agent::Claude => Some("rm -f \"$(command -v claude)\""),
            // Standalone install (see install_command): drop the binary + the
            // managed package tree, keep ~/.codex (auth + config), same as Claude.
            Agent::Codex => {
                Some("rm -f \"$(command -v codex)\" && rm -rf \"$HOME/.codex/packages/standalone\"")
            }
            Agent::Gemini => {
                Some("npm uninstall -g --prefix \"$HOME/.npm-global\" @google/gemini-cli")
            }
            // Keep ~/.gemini/antigravity-cli and keyring credentials intact.
            Agent::Antigravity => Some("rm -f \"$(command -v agy)\""),
            // Remove only the package; preserve native auth/config/session data.
            Agent::Pi => Some(
                "npm uninstall -g --prefix \"$HOME/.npm-global\" @earendil-works/pi-coding-agent",
            ),
            // Shares the Pi binary; removing it here would break Pi sessions.
            Agent::OpenRouter => None,
            // Upstream uninstaller handles venv/FHS binaries and gateway
            // services while preserving ~/.hermes unless the user requests a
            // full wipe inside Hermes itself.
            Agent::Hermes => Some("hermes uninstall"),
            // GLM shares the Claude Code binary — there is nothing GLM-specific to
            // uninstall. Removing it would wrongly delete the user's Claude Code.
            Agent::Glm => None,
            Agent::Kimi => Some("rm -f \"$(command -v kimi)\""),
            Agent::Shell => None,
        }
    }

    /// URL of the project homepage (shown in Settings).
    pub fn homepage(&self) -> Option<&'static str> {
        match self {
            Agent::Claude => Some("https://claude.ai/code"),
            Agent::Codex => Some("https://github.com/openai/codex"),
            Agent::Gemini => Some("https://github.com/google-gemini/gemini-cli"),
            Agent::Antigravity => Some("https://antigravity.google/docs/cli/overview/"),
            Agent::Pi => Some("https://pi.dev/"),
            Agent::OpenRouter => Some("https://openrouter.ai/"),
            Agent::Hermes => Some("https://hermes-agent.nousresearch.com/"),
            Agent::Glm => Some("https://www.z.ai/"),
            Agent::Kimi => Some("https://www.kimi.com/code/docs/en/kimi-code-cli/"),
            Agent::Shell => None,
        }
    }

    /// Returns the shell command to launch this agent.
    /// `initial_prompt` is the first message sent to the agent (if it supports it).
    pub fn launch_command(&self, initial_prompt: Option<&str>) -> String {
        self.launch_command_with(initial_prompt, LaunchOptions::default())
    }

    /// Strict launch builder used by session creation. Provider config errors
    /// are propagated so a corrupt secrets file can never silently launch a
    /// different/default provider profile.
    pub fn try_launch_command(&self, initial_prompt: Option<&str>) -> Result<String> {
        self.try_launch(initial_prompt).map(|launch| launch.command)
    }

    /// Build a complete typed launch. Callers that execute the command must
    /// also pass its environment to rmux atomically.
    pub fn try_launch(&self, initial_prompt: Option<&str>) -> Result<AgentLaunch> {
        self.try_launch_with(initial_prompt, LaunchOptions::default())
    }

    /// Provider-scoped environment. Values deliberately never enter the shell
    /// command string, its argv, logs, or terminal scrollback.
    fn provider_environment(&self, cfg: &ProvidersConfig) -> Vec<(String, String)> {
        // The full env map from providers.toml (ANTHROPIC_API_KEY, OPENAI_*,
        // GLM_API_KEY, OPENROUTER_*, …). We pick only the keys relevant to the
        // launched provider so a Codex pane never sees GLM's token, etc.
        let all = cfg.env_vars();
        let pick = |keys: &[&str]| -> Vec<(String, String)> {
            let mut selected = Vec::new();
            for (k, v) in &all {
                if keys.contains(&k.as_str()) {
                    selected.push((k.clone(), v.clone()));
                }
            }
            selected
        };
        match self {
            Agent::Claude => pick(&["ANTHROPIC_API_KEY"]),
            // Codex authenticates via `codex login` (~/.codex/auth.json). When the
            // user is on a ChatGPT session, injecting OPENAI_API_KEY OVERRIDES that
            // session and, if the key is wrong or out of quota, breaks every Codex
            // pane (401 / "quota exceeded"). So inject the API key only when there
            // is NO ChatGPT session to protect (API-key users still get their key).
            Agent::Codex => {
                if crate::codex_trust::is_chatgpt_session() {
                    Vec::new()
                } else {
                    pick(&["OPENAI_API_KEY", "OPENAI_BASE_URL"])
                }
            }
            // A lingering API key environment variable forces Gemini CLI away
            // from its cached OAuth account. Protect native OAuth the same way
            // Codex protects ChatGPT login from OPENAI_API_KEY overrides.
            Agent::Gemini => {
                if gemini_has_native_oauth() {
                    Vec::new()
                } else {
                    pick(&["GOOGLE_API_KEY", "GEMINI_API_KEY"])
                }
            }
            // Antigravity authenticates through its native keyring / Google
            // sign-in flow. Never leak Gemini API-key state into that session.
            Agent::Antigravity => Vec::new(),
            // GLM = Claude Code redirected to Z.AI. Supply the exact native
            // variable directly so no secret-bearing shell expansion is needed.
            Agent::Glm => {
                let mut selected = Vec::new();
                if !cfg.glm.api_key.is_empty() {
                    selected.push(("ANTHROPIC_AUTH_TOKEN".to_string(), cfg.glm.api_key.clone()));
                }
                selected.push((
                    "ANTHROPIC_BASE_URL".to_string(),
                    "https://api.z.ai/api/anthropic".to_string(),
                ));
                selected
            }
            // Pi is standalone. Only inject OpenRouter credentials when the
            // operator explicitly set `pi.provider = "openrouter"`.
            Agent::Pi => {
                if cfg.pi.provider.trim() == "openrouter" {
                    let mut s = pick(&["OPENROUTER_API_KEY", "OPENROUTER_BASE_URL"]);
                    if !cfg.pi.api_key.is_empty() {
                        s.retain(|(key, _)| key != "OPENROUTER_API_KEY");
                        s.push(("OPENROUTER_API_KEY".to_string(), cfg.pi.api_key.clone()));
                    }
                    s
                } else {
                    Vec::new()
                }
            }
            Agent::OpenRouter => pick(&["OPENROUTER_API_KEY", "OPENROUTER_BASE_URL"]),
            Agent::Hermes => {
                let mut s = pick(&["OPENROUTER_API_KEY", "OPENROUTER_BASE_URL"]);
                if !cfg.hermes.api_key.is_empty() {
                    s.retain(|(key, _)| key != "OPENROUTER_API_KEY");
                    s.push(("OPENROUTER_API_KEY".to_string(), cfg.hermes.api_key.clone()));
                }
                s
            }
            // Kimi CLI reads its Moonshot key from the environment when set.
            Agent::Kimi => pick(&[
                "KIMI_MODEL_NAME",
                "KIMI_MODEL_API_KEY",
                "KIMI_MODEL_PROVIDER_TYPE",
                "KIMI_MODEL_BASE_URL",
            ]),
            Agent::Shell => Vec::new(),
        }
    }

    /// Returns the shell command with options (system prompt file, continue, etc.).
    pub fn launch_command_with(&self, initial_prompt: Option<&str>, opts: LaunchOptions) -> String {
        self.try_launch_command_with(initial_prompt, opts)
            .unwrap_or_else(blocked_launch_command)
    }

    pub fn try_launch_command_with(
        &self,
        initial_prompt: Option<&str>,
        opts: LaunchOptions,
    ) -> Result<String> {
        self.try_launch_with(initial_prompt, opts)
            .map(|launch| launch.command)
    }

    pub fn try_launch_with(
        &self,
        initial_prompt: Option<&str>,
        opts: LaunchOptions,
    ) -> Result<AgentLaunch> {
        let providers = ProvidersConfig::try_load()
            .context("provider configuration is invalid; refusing agent launch")?;
        self.launch_with_providers(initial_prompt, opts, &providers)
    }

    #[cfg(test)]
    fn launch_command_with_providers(
        &self,
        initial_prompt: Option<&str>,
        opts: LaunchOptions,
        providers: &ProvidersConfig,
    ) -> Result<String> {
        self.launch_with_providers(initial_prompt, opts, providers)
            .map(|launch| launch.command)
    }

    pub(crate) fn launch_with_providers(
        &self,
        initial_prompt: Option<&str>,
        opts: LaunchOptions,
        providers: &ProvidersConfig,
    ) -> Result<AgentLaunch> {
        providers.validate()?;
        let environment = self.provider_environment(providers);
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        // PATH guard: panes launch via `bash -c`, which reads NO shell rc and
        // inherits the rmux daemon's (possibly stale) PATH — so `claude`/`bun`/
        // `omega` in ~/.local/bin or ~/.bun/bin can be "command not found", and a
        // dispatched oracle drops to a bare shell instead of running its mission.
        // Prepend the user bin dirs so every launched agent + tool always resolves.
        let path_prefix = format!(
            "{home}/.local/bin:{home}/.hermes/bin:{home}/.hermes/hermes-agent/venv/bin:{home}/.bun/bin:{home}/.npm-global/bin"
        );
        // Cursor (and other agent hosts) start the rmux daemon with
        // NO_COLOR=1 FORCE_COLOR=0. Every pane inherits that, and Claude /
        // Codex / Hermes then emit dim/bold only — no 38;2. Measured
        // 2026-08-25: a live Codex oracle had 62 SGR and 0 color codes;
        // the same Claude splash after FORCE_COLOR=1 emitted 68 truecolor
        // spans. `unset` is required: Node's supports-color treats any
        // *presence* of NO_COLOR as off, and rmux set-environment cannot
        // delete an OS-inherited variable from the daemon process.
        let env_prefix = format!(
            "export PATH={}:$PATH; unset NO_COLOR; export FORCE_COLOR=1 COLORTERM=truecolor; ",
            shell_quote(&path_prefix)
        );

        let command = match self {
            Agent::Claude => {
                // CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1: render in the normal
                // screen (not the alternate screen) so the full conversation
                // flows into rmux's scrollback and scrolls in the panel. Set
                // INLINE on the command because omega launches the agent via
                // `bash -c`, which reads neither ~/.zshenv nor ~/.bashrc, and
                // panes inherit the (older) rmux daemon env — so a shell-rc
                // export never reaches it.
                // The safe unattended default is Claude Code's native `auto`
                // policy. Blanket bypass is available only through the explicit
                // high-risk providers.toml switch.
                // Pre-trust the pane's cwd in ~/.claude.json IMMEDIATELY before
                // claude reads it (claude_trust.rs): with many concurrent
                // sessions the shared config is last-writer-wins, so an earlier
                // acceptance is routinely clobbered and the "trust this folder?"
                // dialog re-appears — hanging dispatched oracles. Best-effort:
                // an old omega binary without the subcommand just skips it.
                let trust_prefix = "omega trust-dir \"$PWD\" >/dev/null 2>&1; ";
                let permission_args = claude_permission_args(
                    opts.permission_mode.as_deref(),
                    providers.claude.dangerously_skip_permissions,
                )?;
                let mut args = format!(
                    "{}{}CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1 exec claude{}",
                    env_prefix, trust_prefix, permission_args
                );
                if let Some(ref sys_file) = opts.system_prompt_file {
                    args.push_str(&format!(
                        " --append-system-prompt-file {}",
                        shell_quote(sys_file)
                    ));
                }
                if opts.resume_conversation {
                    args.push_str(" --continue");
                }
                // Claude-only smart flags (2026-w20+). Silently ignored
                // by older Claude Code installs.
                if let Some(m) = opts
                    .model
                    .as_deref()
                    .or_else(|| nonempty(&providers.claude.model))
                {
                    args.push_str(&format!(" --model {}", shell_quote(m)));
                }
                if let Some(e) = opts
                    .effort
                    .as_deref()
                    .or_else(|| nonempty(&providers.claude.effort))
                {
                    args.push_str(&format!(" --effort {}", shell_quote(e)));
                }
                if opts.max_turns.is_some() || opts.max_budget_usd.is_some() {
                    tracing::warn!(
                        "interactive Claude launch omitted print-only max-turns/max-budget-usd flags"
                    );
                }
                if let Some(ref n) = opts.session_name {
                    args.push_str(&format!(" --name {}", shell_quote(n)));
                }
                // ── Interactive-safe flags (Lane A). All keep the TTY; emit
                //    ONLY when set. NONE of these are headless-only (no --print
                //    / --output-format / --input-format), so the rmux pane stays
                //    attachable. permission_mode is handled above (it replaces
                //    --dangerously-skip-permissions in the base command).
                if let Some(ref sid) = opts.session_id {
                    args.push_str(&format!(" --session-id {}", shell_quote(sid)));
                }
                if opts.fork_session {
                    args.push_str(" --fork-session");
                }
                if let Some(ref tools) = opts.allowed_tools {
                    args.push_str(&format!(" --allowedTools {}", shell_quote(tools)));
                }
                if let Some(ref tools) = opts.disallowed_tools {
                    args.push_str(&format!(" --disallowedTools {}", shell_quote(tools)));
                }
                if let Some(ref configs) = opts.mcp_config {
                    for cfg in configs {
                        args.push_str(&format!(" --mcp-config {}", shell_quote(cfg)));
                    }
                }
                if opts.strict_mcp_config {
                    args.push_str(" --strict-mcp-config");
                }
                if let Some(ref dbg) = opts.debug_file {
                    args.push_str(&format!(" --debug-file {}", shell_quote(dbg)));
                }
                if opts.verbose {
                    args.push_str(" --verbose");
                }
                if opts.exclude_dynamic_prompt_sections {
                    args.push_str(" --exclude-dynamic-system-prompt-sections");
                }
                if opts.brief {
                    args.push_str(" --brief");
                }
                if opts.bare {
                    args.push_str(" --bare");
                }
                // /goal is a slash command that consumes the ENTIRE first message
                // as its condition — so appending a large prompt after `/goal …`
                // blows Claude's ~4000-char /goal limit and the dispatch silently
                // aborts (the 27302-char bug: oracle dispatched but never launched).
                // Only inline /goal when the combined message stays under the cap;
                // otherwise ship the prompt alone. The oracle keeps its done.json
                // contract and, per R-GOAL, runs small goals inside dynamic
                // workflows rather than one giant /goal around the whole mission.
                const GOAL_MSG_MAX: usize = 4000;
                let final_prompt: Option<String> = match (&opts.goal_condition, initial_prompt) {
                    (Some(goal), Some(p)) => {
                        let combined = format!("/goal {}\n\n{}", goal, p);
                        if combined.chars().count() <= GOAL_MSG_MAX {
                            Some(combined)
                        } else {
                            tracing::warn!(
                                combined_len = combined.chars().count(),
                                max = GOAL_MSG_MAX,
                                "/goal + prompt exceeds /goal limit — shipping prompt without /goal (oracle uses done.json + in-workflow small goals)"
                            );
                            Some(p.to_string())
                        }
                    }
                    (Some(goal), None) => Some(format!("/goal {}", goal)),
                    (None, Some(p)) => Some(p.to_string()),
                    (None, None) => None,
                };
                match final_prompt {
                    Some(p) => pane_bash(&format!("{} {}", args, shell_quote(&p))),
                    None => pane_bash(&args),
                }
            }
            Agent::Codex => {
                // Same two guards Claude gets, for the same two reasons:
                //
                // (1) trust prefix — Codex blocks on "Do you trust the contents
                //     of this directory?" BEFORE rendering anything. A detached
                //     omega pane has nobody to press Enter, so the session looks
                //     dead. Pre-trust the cwd in ~/.codex/config.toml first
                //     (codex_trust.rs). Best-effort: an old omega binary without
                //     the subcommand just skips it and the prompt shows once.
                // (2) --no-alt-screen — render inline instead of on the alternate
                //     screen, so the conversation flows into rmux's scrollback and
                //     scrolls in the panel (Claude's CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1).
                //
                // COLORFGBG='15;0' (light-on-dark) instead of NO_COLOR. Codex's
                // composer paints a full-width RGB(30,30,30) band; NO_COLOR used
                // to strip it, but that also killed ALL syntax color and left
                // typed input barely visible — the operator wants color back.
                // COLORFGBG tells Codex the terminal is dark, so it renders a
                // light foreground ON its dark band (readable + colored) instead
                // of inheriting the outer palette and going black-on-black. The
                // dark band only mismatches a LIGHT outer terminal now; on the
                // dark rmux/omega TUI it blends in. Quoted so the ';' is one env
                // value, not a shell separator.
                let trust_prefix = "omega trust-dir \"$PWD\" >/dev/null 2>&1; ";
                // Codex >=0.147: `--approve-for-me` CONFLICTS with `--sandbox`
                // (CLI *or* ~/.codex/config.toml sandbox_mode). Live 0.149.1
                // dies with "`--sandbox` cannot be used with `--approve-for-me`"
                // and the pane used to fall through to bash. The valid
                // unattended pair is workspace-write + never-ask.
                let approval = if providers.codex.ask_for_approval_never {
                    "--sandbox workspace-write --ask-for-approval never"
                } else {
                    "--sandbox workspace-write --ask-for-approval on-request"
                };
                let mut args = format!(
                    "{}{}COLORFGBG='15;0' exec codex --strict-config {}",
                    env_prefix, trust_prefix, approval
                );
                if providers.codex.bypass_hook_trust {
                    args.push_str(" --dangerously-bypass-hook-trust");
                }
                args.push_str(" --no-alt-screen");
                if let Some(model) = nonempty(&providers.codex.model) {
                    args.push_str(&format!(" --model {}", shell_quote(model)));
                }
                let omega_dir = crate::config::omega_dir();
                for writable in [omega_dir.join("state"), omega_dir.join("locks")] {
                    args.push_str(&format!(
                        " --add-dir {}",
                        shell_quote(&writable.to_string_lossy())
                    ));
                }
                for writable in &providers.codex.additional_writable_dirs {
                    args.push_str(&format!(" --add-dir {}", shell_quote(writable)));
                }
                if opts.resume_conversation {
                    args.push_str(" resume --last");
                }
                match initial_prompt {
                    Some(p) => pane_bash(&format!("{} -- {}", args, shell_quote(p))),
                    None => pane_bash(&args),
                }
            }
            Agent::Gemini => {
                let model_arg = nonempty(&providers.gemini.model)
                    .map(|model| format!(" --model {}", shell_quote(model)))
                    .unwrap_or_default();
                let resume_arg = if opts.resume_conversation {
                    " --resume latest"
                } else {
                    ""
                };
                let yolo_arg = if providers.gemini.yolo { " --yolo" } else { "" };
                match initial_prompt {
                    Some(p) => pane_bash(&format!(
                        "{}exec gemini{}{}{} --prompt-interactive {}",
                        env_prefix,
                        model_arg,
                        yolo_arg,
                        resume_arg,
                        shell_quote(p)
                    )),
                    None => pane_bash(&format!(
                        "{}exec gemini{}{}{}",
                        env_prefix, model_arg, yolo_arg, resume_arg
                    )),
                }
            }
            Agent::Antigravity => {
                let mut args = format!("{}exec agy", env_prefix);
                if providers.antigravity.dangerously_skip_permissions {
                    args.push_str(" --dangerously-skip-permissions");
                }
                if let Some(model) = nonempty(&providers.antigravity.model) {
                    args.push_str(&format!(" --model {}", shell_quote(model)));
                }
                if let Some(effort) = nonempty(&providers.antigravity.effort) {
                    args.push_str(&format!(" --effort {}", shell_quote(effort)));
                }
                if opts.resume_conversation {
                    args.push_str(" --continue");
                }
                match initial_prompt {
                    Some(prompt) => pane_bash(&format!(
                        "{} --prompt-interactive {}",
                        args,
                        shell_quote(prompt)
                    )),
                    None => pane_bash(&args),
                }
            }
            Agent::Pi => {
                // Standalone Pi. Empty provider/model → omit the flags so the
                // CLI uses its own default (Google, `pi --help`). Never pin
                // OpenRouter onto Pi; that lane is Agent::OpenRouter.
                let pi_args = pi_provider_model_flags(
                    providers.pi.provider.trim(),
                    providers.pi.model.trim(),
                );
                let resume_arg = if opts.resume_conversation {
                    " --continue"
                } else {
                    ""
                };
                // Official Pi CLI has no tool-yolo. `--approve` only skips
                // project-trust; document that, do not invent a bypass.
                let approve_arg = if providers.pi.approve {
                    " --approve"
                } else {
                    ""
                };
                match initial_prompt {
                    Some(p) => pane_bash(&format!(
                        "{}exec pi {}{}{} -- {}",
                        env_prefix,
                        pi_args,
                        approve_arg,
                        resume_arg,
                        shell_quote(p)
                    )),
                    None => pane_bash(&format!(
                        "{}exec pi {}{}{}",
                        env_prefix, pi_args, approve_arg, resume_arg
                    )),
                }
            }
            Agent::OpenRouter => {
                let model = if providers.openrouter.model.is_empty() {
                    ProvidersConfig::default_model("openrouter")
                } else {
                    providers.openrouter.model.as_str()
                };
                let resume_arg = if opts.resume_conversation {
                    " --continue"
                } else {
                    ""
                };
                let args = format!(
                    "--provider openrouter --model {}{}",
                    shell_quote(model),
                    resume_arg
                );
                match initial_prompt {
                    Some(prompt) => pane_bash(&format!(
                        "{}exec pi {} -- {}",
                        env_prefix,
                        args,
                        shell_quote(prompt)
                    )),
                    None => pane_bash(&format!("{}exec pi {}", env_prefix, args)),
                }
            }
            Agent::Hermes => {
                // Hermes requires the `chat` subcommand. A bare positional prompt
                // is an unrecognized argument and kills the pane. Keep sessions
                // interactive; never `-q` (one-shot exit).
                let hermes_provider = if !providers.hermes.provider.trim().is_empty() {
                    Some(providers.hermes.provider.trim())
                } else if !providers.hermes.api_key.is_empty()
                    || !providers.openrouter.api_key.is_empty()
                    || !providers.openrouter.base_url.is_empty()
                {
                    Some("openrouter")
                } else {
                    None
                };
                let provider_arg = hermes_provider
                    .map(|provider| format!(" --provider {}", shell_quote(provider)))
                    .unwrap_or_default();
                let hermes_args = if providers.hermes.model.is_empty() {
                    String::new()
                } else {
                    format!(" --model {}", shell_quote(&providers.hermes.model))
                };
                let resume_arg = if opts.resume_conversation {
                    " --continue"
                } else {
                    ""
                };
                // Home TUI. `--yolo` / HERMES_YOLO_MODE keep tool calls from
                // blocking a detached pane. Never `-q`: that is a one-shot
                // query that exits and used to drop the pane to bash.
                let yolo_arg = if providers.hermes.yolo { " --yolo" } else { "" };
                // Assignments MUST precede `exec`. `exec HERMES_YOLO_MODE=1 hermes`
                // is `exec` of a command named HERMES_YOLO_MODE=1 → pane dies.
                let yolo_env = if providers.hermes.yolo {
                    "HERMES_YOLO_MODE=1 "
                } else {
                    ""
                };
                let hermes_home_path =
                    crate::tenancy::hermes_launch_home(std::path::Path::new(&home));
                let hermes_home = format!(
                    "export HERMES_HOME={}; ",
                    shell_quote(&hermes_home_path.to_string_lossy())
                );
                let profile_flag = crate::tenancy::hermes_profile_flag()
                    .map(|id| format!(" -p {}", shell_quote(id)))
                    .unwrap_or_default();
                let sandbox = crate::tenancy::launch_scope().and_then(|scope| {
                    crate::tenancy::sandbox_exec_prefix(std::path::Path::new(&home), scope, |s| {
                        shell_quote(s)
                    })
                });
                // Unscoped / no-bwrap: `exec hermes chat` (Home TUI tests).
                // Scoped: `exec hermes -p <scope> chat`. Isolated scopes add
                // `exec bwrap … -- hermes -p <scope> chat` when bwrap exists.
                let exec_hermes = match sandbox {
                    Some(prefix) => format!("{prefix}hermes{profile_flag} chat"),
                    None => format!("hermes{profile_flag} chat"),
                };
                // Hermes chat has no positional prompt (unrecognized arguments →
                // exit). `-q` is a one-shot that also exits. Home/TUI panes stay
                // on interactive `chat`; callers inject the first message after
                // the TUI is up.
                pane_bash(&format!(
                    "{}{}{}exec {}{}{}{}{}",
                    env_prefix,
                    hermes_home,
                    yolo_env,
                    exec_hermes,
                    provider_arg,
                    hermes_args,
                    yolo_arg,
                    resume_arg
                ))
            }
            Agent::Glm => {
                // GLM (Z.AI/Zhipu) = Claude Code redirected to Z.AI's Anthropic-
                // compatible endpoint. The base URL is a constant; the auth token is
                // taken from the structured ANTHROPIC_AUTH_TOKEN environment.
                // The rmux request scopes the redirect to this pane without
                // exposing the credential in shell text.
                // (d) Pass --model when glm.model is configured.
                let model_arg = if providers.glm.model.is_empty() {
                    String::new()
                } else {
                    format!(" --model {}", shell_quote(&providers.glm.model))
                };
                // GLM IS the claude binary, so a detached GLM session hits the
                // GLM uses the Claude binary and therefore the same permission
                // policy: auto by default, explicit high-risk bypass only.
                let trust_prefix = "omega trust-dir \"$PWD\" >/dev/null 2>&1; ";
                let perms = claude_permission_args(
                    opts.permission_mode.as_deref(),
                    providers.glm.dangerously_skip_permissions,
                )?;
                let resume_arg = if opts.resume_conversation {
                    " --continue"
                } else {
                    ""
                };
                match initial_prompt {
                    Some(p) => pane_bash(&format!(
                        "{} {}CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1 exec claude{}{}{} {}",
                        env_prefix,
                        trust_prefix,
                        perms,
                        model_arg,
                        resume_arg,
                        shell_quote(p)
                    )),
                    None => pane_bash(&format!(
                        "{} {}CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1 exec claude{}{}{}",
                        env_prefix, trust_prefix, perms, model_arg, resume_arg
                    )),
                }
            }
            Agent::Kimi => {
                // Kimi Code's current CLI has no positional prompt. `--prompt`
                // is its documented one-shot lane; interactive sessions use
                // `--auto` so detached agents do not block on approval dialogs.
                let model_arg = if providers.kimi.api_key.is_empty() {
                    nonempty(&providers.kimi.model)
                        .map(|model| format!(" --model {}", shell_quote(model)))
                        .unwrap_or_default()
                } else {
                    // The complete KIMI_MODEL_* family above already selects the
                    // direct provider/model. `--model` would override that alias.
                    String::new()
                };
                let resume_arg = if opts.resume_conversation {
                    " --continue"
                } else {
                    ""
                };
                let auto_arg = if providers.kimi.auto { " --auto" } else { "" };
                match initial_prompt {
                    Some(p) => pane_bash(&format!(
                        "{}exec kimi{}{}{} --prompt {}",
                        env_prefix,
                        auto_arg,
                        model_arg,
                        resume_arg,
                        shell_quote(p)
                    )),
                    None => pane_bash(&format!(
                        "{}exec kimi{}{}{}",
                        env_prefix, auto_arg, model_arg, resume_arg
                    )),
                }
            }
            Agent::Shell => match initial_prompt {
                Some(p) => format!(
                    "bash -c {}",
                    shell_quote(&format!("echo {}; exec bash", shell_quote(p)))
                ),
                None => "bash".to_string(),
            },
        };
        Ok(AgentLaunch {
            command,
            environment,
        })
    }

    pub fn is_available(&self) -> bool {
        let home = std::env::var("HOME").unwrap_or_default();
        match self {
            // PATH alone is unreliable under a reduced PATH (systemd/cron/non-login
            // `bash -c` panes that omit ~/.local/bin) — fall back to canonical
            // install locations so doctor doesn't falsely report "claude not on PATH".
            Agent::Claude => claude_available(&home),
            Agent::Codex => {
                has_cmd("codex")
                    || std::path::Path::new(&format!("{}/.local/bin/codex", home)).exists()
                    || std::path::Path::new(&format!("{}/.npm-global/bin/codex", home)).exists()
            }
            Agent::Gemini => {
                has_cmd("gemini")
                    || std::path::Path::new(&format!("{}/.npm-global/bin/gemini", home)).exists()
            }
            Agent::Antigravity => {
                has_cmd("agy")
                    || std::path::Path::new(&format!("{}/.local/bin/agy", home)).exists()
                    || std::path::Path::new(&format!("{}/.gemini/antigravity-cli/bin/agy", home))
                        .exists()
            }
            Agent::Pi => {
                has_cmd("pi")
                    || std::path::Path::new(&format!("{}/.local/bin/pi", home)).exists()
                    || std::path::Path::new(&format!("{}/.npm-global/bin/pi", home)).exists()
            }
            Agent::OpenRouter => {
                has_cmd("pi")
                    || std::path::Path::new(&format!("{}/.local/bin/pi", home)).exists()
                    || std::path::Path::new(&format!("{}/.npm-global/bin/pi", home)).exists()
            }
            Agent::Hermes => {
                has_cmd("hermes")
                    || std::path::Path::new(&format!("{}/.local/bin/hermes", home)).exists()
                    || std::path::Path::new(&format!("{}/.hermes/bin/hermes", home)).exists()
            }
            Agent::Kimi => {
                has_cmd("kimi")
                    || std::path::Path::new(&format!("{}/.local/bin/kimi", home)).exists()
            }
            // GLM runs on the Claude Code binary (redirected at launch) — available
            // iff `claude` is present. Shares Claude's reduced-PATH fallback so it
            // resolves the canonical install locations too, not just $PATH.
            Agent::Glm => claude_available(&home),
            Agent::Shell => true,
        }
    }
}

fn nonempty(value: &str) -> Option<&str> {
    (!value.trim().is_empty()).then_some(value)
}

/// Pi / OpenRouter share the `pi` binary, but the flags must not leak
/// across. Empty provider or model is omitted (Pi's native defaults).
fn pi_provider_model_flags(provider: &str, model: &str) -> String {
    let mut args = String::new();
    if !provider.is_empty() {
        args.push_str(&format!(" --provider {}", shell_quote(provider)));
    }
    if !model.is_empty() {
        args.push_str(&format!(" --model {}", shell_quote(model)));
    }
    args
}

fn claude_permission_args(requested: Option<&str>, explicit_bypass: bool) -> Result<String> {
    let mode = requested.unwrap_or(if explicit_bypass {
        "bypassPermissions"
    } else {
        "auto"
    });
    match mode {
        "acceptEdits" | "auto" | "manual" | "dontAsk" | "plan" => {
            Ok(format!(" --permission-mode {}", shell_quote(mode)))
        }
        "bypassPermissions" if explicit_bypass => {
            Ok(" --dangerously-skip-permissions".to_string())
        }
        "bypassPermissions" => anyhow::bail!(
            "bypassPermissions requires the explicit dangerously_skip_permissions provider setting"
        ),
        other => anyhow::bail!(
            "unsupported Claude permission mode {other:?}; expected acceptEdits, auto, manual, dontAsk, plan, or explicitly-enabled bypassPermissions"
        ),
    }
}

fn blocked_launch_command(error: anyhow::Error) -> String {
    let message = format!("OmegaOS refused agent launch: {error:#}");
    format!(
        "bash -c {}",
        shell_quote(&format!(
            "printf '%s\\n' {} >&2; exit 78",
            shell_quote(&message)
        ))
    )
}

fn has_cmd(name: &str) -> bool {
    let path = std::env::var("PATH").unwrap_or_default();
    path.split(':')
        .any(|dir| std::path::Path::new(dir).join(name).exists())
}

/// True iff the Claude Code binary is reachable — on $PATH, or at a canonical
/// install location the official installer uses. The location fallback keeps
/// detection correct when the caller's PATH is reduced (systemd, cron, non-login
/// `bash -c` panes) and omits ~/.local/bin.
fn claude_available(home: &str) -> bool {
    has_cmd("claude")
        || std::path::Path::new(&format!("{}/.local/bin/claude", home)).exists()
        || std::path::Path::new(&format!("{}/.claude/local/claude", home)).exists()
        || std::path::Path::new(&format!("{}/.npm-global/bin/claude", home)).exists()
}

fn gemini_settings_select_oauth(raw: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(raw)
        .ok()
        .and_then(|value| {
            value
                .pointer("/security/auth/selectedType")
                .or_else(|| value.get("selectedAuthType"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_ascii_lowercase)
        })
        .is_some_and(|selected| selected.starts_with("oauth") || selected == "login-with-google")
}

fn gemini_has_native_oauth() -> bool {
    let Some(home) = dirs::home_dir() else {
        return false;
    };
    let gemini_home = home.join(".gemini");
    if gemini_home.join("oauth_creds.json").is_file() {
        return true;
    }
    std::fs::read_to_string(gemini_home.join("settings.json"))
        .ok()
        .is_some_and(|raw| gemini_settings_select_oauth(&raw))
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// The pane process *is* the agent. Callers must put `exec` immediately
/// before the agent binary so a crash cannot fall through to bash.
/// Agent exit = session death. Never append `; exec bash` after the agent.
fn pane_bash(inner: &str) -> String {
    assert!(
        !inner.contains("; exec bash"),
        "agent launch must not fall through to bash: {inner}"
    );
    format!("bash -c {}", shell_quote(inner))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn launch(agent: Agent, prompt: Option<&str>, opts: LaunchOptions) -> String {
        agent
            .launch_command_with_providers(prompt, opts, &ProvidersConfig::default())
            .unwrap()
    }

    #[test]
    fn gemini_auth_selection_distinguishes_oauth_from_api_keys() {
        assert!(gemini_settings_select_oauth(
            r#"{"security":{"auth":{"selectedType":"oauth-personal"}}}"#
        ));
        assert!(gemini_settings_select_oauth(
            r#"{"selectedAuthType":"login-with-google"}"#
        ));
        assert!(!gemini_settings_select_oauth(
            r#"{"security":{"auth":{"selectedType":"gemini-api-key"}}}"#
        ));
    }

    // The worker/oracle identity contract: when LaunchOptions.session_name is
    // set, the generated Claude command MUST carry `--name <session>` so the
    // Claude conversation shares the rmux session's deterministic identity
    // (resumable via `claude --resume <name>`).
    #[test]
    fn launch_command_with_session_name_emits_name_flag() {
        let opts = LaunchOptions {
            session_name: Some("Verba-worker-fix-auth-401".to_string()),
            ..Default::default()
        };
        let cmd = launch(Agent::Claude, Some("do the thing"), opts);
        // The whole command is wrapped in an outer `bash -c '…'`, so the inner
        // shell_quote renders as '\'' — assert on flag + value, not exact quoting.
        assert!(
            cmd.contains(" --name ") && cmd.contains("Verba-worker-fix-auth-401"),
            "generated command missing --name: {cmd}"
        );
    }

    #[test]
    fn launch_command_without_session_name_has_no_name_flag() {
        let cmd = launch(
            Agent::Claude,
            Some("do the thing"),
            LaunchOptions::default(),
        );
        assert!(!cmd.contains(" --name "), "unexpected --name in: {cmd}");
    }

    // A detached GLM worker runs the claude binary: without pre-trust and a
    // non-blocking permission stance it hangs on dialogs nobody answers.
    #[test]
    fn glm_launch_is_dispatch_safe() {
        let launch = Agent::Glm
            .launch_with_providers(
                Some("do the thing"),
                LaunchOptions::default(),
                &ProvidersConfig::default(),
            )
            .unwrap();
        let cmd = launch.command();
        let environment = launch
            .environment()
            .collect::<std::collections::BTreeMap<_, _>>();
        assert!(
            cmd.contains("omega trust-dir")
                && cmd.contains("--permission-mode")
                && cmd.contains("auto")
                && !cmd.contains("--dangerously-skip-permissions")
                && cmd.contains("CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1")
                && environment.get("ANTHROPIC_BASE_URL") == Some(&"https://api.z.ai/api/anthropic"),
            "GLM launch must be dispatch-safe with a structured redirect: {launch:?}"
        );
    }

    #[test]
    fn glm_launch_honors_permission_mode() {
        let opts = LaunchOptions {
            permission_mode: Some("plan".to_string()),
            ..Default::default()
        };
        let cmd = launch(Agent::Glm, None, opts);
        assert!(
            cmd.contains("--permission-mode")
                && cmd.contains("plan")
                && !cmd.contains("--dangerously-skip-permissions"),
            "GLM must honor an explicit permission mode: {cmd}"
        );
    }

    #[test]
    fn codex_launch_keeps_color_but_stays_terminal_safe() {
        let cmd = launch(Agent::Codex, None, LaunchOptions::default());
        // Color is preserved (inherited NO_COLOR is unset; never NO_COLOR=1);
        // a dark-terminal hint keeps Codex's band readable (light-on-dark)
        // instead of black-on-black; inline render.
        // Never pair --sandbox with --approve-for-me (Codex 0.149 dies).
        assert!(
            cmd.contains("unset NO_COLOR")
                && !cmd.contains("NO_COLOR=1")
                && cmd.contains("FORCE_COLOR=1")
                && cmd.contains("COLORTERM=truecolor")
                && cmd.contains("COLORFGBG=")
                && cmd.contains("15;0")
                && cmd.contains("codex --strict-config")
                && cmd.contains("--sandbox")
                && cmd.contains("workspace-write")
                && cmd.contains("--ask-for-approval")
                && cmd.contains("never")
                && !cmd.contains("--approve-for-me")
                && !cmd.contains("; exec bash")
                && cmd.contains("COLORFGBG=")
                && cmd.contains(" exec codex ")
                && !cmd.contains("exec COLORFGBG=")
                && cmd.contains("--add-dir")
                && cmd.contains("--dangerously-bypass-hook-trust")
                && cmd.contains("--no-alt-screen"),
            "Codex launch must keep color and stay terminal-safe: {cmd}"
        );
    }

    #[test]
    fn home_launch_stays_alive_for_codex_claude_hermes() {
        for agent in [Agent::Codex, Agent::Claude, Agent::Hermes] {
            let cmd = launch(agent, None, LaunchOptions::default());
            assert!(
                !cmd.contains("; exec bash"),
                "{} Home launch must not fall through to bash: {cmd}",
                agent.name()
            );
            assert!(
                cmd.contains("exec "),
                "{} Home pane must exec the agent (same as TUI New {}): {cmd}",
                agent.name(),
                agent.display_name()
            );
            assert!(
                cmd.contains("bash -c "),
                "{} must share the pane_bash wrapper TUI uses: {cmd}",
                agent.name()
            );
        }
        let tui = launch(Agent::Codex, None, LaunchOptions::default());
        let cli = Agent::Codex.try_launch(None).unwrap().command().to_string();
        assert_eq!(
            tui, cli,
            "omega new --agent codex must use the same command as TUI New Codex"
        );
    }

    #[test]
    fn exec_does_not_swallow_environment_assignments() {
        // Live 2026-08-25: `exec VAR=value cmd` → bash: exec: VAR=value: not found
        // (exit 127). Every Home pane (Claude / Codex / Hermes) died on launch.
        for agent in [Agent::Claude, Agent::Codex, Agent::Hermes, Agent::Glm] {
            let cmd = launch(agent, None, LaunchOptions::default());
            assert!(
                !cmd.contains("exec CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN="),
                "bash exec cannot take env assignments: {cmd}"
            );
            assert!(
                !cmd.contains("exec COLORFGBG="),
                "bash exec cannot take env assignments: {cmd}"
            );
            assert!(
                !cmd.contains("exec HERMES_YOLO_MODE="),
                "bash exec cannot take env assignments: {cmd}"
            );
        }
        let claude = launch(Agent::Claude, None, LaunchOptions::default());
        assert!(
            claude.contains("CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1 exec claude"),
            "{claude}"
        );
        let hermes = launch(Agent::Hermes, None, LaunchOptions::default());
        assert!(
            hermes.contains("HERMES_YOLO_MODE=1 exec hermes chat"),
            "{hermes}"
        );
        assert!(hermes.contains(".hermes/bin"), "{hermes}");
        assert!(hermes.contains("HERMES_HOME="), "{hermes}");
        let hermes_prompt = launch(
            Agent::Hermes,
            Some("inspect the repository"),
            LaunchOptions::default(),
        );
        assert!(
            !hermes_prompt.contains("inspect the repository"),
            "hermes chat has no positional prompt (unrecognized arguments): {hermes_prompt}"
        );
    }

    #[test]
    fn pi_home_is_standalone_and_openrouter_is_its_own_lane() {
        let pi = launch(Agent::Pi, None, LaunchOptions::default());
        assert!(
            !pi.contains("--provider openrouter"),
            "Pi must not inherit the OpenRouter provider: {pi}"
        );
        assert!(
            !pi.contains("anthropic/claude-opus-5"),
            "Pi must not pin an OpenRouter model by default: {pi}"
        );
        let openrouter = launch(Agent::OpenRouter, None, LaunchOptions::default());
        assert!(
            openrouter.contains("--provider openrouter"),
            "OpenRouter Home must pin its own provider: {openrouter}"
        );
        assert!(
            Agent::OpenRouter.display_name() == "OpenRouter"
                && !Agent::OpenRouter.display_name().contains("via Pi"),
            "{}",
            Agent::OpenRouter.display_name()
        );
    }

    #[test]
    fn every_agent_pane_unsets_inherited_no_color() {
        // Live 2026-08-25: rmux daemon started from Cursor with NO_COLOR=1
        // FORCE_COLOR=0. Codex oracle capture had 62 SGR and 0 color codes.
        for agent in Agent::all()
            .iter()
            .copied()
            .filter(|agent| *agent != Agent::Shell)
        {
            let cmd = launch(agent, None, LaunchOptions::default());
            assert!(
                cmd.contains("unset NO_COLOR")
                    && cmd.contains("FORCE_COLOR=1")
                    && cmd.contains("COLORTERM=truecolor")
                    && !cmd.contains("NO_COLOR=1"),
                "{} must strip inherited NO_COLOR: {cmd}",
                agent.name()
            );
        }
    }

    #[test]
    fn agent_pane_is_the_agent_not_a_bash_fallback() {
        for agent in [
            Agent::Claude,
            Agent::Codex,
            Agent::Gemini,
            Agent::Antigravity,
            Agent::Pi,
            Agent::OpenRouter,
            Agent::Hermes,
            Agent::Glm,
            Agent::Kimi,
        ] {
            let cmd = launch(
                agent,
                Some("inspect the repository"),
                LaunchOptions::default(),
            );
            assert!(
                !cmd.contains("; exec bash"),
                "{} must not fall through to bash: {cmd}",
                agent.name()
            );
            assert!(
                cmd.contains("exec "),
                "{} pane must exec the agent: {cmd}",
                agent.name()
            );
        }
        let shell = launch(Agent::Shell, None, LaunchOptions::default());
        assert_eq!(shell, "bash");
    }

    #[test]
    fn claude_interactive_defaults_to_auto_and_omits_print_only_limits() {
        let opts = LaunchOptions {
            max_turns: Some(7),
            max_budget_usd: Some(1.25),
            ..Default::default()
        };
        let cmd = launch(Agent::Claude, None, opts);
        assert!(
            cmd.contains("--permission-mode") && cmd.contains("auto"),
            "{cmd}"
        );
        assert!(!cmd.contains("--max-turns"), "{cmd}");
        assert!(!cmd.contains("--max-budget-usd"), "{cmd}");
        assert!(!cmd.contains("dangerously-skip-permissions"), "{cmd}");
    }

    #[test]
    fn claude_bypass_requires_explicit_provider_opt_in() {
        let opts = LaunchOptions {
            permission_mode: Some("bypassPermissions".to_string()),
            ..Default::default()
        };
        let err = Agent::Claude
            .launch_command_with_providers(None, opts.clone(), &ProvidersConfig::default())
            .unwrap_err();
        assert!(err.to_string().contains("explicit"));

        let providers = ProvidersConfig {
            claude: crate::providers::ClaudeConfig {
                dangerously_skip_permissions: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let cmd = Agent::Claude
            .launch_command_with_providers(None, opts, &providers)
            .unwrap();
        assert!(cmd.contains("--dangerously-skip-permissions"), "{cmd}");
    }

    #[test]
    fn kimi_prompt_uses_implicit_auto_policy_and_model_override_contract() {
        let providers = ProvidersConfig {
            kimi: crate::providers::KimiConfig {
                model: "kimi-for-coding".to_string(),
                api_key: "key with ' quote; $(touch nope)".to_string(),
                base_url: "https://api.moonshot.ai/v1".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let launch = Agent::Kimi
            .launch_with_providers(
                Some("inspect; echo unsafe"),
                LaunchOptions::default(),
                &providers,
            )
            .unwrap();
        let cmd = launch.command();
        let env = launch
            .environment()
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(env.get("KIMI_MODEL_NAME"), Some(&"kimi-for-coding"));
        assert_eq!(
            env.get("KIMI_MODEL_API_KEY"),
            Some(&"key with ' quote; $(touch nope)")
        );
        assert!(!cmd.contains("KIMI_MODEL_NAME"), "{cmd}");
        assert!(!cmd.contains("KIMI_MODEL_API_KEY"), "{cmd}");
        assert!(!cmd.contains("key with ' quote; $(touch nope)"), "{cmd}");
        assert!(!cmd.contains("export KIMI_API_KEY="), "{cmd}");
        assert!(cmd.contains("--prompt"), "{cmd}");
        assert!(cmd.contains("--auto"), "{cmd}");
    }

    #[test]
    fn kimi_interactive_session_uses_auto_policy() {
        let cmd = launch(Agent::Kimi, None, LaunchOptions::default());
        assert!(
            cmd.contains("kimi --auto") || cmd.contains("kimi --auto") || cmd.contains("--auto"),
            "{cmd}"
        );
        assert!(!cmd.contains("--prompt"), "{cmd}");
    }

    #[test]
    fn hermes_home_stays_a_tui_and_never_uses_query_lane() {
        let providers = ProvidersConfig {
            hermes: crate::providers::HermesConfig {
                provider: "openrouter".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let cmd = Agent::Hermes
            .launch_command_with_providers(
                Some("inspect the repository"),
                LaunchOptions::default(),
                &providers,
            )
            .unwrap();
        assert!(cmd.contains("hermes chat --provider"), "{cmd}");
        assert!(cmd.contains("openrouter"), "{cmd}");
        assert!(cmd.contains("--yolo"), "{cmd}");
        assert!(cmd.contains("HERMES_YOLO_MODE=1"), "{cmd}");
        assert!(cmd.contains("HERMES_HOME="), "{cmd}");
        assert!(!cmd.contains(" -q "), "{cmd}");
        assert!(!cmd.contains("; exec bash"), "{cmd}");
    }

    #[test]
    fn hermes_install_is_non_interactive() {
        let cmd = Agent::Hermes.install_command().expect("hermes installer");
        assert!(cmd.contains("--skip-setup"), "{cmd}");
        assert!(cmd.contains("--skip-browser"), "{cmd}");
        assert!(cmd.contains("--skip-computer-use"), "{cmd}");
        assert!(cmd.contains("--non-interactive"), "{cmd}");
    }

    #[test]
    fn gemini_prompt_stays_in_an_interactive_session() {
        let cmd = launch(
            Agent::Gemini,
            Some("inspect the repository"),
            LaunchOptions::default(),
        );
        assert!(cmd.contains("--prompt-interactive"), "{cmd}");
        assert!(cmd.contains("--yolo"), "{cmd}");
        assert!(!cmd.contains("; exec bash"), "{cmd}");
    }

    #[test]
    fn antigravity_prompt_stays_interactive_and_autonomous() {
        let cmd = launch(
            Agent::Antigravity,
            Some("inspect the repository"),
            LaunchOptions::default(),
        );
        assert!(cmd.contains("agy --dangerously-skip-permissions"), "{cmd}");
        assert!(cmd.contains("--prompt-interactive"), "{cmd}");
        assert!(!cmd.contains(" -p "), "{cmd}");
    }

    #[test]
    fn patrol_resume_is_mapped_for_every_conversational_adapter() {
        let opts = LaunchOptions {
            resume_conversation: true,
            ..Default::default()
        };
        for (agent, expected) in [
            (Agent::Claude, "--continue"),
            (Agent::Codex, "resume --last"),
            (Agent::Gemini, "--resume latest"),
            (Agent::Antigravity, "--continue"),
            (Agent::Pi, "--continue"),
            (Agent::OpenRouter, "--continue"),
            (Agent::Hermes, "--continue"),
            (Agent::Glm, "--continue"),
            (Agent::Kimi, "--continue"),
        ] {
            let command = launch(agent, None, opts.clone());
            assert!(
                command.contains(expected),
                "{} resume mapping missing {expected}: {command}",
                agent.name()
            );
        }
    }

    #[test]
    fn provider_secrets_are_structured_and_debug_redacted() {
        let secret = "never-in-command-or-debug-42";
        let providers = ProvidersConfig {
            glm: crate::providers::GlmConfig {
                api_key: secret.to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let launch = Agent::Glm
            .launch_with_providers(None, LaunchOptions::default(), &providers)
            .unwrap();
        let env = launch
            .environment()
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(env.get("ANTHROPIC_AUTH_TOKEN"), Some(&secret));
        assert!(!launch.command().contains(secret));
        assert!(!format!("{launch:?}").contains(secret));
    }

    #[test]
    fn injection_shaped_values_round_trip_as_one_shell_argument() {
        let tmp = tempfile::tempdir().unwrap();
        let marker = tmp.path().join("injection-marker");
        let value = format!("model'; touch {}; echo '$HOME", marker.display());
        let output = std::process::Command::new("bash")
            .args(["-c", &format!("printf '%s' {}", shell_quote(&value))])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap(), value);
        assert!(!marker.exists());
    }
}
