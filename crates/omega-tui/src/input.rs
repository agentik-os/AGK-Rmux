use crate::app::{App, InputMode, MenuAction, MonitorAction, ProjectLane, SessionFocus, Tab};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

pub enum Action {
    None,
    Quit,
    AttachSession(String),
    KillSession(String),
    /// Kill all sessions except current + protected + infrastructure.
    KillAllSessions,
    /// Nuclear cleanup: kill all + prune stale state + clear scratch + drop cache.
    NuclearCleanup,
    Refresh,
    /// Toggle terminal mouse capture. OFF → the terminal does native
    /// click-drag text selection + copy/paste; ON → clickable menus + scroll.
    ToggleMouseCapture,
    CreateSession(String),
    CreateSessionWithAgent {
        name: String,
        agent: omega_core::agents::Agent,
        prompt: Option<String>,
    },
    CreateSessionAutoName {
        agent: omega_core::agents::Agent,
        prompt: Option<String>,
    },
    DispatchOracle(String, String),
    /// New-project wizard result: spawn a Claude session that runs
    /// `/omega-new-project <stack> <category> <name>` (provision + scaffold +
    /// vision/PRD/planner). `category` and `stack` come from the typed wizard registries.
    CreateProject {
        name: String,
        category: String,
        stack: String,
        launch_prompt: Option<String>,
        launch_docs: Option<String>,
    },
    /// Start the provisioning-keys wizard (Monitor tab, Telegram-style).
    ProvisioningSetup,
    /// Commit the provisioning-keys wizard — (env_key, value) pairs; blanks are
    /// ignored by the writer (existing values preserved).
    ProvisioningCommit {
        values: Vec<(String, String)>,
    },
    /// Start the Claude OAuth re-login engine (request_reauth) — captures the
    /// authorize URL and surfaces it in the Monitor → Account view.
    LoginClaude,
    /// Submit the OAuth authorize code (Monitor → Account) — runs handle_code
    /// against the waiting reauth session.
    SubmitReauthCode {
        code: String,
    },
    /// Refresh the AISB usage cache (runs the usage-monitor.sh script).
    RefreshBilling,
    /// Open the Telegram bot setup flow.
    TelegramSetup,
    /// Disconnect the currently active Omega Telegram bot.
    TelegramDisconnect,
    /// Rename a session (old, new).
    RenameSession {
        old: String,
        new: String,
    },
    /// Commit a freshly-completed Telegram setup wizard.
    TelegramSetupCommit {
        bot_token: String,
        chat_id: i64,
        user_ids: Vec<i64>,
    },
    /// Run a shell command (Settings Install/Uninstall actions).
    RunShellCommand {
        label: String,
        command: String,
    },
    /// Begin editing a settings text field (opens input modal pre-filled).
    EditSettingsField {
        config_key: String,
        current: String,
        masked: bool,
    },
    /// Toggle a boolean settings field.
    ToggleSettingsBool {
        config_key: String,
    },
    /// Commit an edited settings text field (saves to providers.toml).
    CommitSettingsEdit {
        config_key: String,
        value: String,
    },
    /// Ctrl+L — force a full terminal clear + redraw.
    ForceRedraw,
    /// Menu → Restart OmegaOS: tear down the terminal and re-exec the
    /// `omega menu` binary in place (picks up a freshly-built binary).
    Restart,
    /// Projects tab: open the selected project in a terminal as a NEW blank
    /// session running `agent`, in the project's dir. Never re-attaches to an
    /// existing session — the Sessions tab already does that, and silently
    /// re-attaching is what made "open" feel broken.
    OpenProject {
        name: String,
        path: String,
        agent: omega_core::agents::Agent,
    },
    /// Projects tab → Marketing lane: open the project's DEDICATED marketing
    /// session under the picked LLM agent. `cwd` is `<project>/marketing/`
    /// (created if missing); `prompt` is the scoped marketing-agent brief
    /// (marketing machine + R-MARKETING skills, product code off-limits).
    OpenMarketingSession {
        name: String,
        cwd: String,
        prompt: String,
        agent: omega_core::agents::Agent,
    },
    /// OS tab: open a Claude session scoped to the selected operative system's
    /// directory (`OS/<slug>/`) running its MASTER.md master agent — the same
    /// brain its Telegram bot gets.
    OpenOsSession {
        name: String,
        cwd: String,
        prompt: String,
    },
    /// OS tab: link a Telegram bot to the selected OS (T). Runs the
    /// interactive `omega-os-bot.sh <slug>` in a terminal session — the
    /// operator pastes the @BotFather token there; the bot's brain is the
    /// OS's master agent.
    LinkOsBot {
        slug: String,
    },
    /// Projects tab: open the canonical `/omg-planner` skill for the project.
    RunPlannerForProject {
        name: String,
        path: String,
    },
    /// Projects tab: register an existing folder into the project registry
    /// (reuses `project_manager::add_existing_project`).
    RegisterProject {
        path: String,
    },
    /// Projects tab: remove a project from the registry (two-press confirmed).
    /// Delete a project at one of the three Telegram-parity tiers:
    /// mode = "omega" (unmanage), "local" (+ folder), "all" (+ GitHub repo).
    /// Runs the bot's one-shot CLI — the ONE canonical deletion impl.
    DeleteProjectTier {
        name: String,
        mode: &'static str,
    },
    /// Projects tab: flip the selected project's Telegram toggle (topic sync +
    /// Atlas bot visibility). Writes `ManagedProject.telegram`; the next `/sync`
    /// reconciles the forum topic (creates when ON, removes when OFF).
    ToggleProjectTelegram {
        name: String,
    },
    /// Projects tab: "Delete forever" (two-press confirmed) — runs the canonical
    /// deletion (Telegram topic + dashboard agent + agent-bot + registry + the
    /// local folder) via the bot's one-shot CLI so there is ONE implementation.
    /// Monitor → Project group: persist the Telegram supergroup id
    /// (`TelegramGroupConfig`). The manual fallback to the bot's auto-detect.
    GroupSetupCommit {
        group_id: i64,
    },
    /// Real-time keystroke forwarding to a rmux session (preview interactive
    /// mode). One key per Action — printable chars, special keys, Ctrl-combos
    /// all route through this so plan-mode / OAuth / choice menus work.
    ForwardCharToSession {
        session: String,
        ch: char,
    },
    ForwardKeyToSession {
        session: String,
        key: &'static str,
    },
    /// Multi-char paste forwarded as a single literal write to the session
    /// (used by handle_paste so the entire bracketed-paste block lands as
    /// one PTY write rather than N individual keystrokes).
    SendTextRawToSession {
        session: String,
        text: String,
    },
    /// Insert a literal newline into the agent's input box WITHOUT submitting
    /// (Shift+Enter / Alt+Enter). Empirically, Claude Code treats a trailing
    /// backslash followed by Enter as a newline-insert, so the main loop emits
    /// a `\` text write then an Enter key.
    InsertNewlineToSession {
        session: String,
    },
}

/// Enter on the Monitor → Account section. Context-aware re-login driver: when
/// the authorize URL is already captured (`ShowUrl`), Enter opens the code-input
/// modal; otherwise it (re)starts the OAuth engine.
fn account_enter_action(app: &mut App) -> Action {
    use crate::app::{InputMode, ReauthStatus};
    match &app.reauth_status {
        ReauthStatus::ShowUrl(_) => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::ReauthCode;
            app.status_message = Some(
                "Paste the authorize code from your browser — Enter to submit, Esc to cancel"
                    .to_string(),
            );
            Action::None
        }
        // Generating / Validating — a step is in flight; ignore re-entry.
        ReauthStatus::Generating | ReauthStatus::Validating => {
            app.status_message = Some("Re-login already in progress…".to_string());
            Action::None
        }
        // Idle / Done / Error — (re)start the flow.
        _ => Action::LoginClaude,
    }
}

pub fn handle_event(app: &mut App, event: Event) -> Action {
    match event {
        Event::Key(key) => handle_key(app, key),
        Event::Paste(text) => handle_paste(app, text),
        Event::Mouse(mouse) => handle_mouse(app, mouse),
        Event::Resize(_, _) => Action::ForceRedraw,
        _ => Action::None,
    }
}

fn handle_mouse(app: &mut App, mouse: MouseEvent) -> Action {
    // Every non-Normal input mode paints a modal overlay. Until a modal grows
    // an explicit mouse contract, swallow the event: clicks and wheel motion
    // must never mutate the hidden screen underneath it.
    if !matches!(app.input_mode, InputMode::Normal) {
        return Action::None;
    }
    match mouse.kind {
        MouseEventKind::ScrollDown => {
            scroll_active_panel_at(app, 3, true, mouse.column, mouse.row);
            Action::None
        }
        MouseEventKind::ScrollUp => {
            scroll_active_panel_at(app, 3, false, mouse.column, mouse.row);
            Action::None
        }
        // Drag with the left button held = tmux-style text selection over the
        // preview mirror (mouse capture stays ON, so wheel-scroll keeps
        // working). The anchor is armed on Down below; releasing copies.
        MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
            if app.preview_select_anchor.is_some()
                && app.tab == Tab::Sessions
                && app.sessions_preview_area.is_some()
            {
                app.preview_select_head = Some((mouse.column, mouse.row));
                app.preview_select_dragging = true;
            }
            Action::None
        }
        MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
            if app.preview_select_dragging {
                if let Some(text) = app.take_preview_selection_text() {
                    let chars = text.chars().count();
                    app.pending_clipboard = Some(text);
                    app.set_status_sticky(format!(
                        "✓ {} char{} copied to clipboard",
                        chars,
                        if chars == 1 { "" } else { "s" }
                    ));
                } else {
                    app.clear_preview_selection();
                }
            } else {
                app.clear_preview_selection();
            }
            Action::None
        }
        // Click in a panel = focus it (left = list, right = preview)
        MouseEventKind::Down(_) => {
            // Left press inside the preview arms a possible drag-selection;
            // anywhere else cancels a stale one. A plain click (no drag)
            // keeps its focus meaning via the logic below.
            if matches!(
                mouse.kind,
                MouseEventKind::Down(crossterm::event::MouseButton::Left)
            ) && app.tab == Tab::Sessions
                && rect_hit(app.sessions_preview_area, mouse.column, mouse.row)
            {
                app.preview_select_anchor = Some((mouse.column, mouse.row));
                app.preview_select_head = Some((mouse.column, mouse.row));
                app.preview_select_dragging = false;
            } else {
                app.clear_preview_selection();
            }
            // Menu tab: click an action row → select + run it (like Enter).
            // Guarded to when the list fits on screen (offset 0) so a click never
            // maps to the wrong row while scrolled — keyboard always works too.
            if app.tab == Tab::Menu {
                let a = app.menu_area;
                if app.menu_fits
                    && mouse.column > a.x
                    && mouse.column < a.x + a.width.saturating_sub(1)
                    && mouse.row > a.y
                    && mouse.row < a.y + a.height.saturating_sub(1)
                {
                    let ridx = (mouse.row - a.y - 1) as usize; // rendered row (border-relative)
                    if let Some(Some(act_idx)) = app.menu_rendered_actions.get(ridx).copied() {
                        app.menu_selected = act_idx;
                        return execute_menu_action(app, app.selected_menu_action());
                    }
                }
                return Action::None;
            }
            if app.tab == Tab::Sessions {
                // Hit-test against the rects the renderer actually used
                // (draw_sessions records them each frame) — the old hardcoded
                // `column >= 30` heuristic misrouted clicks on wide terminals
                // (the 25% list extends past col 30) AND narrow ones (the
                // single-column list owns the full width).
                if rect_hit(app.sessions_preview_area, mouse.column, mouse.row) {
                    if matches!(app.session_focus, SessionFocus::List) {
                        // fix6-T5: the click is the mouse twin of the keyboard
                        // Enter — apply the same post-drop-grace rule: no
                        // re-entry while the grace is open, swallow with a
                        // notice instead.
                        if app.in_post_drop_grace() {
                            grace_swallow_notice(app, "click");
                            return Action::None;
                        }
                        // Use the canonical focus path (follow_tail = true) so a
                        // mouse click behaves like the keyboard Enter — entering
                        // chat shows the latest output instead of freezing the view.
                        app.enter_chat_focus();
                    }
                } else if rect_hit(app.sessions_list_area, mouse.column, mouse.row) {
                    // Map the clicked row to its session (headers excluded).
                    // Only trustworthy while the whole list fits on screen
                    // (ListState offset 0) — the same guard as the Menu tab.
                    let clicked = if app.sessions_list_fits {
                        app.sessions_list_area.and_then(|a| {
                            let ridx = (mouse.row - a.y - 1) as usize;
                            app.sessions_rendered_rows.get(ridx).copied().flatten()
                        })
                    } else {
                        None
                    };
                    match clicked {
                        // Second click on the already-selected row = the mouse
                        // twin of Enter: enter chat through the same
                        // grace-guarded path as the preview click above.
                        Some(idx)
                            if idx == app.selected
                                && matches!(app.session_focus, SessionFocus::List) =>
                        {
                            if app.in_post_drop_grace() {
                                grace_swallow_notice(app, "click");
                                return Action::None;
                            }
                            app.enter_chat_focus();
                        }
                        Some(idx) => {
                            // Selecting via click moves the cursor exactly like
                            // keyboard ↑/↓, so it ends the grace the same way
                            // (fix6-T6).
                            app.end_post_drop_grace();
                            app.selected = idx;
                            // Tab-less focus change — set_list_focus keeps the
                            // chord contract (FIX-4): a click must not complete
                            // a Tab-Tab.
                            app.set_list_focus();
                        }
                        None => app.set_list_focus(),
                    }
                } else {
                    // Outside both panels (tab strip, status bar): just make
                    // sure the list has focus, as before.
                    app.set_list_focus();
                }
            }
            Action::None
        }
        _ => Action::None,
    }
}

/// True when (column, row) falls INSIDE the rect's borders — the same
/// border-exclusive test the Menu tab click handler uses.
fn rect_hit(rect: Option<ratatui::layout::Rect>, column: u16, row: u16) -> bool {
    rect.is_some_and(|a| {
        column > a.x
            && column < a.x + a.width.saturating_sub(1)
            && row > a.y
            && row < a.y + a.height.saturating_sub(1)
    })
}

/// Position-aware scroll: in Sessions tab, scrolling over the preview panel
/// (hit-tested against the rect the renderer recorded — not the old
/// hardcoded `column >= 30`) scrolls the preview regardless of focus state.
fn scroll_active_panel_at(app: &mut App, lines: u16, down: bool, column: u16, row: u16) {
    if app.tab == Tab::Sessions && rect_hit(app.sessions_preview_area, column, row) {
        if down {
            app.scroll_preview_down(lines);
        } else {
            app.scroll_preview_up(lines);
        }
        return;
    }
    scroll_active_panel(app, lines, down);
}

fn scroll_active_panel(app: &mut App, lines: u16, down: bool) {
    match app.tab {
        Tab::Sessions => {
            if matches!(
                app.session_focus,
                SessionFocus::Chat | SessionFocus::ChatFullscreen
            ) {
                if down {
                    app.scroll_preview_down(lines);
                } else {
                    app.scroll_preview_up(lines);
                }
            } else {
                // fix6-T6: scrolling the list moves the selection exactly like
                // keyboard ↑/↓, so it must end the grace + Esc-Esc chord the
                // same way — otherwise Esc→scroll→Esc hits the FIX-H mismatch
                // arm claiming the (alive) armed session is gone.
                app.end_post_drop_grace();
                for _ in 0..lines {
                    if down {
                        app.select_next();
                    } else {
                        app.select_prev();
                    }
                }
            }
        }
        Tab::Menu => {
            let max = MenuAction::all().len().saturating_sub(1);
            for _ in 0..lines {
                if down {
                    app.menu_selected = (app.menu_selected + 1).min(max);
                } else {
                    app.menu_selected = app.menu_selected.saturating_sub(1);
                }
            }
        }
        Tab::Settings => {
            if app.detail_focused {
                if down {
                    app.scroll_detail_down(lines);
                } else {
                    app.scroll_detail_up(lines);
                }
            } else {
                for _ in 0..lines {
                    if down {
                        app.settings_tab_next();
                    } else {
                        app.settings_tab_prev();
                    }
                }
            }
        }
        Tab::Projects => {
            if app.detail_focused {
                if down {
                    app.scroll_detail_down(lines);
                } else {
                    app.scroll_detail_up(lines);
                }
            } else {
                for _ in 0..lines {
                    if down {
                        app.projects_tab_next();
                    } else {
                        app.projects_tab_prev();
                    }
                }
            }
        }
        Tab::Os => {
            if app.detail_focused {
                if down {
                    app.scroll_detail_down(lines);
                } else {
                    app.scroll_detail_up(lines);
                }
            } else {
                for _ in 0..lines {
                    if down {
                        app.os_tab_next();
                    } else {
                        app.os_tab_prev();
                    }
                }
            }
        }
        Tab::System => {
            if app.detail_focused {
                if down {
                    app.scroll_detail_down(lines);
                } else {
                    app.scroll_detail_up(lines);
                }
            } else {
                for _ in 0..lines {
                    if down {
                        app.select_info_next();
                    } else {
                        app.select_info_prev();
                    }
                }
            }
        }
        Tab::Help => {
            if down {
                app.scroll_detail_down(lines);
            } else {
                app.scroll_detail_up(lines);
            }
        }
    }
}

/// Tabs that share the list/detail focus contract. Keeping this predicate in
/// one place prevents a new two-column screen from gaining a visible `Tab ->
/// detail` hint without receiving the keyboard handlers that make it true.
fn is_two_column_tab(tab: Tab) -> bool {
    matches!(tab, Tab::Settings | Tab::Projects | Tab::System | Tab::Os)
}

fn uses_detail_scroll(tab: Tab) -> bool {
    is_two_column_tab(tab) || tab == Tab::Help
}

fn handle_paste(app: &mut App, text: String) -> Action {
    // Sessions tab + chat-focused → forward the paste DIRECTLY to the rmux
    // session (no chat_input buffer). Dispatched as ForwardMsg::Paste →
    // send_paste_raw, which wraps the block in bracketed-paste markers and
    // sends NO trailing Enter, so embedded newlines don't submit each line
    // as a separate command and multi-line / special chars survive intact.
    if app.input_mode == InputMode::Normal
        && app.tab == Tab::Sessions
        && matches!(
            app.session_focus,
            SessionFocus::Chat | SessionFocus::ChatFullscreen
        )
    {
        if let Some(entry) = app.selected_session() {
            let session = entry.session.name.clone();
            app.status_message = Some(format!("Pasted {} chars → {}", text.len(), session));
            return Action::SendTextRawToSession { session, text };
        }
    }

    // Active input modal: keep buffer semantics
    app.input_buffer.push_str(&text);
    app.status_message = Some(format!("Pasted {} chars", text.len()));
    Action::None
}

