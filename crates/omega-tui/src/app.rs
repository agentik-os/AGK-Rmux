use omega_core::config::OmegaConfig;
use omega_core::progress::ProgressInfo;
use omega_core::session::{OmegaSession, SessionManager, SessionRole};

// fix7-T2: the input-timing windows, previously five inline "Instant within
// N ms" idioms whose ordering invariant lived in a comment. Same values —
// named so the relationships are visible (and pinned) in one place.
/// DESIGN-015 post-drop grace: destructive single-key hotkeys are ignored
/// this long after a non-deliberate chat-focus drop.
pub(crate) const GRACE_MS: u64 = 800;
/// FIX-2 minimum display time for async-origin sticky status notices.
pub(crate) const STICKY_MS: u64 = 2000;
/// Tab double-tap window (Sessions menu toggle + 2col fullscreen).
pub(crate) const DOUBLE_TAP_MS: u64 = 400;

/// True while `t` is less than `ms` milliseconds in the past — the single
/// idiom behind every input-timing window above.
pub(crate) fn within(t: std::time::Instant, ms: u64) -> bool {
    t.elapsed() < std::time::Duration::from_millis(ms)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Sessions,
    Menu,
    Projects,
    /// The doctrine surface: Laws, Rules, the AISB agents, the architecture,
    /// the installed skills and the whole manual. It used to be its own tab
    /// named "Info", was renamed to "Agentic", and then lost its identity when
    /// Agentic was repurposed into Projects — its sections survived as a
    /// 5-row group buried above the project list. This tab gives it back.
    System,
    Settings,
    /// The AgentikOS operative-systems suite (Mindset OS, Habits OS, …) —
    /// registry + integration status, backed by `omega_core::os_products`.
    Os,
    Help,
}

impl Tab {
    /// Left-to-right order of the tab bar — the ONE source of truth. The bar
    /// labels, the highlighted index and Left/Right cycling all derive from
    /// this array, so a reorder is a single edit here. (They used to be three
    /// hand-kept lists, which is how the bar and the enum drifted apart.)
    pub const ORDER: [Tab; 7] = [
        Tab::Sessions,
        Tab::Projects,
        Tab::Os,
        Tab::Menu,
        Tab::System,
        Tab::Help,
        Tab::Settings,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            Tab::Sessions => "Sessions",
            Tab::Projects => "Projects",
            Tab::Os => "OS",
            Tab::Menu => "Menu",
            Tab::System => "System",
            Tab::Help => "Help",
            Tab::Settings => "Settings",
        }
    }

    /// Position in the tab bar.
    pub fn index(&self) -> usize {
        Self::ORDER.iter().position(|t| t == self).unwrap_or(0)
    }
}

/// Claude OAuth re-login progress, surfaced in the Monitor → Account view.
/// Driven asynchronously: the engine call runs off the event loop and writes
/// the result back into `App::reauth_status` via a shared sink (see main.rs).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ReauthStatus {
    /// No re-login in progress.
    #[default]
    Idle,
    /// `request_reauth` is running (spawn session + /login + capture URL, ~16s).
    Generating,
    /// URL captured — show it and prompt the user to enter the returned code.
    ShowUrl(String),
    /// `handle_code` is running (paste code + watch credentials).
    Validating,
    /// Login finished successfully (human-readable summary, e.g. email + expiry).
    Done(String),
    /// Login failed (human-readable error).
    Error(String),
}

/// The sections of the System tab, top to bottom. Order is the reading order
/// of the system itself: what it is → what binds it → who runs it → what it
/// can do → the manual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoSection {
    Overview,
    Laws,
    Rules,
    AisbAgents,
    Atlas,
    Oracle,
    Workers,
    Skills,
    Docs,
}

impl InfoSection {
    pub fn all() -> &'static [InfoSection] {
        &[
            InfoSection::Overview,
            InfoSection::Laws,
            InfoSection::Rules,
            InfoSection::AisbAgents,
            InfoSection::Atlas,
            InfoSection::Oracle,
            InfoSection::Workers,
            InfoSection::Skills,
            InfoSection::Docs,
        ]
    }

    /// Sections whose right panel carries its own ↑/↓ cursor over a list,
    /// rather than scrolling free-form text.
    pub fn has_sub_cursor(&self) -> bool {
        matches!(self, InfoSection::AisbAgents | InfoSection::Docs)
    }

    pub fn label(&self) -> String {
        match self {
            InfoSection::Overview => "Overview — what OmegaOS is".to_string(),
            // Counts derived from the registries, never hardcoded: the literal
            // "(13)" silently went stale when Council joined the Matrix agents.
            InfoSection::Laws => format!("Laws ({}) — inviolable", omega_core::rules::laws().len()),
            InfoSection::Rules => format!(
                "Rules ({}) — operational doctrine",
                omega_core::rules::all_rules()
                    .iter()
                    .filter(|r| r.kind == omega_core::rules::RuleKind::Rule)
                    .count()
            ),
            InfoSection::AisbAgents => format!(
                "AI Agents ({})",
                omega_core::aisb_agents::AisbAgent::all().len()
            ),
            InfoSection::Atlas => "Atlas — the Director brain".to_string(),
            InfoSection::Oracle => "Oracle — routing & coordination".to_string(),
            InfoSection::Workers => "Workers — dispatch & lifecycle".to_string(),
            InfoSection::Skills => "Skills — the installed arsenal".to_string(),
            InfoSection::Docs => "Documentation — the manual".to_string(),
        }
    }
}

/// Which surface of a project a new session opens: the CODE or its MARKETING
/// (the marketing machine + the project's dedicated marketing agent).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectLane {
    Coding,
    Marketing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    NewNamedSession(String),
    NewSessionPromptDirect(String, String),
    /// Dispatch oracle — step 1: project PICKER (no typing). Holds the project
    /// names + the selected index, mirroring `SelectModel`. The list comes from
    /// the shared `ProjectRegistry` — the SAME source the Telegram dispatch
    /// picker uses — so the added-projects list stays in sync across surfaces.
    DispatchProject(Vec<String>, usize),
    /// Open-project step 1 (Projects tab → Enter on a project): pick the LANE —
    /// Coding session, Marketing session (marketing machine + dedicated agent
    /// in `<project>/marketing/`), or the project's Oracle. Holds
    /// (project name, project path, selected index).
    ProjectOpenLane(String, String, usize),
    /// Open-project step 2: pick the LLM agent among those actually INSTALLED
    /// on this machine (claude / codex / gemini / kimi / …). Same picker for
    /// both lanes. Opening always starts a NEW blank session — re-entering an
    /// existing one is the Sessions tab's job.
    ProjectOpenAgentPick {
        lane: ProjectLane,
        name: String,
        path: String,
        agents: Vec<omega_core::agents::Agent>,
        sel: usize,
    },
    /// Project delete menu (Projects tab → Projects group 'x') — the SAME three escalating
    /// tiers as the Telegram bot's delete menu (omega → local → all), executed
    /// through the bot's one-shot CLI (one canonical deletion impl).
    /// Holds (project name, selected index).
    ProjectDelete(String, usize),
    DispatchMission(String),
    /// Renaming an existing session — holds the original name.
    RenameSession(String),
    /// Live session-list filter — the in-progress query is in `input_buffer`,
    /// the applied query in `session_filter`.
    SessionFilter,

    /// New-project wizard — step 1: project name (text input).
    NewProjectName,
    /// New-project wizard — step 2: category picker. (name, sel) where `sel`
    /// indexes `NEW_PROJECT_CATEGORIES`. Selection lives in the variant so the
    /// wizard needs no extra App state.
    NewProjectCategory(String, usize),
    /// New-project wizard — step 2b (customer projects only): pick/create a credential
    /// group (separate client accounts). (name, category)
    NewProjectCredGroup(String, String),
    /// New-project wizard — step 3: stack picker. (name, category, sel) where
    /// `sel` indexes `NEW_PROJECT_STACKS`.
    NewProjectStack(String, String, usize),
    /// New-project wizard — step 4 (optional): kickoff prompt. (name, category, stack)
    NewProjectLaunchPrompt(String, String, String),
    /// New-project wizard — step 5 (optional): doc paths. (name, category, stack, kickoff)
    NewProjectLaunchDocs(String, String, String, Option<String>),

    /// Provisioning-keys wizard (Monitor tab, Telegram-style). `step` indexes
    /// `PROVISIONING_FIELDS`; `collected` holds the values entered so far (one
    /// per completed step, in order). The current field's value is in
    /// `input_buffer`.
    ProvisioningSetup {
        step: usize,
        collected: Vec<String>,
    },

    /// Telegram setup wizard — step 1: bot token
    TelegramSetupToken,
    /// Step 2: chat id (carries the bot token)
    TelegramSetupChatId(String),
    /// Step 3: optional user id allow-list (carries token + chat_id)
    TelegramSetupUserId(String, String),

    /// Editing a settings text field — holds (config_key, masked).
    EditSettingsField {
        config_key: String,
        masked: bool,
    },
    /// Picking a value from a fixed list via an arrow-key overlay (NO typing).
    /// Holds (config_key, options, selected_index). Mirrors the new-project
    /// category/stack pickers — Up/Down move, Enter commits, Esc cancels.
    SelectModel(String, Vec<String>, usize),

    /// Monitor → Project group: single-field capture of the Telegram supergroup
    /// id (manual fallback to the bot's auto-detect). Persists via
    /// `TelegramGroupConfig`. The in-progress value lives in `input_buffer`.
    GroupSetupId,

    /// Register-existing-folder wizard — step 1: project path (text input).
    /// Distinct from `NewProjectName` (which CREATES a project on disk); this
    /// only registers an already-existing folder into the project registry.
    AddProjectPath,

    /// Claude OAuth re-login — paste the authorize code returned by the browser.
    /// Entered after the URL is shown (Monitor → Account); on submit the main
    /// loop runs `oauth::handle_code` and renders the result.
    ReauthCode,
}

/// New-project wizard option lists. `(id, label)` — `id` is the token passed to
/// the `/omega-new-project` skill; `label` is what the picker shows. Single
/// source of truth for both the menu UI and the spawned command.
pub const NEW_PROJECT_CATEGORIES: &[(&str, &str)] = &[
    (
        "customer",
        "Customer — client work  (customers/ under your projects dir)",
    ),
    (
        "side-business",
        "Side business — your own products  (side-business/ under your projects dir)",
    ),
    (
        "tools",
        "Tools — internal tooling / libraries  (tools/ under your projects dir)",
    ),
];
/// Strategies implemented by the installed `/omega-new-project` skill.
///
/// Keep this list deliberately small: an option is only public when the skill
/// has a real branch for it. `custom` enters the skill's discovery flow rather
/// than pretending that an unimplemented framework recipe exists.
pub const NEW_PROJECT_STACKS: &[(&str, &str)] = &[
    (
        "nextstack",
        "SaaS — Next.js 16 + Convex + Clerk + Stripe + shadcn",
    ),
    (
        "custom",
        "Custom — let the project skill discover and specify the stack",
    ),
];

