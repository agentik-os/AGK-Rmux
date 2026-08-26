//! Provider configuration — typed, persistent, shared across all sessions.
//!
//! Saved to `~/.omega/providers.toml`. When OmegaOS spawns an agent session
//! (claude/codex/gemini/glm), the relevant env vars from this file are
//! injected so every session uses the same credentials and model defaults.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProvidersConfig {
    #[serde(default)]
    pub claude: ClaudeConfig,
    #[serde(default)]
    pub codex: CodexConfig,
    #[serde(default)]
    pub gemini: GeminiConfig,
    #[serde(default)]
    pub antigravity: AntigravityConfig,
    #[serde(default)]
    pub glm: GlmConfig,
    #[serde(default)]
    pub openrouter: OpenRouterConfig,
    #[serde(default)]
    pub pi: PiConfig,
    #[serde(default)]
    pub hermes: HermesConfig,
    #[serde(default)]
    pub kimi: KimiConfig,
    /// Exact authority-file revision captured at load time. This is not
    /// serialized and exists only to reject stale concurrent saves.
    #[doc(hidden)]
    #[serde(skip)]
    pub source_revision: Option<crate::config::PrivateRevision>,
}

impl fmt::Debug for ProvidersConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProvidersConfig")
            .field("claude_model", &self.claude.model)
            .field("claude_has_api_key", &!self.claude.api_key.is_empty())
            .field("codex_model", &self.codex.model)
            .field("codex_has_base_url", &!self.codex.base_url.is_empty())
            .field("codex_has_api_key", &!self.codex.api_key.is_empty())
            .field("gemini_model", &self.gemini.model)
            .field("gemini_has_api_key", &!self.gemini.api_key.is_empty())
            .field("antigravity_model", &self.antigravity.model)
            .field("antigravity_effort", &self.antigravity.effort)
            .field(
                "antigravity_dangerously_skip_permissions",
                &self.antigravity.dangerously_skip_permissions,
            )
            .field("glm_model", &self.glm.model)
            .field("glm_has_api_key", &!self.glm.api_key.is_empty())
            .field("openrouter_model", &self.openrouter.model)
            .field(
                "openrouter_has_base_url",
                &!self.openrouter.base_url.is_empty(),
            )
            .field(
                "openrouter_has_api_key",
                &!self.openrouter.api_key.is_empty(),
            )
            .field("pi_provider", &self.pi.provider)
            .field("pi_model", &self.pi.model)
            .field("pi_has_api_key", &!self.pi.api_key.is_empty())
            .field("hermes_model", &self.hermes.model)
            .field("hermes_has_api_key", &!self.hermes.api_key.is_empty())
            .field("kimi_model", &self.kimi.model)
            .field("kimi_has_base_url", &!self.kimi.base_url.is_empty())
            .field("kimi_provider_type", &self.kimi.provider_type)
            .field("kimi_has_api_key", &!self.kimi.api_key.is_empty())
            .finish()
    }
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PiConfig {
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub model: String,
    /// Optional extra key. Injected as OPENROUTER_API_KEY only when
    /// `pi.provider = "openrouter"`. Pi's own default provider is Google
    /// (`pi --help`); an empty provider leaves that default alone.
    #[serde(default)]
    pub api_key: String,
    // NOTE: a `pi.extension` field was removed (2026-06) — it was an orphan with
    // get/set arms but NO consumer anywhere in the launch path, and the Pi CLI
    // (pi.dev) exposes no documented `--extension` flag to wire it to. Per L2
    // (no fabricated confidence) we removed the dead field rather than invent a
    // consumer. `#[serde(default)]` on the struct means old providers.toml files
    // carrying an `extension = "..."` line still deserialize (the key is ignored).
    #[doc(hidden)]
    #[serde(default, rename = "extension", skip_serializing)]
    pub legacy_extension: Option<toml::Value>,
    /// Official Pi CLI has no tool-yolo. `--approve` only skips project-trust.
    #[serde(default = "default_true")]
    pub approve: bool,
}