/// Open the dispatch step-1 project picker over the shared `ProjectRegistry`
/// (the SAME source the Telegram dispatch picker uses, so the added-projects list
/// stays in sync). No project added yet → a status hint instead of an empty picker.
/// LLM agents actually installed on this machine, in roster order — what the
/// open-project step-2 picker offers (claude / codex / gemini / kimi / …).
/// Shell is not an LLM and never belongs in this picker.
fn installed_agents() -> Vec<omega_core::agents::Agent> {
    omega_core::agents::Agent::all()
        .iter()
        .copied()
        .filter(|a| *a != omega_core::agents::Agent::Shell && a.is_available())
        .collect()
}

/// The Marketing-lane open action: a session in `<project>/marketing/` running
/// the project's DEDICATED marketing agent — marketing machine structure,
/// R-MARKETING skill chain, zernio publishing, product code off-limits.
fn marketing_open_action(name: &str, path: &str, agent: omega_core::agents::Agent) -> Action {
    let slug = std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_else(|| name.to_lowercase());
    let cwd = format!("{}/marketing", path);
    let prompt = format!(
        "Tu es l'agent MARKETING dédié du projet {name}. Tu travailles \
UNIQUEMENT sur son marketing, dans {path}/marketing/ (structure marketing \
machine 00-context…06-branding). Commence par `omega marketing status {slug}` \
pour l'état + la next best action. Si la structure marketing/ manque, \
scaffolde-la avec ~/.omega/marketing-machine/scaffold.sh (lis son usage). \
Chaîne de skills (R-MARKETING, dans l'ordre): /omg-product-marketing-context \
d'abord, puis /omg-content-strategy, /omg-social-content, /omg-ad-creative, \
/omg-brand-identity. Publication UNIQUEMENT via `omega-zernio` (toujours \
--dry-run d'abord, puis vérifier LIVE sur le profil), visuels via `higgsfield \
generate create`. Jamais de tiret cadratin dans la copy (R-NODASH). Ne touche \
pas au code produit.",
        name = name,
        path = path,
        slug = slug,
    );
    Action::OpenMarketingSession {
        name: format!("mkt-{}", slug),
        cwd,
        prompt,
        agent,
    }
}

fn open_dispatch_picker(app: &mut App) {
    if app.project_registry.poisoned {
        app.input_mode = InputMode::Normal;
        app.status_message = Some(
            "Dispatch refused: project registry is invalid; repair projects.json first."
                .to_string(),
        );
        return;
    }
    let names = app
        .project_registry
        .projects
        .iter()
        .map(|project| project.name.clone())
        .collect::<Vec<_>>();
    if names.is_empty() {
        app.input_mode = InputMode::Normal;
        app.status_message = Some(
            "No projects yet — add one in the Projects tab ([n] new / register), then dispatch."
                .to_string(),
        );
    } else {
        app.input_buffer = String::new();
        app.input_mode = InputMode::DispatchProject(names, 0);
        app.status_message = Some("Pick a project — ↑/↓, Enter, Esc".to_string());
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    // Global: Ctrl-T toggles mouse capture. OFF lets you drag-select + copy/paste
    // with the terminal's native selection; ON gives clickable menus + scroll.
    if key.code == KeyCode::Char('t') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::ToggleMouseCapture;
    }
    let mode = app.input_mode.clone();
    match mode {
        InputMode::Normal => handle_key_normal(app, key),

        // Direct-agent flow: type name → create + chat focus (NO prompt step)
        InputMode::NewNamedSession(agent_name) => handle_key_input(app, key, move |app, name| {
            let agent = omega_core::agents::Agent::from_name(&agent_name)
                .unwrap_or(omega_core::agents::Agent::Shell);
            app.input_mode = InputMode::Normal;
            Action::CreateSessionWithAgent {
                name,
                agent,
                prompt: None,
            }
        }),

        // Direct-agent flow: step 2 — optional prompt (name may be empty = auto-generate)
        InputMode::NewSessionPromptDirect(name, agent_name) => {
            let agent = omega_core::agents::Agent::from_name(&agent_name)
                .unwrap_or(omega_core::agents::Agent::Shell);
            match key.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                    if name.is_empty() {
                        Action::CreateSessionAutoName {
                            agent,
                            prompt: None,
                        }
                    } else {
                        Action::CreateSessionWithAgent {
                            name,
                            agent,
                            prompt: None,
                        }
                    }
                }
                KeyCode::Enter => {
                    let value = std::mem::take(&mut app.input_buffer);
                    let prompt = if value.trim().is_empty() {
                        None
                    } else {
                        Some(value)
                    };
                    app.input_mode = InputMode::Normal;
                    if name.is_empty() {
                        Action::CreateSessionAutoName { agent, prompt }
                    } else {
                        Action::CreateSessionWithAgent {
                            name,
                            agent,
                            prompt,
                        }
                    }
                }
                KeyCode::Backspace => {
                    app.input_buffer.pop();
                    Action::None
                }
                KeyCode::Char(c) => {
                    app.input_buffer.push(c);
                    Action::None
                }
                _ => Action::None,
            }
        }

        InputMode::DispatchProject(projects, sel) => {
            let count = projects.len().max(1);
            match key.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                    app.status_message = Some("Cancelled".to_string());
                    Action::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.input_mode = InputMode::DispatchProject(projects, (sel + 1) % count);
                    Action::None
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let next = if sel == 0 { count - 1 } else { sel - 1 };
                    app.input_mode = InputMode::DispatchProject(projects, next);
                    Action::None
                }
                KeyCode::Enter => {
                    let project = projects.get(sel).cloned().unwrap_or_default();
                    app.input_buffer = String::new();
                    app.status_message = Some(format!(
                        "Dispatch to {} — type the mission (Enter to send)",
                        project
                    ));
                    app.input_mode = InputMode::DispatchMission(project);
                    Action::None
                }
                _ => Action::None,
            }
        }
        InputMode::DispatchMission(_) => {
            let project = match &app.input_mode {
                InputMode::DispatchMission(p) => p.clone(),
                _ => return Action::None,
            };
            handle_key_input(app, key, move |app, mission| {
                app.input_mode = InputMode::Normal;
                Action::DispatchOracle(project.clone(), mission)
            })
        }

        // New-project wizard — step 1: name → category picker
        InputMode::NewProjectName => handle_key_input(app, key, |app, value| {
            let name = value.trim().to_lowercase().replace(' ', "-");
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewProjectCategory(name, 0);
            app.status_message =
                Some("↑/↓ category — Enter to continue, Esc to cancel".to_string());
            Action::None
        }),

        // New-project wizard — step 2: category picker (customer/side-business/tools)
        InputMode::NewProjectCategory(name, sel) => {
            let count = crate::app::NEW_PROJECT_CATEGORIES.len();
            match key.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                    app.status_message = Some("Cancelled".to_string());
                    Action::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.input_mode = InputMode::NewProjectCategory(name, (sel + 1) % count);
                    Action::None
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let next = if sel == 0 { count - 1 } else { sel - 1 };
                    app.input_mode = InputMode::NewProjectCategory(name, next);
                    Action::None
                }
                KeyCode::Enter => {
                    let category = crate::app::NEW_PROJECT_CATEGORIES[sel].0.to_string();
                    if category == "customer" {
                        // Customer → choose/create a credential group (separate accounts).
                        app.new_project_cred_group = None;
                        app.input_buffer = String::new();
                        let groups = omega_core::provisioning::list_groups().join(", ");
                        app.input_mode = InputMode::NewProjectCredGroup(name, category);
                        app.status_message = Some(format!(
                            "Credential group — existing: {}  —  type one to reuse OR a new customer name (Enter, Esc=default)",
                            groups
                        ));
                    } else {
                        app.new_project_cred_group = None; // non-customer → shared/default creds
                        app.input_mode = InputMode::NewProjectStack(name, category, 0);
                        app.status_message =
                            Some("↑/↓ stack — Enter to continue, Esc to cancel".to_string());
                    }
                    Action::None
                }
                _ => Action::None,
            }
        }

        // New-project wizard — step 2b (customer projects only): pick/create a credential group.
        InputMode::NewProjectCredGroup(name, category) => match key.code {
            KeyCode::Esc => {
                app.new_project_cred_group = Some("default".to_string());
                app.input_buffer = String::new();
                app.input_mode = InputMode::NewProjectStack(name, category, 0);
                app.status_message = Some("Credential group: default (shared)".to_string());
                Action::None
            }
            KeyCode::Enter => {
                let typed = app.input_buffer.trim().to_string();
                let group = if typed.is_empty() {
                    "default".to_string()
                } else {
                    typed
                };
                let existing = group == "default"
                    || omega_core::provisioning::list_groups()
                        .iter()
                        .any(|g| g == &omega_core::provisioning::sanitize_group(&group));
                app.new_project_cred_group = Some(group.clone());
                app.input_buffer = String::new();
                app.input_mode = InputMode::NewProjectStack(name, category, 0);
                app.status_message = Some(format!(
                    "Credential group: {} ({})",
                    group,
                    if existing {
                        "reuse"
                    } else {
                        "NEW — fill it later with: omega provision set"
                    }
                ));
                Action::None
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
                Action::None
            }
            KeyCode::Char(c) => {
                app.input_buffer.push(c);
                Action::None
            }
            _ => Action::None,
        },

        // New-project wizard — step 3: stack picker → spawn the provisioning session
        InputMode::NewProjectStack(name, category, sel) => {
            let count = crate::app::NEW_PROJECT_STACKS.len();
            match key.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                    app.status_message = Some("Cancelled".to_string());
                    Action::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.input_mode = InputMode::NewProjectStack(name, category, (sel + 1) % count);
                    Action::None
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let next = if sel == 0 { count - 1 } else { sel - 1 };
                    app.input_mode = InputMode::NewProjectStack(name, category, next);
                    Action::None
                }
                KeyCode::Enter => {
                    let stack = crate::app::NEW_PROJECT_STACKS[sel].0.to_string();
                    app.input_buffer = String::new();
                    app.input_mode = InputMode::NewProjectLaunchPrompt(name, category, stack);
                    app.status_message = Some(
                        "Optional kickoff — describe the idea/requirements (Enter to continue, Esc to skip)".to_string(),
                    );
                    Action::None
                }
                _ => Action::None,
            }
        }

        // New-project wizard — step 4 (optional): kickoff prompt.
        InputMode::NewProjectLaunchPrompt(name, category, stack) => match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                Action::CreateProject {
                    name,
                    category,
                    stack,
                    launch_prompt: None,
                    launch_docs: None,
                }
            }
            KeyCode::Enter => {
                let kickoff = if app.input_buffer.trim().is_empty() {
                    None
                } else {
                    Some(std::mem::take(&mut app.input_buffer))
                };
                app.input_buffer = String::new();
                app.input_mode = InputMode::NewProjectLaunchDocs(name, category, stack, kickoff);
                app.status_message = Some(
                    "Optional docs — comma-separated paths to seed the project (Enter to launch, Esc to skip)".to_string(),
                );
                Action::None
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
                Action::None
            }
            KeyCode::Char(c) => {
                app.input_buffer.push(c);
                Action::None
            }
            _ => Action::None,
        },

        // New-project wizard — step 5 (optional): doc paths, then spawn.
        InputMode::NewProjectLaunchDocs(name, category, stack, kickoff) => match key.code {
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                Action::CreateProject {
                    name,
                    category,
                    stack,
                    launch_prompt: kickoff,
                    launch_docs: None,
                }
            }
            KeyCode::Enter => {
                let docs = if app.input_buffer.trim().is_empty() {
                    None
                } else {
                    Some(std::mem::take(&mut app.input_buffer))
                };
                app.input_mode = InputMode::Normal;
                Action::CreateProject {
                    name,
                    category,
                    stack,
                    launch_prompt: kickoff,
                    launch_docs: docs,
                }
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
                Action::None
            }
            KeyCode::Char(c) => {
                app.input_buffer.push(c);
                Action::None
            }
            _ => Action::None,
        },

        InputMode::RenameSession(old_name) => handle_key_input(app, key, move |app, new_name| {
            app.input_mode = InputMode::Normal;
            if new_name.trim() == old_name {
                app.status_message = Some("(no change)".to_string());
                return Action::None;
            }
            Action::RenameSession {
                old: old_name.clone(),
                new: new_name,
            }
        }),

        InputMode::SessionFilter => handle_key_input(app, key, |app, query| {
            app.input_mode = InputMode::Normal;
            let q = query.trim();
            app.session_filter = if q.is_empty() {
                None
            } else {
                Some(q.to_string())
            };
            // Anchor to the top row: the dispatched Refresh re-anchors by the
            // NAME at this index, and the old selection may not survive the
            // new filter at all — row 0 is the predictable landing spot.
            app.selected = 0;
            app.status_message = Some(match &app.session_filter {
                Some(f) => format!("Filter: '{}' (press / to change)", f),
                None => "Filter cleared".to_string(),
            });
            Action::Refresh
        }),

        // ── Telegram setup wizard (3 steps) ─────────────────────────────────
        InputMode::TelegramSetupToken => handle_key_input(app, key, |app, token| {
            app.input_buffer = String::new();
            app.input_mode = InputMode::TelegramSetupChatId(token);
            app.status_message = Some(
                "Step 2/3: Telegram CHAT_ID (numeric, get yours from @userinfobot)".to_string(),
            );
            Action::None
        }),
        InputMode::TelegramSetupChatId(token) => {
            handle_key_input(app, key, move |app, chat_id_str| {
                let chat_id: i64 = match chat_id_str.trim().parse() {
                    Ok(n) => n,
                    Err(_) => {
                        app.status_message = Some(format!(
                            "Invalid chat_id '{}' — must be numeric",
                            chat_id_str
                        ));
                        app.input_mode = InputMode::Normal;
                        return Action::None;
                    }
                };
                app.input_buffer = String::new();
                app.input_mode = InputMode::TelegramSetupUserId(token.clone(), chat_id.to_string());
                app.status_message = Some(
                    "Step 3/3: ALLOWED user_id (or Esc to skip — chat_id-only filter)".to_string(),
                );
                Action::None
            })
        }
        InputMode::EditSettingsField { config_key, .. } => {
            let cfg_key = config_key.clone();
            handle_key_input(app, key, move |app, value| {
                app.input_mode = InputMode::Normal;
                Action::CommitSettingsEdit {
                    config_key: cfg_key,
                    value,
                }
            })
        }
        InputMode::ProjectOpenLane(name, path, sel) => {
            // Step 1 — the LANE: Coding · Marketing (marketing machine + the
            // project's dedicated marketing agent) · Oracle · Cancel. Both
            // session lanes continue to step 2, the installed-agent picker.
            const COUNT: usize = 4;
            let to_agents = |app: &mut App, lane: ProjectLane, name: String, path: String| {
                let agents = installed_agents();
                if agents.is_empty() {
                    app.input_mode = InputMode::Normal;
                    app.status_message =
                        Some("No coding agent installed (Settings → install one)".to_string());
                    return;
                }
                app.input_mode = InputMode::ProjectOpenAgentPick {
                    lane,
                    name,
                    path,
                    agents,
                    sel: 0,
                };
            };
            let oracle = |app: &mut App, name: String| {
                app.status_message = Some(format!(
                    "Oracle {} — mission (Enter to dispatch, Esc)",
                    name
                ));
                app.input_buffer = String::new();
                app.input_mode = InputMode::DispatchMission(name);
            };
            match key.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                    app.status_message = Some("Cancelled".to_string());
                    Action::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.input_mode = InputMode::ProjectOpenLane(name, path, (sel + 1) % COUNT);
                    Action::None
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let next = if sel == 0 { COUNT - 1 } else { sel - 1 };
                    app.input_mode = InputMode::ProjectOpenLane(name, path, next);
                    Action::None
                }
                KeyCode::Char('1') => {
                    to_agents(app, ProjectLane::Coding, name, path);
                    Action::None
                }
                KeyCode::Char('2') => {
                    to_agents(app, ProjectLane::Marketing, name, path);
                    Action::None
                }
                KeyCode::Char('3') => {
                    oracle(app, name);
                    Action::None
                }
                KeyCode::Enter => {
                    match sel {
                        0 => to_agents(app, ProjectLane::Coding, name, path),
                        1 => to_agents(app, ProjectLane::Marketing, name, path),
                        2 => oracle(app, name),
                        _ => {
                            app.input_mode = InputMode::Normal;
                            app.status_message = Some("Cancelled".to_string());
                        }
                    }
                    Action::None
                }
                _ => Action::None,
            }
        }

        InputMode::ProjectOpenAgentPick {
            lane,
            name,
            path,
            agents,
            sel,
        } => {
            // Step 2 — the LLM: only agents actually installed on this machine
            // (claude / codex / gemini / kimi / …). Last row = Cancel.
            let count = agents.len() + 1;
            let open = |app: &mut App, idx: usize| -> Action {
                let Some(agent) = agents.get(idx).copied() else {
                    app.input_mode = InputMode::Normal;
                    app.status_message = Some("Cancelled".to_string());
                    return Action::None;
                };
                app.input_mode = InputMode::Normal;
                match lane {
                    ProjectLane::Coding => Action::OpenProject {
                        name: name.clone(),
                        path: path.clone(),
                        agent,
                    },
                    ProjectLane::Marketing => marketing_open_action(&name, &path, agent),
                }
            };
            match key.code {
                KeyCode::Esc => {
                    // Back to step 1, not a hard cancel.
                    app.input_mode = InputMode::ProjectOpenLane(name, path, 0);
                    Action::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.input_mode = InputMode::ProjectOpenAgentPick {
                        lane,
                        name,
                        path,
                        agents,
                        sel: (sel + 1) % count,
                    };
                    Action::None
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let next = if sel == 0 { count - 1 } else { sel - 1 };
                    app.input_mode = InputMode::ProjectOpenAgentPick {
                        lane,
                        name,
                        path,
                        agents,
                        sel: next,
                    };
                    Action::None
                }
                KeyCode::Char(c @ '1'..='9') => {
                    let idx = (c as usize) - ('1' as usize);
                    if idx < agents.len() {
                        open(app, idx)
                    } else {
                        Action::None
                    }
                }
                KeyCode::Enter => open(app, sel),
                _ => Action::None,
            }
        }

        InputMode::ProjectDelete(name, sel) => {
            const COUNT: usize = 4; // 3 tiers + cancel
            match key.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                    app.status_message = Some("Cancelled".to_string());
                    Action::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.input_mode = InputMode::ProjectDelete(name, (sel + 1) % COUNT);
                    Action::None
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let next = if sel == 0 { COUNT - 1 } else { sel - 1 };
                    app.input_mode = InputMode::ProjectDelete(name, next);
                    Action::None
                }
                // Digit shortcuts mirror the Telegram buttons (1️⃣/2️⃣/3️⃣).
                KeyCode::Char('1') => {
                    app.input_mode = InputMode::Normal;
                    Action::DeleteProjectTier {
                        name,
                        mode: "omega",
                    }
                }
                KeyCode::Char('2') => {
                    app.input_mode = InputMode::Normal;
                    Action::DeleteProjectTier {
                        name,
                        mode: "local",
                    }
                }
                KeyCode::Char('3') => {
                    app.input_mode = InputMode::Normal;
                    Action::DeleteProjectTier { name, mode: "all" }
                }
                KeyCode::Enter => {
                    app.input_mode = InputMode::Normal;
                    match sel {
                        0 => Action::DeleteProjectTier {
                            name,
                            mode: "omega",
                        },
                        1 => Action::DeleteProjectTier {
                            name,
                            mode: "local",
                        },
                        2 => Action::DeleteProjectTier { name, mode: "all" },
                        _ => {
                            app.status_message = Some("Cancelled".to_string());
                            Action::None
                        }
                    }
                }
                _ => Action::None,
            }
        }

        InputMode::SelectModel(config_key, options, sel) => {
            let count = options.len().max(1);
            // Theme selector: live-preview the highlighted theme on every
            // arrow move so the whole TUI re-skins under the overlay.
            let is_theme = config_key == "general.theme";
            match key.code {
                KeyCode::Esc => {
                    if is_theme {
                        // Revert the live preview to the saved theme.
                        crate::theme::set_active_slug(&app.config.theme);
                    }
                    app.input_mode = InputMode::Normal;
                    app.status_message = Some("Cancelled".to_string());
                    Action::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let next = (sel + 1) % count;
                    if is_theme {
                        if let Some(slug) = options.get(next) {
                            crate::theme::set_active_slug(slug);
                        }
                    }
                    app.input_mode = InputMode::SelectModel(config_key, options, next);
                    Action::None
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let next = if sel == 0 { count - 1 } else { sel - 1 };
                    if is_theme {
                        if let Some(slug) = options.get(next) {
                            crate::theme::set_active_slug(slug);
                        }
                    }
                    app.input_mode = InputMode::SelectModel(config_key, options, next);
                    Action::None
                }
                KeyCode::Enter => {
                    let value = options.get(sel).cloned().unwrap_or_default();
                    app.input_mode = InputMode::Normal;
                    Action::CommitSettingsEdit { config_key, value }
                }
                _ => Action::None,
            }
        }

        // Monitor → Project group: single-field group_id capture (manual
        // fallback to the bot's auto-detect). Numeric-validated like the
        // Telegram chat_id step.
        InputMode::GroupSetupId => handle_key_input(app, key, |app, value| {
            app.input_mode = InputMode::Normal;
            match value.trim().parse::<i64>() {
                Ok(group_id) => Action::GroupSetupCommit { group_id },
                Err(_) => {
                    app.status_message = Some(format!(
                        "Invalid group_id '{}' — must be a numeric id",
                        value.trim()
                    ));
                    Action::None
                }
            }
        }),

        // Register-existing-folder: single-field path capture → register it
        // (the folder name becomes the project name, per add_existing_project).
        InputMode::AddProjectPath => handle_key_input(app, key, |app, value| {
            app.input_mode = InputMode::Normal;
            let path = value.trim().to_string();
            if path.is_empty() {
                app.status_message = Some("No path entered".to_string());
                Action::None
            } else {
                Action::RegisterProject { path }
            }
        }),

        // Claude OAuth re-login: paste the authorize code → handle_code.
        InputMode::ReauthCode => handle_key_input(app, key, |app, value| {
            app.input_mode = InputMode::Normal;
            let code = value.trim().to_string();
            if code.is_empty() {
                app.status_message = Some("No code entered".to_string());
                Action::None
            } else {
                Action::SubmitReauthCode { code }
            }
        }),

        InputMode::TelegramSetupUserId(token, chat_id_str) => {
            let token = token.clone();
            let chat_id: i64 = chat_id_str.parse().unwrap_or(0);
            match key.code {
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                    Action::TelegramSetupCommit {
                        bot_token: token,
                        chat_id,
                        user_ids: Vec::new(),
                    }
                }
                KeyCode::Enter => {
                    let value = std::mem::take(&mut app.input_buffer);
                    let user_ids = value
                        .split(',')
                        .filter_map(|s| s.trim().parse::<i64>().ok())
                        .collect();
                    app.input_mode = InputMode::Normal;
                    Action::TelegramSetupCommit {
                        bot_token: token,
                        chat_id,
                        user_ids,
                    }
                }
                KeyCode::Backspace => {
                    app.input_buffer.pop();
                    Action::None
                }
                KeyCode::Char(c) => {
                    app.input_buffer.push(c);
                    Action::None
                }
                _ => Action::None,
            }
        }

        // ── Provisioning-keys wizard (N skippable steps) ────────────────────
        InputMode::ProvisioningSetup { step, collected } => match key.code {
            KeyCode::Esc => provisioning_advance(app, collected, String::new(), step),
            KeyCode::Enter => {
                // Provisioning keys are pasted secrets — trim like
                // handle_key_input does, or a trailing paste-newline is
                // persisted verbatim (invisible in the masked echo).
                let value = std::mem::take(&mut app.input_buffer).trim().to_string();
                provisioning_advance(app, collected, value, step)
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
                Action::None
            }
            KeyCode::Char(c) => {
                app.input_buffer.push(c);
                Action::None
            }
            _ => Action::None,
        },
    }
}