/// Provisioning-keys wizard fields (Monitor tab). `(env_key, prompt, masked)` in
/// step order. `masked` hides the secret in the input echo. Written to
/// `~/.omega/provisioning/services.env` by `omega_core::provisioning`. Each step
/// is skippable (Esc = leave blank → keeps any existing value).
pub const PROVISIONING_FIELDS: &[(&str, &str, bool)] = &[
    (
        "VERCEL_TOKEN",
        "Vercel token — vercel.com/account/tokens (Full Account). Esc to skip.",
        true,
    ),
    (
        "CONVEX_TEAM_TOKEN",
        "Convex team token — dashboard → Team Settings → access token. Esc to skip.",
        true,
    ),
    (
        "CONVEX_TEAM_SLUG",
        "Convex team slug — your team's URL slug (not secret). Esc to skip.",
        false,
    ),
    (
        "GITHUB_TOKEN",
        "GitHub token (repo+workflow) — or Esc to skip & use `gh auth`.",
        true,
    ),
    (
        "STRIPE_SECRET_KEY",
        "Stripe secret key — dashboard → Developers → API keys. Esc to skip.",
        true,
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    NewClaude,
    NewCodex,
    NewGemini,
    NewAntigravity,
    NewPi,
    NewHermes,
    NewGlm,
    NewKimi,
    NewTerminal,
    NewProject,
    DispatchOracle,
    Refresh,
    ToggleProtection,
    KillSelected,
    KillAll,
    NuclearCleanup,
    Restart,
    Quit,
}

impl MenuAction {
    pub fn all() -> &'static [MenuAction] {
        &[
            MenuAction::NewClaude,
            MenuAction::NewCodex,
            MenuAction::NewGemini,
            MenuAction::NewAntigravity,
            MenuAction::NewPi,
            MenuAction::NewHermes,
            MenuAction::NewGlm,
            MenuAction::NewKimi,
            MenuAction::NewTerminal,
            MenuAction::NewProject,
            MenuAction::DispatchOracle,
            MenuAction::Refresh,
            MenuAction::ToggleProtection,
            MenuAction::KillSelected,
            MenuAction::KillAll,
            MenuAction::NuclearCleanup,
            MenuAction::Restart,
            MenuAction::Quit,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            MenuAction::NewClaude => "New Claude session",
            MenuAction::NewCodex => "New Codex session",
            MenuAction::NewGemini => "New Gemini session",
            MenuAction::NewAntigravity => "New Antigravity session",
            MenuAction::NewPi => "New Pi session (earendil-works)",
            MenuAction::NewHermes => "New Hermes session (Nous Research)",
            MenuAction::NewGlm => "New GLM session",
            MenuAction::NewKimi => "New Kimi session (Moonshot AI)",
            MenuAction::NewTerminal => "New Terminal (plain shell)",
            MenuAction::NewProject => {
                "New project  →  pick stack + auto-provision (Convex/Vercel/Clerk/Stripe)"
            }
            MenuAction::DispatchOracle => "Dispatch oracle  →  project + mission",
            MenuAction::Refresh => "Refresh sessions list",
            MenuAction::ToggleProtection => "Toggle protection on selected",
            MenuAction::KillSelected => "Kill selected session",
            MenuAction::KillAll => "Kill ALL sessions (keeps current + protected + infra)",
            MenuAction::NuclearCleanup => {
                "Nuclear cleanup — kill all + prune state + clear scratch + free RAM"
            }
            MenuAction::Restart => "Restart OmegaOS (reload binary)",
            MenuAction::Quit => "Quit OmegaOS",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            MenuAction::NewClaude => "c",
            MenuAction::NewCodex => "o",
            MenuAction::NewGemini => "g",
            MenuAction::NewAntigravity => "a",
            MenuAction::NewPi => "p",
            MenuAction::NewHermes => "h",
            MenuAction::NewGlm => "G",
            MenuAction::NewKimi => "K",
            MenuAction::NewTerminal => "t",
            MenuAction::NewProject => "N",
            MenuAction::DispatchOracle => "d",
            MenuAction::Refresh => "F5",
            MenuAction::ToggleProtection => ".",
            MenuAction::KillSelected => "x",
            MenuAction::KillAll => "—",
            MenuAction::NuclearCleanup => "—",
            MenuAction::Restart => "R",
            MenuAction::Quit => "q",
        }
    }

    pub fn agent(&self) -> Option<omega_core::agents::Agent> {
        match self {
            MenuAction::NewClaude => Some(omega_core::agents::Agent::Claude),
            MenuAction::NewCodex => Some(omega_core::agents::Agent::Codex),
            MenuAction::NewGemini => Some(omega_core::agents::Agent::Gemini),
            MenuAction::NewAntigravity => Some(omega_core::agents::Agent::Antigravity),
            MenuAction::NewPi => Some(omega_core::agents::Agent::Pi),
            MenuAction::NewHermes => Some(omega_core::agents::Agent::Hermes),
            MenuAction::NewGlm => Some(omega_core::agents::Agent::Glm),
            MenuAction::NewKimi => Some(omega_core::agents::Agent::Kimi),
            MenuAction::NewTerminal => Some(omega_core::agents::Agent::Shell),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum SessionRow {
    /// A visible section header (e.g. "── Causio ──")
    Header(String),
    Entry(SessionEntry),
}

impl SessionRow {
    pub fn is_selectable(&self) -> bool {
        matches!(self, SessionRow::Entry(_))
    }
    pub fn as_entry(&self) -> Option<&SessionEntry> {
        match self {
            SessionRow::Entry(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionEntry {
    pub session: OmegaSession,
    pub progress: Option<ProgressInfo>,
    pub is_current: bool,
    pub is_protected: bool,
    pub tree_prefix: String,
}

impl SessionEntry {
    pub fn clone_for_row(&self) -> Self {
        self.clone()
    }
}

fn section_for(session: &OmegaSession) -> String {
    use omega_core::session::SessionRole;
    match (&session.role, &session.project) {
        (SessionRole::Oracle | SessionRole::Worker, Some(p)) => p.clone(),
        (SessionRole::Home, _) => "Home".to_string(),
        (SessionRole::System, _) => "System".to_string(),
        _ => "Other".to_string(),
    }
}

/// Section ordering for the Sessions tab: your own shells first, then the
/// project sections (alphabetical among themselves), then the machinery.
/// Keyed off the same match as `section_for` so the two can't drift.
fn section_rank(session: &OmegaSession) -> u8 {
    use omega_core::session::SessionRole;
    match (&session.role, &session.project) {
        (SessionRole::Home, _) => 0,
        (SessionRole::Oracle | SessionRole::Worker, Some(_)) => 1,
        (SessionRole::System, _) => 2,
        _ => 3,
    }
}

/// Ordering INSIDE a project section: the oracle heads its own workers.
fn role_rank(session: &OmegaSession) -> u8 {
    use omega_core::session::SessionRole;
    match session.role {
        SessionRole::Oracle => 0,
        SessionRole::Worker => 1,
        _ => 2,
    }
}

/// Which side of the Sessions tab has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionFocus {
    /// Default: left list focused, right shows preview only
    List,
    /// Tab pressed once: split layout (list + chat input)
    Chat,
    /// Tab pressed twice quickly: chat takes the full width
    ChatFullscreen,
}

/// Interactive items within a Settings sub-section.
#[derive(Debug, Clone)]
pub enum SettingsField {
    /// Run a shell command (typically install/uninstall).
    /// `confirm_first`: if true, the field shows the command and Enter runs it
    /// without further prompting; if false, fires immediately.
    Action {
        label: String,
        command: String,
        confirm_first: bool,
    },
    /// Open a text input modal that updates a config key on confirm.
    EditText {
        label: String,
        config_key: String,
        current_value: String,
        masked: bool,
    },
    /// Toggle a boolean config key.
    Toggle {
        label: String,
        config_key: String,
        current: bool,
    },
    /// Pick one value from a fixed list via an arrow-key overlay (NO typing).
    /// `current_index` points at the currently-saved value in `options`.
    Select {
        label: String,
        config_key: String,
        options: Vec<String>,
        current_index: usize,
    },
    /// Show informational text only (homepage link, status, etc.).
    Info(String),
}

impl SettingsField {
    pub fn is_actionable(&self) -> bool {
        !matches!(self, SettingsField::Info(_))
    }
    pub fn label(&self) -> &str {
        match self {
            SettingsField::Action { label, .. } => label,
            SettingsField::EditText { label, .. } => label,
            SettingsField::Toggle { label, .. } => label,
            SettingsField::Select { label, .. } => label,
            SettingsField::Info(s) => s,
        }
    }
}

/// Build the effort field: a fixed arrow-key Select over the levels the agent
/// CLI actually accepts (`--effort low|medium|high|xhigh|max` — agents.rs). A
/// saved custom/legacy value is prepended so it stays visible until replaced.
fn effort_field(config_key: &str, current: &str) -> SettingsField {
    let mut options: Vec<String> = ["low", "medium", "high", "xhigh", "max"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let current_index = match options.iter().position(|e| e == current) {
        Some(i) => i,
        None if current.is_empty() => 2, // default suggestion: high
        None => {
            options.insert(0, current.to_string());
            0
        }
    };
    SettingsField::Select {
        label: "Effort".to_string(),
        config_key: config_key.to_string(),
        options,
        current_index,
    }
}

/// Build a model field for a provider. When the provider has a known model
/// list (`providers::models_for`), this is an arrow-key Select (NO typing);
/// otherwise it falls back to a free-text field for account-scoped catalogs
/// such as Antigravity.
fn model_field(provider: &str, config_key: &str, current: &str) -> SettingsField {
    let opts: Vec<String> = omega_core::providers::ProvidersConfig::models_for(provider)
        .iter()
        .map(|s| s.to_string())
        .collect();
    if opts.is_empty() {
        SettingsField::EditText {
            label: "Model".to_string(),
            config_key: config_key.to_string(),
            current_value: current.to_string(),
            masked: false,
        }
    } else {
        // Point at the saved value; if it isn't in the list, prepend it so the
        // user never loses a custom value and index 0 stays valid.
        let mut options = opts;
        let current_index = match options.iter().position(|m| m == current) {
            Some(i) => i,
            None if current.is_empty() => 0,
            None => {
                options.insert(0, current.to_string());
                0
            }
        };
        SettingsField::Select {
            label: "Model".to_string(),
            config_key: config_key.to_string(),
            options,
            current_index,
        }
    }
}

/// Build the field list for a settings section.
/// TTL-cached `Agent::is_available()` for the render path. `fields_for_section`
/// runs every frame while Settings is open and `is_available` stats every PATH
/// dir per agent (`has_cmd`), so the uncached form burned hundreds of stat
/// calls per second at 15-60 FPS. A 2s memo keeps the [+]/[x] install badges
/// honest within one housekeeping tick of an install/uninstall finishing,
/// with zero per-frame filesystem work.
pub(crate) fn agent_available_cached(agent: omega_core::agents::Agent) -> bool {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::time::{Duration, Instant};
    thread_local! {
        static CACHE: RefCell<Option<(Instant, HashMap<&'static str, bool>)>> =
            const { RefCell::new(None) };
    }
    CACHE.with(|cell| {
        let mut slot = cell.borrow_mut();
        let stale = match slot.as_ref() {
            Some((at, _)) => at.elapsed() >= Duration::from_secs(2),
            None => true,
        };
        if stale {
            let map = omega_core::agents::Agent::all()
                .iter()
                .map(|a| (a.name(), a.is_available()))
                .collect();
            *slot = Some((Instant::now(), map));
        }
        slot.as_ref()
            .and_then(|(_, map)| map.get(agent.name()).copied())
            // An agent outside Agent::all() (impossible today) falls back to
            // the live check rather than lying "not installed".
            .unwrap_or_else(|| agent.is_available())
    })
}

/// One line summarising whether this box is keeping itself current, for the
/// top of Settings → General.
///
/// Read from `~/.omega/state/auto-update.json` (written by the daily cron), NOT
/// from `omega update --check`: this runs on every Settings frame, and `--check`
/// does a network `git fetch`. The System tab renders the same state in full
/// (`render_info_overview`); here it is one line, because the operator reading
/// it is about to press the button underneath it, not audit the history.
pub fn update_status_line(config: &OmegaConfig) -> String {
    let st = omega_core::auto_update::AutoUpdateState::load(&config.state_dir);
    let ago = |t: chrono::DateTime<chrono::Utc>| -> String {
        let mins = (chrono::Utc::now() - t).num_minutes().max(0);
        if mins < 60 {
            format!("{}m ago", mins)
        } else if mins < 60 * 48 {
            format!("{}h ago", mins / 60)
        } else {
            format!("{}d ago", mins / (60 * 24))
        }
    };
    let checked = match st.last_check {
        Some(t) => ago(t),
        // Never having checked is a real state, not a blank: the cron may not
        // be installed on this box at all.
        None => "never".to_string(),
    };
    let outcome = st.last_outcome.as_deref().unwrap_or("no run recorded yet");
    format!(
        "OmegaOS v{} · auto-update: {} · last check {} ({})",
        env!("CARGO_PKG_VERSION"),
        config.auto_update.as_str(),
        checked,
        outcome
    )
}

pub fn fields_for_section(
    section: SettingsSection,
    providers: &omega_core::providers::ProvidersConfig,
    config: &OmegaConfig,
) -> Vec<SettingsField> {
    use omega_core::agents::Agent;
    let mut out = Vec::new();
    let agent_for_section = |s: SettingsSection| -> Option<Agent> {
        match s {
            SettingsSection::Claude => Some(Agent::Claude),
            SettingsSection::Codex => Some(Agent::Codex),
            SettingsSection::Gemini => Some(Agent::Gemini),
            SettingsSection::Antigravity => Some(Agent::Antigravity),
            SettingsSection::OpenRouter => Some(Agent::OpenRouter),
            SettingsSection::Pi => Some(Agent::Pi),
            SettingsSection::Hermes => Some(Agent::Hermes),
            SettingsSection::Glm => Some(Agent::Glm),
            SettingsSection::Kimi => Some(Agent::Kimi),
            _ => None,
        }
    };

    match section {
        SettingsSection::General => {
            // ── Keeping OmegaOS current ──────────────────────────────────
            // The update runs as a DETACHED session (Action::RunShellCommand),
            // never in-process: `omega update` rebuilds the binary this very
            // TUI is running from, and the build takes minutes. Spawning it
            // keeps the UI alive and makes the pull + build watchable live.
            out.push(SettingsField::Info(update_status_line(config)));
            out.push(SettingsField::Action {
                label: "[Check] for an OmegaOS update (changes nothing)".to_string(),
                command: "omega update --check".to_string(),
                confirm_first: false,
            });
            out.push(SettingsField::Action {
                label: "[Update] OmegaOS now (pull + rebuild + reinstall)".to_string(),
                command: "omega update".to_string(),
                confirm_first: true,
            });
            out.push(SettingsField::Select {
                label: "Auto-update (daily 03:30)".to_string(),
                config_key: "general.auto_update".to_string(),
                options: vec!["apply".to_string(), "check".to_string(), "off".to_string()],
                current_index: match config.auto_update {
                    omega_core::config::AutoUpdatePolicy::Apply => 0,
                    omega_core::config::AutoUpdatePolicy::Check => 1,
                    omega_core::config::AutoUpdatePolicy::Off => 2,
                },
            });
            out.push(SettingsField::Info(String::new())); // spacer

            out.push(SettingsField::Info(format!(
                "Default model: {}",
                config.default_model
            )));
            out.push(SettingsField::Toggle {
                label: "Auto-open legacy Telegram log viewer".to_string(),
                config_key: "general.auto_spawn_master".to_string(),
                current: config.auto_spawn_master,
            });
            out.push(SettingsField::Toggle {
                label: "Auto-naming sessions".to_string(),
                config_key: "general.auto_naming".to_string(),
                current: config.auto_naming,
            });
            out.push(SettingsField::Toggle {
                label: "Session-menu shortcuts (x/r/b/. — arrows always navigate)".to_string(),
                config_key: "general.session_shortcuts".to_string(),
                current: config.session_shortcuts,
            });
        }
        SettingsSection::Theme => {
            // One Select over the theme registry; the SelectModel overlay
            // live-previews each theme as you arrow through it (input.rs),
            // and ui.rs appends a colored swatch line per theme below.
            let options: Vec<String> = crate::theme::ThemeId::all()
                .iter()
                .map(|t| t.slug().to_string())
                .collect();
            let current_index = options.iter().position(|s| *s == config.theme).unwrap_or(0);
            out.push(SettingsField::Select {
                label: "Active theme".to_string(),
                config_key: "general.theme".to_string(),
                options,
                current_index,
            });
            out.push(SettingsField::Toggle {
                label: "Theme background (OFF = keep the terminal's own background)".to_string(),
                config_key: "general.theme_background".to_string(),
                current: config.theme_background,
            });
        }
        SettingsSection::Install => {
            // Per-agent install / uninstall buttons
            for agent in Agent::all() {
                if matches!(agent, Agent::Shell) {
                    continue;
                }
                let installed = agent_available_cached(*agent);
                let badge = if installed { "[+]" } else { "[x]" };
                let agent_name = agent.name();
                out.push(SettingsField::Info(format!(
                    "{}  {:8}  {}",
                    badge,
                    agent_name,
                    agent.display_name()
                )));
                if let Some(cmd) = agent.install_command() {
                    out.push(SettingsField::Action {
                        label: format!(
                            "    [{}] {}",
                            if installed { "Re-install" } else { "Install" },
                            agent_name
                        ),
                        command: cmd.to_string(),
                        confirm_first: true,
                    });
                }
                if installed {
                    if let Some(cmd) = agent.uninstall_command() {
                        out.push(SettingsField::Action {
                            label: format!("    [Uninstall] {}", agent_name),
                            command: cmd.to_string(),
                            confirm_first: true,
                        });
                    }
                }
                out.push(SettingsField::Info(String::new())); // spacer
            }
        }
        SettingsSection::Claude => {
            let c = &providers.claude;
            out.push(model_field("claude", "claude.model", &c.model));
            // Effort = arrow-key Select (NO typing): free text let invalid values
            // through (e.g. "Ultra"); the CLI only accepts these five levels.
            out.push(effort_field("claude.effort", &c.effort));
            out.push(SettingsField::EditText {
                label: "Anthropic API key".to_string(),
                config_key: "claude.api_key".to_string(),
                current_value: c.api_key.clone(),
                masked: true,
            });
            out.push(SettingsField::Toggle {
                label: "Dangerously skip permissions".to_string(),
                config_key: "claude.dangerously_skip_permissions".to_string(),
                current: c.dangerously_skip_permissions,
            });
            out.extend(install_actions_for(Agent::Claude));
        }
        SettingsSection::Codex => {
            let c = &providers.codex;
            out.push(model_field("codex", "codex.model", &c.model));
            out.push(SettingsField::EditText {
                label: "OpenAI API key".to_string(),
                config_key: "codex.api_key".to_string(),
                current_value: c.api_key.clone(),
                masked: true,
            });
            out.push(SettingsField::EditText {
                label: "Base URL".to_string(),
                config_key: "codex.base_url".to_string(),
                current_value: c.base_url.clone(),
                masked: false,
            });
            out.push(SettingsField::Toggle {
                label: "Bypass hook trust in Omega sessions".to_string(),
                config_key: "codex.bypass_hook_trust".to_string(),
                current: c.bypass_hook_trust,
            });
            out.extend(install_actions_for(Agent::Codex));
        }
        SettingsSection::Gemini => {
            let c = &providers.gemini;
            out.push(model_field("gemini", "gemini.model", &c.model));
            out.push(SettingsField::EditText {
                label: "Google API key".to_string(),
                config_key: "gemini.api_key".to_string(),
                current_value: c.api_key.clone(),
                masked: true,
            });
            out.extend(install_actions_for(Agent::Gemini));
        }
        SettingsSection::Antigravity => {
            let c = &providers.antigravity;
            out.push(model_field("antigravity", "antigravity.model", &c.model));
            let mut effort_options =
                vec!["low".to_string(), "medium".to_string(), "high".to_string()];
            let effort_index = match effort_options.iter().position(|e| e == &c.effort) {
                Some(index) => index,
                None if c.effort.is_empty() => 1,
                None => {
                    effort_options.insert(0, c.effort.clone());
                    0
                }
            };
            out.push(SettingsField::Select {
                label: "Effort".to_string(),
                config_key: "antigravity.effort".to_string(),
                options: effort_options,
                current_index: effort_index,
            });
            out.push(SettingsField::Toggle {
                label: "Dangerously skip permissions".to_string(),
                config_key: "antigravity.dangerously_skip_permissions".to_string(),
                current: c.dangerously_skip_permissions,
            });
            out.extend(install_actions_for(Agent::Antigravity));
        }
        SettingsSection::Glm => {
            let c = &providers.glm;
            out.push(model_field("glm", "glm.model", &c.model));
            out.push(SettingsField::EditText {
                label: "GLM API key".to_string(),
                config_key: "glm.api_key".to_string(),
                current_value: c.api_key.clone(),
                masked: true,
            });
            out.extend(install_actions_for(Agent::Glm));
        }
        SettingsSection::OpenRouter => {
            let c = &providers.openrouter;
            out.push(model_field("openrouter", "openrouter.model", &c.model));
            out.push(SettingsField::EditText {
                label: "OpenRouter API key".to_string(),
                config_key: "openrouter.api_key".to_string(),
                current_value: c.api_key.clone(),
                masked: true,
            });
            out.push(SettingsField::EditText {
                label: "Base URL".to_string(),
                config_key: "openrouter.base_url".to_string(),
                current_value: c.base_url.clone(),
                masked: false,
            });
            out.extend(install_actions_for(Agent::OpenRouter));
        }
        SettingsSection::Pi => {
            let c = &providers.pi;
            out.push(SettingsField::EditText {
                label: "Provider".to_string(),
                config_key: "pi.provider".to_string(),
                current_value: c.provider.clone(),
                masked: false,
            });
            out.push(model_field("pi", "pi.model", &c.model));
            out.push(SettingsField::EditText {
                label: "Pi API key".to_string(),
                config_key: "pi.api_key".to_string(),
                current_value: c.api_key.clone(),
                masked: true,
            });
            out.extend(install_actions_for(Agent::Pi));
        }
        SettingsSection::Hermes => {
            let c = &providers.hermes;
            out.push(SettingsField::EditText {
                label: "Provider (empty = native; key = openrouter)".to_string(),
                config_key: "hermes.provider".to_string(),
                current_value: c.provider.clone(),
                masked: false,
            });
            out.push(model_field("hermes", "hermes.model", &c.model));
            out.push(SettingsField::EditText {
                label: "Hermes API key".to_string(),
                config_key: "hermes.api_key".to_string(),
                current_value: c.api_key.clone(),
                masked: true,
            });
            out.extend(install_actions_for(Agent::Hermes));
        }
        SettingsSection::Kimi => {
            let c = &providers.kimi;
            out.push(model_field("kimi", "kimi.model", &c.model));
            out.push(SettingsField::EditText {
                label: "Kimi API key".to_string(),
                config_key: "kimi.api_key".to_string(),
                current_value: c.api_key.clone(),
                masked: true,
            });
            out.push(SettingsField::EditText {
                label: "Base URL".to_string(),
                config_key: "kimi.base_url".to_string(),
                current_value: c.base_url.clone(),
                masked: false,
            });
            out.push(SettingsField::EditText {
                label: "Provider type (kimi/anthropic/openai)".to_string(),
                config_key: "kimi.provider_type".to_string(),
                current_value: c.provider_type.clone(),
                masked: false,
            });
            out.extend(install_actions_for(Agent::Kimi));
        }
        SettingsSection::Aisb => {
            out.push(SettingsField::Info(format!(
                "Viewer session name: {}",
                omega_core::aisb::MASTER_SESSION_NAME
            )));
            out.push(SettingsField::Info(
                "Read-only mirror of the Telegram conversation log; it is not an agent."
                    .to_string(),
            ));
            out.push(SettingsField::Action {
                label: "[Open AISB conversation viewer]".to_string(),
                command: "omega aisb-view".to_string(),
                confirm_first: false,
            });
            out.push(SettingsField::Action {
                label: "[Stop AISB conversation viewer]".to_string(),
                command: format!("omega kill {}", omega_core::aisb::MASTER_SESSION_NAME),
                confirm_first: true,
            });
        }
        SettingsSection::Telegram => match omega_core::monitor::OmegaTelegramConfig::try_read() {
            Ok(Some(cfg)) => {
                out.push(SettingsField::Info(format!("Enabled: {}", cfg.enabled)));
                out.push(SettingsField::Info(format!("Chat ID: {}", cfg.chat_id)));
                out.push(SettingsField::Info(format!("Relay: {}", cfg.relay_session)));
                out.push(SettingsField::Action {
                    label: "[Disconnect Telegram bot]".to_string(),
                    command: "omega telegram disconnect".to_string(),
                    confirm_first: true,
                });
                out.push(SettingsField::Action {
                    label: "[Run Telegram bot (foreground)]".to_string(),
                    command: "omega telegram run".to_string(),
                    confirm_first: true,
                });
            }
            Ok(None) => {
                out.push(SettingsField::Info(
                    "Not configured. Use the Setup action below or the Monitor tab [T]."
                        .to_string(),
                ));
                out.push(SettingsField::Action {
                    label: "[Set up Telegram bot] (opens wizard)".to_string(),
                    command: "__INTERNAL_TELEGRAM_SETUP__".to_string(),
                    confirm_first: false,
                });
            }
            Err(error) => {
                out.push(SettingsField::Info(format!(
                    "INVALID Telegram configuration: {error}"
                )));
                out.push(SettingsField::Action {
                    label: "[Inspect with omega telegram status]".to_string(),
                    command: "omega telegram status".to_string(),
                    confirm_first: false,
                });
            }
        },
    }

    // Common: every provider section gets an "Open homepage" info line
    if let Some(agent) = agent_for_section(section) {
        if let Some(home) = agent.homepage() {
            out.push(SettingsField::Info(format!("Homepage: {}", home)));
        }
    }

    out
}

fn install_actions_for(agent: omega_core::agents::Agent) -> Vec<SettingsField> {
    let mut out = Vec::new();
    let installed = agent_available_cached(agent);
    let badge = if installed {
        "[+] installed"
    } else {
        "[x] not installed"
    };
    out.push(SettingsField::Info(String::new()));
    out.push(SettingsField::Info(format!("Status: {}", badge)));
    if let Some(cmd) = agent.install_command() {
        out.push(SettingsField::Action {
            label: format!(
                "[{}] {}",
                if installed { "Re-install" } else { "Install" },
                agent.display_name()
            ),
            command: cmd.to_string(),
            // Installing is NON-destructive → launch on the FIRST Enter (a
            // double-Enter "confirm" made it feel like Enter did nothing). Only
            // uninstall (below) keeps the confirm gate.
            confirm_first: false,
        });
    }
    if installed {
        if let Some(cmd) = agent.uninstall_command() {
            out.push(SettingsField::Action {
                label: format!("[Uninstall] {}", agent.display_name()),
                command: cmd.to_string(),
                confirm_first: true,
            });
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    General,
    Theme,
    Install,
    Claude,
    Codex,
    Gemini,
    Antigravity,
    OpenRouter,
    Pi,
    Hermes,
    Glm,
    Kimi,
    Aisb,
    Telegram,
}

impl SettingsSection {
    pub fn all() -> &'static [SettingsSection] {
        &[
            SettingsSection::General,
            SettingsSection::Theme,
            SettingsSection::Install,
            SettingsSection::Claude,
            SettingsSection::Codex,
            SettingsSection::Gemini,
            SettingsSection::Antigravity,
            SettingsSection::OpenRouter,
            SettingsSection::Pi,
            SettingsSection::Hermes,
            SettingsSection::Glm,
            SettingsSection::Kimi,
            SettingsSection::Aisb,
            SettingsSection::Telegram,
        ]
    }
    pub fn label(&self) -> &'static str {
        match self {
            SettingsSection::General => "General",
            SettingsSection::Theme => "Theme",
            SettingsSection::Install => "Install agents",
            SettingsSection::Claude => "Claude (Anthropic)",
            SettingsSection::Codex => "Codex (OpenAI)",
            SettingsSection::Gemini => "Gemini (Google)",
            SettingsSection::Antigravity => "Antigravity (Google)",
            SettingsSection::OpenRouter => "OpenRouter",
            SettingsSection::Pi => "Pi (earendil-works)",
            SettingsSection::Hermes => "Hermes (Nous Research)",
            SettingsSection::Glm => "GLM (Z.AI)",
            SettingsSection::Kimi => "Kimi (Moonshot AI)",
            SettingsSection::Aisb => "AISB viewer (legacy)",
            SettingsSection::Telegram => "Telegram",
        }
    }
}

/// Left-hand rows of the Monitor GROUP inside the Settings tab (top group;
/// the Settings providers sit below it as a second group). Mirrors
/// `SettingsSection` / `InfoSection`: a section list on the left, the selected
/// section's detail on the right. The `Actions` section hosts the interactive
/// `MonitorAction` list. `AccountBilling` merges the connected-account view with
/// live billing; `Telegram` merges the bot config with the project group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorSection {
    AccountBilling,
    Telegram,
    Actions,
}

impl MonitorSection {
    pub fn all() -> &'static [MonitorSection] {
        &[
            MonitorSection::AccountBilling,
            MonitorSection::Telegram,
            MonitorSection::Actions,
        ]
    }
    pub fn label(&self) -> &'static str {
        match self {
            MonitorSection::AccountBilling => "Account & billing",
            MonitorSection::Telegram => "Telegram & projects",
            MonitorSection::Actions => "Actions",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorAction {
    Login,
    TelegramSetup,
    TelegramDisconnect,
    ProvisioningSetup,
    RefreshBilling,
    UpdateOmega,
}

impl MonitorAction {
    pub fn all() -> &'static [MonitorAction] {
        &[
            MonitorAction::Login,
            MonitorAction::TelegramSetup,
            MonitorAction::TelegramDisconnect,
            MonitorAction::ProvisioningSetup,
            MonitorAction::RefreshBilling,
            MonitorAction::UpdateOmega,
        ]
    }
    pub fn label(&self) -> &'static str {
        match self {
            MonitorAction::Login => "Login / re-auth Claude   (opens the guided OAuth session)",
            MonitorAction::TelegramSetup => "Set up Omega Telegram bot   (guided 3-step wizard)",
            MonitorAction::TelegramDisconnect => "Disconnect Telegram bot",
            MonitorAction::ProvisioningSetup => "Set up project provisioning keys   (Vercel/Convex/GitHub/Stripe)",
            MonitorAction::RefreshBilling => "Refresh billing now   (live OAuth usage check)",
            MonitorAction::UpdateOmega => "Update OmegaOS now   (pull + rebuild + reinstall — your ~/.omega state is preserved)",
        }
    }
    pub fn shortcut(&self) -> &'static str {
        match self {
            MonitorAction::Login => "L",
            MonitorAction::TelegramSetup => "T",
            MonitorAction::TelegramDisconnect => "D",
            MonitorAction::ProvisioningSetup => "P",
            MonitorAction::RefreshBilling => "B",
            MonitorAction::UpdateOmega => "U",
        }
    }
}

pub struct App {
    pub tab: Tab,
    pub sessions: Vec<SessionEntry>,
    /// Renderable rows including section headers (parallel to sessions, but
    /// includes Header variants between groups). Built by refresh().
    pub rows: Vec<SessionRow>,
    pub selected: usize,
    pub menu_selected: usize,
    /// Terminal mouse capture state. true → clickable menus + scroll; false →
    /// native terminal text selection / copy-paste. Toggled with Ctrl-T.
    pub mouse_capture: bool,
    /// Last rendered Menu-tab list area + a rendered-row→action-index map, so a
    /// mouse click can hit-test which action was clicked. `menu_fits` is false
    /// when the list is taller than the area (scrolled) — clicks are then ignored.
    pub menu_area: ratatui::layout::Rect,
    pub menu_rendered_actions: Vec<Option<usize>>,
    pub menu_fits: bool,
    /// Last rendered Sessions-tab geometry, same pattern as the Menu cache
    /// above: the REAL list/preview Rects (None when the responsive layout
    /// hides a panel — narrow single-column, fullscreen) plus a rendered-row
    /// → entry-index map (project headers excluded). Mouse hit-testing reads
    /// these instead of the old hardcoded `column >= 30` heuristic, which
    /// misrouted clicks on both wide terminals (25% list extends past col 30)
    /// and narrow ones (full-width list, col 30+ is still the list).
    /// `sessions_list_fits` is false when the list is taller than its area
    /// (scrolled by ListState) — row mapping is then unreliable and clicks
    /// only change panel focus.
    pub sessions_list_area: Option<ratatui::layout::Rect>,
    pub sessions_preview_area: Option<ratatui::layout::Rect>,
    pub sessions_rendered_rows: Vec<Option<usize>>,
    pub sessions_list_fits: bool,
    /// Selected Monitor section (left list). Indexes `MonitorSection::all()`.
    pub monitor_selected: usize,
    /// Cursor within the Monitor `Actions` section's `MonitorAction` list.
    pub monitor_action_selected: usize,
    pub settings_selected: usize,
    /// Which group of the Settings tab the cursor is on: 0 = Monitor group
    /// (`MonitorSection`/`monitor_selected`), 1 = Settings group
    /// (`SettingsSection`/`settings_selected`). The two groups share one
    /// continuous left list separated by a blank gap + `─── header ───`.
    pub settings_group: u8,
    /// Cursor within the focused Settings section's interactive field list.
    pub settings_field_selected: usize,
    /// Field awaiting a second Enter to confirm a destructive Action (the
    /// `confirm_first` flag). Cleared on navigation or section change.
    /// fix6-T1 (FIX-H pattern): carries the armed field's IDENTITY (its label)
    /// pinned at arm time alongside the index — the field list is re-derived
    /// live (`fields_for_section`), so a background install/uninstall finishing
    /// between arm and confirm can insert/remove rows and shift the index onto
    /// a DIFFERENT field. Fire sites and `armed_confirm_warning` verify the
    /// pinned label still matches `fields.get(idx)`, else disarm with a notice.
    pub settings_confirm_pending: Option<(usize, String)>,
    /// Two-press confirm for destructive menu items (KillAll / NuclearCleanup):
    /// first Enter arms it, second Enter on the same item fires.
    pub menu_confirm_pending: Option<MenuAction>,
    /// Session-list status badges (done/blocked) by name, refreshed each tick
    /// from done.json + worker-blocked signals.
    pub session_badges: std::collections::HashMap<String, omega_core::done::DoneStatus>,
    /// Active session-list filter (case-insensitive substring on name); None =
    /// show all. Applied at the data source in refresh() so navigation is
    /// unaffected — the list is simply narrower.
    pub session_filter: Option<String>,
    /// New-project wizard: chosen credential group for a client project (set in
    /// the cred-group step, read by the CreateProject handler). None = default.
    pub new_project_cred_group: Option<String>,
    /// Cursor in the System tab's left section list (`InfoSection::all()`).
    pub info_section_selected: usize,
    /// When the AI Agents sub-section is active, which agent is highlighted.
    pub info_agent_selected: usize,
    /// When the Documentation sub-section is active, which document is open.
    pub info_doc_selected: usize,
    /// The installed manual, discovered once at startup (a filesystem walk has
    /// no business running inside the 5s refresh tick or the render loop).
    pub docs: Vec<omega_core::docs::DocEntry>,
    /// Body of the document under `info_doc_selected`, loaded lazily and cached
    /// by relative path so arrowing through the list re-reads nothing.
    pub doc_body: Option<(String, String)>,
    /// The installed skills, discovered once at startup alongside `docs`.
    pub skills: Vec<omega_core::skill_registry::Skill>,
    /// Strict discovery failure, kept distinct from a genuinely empty catalog
    /// so the System tab never presents malformed authority as zero skills.
    pub skills_error: Option<String>,
    /// Set when a sub-cursor moves, so the renderer scrolls the detail panel to
    /// keep that row visible — for ONE frame. Any explicit scroll clears it.
    /// Without the flag the renderer re-snapped every frame, which pinned the
    /// panel to the document list and made the document BODY unreachable.
    pub detail_follow_cursor: bool,
    /// What the daily update cron last did. Re-read on the refresh tick (it is
    /// one small file) so the System tab shows the real state of the machine,
    /// not a snapshot from whenever the TUI happened to start.
    pub auto_update_state: omega_core::auto_update::AutoUpdateState,
    pub should_quit: bool,
    pub status_message: Option<String>,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub config: OmegaConfig,
    pub preview_content: String,
    /// Actual rendered inner size of the preview panel (text area, minus
    /// borders), written by the renderer each frame. main.rs reads this to
    /// resize the rmux pane to the EXACT panel width — computing from a
    /// terminal-width percentage was off by a few cols and clipped the
    /// agent's content on the right.
    pub preview_inner_width: u16,
    pub preview_inner_height: u16,
    /// Per-session Claude model + cumulative token count, shown on the right
    /// of the preview title. Keyed by session name; refreshed off the hot
    /// path (main.rs, throttled). `(short_model, tokens)`.
    pub session_meta: std::collections::HashMap<String, (String, u64)>,
    /// Per-session compact git status (e.g. "↑4h • main" or just "main")
    /// shown in the status bar on the Sessions tab instead of the Focus
    /// hint chatter — that lives in the Help tab. Cached + refreshed off
    /// the hot path (main.rs, ~10s throttle).
    pub session_git_status: std::collections::HashMap<String, String>,
    /// REAL cursor position from the pane snapshot (row, col, visible),
    /// zero-based within the visible pane. Used to paint the caret exactly
    /// where the agent's input cursor is, instead of guessing the last
    /// non-empty line. None when no session is previewed.
    pub preview_cursor: Option<(u16, u16, bool)>,
    /// Styled preview rows (fg/bg/bold per span) from the pane snapshot.
    /// Renders Claude's colored UI — crucially the `/` command-menu
    /// selection highlight that plain text drops. None when browsing
    /// scrollback (plain-text path).
    pub preview_styled: Option<Vec<omega_core::session::PreviewLine>>,
    /// Pane revision of the currently-cached styled preview. The capture is
    /// skipped (no restyle/flatten) when the pane's revision still matches this.
    pub preview_revision: u64,
    /// Session target that owns `preview_content` and the cached live preview.
    /// Bound once on a session switch so another target's frame is cleared and
    /// capture retries for the same target retain their failure streak.
    pub preview_session: Option<String>,
    /// Consecutive pane-capture failures for the current session. A pane in
    /// transition (a login pane swap, a zoom/terminal-resize reflow, a brief
    /// daemon hiccup) errors for a frame or two — clobbering the last-good
    /// preview with "(session has no pane content)" was the bug behind the
    /// view sticking on that message until a manual Ctrl+R. We keep the last
    /// good frame and force a recapture until this streak crosses a threshold,
    /// so only a GENUINELY dead pane shows the message.
    pub preview_fail_streak: u32,
    /// Scroll position measured as LINES UP FROM THE TAIL (0 = newest line).
    /// Bottom-anchored so the view stays stable when the capture buffer grows
    /// (visible-only → full scrollback history): "3 lines up from the tail"
    /// means the same thing whether the buffer is 50 or 1000 lines, so
    /// scrolling never jumps. Converted to a from-top offset at render time.
    pub preview_scroll: u16,
    /// Max scrollable lines (`content_lines - viewport_height`), written by
    /// the renderer each frame so the scroll setters can clamp against the
    /// real viewport without knowing the panel geometry themselves.
    pub preview_max_scroll: u16,
    /// Auto-follow tail — preview stays glued to the bottom (latest content)
    /// unless the user manually scrolls up. Mirrors `preview_scroll == 0`.
    pub preview_follow_tail: bool,
    /// Set true on the tail→history TRANSITION (first scroll-up out of tail
    /// mode). Signals the event loop to load full scrollback IMMEDIATELY —
    /// before the next draw — so the renderer computes a real `preview_max_scroll`
    /// on the SAME frame the user first scrolled. Without it, the first press
    /// sees the short visible buffer (max_scroll == 0) and the view can't move
    /// until the next cadence tick, requiring a wasted double-press. The loop
    /// clears it after consuming.
    pub preview_needs_history: bool,
    /// Session whose FULL scrollback is cached in `preview_content` while the
    /// user browses history. Set after a one-time deep capture on entry to
    /// history mode; reused on every subsequent frame so we don't re-capture
    /// the whole buffer at the idle cadence. Cleared when we return to the tail.
    pub preview_history_for: Option<String>,
    /// Styled rows for the SCROLLED-BACK preview, parsed out of
    /// `capture-pane -e`. Kept separate from `preview_styled` on purpose:
    /// that one is the live tail (exactly one viewport, cursor-aware, drives
    /// the menu-highlight heuristic and the no-scroll case), while this is a
    /// whole 500k-line history that the plain-text scroll math owns. Sharing
    /// one field would have made the history inherit the tail's assumptions.
    /// Without it, scrolling up dropped every color and the mirror of a Claude
    /// or Codex conversation turned grayscale.
    pub preview_history_styled: Option<Vec<omega_core::session::PreviewLine>>,
    /// Mouse drag-selection over the preview mirror (tmux-style: capture stays
    /// ON, so wheel-scroll keeps working while you select). Screen-absolute
    /// cell where the left button went down / last drag position.
    pub preview_select_anchor: Option<(u16, u16)>,
    pub preview_select_head: Option<(u16, u16)>,
    /// True once a Drag event arrived after the Down — distinguishes a
    /// selection gesture from a plain focus click.
    pub preview_select_dragging: bool,
    /// Plain text of the preview viewport rows exactly as last rendered
    /// (viewport-relative, one String per visible row). Written by the
    /// renderer each frame so button-release can resolve the drag rectangle
    /// to real text.
    pub preview_screen_rows: Vec<String>,
    /// Selected text waiting to be pushed to the terminal clipboard via
    /// OSC 52 (drained by the run loop; rmux forwards OSC 52 to the outer
    /// terminal when the TUI runs nested).
    pub pending_clipboard: Option<String>,
    pub session_focus: SessionFocus,
    /// Set when chat focus was dropped without a deliberate navigation key
    /// (the session vanished mid-typing) — destructive single-key hotkeys
    /// (q / x / Enter / Esc-quit) are ignored while inside the grace window
    /// so an in-flight keystream can't kill/quit (DESIGN-015 / NEW-3).
    pub focus_drop_at: Option<std::time::Instant>,
    /// Async-origin status notices (vanish, forwarder errors) keep a minimum
    /// display time (`STICKY_MS` from this set-instant): the keypress TTL
    /// must not clear them inside that window, or the typist they're
    /// addressed to never sees them (FIX-2).
    pub status_sticky_at: Option<std::time::Instant>,
    /// The exact message `status_sticky_at` protects (FIX-G/D-7): the TTL
    /// exemption applies only while `status_message` still holds this text,
    /// so a plain overwrite can't inherit a dangling sticky window.
    pub status_sticky_msg: Option<String>,
    /// Tracks the last Tab press for double-tap detection (any tab).
    pub last_tab_press: Option<std::time::Instant>,
    /// Focus state at the START of a Tab sequence, captured on the first tap so
    /// a following double-tap can toggle the left menu cleanly (the single tap
    /// already moved focus, so the double tap reverts to this and toggles).
    pub tab_seq_start: Option<SessionFocus>,
    /// Generic right-panel focus for non-Sessions 2-column tabs (Settings/Info).
    /// false = list focused, true = detail focused.
    pub detail_focused: bool,
    /// Detail panel fullscreen (Tab-Tab on a 2-column tab).
    pub detail_fullscreen: bool,
    /// Scroll position for the detail panel in Settings/Info/Monitor.
    pub detail_scroll: u16,
    /// Max scrollable offset for the detail panel (`content_lines -
    /// panel_height`), written by the renderer each frame — the same contract
    /// as `preview_max_scroll`. End and the scroll setters clamp against it:
    /// an unclamped offset (the old `u16::MAX / 2` End) scrolled the Paragraph
    /// thousands of lines past its content, rendering an empty panel that only
    /// Home could recover.
    pub detail_max_scroll: u16,
    pub current_session: Option<String>,
    /// Projects tab — selected project index.
    pub projects_selected: usize,
    /// Cached project registry for the Projects tab.
    pub project_registry: omega_core::project_manager::ProjectRegistry,
    /// OS tab — selected operative-system index.
    pub os_selected: usize,
    /// Cached OS-suite entries (loaded on tab entry / F5 — fs stat only).
    pub os_entries: Vec<omega_core::os_products::OsEntry>,
    /// Two-press confirm for "Delete forever" (Projects tab 'D'): holds the
    /// project name armed by the first press; second 'D' on the same name fires
    /// the destructive HardDeleteProject. Cleared on cursor move, Esc, and tab
    /// switch (FIX-B — the Esc cancel was advertised but unwired before fix5).
    pub project_delete_pending: Option<String>,
    /// Two-press confirm for the Monitor Telegram section's Enter→disconnect.
    /// Armed by the first focused-Enter, fired by the second. Cleared on nav.
    pub monitor_disconnect_armed: bool,
    /// Lazily-loaded providers config. Settings reads this on every keystroke,
    /// so we cache it here and only reload from disk after an edit/toggle
    /// commit (see `invalidate_providers`). Avoids per-keystroke disk I/O.
    providers_cache: Option<omega_core::providers::ProvidersConfig>,
    /// Claude OAuth re-login progress (Monitor → Account). Updated by the async
    /// engine via a shared sink drained each tick in the main loop.
    pub reauth_status: ReauthStatus,
}

fn resolve_current_session_with<E, D>(mut env_var: E, mut display_session: D) -> Option<String>
where
    E: FnMut(&str) -> Option<String>,
    D: FnMut(&str) -> Option<String>,
{
    fn nonempty(value: String) -> Option<String> {
        let value = value.trim();
        (!value.is_empty()).then(|| value.to_string())
    }

    env_var("RMUX_PANE")
        .and_then(nonempty)
        .and_then(|pane| display_session(&pane))
        .and_then(nonempty)
        .or_else(|| env_var("RMUX_SESSION").and_then(nonempty))
        // Legacy fallback for users coming from a tmux setup
        .or_else(|| env_var("TMUX_SESSION").and_then(nonempty))
}

impl App {
    pub fn new(config: OmegaConfig) -> Self {
        let current_session = resolve_current_session_with(
            |name| std::env::var(name).ok(),
            |pane| {
                let output = std::process::Command::new("rmux")
                    .args(["display-message", "-p", "-t", pane, "#{session_name}"])
                    .output()
                    .ok()?;
                if !output.status.success() {
                    return None;
                }
                String::from_utf8(output.stdout).ok()
            },
        );
        let (skills, skills_error) =
            match omega_core::skill_registry::SkillRegistry::discover_default() {
                Ok(registry) => (registry.list().into_iter().cloned().collect(), None),
                Err(error) => (Vec::new(), Some(error.to_string())),
            };

        Self {
            tab: Tab::Sessions,
            sessions: Vec::new(),
            rows: Vec::new(),
            selected: 0,
            menu_selected: 0,
            mouse_capture: true,
            menu_area: ratatui::layout::Rect::default(),
            menu_rendered_actions: Vec::new(),
            menu_fits: true,
            sessions_list_area: None,
            sessions_preview_area: None,
            sessions_rendered_rows: Vec::new(),
            sessions_list_fits: false,
            monitor_selected: 0,
            monitor_action_selected: 0,
            settings_selected: 0,
            settings_group: 0,
            settings_field_selected: 0,
            settings_confirm_pending: None,
            menu_confirm_pending: None,
            session_badges: std::collections::HashMap::new(),
            session_filter: None,
            new_project_cred_group: None,
            info_section_selected: 0,
            info_agent_selected: 0,
            info_doc_selected: 0,
            // One walk at startup, never in the refresh tick: the manual and
            // the skill arsenal only change on install/update.
            docs: omega_core::docs::discover(),
            doc_body: None,
            skills,
            skills_error,
            detail_follow_cursor: false,
            auto_update_state: omega_core::auto_update::AutoUpdateState::load(&config.state_dir),
            should_quit: false,
            status_message: None,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            config,
            preview_content: String::new(),
            preview_styled: None,
            preview_revision: 0,
            preview_session: None,
            preview_fail_streak: 0,
            preview_inner_width: 0,
            preview_inner_height: 0,
            session_meta: std::collections::HashMap::new(),
            session_git_status: std::collections::HashMap::new(),
            preview_cursor: None,
            preview_scroll: 0,
            preview_max_scroll: 0,
            preview_follow_tail: true,
            preview_needs_history: false,
            preview_history_for: None,
            preview_history_styled: None,
            preview_select_anchor: None,
            preview_select_head: None,
            preview_select_dragging: false,
            preview_screen_rows: Vec::new(),
            pending_clipboard: None,
            session_focus: SessionFocus::List,
            focus_drop_at: None,
            status_sticky_at: None,
            status_sticky_msg: None,
            last_tab_press: None,
            tab_seq_start: None,
            detail_focused: false,
            detail_fullscreen: false,
            detail_scroll: 0,
            detail_max_scroll: 0,
            current_session,
            projects_selected: 0,
            project_registry: omega_core::project_manager::ProjectRegistry::load(),
            os_selected: 0,
            // Same lazy contract as marketing: filled on first OS-tab entry.
            os_entries: Vec::new(),
            project_delete_pending: None,
            monitor_disconnect_armed: false,
            providers_cache: None,
            reauth_status: ReauthStatus::Idle,
        }
    }

    /// Cached providers config, cloned out for the caller. The disk read +
    /// TOML parse (the expensive part) happens once; subsequent calls clone
    /// the in-memory struct, which is cheap relative to per-keystroke I/O.
    /// A clone (not a borrow) sidesteps the simultaneous `&app.config` borrow
    /// that `fields_for_section` needs.
    pub fn providers(&mut self) -> omega_core::providers::ProvidersConfig {
        if self.providers_cache.is_none() {
            self.providers_cache = Some(omega_core::providers::ProvidersConfig::load());
        }
        self.providers_cache.as_ref().unwrap().clone()
    }

    /// Drop the cached providers so the next `providers()` re-reads from disk.
    /// Call after any commit/toggle that mutates `~/.omega/providers.toml`.
    pub fn invalidate_providers(&mut self) {
        self.providers_cache = None;
    }

    pub fn refresh_projects(&mut self) {
        self.project_registry = omega_core::project_manager::ProjectRegistry::load();
        // Selection index 0 is the pinned "OS System" quick-access row; a
        // project lives at 1..=len (registry index = projects_selected - 1).
        // Clamp to the last valid row (len, the last project) if we overshoot.
        let max = self.project_registry.projects.len();
        if self.projects_selected > max {
            self.projects_selected = max;
        }
    }

    /// The selected project, or None when the pinned "OS System" row (index 0)
    /// is the current selection — that row is a shortcut to the OS tab, not a
    /// project, so every project action gracefully no-ops on it.
    pub fn selected_project(&self) -> Option<&omega_core::project_manager::ManagedProject> {
        if self.projects_selected == 0 {
            return None;
        }
        self.project_registry
            .projects
            .get(self.projects_selected - 1)
    }

    /// True when the pinned "OS System" quick-access row (always index 0 of the
    /// Projects list) is selected. Enter on it jumps straight to the OS tab.
    pub fn projects_os_pinned(&self) -> bool {
        self.projects_selected == 0
    }

    /// (Re)load the OS-suite entries (OS tab entry / F5). Registry is static;
    /// the fs stat per OS is cheap, so a full rebuild is fine.
    pub fn refresh_os(&mut self) {
        self.os_entries = omega_core::os_products::list_os_entries();
        if self.os_selected >= self.os_entries.len() {
            self.os_selected = self.os_entries.len().saturating_sub(1);
        }
    }

    pub fn selected_os_entry(&self) -> Option<&omega_core::os_products::OsEntry> {
        self.os_entries.get(self.os_selected)
    }

    pub fn os_tab_next(&mut self) {
        if self.os_entries.is_empty() {
            return;
        }
        self.os_selected = (self.os_selected + 1) % self.os_entries.len();
        self.on_os_nav_change();
    }

    pub fn os_tab_prev(&mut self) {
        if self.os_entries.is_empty() {
            return;
        }
        self.os_selected = if self.os_selected == 0 {
            self.os_entries.len() - 1
        } else {
            self.os_selected - 1
        };
        self.on_os_nav_change();
    }

    fn on_os_nav_change(&mut self) {
        self.detail_scroll = 0;
        self.detail_follow_cursor = false;
    }

    /// Select a session by name (used after creating a session to auto-focus it).
    pub fn select_by_name(&mut self, name: &str) -> bool {
        for (i, entry) in self.sessions.iter().enumerate() {
            if entry.session.name == name {
                self.selected = i;
                return true;
            }
        }
        false
    }

    /// Enter chat focus on the currently selected session.
    pub fn enter_chat_focus(&mut self) {
        self.session_focus = SessionFocus::Chat;
        // Tab-less focus change — clear the chord so a Tab right after isn't
        // misread as the second tap of a Tab-Tab (AF-7 contract, FIX-4).
        self.reset_tab_chord();
        self.preview_follow_tail = true;
        self.preview_scroll = 0;
    }

    /// Tab-less return to the session list: focus + chord reset in one place
    /// (FIX-4) so every non-Tab writer keeps the chord contract below true.
    pub fn set_list_focus(&mut self) {
        self.session_focus = SessionFocus::List;
        self.reset_tab_chord();
    }

    /// Jump the selection to the next session flagged Blocked or Failed
    /// (wraps around). Returns the name jumped to, if any.
    pub fn jump_to_next_flagged(&mut self) -> Option<String> {
        use omega_core::done::DoneStatus;
        let n = self.sessions.len();
        if n == 0 {
            return None;
        }
        let mut target = None;
        for off in 1..=n {
            let idx = (self.selected + off) % n;
            if matches!(
                self.session_badges.get(&self.sessions[idx].session.name),
                Some(DoneStatus::Blocked) | Some(DoneStatus::Failed)
            ) {
                target = Some(idx);
                break;
            }
        }
        let idx = target?;
        self.selected = idx;
        Some(self.sessions[idx].session.name.clone())
    }

    /// Clear the Tab double-tap chord state. Any non-Tab path that changes
    /// session focus (chat Esc, chat Ctrl+X, a vanished session's forced drop
    /// to the list) must call this, or a Tab pressed within the 400ms window
    /// right after is misread as the SECOND tap of a chord and lands in
    /// ChatFullscreen instead of navigating (AF-7).
    pub fn reset_tab_chord(&mut self) {
        self.last_tab_press = None;
        self.tab_seq_start = None;
    }

    /// F-7 clear-on-input TTL — called by the event loop on every key press
    /// and mouse Down (NEW-6) BEFORE dispatch. Exemption:
    /// - Async-origin sticky notices (FIX-2/NEW-2): a vanish notice or
    ///   forwarder error targets a user mid-typing; their in-flight keystroke
    ///   must not consume the message addressed to them. Time-based minimum
    ///   display instead of the keypress TTL.
    ///
    /// Armed two-press confirms need no exemption here (fix6-T9c): their
    /// warnings are STATE-driven via `armed_confirm_warning()` (FIX-A), which
    /// `draw_status_bar` renders with priority — clearing `status_message`
    /// can't hide them, and this fn never touches the armed state itself.
    pub fn consume_status_ttl(&mut self) {
        if let Some(at) = self.status_sticky_at {
            // FIX-G (D-7): the window belongs to the exact message
            // set_status_sticky wrote. A plain `status_message = Some(..)`
            // overwrite inside the window must NOT inherit the exemption —
            // the dangling window would TTL-shield the WRONG message.
            if self.status_message == self.status_sticky_msg && within(at, STICKY_MS) {
                return;
            }
            self.status_sticky_at = None;
            self.status_sticky_msg = None;
        }
        self.status_message = None;
    }

    /// Set an async-origin status notice with a minimum display time so it
    /// survives in-flight keystrokes (FIX-2). Nav hints and key-triggered
    /// acks keep the plain keypress TTL (`status_message = Some(..)`).
    pub fn set_status_sticky(&mut self, msg: String) {
        self.status_message = Some(msg.clone());
        // FIX-G (D-7): remember WHICH message the window belongs to, so a
        // plain overwrite inside the window doesn't inherit the exemption.
        self.status_sticky_msg = Some(msg);
        self.status_sticky_at = Some(std::time::Instant::now());
    }

    /// True while an async-origin sticky notice is inside its minimum display
    /// window (FIX-G/D-8): the event loop's per-tab hint seeding must not
    /// overwrite it. fix7-T3: like the msg-pinned TTL check above, the window
    /// only protects the EXACT message it was set for — once `status_message`
    /// was overwritten, a dangling window must not shield the new text.
    pub fn status_sticky_unexpired(&self) -> bool {
        self.status_sticky_msg.is_some()
            && self.status_message == self.status_sticky_msg
            && self.status_sticky_at.is_some_and(|t| within(t, STICKY_MS))
    }

    /// fix6-T8: render-side sticky expiry. `consume_status_ttl` only runs on
    /// input, so for an IDLE operator an expired async notice masked the
    /// Sessions git segment indefinitely. `draw_status_bar` calls this each
    /// frame: once the deadline has passed — and `status_message` still holds
    /// the exact message the deadline protects (FIX-G) — the pair clears so
    /// `git_text` resumes without a keypress. An overwritten message keeps the
    /// normal keypress TTL (the user is demonstrably at the keyboard).
    pub fn expire_sticky_status(&mut self) {
        if let Some(at) = self.status_sticky_at {
            if !within(at, STICKY_MS) && self.status_message == self.status_sticky_msg {
                self.status_message = None;
                self.status_sticky_at = None;
                self.status_sticky_msg = None;
            }
        }
    }

    /// FIX-A (fix5): single source of truth for "an armed two-press confirm
    /// is live", covering ALL four armed states. `draw_status_bar` renders
    /// this with priority over `status_message`, so the warning is
    /// STATE-DRIVEN — TTL-immune and overwrite-immune by construction. The
    /// entire R-1/R-2/D-3/D-4 class (launcher prompts, Ctrl-T, paste, sticky
    /// forwarder errors overwriting an armed warning) dies here: as long as
    /// the state is armed, the warning is on screen, whatever else wrote to
    /// the status line.
    pub fn armed_confirm_warning(&mut self) -> Option<String> {
        if let Some(action) = self.menu_confirm_pending {
            let verb = if matches!(action, MenuAction::NuclearCleanup) {
                "NUCLEAR CLEANUP (kill all + prune state + free RAM)"
            } else {
                "KILL ALL sessions"
            };
            return Some(format!(
                "[!] {} — press Enter again to CONFIRM, Esc to cancel",
                verb
            ));
        }
        if let Some(name) = &self.project_delete_pending {
            return Some(format!(
                "Press D again to DELETE LOCAL MACHINE '{}' (OmegaOS + kill oracle + rm -rf LOCAL FOLDER; GitHub kept) — Esc to cancel",
                name
            ));
        }
        if let Some((idx, pinned)) = self.settings_confirm_pending.clone() {
            // Re-derive the field at the armed index and verify it is STILL
            // the field that was armed (fix6-T1): the list is rebuilt live,
            // so a background [Install] completing between arm and confirm
            // can shift rows under the index. A silent re-label here would
            // make the warning lie about what the confirming Enter fires.
            let section = self.selected_settings_section();
            let providers = self.providers();
            let fields = fields_for_section(section, &providers, &self.config);
            match fields.get(idx) {
                Some(f) if f.label() == pinned => {
                    return Some(match f {
                        SettingsField::EditText { label, .. } => {
                            format!("Press x again to clear: {} (Esc to cancel)", label.trim())
                        }
                        f => format!(
                            "Press Enter again to confirm: {} (Esc to cancel)",
                            f.label().trim()
                        ),
                    });
                }
                _ => {
                    // The armed field moved or vanished — disarm with a
                    // notice instead of re-labeling onto a different field.
                    self.settings_confirm_pending = None;
                    self.status_message =
                        Some("Confirm cancelled — the settings list changed".to_string());
                }
            }
        }
        if self.monitor_disconnect_armed {
            return Some(
                "Press Enter again to DISCONNECT the Telegram bot (Esc to cancel)".to_string(),
            );
        }
        None
    }

    /// DESIGN-015: true while inside the destructive-hotkey grace that
    /// follows a non-deliberate focus drop (the focused session vanished).
    pub fn in_post_drop_grace(&self) -> bool {
        self.focus_drop_at.is_some_and(|t| within(t, GRACE_MS))
    }

    /// A deliberate navigation key ends the grace window early — the user is
    /// demonstrably interacting with the list.
    pub fn end_post_drop_grace(&mut self) {
        self.focus_drop_at = None;
    }

    /// Handle a Tab press in the Sessions tab.
    ///   • Single Tab  → NAVIGATE: toggle the session list ↔ the session itself
    ///     (List ↔ Chat; from fullscreen → back to the list).
    ///   • Tab-Tab (rapid, <400ms) → toggle the LEFT SESSION MENU: hide it so the
    ///     Claude session takes the full width (great on small screens), or show
    ///     it again. The toggle keys off the state the sequence STARTED in, so it
    ///     is a clean hide↔show even though the first tap already moved focus.
    /// Called from BOTH the list handler and the chat handler so the behavior is
    /// identical wherever the sequence begins.
    pub fn handle_tab_in_sessions(&mut self) {
        let is_double = self
            .last_tab_press
            .is_some_and(|t| within(t, DOUBLE_TAP_MS));

        if is_double {
            // Second tap: revert the first tap's navigation and toggle the menu.
            self.last_tab_press = None;
            let start = self.tab_seq_start.take().unwrap_or(self.session_focus);
            self.session_focus = match start {
                SessionFocus::ChatFullscreen => SessionFocus::List, // show the menu
                _ => SessionFocus::ChatFullscreen,                  // hide the menu
            };
        } else {
            // First tap: remember the starting state, then navigate.
            self.tab_seq_start = Some(self.session_focus);
            self.last_tab_press = Some(std::time::Instant::now());
            self.session_focus = match self.session_focus {
                SessionFocus::List => SessionFocus::Chat,
                _ => SessionFocus::List,
            };
        }

        if self.session_focus != SessionFocus::List {
            self.preview_follow_tail = true;
            self.preview_scroll = 0;
        }
    }

    /// Tab in a 2-column tab (Settings / Info): single = toggle list↔detail,
    /// double = fullscreen detail, double again = back to list.
    pub fn handle_tab_in_2col(&mut self) {
        let is_double = self
            .last_tab_press
            .is_some_and(|t| within(t, DOUBLE_TAP_MS));
        self.last_tab_press = Some(std::time::Instant::now());

        if is_double {
            // Double tap: toggle fullscreen
            self.detail_fullscreen = !self.detail_fullscreen;
            self.detail_focused = self.detail_fullscreen;
        } else {
            if self.detail_fullscreen {
                // Single tap from fullscreen → back to list
                self.detail_fullscreen = false;
                self.detail_focused = false;
            } else {
                // Toggle list ↔ detail
                self.detail_focused = !self.detail_focused;
            }
        }
    }

    /// Reset 2-column focus to list when switching tabs.
    pub fn reset_2col_focus(&mut self) {
        self.detail_focused = false;
        self.detail_fullscreen = false;
        self.detail_scroll = 0;
    }

    pub fn scroll_detail_down(&mut self, lines: u16) {
        // Clamp to the renderer-published bound: an unbounded saturating_add
        // let mouse wheels accumulate thousands of invisible offset lines past
        // the end that then had to be scrolled back through.
        self.detail_scroll = self
            .detail_scroll
            .saturating_add(lines)
            .min(self.detail_max_scroll);
        self.detail_follow_cursor = false;
    }
    pub fn scroll_detail_up(&mut self, lines: u16) {
        self.detail_scroll = self.detail_scroll.saturating_sub(lines);
        self.detail_follow_cursor = false;
    }

    /// Cancel an in-flight mouse selection. Called when the view shifts under
    /// the gesture (wheel scroll, session switch) — the anchor is a SCREEN
    /// cell, so once the content moves the highlight would lie about what
    /// release will copy.
    pub fn clear_preview_selection(&mut self) {
        self.preview_select_anchor = None;
        self.preview_select_head = None;
        self.preview_select_dragging = false;
    }

    /// Resolve the current drag rectangle to the normalized viewport range
    /// `((start_col,start_row),(end_col,end_row))`, rows/cols 0-based inside
    /// the preview borders, start ≤ end in row-major order.
    pub fn preview_selection_viewport(&self) -> Option<((usize, usize), (usize, usize))> {
        let area = self.sessions_preview_area?;
        let (ac, ar) = self.preview_select_anchor?;
        let (hc, hr) = self.preview_select_head?;
        let to_vp = |c: u16, r: u16| {
            (
                (c.max(area.x + 1).min(area.x + area.width.saturating_sub(2)) - (area.x + 1))
                    as usize,
                (r.max(area.y + 1)
                    .min(area.y + area.height.saturating_sub(2))
                    - (area.y + 1)) as usize,
            )
        };
        let a = to_vp(ac, ar);
        let h = to_vp(hc, hr);
        // Row-major normalize: (row, col) ordering decides direction.
        if (h.1, h.0) < (a.1, a.0) {
            Some((h, a))
        } else {
            Some((a, h))
        }
    }

    /// Resolve the finished drag to the selected TEXT (from the screen rows
    /// the renderer captured) and clear the selection. Single-row drags slice
    /// one line; multi-row drags take first row from start-col, middle rows
    /// whole, last row up to end-col — terminal-selection semantics. Rows are
    /// right-trimmed so the padded cells don't become trailing spaces.
    pub fn take_preview_selection_text(&mut self) -> Option<String> {
        let sel = self.preview_selection_viewport();
        self.clear_preview_selection();
        let ((sc, sr), (ec, er)) = sel?;
        let mut out: Vec<String> = Vec::new();
        for r in sr..=er {
            let Some(row) = self.preview_screen_rows.get(r) else {
                break;
            };
            let from = if r == sr { sc } else { 0 };
            let to = if r == er { ec + 1 } else { usize::MAX };
            out.push(slice_display_cols(row, from, to).trim_end().to_string());
        }
        let text = out.join("\n");
        if text.trim().is_empty() {
            None
        } else {
            Some(text)
        }
    }

    // Scroll is measured from the tail: 0 = newest. "Down" moves toward the
    // newest line (decreasing the from-tail offset), "up" moves into history.

    pub fn scroll_preview_down(&mut self, lines: u16) {
        self.clear_preview_selection();
        self.preview_scroll = self.preview_scroll.saturating_sub(lines);
        // Reaching the tail re-glues to live follow (and lets refresh_preview
        // switch back to the cheap visible-only capture).
        if self.preview_scroll == 0 {
            self.preview_follow_tail = true;
        }
    }

    pub fn scroll_preview_up(&mut self, lines: u16) {
        self.clear_preview_selection();
        // When leaving tail mode, preview_max_scroll is 0 (content == viewport).
        // Skip the clamp on this first transition so the scroll position actually
        // advances; the next renderer tick loads full history and sets a real
        // preview_max_scroll that subsequent presses clamp against.
        let was_following = self.preview_follow_tail;
        self.preview_follow_tail = false;
        if was_following && self.preview_max_scroll == 0 {
            self.preview_scroll = lines;
            // First press out of tail: the visible buffer is short, so the
            // renderer would clamp the view back to the bottom. Tell the loop
            // to load scrollback NOW so this same press actually moves the view.
            self.preview_needs_history = true;
        } else {
            self.preview_scroll = self
                .preview_scroll
                .saturating_add(lines)
                .min(self.preview_max_scroll);
        }
    }

    pub fn scroll_preview_home(&mut self) {
        // Top of available history.
        self.preview_scroll = self.preview_max_scroll;
        self.preview_follow_tail = false;
    }

    pub fn scroll_preview_end(&mut self) {
        self.preview_scroll = 0;
        self.preview_follow_tail = true;
    }

    pub fn select_menu_next(&mut self) {
        let count = MenuAction::all().len();
        self.menu_selected = (self.menu_selected + 1) % count;
    }

    pub fn select_menu_prev(&mut self) {
        let count = MenuAction::all().len();
        self.menu_selected = if self.menu_selected == 0 {
            count - 1
        } else {
            self.menu_selected - 1
        };
    }

    pub fn selected_menu_action(&self) -> MenuAction {
        MenuAction::all()[self.menu_selected]
    }

    // ── Monitor section list (left panel) ───────────────────────────────────
    /// True when the Settings-tab cursor sits on a Monitor-group row.
    pub fn settings_on_monitor(&self) -> bool {
        self.settings_group == 0
    }

    /// Side effects shared by every Settings-tab cursor move: reset the action
    /// cursor, scroll, disconnect-arm, field cursor and confirm-arm.
    fn on_settings_nav_change(&mut self) {
        self.monitor_action_selected = 0;
        self.detail_scroll = 0;
        self.monitor_disconnect_armed = false;
        self.settings_field_selected = 0;
        self.settings_confirm_pending = None;
    }

    /// Advance the unified Settings-tab cursor: walks the Monitor group, then
    /// the Settings group, then wraps back to the top of the Monitor group.
    pub fn settings_tab_next(&mut self) {
        let mlen = MonitorSection::all().len();
        let slen = SettingsSection::all().len();
        if self.settings_group == 0 {
            if self.monitor_selected + 1 < mlen {
                self.monitor_selected += 1;
            } else {
                self.settings_group = 1;
                self.settings_selected = 0;
            }
        } else if self.settings_selected + 1 < slen {
            self.settings_selected += 1;
        } else {
            self.settings_group = 0;
            self.monitor_selected = 0;
        }
        self.on_settings_nav_change();
    }

    pub fn settings_tab_prev(&mut self) {
        let mlen = MonitorSection::all().len();
        let slen = SettingsSection::all().len();
        if self.settings_group == 0 {
            if self.monitor_selected > 0 {
                self.monitor_selected -= 1;
            } else {
                self.settings_group = 1;
                self.settings_selected = slen.saturating_sub(1);
            }
        } else if self.settings_selected > 0 {
            self.settings_selected -= 1;
        } else {
            self.settings_group = 0;
            self.monitor_selected = mlen.saturating_sub(1);
        }
        self.on_settings_nav_change();
    }

    pub fn selected_monitor_section(&self) -> MonitorSection {
        MonitorSection::all()[self.monitor_selected.min(MonitorSection::all().len() - 1)]
    }

    // ── Monitor action cursor (inside the Actions section) ───────────────────
    pub fn select_monitor_action_next(&mut self) {
        let count = MonitorAction::all().len();
        self.monitor_action_selected = (self.monitor_action_selected + 1) % count;
    }

    pub fn select_monitor_action_prev(&mut self) {
        let count = MonitorAction::all().len();
        self.monitor_action_selected = if self.monitor_action_selected == 0 {
            count - 1
        } else {
            self.monitor_action_selected - 1
        };
    }

    pub fn selected_monitor_action(&self) -> MonitorAction {
        MonitorAction::all()[self.monitor_action_selected]
    }

    pub fn selected_settings_section(&self) -> SettingsSection {
        SettingsSection::all()[self.settings_selected]
    }

    /// Reset field cursor when section changes
    pub fn reset_settings_field(&mut self) {
        self.settings_field_selected = 0;
    }

    pub fn select_settings_field_next(&mut self, max: usize) {
        if max == 0 {
            return;
        }
        self.settings_field_selected = (self.settings_field_selected + 1) % max;
    }
    pub fn select_settings_field_prev(&mut self, max: usize) {
        if max == 0 {
            return;
        }
        self.settings_field_selected = if self.settings_field_selected == 0 {
            max - 1
        } else {
            self.settings_field_selected - 1
        };
    }

    fn prepare_preview_session_switch(&mut self, name: &str) {
        if self.preview_session.as_deref() != Some(name) {
            self.clear_preview_selection();
            self.preview_screen_rows.clear();
            self.preview_content.clear();
            self.preview_styled = None;
            self.preview_cursor = None;
            self.preview_revision = 0;
            self.preview_fail_streak = 0;
            self.preview_session = Some(name.to_string());
        }
    }

    fn record_preview_failure(
        &mut self,
        name: &str,
        capture_kind: &str,
        error: &dyn std::fmt::Display,
    ) {
        self.preview_fail_streak = self.preview_fail_streak.saturating_add(1);
        self.preview_revision = 0;
        if self.preview_fail_streak >= 3 {
            if self.preview_fail_streak == 3 {
                omega_core::tuilog::log(format!(
                    "preview: {capture_kind} capture for '{name}' failed 3 consecutive ticks — showing placeholder; last error: {error}"
                ));
            }
            self.preview_content = String::from("(session has no pane content)");
            self.preview_styled = None;
            self.preview_cursor = None;
        }
    }

    pub async fn refresh_preview(&mut self) -> anyhow::Result<()> {
        let name = match self.selected_session() {
            Some(e) => e.session.name.clone(),
            None => {
                // No session selected → nothing can fail. Zero the per-session
                // failure counter so a dead session's streak can't poison the
                // next session that becomes selected.
                self.clear_preview_selection();
                self.preview_screen_rows.clear();
                self.preview_content = String::new();
                self.preview_styled = None;
                self.preview_cursor = None;
                self.preview_revision = 0;
                self.preview_fail_streak = 0;
                self.preview_session = None;
                self.preview_history_for = None;
                self.preview_history_styled = None;
                return Ok(());
            }
        };

        self.prepare_preview_session_switch(&name);

        // Avoid recursion: if previewing the session we're running inside, show static msg
        if let Some(ref cur) = self.current_session {
            if cur == &name {
                self.preview_content =
                    "(this is the session running OmegaOS — preview disabled to prevent recursion)"
                        .to_string();
                // Drop the styled snapshot too — the renderer prefers it over
                // preview_content, so leaving it showed the PREVIOUSLY
                // selected session's frozen mirror under this session's title.
                self.preview_styled = None;
                self.preview_cursor = None;
                self.preview_history_for = None;
                self.preview_history_styled = None;
                return Ok(());
            }
        }

        // Hot tail path stays on the cheap visible-only snapshot. Only when the
        // user is browsing history (follow_tail == false) do we pay for a full
        // scrollback capture, so there is real content above the screen to
        // scroll into instead of an empty void.
        if self.preview_follow_tail {
            // Back on the tail: drop any cached scrollback so the next scroll-up
            // re-captures fresh history (picking up lines added since). The
            // styled rows are part of that cache and must die with it, or the
            // renderer would paint stale history over the live tail.
            self.preview_history_for = None;
            self.preview_history_styled = None;
            // A missing daemon is a preview capture failure, not a reason to
            // fail the whole TUI refresh. This also keeps cache-only unit tests
            // hermetic instead of requiring a real rmux binary.
            let mgr = match omega_core::session::SessionManager::connect_cached().await {
                Ok(manager) => manager,
                Err(error) => {
                    self.record_preview_failure(&name, "styled", &error);
                    return Ok(());
                }
            };
            // Tail path: capture STYLED rows + text + REAL cursor together.
            // Styled rows carry the `/` selector highlight + Claude's
            // colored UI; plain text is kept as a fallback + for scroll math.
            // Revision gate: only pay the full restyle+flatten when the pane
            // actually changed. Force a fresh capture (since=0) on a session
            // switch so the new pane's content always loads.
            let cache_valid = self.preview_styled.is_some()
                && self.preview_session.as_deref() == Some(name.as_str());
            let since = if cache_valid {
                self.preview_revision
            } else {
                0
            };
            match mgr.capture_pane_styled(&name, since).await {
                // Pane unchanged since last render — keep the cached preview,
                // skip the ~10k-cell restyle + the text flatten entirely.
                Ok(omega_core::session::StyledCapture::Unchanged) => {
                    self.preview_fail_streak = 0;
                }
                Ok(omega_core::session::StyledCapture::Changed {
                    rows,
                    cursor_row,
                    cursor_col,
                    cursor_visible,
                    revision,
                }) => {
                    // Flatten styled rows to plain text for scroll/cursor math.
                    self.preview_content = rows
                        .iter()
                        .map(|line| line.iter().map(|s| s.text.as_str()).collect::<String>())
                        .collect::<Vec<_>>()
                        .join("\n");
                    self.preview_styled = Some(rows);
                    self.preview_cursor = Some((cursor_row, cursor_col, cursor_visible));
                    self.preview_revision = revision;
                    self.preview_session = Some(name.clone());
                    self.preview_fail_streak = 0;
                }
                Err(e) => {
                    // A capture can fail transiently while a pane is in
                    // transition: a claude-login pane swap, a zoom / terminal
                    // resize reflow, or a brief daemon hiccup. Do NOT flash
                    // "(session has no pane content)" and stick on it — keep the
                    // last good frame and force a full recapture next tick
                    // (revision=0). Only a SUSTAINED failure (≥3 ticks) — i.e. a
                    // genuinely dead pane — replaces the view with the message.
                    self.record_preview_failure(&name, "styled", &e);
                    // else: retain this target's last-good frame, or the cleared
                    // frame prepared above when this tick switched sessions.
                }
            }
        } else {
            // History-browsing path: plain text (scrollback has no styling),
            // cursor meaningless when scrolled back.
            self.preview_styled = None;
            self.preview_cursor = None;
            // Lazy history: capture the full retained scrollback ONCE on entry to
            // history-browsing for this session, then reuse it while scrolling —
            // instead of re-capturing the whole buffer every cadence tick (which
            // pegged the daemon while browsing). The tail path clears the cache,
            // so the next scroll-up gets a fresh deep capture. Depth matches the
            // rmux history-limit so the user can scroll to the very top.
            if self.preview_history_for.as_deref() != Some(name.as_str()) {
                let mgr = match omega_core::session::SessionManager::connect_cached().await {
                    Ok(manager) => manager,
                    Err(error) => {
                        self.preview_history_for = None;
                        self.preview_history_styled = None;
                        self.record_preview_failure(&name, "history", &error);
                        return Ok(());
                    }
                };
                match mgr.capture_pane_history(&name, 500_000).await {
                    Ok(content) => {
                        // The capture now carries its attributes (-e), so split
                        // it once into styled rows for the renderer and stripped
                        // text for everything downstream that must never see an
                        // escape byte: the scroll math, and the drag-select copy.
                        let (rows, plain) = omega_core::session::styled_rows_from_ansi(&content);
                        self.preview_content = plain;
                        self.preview_history_styled = Some(rows);
                        self.preview_history_for = Some(name.clone());
                        self.preview_fail_streak = 0;
                    }
                    Err(e) => {
                        // Same sticky-last-good policy as the tail path: a
                        // transient capture error must not clobber the view.
                        self.record_preview_failure(&name, "history", &e);
                        self.preview_history_for = None;
                        self.preview_history_styled = None;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn refresh(&mut self) -> anyhow::Result<()> {
        // Snapshot user-toggled protection flags BEFORE clearing self.sessions
        // — otherwise every 5s refresh wipes the lock the user just set.
        let protected_names: std::collections::HashSet<String> = self
            .sessions
            .iter()
            .filter(|e| e.is_protected)
            .map(|e| e.session.name.clone())
            .collect();

        // F-1 (critical): snapshot the selected session's NAME before the rebuild
        // — `self.selected` is a bare index into a list any other actor can
        // reshuffle (session create/kill between refreshes), so an index-only
        // clamp silently retargets chat-focused keystrokes to whatever session
        // lands on the old row.
        let selected_name: Option<String> = self
            .sessions
            .get(self.selected)
            .map(|e| e.session.name.clone());

        // Snapshot for the post-rebuild diff log: when sessions appear in /
        // vanish from the list, the WHY ends up in ~/.omega/logs/tui.log.
        let prev_names: std::collections::HashSet<String> = self
            .sessions
            .iter()
            .map(|e| e.session.name.clone())
            .collect();

        // Cached daemon socket — refresh runs every ~2s, so a fresh connect()
        // each time is wasteful. Matches refresh_preview()'s connect_cached().
        let mgr = SessionManager::connect_cached().await?;
        let raw_sessions = mgr.list_sessions().await?;

        // Hide infrastructure daemons (Telegram bridge, reauth helper).
        // Same list as the Telegram bridge filters in /sessions — keep them
        // in sync if you add a new background process.
        // aisb-reauth is intentionally VISIBLE: when the operator triggers a
        // Claude login it must show up in the sessions table so they can see the
        // login session open, run /login, and close on success.
        let hidden_prefixes = ["omega-telegram-bridge"];
        let filter_lc = self.session_filter.as_ref().map(|q| q.to_lowercase());
        let mut sessions: Vec<_> = raw_sessions
            .into_iter()
            .filter(|s| !hidden_prefixes.iter().any(|p| s.name.starts_with(p)))
            .filter(|s| match &filter_lc {
                Some(q) => s.name.to_lowercase().contains(q.as_str()),
                None => true,
            })
            .collect();
        // Group-by below only cuts when the section CHANGES between two adjacent
        // rows, so an unsorted list makes one project surface as several separate
        // blocks scattered down the tab. Sort by section first — Home, then the
        // projects alphabetically, then System/Other — and inside a section put
        // the oracle above the workers it dispatched, each set name-ordered.
        sessions.sort_by(|a, b| {
            section_rank(a)
                .cmp(&section_rank(b))
                .then_with(|| section_for(a).cmp(&section_for(b)))
                .then_with(|| role_rank(a).cmp(&role_rank(b)))
                .then_with(|| a.name.cmp(&b.name))
        });

        // Status badges (done/blocked) for the list — read once per refresh.
        // A worker-blocked signal wins over a stale done.json.
        self.session_badges.clear();
        for d in omega_core::done::DoneSignal::read_all(&self.config.state_dir) {
            self.session_badges.insert(d.session, d.status);
        }
        for b in omega_core::done::WorkerBlocked::read_all(&self.config.state_dir) {
            self.session_badges
                .insert(b.session, omega_core::done::DoneStatus::Blocked);
        }

        // One small JSON, beside the progress files already read here — so the
        // System tab reports what the cron actually did last night, live.
        self.auto_update_state =
            omega_core::auto_update::AutoUpdateState::load(&self.config.state_dir);

        let all_progress = ProgressInfo::read_all(&self.config.state_dir);

        // Worker → governing-oracle map, read from each oracle's persisted
        // state (spawn-worker records the link there). Lets the menu nest a
        // worker under the SPECIFIC oracle that spawned it, not just its
        // project. Workers with no recorded parent fall back (single-oracle
        // project → that oracle; multi-oracle → project level — no guessing).
        let mut worker_oracle: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for st in omega_core::oracle_lifecycle::OracleState::read_all(&self.config.state_dir) {
            for w in &st.workers {
                worker_oracle.insert(w.session_name.clone(), st.oracle_name.clone());
            }
        }

        self.sessions.clear();
        self.rows.clear();

        // The retired `aisb-master` rmux session (a legacy Telegram-conversation
        // viewer) is no longer pinned in the menu — the brain is the Atlas Telegram
        // bot now. It is filtered out of the project sections below too, so it never
        // appears in the session list.

        // ── Section 2: Project-grouped sessions (Oracles + Workers + Home) ──
        let mut last_section: Option<String> = None;
        let mut group: Vec<&OmegaSession> = Vec::new();

        for session in sessions.iter() {
            if omega_core::aisb::is_viewer(&session.name) {
                continue;
            }
            let section_label = section_for(session);
            if last_section.as_ref() != Some(&section_label) && !group.is_empty() {
                self.flush_group_rows(
                    &group,
                    &all_progress,
                    &worker_oracle,
                    last_section.as_deref(),
                );
                group.clear();
            }
            group.push(session);
            last_section = Some(section_label);
        }
        if !group.is_empty() {
            self.flush_group_rows(
                &group,
                &all_progress,
                &worker_oracle,
                last_section.as_deref(),
            );
        }

        // Restore the user's manual protection toggles
        for entry in self.sessions.iter_mut() {
            if protected_names.contains(&entry.session.name) {
                entry.is_protected = true;
            }
        }
        // Same for the rendered row clones
        for row in self.rows.iter_mut() {
            if let SessionRow::Entry(e) = row {
                if protected_names.contains(&e.session.name) {
                    e.is_protected = true;
                }
            }
        }

        // View-diff log: record every appearance/disappearance with the list
        // size, so "the interface lost my session" has a forensic trail.
        let now_names: std::collections::HashSet<String> = self
            .sessions
            .iter()
            .map(|e| e.session.name.clone())
            .collect();
        if now_names != prev_names && !prev_names.is_empty() {
            let added: Vec<&str> = now_names
                .difference(&prev_names)
                .map(String::as_str)
                .collect();
            let removed: Vec<&str> = prev_names
                .difference(&now_names)
                .map(String::as_str)
                .collect();
            omega_core::tuilog::log(format!(
                "session list changed: +{added:?} -{removed:?} → {} listed",
                now_names.len()
            ));
        }

        // F-1: re-anchor the selection by NAME so a list mutation never moves
        // it off the session the user is looking at (or chatting with).
        self.reanchor_selection(selected_name.as_deref());

        Ok(())
    }

    /// F-1 re-anchor after a list rebuild: restore the selection to
    /// `selected_name`; when that session vanished, drop a chat-focused user
    /// back to the list (their keystream was aimed at the dead session) and
    /// clamp the index. Sync + daemon-free so it is directly testable.
    fn reanchor_selection(&mut self, selected_name: Option<&str>) {
        let reanchored = match selected_name {
            Some(name) => self.select_by_name(name),
            None => false,
        };
        if !reanchored {
            // The selected session vanished (or nothing was selected). If the
            // user was chat-focused, their keystream was aimed at the dead
            // session — drop to the list instead of silently retargeting
            // whatever session now occupies the old index.
            if matches!(
                self.session_focus,
                SessionFocus::Chat | SessionFocus::ChatFullscreen
            ) {
                // Tab-less forced drop: set_list_focus clears the chord state
                // so an immediate Tab isn't misread as a double-tap (AF-7).
                omega_core::tuilog::log(format!(
                    "selected session {:?} vanished while chat-focused — dropped to list ({} listed)",
                    selected_name.unwrap_or("<none>"),
                    self.sessions.len()
                ));
                self.set_list_focus();
                // The user may still be typing at the dead session — open the
                // destructive-hotkey grace so the in-flight keystream can't
                // land on q/x/Enter in list mode (DESIGN-015 / NEW-3).
                self.focus_drop_at = Some(std::time::Instant::now());
                if let Some(name) = selected_name {
                    // Async-origin notice: minimum display time so the same
                    // in-flight keystroke can't consume it (FIX-2 / NEW-2).
                    self.set_status_sticky(format!("{name} ended — back to list"));
                }
            }
            // Index fallback: keep `selected` in bounds when the anchor is gone.
            if self.selected >= self.sessions.len() && !self.sessions.is_empty() {
                self.selected = self.sessions.len() - 1;
            }
        }
    }

    fn flush_group_rows(
        &mut self,
        group: &[&OmegaSession],
        all_progress: &[ProgressInfo],
        worker_oracle: &std::collections::HashMap<String, String>,
        section_label: Option<&str>,
    ) {
        if let Some(label) = section_label {
            self.rows.push(SessionRow::Header(format!("─ {} ─", label)));
        }

        let oracles: Vec<&OmegaSession> = group
            .iter()
            .copied()
            .filter(|s| s.role == SessionRole::Oracle)
            .collect();
        let workers: Vec<&OmegaSession> = group
            .iter()
            .copied()
            .filter(|s| s.role == SessionRole::Worker)
            .collect();
        let others: Vec<&OmegaSession> = group
            .iter()
            .copied()
            .filter(|s| s.role != SessionRole::Oracle && s.role != SessionRole::Worker)
            .collect();

        // Resolve a worker's governing oracle: the recorded link first; else,
        // only when the project has exactly ONE oracle, that sole oracle; else
        // None (unattributed — never guessed between several oracles).
        let sole_oracle: Option<&str> = if oracles.len() == 1 {
            Some(oracles[0].name.as_str())
        } else {
            None
        };
        let parent_of = |w: &OmegaSession| -> Option<String> {
            worker_oracle
                .get(&w.name)
                .cloned()
                .or_else(|| sole_oracle.map(|s| s.to_string()))
        };

        // Each oracle, immediately followed by the workers it governs (├/└).
        let mut shown: std::collections::HashSet<String> = std::collections::HashSet::new();
        for &oracle in &oracles {
            self.push_session_row(oracle, all_progress, String::new());
            let mine: Vec<&OmegaSession> = workers
                .iter()
                .copied()
                .filter(|w| parent_of(w).as_deref() == Some(oracle.name.as_str()))
                .collect();
            let n = mine.len();
            for (i, w) in mine.into_iter().enumerate() {
                let prefix = if i + 1 == n { "  └ " } else { "  ├ " };
                self.push_session_row(w, all_progress, prefix.to_string());
                shown.insert(w.name.clone());
            }
        }

        // Workers with no resolvable parent in this group (multi-oracle project
        // with no recorded link, or a killed oracle): project level, after the
        // oracles. Honest — we show them rather than hide or misattribute them.
        for &w in &workers {
            if !shown.contains(&w.name) {
                self.push_session_row(w, all_progress, String::new());
            }
        }

        // Anything else in the section (e.g. a stray shell classified here).
        for &o in &others {
            self.push_session_row(o, all_progress, String::new());
        }
    }

    /// Append one session as a list row (+ its progress + tree prefix).
    /// `is_protected` is restored by the caller after the whole group is built.
    fn push_session_row(
        &mut self,
        session: &OmegaSession,
        all_progress: &[ProgressInfo],
        tree_prefix: String,
    ) {
        let progress = all_progress
            .iter()
            .find(|p| p.session == session.name)
            .cloned();
        let entry = SessionEntry {
            session: session.clone(),
            progress,
            is_current: false,
            is_protected: false,
            tree_prefix,
        };
        self.sessions.push(entry);
        self.rows.push(SessionRow::Entry(
            self.sessions.last().unwrap().clone_for_row(),
        ));
    }

    pub fn selected_session(&self) -> Option<&SessionEntry> {
        self.sessions.get(self.selected)
    }

    /// Shared tab-switch hygiene: leaving a tab cancels EVERY armed two-press
    /// confirm (FIX-1 + FIX-B — an armed destructive state must not survive
    /// into a tab where its context is invisible) and clears the Tab chord so
    /// it can't leak into another tab's double-tap detection (FIX-4 —
    /// `handle_tab_in_2col` shares `last_tab_press`).
    pub(crate) fn leave_tab(&mut self) {
        self.menu_confirm_pending = None;
        self.project_delete_pending = None;
        self.settings_confirm_pending = None;
        self.monitor_disconnect_armed = false;
        self.reset_tab_chord();
    }

    pub fn next_tab(&mut self) {
        self.leave_tab();
        let i = self.tab.index();
        self.tab = Tab::ORDER[(i + 1) % Tab::ORDER.len()];
    }

    pub fn prev_tab(&mut self) {
        self.leave_tab();
        let i = self.tab.index();
        self.tab = Tab::ORDER[(i + Tab::ORDER.len() - 1) % Tab::ORDER.len()];
    }

    pub fn select_info_next(&mut self) {
        let count = InfoSection::all().len();
        self.info_section_selected = (self.info_section_selected + 1) % count;
        self.on_info_nav_change();
    }

    pub fn select_info_prev(&mut self) {
        let count = InfoSection::all().len();
        self.info_section_selected = if self.info_section_selected == 0 {
            count - 1
        } else {
            self.info_section_selected - 1
        };
        self.on_info_nav_change();
    }

    /// Every System-tab section change resets the sub-cursors and the scroll —
    /// landing halfway down the previous section's text reads as a broken panel.
    fn on_info_nav_change(&mut self) {
        self.info_agent_selected = 0;
        self.info_doc_selected = 0;
        self.detail_scroll = 0;
        // Agents and Docs render a submenu in the detail pane. Auto-focus it
        // so ↑/↓ picks an agent or a manual page instead of silently walking
        // the section list (the dead-menu bug).
        self.detail_focused = self.selected_info_section().has_sub_cursor();
    }

    pub fn selected_info_section(&self) -> InfoSection {
        InfoSection::all()[self.info_section_selected.min(InfoSection::all().len() - 1)]
    }

    /// Number of navigable rows in the Projects list: the pinned "OS System"
    /// quick-access row (always index 0) plus one row per registered project.
    /// Always ≥1, so on an empty registry the pinned row stays selectable.
    fn projects_len(&self) -> usize {
        self.project_registry.projects.len() + 1
    }

    /// Advance the Projects-tab cursor. Index 0 is the pinned OS-suite shortcut,
    /// 1..=N the projects; the cursor wraps across the whole list.
    pub fn projects_tab_next(&mut self) {
        let plen = self.projects_len();
        self.projects_selected = (self.projects_selected + 1) % plen;
        self.on_projects_nav_change();
    }

    pub fn projects_tab_prev(&mut self) {
        let plen = self.projects_len();
        self.projects_selected = if self.projects_selected == 0 {
            plen - 1
        } else {
            self.projects_selected - 1
        };
        self.on_projects_nav_change();
    }

    fn on_projects_nav_change(&mut self) {
        self.detail_scroll = 0;
        self.project_delete_pending = None;
    }

    /// Documents in the manual, ≥1 so the "(no docs installed)" placeholder
    /// row stays selectable.
    fn docs_len(&self) -> usize {
        self.docs.len().max(1)
    }

    pub fn select_info_doc_next(&mut self) {
        let len = self.docs_len();
        self.info_doc_selected = (self.info_doc_selected + 1) % len;
        self.detail_follow_cursor = true;
    }

    pub fn select_info_doc_prev(&mut self) {
        let len = self.docs_len();
        self.info_doc_selected = if self.info_doc_selected == 0 {
            len - 1
        } else {
            self.info_doc_selected - 1
        };
        self.detail_follow_cursor = true;
    }

    /// The document under the cursor, if any is installed.
    pub fn selected_doc(&self) -> Option<&omega_core::docs::DocEntry> {
        self.docs.get(
            self.info_doc_selected
                .min(self.docs.len().saturating_sub(1)),
        )
    }

    /// Body of the selected document, read from disk on first request and
    /// cached by relative path. Returns None when no docs are installed.
    pub fn selected_doc_body(&mut self) -> Option<&str> {
        let (rel, path) = {
            let doc = self.selected_doc()?;
            (doc.rel_path.clone(), doc.path.clone())
        };
        let stale = !matches!(&self.doc_body, Some((cached, _)) if *cached == rel);
        if stale {
            self.doc_body = Some((rel, omega_core::docs::read_body(&path)));
        }
        self.doc_body.as_ref().map(|(_, body)| body.as_str())
    }

    pub fn select_info_agent_next(&mut self) {
        let count = omega_core::aisb_agents::AisbAgent::all().len();
        self.info_agent_selected = (self.info_agent_selected + 1) % count;
        self.detail_follow_cursor = true;
    }
    pub fn select_info_agent_prev(&mut self) {
        let count = omega_core::aisb_agents::AisbAgent::all().len();
        self.info_agent_selected = if self.info_agent_selected == 0 {
            count - 1
        } else {
            self.info_agent_selected - 1
        };
        self.detail_follow_cursor = true;
    }

    pub fn select_next(&mut self) {
        if !self.sessions.is_empty() {
            self.selected = (self.selected + 1) % self.sessions.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.sessions.is_empty() {
            self.selected = if self.selected == 0 {
                self.sessions.len() - 1
            } else {
                self.selected - 1
            };
        }
    }
}

/// Slice a row by DISPLAY columns `[from, to)` — emoji/CJK occupy 2 cells, so
/// byte/char indexing would cut the wrong region. A wide char is included
/// when its starting cell falls inside the range (terminal-selection feel).
pub(crate) fn slice_display_cols(row: &str, from: usize, to: usize) -> String {
    use unicode_width::UnicodeWidthChar;
    let mut col = 0usize;
    let mut out = String::new();
    for ch in row.chars() {
        let w = ch.width().unwrap_or(0);
        if col >= to {
            break;
        }
        if col >= from {
            out.push(ch);
        }
        col += w;
    }
    out
}

#[cfg(test)]
mod selection_tests {
    use super::slice_display_cols;

    #[test]
    fn ascii_range() {
        assert_eq!(slice_display_cols("hello world", 6, 11), "world");
        assert_eq!(slice_display_cols("hello", 0, usize::MAX), "hello");
        assert_eq!(slice_display_cols("hello", 7, 9), "");
    }

    #[test]
    fn wide_chars_count_two_cells() {
        // "日本" = 4 cells; slicing cells [2,4) must yield the second char.
        assert_eq!(slice_display_cols("日本x", 2, 4), "本");
        // A wide char starting inside the range is included whole.
        assert_eq!(slice_display_cols("a日b", 1, 3), "日");
    }
}

#[cfg(test)]
mod nesting_tests {
    use super::*;
    use omega_core::config::OmegaConfig;
    use omega_core::session::OmegaSession;

    fn rows_of(app: &App) -> Vec<(String, String)> {
        app.rows
            .iter()
            .filter_map(|r| match r {
                SessionRow::Entry(e) => Some((e.tree_prefix.clone(), e.session.name.clone())),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn single_oracle_nests_all_its_workers() {
        // One oracle in the project → every worker hangs under it (├/└), even
        // with no recorded link (single-oracle fallback). This is the live
        // DentistryGPT case once only one oracle remains.
        let mut app = App::new(OmegaConfig::default());
        let sessions = [
            OmegaSession::classify("oracle-DentistryGPT-2"),
            OmegaSession::classify("DentistryGPT-worker-agent-actions"),
            OmegaSession::classify("DentistryGPT-worker-e2e-agents"),
        ];
        let group: Vec<&OmegaSession> = sessions.iter().collect();
        let map = std::collections::HashMap::new();
        app.flush_group_rows(&group, &[], &map, Some("DentistryGPT"));
        let rows = rows_of(&app);
        assert_eq!(rows[0], (String::new(), "oracle-DentistryGPT-2".into()));
        assert_eq!(rows[1].1, "DentistryGPT-worker-agent-actions");
        assert!(rows[1].0.contains('├'), "expected ├, got {:?}", rows[1].0);
        assert_eq!(rows[2].1, "DentistryGPT-worker-e2e-agents");
        assert!(rows[2].0.contains('└'), "expected └, got {:?}", rows[2].0);
    }

    #[test]
    fn recorded_links_nest_each_worker_under_its_own_oracle() {
        // Two oracles in one project: the recorded worker→oracle map decides
        // which worker nests under which oracle (no guessing).
        let mut app = App::new(OmegaConfig::default());
        let sessions = [
            OmegaSession::classify("oracle-Causio-1"),
            OmegaSession::classify("oracle-Causio-2"),
            OmegaSession::classify("Causio-worker-a"),
            OmegaSession::classify("Causio-worker-b"),
        ];
        let group: Vec<&OmegaSession> = sessions.iter().collect();
        let mut map = std::collections::HashMap::new();
        map.insert("Causio-worker-a".to_string(), "oracle-Causio-2".to_string());
        map.insert("Causio-worker-b".to_string(), "oracle-Causio-1".to_string());
        app.flush_group_rows(&group, &[], &map, Some("Causio"));
        let order: Vec<String> = rows_of(&app).into_iter().map(|(_, n)| n).collect();
        assert_eq!(
            order,
            vec![
                "oracle-Causio-1",
                "Causio-worker-b", // its recorded child
                "oracle-Causio-2",
                "Causio-worker-a", // its recorded child
            ]
        );
    }
}

#[cfg(test)]
mod reanchor_tests {
    use super::*;
    use omega_core::config::OmegaConfig;
    use omega_core::session::OmegaSession;

    fn entry(name: &str) -> SessionEntry {
        SessionEntry {
            session: OmegaSession::classify(name),
            progress: None,
            is_current: false,
            is_protected: false,
            tree_prefix: String::new(),
        }
    }

    /// Daemon-free App with `names` as the session list (no rmux needed —
    /// same pattern as nesting_tests above).
    fn app_with_sessions(names: &[&str]) -> App {
        let mut app = App::new(OmegaConfig::default());
        app.tab = Tab::Sessions;
        app.sessions = names.iter().map(|n| entry(n)).collect();
        app
    }

    #[test]
    fn reanchor_keeps_selection_on_name_across_list_mutation() {
        // AF-2/CA-3 regression (F-1): a session appearing above the selection
        // shifts every index — the NAME anchor must survive the shift.
        let mut app = app_with_sessions(&["a", "b", "c"]);
        app.selected = 1; // "b"
        app.sessions.insert(0, entry("new-arrival"));
        app.reanchor_selection(Some("b"));
        assert_eq!(app.selected, 2);
        assert_eq!(app.selected_session().unwrap().session.name, "b");
    }

    #[test]
    fn reanchor_clamps_index_when_the_name_vanished() {
        let mut app = app_with_sessions(&["a", "b"]);
        app.selected = 5; // stale out-of-range index from the bigger old list
        app.reanchor_selection(Some("gone"));
        assert_eq!(app.selected, 1, "fall back to the last in-bounds row");
    }

    #[test]
    fn chat_on_vanished_session_drops_to_list_and_clears_chord() {
        // AF-7 mirror: the forced Chat→List drop is a Tab-less focus change,
        // so it must also clear the double-tap chord state.
        let mut app = app_with_sessions(&["a"]);
        app.selected = 0;
        app.session_focus = SessionFocus::Chat;
        app.last_tab_press = Some(std::time::Instant::now());
        app.tab_seq_start = Some(SessionFocus::List);
        app.reanchor_selection(Some("gone"));
        assert_eq!(app.session_focus, SessionFocus::List);
        assert!(
            app.last_tab_press.is_none(),
            "chord timestamp must be cleared"
        );
        assert!(app.tab_seq_start.is_none(), "chord start must be cleared");
        assert!(
            app.status_message
                .as_deref()
                .unwrap_or("")
                .contains("gone ended"),
            "vanish notice must be set, got {:?}",
            app.status_message
        );
    }

    // DESIGN-015 + FIX-2: the forced vanish-drop must open the destructive-
    // hotkey grace AND make its notice sticky (survive the keypress TTL).
    #[test]
    fn vanish_drop_opens_grace_and_sticky_notice() {
        let mut app = app_with_sessions(&["a"]);
        app.selected = 0;
        app.session_focus = SessionFocus::Chat;
        app.reanchor_selection(Some("gone"));
        assert!(
            app.in_post_drop_grace(),
            "vanish drop must start the grace window"
        );
        // The in-flight keystroke's TTL must NOT consume the vanish notice.
        app.consume_status_ttl();
        assert!(
            app.status_message
                .as_deref()
                .unwrap_or("")
                .contains("gone ended"),
            "sticky vanish notice must survive the keypress TTL, got {:?}",
            app.status_message
        );
    }

    // FIX-1 + FIX-2 TTL exemptions, plus normal-path clear and expiry.
    #[test]
    fn status_ttl_exempts_armed_confirm_and_unexpired_sticky() {
        let mut app = app_with_sessions(&["a"]);
        // Plain message → cleared by the TTL.
        app.status_message = Some("hint".into());
        app.consume_status_ttl();
        assert!(
            app.status_message.is_none(),
            "plain hints keep the keypress TTL"
        );
        // Armed destructive-menu confirm: the TTL clears the MESSAGE (fix6-T9c
        // removed the FIX-1 exemption — FIX-A made it redundant) but never the
        // STATE — the warning stays on screen via armed_confirm_warning().
        app.menu_confirm_pending = Some(MenuAction::KillAll);
        app.status_message = Some("[!] KILL ALL".into());
        app.consume_status_ttl();
        assert!(
            app.status_message.is_none(),
            "T9c: no keypress-TTL exemption for armed menu confirms"
        );
        assert_eq!(
            app.menu_confirm_pending,
            Some(MenuAction::KillAll),
            "TTL must never disarm"
        );
        assert!(
            app.armed_confirm_warning()
                .unwrap_or_default()
                .contains("KILL ALL"),
            "the state-driven warning must still render"
        );
        app.menu_confirm_pending = None;
        // Expired sticky → cleared like a plain message.
        app.set_status_sticky("async notice".into());
        app.status_sticky_at =
            std::time::Instant::now().checked_sub(std::time::Duration::from_millis(STICKY_MS + 1));
        app.consume_status_ttl();
        assert!(app.status_message.is_none(), "expired sticky must clear");
    }

    // FIX-A (fix5): the armed-confirm warning is STATE-driven. For ALL FOUR
    // armed sites the keypress TTL may clear status_message — the warning
    // must still be available to draw_status_bar via armed_confirm_warning(),
    // so it stays on screen until confirm/cancel whatever else happens.
    #[test]
    fn armed_confirm_warning_covers_all_four_sites_and_survives_ttl() {
        let mut app = app_with_sessions(&["a"]);

        // 1. Menu KillAll / NuclearCleanup.
        app.menu_confirm_pending = Some(MenuAction::KillAll);
        app.status_message = None; // simulate a wiped/overwritten status line
        app.consume_status_ttl();
        assert!(
            app.armed_confirm_warning()
                .unwrap_or_default()
                .contains("KILL ALL"),
            "menu warning must be state-derived"
        );
        app.menu_confirm_pending = None;

        // 2. Projects 'D' delete-forever (rm -rf class).
        app.project_delete_pending = Some("Demo".into());
        app.consume_status_ttl();
        let warn = app.armed_confirm_warning().unwrap_or_default();
        assert!(
            warn.contains("Demo") && warn.contains("rm -rf"),
            "project-delete warning must name the project and the harm, got {warn:?}"
        );
        app.project_delete_pending = None;

        // 3. Settings destructive-action / clear-field confirm — armed with
        // the field's pinned IDENTITY (fix6-T1), so the warning only renders
        // while the field at the index is still the armed one.
        let real_label = {
            let section = app.selected_settings_section();
            let providers = app.providers();
            let fields = fields_for_section(section, &providers, &app.config);
            fields[0].label().to_string()
        };
        app.settings_confirm_pending = Some((0, real_label));
        app.consume_status_ttl();
        assert!(
            app.armed_confirm_warning().is_some(),
            "settings confirm must render a state-driven warning"
        );
        app.settings_confirm_pending = None;

        // 4. Monitor Telegram disconnect.
        app.monitor_disconnect_armed = true;
        app.consume_status_ttl();
        assert!(
            app.armed_confirm_warning()
                .unwrap_or_default()
                .contains("DISCONNECT"),
            "monitor disconnect warning must be state-derived"
        );
        app.monitor_disconnect_armed = false;

        assert!(
            app.armed_confirm_warning().is_none(),
            "no armed state → no warning"
        );
    }

    // fix6-T1: when the field list shifts under an armed settings confirm
    // (background [Install] finishing inserts/removes rows), the warning must
    // NOT silently re-label onto the field now at the index — it disarms with
    // a notice instead.
    #[test]
    fn settings_confirm_warning_disarms_on_identity_mismatch() {
        let mut app = app_with_sessions(&["a"]);
        app.settings_confirm_pending = Some((0, "[Uninstall] ghost-agent".into()));
        let warn = app.armed_confirm_warning();
        assert!(
            warn.is_none(),
            "a shifted field must not render a re-labeled warning"
        );
        assert_eq!(
            app.settings_confirm_pending, None,
            "identity mismatch must disarm the confirm"
        );
        assert!(
            app.status_message
                .as_deref()
                .unwrap_or("")
                .contains("cancelled"),
            "the disarm must be announced, got {:?}",
            app.status_message
        );
    }

    // fix6-T8: an expired async sticky notice must clear render-side (no
    // keypress needed) so it can't mask the git segment for an idle operator.
    #[test]
    fn expired_sticky_clears_render_side_without_input() {
        let mut app = app_with_sessions(&["a"]);
        app.set_status_sticky("async notice".into());
        // Unexpired → kept.
        app.expire_sticky_status();
        assert_eq!(app.status_message.as_deref(), Some("async notice"));
        // Expired + message still the protected one → the pair clears.
        app.status_sticky_at =
            std::time::Instant::now().checked_sub(std::time::Duration::from_millis(STICKY_MS + 1));
        app.expire_sticky_status();
        assert!(
            app.status_message.is_none(),
            "expired sticky must clear without input"
        );
        assert!(app.status_sticky_at.is_none() && app.status_sticky_msg.is_none());
        // Expired but OVERWRITTEN by a plain key-triggered hint → the hint
        // keeps the normal keypress TTL (the user is at the keyboard).
        app.set_status_sticky("stale".into());
        app.status_sticky_at =
            std::time::Instant::now().checked_sub(std::time::Duration::from_millis(STICKY_MS + 1));
        app.status_message = Some("fresh hint".into());
        app.expire_sticky_status();
        assert_eq!(app.status_message.as_deref(), Some("fresh hint"));
    }

    // fix7-T3: the hint-seeding guard mirrors the FIX-G identity rule — the
    // sticky window only shields the EXACT message it was set for. A plain
    // overwrite must not inherit a dangling window (the old deadline-only
    // check TTL-shielded WHATEVER text happened to be on the status line).
    #[test]
    fn sticky_unexpired_requires_message_identity() {
        let mut app = app_with_sessions(&["a"]);
        assert!(!app.status_sticky_unexpired(), "no sticky set → no shield");
        app.set_status_sticky("async notice".into());
        assert!(
            app.status_sticky_unexpired(),
            "a live, intact sticky must block per-tab hint seeding"
        );
        // Overwritten inside the window → the shield must drop with it.
        app.status_message = Some("per-tab hint".into());
        assert!(
            !app.status_sticky_unexpired(),
            "an overwritten message must not inherit the sticky window"
        );
        // Identity restored but window expired → no shield either.
        app.status_message = Some("async notice".into());
        app.status_sticky_at =
            std::time::Instant::now().checked_sub(std::time::Duration::from_millis(STICKY_MS + 1));
        assert!(
            !app.status_sticky_unexpired(),
            "an expired window must not shield"
        );
    }
}

#[cfg(test)]
mod preview_cache_tests {
    use super::*;
    use crate::ui::draw_sessions_right;
    use omega_core::config::OmegaConfig;
    use omega_core::session::{OmegaSession, PreviewSpan};
    use ratatui::{backend::TestBackend, Terminal};

    fn render_sessions_preview_on(terminal: &mut Terminal<TestBackend>, app: &mut App) -> String {
        terminal
            .draw(|frame| {
                let area = frame.area();
                draw_sessions_right(frame, app, area, false);
            })
            .expect("render sessions preview");
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }

    fn render_sessions_preview(app: &mut App) -> String {
        let mut terminal = Terminal::new(TestBackend::new(140, 10)).expect("test terminal");
        render_sessions_preview_on(&mut terminal, app)
    }

    #[test]
    fn current_session_resolution_prefers_rmux_pane_name_over_rmux_tuple() {
        let mut queried = Vec::new();
        let resolved = resolve_current_session_with(
            |name| {
                queried.push(name.to_string());
                match name {
                    "RMUX" => Some("/tmp/rmux-1004/default,4242,%7".to_string()),
                    "RMUX_PANE" => Some("%7".to_string()),
                    "RMUX_SESSION" => Some("rmux-session-fallback".to_string()),
                    "TMUX_SESSION" => Some("tmux-session-fallback".to_string()),
                    _ => None,
                }
            },
            |pane| {
                assert_eq!(pane, "%7");
                Some("omegaos-worker-preview-session-identity".to_string())
            },
        );

        assert_eq!(
            resolved.as_deref(),
            Some("omegaos-worker-preview-session-identity")
        );
        assert!(
            !queried.iter().any(|name| name == "RMUX"),
            "the socket,pid,pane tuple must never participate in identity resolution"
        );

        let rmux_fallback = resolve_current_session_with(
            |name| match name {
                "RMUX_PANE" => Some("  ".to_string()),
                "RMUX_SESSION" => Some(" rmux-session-fallback ".to_string()),
                "TMUX_SESSION" => Some("tmux-session-fallback".to_string()),
                _ => None,
            },
            |_| panic!("an empty pane identifier must not be resolved"),
        );
        assert_eq!(rmux_fallback.as_deref(), Some("rmux-session-fallback"));

        let legacy_fallback = resolve_current_session_with(
            |name| match name {
                "RMUX_PANE" => Some("%9".to_string()),
                "RMUX_SESSION" => Some("  ".to_string()),
                "TMUX_SESSION" => Some(" tmux-session-fallback ".to_string()),
                _ => None,
            },
            |_| Some("\n".to_string()),
        );
        assert_eq!(legacy_fallback.as_deref(), Some("tmux-session-fallback"));
    }

    #[tokio::test]
    async fn refresh_preview_clears_styled_history_before_static_current_session() {
        let selected_name = "session-b";
        let mut app = App::new(OmegaConfig::default());
        app.sessions = vec![SessionEntry {
            session: OmegaSession::classify(selected_name),
            progress: None,
            is_current: false,
            is_protected: false,
            tree_prefix: String::new(),
        }];
        app.selected = 0;
        app.current_session = Some(selected_name.to_string());
        app.preview_follow_tail = false;
        app.preview_history_for = Some("session-a".to_string());
        let stale_history: Vec<Vec<PreviewSpan>> =
            omega_core::session::styled_rows_from_ansi("\x1b[35mstale session-a task card\x1b[0m")
                .0;
        app.preview_history_styled = Some(stale_history);

        app.refresh_preview().await.expect("static preview refresh");

        assert_eq!(
            app.preview_content,
            "(this is the session running OmegaOS — preview disabled to prevent recursion)"
        );
        assert!(
            app.preview_history_for.is_none(),
            "the static session must not retain another session's history identity"
        );
        assert!(
            app.preview_history_styled.is_none(),
            "the static message must win over another session's styled history"
        );

        let mut terminal = Terminal::new(TestBackend::new(110, 8)).expect("test terminal");
        terminal
            .draw(|frame| {
                let area = frame.area();
                draw_sessions_right(frame, &mut app, area, false);
            })
            .expect("render static preview");
        let rendered = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(
            rendered.contains(
                "(this is the session running OmegaOS — preview disabled to prevent recursion)"
            ),
            "the rendered panel must contain the static self-preview message"
        );
        assert!(
            !rendered.contains("stale session-a task card"),
            "the rendered panel must not contain the prior session's styled task card"
        );
        assert!(
            !rendered.contains("PAUSED"),
            "a static current-session frame cannot be paused at zero of zero: {rendered:?}"
        );
    }

    #[tokio::test]
    async fn no_session_refresh_drops_copyable_rows_and_drag() {
        let mut app = App::new(OmegaConfig::default());
        app.preview_content = "stale session-a body".to_string();
        app.preview_session = Some("session-a".to_string());
        app.preview_follow_tail = false;
        app.sessions_preview_area = Some(ratatui::layout::Rect::new(0, 0, 40, 8));
        app.preview_screen_rows = vec!["stale session-a copy marker".to_string()];
        app.preview_select_anchor = Some((1, 1));
        app.preview_select_head = Some((8, 1));
        app.preview_select_dragging = true;

        app.refresh_preview().await.expect("empty preview refresh");

        assert!(app.preview_screen_rows.is_empty());
        assert!(app.preview_select_anchor.is_none());
        assert!(app.preview_select_head.is_none());
        assert!(!app.preview_select_dragging);
        assert_eq!(
            app.take_preview_selection_text(),
            None,
            "an empty reset must make the prior frame impossible to copy"
        );
    }

    #[test]
    fn empty_preview_never_renders_paused_zero_of_zero() {
        let mut app = App::new(OmegaConfig::default());
        app.preview_follow_tail = false;
        let rendered = render_sessions_preview(&mut app);
        assert!(rendered.contains("(select a session to preview)"));
        assert!(
            !rendered.contains("PAUSED"),
            "an empty preview cannot be paused at zero of zero: {rendered:?}"
        );
    }

    #[test]
    fn same_index_session_identity_render_transition_replaces_a_with_b() {
        const SESSION_A: &str = "same-index-session-a";
        const SESSION_B: &str = "same-index-session-b";
        const MARKER_A: &str = "ONLY_SESSION_A_MARKER";
        const MARKER_B: &str = "ONLY_SESSION_B_MARKER";

        let entry = |name| SessionEntry {
            session: OmegaSession::classify(name),
            progress: None,
            is_current: false,
            is_protected: false,
            tree_prefix: String::new(),
        };
        let mut app = App::new(OmegaConfig::default());
        app.sessions = vec![entry(SESSION_A)];
        app.selected = 0;
        app.preview_session = Some(SESSION_A.to_string());
        app.preview_content = MARKER_A.to_string();
        app.preview_follow_tail = false;
        app.preview_scroll = 2;

        let mut terminal = Terminal::new(TestBackend::new(140, 10)).expect("test terminal");
        let frame_a = render_sessions_preview_on(&mut terminal, &mut app);
        assert!(frame_a.contains(SESSION_A));
        assert!(frame_a.contains(MARKER_A));

        app.sessions[0] = entry(SESSION_B);
        app.prepare_preview_session_switch(SESSION_B);

        let loading_b = render_sessions_preview_on(&mut terminal, &mut app);
        assert!(loading_b.contains(SESSION_B));
        assert!(
            loading_b.contains(&format!("(loading preview for {SESSION_B}...)")),
            "a selected target with no body needs target-aware loading copy: {loading_b:?}"
        );
        assert!(!loading_b.contains("(select a session to preview)"));
        assert!(!loading_b.contains(MARKER_A));
        assert!(
            !loading_b.contains("PAUSED"),
            "a transient empty target cannot show PAUSED zero-of-zero: {loading_b:?}"
        );

        app.preview_content = (0..12)
            .map(|row| format!("{MARKER_B} row {row}"))
            .collect::<Vec<_>>()
            .join("\n");
        let frame_b = render_sessions_preview_on(&mut terminal, &mut app);

        assert!(frame_b.contains(SESSION_B));
        assert!(frame_b.contains(MARKER_B));
        assert!(!frame_b.contains(MARKER_A));
        assert!(
            frame_b.contains("PAUSED"),
            "an ordinary capturable A-to-B switch must preserve paused history mode: {frame_b:?}"
        );
    }

    #[tokio::test]
    async fn paused_history_switch_to_missing_session_clears_stale_frame_and_reaches_placeholder() {
        const SESSION_A: &str = "preview-history-session-a";
        const MISSING_SESSION_B: &str = "preview-history-missing-session-b-0ff3aa1";
        const STALE_PLAIN_A: &str = "stale plain frame owned by session a";
        const STALE_STYLED_A: &str = "stale styled frame owned by session a";

        let mut app = App::new(OmegaConfig::default());
        app.sessions = [SESSION_A, MISSING_SESSION_B]
            .into_iter()
            .map(|name| SessionEntry {
                session: OmegaSession::classify(name),
                progress: None,
                is_current: false,
                is_protected: false,
                tree_prefix: String::new(),
            })
            .collect();
        app.selected = 1;
        app.current_session = None;
        app.preview_follow_tail = false;
        app.preview_content = STALE_PLAIN_A.to_string();
        app.preview_session = Some(SESSION_A.to_string());
        app.preview_history_for = Some(SESSION_A.to_string());
        app.preview_history_styled = Some(
            omega_core::session::styled_rows_from_ansi(&format!("\x1b[35m{STALE_STYLED_A}\x1b[0m"))
                .0,
        );

        app.refresh_preview()
            .await
            .expect("first failed history refresh");
        let tick_one = render_sessions_preview(&mut app);

        app.refresh_preview()
            .await
            .expect("second failed history refresh");
        app.refresh_preview()
            .await
            .expect("third failed history refresh");
        let sustained_failure = render_sessions_preview(&mut app);

        assert!(
            tick_one.contains(MISSING_SESSION_B),
            "the first failed tick must be rendered under session B's title: {tick_one:?}"
        );
        assert!(
            tick_one.contains(&format!("(retrying preview for {MISSING_SESSION_B}...)")),
            "the transiently missing target needs target-aware retry copy: {tick_one:?}"
        );
        assert!(!tick_one.contains("(select a session to preview)"));
        assert_eq!(
            (
                tick_one.contains(STALE_PLAIN_A),
                tick_one.contains(STALE_STYLED_A),
                app.preview_fail_streak,
                sustained_failure.contains("(session has no pane content)"),
            ),
            (false, false, 3, true),
            "a missing history target must clear session A on tick 1 and reach the existing placeholder threshold; tick_one={tick_one:?}; sustained={sustained_failure:?}"
        );
    }

    #[tokio::test]
    async fn paused_history_switch_to_missing_session_clears_prior_static_frame() {
        const STATIC_SESSION: &str = "preview-static-current-session";
        const MISSING_HISTORY_SESSION: &str =
            "preview-history-missing-after-static-session-0ff3aa1";
        const RECURSION_MESSAGE: &str =
            "(this is the session running OmegaOS — preview disabled to prevent recursion)";

        let mut app = App::new(OmegaConfig::default());
        app.sessions = [STATIC_SESSION, MISSING_HISTORY_SESSION]
            .into_iter()
            .map(|name| SessionEntry {
                session: OmegaSession::classify(name),
                progress: None,
                is_current: false,
                is_protected: false,
                tree_prefix: String::new(),
            })
            .collect();
        app.selected = 0;
        app.current_session = Some(STATIC_SESSION.to_string());
        app.preview_follow_tail = false;
        app.preview_session = Some(MISSING_HISTORY_SESSION.to_string());

        app.refresh_preview().await.expect("static preview refresh");
        let static_frame = render_sessions_preview(&mut app);
        assert_eq!(app.preview_session.as_deref(), Some(STATIC_SESSION));
        assert!(
            static_frame.contains(RECURSION_MESSAGE),
            "the setup must render the static recursion frame: {static_frame:?}"
        );

        app.selected = 1;
        app.refresh_preview()
            .await
            .expect("failed history refresh after static frame");
        let switched_frame = render_sessions_preview(&mut app);

        assert!(
            switched_frame.contains(MISSING_HISTORY_SESSION),
            "the failed tick must be rendered under the new history target: {switched_frame:?}"
        );
        assert!(
            !switched_frame.contains(RECURSION_MESSAGE),
            "the prior static recursion frame must not render under the new history target: {switched_frame:?}"
        );
    }

    #[tokio::test]
    async fn paused_history_same_session_reuses_cached_frame() {
        const SESSION_A: &str = "preview-history-cached-session-a";
        const CACHED_PLAIN: &str = "cached plain history for session a";
        const CACHED_STYLED: &str = "cached styled history for session a";

        let mut app = App::new(OmegaConfig::default());
        app.sessions = vec![SessionEntry {
            session: OmegaSession::classify(SESSION_A),
            progress: None,
            is_current: false,
            is_protected: false,
            tree_prefix: String::new(),
        }];
        app.selected = 0;
        app.current_session = None;
        app.preview_follow_tail = false;
        app.preview_content = CACHED_PLAIN.to_string();
        app.preview_session = Some(SESSION_A.to_string());
        app.preview_history_for = Some(SESSION_A.to_string());
        app.preview_history_styled = Some(
            omega_core::session::styled_rows_from_ansi(&format!("\x1b[36m{CACHED_STYLED}\x1b[0m"))
                .0,
        );

        app.refresh_preview().await.expect("cached history refresh");
        let rendered = render_sessions_preview(&mut app);

        assert_eq!(app.preview_content, CACHED_PLAIN);
        assert_eq!(app.preview_history_for.as_deref(), Some(SESSION_A));
        assert!(
            rendered.contains(CACHED_STYLED),
            "same-session paused history must reuse its styled cache: {rendered:?}"
        );
    }

    #[tokio::test]
    async fn paused_history_empty_gap_invalidates_owner_before_same_session_reappears() {
        const MISSING_SESSION_A: &str = "preview-history-missing-empty-gap-a-0ff3aa1";
        const STALE_PLAIN_A: &str = "stale plain history across empty gap";
        const STALE_LIVE_STYLED_A: &str = "stale live styled preview across empty gap";
        const STALE_STYLED_A: &str = "stale styled history across empty gap";

        let entry = SessionEntry {
            session: OmegaSession::classify(MISSING_SESSION_A),
            progress: None,
            is_current: false,
            is_protected: false,
            tree_prefix: String::new(),
        };
        let mut app = App::new(OmegaConfig::default());
        app.sessions = vec![entry.clone()];
        app.selected = 0;
        app.current_session = None;
        app.preview_follow_tail = false;
        app.preview_content = STALE_PLAIN_A.to_string();
        app.preview_styled = Some(
            omega_core::session::styled_rows_from_ansi(&format!(
                "\x1b[31m{STALE_LIVE_STYLED_A}\x1b[0m"
            ))
            .0,
        );
        app.preview_cursor = Some((4, 7, true));
        app.preview_revision = 42;
        app.preview_session = Some(MISSING_SESSION_A.to_string());
        app.preview_history_for = Some(MISSING_SESSION_A.to_string());
        app.preview_history_styled = Some(
            omega_core::session::styled_rows_from_ansi(&format!("\x1b[35m{STALE_STYLED_A}\x1b[0m"))
                .0,
        );

        app.sessions.clear();
        app.refresh_preview().await.expect("empty-list refresh");
        let empty_frame = render_sessions_preview(&mut app);

        assert_eq!(
            (
                app.preview_content.is_empty(),
                app.preview_styled.is_none(),
                app.preview_cursor.is_none(),
                app.preview_revision,
                app.preview_session.is_none(),
                app.preview_history_for.is_none(),
                app.preview_history_styled.is_none(),
                app.preview_fail_streak,
                empty_frame.contains("(select a session to preview)"),
            ),
            (true, true, true, 0, true, true, true, 0, true),
            "an empty list must invalidate every preview cache owner: {empty_frame:?}"
        );

        app.sessions.push(entry);
        app.refresh_preview()
            .await
            .expect("same missing session refresh after empty gap");
        let reappeared_frame = render_sessions_preview(&mut app);

        assert_eq!(app.preview_session.as_deref(), Some(MISSING_SESSION_A));
        assert_eq!(
            app.preview_fail_streak, 1,
            "the reappearing target must attempt one fresh capture"
        );
        assert!(
            !reappeared_frame.contains(STALE_PLAIN_A),
            "stale plain history must not render after the empty gap: {reappeared_frame:?}"
        );
        assert!(
            !reappeared_frame.contains(STALE_STYLED_A),
            "stale styled history must not render after the empty gap: {reappeared_frame:?}"
        );
    }

    #[test]
    fn switching_preview_target_clears_previous_session_frame_once() {
        let mut app = App::new(OmegaConfig::default());
        app.preview_content = "Prompt: not sent".to_string();
        app.preview_styled = Some(
            omega_core::session::styled_rows_from_ansi(
                "\x1b[35mPrompt: not sent\x1b[0m\n\x1b[36mold styled row\x1b[0m",
            )
            .0,
        );
        app.preview_cursor = Some((4, 7, true));
        app.preview_revision = 42;
        app.preview_session = Some("session-a".to_string());
        app.preview_fail_streak = 3;
        app.sessions_preview_area = Some(ratatui::layout::Rect::new(0, 0, 40, 8));
        app.preview_screen_rows = vec!["stale copy row".to_string()];
        app.preview_select_anchor = Some((1, 1));
        app.preview_select_head = Some((6, 1));
        app.preview_select_dragging = true;

        let paused_history =
            omega_core::session::styled_rows_from_ansi("\x1b[33mpaused history row\x1b[0m").0;
        app.preview_history_for = Some("session-a".to_string());
        app.preview_history_styled = Some(paused_history.clone());

        app.prepare_preview_session_switch("session-b");

        assert_eq!(app.preview_content, "");
        assert!(
            app.preview_styled.is_none(),
            "the new session must not inherit old styled rows"
        );
        assert!(
            app.preview_cursor.is_none(),
            "the new session must not inherit the old cursor"
        );
        assert_eq!(app.preview_revision, 0);
        assert_eq!(app.preview_session.as_deref(), Some("session-b"));
        assert_eq!(app.preview_fail_streak, 0);
        assert!(app.preview_screen_rows.is_empty());
        assert!(app.preview_select_anchor.is_none());
        assert!(app.preview_select_head.is_none());
        assert!(!app.preview_select_dragging);
        assert_eq!(
            app.take_preview_selection_text(),
            None,
            "the prior target's rendered rows must not remain copyable"
        );
        assert_eq!(
            app.preview_history_for.as_deref(),
            Some("session-a"),
            "target switch preparation must not change paused-history ownership"
        );
        let paused_history_after = app
            .preview_history_styled
            .as_ref()
            .expect("paused-history rows must remain cached")
            .iter()
            .map(|line| {
                line.iter()
                    .map(|span| span.text.as_str())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(paused_history_after, "paused history row");

        app.preview_fail_streak = 1;
        app.prepare_preview_session_switch("session-b");
        assert_eq!(
            app.preview_fail_streak, 1,
            "a retry for the same target must retain its failure streak"
        );
    }
}

#[cfg(test)]
mod settings_update_tests {
    use super::*;
    use omega_core::config::{AutoUpdatePolicy, OmegaConfig};

    fn general_fields(config: &OmegaConfig) -> Vec<SettingsField> {
        let providers = omega_core::providers::ProvidersConfig::default();
        fields_for_section(SettingsSection::General, &providers, config)
    }

    fn action(fields: &[SettingsField], needle: &str) -> (String, bool) {
        fields
            .iter()
            .find_map(|f| match f {
                SettingsField::Action {
                    label,
                    command,
                    confirm_first,
                } if command == needle => Some((label.clone(), *confirm_first)),
                _ => None,
            })
            .unwrap_or_else(|| panic!("no action running `{}` in General: {:?}", needle, fields))
    }

    #[test]
    fn general_offers_check_and_update_with_confirm_on_the_destructive_half() {
        let config = OmegaConfig::default();
        let fields = general_fields(&config);

        // `--check` changes nothing, so it fires on the first Enter.
        let (_, check_confirms) = action(&fields, "omega update --check");
        assert!(
            !check_confirms,
            "a read-only check must not cost two presses"
        );

        // The real update rebuilds and replaces the running binary — two-press.
        let (label, update_confirms) = action(&fields, "omega update");
        assert!(
            update_confirms,
            "the real update must be armed before it fires"
        );
        assert!(
            label.contains("Update"),
            "the row must read as an update button: {}",
            label
        );
    }

    #[test]
    fn auto_update_select_points_at_the_saved_policy() {
        for (policy, expected) in [
            (AutoUpdatePolicy::Apply, "apply"),
            (AutoUpdatePolicy::Check, "check"),
            (AutoUpdatePolicy::Off, "off"),
        ] {
            let config = OmegaConfig {
                auto_update: policy,
                ..OmegaConfig::default()
            };
            let fields = general_fields(&config);
            let (options, idx) = fields
                .iter()
                .find_map(|f| match f {
                    SettingsField::Select {
                        config_key,
                        options,
                        current_index,
                        ..
                    } if config_key == "general.auto_update" => {
                        Some((options.clone(), *current_index))
                    }
                    _ => None,
                })
                .expect("General must expose the auto-update policy");
            assert_eq!(
                options[idx], expected,
                "the picker must open on the SAVED policy, not on the first option"
            );
        }
    }

    #[test]
    fn status_line_reports_never_checked_without_inventing_a_date() {
        // A path that cannot exist → the same state as a box whose update cron
        // has never run. It must say so, not render a blank or a fake time.
        let config = OmegaConfig {
            state_dir: std::path::PathBuf::from("/nonexistent/omega-settings-test"),
            ..OmegaConfig::default()
        };
        let line = update_status_line(&config);
        assert!(
            line.contains("OmegaOS v"),
            "must name the running version: {}",
            line
        );
        assert!(
            line.contains("last check never"),
            "must admit it never checked: {}",
            line
        );
    }

    #[test]
    fn status_line_reads_the_cron_state_when_there_is_one() {
        let dir = std::env::temp_dir().join(format!("omega-settings-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let state = omega_core::auto_update::AutoUpdateState {
            last_check: Some(chrono::Utc::now()),
            last_outcome: Some("already up to date".to_string()),
            ..Default::default()
        };
        state.save(&dir).unwrap();

        let config = OmegaConfig {
            state_dir: dir.clone(),
            ..OmegaConfig::default()
        };
        let line = update_status_line(&config);
        assert!(
            line.contains("already up to date"),
            "the last outcome is the whole point of the line: {}",
            line
        );
        assert!(
            line.contains("0m ago"),
            "a check from this second must read as fresh: {}",
            line
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn aisb_settings_describe_a_read_only_viewer_without_a_provider_selector() {
        let config = OmegaConfig::default();
        let providers = omega_core::providers::ProvidersConfig::default();
        let general = fields_for_section(SettingsSection::General, &providers, &config);
        let viewer = fields_for_section(SettingsSection::Aisb, &providers, &config);
        let rendered = general
            .iter()
            .chain(viewer.iter())
            .map(|field| field.label())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(!rendered.contains("Default AISB agent"));
        assert!(!rendered.contains("Current AISB agent"));
        assert!(rendered.contains("read-only") || rendered.contains("Read-only"));
        assert!(viewer.iter().any(|field| matches!(
            field,
            SettingsField::Action { command, .. } if command == "omega aisb-view"
        )));
    }
}