impl Default for PiConfig {
    fn default() -> Self {
        Self {
            provider: String::new(),
            model: String::new(),
            api_key: String::new(),
            legacy_extension: None,
            approve: true,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HermesConfig {
    /// Hermes provider id. Empty means OpenRouter when an Omega-managed key
    /// is configured, otherwise Hermes' own native configuration decides.
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
    /// Home TUI: `--yolo` + `HERMES_YOLO_MODE=1`. Detached panes have nobody
    /// to click through tool approvals.
    #[serde(default = "default_true")]
    pub yolo: bool,
}

impl Default for HermesConfig {
    fn default() -> Self {
        Self {
            provider: String::new(),
            model: String::new(),
            api_key: String::new(),
            yolo: true,
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OpenRouterConfig {
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub base_url: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ClaudeConfig {
    /// "fable" | "opus" | "sonnet" | "haiku" | full model id like "claude-fable-5"
    #[serde(default)]
    pub model: String,
    /// "low" | "medium" | "high" | "max" — Claude effort level
    #[serde(default)]
    pub effort: String,
    /// Anthropic API key (only used if not via OAuth)
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub dangerously_skip_permissions: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CodexConfig {
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub base_url: String,
    /// Extra writable roots granted to Codex in addition to the project and
    /// OmegaOS's state/lock directories. Values must be absolute paths.
    #[serde(default)]
    pub additional_writable_dirs: Vec<String>,
    /// Run enabled hooks without persisted trust for Omega-managed detached
    /// sessions. This also trusts other enabled hooks, so operators can disable
    /// it when they prefer Codex's interactive review.
    pub bypass_hook_trust: bool,
    /// Unattended pair: `--sandbox workspace-write --ask-for-approval never`.
    /// Never combine `--sandbox` with `--approve-for-me` (Codex 0.149 dies).
    #[serde(default = "default_true")]
    pub ask_for_approval_never: bool,
}

impl Default for CodexConfig {
    fn default() -> Self {
        Self {
            model: String::new(),
            api_key: String::new(),
            base_url: String::new(),
            additional_writable_dirs: Vec::new(),
            bypass_hook_trust: true,
            ask_for_approval_never: true,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GeminiConfig {
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_true")]
    pub yolo: bool,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            model: String::new(),
            api_key: String::new(),
            yolo: true,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AntigravityConfig {
    /// Optional account-visible model slug from `agy models`.
    #[serde(default)]
    pub model: String,
    /// Optional reasoning effort: low, medium, or high.
    #[serde(default)]
    pub effort: String,
    /// Antigravity otherwise asks for tool approvals. Detached Omega sessions
    /// need an explicit, visible high-risk opt-in for full autonomy.
    #[serde(default)]
    pub dangerously_skip_permissions: bool,
}

impl Default for AntigravityConfig {
    fn default() -> Self {
        Self {
            model: String::new(),
            effort: String::new(),
            // An Omega-managed session may be detached and has no operator at
            // the approval prompt. Selecting this provider is the opt-in.
            dangerously_skip_permissions: true,
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GlmConfig {
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
    /// Explicit high-risk opt-in for the Claude-backed GLM adapter. The safe
    /// default is Claude Code's `auto` permission mode.
    #[serde(default)]
    pub dangerously_skip_permissions: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct KimiConfig {
    /// Kimi model id or configured alias. OAuth users may leave this empty and
    /// let Kimi Code use its own `default_model`.
    pub model: String,
    /// Direct API credential. Kimi Code only accepts this from its explicit
    /// `KIMI_MODEL_*` override channel, not from a plain `KIMI_API_KEY` export.
    pub api_key: String,
    pub base_url: String,
    pub provider_type: String,
    /// Force `--auto` on interactive and `--prompt` launches so detached
    /// panes do not block on approval dialogs.
    #[serde(default = "default_true")]
    pub auto: bool,
}

impl Default for KimiConfig {
    fn default() -> Self {
        Self {
            model: String::new(),
            api_key: String::new(),
            base_url: String::new(),
            provider_type: "kimi".to_string(),
            auto: true,
        }
    }
}

macro_rules! impl_redacted_provider_debug {
    ($type:ty, $name:literal, [$($field:ident),* $(,)?]) => {
        impl fmt::Debug for $type {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut debug = formatter.debug_struct($name);
                $(debug.field(stringify!($field), &self.$field);)*
                debug
                    .field("has_api_key", &!self.api_key.is_empty())
                    .finish()
            }
        }
    };
}

impl_redacted_provider_debug!(PiConfig, "PiConfig", [provider, model, approve]);
impl_redacted_provider_debug!(HermesConfig, "HermesConfig", [provider, model, yolo]);
impl_redacted_provider_debug!(OpenRouterConfig, "OpenRouterConfig", [model]);
impl_redacted_provider_debug!(
    ClaudeConfig,
    "ClaudeConfig",
    [model, effort, dangerously_skip_permissions]
);
impl_redacted_provider_debug!(
    CodexConfig,
    "CodexConfig",
    [
        model,
        additional_writable_dirs,
        bypass_hook_trust,
        ask_for_approval_never
    ]
);
impl_redacted_provider_debug!(GeminiConfig, "GeminiConfig", [model, yolo]);
impl fmt::Debug for AntigravityConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AntigravityConfig")
            .field("model", &self.model)
            .field("effort", &self.effort)
            .field(
                "dangerously_skip_permissions",
                &self.dangerously_skip_permissions,
            )
            .finish()
    }
}
impl_redacted_provider_debug!(
    GlmConfig,
    "GlmConfig",
    [model, dangerously_skip_permissions]
);
impl_redacted_provider_debug!(KimiConfig, "KimiConfig", [model, provider_type, auto]);

impl ProvidersConfig {
    pub fn path() -> PathBuf {
        crate::config::omega_dir().join("providers.toml")
    }

    /// Compatibility loader for read-only UI/diagnostic surfaces. Runtime
    /// authority must use [`Self::try_load`]. Mutations remain fail-closed in
    /// [`Self::save`], even if a legacy caller obtained this fallback value.
    pub fn load() -> Self {
        match Self::try_load() {
            Ok(config) => config,
            Err(error) => {
                eprintln!(
                    "omega: provider config unavailable; diagnostic defaults only, agent launch is blocked: {error:#}"
                );
                tracing::error!(error = %error, "provider config unavailable; diagnostic defaults only");
                Self::default()
            }
        }
    }

    /// Strict provider configuration loader for every launch/mutation path.
    pub fn try_load() -> Result<Self> {
        let path = Self::path();
        Self::load_from(&path)
    }

    fn load_from(path: &Path) -> Result<Self> {
        let Some(raw) = crate::config::read_private_optional_string(path)? else {
            return Ok(Self::default());
        };
        let mut config: Self = toml::from_str(&raw).map_err(|error: toml::de::Error| {
            crate::config::private_toml_error("provider config", path, error.span())
        })?;
        config
            .validate()
            .with_context(|| format!("validating provider config {}", path.display()))?;
        config.source_revision = Some(crate::config::private_revision(raw.as_bytes()));
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        self.save_to(&path)
    }

    fn save_to(&self, path: &Path) -> Result<()> {
        self.validate()
            .with_context(|| format!("validating provider config for {}", path.display()))?;
        let content = toml::to_string_pretty(self).context("serializing providers config")?;
        crate::config::with_private_file_lock(path, || {
            crate::config::require_private_revision(path, self.source_revision.as_ref())?;
            crate::config::atomic_write_private(path, content.as_bytes())
                .context("writing providers config")
        })
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if !self.antigravity.effort.is_empty()
            && !matches!(self.antigravity.effort.as_str(), "low" | "medium" | "high")
        {
            anyhow::bail!("invalid antigravity.effort; expected low, medium, or high");
        }
        match self.kimi.provider_type.as_str() {
            "kimi" | "anthropic" | "openai" => {}
            _ => anyhow::bail!("invalid kimi.provider_type; expected kimi, anthropic, or openai"),
        }
        for path in &self.codex.additional_writable_dirs {
            if path.chars().any(char::is_control) {
                anyhow::bail!("codex.additional_writable_dirs entry contains control characters");
            }
            let writable = Path::new(path);
            if !writable.is_absolute()
                || writable.parent().is_none()
                || writable
                    .components()
                    .any(|component| component == std::path::Component::ParentDir)
            {
                anyhow::bail!(
                    "codex.additional_writable_dirs entry must be absolute, non-root, and traversal-free: {path:?}"
                );
            }
        }
        Ok(())
    }

    /// The stored API key for a provider (empty string if none / unknown).
    pub fn api_key_for(&self, provider: &str) -> &str {
        match provider {
            "claude" => &self.claude.api_key,
            "codex" => &self.codex.api_key,
            "gemini" => &self.gemini.api_key,
            "antigravity" => "",
            "glm" => &self.glm.api_key,
            "openrouter" => &self.openrouter.api_key,
            "pi" => &self.pi.api_key,
            "hermes" => &self.hermes.api_key,
            "kimi" => &self.kimi.api_key,
            _ => "",
        }
    }

    /// Build the env vars that should be injected into every agent session.
    pub fn env_vars(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        if !self.claude.api_key.is_empty() {
            out.push(("ANTHROPIC_API_KEY".to_string(), self.claude.api_key.clone()));
        }
        if !self.codex.api_key.is_empty() {
            out.push(("OPENAI_API_KEY".to_string(), self.codex.api_key.clone()));
        }
        if !self.codex.base_url.is_empty() {
            out.push(("OPENAI_BASE_URL".to_string(), self.codex.base_url.clone()));
        }
        if !self.gemini.api_key.is_empty() {
            out.push(("GOOGLE_API_KEY".to_string(), self.gemini.api_key.clone()));
            out.push(("GEMINI_API_KEY".to_string(), self.gemini.api_key.clone()));
        }
        if !self.glm.api_key.is_empty() {
            out.push(("GLM_API_KEY".to_string(), self.glm.api_key.clone()));
        }
        if !self.openrouter.api_key.is_empty() {
            out.push((
                "OPENROUTER_API_KEY".to_string(),
                self.openrouter.api_key.clone(),
            ));
        }
        if !self.openrouter.base_url.is_empty() {
            out.push((
                "OPENROUTER_BASE_URL".to_string(),
                self.openrouter.base_url.clone(),
            ));
        }
        // Current Kimi Code deliberately ignores plain KIMI_API_KEY exports.
        // Its documented ephemeral-provider channel requires NAME + API_KEY;
        // only emit the complete pair so an incomplete override never bricks an
        // otherwise healthy OAuth session.
        if !self.kimi.api_key.is_empty() {
            let model = if self.kimi.model.is_empty() {
                Self::default_model("kimi")
            } else {
                &self.kimi.model
            };
            out.push(("KIMI_MODEL_NAME".to_string(), model.to_string()));
            out.push(("KIMI_MODEL_API_KEY".to_string(), self.kimi.api_key.clone()));
            out.push((
                "KIMI_MODEL_PROVIDER_TYPE".to_string(),
                self.kimi.provider_type.clone(),
            ));
            if !self.kimi.base_url.is_empty() {
                out.push((
                    "KIMI_MODEL_BASE_URL".to_string(),
                    self.kimi.base_url.clone(),
                ));
            }
        }
        out
    }

    // ───────── Provider catalog (static metadata) ─────────

    /// All known providers in canonical order. Static slice — safe to expose.
    pub fn all_providers() -> Vec<&'static str> {
        vec![
            "claude",
            "codex",
            "gemini",
            "antigravity",
            "glm",
            "openrouter",
            "pi",
            "hermes",
            "kimi",
            "shell",
        ]
    }

    pub fn default_provider() -> &'static str {
        "codex"
    }

    /// Default model id for a provider (used when /model <provider> has no
    /// model arg).
    pub fn default_model(provider: &str) -> &'static str {
        match provider {
            "claude" => "opus",
            // Let Codex resolve the current recommended GPT-5.6 variant for
            // the account while still exposing explicit Sol/Terra/Luna picks.
            "codex" => "gpt-5.6",
            // Gemini CLI's account-aware router is safer than pinning a preview
            // model an OAuth/API-key account may not be entitled to.
            "gemini" => "auto",
            "antigravity" => "auto",
            "glm" => "glm-5.3",
            // Operator directive 2026-07-24: Claude Opus 5 is THE default brain
            // everywhere a tier has not been deliberately pinned (R-MODEL).
            "openrouter" | "hermes" => "anthropic/claude-opus-5",
            // Empty = omit `--model` and let the Pi CLI pick (default provider
            // is Google). Do not pin an OpenRouter id onto a standalone Pi.
            "pi" => "",
            "kimi" => "kimi-for-coding",
            "shell" => "",
            _ => "",
        }
    }

    /// Available models for a provider (used to list options in /model UI).
    pub fn models_for(provider: &str) -> Vec<&'static str> {
        match provider {
            // "opus" is first = the default, and resolves to claude-opus-5[1m]
            // at dispatch time (dispatch::resolve_model_flag). The explicit
            // ids sit beside the aliases so a session can be pinned exactly.
            "claude" => vec![
                "opus",
                "claude-opus-5",
                "sonnet",
                "claude-sonnet-5",
                "haiku",
                "fable",
            ],
            "codex" => vec![
                "gpt-5.6",
                "gpt-5.6-sol",
                "gpt-5.6-terra",
                "gpt-5.6-luna",
                "gpt-5.5",
            ],
            // Prefer account-aware aliases and only list model ids accepted by
            // Gemini CLI 0.56 / the current Gemini API catalog.
            "gemini" => vec![
                "auto",
                "pro",
                "flash",
                "gemini-3.1-pro-preview",
                "gemini-3.6-flash",
                "gemini-3.5-flash",
                "gemini-3.1-flash-lite",
                "gemini-2.5-pro",
            ],
            // `agy models` is account-scoped and changes independently of
            // OmegaOS. Empty means UI callers offer a free-text field.
            "antigravity" => vec![],
            "glm" => vec!["glm-5.3", "glm-5-turbo", "glm-4.7"],
            // Pi is standalone (CLI default: Google). Empty catalog = free-text
            // / `pi auth`. OpenRouter models live on the OpenRouter picker.
            "pi" => vec![],
            // Hermes can pin OpenRouter; OpenRouter is its own provider.
            "hermes" | "openrouter" => vec![
                "anthropic/claude-opus-5",
                "anthropic/claude-sonnet-5",
                "anthropic/claude-sonnet-4.6",
                "anthropic/claude-opus-4.8",
                "openai/gpt-5.5",
                "google/gemini-3.1-pro-preview",
                "z-ai/glm-5.3",
                "deepseek/deepseek-chat",
                // Cloaked/stealth listing (added 2026-08-23): a reasoning model
                // aimed at coding and sustained agentic work — 1M context, 131k
                // max output, tools + vision, reasoning always on, and priced at
                // 0 for as long as the stealth window lasts. It sits LAST and is
                // deliberately NOT the default: a cloaked endpoint means an
                // undisclosed vendor sees the prompts, so it is an opt-in lane
                // (R-MODEL), and the slug disappears when it is un-cloaked.
                "stealth/ox-alpha",
            ],
            "kimi" => vec![
                "kimi-for-coding",
                "kimi-for-coding-highspeed",
                "k3",
                "k3-256k",
                "kimi-k2.6",
            ],
            "shell" => vec![],
            _ => vec![],
        }
    }

    /// Auth type for a provider: "oauth" | "api_key" | "config".
    pub fn auth_type(provider: &str) -> &'static str {
        match provider {
            "claude" | "gemini" | "antigravity" => "oauth",
            "codex" => "oauth_or_api_key",
            "glm" | "openrouter" | "pi" | "hermes" => "api_key",
            "kimi" => "oauth_or_api_key",
            "shell" => "local",
            _ => "unknown",
        }
    }

    /// Relative cred file path under ~/.omega/.
    pub fn cred_file(provider: &str) -> String {
        format!("credentials/{}.json", provider)
    }

    /// True if the provider exists in the catalog.
    pub fn is_known(provider: &str) -> bool {
        Self::all_providers().contains(&provider)
    }

    /// Static execution capabilities used by the router before a session is
    /// spawned. This describes the OmegaOS adapter, not a marketing claim made
    /// by a model vendor.
    pub fn capabilities_for(provider: &str) -> Option<BTreeSet<ProviderCapability>> {
        let capabilities = match provider {
            "claude" => &[
                ProviderCapability::Reasoning,
                ProviderCapability::CodeEditing,
                ProviderCapability::ToolCalling,
                ProviderCapability::Delegation,
                ProviderCapability::Vision,
                ProviderCapability::LongContext,
            ][..],
            "codex" => &[
                ProviderCapability::Reasoning,
                ProviderCapability::CodeEditing,
                ProviderCapability::ToolCalling,
                ProviderCapability::Delegation,
                ProviderCapability::Vision,
            ][..],
            "gemini" => &[
                ProviderCapability::Reasoning,
                ProviderCapability::CodeEditing,
                ProviderCapability::ToolCalling,
                ProviderCapability::Vision,
                ProviderCapability::LongContext,
            ][..],
            "antigravity" => &[
                ProviderCapability::Reasoning,
                ProviderCapability::CodeEditing,
                ProviderCapability::ToolCalling,
                ProviderCapability::Delegation,
                ProviderCapability::Vision,
                ProviderCapability::LongContext,
            ][..],
            "glm" | "openrouter" | "pi" | "hermes" => &[
                ProviderCapability::Reasoning,
                ProviderCapability::CodeEditing,
                ProviderCapability::ToolCalling,
            ][..],
            "kimi" => &[
                ProviderCapability::Reasoning,
                ProviderCapability::CodeEditing,
                ProviderCapability::ToolCalling,
                ProviderCapability::Delegation,
                ProviderCapability::Vision,
                ProviderCapability::LongContext,
            ][..],
            "shell" => &[
                ProviderCapability::LocalExecution,
                ProviderCapability::DeterministicCommands,
            ][..],
            _ => return None,
        };
        Some(capabilities.iter().copied().collect())
    }

    /// Negotiate required and optional capabilities before spawn.
    ///
    /// A named provider is never silently replaced: a mismatch is returned to
    /// the caller. Without a preference, candidates missing any required
    /// capability are excluded and optional capability coverage is used only
    /// as a deterministic tie-breaker.
    pub fn negotiate_provider(
        preferred: Option<&str>,
        required: &[ProviderCapability],
        optional: &[ProviderCapability],
    ) -> std::result::Result<ProviderSelection, ProviderNegotiationError> {
        if let Some(provider) = preferred {
            let capabilities = Self::capabilities_for(provider)
                .ok_or_else(|| ProviderNegotiationError::UnknownProvider(provider.to_string()))?;
            let missing_required = missing_capabilities(required, &capabilities);
            if !missing_required.is_empty() {
                return Err(ProviderNegotiationError::RequiredCapabilitiesMissing {
                    provider: provider.to_string(),
                    missing: missing_required,
                });
            }
            return Ok(ProviderSelection::new(
                provider,
                capabilities,
                optional,
                false,
            ));
        }

        let mut candidates: Vec<ProviderSelection> = Self::all_providers()
            .into_iter()
            .filter_map(|provider| {
                let capabilities = Self::capabilities_for(provider)?;
                missing_capabilities(required, &capabilities)
                    .is_empty()
                    .then(|| ProviderSelection::new(provider, capabilities, optional, true))
            })
            .collect();
        candidates.sort_by(|left, right| {
            right
                .optional_capabilities_satisfied
                .len()
                .cmp(&left.optional_capabilities_satisfied.len())
                .then_with(|| provider_rank(&left.provider).cmp(&provider_rank(&right.provider)))
        });
        candidates
            .into_iter()
            .next()
            .ok_or_else(|| ProviderNegotiationError::NoProviderSatisfies {
                required: required.to_vec(),
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCapability {
    Reasoning,
    CodeEditing,
    ToolCalling,
    Delegation,
    Vision,
    LongContext,
    LocalExecution,
    DeterministicCommands,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderSelection {
    pub provider: String,
    pub capabilities: BTreeSet<ProviderCapability>,
    pub optional_capabilities_satisfied: Vec<ProviderCapability>,
    /// True when OmegaOS selected a provider from capabilities rather than
    /// honoring an explicit caller preference.
    pub automatically_selected: bool,
}

impl ProviderSelection {
    fn new(
        provider: &str,
        capabilities: BTreeSet<ProviderCapability>,
        optional: &[ProviderCapability],
        automatically_selected: bool,
    ) -> Self {
        let optional_capabilities_satisfied = optional
            .iter()
            .copied()
            .filter(|capability| capabilities.contains(capability))
            .collect();
        Self {
            provider: provider.to_string(),
            capabilities,
            optional_capabilities_satisfied,
            automatically_selected,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProviderNegotiationError {
    #[error("unknown provider: {0}")]
    UnknownProvider(String),
    #[error("provider {provider} is missing required capabilities: {missing:?}")]
    RequiredCapabilitiesMissing {
        provider: String,
        missing: Vec<ProviderCapability>,
    },
    #[error("no provider satisfies required capabilities: {required:?}")]
    NoProviderSatisfies { required: Vec<ProviderCapability> },
}

fn missing_capabilities(
    required: &[ProviderCapability],
    available: &BTreeSet<ProviderCapability>,
) -> Vec<ProviderCapability> {
    required
        .iter()
        .copied()
        .filter(|capability| !available.contains(capability))
        .collect()
}

fn provider_rank(provider: &str) -> usize {
    ProvidersConfig::all_providers()
        .iter()
        .position(|candidate| *candidate == provider)
        .unwrap_or(usize::MAX)
}

/// Mirror the global provider/model selected for newly spawned sessions.
/// Persisted for diagnostics and non-Rust clients; `OmegaConfig::agent_command`
/// remains the launch authority.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ActiveModel {
    #[serde(default = "default_active_provider")]
    pub active_provider: String,
    #[serde(default = "default_active_model")]
    pub active_model: String,
    #[doc(hidden)]
    #[serde(skip)]
    pub source_revision: Option<crate::config::PrivateRevision>,
}

fn default_active_provider() -> String {
    ProvidersConfig::default_provider().to_string()
}

fn default_active_model() -> String {
    ProvidersConfig::default_model(ProvidersConfig::default_provider()).to_string()
}

impl Default for ActiveModel {
    fn default() -> Self {
        Self {
            active_provider: default_active_provider(),
            active_model: default_active_model(),
            source_revision: None,
        }
    }
}

impl ActiveModel {
    fn path() -> PathBuf {
        crate::config::omega_dir()
            .join("state")
            .join("active-model.json")
    }

    fn legacy_path() -> PathBuf {
        crate::config::omega_dir()
            .join("state")
            .join("telegram-active-model.json")
    }

    pub fn load() -> Self {
        match Self::try_load() {
            Ok(model) => model,
            Err(error) => {
                eprintln!(
                    "omega: active-model state unavailable; diagnostic default only: {error:#}"
                );
                tracing::error!(error = %error, "active-model state unavailable");
                Self::default()
            }
        }
    }

    pub fn try_load() -> Result<Self> {
        let path = Self::path();
        if path.exists() {
            return Self::load_from(&path);
        }
        let legacy = Self::legacy_path();
        if legacy.exists() {
            let mut model = Self::load_from(&legacy)?;
            // The revision belongs to the legacy authority, not the new path.
            model.source_revision = None;
            return Ok(model);
        }
        Self::load_from(&path)
    }

    fn load_from(path: &Path) -> Result<Self> {
        let Some(raw) = crate::config::read_private_optional_string(path)? else {
            return Ok(Self::default());
        };
        let mut model: Self = serde_json::from_str(&raw)
            .with_context(|| format!("parsing active-model state {}", path.display()))?;
        model
            .validate()
            .with_context(|| format!("validating active-model state {}", path.display()))?;
        model.source_revision = Some(crate::config::private_revision(raw.as_bytes()));
        Ok(model)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        self.save_to(&path)
    }

    fn save_to(&self, path: &Path) -> Result<()> {
        self.validate()
            .with_context(|| format!("validating active-model state for {}", path.display()))?;
        let json = serde_json::to_string_pretty(self)?;
        crate::config::with_private_file_lock(path, || {
            crate::config::require_private_revision(path, self.source_revision.as_ref())?;
            crate::config::atomic_write_private(path, json.as_bytes())
                .context("writing active-model state")
        })
    }

    fn validate(&self) -> Result<()> {
        if !ProvidersConfig::is_known(&self.active_provider) {
            anyhow::bail!("unknown active provider: {:?}", self.active_provider);
        }
        if self.active_provider != "shell" && self.active_model.trim().is_empty() {
            anyhow::bail!(
                "active model is empty for provider {:?}",
                self.active_provider
            );
        }
        Ok(())
    }

    /// Set the active provider+model. If `model` is empty, falls back to the
    /// provider's default model.
    pub fn set(provider: &str, model: Option<&str>) -> Result<Self> {
        if !ProvidersConfig::is_known(provider) {
            anyhow::bail!("unknown provider: {}", provider);
        }
        let resolved_model = match model {
            Some(m) if !m.is_empty() => m.to_string(),
            _ => ProvidersConfig::default_model(provider).to_string(),
        };
        let new = Self {
            active_provider: provider.to_string(),
            active_model: resolved_model,
            source_revision: Self::try_load()?.source_revision,
        };
        new.save()?;
        Ok(new)
    }
}

#[cfg(test)]
mod provider_capability_tests {
    use super::*;

    #[test]
    fn catalog_covers_every_agent_adapter() {
        for provider in [
            "claude",
            "codex",
            "gemini",
            "antigravity",
            "glm",
            "openrouter",
            "pi",
            "hermes",
            "kimi",
            "shell",
        ] {
            assert!(ProvidersConfig::is_known(provider), "{provider}");
            assert!(
                ProvidersConfig::capabilities_for(provider).is_some(),
                "{provider}"
            );
        }
    }

    #[test]
    fn openrouter_catalog_offers_ox_alpha_without_moving_the_default() {
        // OpenRouter and Hermes share the curated OpenRouter catalog.
        // Pi is standalone and must not inherit that default.
        for provider in ["openrouter", "hermes"] {
            assert!(
                ProvidersConfig::models_for(provider).contains(&"stealth/ox-alpha"),
                "{provider} is missing stealth/ox-alpha"
            );
            assert_eq!(
                ProvidersConfig::default_model(provider),
                "anthropic/claude-opus-5",
                "{provider}"
            );
        }
        assert!(
            ProvidersConfig::models_for("pi").is_empty(),
            "Pi picker must not be the OpenRouter catalog"
        );
        assert_eq!(ProvidersConfig::default_model("pi"), "");
    }

    #[test]
    fn kimi_is_first_class_and_uses_the_supported_override_channel() {
        assert!(ProvidersConfig::is_known("kimi"));
        assert_eq!(ProvidersConfig::default_model("kimi"), "kimi-for-coding");
        assert!(ProvidersConfig::models_for("kimi").contains(&"k3"));
        assert_eq!(ProvidersConfig::auth_type("kimi"), "oauth_or_api_key");

        let config = ProvidersConfig {
            kimi: KimiConfig {
                api_key: "secret".to_string(),
                base_url: "https://api.moonshot.ai/v1".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let env: std::collections::BTreeMap<_, _> = config.env_vars().into_iter().collect();
        assert_eq!(
            env.get("KIMI_MODEL_NAME").map(String::as_str),
            Some("kimi-for-coding")
        );
        assert_eq!(
            env.get("KIMI_MODEL_API_KEY").map(String::as_str),
            Some("secret")
        );
        assert_eq!(
            env.get("KIMI_MODEL_PROVIDER_TYPE").map(String::as_str),
            Some("kimi")
        );
        assert!(!env.contains_key("KIMI_API_KEY"));
        assert!(!env.contains_key("MOONSHOT_API_KEY"));
    }

    #[test]
    fn current_cli_catalog_avoids_retired_or_fabricated_model_ids() {
        assert_eq!(ProvidersConfig::default_model("codex"), "gpt-5.6");
        assert!(!ProvidersConfig::models_for("codex").contains(&"gpt-5.5-codex"));
        assert_eq!(ProvidersConfig::default_model("gemini"), "auto");
        assert!(ProvidersConfig::models_for("gemini").contains(&"gemini-3.1-pro-preview"));
        assert!(!ProvidersConfig::models_for("gemini").contains(&"gemini-3.1-pro"));
        assert_eq!(ProvidersConfig::default_model("glm"), "glm-5.3");
    }

    #[test]
    fn antigravity_is_first_class_and_uses_native_oauth() {
        assert!(ProvidersConfig::is_known("antigravity"));
        assert_eq!(ProvidersConfig::auth_type("antigravity"), "oauth");
        assert!(ProvidersConfig::models_for("antigravity").is_empty());
        assert!(
            ProvidersConfig::default()
                .antigravity
                .dangerously_skip_permissions
        );
    }

    #[test]
    fn malformed_provider_config_fails_and_cannot_be_overwritten() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("providers.toml");
        let corrupt = b"[codex\napi_key = 'secret'\n";
        std::fs::write(&path, corrupt).unwrap();
        assert!(ProvidersConfig::load_from(&path).is_err());

        let result = ProvidersConfig::default().save_to(&path);
        assert!(
            result.is_err(),
            "mutation must not replace corrupt provider config"
        );
        assert_eq!(std::fs::read(path).unwrap(), corrupt);
    }

    #[test]
    fn provider_config_rejects_unknown_keys_and_redacts_debug_output() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("providers.toml");
        std::fs::write(&path, "[codex]\napi_key = 'top-secret'\nmodle = 'typo'\n").unwrap();
        let error = ProvidersConfig::load_from(&path).unwrap_err();
        assert!(!format!("{error:#}").contains("top-secret"), "{error:#}");
        assert!(
            format!("{error:#}").contains("strict TOML/schema validation"),
            "{error:#}"
        );

        let config = ProvidersConfig {
            codex: CodexConfig {
                api_key: "top-secret".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let debug = format!("{config:?}");
        assert!(!debug.contains("top-secret"));
        assert!(debug.contains("codex_has_api_key: true"));
        let nested_debug = format!("{:?}", config.codex);
        assert!(!nested_debug.contains("top-secret"));
        assert!(nested_debug.contains("has_api_key: true"));
    }

    #[test]
    fn stale_provider_and_active_model_saves_are_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let provider_path = tmp.path().join("providers.toml");
        ProvidersConfig::default().save_to(&provider_path).unwrap();
        let mut first = ProvidersConfig::load_from(&provider_path).unwrap();
        let mut stale = ProvidersConfig::load_from(&provider_path).unwrap();
        first.codex.model = "first-writer".to_string();
        stale.codex.model = "stale-writer".to_string();
        first.save_to(&provider_path).unwrap();
        assert!(stale.save_to(&provider_path).is_err());
        assert_eq!(
            ProvidersConfig::load_from(&provider_path)
                .unwrap()
                .codex
                .model,
            "first-writer"
        );

        let model_path = tmp.path().join("active-model.json");
        ActiveModel::default().save_to(&model_path).unwrap();
        let mut first = ActiveModel::load_from(&model_path).unwrap();
        let mut stale = ActiveModel::load_from(&model_path).unwrap();
        first.active_model = "first-writer".to_string();
        stale.active_model = "stale-writer".to_string();
        first.save_to(&model_path).unwrap();
        assert!(stale.save_to(&model_path).is_err());
        assert_eq!(
            ActiveModel::load_from(&model_path).unwrap().active_model,
            "first-writer"
        );
    }

    #[cfg(unix)]
    #[test]
    fn provider_config_rejects_symlinks_dangling_symlinks_and_hardlinks() {
        use std::os::unix::fs::{symlink, PermissionsExt};

        let tmp = tempfile::tempdir().unwrap();
        let victim = tmp.path().join("victim.toml");
        let authority = tmp.path().join("providers.toml");
        std::fs::write(&victim, "[codex]\nmodel = \"gpt-5.6\"\n").unwrap();
        let original_mode = std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777;

        symlink(&victim, &authority).unwrap();
        assert!(ProvidersConfig::load_from(&authority).is_err());
        assert_eq!(
            std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777,
            original_mode
        );

        std::fs::remove_file(&authority).unwrap();
        symlink(tmp.path().join("missing.toml"), &authority).unwrap();
        assert!(ProvidersConfig::load_from(&authority).is_err());

        std::fs::remove_file(&authority).unwrap();
        std::fs::hard_link(&victim, &authority).unwrap();
        let error = ProvidersConfig::load_from(&authority).unwrap_err();
        assert!(error.to_string().contains("hard links"), "{error:#}");
        assert_eq!(
            std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777,
            original_mode,
            "a rejected authority hardlink must not chmod the shared inode"
        );
    }

    #[test]
    fn provider_save_is_atomic_private_and_validates_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("providers.toml");
        let mut config = ProvidersConfig {
            codex: CodexConfig {
                api_key: "secret".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        config.save_to(&path).unwrap();
        let loaded = ProvidersConfig::load_from(&path).unwrap();
        assert_eq!(loaded.codex.api_key, "secret");
        assert_eq!(
            std::fs::read_dir(tmp.path())
                .unwrap()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_name().to_string_lossy().contains("omega-tmp"))
                .count(),
            0
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }

        config.codex.additional_writable_dirs = vec!["relative/path".to_string()];
        assert!(config.save_to(&path).is_err());
        config.codex.additional_writable_dirs = vec!["/".to_string()];
        assert!(config.validate().is_err());
        config.codex.additional_writable_dirs = vec!["/tmp/safe/../../".to_string()];
        assert!(config.validate().is_err());
    }

    #[test]
    fn invalid_kimi_provider_type_is_rejected() {
        let config = ProvidersConfig {
            kimi: KimiConfig {
                provider_type: "kimi; touch /tmp/nope".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn malformed_active_model_is_not_defaulted_or_overwritten() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("active-model.json");
        let corrupt = b"{not-json";
        std::fs::write(&path, corrupt).unwrap();
        assert!(ActiveModel::load_from(&path).is_err());
        assert!(ActiveModel::default().save_to(&path).is_err());
        assert_eq!(std::fs::read(path).unwrap(), corrupt);
    }

    #[cfg(unix)]
    #[test]
    fn active_model_rejects_symlinks_dangling_symlinks_and_hardlinks() {
        use std::os::unix::fs::{symlink, PermissionsExt};

        let tmp = tempfile::tempdir().unwrap();
        let victim = tmp.path().join("victim.json");
        let authority = tmp.path().join("active-model.json");
        std::fs::write(
            &victim,
            r#"{"active_provider":"codex","active_model":"gpt-5.6"}"#,
        )
        .unwrap();
        let original_mode = std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777;

        symlink(&victim, &authority).unwrap();
        assert!(ActiveModel::load_from(&authority).is_err());
        assert_eq!(
            std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777,
            original_mode
        );

        std::fs::remove_file(&authority).unwrap();
        symlink(tmp.path().join("missing.json"), &authority).unwrap();
        assert!(ActiveModel::load_from(&authority).is_err());

        std::fs::remove_file(&authority).unwrap();
        std::fs::hard_link(&victim, &authority).unwrap();
        let error = ActiveModel::load_from(&authority).unwrap_err();
        assert!(error.to_string().contains("hard links"), "{error:#}");
        assert_eq!(
            std::fs::metadata(&victim).unwrap().permissions().mode() & 0o777,
            original_mode,
            "a rejected authority hardlink must not chmod the shared inode"
        );
    }

    #[test]
    fn active_model_rejects_unknown_provider_and_writes_privately() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("active-model.json");
        let unknown = ActiveModel {
            active_provider: "codxe".to_string(),
            active_model: "gpt".to_string(),
            source_revision: None,
        };
        assert!(unknown.save_to(&path).is_err());

        ActiveModel::default().save_to(&path).unwrap();
        assert_eq!(
            ActiveModel::load_from(&path).unwrap().active_provider,
            "codex"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn required_capability_mismatch_fails_before_spawn() {
        let result = ProvidersConfig::negotiate_provider(
            Some("shell"),
            &[ProviderCapability::CodeEditing],
            &[],
        );
        assert!(matches!(
            result,
            Err(ProviderNegotiationError::RequiredCapabilitiesMissing {
                provider,
                missing,
            }) if provider == "shell" && missing == vec![ProviderCapability::CodeEditing]
        ));
    }

    #[test]
    fn optional_capabilities_rank_without_becoming_requirements() {
        let selection = ProvidersConfig::negotiate_provider(
            None,
            &[ProviderCapability::Reasoning],
            &[
                ProviderCapability::Delegation,
                ProviderCapability::LongContext,
            ],
        )
        .unwrap();
        assert_eq!(selection.provider, "claude");
        assert_eq!(selection.optional_capabilities_satisfied.len(), 2);
        assert!(selection.automatically_selected);
    }

    #[test]
    fn impossible_required_set_returns_no_provider() {
        let result = ProvidersConfig::negotiate_provider(
            None,
            &[
                ProviderCapability::CodeEditing,
                ProviderCapability::DeterministicCommands,
            ],
            &[],
        );
        assert!(matches!(
            result,
            Err(ProviderNegotiationError::NoProviderSatisfies { .. })
        ));
    }
}