fn handle_key_normal(app: &mut App, key: KeyEvent) -> Action {
    // When in Sessions tab and chat-focused (split or fullscreen), route keys
    // to the chat input handler first.
    if app.tab == Tab::Sessions
        && matches!(
            app.session_focus,
            SessionFocus::Chat | SessionFocus::ChatFullscreen
        )
    {
        return handle_key_chat(app, key);
    }

    // fix6-T2: the DESIGN-015 post-drop grace, enforced at ONE altitude.
    // The old per-arm deny-list (q/x/Enter/Tab/./Esc) was opt-in: every
    // unguarded key — launchers c/C/g/p/G/t/h, dispatch 'd', rename 'r' —
    // opened a modal that bypassed the grace entirely (once input_mode !=
    // Normal, handle_key routes by mode before any guard). Swallow every
    // key up front, except:
    //   • navigation (↑/↓/j/k/PgUp/PgDn/Home/End/F5/F1/←/→/Tab-nav) — ↑/↓
    //     end the grace in their own arms (deliberate driving);
    //   • Esc — its arm owns the layered cancel/Esc-Esc-chord semantics and
    //     keeps its own grace swallow AFTER the chord check (FIX-D), so the
    //     chord still fires inside the grace;
    //   • Enter/Tab IFF the selection still equals the session pinned at
    //     Esc time (fix6-T4) — re-entering the SAME chat is retarget-safe,
    //     so the advertised "Enter = back to chat" hint isn't dead for the
    //     full 800ms the same keypress armed.
    // fix7-T1: Ctrl+L / Ctrl+R are deliberate two-key TUI COMMANDS with
    // CONTROL-guarded arms — hoisted ABOVE the grace intercept (same pattern
    // as the global Ctrl+T in handle_key) so the 800ms window can't eat a
    // redraw/restart. Only these two are exempt: in chat focus, Ctrl/Alt
    // combos forward to the agent PTY as readline input (Ctrl+C interrupt,
    // Ctrl+W / Alt+Backspace kill-word), so an in-flight combo after a drop
    // IS typed text — and the match arms below test key.code only, meaning a
    // blanket modifier exemption would let Ctrl+C open the 'c' launcher
    // modal and Ctrl+X hit the bare 'x' kill arm on a clamped-in neighbor —
    // exactly the fix6-T2 / FIX-F holes this intercept closes.
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        // Ctrl+L — force full terminal redraw (fixes corrupted view).
        if key.code == KeyCode::Char('l') {
            return Action::ForceRedraw;
        }
        // Ctrl+R — RELOAD the TUI: tear down + re-exec the freshly-built
        // binary (browser-style ^R = reload). Soft in-place refresh (no
        // teardown) is on F5 / the menu's Refresh item; menu "R" also reloads.
        if key.code == KeyCode::Char('r') {
            return Action::Restart;
        }
    }

    if app.in_post_drop_grace() {
        let shift_tab = key.code == KeyCode::Tab && key.modifiers.contains(KeyModifiers::SHIFT);
        let nav = shift_tab
            || matches!(
                key.code,
                KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::PageUp
                    | KeyCode::PageDown
                    | KeyCode::Home
                    | KeyCode::End
                    | KeyCode::Left
                    | KeyCode::Right
                    | KeyCode::BackTab
                    | KeyCode::F(1)
                    | KeyCode::F(5)
                    | KeyCode::Esc
                    | KeyCode::Char('j')
                    | KeyCode::Char('k')
            );
        if !nav {
            match key.code {
                KeyCode::Enter | KeyCode::Tab => {
                    // The only remaining focus drop is a session vanishing
                    // under the cursor — re-entry would aim the in-flight
                    // keystream at whatever slid into the selection.
                    return grace_swallow_notice(
                        app,
                        if key.code == KeyCode::Enter {
                            "Enter"
                        } else {
                            "Tab"
                        },
                    );
                }
                KeyCode::Char(c) => {
                    let mut buf = [0u8; 4];
                    let name = &*c.encode_utf8(&mut buf);
                    return grace_swallow_notice(app, name);
                }
                _ => return grace_swallow_notice(app, "key"),
            }
        }
    }

    match key.code {
        // (Ctrl+L redraw / Ctrl+R restart handled ABOVE the grace intercept —
        // fix7-T1.)

        // Quit (in-flight 'q' during the post-drop grace is swallowed by the
        // fix6-T2 intercept above).
        KeyCode::Char('q') => {
            app.should_quit = true;
            Action::Quit
        }

        // Tab switching: ←/→ for tabs, Tab inside Sessions toggles focus list↔chat
        KeyCode::Left => {
            app.prev_tab();
            app.reset_2col_focus();
            Action::None
        }
        KeyCode::Right => {
            app.next_tab();
            app.reset_2col_focus();
            Action::None
        }
        KeyCode::BackTab => {
            app.prev_tab();
            app.reset_2col_focus();
            Action::None
        }
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.prev_tab();
            } else if app.tab == Tab::Sessions {
                // FIX-F (R-3): an in-flight Tab right after a vanish/Esc drop
                // must not re-enter chat on the clamped-to NEIGHBOR — enforced
                // by the fix6-T2 intercept (with the T4 pinned-session
                // exemption) before this arm is reached.
                app.handle_tab_in_sessions();
                app.status_message = Some(match app.session_focus {
                    SessionFocus::List => "Session list — Tab: open · Tab-Tab: hide/show menu".to_string(),
                    SessionFocus::Chat => "In session — Shift+↑↓ read (pause) · End: back live · Tab: list · Ctrl+X: close".to_string(),
                    SessionFocus::ChatFullscreen => "Session FULLSCREEN — Ctrl+X: close · Tab-Tab: show menu".to_string(),
                });
            } else if is_two_column_tab(app.tab) {
                // 2-column tabs: Tab toggles list↔detail, Tab-Tab → fullscreen
                app.handle_tab_in_2col();
                // When entering detail on the Settings group, snap cursor to the
                // first actionable field (the Monitor group has no field list).
                if app.tab == Tab::Settings && app.detail_focused && !app.settings_on_monitor() {
                    let section = app.selected_settings_section();
                    let providers = app.providers();
                    let fields = crate::app::fields_for_section(section, &providers, &app.config);
                    if let Some(first) = fields.iter().position(|f| f.is_actionable()) {
                        app.settings_field_selected = first;
                    }
                }
                app.status_message = Some(if app.detail_fullscreen {
                    "Focus: detail FULLSCREEN (Tab → list, Tab-Tab → exit)".to_string()
                } else if app.detail_focused {
                    "Focus: detail (↑/↓ navigate, Enter activate, Tab → list)".to_string()
                } else {
                    "Focus: section list (Tab → detail, Tab-Tab → fullscreen)".to_string()
                });
            } else {
                app.next_tab();
            }
            Action::None
        }

        // Scroll: depends on the active tab + focus. PageUp/PageDown super-
        // scroll a FULL page of the preview (Termius swipe rips through fast).
        KeyCode::PageDown => {
            if uses_detail_scroll(app.tab) {
                app.scroll_detail_down(10);
            } else {
                app.scroll_preview_down(app.preview_inner_height.max(10));
            }
            Action::None
        }
        KeyCode::PageUp => {
            if uses_detail_scroll(app.tab) {
                app.scroll_detail_up(10);
            } else {
                app.scroll_preview_up(app.preview_inner_height.max(10));
            }
            Action::None
        }
        KeyCode::Home => {
            if uses_detail_scroll(app.tab) {
                app.detail_scroll = 0;
                // Explicit scroll wins over the sub-cursor snap-back.
                app.detail_follow_cursor = false;
            } else {
                app.scroll_preview_home();
            }
            Action::None
        }
        KeyCode::End => {
            if uses_detail_scroll(app.tab) {
                // Jump to the renderer-published bound, not a huge sentinel:
                // u16::MAX/2 scrolled the Paragraph ~32k lines past its
                // content — an empty panel only Home could recover.
                app.detail_scroll = app.detail_max_scroll;
                app.detail_follow_cursor = false;
            } else {
                app.scroll_preview_end();
            }
            Action::None
        }

        // Alt+↑ / Alt+↓ (Option on macOS) — scroll the active panel
        // (preview, detail, etc.) regardless of focus. Faster than the
        // selection-cycle that bare arrows do.
        KeyCode::Up if key.modifiers.contains(KeyModifiers::ALT) => {
            scroll_active_panel(app, 3, false);
            Action::None
        }
        KeyCode::Down if key.modifiers.contains(KeyModifiers::ALT) => {
            scroll_active_panel(app, 3, true);
            Action::None
        }

        // Sessions tab: navigate with ARROWS ONLY — j/k must NOT move the cursor
        // (operator preference). Catch bare j/k here first so they're ignored on
        // Sessions; on every other tab j/k still navigate via the arms below.
        KeyCode::Char('j') | KeyCode::Char('k') if app.tab == Tab::Sessions => Action::None,
        // Navigation: ↑/↓ AND j/k — context-aware (sessions vs menu)
        KeyCode::Down | KeyCode::Char('j') => {
            // A deliberate navigation key ends the post-drop grace early
            // (DESIGN-015) — the user is demonstrably driving the list.
            app.end_post_drop_grace();
            // Settings tab + detail focused: Monitor group → action cursor (on
            // the Actions section) or scroll; Settings group → navigate fields.
            if app.tab == Tab::Settings && app.detail_focused {
                if app.settings_on_monitor() {
                    if matches!(
                        app.selected_monitor_section(),
                        crate::app::MonitorSection::Actions
                    ) {
                        app.select_monitor_action_next();
                    } else {
                        app.scroll_detail_down(1);
                    }
                } else {
                    let section = app.selected_settings_section();
                    let providers = app.providers();
                    let fields = crate::app::fields_for_section(section, &providers, &app.config);
                    advance_to_next_actionable(app, &fields, true);
                }
                return Action::None;
            }
            // System tab + detail focused: sections with their own list (AI
            // Agents, Documentation) move that cursor; the rest scroll the text.
            if app.tab == Tab::System && app.detail_focused {
                match app.selected_info_section() {
                    crate::app::InfoSection::AisbAgents => app.select_info_agent_next(),
                    crate::app::InfoSection::Docs => app.select_info_doc_next(),
                    _ => app.scroll_detail_down(1),
                }
                return Action::None;
            }
            if app.tab == Tab::Projects && app.detail_focused {
                app.scroll_detail_down(1);
                return Action::None;
            }
            if app.tab == Tab::Os && app.detail_focused {
                app.scroll_detail_down(1);
                return Action::None;
            }
            match app.tab {
                Tab::Sessions => app.select_next(),
                Tab::Menu => app.select_menu_next(),
                Tab::Settings => app.settings_tab_next(),
                Tab::Projects => app.projects_tab_next(),
                Tab::Os => app.os_tab_next(),
                Tab::System => app.select_info_next(),
                Tab::Help => app.scroll_detail_down(1),
            }
            Action::None
        }

        KeyCode::Up | KeyCode::Char('k') => {
            // Same grace-ending semantics as Down above (DESIGN-015).
            app.end_post_drop_grace();
            if app.tab == Tab::Settings && app.detail_focused {
                if app.settings_on_monitor() {
                    if matches!(
                        app.selected_monitor_section(),
                        crate::app::MonitorSection::Actions
                    ) {
                        app.select_monitor_action_prev();
                    } else {
                        app.scroll_detail_up(1);
                    }
                } else {
                    let section = app.selected_settings_section();
                    let providers = app.providers();
                    let fields = crate::app::fields_for_section(section, &providers, &app.config);
                    advance_to_next_actionable(app, &fields, false);
                }
                return Action::None;
            }
            if app.tab == Tab::System && app.detail_focused {
                match app.selected_info_section() {
                    crate::app::InfoSection::AisbAgents => app.select_info_agent_prev(),
                    crate::app::InfoSection::Docs => app.select_info_doc_prev(),
                    _ => app.scroll_detail_up(1),
                }
                return Action::None;
            }
            if app.tab == Tab::Projects && app.detail_focused {
                app.scroll_detail_up(1);
                return Action::None;
            }
            if app.tab == Tab::Os && app.detail_focused {
                app.scroll_detail_up(1);
                return Action::None;
            }
            match app.tab {
                Tab::Sessions => app.select_prev(),
                Tab::Menu => app.select_menu_prev(),
                Tab::Settings => app.settings_tab_prev(),
                Tab::Projects => app.projects_tab_prev(),
                Tab::Os => app.os_tab_prev(),
                Tab::System => app.select_info_prev(),
                Tab::Help => app.scroll_detail_up(1),
            }
            Action::None
        }

        // [ and ] jump between System sections while the detail panel holds the
        // ↑/↓ keys (agent list, document list) — the arrows are taken by tabs.
        KeyCode::Char('[') if app.tab == Tab::System => {
            app.select_info_prev();
            Action::None
        }
        KeyCode::Char(']') if app.tab == Tab::System => {
            app.select_info_next();
            Action::None
        }

        // Enter: context-aware. Crucially: Enter in 2-col tabs behaves like
        // Tab — focuses the right panel — instead of taking the user away
        // from Omega.
        KeyCode::Enter => match app.tab {
            Tab::Sessions => {
                // DESIGN-015: an in-flight Enter during the post-drop grace
                // is handled by the fix6-T2 intercept (swallowed unless the
                // selection still equals the pinned chord session — T4).
                // Two-panel default: Enter focuses the preview (acts like Tab).
                // Once focused, Enter is forwarded to the rmux session below
                // (interactive passthrough — see the SessionFocus::Chat branch
                // earlier in this function).
                if app.selected_session().is_some() {
                    if app.session_focus == SessionFocus::List {
                        // Canonical focus path (chord reset + follow tail,
                        // FIX-4) — same as the mouse click.
                        app.enter_chat_focus();
                        // DESIGN-016: describe the REAL runtime bindings
                        // (single Tab → list, Tab-Tab → fullscreen; "/" is
                        // forwarded to the agent, not captured).
                        app.status_message = Some(
                            "Focus: chat — keys forward to agent (Tab → list · Tab-Tab → fullscreen)".to_string(),
                        );
                    }
                    Action::None
                } else {
                    Action::None
                }
            }
            Tab::Menu => execute_menu_action(app, app.selected_menu_action()),
            Tab::Settings => {
                // 2-column model. Enter on the section list focuses the detail
                // panel. Once focused, behaviour splits by group: the Monitor
                // group routes each section to its primary action; the Settings
                // group activates the selected provider field.
                if !app.detail_focused {
                    app.detail_focused = true;
                    app.detail_scroll = 0;
                    // Settings group → snap to the first actionable field.
                    if !app.settings_on_monitor() {
                        let section = app.selected_settings_section();
                        let providers = app.providers();
                        let fields =
                            crate::app::fields_for_section(section, &providers, &app.config);
                        if let Some(first) = fields.iter().position(|f| f.is_actionable()) {
                            app.settings_field_selected = first;
                        }
                    }
                    app.status_message = Some(
                        "Focus: detail (↑/↓ navigate, Enter activate, Tab → list, Tab-Tab → fullscreen)".to_string(),
                    );
                    Action::None
                } else if app.settings_on_monitor() {
                    // ── Monitor group: focused-Enter routes the section to its
                    // primary wizard/action (no command-line, no new wizard). ──
                    use crate::app::MonitorSection;
                    match app.selected_monitor_section() {
                        MonitorSection::Actions => {
                            let action = app.selected_monitor_action();
                            execute_monitor_action(action)
                        }
                        // Account & billing → the OAuth re-login engine. Context-
                        // aware: once the authorize URL is captured, Enter opens
                        // the code modal; otherwise it kicks off request_reauth.
                        MonitorSection::AccountBilling => account_enter_action(app),
                        // Telegram & projects → set up when absent, or a two-press-
                        // confirmed disconnect when a config exists.
                        MonitorSection::Telegram => {
                            if omega_core::monitor::OmegaTelegramConfig::exists() {
                                if app.monitor_disconnect_armed {
                                    app.monitor_disconnect_armed = false;
                                    Action::TelegramDisconnect
                                } else {
                                    // Arm — the warning renders state-driven
                                    // via armed_confirm_warning (FIX-A/T9b).
                                    app.monitor_disconnect_armed = true;
                                    Action::None
                                }
                            } else {
                                Action::TelegramSetup
                            }
                        }
                    }
                } else {
                    // ── Settings group: activate the selected provider field. ──
                    let section = app.selected_settings_section();
                    let providers = app.providers();
                    let fields = crate::app::fields_for_section(section, &providers, &app.config);
                    let idx = app
                        .settings_field_selected
                        .min(fields.len().saturating_sub(1));
                    match fields.into_iter().nth(idx) {
                        Some(crate::app::SettingsField::Action {
                            label,
                            command,
                            confirm_first,
                        }) => {
                            // Special: trigger Telegram wizard inline
                            if command == "__INTERNAL_TELEGRAM_SETUP__" {
                                app.settings_confirm_pending = None;
                                Action::TelegramSetup
                            } else if confirm_first {
                                // fix6-T1: two-press confirm keyed on the
                                // field's pinned IDENTITY, not the bare index
                                // — the list is re-derived live, so a
                                // background install finishing between arm
                                // and confirm can shift rows under the index.
                                match app.settings_confirm_pending.take() {
                                    Some((aidx, alabel)) if aidx == idx && alabel == label => {
                                        Action::RunShellCommand { label, command }
                                    }
                                    Some(_) => {
                                        // The armed field moved/vanished —
                                        // never fire whatever sits there now.
                                        app.status_message = Some(
                                            "Confirm cancelled — the settings list changed"
                                                .to_string(),
                                        );
                                        Action::None
                                    }
                                    None => {
                                        // First Enter → arm; the warning is
                                        // state-driven (armed_confirm_warning,
                                        // FIX-A) — no status duplicate (T9b).
                                        app.settings_confirm_pending = Some((idx, label.clone()));
                                        Action::None
                                    }
                                }
                            } else {
                                app.settings_confirm_pending = None;
                                Action::RunShellCommand { label, command }
                            }
                        }
                        Some(crate::app::SettingsField::EditText {
                            config_key,
                            current_value,
                            masked,
                            ..
                        }) => {
                            app.settings_confirm_pending = None;
                            Action::EditSettingsField {
                                config_key,
                                current: current_value,
                                masked,
                            }
                        }
                        Some(crate::app::SettingsField::Select {
                            config_key,
                            options,
                            current_index,
                            ..
                        }) => {
                            // Open the arrow-key overlay (no typing).
                            app.settings_confirm_pending = None;
                            app.input_mode = crate::app::InputMode::SelectModel(
                                config_key,
                                options,
                                current_index,
                            );
                            Action::None
                        }
                        Some(crate::app::SettingsField::Toggle { config_key, .. }) => {
                            app.settings_confirm_pending = None;
                            Action::ToggleSettingsBool { config_key }
                        }
                        _ => Action::None,
                    }
                }
            }
            Tab::Projects => {
                {
                    if app.projects_os_pinned() {
                        // The pinned "OS System" quick-access row: jump straight
                        // to the AgentikOS suite tab (the lazy loader in the main
                        // loop refreshes os_entries on tab change if empty).
                        app.leave_tab();
                        app.tab = Tab::Os;
                        app.detail_focused = false;
                        app.status_message = Some("AgentikOS suite — the OS tab".to_string());
                        Action::None
                    } else if app.project_registry.projects.is_empty() {
                        // Empty registry: Enter opens the same add-project modal
                        // as 'n' — the literal "Enter adds a project" affordance.
                        app.input_buffer = String::new();
                        app.input_mode = InputMode::AddProjectPath;
                        app.status_message = Some(
                            "Add project — path to an existing folder (Enter to register, Esc to cancel)".to_string(),
                        );
                        Action::None
                    } else if !app.detail_focused {
                        app.detail_focused = true;
                        app.detail_scroll = 0;
                        app.status_message = Some(
                            "Focus: project detail (↑/↓ scroll, Enter → open in terminal, Tab → list)".to_string(),
                        );
                        Action::None
                    } else {
                        // Detail focused → Enter asks WHICH agent to open the
                        // project with, then spawns a new blank session.
                        match app.selected_project() {
                            Some(p) => {
                                let name = p.name.clone();
                                let path = p.path.to_string_lossy().to_string();
                                app.input_mode = InputMode::ProjectOpenLane(name, path, 0);
                                app.status_message = Some(
                                    "Open project — Coding / Marketing / Oracle (↑/↓, Enter, Esc)"
                                        .to_string(),
                                );
                                Action::None
                            }
                            None => {
                                app.status_message = Some("No project selected".to_string());
                                Action::None
                            }
                        }
                    }
                }
            }
            Tab::System => {
                // Enter focuses the detail panel so the Laws, the agents, the
                // skills and the manual can be read and scrolled.
                if !app.detail_focused {
                    app.detail_focused = true;
                    app.detail_scroll = 0;
                    app.status_message = Some(
                        "Focus: detail (↑/↓ scroll or navigate, [/] section, Tab → list, Tab-Tab → fullscreen)"
                            .to_string(),
                    );
                }
                Action::None
            }
            Tab::Os => {
                // Match every other two-column tab: the first Enter moves focus
                // into the detail panel; only a deliberate second Enter opens
                // the selected OS master prompt.
                if !app.detail_focused {
                    app.detail_focused = true;
                    app.detail_scroll = 0;
                    app.status_message = Some(
                        "Focus: OS detail (up/down scroll, Enter opens master, Tab returns to list)"
                            .to_string(),
                    );
                    return Action::None;
                }

                // Detail-focused Enter opens a session in the selected OS's
                // directory with its MASTER.md prompt. Fallback: the generic
                // integrator prompt.
                match app.selected_os_entry() {
                    Some(e) => match e.path.clone() {
                        Some(path) if path.is_dir() => {
                            let prompt = std::fs::read_to_string(path.join("MASTER.md"))
                                .ok()
                                .filter(|s| !s.trim().is_empty())
                                .unwrap_or_else(|| {
                                    format!(
                                        "Tu es l'agent maître de {name} ({slug}), un operative \
system de la suite AgentikOS. Travaille UNIQUEMENT dans {path} : lis son README.md, \
intègre le payload (zip arrivé via la boîte Deposit) quand il est là, documente son \
fonctionnement (entrypoint, deps, config) et garde la parité install.sh (Law 0). \
Les secrets restent dans ~/.omega/secrets/, jamais dans le dossier. \
Statut actuel: {status}.",
                                        name = e.product.name,
                                        slug = e.product.slug,
                                        path = path.to_string_lossy(),
                                        status = e.status_label(),
                                    )
                                });
                            Action::OpenOsSession {
                                name: format!("os-{}", e.product.slug),
                                cwd: path.to_string_lossy().to_string(),
                                prompt,
                            }
                        }
                        _ => {
                            app.status_message = Some(format!(
                                "{} — OS/{} folder not found (F5 to rescan)",
                                e.product.name, e.product.slug
                            ));
                            Action::None
                        }
                    },
                    None => {
                        app.status_message = Some("No OS selected (F5 to load)".to_string());
                        Action::None
                    }
                }
            }
            Tab::Help => Action::None,
        },

        // OS tab: 'T' → link a Telegram bot to the selected operative system.
        KeyCode::Char('T') if app.tab == Tab::Os => match app.selected_os_entry() {
            Some(e) => Action::LinkOsBot {
                slug: e.product.slug.to_string(),
            },
            None => {
                app.status_message = Some("No OS selected (F5 to load)".to_string());
                Action::None
            }
        },

        // Settings tab → Monitor group: letter shortcuts (only when the cursor
        // sits in the Monitor group, so they don't fire while editing providers).
        KeyCode::Char('L') if app.tab == Tab::Settings && app.settings_on_monitor() => {
            Action::LoginClaude
        }
        KeyCode::Char('T') if app.tab == Tab::Settings && app.settings_on_monitor() => {
            Action::TelegramSetup
        }
        KeyCode::Char('P') if app.tab == Tab::Settings && app.settings_on_monitor() => {
            Action::ProvisioningSetup
        }
        KeyCode::Char('D') if app.tab == Tab::Settings && app.settings_on_monitor() => {
            Action::TelegramDisconnect
        }
        KeyCode::Char('B') if app.tab == Tab::Settings && app.settings_on_monitor() => {
            Action::RefreshBilling
        }
        KeyCode::Char('U') if app.tab == Tab::Settings && app.settings_on_monitor() => {
            execute_monitor_action(MonitorAction::UpdateOmega)
        }

        // Projects tab: 'n' opens a guided "register existing folder" modal
        // (the in-TUI replacement for the `omega project add` CLI hint). For a
        // green-field scaffold the Menu tab's New-project wizard still applies.
        KeyCode::Char('n') if app.tab == Tab::Projects => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::AddProjectPath;
            app.status_message = Some(
                "Add project — path to an existing folder (Enter to register, Esc to cancel)"
                    .to_string(),
            );
            Action::None
        }
        // Settings group: 'x' clears the selected text field (e.g. unlink a saved
        // API key) — two-press confirm, same affordance as destructive actions.
        // Backspace can't work here: it's only meaningful inside the edit modal.
        KeyCode::Char('x') | KeyCode::Char('X')
            if app.tab == Tab::Settings && app.detail_focused && !app.settings_on_monitor() =>
        {
            let section = app.selected_settings_section();
            let providers = app.providers();
            let fields = crate::app::fields_for_section(section, &providers, &app.config);
            let idx = app
                .settings_field_selected
                .min(fields.len().saturating_sub(1));
            match fields.into_iter().nth(idx) {
                Some(crate::app::SettingsField::EditText {
                    config_key,
                    current_value,
                    label,
                    ..
                }) if !current_value.is_empty() => {
                    // fix6-T1: same pinned-identity confirm as the Enter arm.
                    match app.settings_confirm_pending.take() {
                        Some((aidx, alabel)) if aidx == idx && alabel == label => {
                            app.status_message = Some(format!("Cleared: {}", label.trim()));
                            Action::CommitSettingsEdit {
                                config_key,
                                value: String::new(),
                            }
                        }
                        Some(_) => {
                            app.status_message =
                                Some("Confirm cancelled — the settings list changed".to_string());
                            Action::None
                        }
                        None => {
                            // Arm — warning is state-driven (FIX-A/T9b).
                            app.settings_confirm_pending = Some((idx, label.clone()));
                            Action::None
                        }
                    }
                }
                _ => Action::None,
            }
        }
        // Projects tab: 'x' opens the DELETE menu — the same three escalating
        // tiers as the Telegram bot (visible options, no hidden hotkey-guessing).
        KeyCode::Char('x') | KeyCode::Char('X') if app.tab == Tab::Projects => {
            match app.selected_project().map(|p| p.name.clone()) {
                Some(name) => {
                    app.input_mode = InputMode::ProjectDelete(name, 0);
                    Action::None
                }
                None => {
                    app.status_message = Some("No project selected".to_string());
                    Action::None
                }
            }
        }
        KeyCode::Char('T') if app.tab == Tab::Projects => {
            match app.selected_project().map(|p| p.name.clone()) {
                Some(name) => Action::ToggleProjectTelegram { name },
                None => {
                    app.status_message = Some("No project selected".to_string());
                    Action::None
                }
            }
        }
        // Projects tab: 'D' = Delete forever (two-press confirm) — removes the
        // project from OmegaOS AND deletes its local folder. Distinct from 'x'
        // (registry-only removal). First press arms it; second 'D' fires.
        KeyCode::Char('D') if app.tab == Tab::Projects => {
            match app.selected_project().map(|p| p.name.clone()) {
                Some(name) => {
                    if app.project_delete_pending.as_deref() == Some(name.as_str()) {
                        app.project_delete_pending = None;
                        Action::DeleteProjectTier {
                            name,
                            mode: "local",
                        }
                    } else {
                        // Arm — the warning renders state-driven via
                        // armed_confirm_warning (FIX-A/T9b).
                        app.project_delete_pending = Some(name.clone());
                        Action::None
                    }
                }
                None => {
                    app.status_message = Some("No project selected".to_string());
                    Action::None
                }
            }
        }

        // Shortcut keys (work in any tab) — direct agent launchers
        KeyCode::Char('c') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewNamedSession("claude".to_string());
            app.status_message =
                Some("Session name for new Claude (Enter, Esc to cancel)".to_string());
            Action::None
        }
        KeyCode::Char('o') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewNamedSession("codex".to_string());
            app.status_message =
                Some("Session name for new Codex (Enter, Esc to cancel)".to_string());
            Action::None
        }
        KeyCode::Char('g') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewNamedSession("gemini".to_string());
            app.status_message =
                Some("Session name for new Gemini (Enter, Esc to cancel)".to_string());
            Action::None
        }
        KeyCode::Char('a') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewNamedSession("antigravity".to_string());
            app.status_message =
                Some("Session name for new Antigravity (Enter, Esc to cancel)".to_string());
            Action::None
        }
        // Projects tab: 'p' runs the planner for the selected project
        // (the global 'p' = new Pi session applies on every other tab).
        KeyCode::Char('p') if app.tab == Tab::Projects => match app.selected_project() {
            Some(p) => Action::RunPlannerForProject {
                name: p.name.clone(),
                path: p.path.to_string_lossy().to_string(),
            },
            None => {
                app.status_message = Some("No project selected".to_string());
                Action::None
            }
        },
        KeyCode::Char('p') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewNamedSession("pi".to_string());
            app.status_message = Some("Session name for new Pi (Enter, Esc to cancel)".to_string());
            Action::None
        }
        KeyCode::Char('G') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewNamedSession("glm".to_string());
            app.status_message =
                Some("Session name for new GLM (Enter, Esc to cancel)".to_string());
            Action::None
        }
        KeyCode::Char('K') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewNamedSession("kimi".to_string());
            app.status_message =
                Some("Session name for new Kimi (Enter, Esc to cancel)".to_string());
            Action::None
        }
        KeyCode::Char('t') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewNamedSession("shell".to_string());
            app.status_message =
                Some("Session name for new Terminal (Enter, Esc to cancel)".to_string());
            Action::None
        }
        // 'h' is advertised in the Help/shortcut row as Hermes; wire it like
        // the other global launchers (p/G/t) so the advert isn't a dead key.
        KeyCode::Char('h') => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewNamedSession("hermes".to_string());
            app.status_message =
                Some("Session name for new Hermes (Enter, Esc to cancel)".to_string());
            Action::None
        }

        // Projects tab: 'd' pre-fills the dispatch with the selected project,
        // skipping the project-name step → straight to mission entry.
        KeyCode::Char('d') if app.tab == Tab::Projects => {
            match app.selected_project().map(|p| p.name.clone()) {
                Some(name) => {
                    app.input_buffer = String::new();
                    app.input_mode = InputMode::DispatchMission(name.clone());
                    app.status_message = Some(format!(
                        "Dispatch to {} — type the mission (Enter to send)",
                        name
                    ));
                }
                None => open_dispatch_picker(app),
            }
            Action::None
        }
        KeyCode::Char('d') => {
            open_dispatch_picker(app);
            Action::None
        }

        // Kill — both lowercase x and uppercase X work. Sessions tab only:
        // the list isn't visible elsewhere, so don't kill a hidden selection
        // from another tab.
        KeyCode::Char('x') | KeyCode::Char('X')
            if app.tab == Tab::Sessions && app.config.session_shortcuts =>
        {
            // DESIGN-015: an in-flight 'x' right after a focus drop is
            // swallowed by the fix6-T2 intercept before this arm.
            if let Some(entry) = app.selected_session() {
                if !entry.is_protected {
                    Action::KillSession(entry.session.name.clone())
                } else {
                    app.status_message =
                        Some("Session is protected (press . to unlock)".to_string());
                    Action::None
                }
            } else {
                Action::None
            }
        }

        // Rename selected session (Sessions tab only)
        KeyCode::Char('r') | KeyCode::Char('R')
            if app.tab == Tab::Sessions && app.config.session_shortcuts =>
        {
            if let Some(entry) = app.selected_session() {
                let old = entry.session.name.clone();
                app.input_buffer = old.clone();
                app.input_mode = InputMode::RenameSession(old.clone());
                app.status_message = Some(format!(
                    "Rename '{}' (Enter to confirm, Esc to cancel)",
                    old
                ));
            } else {
                app.status_message = Some("No session selected".to_string());
            }
            Action::None
        }

        // Refresh
        // Sessions tab: '/' filters the list; 'b' jumps to the next blocked/failed.
        KeyCode::Char('/') if app.tab == Tab::Sessions && app.config.session_shortcuts => {
            app.input_buffer = app.session_filter.clone().unwrap_or_default();
            app.input_mode = InputMode::SessionFilter;
            app.status_message =
                Some("Filter: type a substring, Enter to apply, empty+Enter clears".to_string());
            Action::None
        }
        KeyCode::Char('b') if app.tab == Tab::Sessions && app.config.session_shortcuts => {
            match app.jump_to_next_flagged() {
                Some(name) => app.status_message = Some(format!("→ {}", name)),
                None => app.status_message = Some("No blocked/failed sessions".to_string()),
            }
            Action::None
        }

        KeyCode::F(5) => Action::Refresh,

        KeyCode::Char('.') if app.tab == Tab::Sessions && app.config.session_shortcuts => {
            // FIX-F (R-4): a sentence-final '.' in an in-flight keystream is
            // swallowed by the fix6-T2 intercept before this arm.
            // Toggle on the source entry, then return owned (name, state) so the
            // &mut borrow ends before we touch app.rows / app.status_message.
            let toggled = app.sessions.get_mut(app.selected).map(|entry| {
                entry.is_protected = !entry.is_protected;
                (entry.session.name.clone(), entry.is_protected)
            });
            if let Some((nm, prot)) = toggled {
                // Reflect in the RENDERED rows immediately — they are clones made
                // at refresh time, so without this the § marker lags ~2s.
                for row in app.rows.iter_mut() {
                    if let crate::app::SessionRow::Entry(e) = row {
                        if e.session.name == nm {
                            e.is_protected = prot;
                        }
                    }
                }
                app.status_message = Some(format!(
                    "{} is now {}",
                    nm,
                    if prot { "protected" } else { "unprotected" }
                ));
            }
            Action::None
        }

        KeyCode::F(1) => {
            // fix6-T7: route through leave_tab() — a direct `app.tab =` write
            // bypassed the armed-confirm/chord hygiene every other switch has.
            app.leave_tab();
            app.tab = Tab::Help;
            app.detail_scroll = 0;
            Action::None
        }

        KeyCode::Esc => {
            // Cancel EVERY armed two-press confirm ATOMICALLY (fix6-T3): the
            // old `a.take() || b.take() || …` chain short-circuited after the
            // first armed state, so a dual-arm (reachable via direct tab
            // writers that skipped leave_tab) needed one Esc per state. All
            // take()s evaluate eagerly; one Esc clears them all (FIX-B/R-5 —
            // every warning advertises "Esc to cancel", so none may be dead).
            let menu = app.menu_confirm_pending.take().is_some();
            let proj_delete = app.project_delete_pending.take().is_some();
            let settings = app.settings_confirm_pending.take().is_some();
            let disconnect = app.monitor_disconnect_armed;
            app.monitor_disconnect_armed = false;
            if menu || proj_delete || settings || disconnect {
                app.status_message = Some("Cancelled".to_string());
                return Action::None;
            }
            // Esc is a layered "back" key:
            // 1. If detail/fullscreen focused → return to section list
            // 2. If on section list → go to Sessions tab
            // 3. If on Sessions tab → quit
            if app.detail_fullscreen {
                app.detail_fullscreen = false;
                app.status_message = Some("Exited fullscreen".to_string());
                Action::None
            } else if app.detail_focused {
                app.detail_focused = false;
                app.detail_scroll = 0;
                app.status_message = Some("Focus: section list".to_string());
                Action::None
            } else if app.tab == Tab::Sessions {
                // Chat/ChatFullscreen Esc never reaches here — the router
                // short-circuits into handle_key_chat, which forwards Esc to
                // the agent's PTY and swallows the dead-session Esc (AF-5)
                // instead of re-dispatching it into this quit arm. The old
                // Esc-Esc literal-ESC chord (DESIGN-014) is gone with it: a
                // chat Esc IS the literal ESC now, so there is nothing left
                // for a second Esc to rescue.
                //
                // DESIGN-015: inside the post-drop grace (a session vanished
                // under the cursor mid-keystream) a stray Esc must not quit
                // the TUI in one press — swallowed with a notice instead.
                if app.in_post_drop_grace() {
                    return grace_swallow_notice(app, "Esc");
                }
                app.should_quit = true;
                Action::Quit
            } else {
                // fix6-T7: leave_tab hygiene on the Esc→Sessions jump too.
                app.leave_tab();
                app.tab = Tab::Sessions;
                Action::None
            }
        }

        _ => Action::None,
    }
}

/// FIX-D (D-11): the post-drop grace swallows a destructive key — tell the
/// user instead of silently eating the press (a fast Esc→Enter or Esc→q used
/// to disappear with zero feedback).
fn grace_swallow_notice(app: &mut App, key: &str) -> Action {
    app.status_message = Some(format!(
        "'{}' ignored — just left a session (↑/↓ first, or wait a moment)",
        key
    ));
    Action::None
}

/// Move the settings field cursor to the next/previous actionable field,
/// skipping Info (non-interactable) entries.
fn advance_to_next_actionable(app: &mut App, fields: &[crate::app::SettingsField], forward: bool) {
    let n = fields.len();
    if n == 0 {
        return;
    }
    let actionable: Vec<usize> = fields
        .iter()
        .enumerate()
        .filter_map(|(i, f)| if f.is_actionable() { Some(i) } else { None })
        .collect();
    if actionable.is_empty() {
        return;
    }
    // Moving the field cursor cancels any pending destructive confirmation.
    app.settings_confirm_pending = None;
    let current = app.settings_field_selected;
    let target = if forward {
        actionable
            .iter()
            .copied()
            .find(|i| *i > current)
            .unwrap_or(actionable[0])
    } else {
        actionable
            .iter()
            .copied()
            .rev()
            .find(|i| *i < current)
            .unwrap_or(*actionable.last().unwrap())
    };
    app.settings_field_selected = target;
}

fn execute_monitor_action(action: MonitorAction) -> Action {
    match action {
        MonitorAction::Login => Action::LoginClaude,
        MonitorAction::TelegramSetup => Action::TelegramSetup,
        MonitorAction::TelegramDisconnect => Action::TelegramDisconnect,
        MonitorAction::ProvisioningSetup => Action::ProvisioningSetup,
        MonitorAction::RefreshBilling => Action::RefreshBilling,
        // Update runs as a DETACHED session (same reason the General-section
        // update does): `omega update` rebuilds the very binary this TUI runs
        // from and the build takes minutes, so spawning it keeps the UI alive
        // and makes the pull + build watchable. `omega update` is fast-forward
        // only, never over local changes, and preserves ~/.omega state.
        MonitorAction::UpdateOmega => Action::RunShellCommand {
            label: "OmegaOS update".to_string(),
            command: "omega update".to_string(),
        },
    }
}

/// Chat-input mode — REAL-TIME keystroke passthrough to the streamed rmux
/// session. Every key (printable, Enter, Backspace, arrows, Ctrl-combos)
/// is forwarded one-by-one so plan mode, OAuth code paste, and choice
/// menus work natively inside the agent.
///
/// Esc is NOT TUI-local: it forwards a real ESC to the agent so modal
/// programs (vim, less, fzf, Claude's prompts) can be escaped from inside
/// the session. Tab is the way back to the list.
///
/// TUI-local keys (never forwarded):
///   Tab           → cycle focus (List → Chat → Fullscreen → List)
///   Alt+Up/Down   → scroll preview
///   PageUp/Down   → scroll preview
///   Home/End      → scroll preview to top/bottom
///   Ctrl+L        → handled by the global redraw branch BEFORE this
fn handle_key_chat(app: &mut App, key: KeyEvent) -> Action {
    let session = match app.selected_session() {
        Some(entry) => entry.session.name.clone(),
        None => {
            // No session to chat with — e.g. the focused session died and the
            // list emptied. Returning None here swallowed EVERY key (even Tab
            // and q), soft-locking the whole UI. Instead drop back to list focus
            // and re-dispatch this keystroke in normal mode so it still acts
            // (Tab, q, arrows all work again). No recursion: focus is now List,
            // so handle_key_normal won't route back into the chat handler.
            app.set_list_focus();
            // EXCEPT Esc (AF-5): chat Esc forwards to the agent's PTY, and
            // there is no PTY left to forward to. Re-dispatching it would hit
            // the Sessions-tab quit arm and close the whole TUI in ONE press.
            if key.code == KeyCode::Esc {
                app.status_message = Some("Focus: session list".to_string());
                return Action::None;
            }
            return handle_key_normal(app, key);
        }
    };

    // --- TUI-local (never forwarded) ---

    // Ctrl+R — RELOAD the TUI even from chat focus (browser-style ^R = reload):
    // tear down + re-exec the freshly-built binary. Consistent with list focus.
    // Soft in-place refresh stays on F5 / the menu's Refresh item.
    if key.code == KeyCode::Char('r') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::Restart;
    }

    // Ctrl+X — CLOSE (kill) the session you're focused in, WITHOUT first Tabbing
    // back to the list. Plain 'x' must stay typable into the agent, so the close
    // action is the Ctrl+X chord (mnemonic: eXit; same deliberate override style
    // as Ctrl+R above). Drop to the list first so the user lands on a live
    // selection after the focused pane dies instead of a dead/empty chat.
    // Same guard as the list-focus 'x' (F-12): a protected session never dies.
    if key.code == KeyCode::Char('x') && key.modifiers.contains(KeyModifiers::CONTROL) {
        if app.selected_session().is_some_and(|e| e.is_protected) {
            // FIX-7/DESIGN-016: in chat focus '.' is forwarded to the PTY —
            // the unlock toggle only exists in list focus, so say so.
            app.status_message = Some("Session is protected (Tab, then . to unlock)".to_string());
            return Action::None;
        }
        app.set_list_focus();
        return Action::KillSession(session);
    }

    // Super-scroll: PageUp/PageDown jump a FULL page (viewport height) so a
    // Termius up/down swipe rips through history fast. Home/End below jump to
    // the absolute top/bottom of the full scrollback.
    let page = app.preview_inner_height.max(10);
    match key.code {
        KeyCode::PageUp => {
            app.scroll_preview_up(page);
            return Action::None;
        }
        KeyCode::PageDown => {
            app.scroll_preview_down(page);
            return Action::None;
        }
        _ => {}
    }

    // Tab behavior:
    //   Shift+Tab  → FORWARD to the focused agent session. Providers may use
    //                this terminal key for their own mode cycling.
    //   Tab        → return to the session list (back out of chat focus).
    // crossterm delivers Shift+Tab as KeyCode::BackTab (and on some
    // terminals as Tab+SHIFT) — handle both, forward the rmux "BTab".
    if key.code == KeyCode::BackTab
        || (key.code == KeyCode::Tab && key.modifiers.contains(KeyModifiers::SHIFT))
    {
        return Action::ForwardKeyToSession {
            session,
            key: "BTab",
        };
    }
    if key.code == KeyCode::Tab {
        // Unified Tab behavior (same as from the list): single Tab navigates
        // list↔session, Tab-Tab toggles the left session menu (hide↔show).
        // Shift+Tab (forwarded as BTab) is handled above.
        app.handle_tab_in_sessions();
        app.status_message = Some(match app.session_focus {
            SessionFocus::List => "Session list — Tab: open · Tab-Tab: hide/show menu".to_string(),
            SessionFocus::Chat => {
                "In session — Shift+↑↓ read (pause) · End: back live · Tab: list · Tab-Tab: menu"
                    .to_string()
            }
            SessionFocus::ChatFullscreen => "Session FULLSCREEN — Tab-Tab: show menu".to_string(),
        });
        return Action::None;
    }

    // Alt+arrows and Shift+arrows = TUI scroll preview.
    //
    // Shift is here for PHONES. Reading agent output from Termius, the plain
    // arrows are forwarded to the agent (correct — Claude needs them), so the
    // preview stays glued to the tail and the text keeps sliding away under the
    // reader. Alt+arrow already froze it, but Alt is not on a phone's key row
    // while Shift always is. PageUp/PageDown work too; this is the gesture that
    // survives a soft keyboard.
    if key.modifiers.contains(KeyModifiers::ALT) || key.modifiers.contains(KeyModifiers::SHIFT) {
        match key.code {
            KeyCode::Up => {
                app.scroll_preview_up(3);
                return Action::None;
            }
            KeyCode::Down => {
                app.scroll_preview_down(3);
                return Action::None;
            }
            _ => {}
        }
    }

    // Home / End = jump to the very top / bottom of the full scrollback.
    // (PageUp/PageDown super-scroll handled above.)
    match key.code {
        KeyCode::Home => {
            app.scroll_preview_home();
            return Action::None;
        }
        KeyCode::End => {
            app.scroll_preview_end();
            return Action::None;
        }
        _ => {}
    }

    // --- Forwarded to rmux session ---
    // Important: rmux key names come from rmux-core/src/keys/string_table.rs.
    // The backspace key is "BSpace" (NOT "BackSpace"); forward-delete is
    // "Delete" or "DC". Using the wrong name makes rmux send the literal text
    // instead of the key event — that was the original "BackSpace appears in
    // the agent's input" bug.

    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    match key.code {
        // (Tab + Shift+Tab handled earlier: Tab=back to list,
        //  Shift+Tab=forward BTab to the focused agent session.)

        // Word-delete (readline conventions):
        //   Ctrl+W            → kill word back   (universal)
        //   Shift+Backspace   → kill word back   (tmux convention, what the user wants)
        //   Alt+Backspace     → kill word back   (macOS / readline)
        //   Ctrl+Backspace    → kill word back   (Windows convention)
        KeyCode::Backspace if shift || alt || ctrl => Action::ForwardKeyToSession {
            session,
            key: "C-w",
        },
        KeyCode::Backspace => Action::ForwardKeyToSession {
            session,
            key: "BSpace",
        },
        // Forward-delete word:
        //   Shift+Delete or Alt+Delete → kill word forward (M-d in readline)
        KeyCode::Delete if shift || alt => Action::ForwardKeyToSession {
            session,
            key: "M-d",
        },
        KeyCode::Delete => Action::ForwardKeyToSession {
            session,
            key: "Delete",
        },
        // Shift+Enter / Alt+Enter → insert a newline in the input (don't
        // submit), matching real Claude Code multi-line input. Plain Enter
        // still submits.
        KeyCode::Enter if shift || alt => Action::InsertNewlineToSession { session },
        KeyCode::Enter => Action::ForwardKeyToSession {
            session,
            key: "Enter",
        },
        // Esc (plain or Alt) → literal ESC to the agent's PTY, ALWAYS.
        // Modal programs inside the session — vim, less, fzf, Claude's own
        // prompts and permission dialogs — need a real ESC, and swallowing it
        // TUI-side left the user stuck inside them. Leaving the session is
        // Tab (list) / Ctrl+X (close), so Esc never has to double as "back".
        KeyCode::Esc => Action::ForwardKeyToSession {
            session,
            key: "Escape",
        },
        KeyCode::Up => Action::ForwardKeyToSession { session, key: "Up" },
        KeyCode::Down => Action::ForwardKeyToSession {
            session,
            key: "Down",
        },
        KeyCode::Left if ctrl || alt => Action::ForwardKeyToSession {
            session,
            key: "M-b",
        },
        KeyCode::Right if ctrl || alt => Action::ForwardKeyToSession {
            session,
            key: "M-f",
        },
        KeyCode::Left => Action::ForwardKeyToSession {
            session,
            key: "Left",
        },
        KeyCode::Right => Action::ForwardKeyToSession {
            session,
            key: "Right",
        },
        KeyCode::Insert => Action::ForwardKeyToSession { session, key: "IC" },
        KeyCode::F(n) if (1..=12).contains(&n) => {
            let key_str: &'static str = match n {
                1 => "F1",
                2 => "F2",
                3 => "F3",
                4 => "F4",
                5 => "F5",
                6 => "F6",
                7 => "F7",
                8 => "F8",
                9 => "F9",
                10 => "F10",
                11 => "F11",
                12 => "F12",
                _ => return Action::None,
            };
            Action::ForwardKeyToSession {
                session,
                key: key_str,
            }
        }
        KeyCode::Char(c) => {
            // Option+< / Option+> (readline beginning/end of buffer):
            //   M-<  → readline beginning-of-buffer
            //   M->  → readline end-of-buffer
            // User-friendly text navigation in Claude's input box.
            if alt {
                match c {
                    '<' | ',' if shift || c == '<' => {
                        return Action::ForwardKeyToSession {
                            session,
                            key: "M-<",
                        };
                    }
                    '>' | '.' if shift || c == '>' => {
                        return Action::ForwardKeyToSession {
                            session,
                            key: "M->",
                        };
                    }
                    _ => {}
                }
                // Meta+letter — forward WITH the meta prefix. Mac terminals
                // send Option+←/→ as ESC b / ESC f (readline word-jump):
                // dropping the modifier typed a literal 'b' into the agent
                // instead of jumping a word. Full a–z so every readline meta
                // binding (M-b/M-f word motion, M-d kill-word, …) survives.
                let lower = c.to_ascii_lowercase();
                if lower.is_ascii_lowercase() {
                    const META: [&str; 26] = [
                        "M-a", "M-b", "M-c", "M-d", "M-e", "M-f", "M-g", "M-h", "M-i", "M-j",
                        "M-k", "M-l", "M-m", "M-n", "M-o", "M-p", "M-q", "M-r", "M-s", "M-t",
                        "M-u", "M-v", "M-w", "M-x", "M-y", "M-z",
                    ];
                    return Action::ForwardKeyToSession {
                        session,
                        key: META[(lower as u8 - b'a') as usize],
                    };
                }
            }
            // Ctrl+<letter> → rmux "C-<letter>" so Ctrl+C interrupts the agent
            if ctrl {
                let lower = c.to_ascii_lowercase();
                let key_str: &'static str = match lower {
                    'a' => "C-a",
                    'b' => "C-b",
                    'c' => "C-c",
                    'd' => "C-d",
                    'e' => "C-e",
                    'f' => "C-f",
                    'g' => "C-g",
                    'h' => "C-h",
                    'i' => "C-i",
                    'j' => "C-j",
                    'k' => "C-k",
                    'l' => "C-l",
                    'm' => "C-m",
                    'n' => "C-n",
                    'o' => "C-o",
                    'p' => "C-p",
                    'q' => "C-q",
                    'r' => "C-r",
                    's' => "C-s",
                    't' => "C-t",
                    'u' => "C-u",
                    'v' => "C-v",
                    'w' => "C-w",
                    'x' => "C-x",
                    'y' => "C-y",
                    'z' => "C-z",
                    _ => return Action::None,
                };
                Action::ForwardKeyToSession {
                    session,
                    key: key_str,
                }
            } else {
                // Forward EVERY printable char to the agent — including a
                // leading "/". We must NOT intercept "/" for OmegaOS command
                // capture (the deleted cmd_capture experiment): doing so
                // swallowed the keystrokes Claude Code needs for its OWN "/"
                // slash-command menu, so the menu never opened, the selection
                // was invisible, and the capture state could wedge the whole
                // TUI. Claude's "/" works natively; reach OmegaOS tabs via the
                // menu (Tab out of chat) instead.
                Action::ForwardCharToSession { session, ch: c }
            }
        }
        _ => Action::None,
    }
}

fn execute_menu_action(app: &mut App, action: MenuAction) -> Action {
    // Destructive items (KillAll / NuclearCleanup) need a two-press confirm:
    // first Enter arms, second Enter on the SAME item fires. Any other
    // selection disarms.
    let armed = app.menu_confirm_pending == Some(action);
    if !armed {
        app.menu_confirm_pending = None;
    }
    if matches!(action, MenuAction::KillAll | MenuAction::NuclearCleanup) {
        if armed {
            app.menu_confirm_pending = None;
            return match action {
                MenuAction::KillAll => Action::KillAllSessions,
                MenuAction::NuclearCleanup => Action::NuclearCleanup,
                _ => Action::None,
            };
        }
        // Arm — the warning renders state-driven via armed_confirm_warning
        // (FIX-A/T9b), so no status_message duplicate to drift out of sync.
        app.menu_confirm_pending = Some(action);
        return Action::None;
    }

    // Per-agent direct launchers — ALL go straight to session creation with
    // chat focus. No "initial prompt" step. The user talks via the chat input
    // box once the session is up.
    if let Some(agent) = action.agent() {
        // Guard: block launch if the agent CLI is not installed.
        // Shell is always available, so it short-circuits.
        if !matches!(agent, omega_core::agents::Agent::Shell) && !agent.is_available() {
            let install_hint = agent
                .install_command()
                .map(|c| format!(" — install: {}", c))
                .unwrap_or_default();
            app.status_message = Some(format!(
                "{} not installed. Open Settings → Install agents{}",
                agent.display_name(),
                install_hint
            ));
            return Action::None;
        }

        if app.config.auto_naming {
            // Fire-and-attach: agent + auto-name + no prompt, then chat focus
            return Action::CreateSessionAutoName {
                agent,
                prompt: None,
            };
        }
        // Auto-naming disabled → only ask for name (still no prompt step)
        app.input_buffer = String::new();
        app.input_mode = InputMode::NewNamedSession(agent.name().to_string());
        app.status_message = Some(format!(
            "Session name for new {} (Enter to launch, Esc to cancel)",
            agent.display_name()
        ));
        return Action::None;
    }

    match action {
        MenuAction::DispatchOracle => {
            open_dispatch_picker(app);
            Action::None
        }
        MenuAction::NewProject => {
            app.input_buffer = String::new();
            app.input_mode = InputMode::NewProjectName;
            app.status_message =
                Some("New project — name (Enter to continue, Esc to cancel)".to_string());
            Action::None
        }
        MenuAction::Refresh => Action::Refresh,
        MenuAction::ToggleProtection => {
            if let Some(entry) = app.sessions.get_mut(app.selected) {
                entry.is_protected = !entry.is_protected;
                app.status_message = Some(format!(
                    "{} {}",
                    entry.session.name,
                    if entry.is_protected {
                        "protected"
                    } else {
                        "unprotected"
                    }
                ));
            }
            Action::None
        }
        MenuAction::KillSelected => {
            if let Some(entry) = app.selected_session() {
                if entry.is_protected {
                    app.status_message = Some("Selected session is protected".to_string());
                    Action::None
                } else {
                    Action::KillSession(entry.session.name.clone())
                }
            } else {
                app.status_message = Some("No session selected".to_string());
                Action::None
            }
        }
        MenuAction::Restart => Action::Restart,
        MenuAction::Quit => {
            app.should_quit = true;
            Action::Quit
        }
        // Per-agent variants handled above
        _ => Action::None,
    }
}

/// Advance the provisioning-keys wizard: record `value` for the current `step`,
/// then move to the next field or, when the last field is done, emit the commit
/// action with all (key, value) pairs zipped from `PROVISIONING_FIELDS`.
fn provisioning_advance(
    app: &mut App,
    mut collected: Vec<String>,
    value: String,
    step: usize,
) -> Action {
    let fields = crate::app::PROVISIONING_FIELDS;
    collected.push(value);
    let next = step + 1;
    if next >= fields.len() {
        app.input_mode = InputMode::Normal;
        let values: Vec<(String, String)> = fields
            .iter()
            .zip(collected.iter())
            .map(|((k, _, _), v)| (k.to_string(), v.clone()))
            .collect();
        Action::ProvisioningCommit { values }
    } else {
        app.input_buffer = String::new();
        app.input_mode = InputMode::ProvisioningSetup {
            step: next,
            collected,
        };
        app.status_message = Some(format!(
            "Step {}/{}: {}",
            next + 1,
            fields.len(),
            fields[next].1
        ));
        Action::None
    }
}

fn handle_key_input<F>(app: &mut App, key: KeyEvent, on_submit: F) -> Action
where
    F: FnOnce(&mut App, String) -> Action,
{
    match key.code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.input_buffer = String::new();
            app.status_message = Some("Cancelled".to_string());
            Action::None
        }
        KeyCode::Enter => {
            let value = std::mem::take(&mut app.input_buffer);
            if value.trim().is_empty() {
                app.input_mode = InputMode::Normal;
                app.status_message = Some("Cancelled (empty input)".to_string());
                Action::None
            } else {
                // Every handle_key_input field is single-line (names, tokens,
                // paths, codes), so edge whitespace is never meaningful — but
                // a bracketed PASTE delivers it verbatim, and a bot token with
                // a trailing '\n' was written to telegram.toml as-is: the bot
                // 401'd while the masked echo looked perfectly fine. Submit
                // trimmed.
                on_submit(app, value.trim().to_string())
            }
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
            Action::None
        }
        KeyCode::Char(c) => {
            app.input_buffer.push(c);
            Action::None
        }
        _ => Action::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::SessionEntry;
    use omega_core::config::OmegaConfig;
    use omega_core::session::OmegaSession;

    fn test_app() -> App {
        App::new(OmegaConfig::default())
    }
    fn press(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
    }

    // Dispatch step-1 is a PROJECT PICKER (no typing): ↑/↓ navigate the
    // added-projects list, Enter selects → step-2 mission entry. (User ask:
    // present the added projects instead of forcing the operator to type a name.)
    #[test]
    fn dispatch_picker_navigates_and_selects() {
        let mut app = test_app();
        let projects = vec!["Alpha".to_string(), "Beta".to_string(), "Gamma".to_string()];
        app.input_mode = InputMode::DispatchProject(projects, 0);

        let down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let up = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        // Down twice → index 2 (Gamma), Up once → index 1 (Beta).
        handle_key(&mut app, down);
        handle_key(&mut app, down);
        handle_key(&mut app, up);
        assert!(matches!(&app.input_mode, InputMode::DispatchProject(_, 1)));

        // Enter commits the highlighted project and advances to mission entry.
        let action = handle_key(&mut app, enter);
        assert!(matches!(action, Action::None));
        assert!(matches!(&app.input_mode, InputMode::DispatchMission(p) if p == "Beta"));
    }

    #[test]
    fn dispatch_picker_refuses_a_poisoned_project_registry() {
        let mut app = test_app();
        app.project_registry.poisoned = true;

        open_dispatch_picker(&mut app);

        assert!(matches!(app.input_mode, InputMode::Normal));
        assert!(app
            .status_message
            .as_deref()
            .is_some_and(|message| message.contains("project registry is invalid")));
    }

    // The tab bar reads Sessions · Projects · OS · Menu · System · Help ·
    // Settings, and Right/Left walk it in that visual order. Locked down
    // because the order used to live in three hand-kept lists that drifted apart.
    #[test]
    fn tab_order_matches_the_bar_and_cycles_both_ways() {
        let titles: Vec<&str> = Tab::ORDER.iter().map(|t| t.title()).collect();
        assert_eq!(
            titles,
            vec!["Sessions", "Projects", "OS", "Menu", "System", "Help", "Settings"]
        );
        for (i, t) in Tab::ORDER.iter().enumerate() {
            assert_eq!(t.index(), i, "{} must sit at bar position {}", t.title(), i);
        }

        // Right walks forward and wraps.
        let mut app = test_app();
        app.tab = Tab::Sessions;
        for expected in Tab::ORDER
            .iter()
            .skip(1)
            .chain(std::iter::once(&Tab::Sessions))
        {
            app.next_tab();
            assert_eq!(app.tab, *expected, "next_tab landed on the wrong tab");
        }
        // Left walks back and wraps.
        for expected in Tab::ORDER.iter().rev() {
            app.prev_tab();
            assert_eq!(app.tab, *expected, "prev_tab landed on the wrong tab");
        }
    }

    // Reading agent output from a phone: the plain arrows belong to the agent,
    // so without a modifier-free-ish alternative the preview stays glued to the
    // tail and the text slides away under the reader. Alt+arrow already worked;
    // Shift+arrow is the one a soft keyboard actually has. Both must scroll,
    // and scrolling up must PAUSE the tail-follow.
    #[test]
    fn shift_and_alt_arrows_pause_the_tail_so_it_can_be_read() {
        for modifier in [KeyModifiers::SHIFT, KeyModifiers::ALT] {
            let mut app = test_app();
            // Chat focus needs a session to chat with — with an empty list the
            // handler drops back to list mode and never reaches the scroll keys.
            app.sessions.push(SessionEntry {
                session: OmegaSession::classify("test-worker"),
                progress: None,
                is_current: false,
                is_protected: false,
                tree_prefix: String::new(),
            });
            app.selected = 0;
            app.session_focus = crate::app::SessionFocus::Chat;
            app.preview_follow_tail = true;
            app.preview_scroll = 0;

            handle_key(&mut app, KeyEvent::new(KeyCode::Up, modifier));
            assert!(
                !app.preview_follow_tail,
                "{:?}+Up must stop the view chasing the tail",
                modifier
            );
            assert!(
                app.preview_scroll > 0,
                "{:?}+Up must move into history",
                modifier
            );

            // And back down to the tail re-glues to live.
            app.preview_max_scroll = 100;
            handle_key(&mut app, KeyEvent::new(KeyCode::Down, modifier));
            handle_key(&mut app, KeyEvent::new(KeyCode::Down, modifier));
            assert!(
                app.preview_scroll < 6,
                "{:?}+Down must walk back toward live",
                modifier
            );
        }
    }

    // The doctrine surface must stay REACHABLE as its own tab. It was lost once
    // already: Info → renamed Agentic → Agentic repurposed into Projects, and
    // its sections survived only as a buried group above the project list. This
    // test fails the moment System stops being a top-level tab.
    #[test]
    fn system_tab_is_a_real_tab_carrying_the_doctrine() {
        assert!(
            Tab::ORDER.contains(&Tab::System),
            "System must sit in the tab bar, not inside another tab"
        );

        use crate::app::InfoSection;
        let sections = InfoSection::all();
        // The five things the operator asked to be able to read.
        for required in [
            InfoSection::Laws,
            InfoSection::Rules,
            InfoSection::AisbAgents,
            InfoSection::Skills,
            InfoSection::Docs,
        ] {
            assert!(
                sections.contains(&required),
                "{:?} must be readable",
                required
            );
        }
    }

    #[test]
    fn help_page_keys_scroll_the_help_document_not_the_session_preview() {
        let mut app = test_app();
        app.tab = Tab::Help;
        app.detail_max_scroll = 100;
        app.preview_scroll = 7;
        app.preview_max_scroll = 200;

        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE),
        );
        assert_eq!(app.detail_scroll, 10);
        assert_eq!(
            app.preview_scroll, 7,
            "Help keys must not mutate session preview state"
        );

        handle_key(&mut app, KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
        assert_eq!(app.detail_scroll, 100);
        handle_key(&mut app, KeyEvent::new(KeyCode::Home, KeyModifiers::NONE));
        assert_eq!(app.detail_scroll, 0);
    }

    // ↑/↓ walk the sections while the list has focus, and wrap both ways.
    #[test]
    fn system_sections_navigate_and_wrap() {
        use crate::app::InfoSection;
        let mut app = test_app();
        app.tab = Tab::System;
        app.detail_focused = false;
        let last = InfoSection::all().len() - 1;

        let down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let up = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);

        handle_key(&mut app, down);
        assert_eq!(app.info_section_selected, 1);
        handle_key(&mut app, up);
        assert_eq!(app.info_section_selected, 0);
        // Up from the first section wraps to the last.
        handle_key(&mut app, up);
        assert_eq!(app.info_section_selected, last);
        // Last section is Docs: landing auto-focuses the manual list, so Down
        // would move a document, not wrap. [ / ] (or Tab) leave the submenu.
        assert!(
            app.detail_focused,
            "Docs submenu must take ↑/↓ so the manual is not a dead list"
        );
        handle_key(&mut app, press(']'));
        assert_eq!(
            app.info_section_selected, 0,
            "] from the last section wraps home"
        );
    }

    // With the detail focused, ↑/↓ belong to the section's own list (agents,
    // documents) — [ and ] are what change section, since the arrows are taken.
    #[test]
    fn focused_detail_moves_the_sub_cursor_and_brackets_change_section() {
        use crate::app::InfoSection;
        let mut app = test_app();
        app.tab = Tab::System;
        app.info_section_selected = InfoSection::all()
            .iter()
            .position(|s| *s == InfoSection::AisbAgents)
            .expect("AI Agents section");
        app.detail_focused = true;

        let before = app.info_section_selected;
        handle_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.info_agent_selected, 1, "↑/↓ drive the agent list");
        assert_eq!(
            app.info_section_selected, before,
            "and never the section list"
        );

        handle_key(&mut app, press(']'));
        assert_eq!(
            app.info_section_selected,
            before + 1,
            "] moves to the next section"
        );
        handle_key(&mut app, press('['));
        assert_eq!(app.info_section_selected, before, "[ moves back");
        assert_eq!(
            app.info_agent_selected, 0,
            "a section change resets the sub-cursor"
        );
    }

    // The Projects tab is projects and nothing else now — its cursor must walk
    // the project rows directly, never stall on a phantom leading group that no
    // longer renders. Registry is set explicitly: the machine's real one would
    // make this pass or fail depending on how many projects are registered.
    #[test]
    fn projects_cursor_walks_projects_only() {
        let mut app = test_app();
        app.tab = Tab::Projects;
        app.detail_focused = false;
        app.project_registry.projects.clear();

        // Empty registry: only the pinned OS row, the cursor stays on it.
        handle_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.projects_selected, 0);
    }

    // The pinned "OS System" quick-access row is selection index 0: projects
    // shift to 1..=N, selected_project() offsets by one, and Enter on the pinned
    // row jumps straight to the OS tab (the operator ask: reach the suite from
    // the Projects list without cycling tabs).
    #[test]
    fn projects_pinned_os_row_maps_indices_and_enter_jumps_to_os_tab() {
        use omega_core::project_manager::ManagedProject;
        let mk = |n: &str| ManagedProject {
            name: n.to_string(),
            path: std::path::PathBuf::from(format!("/tmp/{n}")),
            telegram_topic_id: None,
            oracle_session: None,
            git_email: None,
            created_at: String::new(),
            telegram: None,
            category: None,
        };
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        // Index 0 = pinned OS row: no project, reports pinned, Enter → OS tab.
        let mut app = test_app();
        app.tab = Tab::Projects;
        app.detail_focused = false;
        app.project_registry.projects = vec![mk("Alpha"), mk("Beta")];
        app.projects_selected = 0;
        assert!(app.projects_os_pinned());
        assert!(app.selected_project().is_none());
        let action = handle_key(&mut app, enter);
        assert!(matches!(action, Action::None));
        assert_eq!(app.tab, Tab::Os, "Enter on the pinned row opens the OS tab");
        assert!(
            !app.detail_focused,
            "the jump does not leave detail focused"
        );

        // Index 1..=N are the projects, shifted one past the pinned row.
        let mut app = test_app();
        app.tab = Tab::Projects;
        app.detail_focused = false;
        app.project_registry.projects = vec![mk("Alpha"), mk("Beta")];
        app.projects_selected = 1;
        assert_eq!(
            app.selected_project().map(|p| p.name.as_str()),
            Some("Alpha")
        );
        app.projects_selected = 2;
        assert_eq!(
            app.selected_project().map(|p| p.name.as_str()),
            Some("Beta")
        );

        // Arrow-nav wraps across pinned + projects (count = N + 1 = 3).
        app.projects_selected = 2;
        handle_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(
            app.projects_selected, 0,
            "Down past the last project wraps to the pinned row"
        );
        handle_key(&mut app, KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(
            app.projects_selected, 2,
            "Up from the pinned row wraps to the last project"
        );
    }

    // Opening a project goes lane -> installed-agent -> action: the coding
    // lane emits OpenProject with the picked agent, the marketing lane emits
    // OpenMarketingSession into <project>/marketing/ with the picked agent.
    #[test]
    fn open_project_two_step_picker_emits_lane_and_agent() {
        use omega_core::agents::Agent;
        let mut app = test_app();
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        // Coding lane, second agent picked via Down.
        app.input_mode = InputMode::ProjectOpenAgentPick {
            lane: ProjectLane::Coding,
            name: "Verba".into(),
            path: "/tmp/verba".into(),
            agents: vec![Agent::Codex, Agent::Claude],
            sel: 0,
        };
        handle_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        match handle_key(&mut app, enter) {
            Action::OpenProject { name, path, agent } => {
                assert_eq!(name, "Verba");
                assert_eq!(path, "/tmp/verba");
                assert_eq!(agent, Agent::Claude);
            }
            _ => panic!("expected OpenProject with Claude"),
        }
        assert!(matches!(app.input_mode, InputMode::Normal));

        // Marketing lane: session lands in <project>/marketing/ with the agent.
        app.input_mode = InputMode::ProjectOpenAgentPick {
            lane: ProjectLane::Marketing,
            name: "Verba".into(),
            path: "/tmp/verba".into(),
            agents: vec![Agent::Codex, Agent::Claude],
            sel: 0,
        };
        match handle_key(&mut app, enter) {
            Action::OpenMarketingSession {
                name,
                cwd,
                prompt,
                agent,
            } => {
                assert_eq!(name, "mkt-verba");
                assert_eq!(cwd, "/tmp/verba/marketing");
                assert!(prompt.contains("MARKETING"));
                assert_eq!(agent, Agent::Codex);
            }
            _ => panic!("expected OpenMarketingSession"),
        }
    }

    // Step 1: '3' routes to the oracle mission prompt; Cancel/Esc open nothing.
    // Step 2: Esc goes BACK to step 1, digits pick an agent directly.
    #[test]
    fn open_project_lane_picker_routes_and_cancels() {
        use omega_core::agents::Agent;
        let mut app = test_app();

        // '3' -> oracle mission input for THIS project.
        app.input_mode = InputMode::ProjectOpenLane("Verba".into(), "/tmp/verba".into(), 0);
        let action = handle_key(&mut app, press('3'));
        assert!(matches!(action, Action::None));
        assert!(matches!(&app.input_mode, InputMode::DispatchMission(p) if p == "Verba"));

        // Cancel row -> nothing opened.
        app.input_mode = InputMode::ProjectOpenLane("Verba".into(), "/tmp/verba".into(), 3);
        let action = handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(
            matches!(action, Action::None),
            "Cancel must not open a session"
        );
        assert!(matches!(app.input_mode, InputMode::Normal));

        // Step 2 Esc -> back to the lane picker (not a hard cancel).
        app.input_mode = InputMode::ProjectOpenAgentPick {
            lane: ProjectLane::Coding,
            name: "Verba".into(),
            path: "/tmp/verba".into(),
            agents: vec![Agent::Codex],
            sel: 0,
        };
        handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(matches!(app.input_mode, InputMode::ProjectOpenLane(..)));

        // Digit '1' on step 2 picks the first installed agent.
        app.input_mode = InputMode::ProjectOpenAgentPick {
            lane: ProjectLane::Coding,
            name: "Verba".into(),
            path: "/tmp/verba".into(),
            agents: vec![Agent::Codex, Agent::Claude],
            sel: 0,
        };
        match handle_key(&mut app, press('1')) {
            Action::OpenProject { agent, .. } => assert_eq!(agent, Agent::Codex),
            _ => panic!("expected OpenProject via digit"),
        }
    }

    // Up from the first item wraps to the last (matches the SelectModel picker).
    #[test]
    fn dispatch_picker_wraps_at_edges() {
        let mut app = test_app();
        app.input_mode = InputMode::DispatchProject(vec!["A".into(), "B".into(), "C".into()], 0);
        let up = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        handle_key(&mut app, up);
        assert!(
            matches!(&app.input_mode, InputMode::DispatchProject(_, 2)),
            "Up from first item wraps to last"
        );
    }

    // Bug B: 'h' is advertised as the Hermes launcher; it must work from any
    // tab like the other global launchers (p/G/t), not be a dead key.
    #[test]
    fn h_launches_hermes_from_any_tab() {
        let mut app = test_app();
        app.tab = Tab::Settings;
        let action = handle_key(&mut app, press('h'));
        assert!(matches!(action, Action::None));
        assert!(matches!(&app.input_mode, InputMode::NewNamedSession(s) if s == "hermes"));
    }

    // Bug A: destructive keys act only on the Sessions tab, never on a hidden
    // selection from another tab.
    #[test]
    fn kill_key_is_guarded_to_sessions_tab() {
        let mut app = test_app();
        app.sessions.push(SessionEntry {
            session: OmegaSession::classify("test-worker"),
            progress: None,
            is_current: false,
            is_protected: false,
            tree_prefix: String::new(),
        });
        app.selected = 0;

        app.tab = Tab::Settings;
        assert!(
            matches!(handle_key(&mut app, press('x')), Action::None),
            "x off the Sessions tab must not kill the hidden selection"
        );

        app.tab = Tab::Sessions;
        assert!(
            matches!(handle_key(&mut app, press('x')), Action::KillSession(n) if n == "test-worker"),
            "x on the Sessions tab must kill the selected session"
        );
    }

    // Soft-lock regression: chat focus with an EMPTY session list used to
    // swallow every key (even q and Tab) because handle_key_chat returned
    // Action::None when selected_session() was None — locking the whole UI. It
    // must instead recover to list focus and still act on the keystroke.
    #[test]
    fn chat_focus_with_empty_list_does_not_soft_lock() {
        let mut app = test_app();
        app.tab = Tab::Sessions;
        app.session_focus = SessionFocus::Chat;
        assert!(
            app.sessions.is_empty(),
            "precondition: no session to chat with"
        );

        let action = handle_key(&mut app, press('q'));
        assert!(
            matches!(action, Action::Quit),
            "q must still quit, not be swallowed in empty-list chat focus"
        );
        assert!(
            matches!(app.session_focus, SessionFocus::List),
            "focus must recover to the session list, not stay stuck in chat"
        );
    }

    /// Daemon-free App chat-focused on a single session (no rmux needed).
    fn chat_app(name: &str) -> App {
        let mut app = test_app();
        app.tab = Tab::Sessions;
        app.sessions.push(SessionEntry {
            session: OmegaSession::classify(name),
            progress: None,
            is_current: false,
            is_protected: false,
            tree_prefix: String::new(),
        });
        app.selected = 0;
        app.session_focus = SessionFocus::Chat;
        app
    }

    // Esc belongs to the AGENT, not the TUI: modal programs inside the session
    // (vim, less, fzf, Claude's own prompts) are escaped from inside. Leaving
    // the session is Tab. Regression: Esc used to drop to the list, which left
    // the user stuck inside any prompt that reads ESC.
    #[test]
    fn chat_esc_forwards_to_agent_and_stays_in_chat() {
        let mut app = chat_app("oracle-Demo-1");
        let act = handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(
            matches!(act, Action::ForwardKeyToSession { ref session, key }
                if session == "oracle-Demo-1" && key == "Escape"),
            "chat Esc must forward rmux \"Escape\" to the focused session"
        );
        assert_eq!(
            app.session_focus,
            SessionFocus::Chat,
            "Esc must not exit chat focus — Tab does that"
        );
        assert!(!app.should_quit, "chat Esc must never quit the TUI");
        assert!(!app.in_post_drop_grace(), "no focus drop → no grace");
    }

    // Tab (not Esc) is the way out of a session, and it must leave no stale
    // double-tap state behind (AF-7/AF-3/CA-2): Tab → Tab inside the 400ms
    // window from the LIST must navigate, not read as tap #2 of a Tab-Tab.
    #[test]
    fn chat_tab_returns_to_list() {
        let mut app = chat_app("oracle-Demo-1");
        handle_key(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.session_focus, SessionFocus::List);
    }

    // F-12/DESIGN-004 regression: a protected session must NEVER die from a
    // chat-focus Ctrl+X — same guard as the list-focus 'x'.
    #[test]
    fn chat_ctrl_x_never_kills_a_protected_session() {
        let mut app = chat_app("oracle-Demo-1");
        app.sessions[0].is_protected = true;
        let act = handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL),
        );
        assert!(
            matches!(act, Action::None),
            "protected session must not die"
        );
        assert_eq!(
            app.session_focus,
            SessionFocus::Chat,
            "nothing was killed — stay in chat"
        );
        assert!(app
            .status_message
            .as_deref()
            .unwrap_or("")
            .contains("protected"));
    }

    #[test]
    fn chat_ctrl_x_kills_unprotected_and_drops_to_list() {
        let mut app = chat_app("oracle-Demo-1");
        let act = handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL),
        );
        assert!(matches!(act, Action::KillSession(ref n) if n == "oracle-Demo-1"));
        assert_eq!(app.session_focus, SessionFocus::List);
        assert!(
            app.last_tab_press.is_none() && app.tab_seq_start.is_none(),
            "Ctrl+X focus drop must clear the Tab chord state"
        );
    }

    // AF-1: Alt+Esc must deliver a literal ESC to the agent PTY (vim/less).
    #[test]
    fn chat_alt_esc_forwards_literal_escape() {
        let mut app = chat_app("oracle-Demo-1");
        let act = handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::ALT));
        assert!(
            matches!(act, Action::ForwardKeyToSession { key, .. } if key == "Escape"),
            "Alt+Esc must forward rmux key \"Escape\""
        );
        assert_eq!(
            app.session_focus,
            SessionFocus::Chat,
            "Alt+Esc must not exit chat focus"
        );
    }

    // AF-5: when the focused session died, Esc must mean "back to the list",
    // not fall through to the Sessions-tab quit arm (one-keypress app exit).
    #[test]
    fn dead_session_esc_does_not_quit_in_one_press() {
        let mut app = test_app();
        app.tab = Tab::Sessions;
        app.session_focus = SessionFocus::Chat;
        assert!(
            app.sessions.is_empty(),
            "precondition: focused session is gone"
        );

        let act = handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(
            matches!(act, Action::None),
            "Esc must be swallowed, not quit"
        );
        assert!(
            !app.should_quit,
            "one Esc on a dead session must not quit the TUI"
        );
        assert_eq!(app.session_focus, SessionFocus::List);
    }

    // FIX-1/NEW-1 regression: the clear-on-keypress TTL must NOT wipe the
    // KillAll confirm warning while the armed state persists — arm → Down →
    // Enter must either still show the warning or not fire the mass-kill.
    #[test]
    fn menu_confirm_warning_survives_ttl_and_navigation() {
        let mut app = test_app();
        app.tab = Tab::Menu;
        app.menu_selected = MenuAction::all()
            .iter()
            .position(|a| matches!(a, MenuAction::KillAll))
            .unwrap();

        // First Enter arms + the warning renders state-driven (FIX-A/T9b —
        // arm sites no longer duplicate the text into status_message).
        let act = handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(matches!(act, Action::None));
        assert_eq!(app.menu_confirm_pending, Some(MenuAction::KillAll));
        assert!(app
            .armed_confirm_warning()
            .unwrap_or_default()
            .contains("CONFIRM"));

        // The next keypress's TTL (runs BEFORE dispatch) can't touch the
        // state-driven warning.
        app.consume_status_ttl();
        assert!(
            app.armed_confirm_warning()
                .unwrap_or_default()
                .contains("CONFIRM"),
            "TTL must not wipe the confirm warning while armed"
        );

        // Down (browse) keeps both state and indicator in lockstep.
        handle_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        app.consume_status_ttl();
        let visible = app
            .armed_confirm_warning()
            .unwrap_or_default()
            .contains("CONFIRM");

        // Enter on the OTHER row must not fire the armed mass-kill.
        let act = handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        let fired = matches!(act, Action::KillAllSessions);
        assert!(
            visible && !fired,
            "arm → Down → Enter must show the warning AND not fire (visible={visible}, fired={fired})"
        );
    }

    // FIX-1 companion: the TTL must never DISARM — the second Enter on the
    // same row still fires (a TTL-disarm would re-arm instead).
    #[test]
    fn menu_confirm_second_enter_still_fires_after_ttl() {
        let mut app = test_app();
        app.tab = Tab::Menu;
        app.menu_selected = MenuAction::all()
            .iter()
            .position(|a| matches!(a, MenuAction::KillAll))
            .unwrap();
        handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        app.consume_status_ttl(); // the confirming Enter's own TTL pass
        let act = handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(
            matches!(act, Action::KillAllSessions),
            "confirming Enter must fire — TTL must not have disarmed"
        );
        assert_eq!(app.menu_confirm_pending, None);
    }

    // FIX-1: leaving the tab cancels the armed confirm — the per-tab hint
    // overwrites the warning, so the armed state must not outlive it.
    #[test]
    fn menu_confirm_disarmed_on_tab_switch() {
        let mut app = test_app();
        app.tab = Tab::Menu;
        app.menu_selected = MenuAction::all()
            .iter()
            .position(|a| matches!(a, MenuAction::KillAll))
            .unwrap();
        handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        handle_key(&mut app, KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(app.menu_confirm_pending, None, "tab switch must disarm");
    }

    // The DESIGN-014 Esc-Esc chord is gone: a chat Esc IS the literal ESC now.
    // Legacy terminals deliver Alt+Esc as a split ESC ESC pair — both halves
    // must forward to the agent and neither may quit the TUI (the old hazard).
    #[test]
    fn repeated_chat_esc_forwards_each_time_and_never_quits() {
        let mut app = chat_app("oracle-Demo-1");
        let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        for press in 1..=2 {
            let act = handle_key(&mut app, esc);
            assert!(
                matches!(act, Action::ForwardKeyToSession { key, .. } if key == "Escape"),
                "Esc #{press} must forward rmux \"Escape\""
            );
            assert!(!app.should_quit, "Esc #{press} must never quit the TUI");
            assert_eq!(app.session_focus, SessionFocus::Chat);
        }
    }

    /// The only remaining non-deliberate focus drop: the focused session
    /// vanished under the cursor (App::refresh clamps to the list and opens
    /// the DESIGN-015 grace). Mirrors that arm for the grace tests below.
    fn vanish_drop(app: &mut App) {
        app.set_list_focus();
        app.focus_drop_at = Some(std::time::Instant::now());
    }

    // DESIGN-015/NR-2: a vanish-drop mid-keystream must not let the in-flight
    // keys fall through to the destructive single-key list hotkeys (x kill /
    // q quit); a deliberate navigation key re-enables them.
    #[test]
    fn vanish_drop_grace_blocks_destructive_hotkeys() {
        let mut app = chat_app("oracle-Demo-1");
        vanish_drop(&mut app);
        assert_eq!(app.session_focus, SessionFocus::List);

        let act = handle_key(&mut app, press('x'));
        assert!(matches!(act, Action::None), "in-flight 'x' must not kill");
        assert_eq!(app.session_focus, SessionFocus::List);
        let act = handle_key(&mut app, press('q'));
        assert!(
            matches!(act, Action::None) && !app.should_quit,
            "in-flight 'q' must not quit"
        );

        // A navigation key ends the grace — deliberate hotkeys work again.
        handle_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        let act = handle_key(&mut app, press('q'));
        assert!(
            matches!(act, Action::Quit),
            "after navigation, q must quit again"
        );
    }

    // DESIGN-015: the grace expires by time too (no navigation needed).
    #[test]
    fn post_drop_grace_expires_after_window() {
        let mut app = chat_app("oracle-Demo-1");
        vanish_drop(&mut app);
        // Backdate the drop beyond the 800ms window (FIX-D widened it).
        app.focus_drop_at =
            std::time::Instant::now().checked_sub(std::time::Duration::from_millis(900));
        let act = handle_key(&mut app, press('q'));
        assert!(matches!(act, Action::Quit), "expired grace must restore q");
    }

    // FIX-4/NEW-4: Tab (arm) → Enter (enter chat) → Tab inside 400ms must
    // navigate as a fresh first tap, not complete a stale chord into
    // ChatFullscreen — enter_chat_focus() now owns the chord reset.
    #[test]
    fn tab_enter_tab_does_not_complete_stale_chord() {
        let mut app = chat_app("oracle-Demo-1");
        // Tab from chat → List (first tap, chord armed).
        handle_key(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.session_focus, SessionFocus::List);
        // Enter → chat via the canonical path (must clear the chord).
        handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.session_focus, SessionFocus::Chat);
        assert!(
            app.last_tab_press.is_none(),
            "Enter→chat must clear the chord"
        );
        // Tab right after → fresh first tap → List, NOT ChatFullscreen.
        handle_key(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(
            app.session_focus,
            SessionFocus::List,
            "post-Enter Tab must navigate, not land in ChatFullscreen"
        );
    }

    // FIX-4 (2col leak): a Tab pressed on Sessions must not leak into the
    // Settings tab's double-tap detection after a Left/Right tab switch.
    #[test]
    fn tab_chord_does_not_leak_across_tab_switch() {
        let mut app = chat_app("oracle-Demo-1");
        // Tab from chat → List: first tap, chord armed, list focus (so the
        // following Right reaches the normal handler, not the chat handler).
        handle_key(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)); // arm
        assert_eq!(app.session_focus, SessionFocus::List);
        handle_key(&mut app, KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)); // → Projects (a 2col tab)
        assert_eq!(app.tab, Tab::Projects);
        handle_key(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert!(
            !app.detail_fullscreen,
            "a leaked Sessions chord must not fullscreen the 2col detail"
        );
        assert!(
            app.detail_focused,
            "single Tab on a 2col tab focuses the detail"
        );
    }

    // FIX-B (R-5): Esc must cancel EVERY armed two-press confirm. The
    // project-delete warning advertised "Esc to cancel" while the Esc arm
    // omitted the flag — an advertised-but-dead cancel in front of an rm -rf.
    #[test]
    fn esc_cancels_every_armed_confirm_state() {
        let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        // project_delete_pending (Projects tab 'D' — the rm -rf class).
        let mut app = test_app();
        app.tab = Tab::Projects;
        app.project_delete_pending = Some("Demo".into());
        assert!(matches!(handle_key(&mut app, esc), Action::None));
        assert_eq!(app.project_delete_pending, None, "Esc must disarm 'D'");
        assert_eq!(app.status_message.as_deref(), Some("Cancelled"));

        // settings_confirm_pending (Settings Enter/x arm — uninstall class).
        let mut app = test_app();
        app.tab = Tab::Settings;
        app.detail_focused = true;
        app.settings_confirm_pending = Some((1, "[Uninstall] demo".into()));
        handle_key(&mut app, esc);
        assert_eq!(
            app.settings_confirm_pending, None,
            "Esc must disarm settings confirm"
        );
        assert!(
            app.detail_focused,
            "the cancel CONSUMES the Esc — focus unchanged"
        );

        // monitor_disconnect_armed (wired pre-fix5 — keep covered).
        let mut app = test_app();
        app.tab = Tab::Settings;
        app.monitor_disconnect_armed = true;
        handle_key(&mut app, esc);
        assert!(
            !app.monitor_disconnect_armed,
            "Esc must disarm the disconnect"
        );

        // menu_confirm_pending (KillAll).
        let mut app = test_app();
        app.tab = Tab::Menu;
        app.menu_confirm_pending = Some(MenuAction::KillAll);
        handle_key(&mut app, esc);
        assert_eq!(
            app.menu_confirm_pending, None,
            "Esc must disarm the menu confirm"
        );
    }

    // FIX-B (D-9): a tab switch (leave_tab) cancels every armed confirm —
    // none may survive into a tab where its context is invisible.
    #[test]
    fn tab_switch_cancels_all_armed_confirms() {
        let mut app = test_app();
        app.tab = Tab::Projects;
        app.project_delete_pending = Some("Demo".into());
        app.settings_confirm_pending = Some((0, "[Uninstall] demo".into()));
        app.monitor_disconnect_armed = true;
        handle_key(&mut app, KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(app.project_delete_pending, None);
        assert_eq!(app.settings_confirm_pending, None);
        assert!(!app.monitor_disconnect_armed);
    }

    // FIX-D (D-5/D-11): an in-flight Esc landing in the list right after a
    // vanish-drop must be swallowed WITH a notice, never quit the TUI — the
    // keystream was aimed at a session that died, not at the quit hotkey.
    #[test]
    fn esc_in_vanish_grace_swallowed_with_notice_not_quit() {
        let mut app = chat_app("oracle-Demo-1");
        vanish_drop(&mut app);

        let act = handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(matches!(act, Action::None), "the in-grace Esc must not act");
        assert!(!app.should_quit, "the in-grace Esc must NOT quit the TUI");
        assert!(
            app.status_message
                .as_deref()
                .unwrap_or("")
                .contains("ignored"),
            "the swallow must be announced, got {:?}",
            app.status_message
        );
    }

    /// fix6-T1 helper: App parked on the Settings → Install section with the
    /// detail focused and the cursor on the first confirm-first Action field.
    /// Returns (app, field_idx, field_label).
    fn settings_install_app() -> (App, usize, String) {
        let mut app = test_app();
        app.tab = Tab::Settings;
        app.settings_group = 1; // Settings group (not Monitor)
        app.settings_selected = crate::app::SettingsSection::all()
            .iter()
            .position(|s| matches!(s, crate::app::SettingsSection::Install))
            .unwrap();
        app.detail_focused = true;
        let providers = app.providers();
        let fields = crate::app::fields_for_section(
            crate::app::SettingsSection::Install,
            &providers,
            &app.config,
        );
        let idx = fields
            .iter()
            .position(|f| {
                matches!(
                    f,
                    crate::app::SettingsField::Action {
                        confirm_first: true,
                        ..
                    }
                )
            })
            .expect("Install section always exposes a confirm-first action");
        let label = fields[idx].label().to_string();
        app.settings_field_selected = idx;
        (app, idx, label)
    }

    // fix6-T1: a row shift between arm and confirm (a background [Install]
    // finishing inserts/removes rows) must NOT fire the field now sitting at
    // the armed index — disarm with a notice instead.
    #[test]
    fn settings_confirm_does_not_fire_after_row_shift() {
        let (mut app, idx, _label) = settings_install_app();
        // Arm with a STALE pinned identity — simulates the field list having
        // shifted under the index after the arming Enter.
        app.settings_confirm_pending = Some((idx, "[Uninstall] ghost-agent".into()));
        let act = handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(
            !matches!(act, Action::RunShellCommand { .. }),
            "the confirming Enter must NOT fire a different field"
        );
        assert_eq!(
            app.settings_confirm_pending, None,
            "identity mismatch must disarm"
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

    // fix6-T1 companion: the unshifted happy path still arms on the first
    // Enter (pinning the field identity) and fires on the second.
    #[test]
    fn settings_confirm_arms_identity_and_fires_when_stable() {
        let (mut app, idx, label) = settings_install_app();
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let act = handle_key(&mut app, enter);
        assert!(matches!(act, Action::None));
        assert_eq!(
            app.settings_confirm_pending,
            Some((idx, label)),
            "first Enter must pin the field identity"
        );
        let act = handle_key(&mut app, enter);
        assert!(
            matches!(act, Action::RunShellCommand { .. }),
            "second Enter on the unchanged field must fire"
        );
        assert_eq!(app.settings_confirm_pending, None);
    }

    // fix6-T2: the grace is a deny-by-default intercept — previously-unguarded
    // modal openers (launchers c/C/g/p/G/t/h, dispatch 'd', rename 'r') must
    // be swallowed during the grace instead of opening a modal that bypasses
    // it (once input_mode != Normal, handle_key routes by mode, ungated).
    #[test]
    fn grace_swallows_unguarded_modal_openers() {
        for key in ['c', 'C', 'g', 'p', 'G', 't', 'h', 'd', 'r'] {
            let mut app = chat_app("oracle-Demo-1");
            vanish_drop(&mut app);
            assert_eq!(app.session_focus, SessionFocus::List);
            let act = handle_key(&mut app, press(key));
            assert!(
                matches!(act, Action::None),
                "'{key}' must be swallowed in the grace"
            );
            assert!(
                matches!(app.input_mode, InputMode::Normal),
                "'{key}' must not open a modal during the grace"
            );
            assert!(
                app.status_message
                    .as_deref()
                    .unwrap_or("")
                    .contains("ignored"),
                "the swallow must be announced for '{key}', got {:?}",
                app.status_message
            );
        }
    }

    // fix6-T3: Esc cancels ALL armed two-press confirms atomically — the old
    // short-circuit chain cleared only the first armed state, so a dual-arm
    // needed one Esc per state.
    #[test]
    fn esc_cancels_dual_armed_states_atomically() {
        let mut app = test_app();
        app.tab = Tab::Menu;
        app.menu_confirm_pending = Some(MenuAction::KillAll);
        app.settings_confirm_pending = Some((0, "[Uninstall] demo".into()));
        app.project_delete_pending = Some("Demo".into());
        app.monitor_disconnect_armed = true;
        let act = handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(matches!(act, Action::None));
        assert_eq!(
            app.menu_confirm_pending, None,
            "one Esc must clear the menu arm"
        );
        assert_eq!(app.settings_confirm_pending, None, "…and the settings arm");
        assert_eq!(
            app.project_delete_pending, None,
            "…and the project-delete arm"
        );
        assert!(!app.monitor_disconnect_armed, "…and the disconnect arm");
        assert_eq!(app.status_message.as_deref(), Some("Cancelled"));
    }

    // fix6-T4: inside a vanish-drop grace, Enter/Tab stay swallowed — the
    // session the keystream was aimed at is gone, and whatever clamped into
    // the selected slot must not receive the in-flight keys. A deliberate
    // navigation key (↑/↓) ends the grace and restores re-entry.
    #[test]
    fn grace_enter_and_tab_swallowed_until_navigation() {
        let mut app = chat_app("oracle-Demo-1");
        vanish_drop(&mut app);
        let act = handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(
            matches!(act, Action::None),
            "in-grace Enter must be swallowed"
        );
        assert_eq!(app.session_focus, SessionFocus::List);

        let mut app = chat_app("oracle-Demo-1");
        vanish_drop(&mut app);
        let act = handle_key(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert!(
            matches!(act, Action::None),
            "in-grace Tab must be swallowed"
        );
        assert_eq!(app.session_focus, SessionFocus::List);

        // ↑/↓ proves a human is driving the list: the grace ends, Enter works.
        handle_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert!(!app.in_post_drop_grace(), "navigation ends the grace");
        handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(
            app.session_focus,
            SessionFocus::Chat,
            "Enter re-enters chat after nav"
        );
    }

    fn left_click(column: u16) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column,
            row: 5,
            modifiers: KeyModifiers::NONE,
        }
    }

    /// Seed the rendered-geometry cache the way draw_sessions records it on a
    /// 120x40 terminal (25/75 split): hit-testing reads these rects, not a
    /// hardcoded column threshold.
    fn seed_sessions_geometry(app: &mut App) {
        app.sessions_list_area = Some(ratatui::layout::Rect::new(0, 0, 30, 40));
        app.sessions_preview_area = Some(ratatui::layout::Rect::new(30, 0, 90, 40));
        app.sessions_rendered_rows = vec![None, Some(0)]; // header + one entry
        app.sessions_list_fits = true;
    }

    // fix6-T5: a preview-panel click is the mouse twin of the keyboard Enter —
    // it obeys the same grace rule: swallowed with a notice while the grace
    // that followed a vanish-drop is open.
    #[test]
    fn grace_click_swallowed_with_notice() {
        let mut app = chat_app("oracle-Demo-1");
        seed_sessions_geometry(&mut app);
        vanish_drop(&mut app);
        handle_event(&mut app, Event::Mouse(left_click(40)));
        assert_eq!(
            app.session_focus,
            SessionFocus::List,
            "unpinned click must be swallowed"
        );
        assert!(
            app.status_message
                .as_deref()
                .unwrap_or("")
                .contains("ignored"),
            "the swallow must be announced, got {:?}",
            app.status_message
        );
    }

    // TUI-2: a click on a NON-selected list row selects it (no chat entry);
    // a second click on the selected row enters chat. Clicks inside the list
    // never route to the preview, even past the old col-30 threshold.
    #[test]
    fn list_click_selects_row_then_enters_chat() {
        let mut app = chat_app("oracle-Demo-1");
        app.sessions.push(SessionEntry {
            session: OmegaSession::classify("oracle-Demo-2"),
            progress: None,
            is_current: false,
            is_protected: false,
            tree_prefix: String::new(),
        });
        app.session_focus = SessionFocus::List;
        seed_sessions_geometry(&mut app);
        app.sessions_rendered_rows = vec![None, Some(0), Some(1)]; // header + 2 entries
        app.selected = 0;

        // Row 2 (rendered row index 1 → entry 1) is NOT selected: click selects.
        let click_row2 = MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: 10,
            row: 3, // area.y(0) + border(1) + rendered row 2
            modifiers: KeyModifiers::NONE,
        };
        handle_event(&mut app, Event::Mouse(click_row2));
        assert_eq!(app.selected, 1, "click must select the clicked row");
        assert_eq!(
            app.session_focus,
            SessionFocus::List,
            "first click on a new row must NOT enter chat"
        );

        // Same row again (now selected) → mouse twin of Enter: chat focus.
        handle_event(&mut app, Event::Mouse(click_row2));
        assert_eq!(
            app.session_focus,
            SessionFocus::Chat,
            "second click on the selected row enters chat"
        );
    }

    // fix6-T6: scrolling the list moves the selection like keyboard ↑/↓, so it
    // must end the grace the same way.
    #[test]
    fn list_scroll_ends_grace_like_arrow_keys() {
        let mut app = chat_app("oracle-Demo-1");
        vanish_drop(&mut app);
        assert!(app.in_post_drop_grace());
        let scroll = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 5, // over the list, not the preview
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        handle_event(&mut app, Event::Mouse(scroll));
        assert!(!app.in_post_drop_grace(), "scroll must end the grace");
    }

    // fix6-T7: direct tab writers route through leave_tab — F1→Help and the
    // non-Sessions Esc→Sessions jump must clear armed confirms + Tab chord.
    #[test]
    fn f1_and_esc_jump_run_leave_tab_hygiene() {
        // F1 from Menu with an armed confirm.
        let mut app = test_app();
        app.tab = Tab::Menu;
        app.menu_confirm_pending = Some(MenuAction::KillAll);
        handle_key(&mut app, KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE));
        assert_eq!(app.tab, Tab::Help);
        assert_eq!(
            app.menu_confirm_pending, None,
            "F1 must disarm via leave_tab"
        );

        // Esc→Sessions jump from a non-Sessions tab clears the Tab chord.
        let mut app = test_app();
        app.tab = Tab::Projects;
        app.last_tab_press = Some(std::time::Instant::now());
        handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(app.tab, Tab::Sessions);
        assert!(
            app.last_tab_press.is_none(),
            "the Esc jump must run leave_tab hygiene"
        );
    }

    // fix7-T1: Ctrl+L (redraw) and Ctrl+R (restart) are deliberate two-key
    // TUI commands hoisted ABOVE the grace intercept — the deny-by-default
    // swallow must not eat them. Plain chars stay swallowed, and Ctrl combos
    // WITHOUT a CONTROL-guarded command arm stay swallowed too: the match
    // arms test key.code only, so e.g. Ctrl+C / Ctrl+X would false-match the
    // bare 'c' launcher / 'x' kill arms (the fix6-T2 / FIX-F holes).
    #[test]
    fn grace_passes_ctrl_commands_but_swallows_typed_input() {
        let mut app = chat_app("oracle-Demo-1");
        vanish_drop(&mut app);
        assert!(app.in_post_drop_grace(), "precondition: grace is open");

        let act = handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL),
        );
        assert!(
            matches!(act, Action::ForceRedraw),
            "Ctrl+L must redraw during the grace"
        );
        let act = handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL),
        );
        assert!(
            matches!(act, Action::Restart),
            "Ctrl+R must restart during the grace"
        );
        assert!(
            app.in_post_drop_grace(),
            "the pass-through must not end the grace"
        );

        // Plain typed chars are still swallowed…
        let act = handle_key(&mut app, press('c'));
        assert!(matches!(act, Action::None), "plain 'c' must stay swallowed");
        assert!(matches!(app.input_mode, InputMode::Normal));

        // …and so are PTY-bound Ctrl combos: in chat they forward to the
        // agent (Ctrl+C interrupt), so post-drop they are in-flight input.
        let act = handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        );
        assert!(
            matches!(act, Action::None),
            "Ctrl+C must be swallowed in the grace"
        );
        assert!(
            matches!(app.input_mode, InputMode::Normal),
            "Ctrl+C must not open the launcher modal during the grace"
        );
        let act = handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL),
        );
        assert!(
            matches!(act, Action::None),
            "Ctrl+X must not kill the clamped-in selection during the grace"
        );
    }

    #[test]
    fn project_planner_shortcut_emits_the_plan_create_contract() {
        use omega_core::project_manager::ManagedProject;
        let mut app = test_app();
        app.tab = Tab::Projects;
        app.projects_selected = 1; // index 0 is the pinned OS row
        app.project_registry.projects = vec![ManagedProject {
            name: "ContractProject".to_string(),
            path: std::path::PathBuf::from("/tmp/contract-project"),
            telegram_topic_id: None,
            oracle_session: None,
            git_email: None,
            created_at: String::new(),
            telegram: None,
            category: None,
        }];

        let action = handle_key(&mut app, press('p'));
        assert!(matches!(
            action,
            Action::RunPlannerForProject { name, path }
                if name == "ContractProject" && path == "/tmp/contract-project"
        ));
    }

    #[test]
    fn new_project_wizard_only_emits_supported_strategy_ids() {
        let ids: Vec<&str> = crate::app::NEW_PROJECT_STACKS
            .iter()
            .map(|(id, _)| *id)
            .collect();
        assert_eq!(ids, vec!["nextstack", "custom"]);

        let mut app = test_app();
        app.input_mode =
            InputMode::NewProjectStack("demo".to_string(), "side-business".to_string(), 0);
        handle_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(matches!(
            app.input_mode,
            InputMode::NewProjectLaunchPrompt(_, _, ref stack) if stack == "custom"
        ));
    }

    fn os_entry(product: omega_core::os_products::OsProduct) -> omega_core::os_products::OsEntry {
        omega_core::os_products::OsEntry {
            product,
            readiness: omega_core::os_products::OsReadiness {
                level: omega_core::os_products::OsReadinessLevel::Reference,
                directory_present: true,
                master_present: false,
                payload_present: true,
                manifest: omega_core::os_products::OsManifestStatus::Missing,
                runtime_present: false,
                tests_present: false,
                event_schema_status: None,
            },
            path: Some(std::path::PathBuf::from("/tmp")),
            bot_linked: false,
        }
    }

    #[test]
    fn os_tab_obeys_the_two_column_keyboard_contract() {
        let products = omega_core::os_products::OsProduct::all();
        let mut app = test_app();
        app.tab = Tab::Os;
        app.os_entries = vec![os_entry(products[0]), os_entry(products[1])];

        // Moving the list cursor changes the detail document, so it must not
        // retain an offset from the previously selected OS.
        app.detail_scroll = 17;
        handle_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.os_selected, 1);
        assert_eq!(app.detail_scroll, 0, "OS selection must open at the top");
        handle_key(&mut app, KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.os_selected, 0);

        app.detail_max_scroll = 100;

        handle_key(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert!(app.detail_focused, "Tab must focus the OS detail panel");

        let selected = app.os_selected;
        handle_key(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(
            app.os_selected, selected,
            "detail arrows must not move the OS list"
        );
        assert_eq!(
            app.detail_scroll, 1,
            "detail arrows must scroll the OS detail"
        );

        handle_key(&mut app, KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
        assert_eq!(app.detail_scroll, 100);
        handle_key(&mut app, KeyEvent::new(KeyCode::Home, KeyModifiers::NONE));
        assert_eq!(app.detail_scroll, 0);

        // A rapid second Tab uses the same fullscreen contract as Settings,
        // Projects, and System.
        app.last_tab_press = Some(std::time::Instant::now());
        handle_key(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert!(app.detail_fullscreen);
    }

    #[test]
    fn os_enter_focuses_detail_before_opening_the_master_prompt() {
        let mut app = test_app();
        app.tab = Tab::Os;
        app.os_entries = vec![os_entry(omega_core::os_products::OsProduct::all()[0])];

        let first = handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(matches!(first, Action::None));
        assert!(app.detail_focused);

        let second = handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(matches!(second, Action::OpenOsSession { .. }));
    }

    #[test]
    fn every_modal_swallows_mouse_events_before_the_underlay_can_change() {
        let modes = vec![
            InputMode::NewNamedSession("codex".into()),
            InputMode::NewSessionPromptDirect("s".into(), "codex".into()),
            InputMode::DispatchProject(vec!["p".into()], 0),
            InputMode::ProjectOpenLane("p".into(), "/tmp".into(), 0),
            InputMode::ProjectOpenAgentPick {
                lane: ProjectLane::Coding,
                name: "p".into(),
                path: "/tmp".into(),
                agents: vec![omega_core::agents::Agent::Codex],
                sel: 0,
            },
            InputMode::ProjectDelete("p".into(), 0),
            InputMode::DispatchMission("p".into()),
            InputMode::RenameSession("s".into()),
            InputMode::SessionFilter,
            InputMode::NewProjectName,
            InputMode::NewProjectCategory("p".into(), 0),
            InputMode::NewProjectCredGroup("p".into(), "customer".into()),
            InputMode::NewProjectStack("p".into(), "tools".into(), 0),
            InputMode::NewProjectLaunchPrompt("p".into(), "tools".into(), "custom".into()),
            InputMode::NewProjectLaunchDocs("p".into(), "tools".into(), "custom".into(), None),
            InputMode::ProvisioningSetup {
                step: 0,
                collected: Vec::new(),
            },
            InputMode::TelegramSetupToken,
            InputMode::TelegramSetupChatId("token".into()),
            InputMode::TelegramSetupUserId("token".into(), "chat".into()),
            InputMode::EditSettingsField {
                config_key: "general.theme".into(),
                masked: false,
            },
            InputMode::SelectModel("model".into(), vec!["one".into()], 0),
            InputMode::GroupSetupId,
            InputMode::AddProjectPath,
            InputMode::ReauthCode,
        ];
        let mouse = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 1,
            row: 1,
            modifiers: KeyModifiers::NONE,
        };

        for mode in modes {
            let mut app = test_app();
            app.tab = Tab::Projects;
            app.detail_focused = true;
            app.detail_max_scroll = 50;
            app.detail_scroll = 7;
            app.input_mode = mode.clone();
            let action = handle_event(&mut app, Event::Mouse(mouse));
            assert!(
                matches!(action, Action::None),
                "modal {:?} must swallow mouse",
                mode
            );
            assert_eq!(app.detail_scroll, 7, "modal {:?} leaked wheel scroll", mode);
        }
    }
}
