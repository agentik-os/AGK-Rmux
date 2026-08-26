use crate::app::{
    App, InfoSection, InputMode, MenuAction, MonitorAction, MonitorSection, SessionEntry,
    SessionFocus, SessionRow, SettingsSection, Tab,
};
use crate::preview::{
    provider as preview_provider, reflow_cursor as reflowed_cursor,
    reflow_lines as reflow_preview_lines,
};
use omega_core::done::DoneStatus;
use omega_core::session::{PreviewColor, PreviewSpan, SessionRole};

/// Map a preview color to a ratatui Color, PRESERVING depth so the emitted
/// escape matches the terminal's capability. ANSI 0–15 → the 16-color named
/// variants (emit `3x`/`9x`, render on any color terminal incl. over mosh);
/// 16–255 → 256-indexed; true RGB → 24-bit. (Forcing everything to RGB made
/// Claude's 16-color stream render as grey truecolor on non-truecolor terminals.)
fn preview_to_color(c: PreviewColor) -> Color {
    match c {
        PreviewColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
        PreviewColor::Indexed(i) => match i {
            0 => Color::Reset,
            1 => Color::Red,
            2 => Color::Green,
            3 => Color::Yellow,
            4 => Color::Blue,
            5 => Color::Magenta,
            6 => Color::Cyan,
            7 => Color::Gray,
            8 => Color::DarkGray,
            9 => Color::LightRed,
            10 => Color::LightGreen,
            11 => Color::LightYellow,
            12 => Color::LightBlue,
            13 => Color::LightMagenta,
            14 => Color::LightCyan,
            // 0 (black) and 15 (white) → Reset (the terminal's own fg) instead of
            // a fixed black/white, which goes invisible when the theme's bg is the
            // same shade. This is the mirror passthrough for Claude's own output —
            // critical now that Claude renders to the normal screen (the whole
            // conversation, incl. the user's own messages, flows through here).
            15 => Color::Reset,
            n => Color::Indexed(n),
        },
    }
}

/// Foreground mapping for a span that carries an EXPLICIT background. The
/// 0/15 → Reset safety net above exists so black/white text on the DEFAULT
/// background can't vanish into a same-shade theme canvas — but on a span
/// with its own bg it backfires: the REVERSE fallback in omega-core
/// (`styled_rows_from_snapshot`) synthesizes black-on-gray precisely so the
/// swap stays visible, and 0→Reset turned that into default-fg-on-gray
/// (near-white on light gray on dark terminals — selections unreadable).
/// With an explicit bg the invisibility risk is gone, so black/white stay
/// literal; everything else keeps the depth-preserving passthrough.
fn preview_fg_color(c: PreviewColor, has_explicit_bg: bool) -> Color {
    match c {
        PreviewColor::Indexed(0) if has_explicit_bg => Color::Black,
        PreviewColor::Indexed(15) if has_explicit_bg => Color::White,
        other => preview_to_color(other),
    }
}

/// Return the WCAG contrast ratio for two captured terminal colours.  The
/// source terminal may use ANSI names or the xterm palette, so this is an
/// intentionally conservative approximation used only to reject unsafe
/// foregrounds, never to recolour readable output.
fn preview_contrast_ratio(fg: PreviewColor, bg: PreviewColor) -> f64 {
    let relative_luminance = |color: PreviewColor| {
        let (r, g, b) = preview_rgb(color);
        let linear = |channel: u8| {
            let value = f64::from(channel) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
    };
    let fg_l = relative_luminance(fg);
    let bg_l = relative_luminance(bg);
    (fg_l.max(bg_l) + 0.05) / (fg_l.min(bg_l) + 0.05)
}

/// Pick a literal terminal-safe foreground when a nested TUI supplied a
/// colour that is too close to its own background.  This fixes black-on-dark
/// and white-on-light spans as well as arbitrary RGB pairs, while preserving
/// the original colour whenever it already meets AA contrast.
fn contrast_safe_preview_fg(fg: PreviewColor, bg: PreviewColor) -> Color {
    if preview_contrast_ratio(fg, bg) >= 4.5 {
        return preview_fg_color(fg, true);
    }
    let (black, white) = (
        preview_contrast_ratio(PreviewColor::Indexed(0), bg),
        preview_contrast_ratio(PreviewColor::Indexed(15), bg),
    );
    if black >= white {
        Color::Black
    } else {
        Color::White
    }
}

/// Backgrounds are always explicit. Keep ANSI black/white literal instead of
/// applying the foreground safety mapping in `preview_to_color`.
fn preview_bg_color(c: PreviewColor) -> Color {
    match c {
        PreviewColor::Indexed(0) => Color::Black,
        PreviewColor::Indexed(15) => Color::White,
        other => preview_to_color(other),
    }
}

/// Resolve the source terminal's default foreground when a mirrored span has
/// an explicit background. `Color::Reset` would resolve against the OUTER
/// terminal instead: a light outer theme supplies a dark default foreground,
/// which made Codex's dark composer (`bg=rgb(30,30,30), fg=default`) render as
/// an unreadable black band. Pick whichever of black/white has more contrast
/// with the source background so nested TUIs remain readable on either theme.
fn preview_default_fg_for_bg(bg: PreviewColor) -> Color {
    let (r, g, b) = preview_rgb(bg);
    let linear = |channel: u8| {
        let value = f64::from(channel) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    let luminance = 0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b);

    // Black and white have equal WCAG contrast at relative luminance ~0.179.
    if luminance > 0.179 {
        Color::Black
    } else {
        Color::White
    }
}

/// Approximate a preview color as RGB for contrast selection. RGB and the
/// xterm 256-color cube are exact; ANSI 0-15 use the conventional palette
/// because their real values are owned by the outer terminal and unavailable
/// in an rmux snapshot.
fn preview_rgb(color: PreviewColor) -> (u8, u8, u8) {
    match color {
        PreviewColor::Rgb(r, g, b) => (r, g, b),
        PreviewColor::Indexed(index @ 0..=15) => {
            const ANSI: [(u8, u8, u8); 16] = [
                (0, 0, 0),
                (128, 0, 0),
                (0, 128, 0),
                (128, 128, 0),
                (0, 0, 128),
                (128, 0, 128),
                (0, 128, 128),
                (192, 192, 192),
                (128, 128, 128),
                (255, 0, 0),
                (0, 255, 0),
                (255, 255, 0),
                (0, 0, 255),
                (255, 0, 255),
                (0, 255, 255),
                (255, 255, 255),
            ];
            ANSI[usize::from(index)]
        }
        PreviewColor::Indexed(index @ 16..=231) => {
            const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
            let cube = index - 16;
            (
                LEVELS[usize::from(cube / 36)],
                LEVELS[usize::from((cube % 36) / 6)],
                LEVELS[usize::from(cube % 6)],
            )
        }
        PreviewColor::Indexed(index) => {
            let gray = 8 + (index - 232) * 10;
            (gray, gray, gray)
        }
    }
}

/// Convert one captured rmux span into its ratatui style. Kept as a helper so
/// the default-foreground/explicit-background contract has direct regression
/// tests instead of depending on a full live daemon fixture.
fn preview_span_style(sp: &PreviewSpan) -> Style {
    let mut style = Style::default();
    if let Some(c) = sp.fg {
        style = style.fg(match sp.bg {
            Some(bg) => contrast_safe_preview_fg(c, bg),
            None => preview_fg_color(c, false),
        });
    } else if let Some(bg) = sp.bg {
        style = style.fg(preview_default_fg_for_bg(bg));
    }
    if let Some(c) = sp.bg {
        style = style.bg(preview_bg_color(c));
    }
    if sp.bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    // DIM reduces perceived luminance. Keep it only when the already-selected
    // foreground has a generous contrast margin; this prevents dim ANSI text
    // from disappearing on light Termius palettes or bright basic terminals.
    let dim_safe = match (sp.fg, sp.bg) {
        (Some(fg), Some(bg)) => preview_contrast_ratio(fg, bg) >= 7.0,
        (None, Some(bg)) => {
            preview_contrast_ratio(PreviewColor::Indexed(15), bg) >= 7.0
                || preview_contrast_ratio(PreviewColor::Indexed(0), bg) >= 7.0
        }
        _ => true,
    };
    if sp.dim && dim_safe {
        style = style.add_modifier(Modifier::DIM);
    }
    if sp.italic {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if sp.underline {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    style
}

/// Add semantic emphasis without throwing away the nested terminal's ANSI
/// foreground/background/modifiers. The previous row classifiers rebuilt
/// styles from `Style::default()`, so a highlighted task or prompt could lose
/// its provider-owned colors and, for composer rows, its contrast background.
fn emphasized_preview_row(
    row: &[PreviewSpan],
    fallback_fg: Option<Color>,
    modifier: Option<Modifier>,
) -> Line<'static> {
    Line::from(
        row.iter()
            .map(|span| {
                let mut style = preview_span_style(span);
                if span.fg.is_none() {
                    if let Some(color) = fallback_fg {
                        style = style.fg(color);
                    }
                }
                if let Some(modifier) = modifier {
                    style = style.add_modifier(modifier);
                }
                Span::styled(span.text.clone(), style)
            })
            .collect::<Vec<_>>(),
    )
}

#[cfg(test)]
mod preview_style_tests {
    use super::*;

    fn span(fg: Option<PreviewColor>, bg: Option<PreviewColor>, dim: bool) -> PreviewSpan {
        PreviewSpan {
            text: "Summarize recent commits".to_string(),
            fg,
            bg,
            bold: false,
            dim,
            italic: false,
            underline: false,
        }
    }

    #[test]
    fn codex_dark_composer_gets_an_explicit_light_foreground() {
        // Codex 0.145 emits this exact combination after rmux reports its
        // white-on-black fallback palette: RGB(30,30,30), default fg, DIM.
        let style = preview_span_style(&span(None, Some(PreviewColor::Rgb(30, 30, 30)), true));

        assert_eq!(style.fg, Some(Color::White));
        assert_eq!(style.bg, Some(Color::Rgb(30, 30, 30)));
        assert!(style.add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn light_explicit_background_gets_a_dark_foreground() {
        let style = preview_span_style(&span(None, Some(PreviewColor::Rgb(245, 245, 245)), false));

        assert_eq!(style.fg, Some(Color::Black));
        assert_eq!(style.bg, Some(Color::Rgb(245, 245, 245)));
    }

    #[test]
    fn ansi_black_and_white_backgrounds_remain_explicit_and_contrasted() {
        let dark = preview_span_style(&span(None, Some(PreviewColor::Indexed(0)), false));
        let light = preview_span_style(&span(None, Some(PreviewColor::Indexed(15)), false));

        assert_eq!((dark.fg, dark.bg), (Some(Color::White), Some(Color::Black)));
        assert_eq!(
            (light.fg, light.bg),
            (Some(Color::Black), Some(Color::White))
        );
    }

    #[test]
    fn explicit_foreground_is_preserved_over_an_explicit_background() {
        let style = preview_span_style(&span(
            Some(PreviewColor::Rgb(195, 147, 255)),
            Some(PreviewColor::Rgb(30, 30, 30)),
            false,
        ));

        assert_eq!(style.fg, Some(Color::Rgb(195, 147, 255)));
        assert_eq!(style.bg, Some(Color::Rgb(30, 30, 30)));
    }

    #[test]
    fn unsafe_explicit_foreground_is_replaced_by_a_contrasting_one() {
        // Codex can emit a literal black foreground while its composer owns a
        // dark RGB surface. Never pass that pair through to a light or dark
        // outer terminal where the text becomes invisible.
        let dark = preview_span_style(&span(
            Some(PreviewColor::Rgb(0, 0, 0)),
            Some(PreviewColor::Rgb(30, 30, 30)),
            false,
        ));
        assert_eq!(dark.fg, Some(Color::White));

        let light = preview_span_style(&span(
            Some(PreviewColor::Rgb(255, 255, 255)),
            Some(PreviewColor::Rgb(245, 245, 245)),
            false,
        ));
        assert_eq!(light.fg, Some(Color::Black));
    }

    #[test]
    fn dim_is_removed_when_it_would_erase_a_low_margin_foreground() {
        let style = preview_span_style(&span(
            Some(PreviewColor::Rgb(90, 90, 90)),
            Some(PreviewColor::Rgb(30, 30, 30)),
            true,
        ));
        assert!(!style.add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn terminal_defaults_stay_unstyled_without_an_explicit_background() {
        let style = preview_span_style(&span(None, None, false));

        assert_eq!(style.fg, None);
        assert_eq!(style.bg, None);
    }

    #[test]
    fn semantic_emphasis_preserves_provider_ansi_and_background() {
        let row = vec![PreviewSpan {
            text: "› typed input".to_string(),
            fg: Some(PreviewColor::Rgb(195, 147, 255)),
            bg: Some(PreviewColor::Rgb(30, 30, 30)),
            bold: false,
            dim: false,
            italic: true,
            underline: true,
        }];

        let line = emphasized_preview_row(&row, Some(Color::Green), Some(Modifier::BOLD));
        let style = line.spans[0].style;
        assert_eq!(style.fg, Some(Color::Rgb(195, 147, 255)));
        assert_eq!(style.bg, Some(Color::Rgb(30, 30, 30)));
        assert!(style.add_modifier.contains(Modifier::BOLD));
        assert!(style.add_modifier.contains(Modifier::ITALIC));
        assert!(style.add_modifier.contains(Modifier::UNDERLINED));
    }
}

/// Re-style a rendered line so display columns `[from, to)` carry REVERSED —
/// the mouse drag-selection highlight. Splits spans at the boundaries and
/// counts DISPLAY width (emoji/CJK = 2 cells) so the highlight tracks the
/// pointer on wide glyphs.
fn reverse_cols(line: &Line<'_>, from: usize, to: usize) -> Line<'static> {
    use unicode_width::UnicodeWidthChar;
    let mut out: Vec<Span<'static>> = Vec::with_capacity(line.spans.len() + 2);
    let mut col = 0usize;
    for sp in &line.spans {
        let mut seg = String::new();
        let mut seg_rev: Option<bool> = None;
        for ch in sp.content.chars() {
            let rev = col >= from && col < to;
            if seg_rev != Some(rev) {
                if let (Some(prev), false) = (seg_rev, seg.is_empty()) {
                    let style = if prev {
                        sp.style.add_modifier(Modifier::REVERSED)
                    } else {
                        sp.style
                    };
                    out.push(Span::styled(std::mem::take(&mut seg), style));
                }
                seg_rev = Some(rev);
            }
            seg.push(ch);
            col += ch.width().unwrap_or(0);
        }
        if let (Some(prev), false) = (seg_rev, seg.is_empty()) {
            let style = if prev {
                sp.style.add_modifier(Modifier::REVERSED)
            } else {
                sp.style
            };
            out.push(Span::styled(seg, style));
        }
    }
    Line::from(out)
}
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame,
};

// All TUI-chrome colors are semantic theme roles (Settings -> Theme). Only the
// `preview_to_color` passthrough above keeps raw terminal colors -- that is
// the agent's own pane content, never re-skinned.
use crate::theme as th;

// ── Render-path I/O memo ────────────────────────────────────────────────────
// draw() runs at 15-60 FPS (TICK_ACTIVE 16ms / TICK_IDLE 66ms in main.rs), so
// anything a renderer reads from the OS multiplies by the frame rate: the
// status bar's SystemStats::read() forks a `df` subprocess, and the
// monitor/project detail renderers re-read + re-parse JSON/TOML files — per
// frame, per TUI. (The run-loop comment records that 5 idle menus once
// saturated a 2-core VPS.) These memos hold each read for RENDER_TTL; status
// surfaces tolerate a couple seconds of staleness invisibly. Same medicine as
// `App::providers_cache`, generalized for the renderers that take no `App`.
const RENDER_TTL: std::time::Duration = std::time::Duration::from_secs(2);
type UsageSnapshotRead = Result<Option<omega_core::monitor::UsageSnapshot>, String>;
type TelegramConfigRead = Result<Option<omega_core::monitor::OmegaTelegramConfig>, String>;
type TelegramGroupRead = Result<Option<omega_core::telegram_group::TelegramGroupConfig>, String>;

/// True when an ancestor of this process is mosh-server — the transport that
/// silently eats the mouse handshake (mobile-shell/mosh#101). Walked once and
/// cached for the process lifetime: ancestry can't change after spawn.
fn under_mosh() -> bool {
    static UNDER: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *UNDER.get_or_init(|| {
        let mut pid = std::process::id();
        for _ in 0..16 {
            let Ok(stat) = std::fs::read_to_string(format!("/proc/{}/stat", pid)) else {
                return false;
            };
            // /proc/<pid>/stat: "pid (comm) state ppid …" — comm may contain
            // spaces/parens, so split on the LAST ')'.
            let Some(rest) = stat.rsplit_once(')').map(|(_, r)| r) else {
                return false;
            };
            let comm = stat
                .split_once('(')
                .and_then(|(_, r)| r.rsplit_once(')').map(|(c, _)| c))
                .unwrap_or("");
            if comm.contains("mosh-server") {
                return true;
            }
            let Some(ppid) = rest
                .split_whitespace()
                .nth(1)
                .and_then(|p| p.parse::<u32>().ok())
            else {
                return false;
            };
            if ppid <= 1 {
                return false;
            }
            pid = ppid;
        }
        false
    })
}

fn render_memo<T: Clone + 'static>(
    slot: &'static std::thread::LocalKey<std::cell::RefCell<Option<(std::time::Instant, T)>>>,
    load: impl FnOnce() -> T,
) -> T {
    slot.with(|cell| {
        let mut cached = cell.borrow_mut();
        if let Some((at, v)) = cached.as_ref() {
            if at.elapsed() < RENDER_TTL {
                return v.clone();
            }
        }
        let v = load();
        *cached = Some((std::time::Instant::now(), v.clone()));
        v
    })
}

/// Global breathing room around the whole UI — some terminals render cell
/// (0,0) flush against the window edge, which looks cramped. The theme
/// background still paints the FULL frame, so the margin shows the theme's
/// canvas (or the terminal's own bg for Omega/Transparent). Skipped on tiny
/// terminals where every row counts.
fn padded(area: Rect) -> Rect {
    if area.width < 70 || area.height < 20 {
        return area;
    }
    area.inner(ratatui::layout::Margin {
        horizontal: 2,
        vertical: 1,
    })
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    // Theme background: paint the whole frame first so every widget sits on
    // the active theme's canvas (None = keep the terminal's own background).
    // This is what makes themes visually distinct at a glance -- accent
    // colors alone read as near-identical between palettes.
    // Skippable via Settings → Theme → "Theme background": OFF keeps the
    // terminal's own background (transparency / background image) visible.
    if app.config.theme_background {
        if let Some(bg) = th::bg() {
            // fg too: spans rendered without an explicit fg inherit this themed
            // text color instead of the terminal default (which may be unreadable
            // on the painted background -- e.g. white-on-white for Paper).
            frame.render_widget(
                Block::default().style(Style::default().fg(th::text()).bg(bg)),
                frame.area(),
            );
        }
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(padded(frame.area()));

    draw_tabs(frame, app, chunks[0]);

    match app.tab {
        Tab::Sessions => draw_sessions(frame, app, chunks[1]),
        Tab::Menu => draw_menu(frame, app, chunks[1]),
        Tab::Projects => draw_projects(frame, app, chunks[1]),
        Tab::System => draw_system(frame, app, chunks[1]),
        Tab::Settings => draw_settings(frame, app, chunks[1]),
        Tab::Os => draw_os(frame, app, chunks[1]),
        Tab::Help => draw_help(frame, app, chunks[1]),
    }

    draw_status_bar(frame, app, chunks[2]);

    // Overlay modals — drawn LAST so they paint on top of everything
    match app.input_mode {
        InputMode::TelegramSetupToken
        | InputMode::TelegramSetupChatId(_)
        | InputMode::TelegramSetupUserId(_, _) => {
            draw_telegram_setup_modal(frame, app);
        }
        InputMode::RenameSession(_) => draw_simple_input_modal(
            frame,
            app,
            "Rename session",
            "New session name (Enter to confirm, Esc to cancel)",
            false,
        ),
        InputMode::NewNamedSession(ref agent) => {
            let title = format!("New {} session", agent);
            let hint = "Session name (Enter to launch, Esc to cancel)".to_string();
            draw_simple_input_modal_owned(frame, app, &title, &hint, false);
        }
        InputMode::DispatchProject(..) => draw_dispatch_picker(frame, app),
        InputMode::DispatchMission(ref p) => {
            let title = "Dispatch oracle — step 2/2".to_string();
            let hint = format!(
                "Mission for project '{}' (Enter to dispatch, Esc to cancel)",
                p
            );
            draw_simple_input_modal_owned(frame, app, &title, &hint, false);
        }
        InputMode::EditSettingsField {
            ref config_key,
            masked,
        } => {
            let title = format!("Edit setting: {}", config_key);
            let hint = "Type new value, Enter to save, Esc to cancel".to_string();
            draw_simple_input_modal_owned(frame, app, &title, &hint, masked);
        }
        InputMode::NewProjectName => draw_simple_input_modal(
            frame,
            app,
            "New project — step 1/3",
            "Project name (Enter to continue, Esc to cancel)",
            false,
        ),
        InputMode::NewProjectCategory(..) | InputMode::NewProjectStack(..) => {
            draw_new_project_picker(frame, app);
        }
        InputMode::SelectModel(..) => {
            draw_model_picker(frame, app);
        }
        InputMode::ProjectOpenLane(..) | InputMode::ProjectOpenAgentPick { .. } => {
            draw_project_open_picker(frame, app);
        }
        InputMode::ProjectDelete(..) => {
            draw_project_delete_picker(frame, app);
        }
        InputMode::NewProjectCredGroup(..) => {
            let groups = omega_core::provisioning::list_groups().join(", ");
            draw_simple_input_modal(
                frame,
                app,
                "Client credentials — group",
                &format!(
                    "Existing: {}  —  type one to reuse, or a NEW client name (Enter; Esc = default)",
                    groups
                ),
                false,
            );
        }
        InputMode::NewProjectLaunchPrompt(..) => draw_simple_input_modal(
            frame,
            app,
            "New project — kickoff (optional)",
            "Describe the idea / requirements (Enter to continue, Esc to skip)",
            false,
        ),
        InputMode::NewProjectLaunchDocs(..) => draw_simple_input_modal(
            frame,
            app,
            "New project — docs (optional)",
            "Comma-separated doc paths to seed the project (Enter to launch, Esc to skip)",
            false,
        ),
        InputMode::ProvisioningSetup { step, .. } => {
            let fields = crate::app::PROVISIONING_FIELDS;
            let (key, hint, masked) = fields
                .get(step)
                .map(|f| (f.0, f.1, f.2))
                .unwrap_or(("", "", true));
            let title = format!("Provisioning keys — {}/{}: {}", step + 1, fields.len(), key);
            draw_simple_input_modal_owned(frame, app, &title, hint, masked);
        }
        InputMode::GroupSetupId => draw_simple_input_modal(
            frame,
            app,
            "Telegram project group",
            "Supergroup id (negative, e.g. -1001234567890) — Enter to save, Esc to cancel",
            false,
        ),
        InputMode::AddProjectPath => draw_simple_input_modal(
            frame,
            app,
            "Add project — register an existing folder",
            "Absolute path to the project folder (Enter to register, Esc to cancel)",
            false,
        ),
        InputMode::ReauthCode => draw_simple_input_modal(
            frame,
            app,
            "Claude re-login — paste authorize code",
            "Paste the code from your browser (Enter to submit, Esc to cancel)",
            false,
        ),
        _ => {}
    }
}

/// Overlay picker for new-project wizard steps 2 (category) and 3 (stack).
/// Reads the option list + selection index straight from the InputMode variant.
fn draw_new_project_picker(frame: &mut Frame, app: &App) {
    let (title, options, sel): (String, &[(&str, &str)], usize) = match &app.input_mode {
        InputMode::NewProjectCategory(name, sel) => (
            format!(
                " New project [{}] — step 2/3: category — ↑/↓, Enter, Esc ",
                name
            ),
            crate::app::NEW_PROJECT_CATEGORIES,
            *sel,
        ),
        InputMode::NewProjectStack(name, _category, sel) => (
            format!(
                " New project [{}] — step 3/3: stack — ↑/↓, Enter, Esc ",
                name
            ),
            crate::app::NEW_PROJECT_STACKS,
            *sel,
        ),
        _ => return,
    };

    let area = centered_rect(60, 50, frame.area());
    frame.render_widget(Clear, area);

    let inner_w = area.width.saturating_sub(2) as usize;
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, (id, label))| {
            let selected = i == sel;
            if selected {
                // Full-width bar — consistent with the Select overlay.
                let row = format!(" ▶ {:10} {}", id, label);
                let pad = inner_w.saturating_sub(row.chars().count());
                ListItem::new(Line::from(Span::styled(
                    format!("{}{}", row, " ".repeat(pad)),
                    Style::default()
                        .fg(th::sel_fg())
                        .bg(th::accent())
                        .add_modifier(Modifier::BOLD),
                )))
            } else {
                ListItem::new(Line::from(vec![
                    Span::raw("   "),
                    Span::styled(
                        format!(" {:10} ", id),
                        Style::default()
                            .fg(th::accent2())
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(*label, Style::default().fg(th::text())),
                ]))
            }
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(th::accent()))
            .style(Style::default().fg(th::text()).bg(modal_bg(app))),
    );
    frame.render_widget(list, area);
}

/// Modal canvas color: the theme background, unless the user turned theme
/// backgrounds off (Settings → Theme) — then fall back to the terminal's own.
fn modal_bg(app: &App) -> Color {
    if app.config.theme_background {
        th::bg().unwrap_or(Color::Reset)
    } else {
        Color::Reset
    }
}

/// Arrow-key overlay to pick a model from a fixed list (NO typing). Mirrors
/// `draw_new_project_picker`. Reads the option list + selection straight from
/// the `SelectModel` InputMode variant.
fn draw_model_picker(frame: &mut Frame, app: &App) {
    let (config_key, options, sel): (&str, &[String], usize) = match &app.input_mode {
        InputMode::SelectModel(key, opts, sel) => (key.as_str(), opts.as_slice(), *sel),
        _ => return,
    };

    // Theme selector rows show the human label, not the raw slug.
    let is_theme = config_key == "general.theme";
    // Wider modal for themes: label + blurb need the room.
    let area = centered_rect(if is_theme { 70 } else { 50 }, 50, frame.area());
    frame.render_widget(Clear, area);

    let inner_w = area.width.saturating_sub(2) as usize;

    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let selected = i == sel;
            let text = if is_theme {
                match crate::theme::ThemeId::from_slug(opt) {
                    Some(id) => format!("{:26}{}", id.label(), id.blurb()),
                    None => opt.clone(),
                }
            } else {
                opt.clone()
            };
            // Pad to the full inner width: the highlight reads as a SOLID
            // selection bar across the modal (the old text-chip highlight was
            // easy to miss on several themes).
            let row = format!(" {} {}", if selected { "▶" } else { " " }, text);
            // Char-safe ellipsis instead of a hard mid-word clip at the
            // border (slicing by byte would also panic on multi-byte chars).
            let row = if row.chars().count() > inner_w && inner_w > 1 {
                format!("{}…", row.chars().take(inner_w - 1).collect::<String>())
            } else {
                row
            };
            let pad = inner_w.saturating_sub(row.chars().count());
            let row = format!("{}{}", row, " ".repeat(pad));
            let style = if selected {
                Style::default()
                    .fg(th::sel_fg())
                    .bg(th::accent())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(th::text())
            };
            ListItem::new(Line::from(Span::styled(row, style)))
        })
        .collect();

    let title = if is_theme {
        " Theme — ↑/↓ live preview, Enter saves, Esc reverts ".to_string()
    } else {
        format!(" Select {} — ↑/↓, Enter, Esc ", config_key)
    };
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(th::accent()))
            .style(Style::default().fg(th::text()).bg(modal_bg(app))),
    );
    // Scroll-follow: stateful render keeps the selected row visible when the
    // list is taller than the modal (15 themes vs ~10 visible rows at 80x24 —
    // arrowing below the fold was blind). Selection visuals stay baked into
    // the items; the state only drives the offset window.
    let mut state = ListState::default();
    state.select(Some(sel));
    frame.render_stateful_widget(list, area, &mut state);
}

/// Open-project picker — two steps. Step 1: the LANE (Coding session /
/// Marketing session / Oracle). Step 2: the LLM agent, listing only agents
/// actually INSTALLED on this machine (claude / codex / gemini / kimi / …).
/// ↑/↓ or digits, Enter, Esc (step 2 Esc goes back to step 1).
fn draw_project_open_picker(frame: &mut Frame, app: &App) {
    let (title, options, sel): (String, Vec<String>, usize) = match &app.input_mode {
        InputMode::ProjectOpenLane(name, _path, sel) => (
            format!(" ▶ Open {} — what do you want to work on? ", name),
            vec![
                "1. Coding — new session in the project".to_string(),
                "2. Marketing — marketing machine + dedicated agent (project/marketing/)"
                    .to_string(),
                "3. Oracle — the project's own orchestrator (asks for a mission)".to_string(),
                "   Cancel".to_string(),
            ],
            *sel,
        ),
        InputMode::ProjectOpenAgentPick {
            lane,
            name,
            agents,
            sel,
            ..
        } => {
            let lane_label = match lane {
                crate::app::ProjectLane::Coding => "coding",
                crate::app::ProjectLane::Marketing => "marketing",
            };
            let mut rows: Vec<String> = agents
                .iter()
                .enumerate()
                .map(|(i, a)| format!("{}. {}", i + 1, a.display_name()))
                .collect();
            rows.push("   Cancel".to_string());
            (
                format!(
                    " ▶ Open {} ({}) — pick the LLM (installed only) ",
                    name, lane_label
                ),
                rows,
                *sel,
            )
        }
        _ => return,
    };
    let height = (options.len() as u16 + 4).max(8).min(frame.area().height);
    let area = centered_rect_abs(74, height, frame.area());
    frame.render_widget(Clear, area);
    let inner_w = area.width.saturating_sub(2) as usize;
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let style = if i == sel {
                Style::default()
                    .fg(th::sel_fg())
                    .bg(th::accent())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(th::bright())
            };
            let row = format!("{} {}", if i == sel { "▶" } else { " " }, opt);
            let pad = if i == sel {
                inner_w.saturating_sub(row.chars().count())
            } else {
                0
            };
            ListItem::new(Line::from(Span::styled(
                format!("{}{}", row, " ".repeat(pad)),
                style,
            )))
        })
        .collect();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(th::accent())),
    );
    frame.render_widget(list, area);
}

/// Project delete menu — the SAME three escalating tiers as the Telegram bot
/// (omega → local → all), as a visible arrow-key overlay. ↑/↓ or 1/2/3, Enter,
/// Esc. Destructive → red border, explicit consequences per tier.
fn draw_project_delete_picker(frame: &mut Frame, app: &App) {
    let (name, sel): (&str, usize) = match &app.input_mode {
        InputMode::ProjectDelete(name, sel) => (name.as_str(), *sel),
        _ => return,
    };
    let options = [
        "1. Remove from OmegaOS — topic + agent-bot + registry (folder & GitHub kept)",
        "2. Delete local machine — that + kill oracle + DELETE the local folder (GitHub kept)",
        "3. Delete ALL (+ GitHub) — that + DELETE the GitHub repo (nothing remains)",
        "   Cancel",
    ];
    let area = centered_rect(80, 30, frame.area());
    frame.render_widget(Clear, area);
    let inner_w = area.width.saturating_sub(2) as usize;
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let danger = matches!(i, 1 | 2);
            let style = if i == sel {
                Style::default()
                    .fg(th::sel_fg())
                    .bg(if danger { th::error() } else { th::accent() })
                    .add_modifier(Modifier::BOLD)
            } else if danger {
                Style::default().fg(th::error())
            } else {
                Style::default().fg(th::bright())
            };
            // Full-width bar when selected — consistent with the other pickers.
            let row = format!("{} {}", if i == sel { "▶" } else { " " }, opt);
            let pad = if i == sel {
                inner_w.saturating_sub(row.chars().count())
            } else {
                0
            };
            ListItem::new(Line::from(Span::styled(
                format!("{}{}", row, " ".repeat(pad)),
                style,
            )))
        })
        .collect();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" 🗑 Delete {} — ↑/↓ or 1/2/3, Enter, Esc ", name))
            .border_style(Style::default().fg(th::error())),
    );
    frame.render_widget(list, area);
}

/// Dispatch oracle — step 1: project picker overlay (no typing). Lists the
/// added projects from the shared ProjectRegistry; ↑/↓ move, Enter selects,
/// Esc cancels. Mirrors `draw_model_picker`.
fn draw_dispatch_picker(frame: &mut Frame, app: &App) {
    let (projects, sel): (&[String], usize) = match &app.input_mode {
        InputMode::DispatchProject(projects, sel) => (projects.as_slice(), *sel),
        _ => return,
    };

    let area = centered_rect(50, 50, frame.area());
    frame.render_widget(Clear, area);

    let items: Vec<ListItem> = projects
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let selected = i == sel;
            let prefix = if selected { "▶ " } else { "  " };
            let style = if selected {
                Style::default()
                    .fg(th::sel_fg())
                    .bg(th::accent())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(th::text())
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(th::accent())),
                Span::styled(format!(" 🚀 {} ", p), style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Dispatch oracle — step 1/2: pick a project — ↑/↓, Enter, Esc ")
            .border_style(Style::default().fg(th::accent())),
    );
    frame.render_widget(list, area);
}

/// Centered overlay modal for the 3-step Telegram setup wizard.
fn draw_telegram_setup_modal(frame: &mut Frame, app: &App) {
    let (step_num, step_label, hint, masked, value): (u8, &str, String, bool, String) = match &app.input_mode {
        InputMode::TelegramSetupToken => (
            1,
            "BOT_TOKEN",
            "Paste the bot token from @BotFather. Bracketed paste handles long tokens — no need to type.".to_string(),
            true,
            app.input_buffer.clone(),
        ),
        InputMode::TelegramSetupChatId(_) => (
            2,
            "CHAT_ID",
            "Numeric chat id. Get yours by sending /start to @userinfobot on Telegram.".to_string(),
            false,
            app.input_buffer.clone(),
        ),
        InputMode::TelegramSetupUserId(_, chat) => (
            3,
            "ALLOWED user_ids (optional)",
            format!(
                "Comma-separated user_ids allowed to talk to the bot (chat_id={}). Esc to skip.",
                chat
            ),
            false,
            app.input_buffer.clone(),
        ),
        _ => return,
    };

    let area = centered_rect(70, 50, frame.area());
    frame.render_widget(Clear, area);

    let display = if masked { mask_inline(&value) } else { value };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Telegram Setup ",
                Style::default()
                    .fg(th::accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("· Step {}/3", step_num),
                Style::default()
                    .fg(th::accent2())
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", hint),
            Style::default().fg(th::dim()),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("    "),
            Span::styled(
                format!("{}: ", step_label),
                Style::default()
                    .fg(th::accent())
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    ▶ ", Style::default().fg(th::accent2())),
            Span::styled(
                display,
                Style::default().fg(th::text()).add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(th::accent2())),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "    [Enter] confirm     [Esc] cancel     [Backspace] erase",
            Style::default().fg(th::dim()),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "    The whole flow is inline — no shell required.",
            Style::default().fg(th::dim()),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Telegram Setup ")
        .border_style(Style::default().fg(th::accent2()));
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

/// Centered overlay modal for a single-line text input (rename, new session, etc.).
fn draw_simple_input_modal(frame: &mut Frame, app: &App, title: &str, hint: &str, masked: bool) {
    draw_simple_input_modal_owned(frame, app, title, hint, masked);
}

fn draw_simple_input_modal_owned(
    frame: &mut Frame,
    app: &App,
    title: &str,
    hint: &str,
    masked: bool,
) {
    let area = centered_rect(60, 30, frame.area());
    frame.render_widget(Clear, area);

    let display = if masked {
        mask_inline(&app.input_buffer)
    } else {
        app.input_buffer.clone()
    };

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", title),
            Style::default()
                .fg(th::accent())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", hint),
            Style::default().fg(th::dim()),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("    ▶ ", Style::default().fg(th::accent2())),
            Span::styled(
                display,
                Style::default().fg(th::text()).add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(th::accent2())),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "    [Enter] confirm     [Esc] cancel     [Backspace] erase",
            Style::default().fg(th::dim()),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title))
        .border_style(Style::default().fg(th::accent2()));
    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

/// Centered rect with ABSOLUTE width/height (clamped to the frame) — for
/// pickers whose row count is dynamic (e.g. the installed-agent list).
fn centered_rect_abs(width: u16, height: u16, r: Rect) -> Rect {
    let w = width.min(r.width);
    let h = height.min(r.height);
    Rect {
        x: r.x + (r.width.saturating_sub(w)) / 2,
        y: r.y + (r.height.saturating_sub(h)) / 2,
        width: w,
        height: h,
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Narrower than this and a 25/75 split leaves the left list ~15 columns —
/// too narrow for any real label. Below it the 2-column tabs show one column
/// at a time, switched with Tab.
const TWO_COLUMN_MIN_WIDTH: u16 = 90;

/// Width the full tab bar needs: every label, ` │ ` between each, plus the
/// block's two border columns and its one-column padding on each side.
fn tab_bar_width() -> usize {
    let labels: usize = Tab::ORDER.iter().map(|t| t.title().chars().count()).sum();
    let separators = Tab::ORDER.len().saturating_sub(1) * 3;
    labels + separators + 4
}

fn draw_tabs(frame: &mut Frame, app: &mut App, area: Rect) {
    // On a narrow terminal — a phone in portrait is ~60 columns — the full bar
    // does not fit, and ratatui clips it mid-word: at 70 columns the last tab
    // read "Settin", at 60 it vanished entirely, so Settings did not appear to
    // exist. Collapse to the active tab plus its position instead: it always
    // fits, it never lies about how many tabs there are, and ←/→ still walk them.
    if (area.width as usize) < tab_bar_width() {
        let label = format!(
            " ‹ {} ›  {}/{} ",
            app.tab.title(),
            app.tab.index() + 1,
            Tab::ORDER.len()
        );
        let compact = Paragraph::new(Line::from(Span::styled(
            label,
            Style::default()
                .fg(th::accent())
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )))
        .block(Block::default().borders(Borders::ALL).title(" OmegaOS "));
        frame.render_widget(compact, area);
        return;
    }

    // Both derive from Tab::ORDER — reorder the tab bar there, not here.
    let titles: Vec<&str> = Tab::ORDER.iter().map(|t| t.title()).collect();
    let selected = app.tab.index();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" OmegaOS "))
        .select(selected)
        // All tabs use the same color when inactive; active tab is
        // accent+bold+underline — the underline is the non-color cue so
        // focus survives bold-as-bright emulators (a11y: never hue-only).
        // No background fill — keeps the toolbar visually clean and avoids
        // any single tab looking "different" beyond just being highlighted.
        .style(Style::default().fg(th::dim()))
        .highlight_style(
            Style::default()
                .fg(th::accent())
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );

    frame.render_widget(tabs, area);
}

/// A dim `─── label ───` group header row for a grouped left-list (same grammar
/// as the Menu tab's section headers).
fn group_header(label: &str) -> ListItem<'static> {
    ListItem::new(Line::from(Span::styled(
        format!("  ─── {} ───", label),
        Style::default()
            .fg(th::accent2())
            .add_modifier(Modifier::BOLD),
    )))
}

/// A selectable section row for a grouped left-list. `current` = this is the
/// cursor row; `focused_sel` = cursor row AND the list panel holds focus (full
/// cyan highlight). Visual selection is baked into the item — the `ListState`
/// only drives scrolling.
fn section_row(label: String, current: bool, focused_sel: bool) -> ListItem<'static> {
    let prefix = if current { "▶ " } else { "  " };
    let style = if focused_sel {
        Style::default()
            .fg(th::sel_fg())
            .bg(th::accent())
            .add_modifier(Modifier::BOLD)
    } else if current {
        Style::default().fg(th::text()).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    ListItem::new(Line::from(vec![
        Span::styled(prefix.to_string(), Style::default().fg(th::accent())),
        Span::styled(label, style),
    ]))
}

fn draw_sessions(frame: &mut Frame, app: &mut App, area: Rect) {
    let list_focused = app.session_focus == SessionFocus::List;
    let chat_focused = matches!(
        app.session_focus,
        SessionFocus::Chat | SessionFocus::ChatFullscreen
    );
    let fullscreen = app.session_focus == SessionFocus::ChatFullscreen;

    // Responsive layout. On narrow terminals (mobile / phone-width SSH) a
    // 25/75 split squeezes the Claude preview into an unusable sliver, so we
    // collapse to a SINGLE column: the focused panel fills the whole width —
    // the session list while browsing, the Claude preview while chatting.
    // Wide terminals keep the two-column 25/75 split. Fullscreen is always
    // preview-only. Tab toggles focus, so on mobile it flips list ⇄ chat,
    // each full-width.
    // Breakpoint: below this, a 25/75 split would leave Claude under ~70
    // cols (unusable). At 100 the split still gives the preview ≥75 cols;
    // below it we go single-column so the focused panel gets the full width.
    const NARROW_COLS: u16 = 100;
    let narrow = area.width < NARROW_COLS;

    let (list_area, preview_area): (Option<Rect>, Option<Rect>) = if fullscreen {
        (None, Some(area))
    } else if narrow {
        if chat_focused {
            (None, Some(area))
        } else {
            (Some(area), None)
        }
    } else {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(area);
        (Some(split[0]), Some(split[1]))
    };

    // Record the REAL panel rects for mouse hit-testing (input.rs
    // handle_mouse) — the Menu tab's geometry-cache pattern. The old
    // `column >= 30` heuristic misrouted clicks whenever the layout above
    // disagreed with it (25% of a wide terminal passes col 30; the narrow
    // single-column list owns the whole width).
    app.sessions_list_area = list_area;
    app.sessions_preview_area = preview_area;

    // Preview-only frame (fullscreen, or narrow + chat-focused): render the
    // Claude view full-width and return.
    if list_area.is_none() {
        app.sessions_rendered_rows.clear();
        app.sessions_list_fits = false;
        if let Some(pa) = preview_area {
            draw_sessions_right(frame, app, pa, chat_focused);
        }
        return;
    }
    let _ = list_focused;

    // ── Left: session list with project headers ─────────────────────────────
    // Rendered-row → entry-index map for click hit-testing (headers = None),
    // mirroring `menu_rendered_actions`. Valid only while the whole list fits
    // (ListState offset 0) — `sessions_list_fits` below guards that.
    let mut rendered_rows: Vec<Option<usize>> = Vec::with_capacity(app.rows.len());
    let mut entry_idx: usize = 0;
    let items: Vec<ListItem> = app
        .rows
        .iter()
        .map(|row| match row {
            SessionRow::Header(label) => {
                rendered_rows.push(None);
                ListItem::new(Line::from(vec![Span::styled(
                    format!("  {} ", label),
                    Style::default().fg(th::dim()).add_modifier(Modifier::BOLD),
                )]))
            }
            SessionRow::Entry(entry) => {
                rendered_rows.push(Some(entry_idx));
                // `selected` = focused selection bar (you're browsing the list).
                // `active`   = this is the session whose Claude pane fills the
                //              right panel right now — keep it visibly marked in
                //              the left list even while you type in the chat
                //              panel (list unfocused), so you always see WHICH
                //              session you're inside.
                let is_sel = entry_idx == app.selected;
                let item = render_session_item(
                    entry,
                    is_sel && list_focused,
                    is_sel && !list_focused,
                    app.session_badges.get(&entry.session.name).copied(),
                );
                entry_idx += 1;
                item
            }
        })
        .collect();

    let list_border_style = if list_focused {
        Style::default().fg(th::accent())
    } else {
        Style::default().fg(th::dim())
    };

    // Map app.selected (entry index) to rendered row index (includes headers)
    let rendered_selected = {
        let mut eidx: usize = 0;
        let mut result: Option<usize> = None;
        for (row_idx, row) in app.rows.iter().enumerate() {
            if let SessionRow::Entry(_) = row {
                if eidx == app.selected {
                    result = Some(row_idx);
                    break;
                }
                eidx += 1;
            }
        }
        result
    };

    let list_hint = match app.session_focus {
        crate::app::SessionFocus::List => "LIST  x:kill  .:lock  r:rename",
        _ => "CHAT (Tab → list to manage)",
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " Sessions ({}) — {} ",
                    app.sessions.len(),
                    list_hint
                ))
                .border_style(list_border_style),
        )
        .highlight_style(Style::default());

    // Click → row mapping is only trustworthy at ListState offset 0; a taller
    // list scrolls internally, so clicks then just change panel focus (the
    // same `menu_fits` guard the Menu tab uses).
    app.sessions_list_fits = list_area
        .map(|la| rendered_rows.len() <= la.height.saturating_sub(2) as usize)
        .unwrap_or(false);
    app.sessions_rendered_rows = rendered_rows;

    let mut state = ListState::default().with_selected(rendered_selected);
    if let Some(la) = list_area {
        frame.render_stateful_widget(list, la, &mut state);
    }

    // Wide layout shows the preview alongside the list. (Narrow + list-focused
    // is single-column and already returned above.)
    if let Some(pa) = preview_area {
        draw_sessions_right(frame, app, pa, chat_focused);
    }
}

/// Render the right column of the Sessions tab (preview + optional chat input).
/// Used both in split layout and chat-fullscreen mode.
pub(crate) fn draw_sessions_right(
    frame: &mut Frame,
    app: &mut App,
    area: Rect,
    chat_focused: bool,
) {
    let fullscreen = app.session_focus == SessionFocus::ChatFullscreen;

    let selected_name = app
        .selected_session()
        .map(|entry| entry.session.name.clone());
    let selected_provider = app
        .selected_session()
        .and_then(|entry| entry.session.provider.clone());
    let selected_model = selected_name
        .as_ref()
        .and_then(|name| app.session_meta.get(name))
        .map(|(model, _)| model.as_str());
    let provider = preview_provider(
        selected_provider.as_deref(),
        selected_name.as_deref().unwrap_or_default(),
        selected_model,
        &app.preview_content,
    );
    let preview_title = match selected_name.as_deref() {
        Some(name) => {
            let suffix = if fullscreen {
                "  [FULLSCREEN — Tab-Tab to exit]"
            } else {
                ""
            };
            format!(" {} · {}{} ", provider.label(), name, suffix)
        }
        None => " Preview ".to_string(),
    };

    let preview_border_style = if chat_focused {
        Style::default()
            .fg(provider.accent())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(th::dim())
    };

    // Record the REAL text-area dimensions so main.rs can resize the rmux
    // pane to exactly this width (avoids the right-edge clipping that a
    // terminal-percentage estimate produced).
    app.preview_inner_width = area.width.saturating_sub(2);
    app.preview_inner_height = area.height.saturating_sub(2);

    let viewport_height = area.height.saturating_sub(2);

    let mut preview_lines: Vec<Line> = if app.preview_content.is_empty() {
        let empty_copy = match selected_name.as_deref() {
            Some(name) if app.preview_fail_streak > 0 => {
                format!("(retrying preview for {name}...)")
            }
            Some(name) => format!("(loading preview for {name}...)"),
            None => "(select a session to preview)".to_string(),
        };
        vec![Line::from(Span::styled(
            empty_copy,
            Style::default().fg(th::dim()),
        ))]
    } else if let Some(styled) = &app.preview_styled {
        // Styled path: build colored spans from the pane snapshot so the
        // `/` command-menu selection highlight + Claude's colored UI show.
        //
        // Claude marks the SELECTED menu row with one subtle cue only: the
        // text turns its accent blue (ANSI 94 → rgb 59,142,234) while every
        // sibling stays gray (229,229,229). Empirically (examples/dump_styled)
        // that hue shift is too faint in the mirror — the user "can't tell
        // which one is selected". Since this is a mirror (not the real Claude
        // TTY) we are free to make it unmistakable: any row whose leading
        // visible span is the accent blue gets a full-width highlight bar
        // (▶ marker + blue background + bold).
        //
        // GATED on a menu actually being open: Claude paints OTHER things in
        // the same bright blue (the "Tip:" suggested command, e.g.
        // "/plugin install …"), and the color-only cue turned those into
        // phantom full-width accent bars in the mirror (reported via a
        // Telegram screenshot, 2026-06-11). A completion menu is only ever
        // open while the input box is being typed into, and the snapshot
        // cursor sits in that box — so require the cursor row to be an input
        // line whose content starts with a menu trigger (/ @ #).
        const ACCENT: PreviewColor = PreviewColor::Indexed(12); // Claude's bright-blue selection accent
        let menu_open = app.preview_cursor.is_some_and(|(crow, _, _)| {
            styled.get(crow as usize).is_some_and(|row| {
                let text: String = row.iter().map(|s| s.text.as_str()).collect();
                let after_prompt = text.trim_start().strip_prefix('❯');
                after_prompt.is_some_and(|rest| {
                    matches!(rest.trim_start().chars().next(), Some('/' | '@' | '#'))
                })
            })
        });
        let inner_w = area.width.saturating_sub(2) as usize;
        styled
            .iter()
            .map(|row| {
                let is_selected = menu_open
                    && row
                        .iter()
                        .find(|s| !s.text.trim().is_empty())
                        .is_some_and(|s| s.fg == Some(ACCENT));

                if is_selected {
                    // Selected line: the same highlight every other selection in
                    // the app uses — black on cyan. Named colors track the
                    // terminal theme's palette, so it reads on light AND dark
                    // themes, instead of the old hardcoded dark-navy Rgb bar that
                    // looked like a black block on a light Termius theme.
                    let bar_bg = th::accent();
                    let mut spans: Vec<Span> = Vec::with_capacity(row.len() + 2);
                    let mut width = 0usize;
                    spans.push(Span::styled(
                        "▶ ",
                        Style::default()
                            .fg(th::sel_fg())
                            .bg(bar_bg)
                            .add_modifier(Modifier::BOLD),
                    ));
                    width += 2;
                    for sp in row {
                        // Display width (columns), not char count: emoji/CJK
                        // render 2+ cols per char, so counting chars under-pads
                        // and the bar bleeds past the panel. Span::width() uses
                        // unicode-width internally (no new dep needed).
                        width += Span::raw(sp.text.as_str()).width();
                        spans.push(Span::styled(
                            sp.text.clone(),
                            Style::default()
                                .fg(th::sel_fg())
                                .bg(bar_bg)
                                .add_modifier(Modifier::BOLD),
                        ));
                    }
                    // Pad to full inner width so the bar spans the whole row.
                    if width < inner_w {
                        spans.push(Span::styled(
                            " ".repeat(inner_w - width),
                            Style::default().bg(bar_bg),
                        ));
                    }
                    Line::from(spans)
                } else {
                    // Extra emphasis on top of the ANSI baseline (which rmux
                    // already preserves — verified via examples/dump_styled:
                    // bright red 241,76,76 is captured). The user wants
                    // tmux-style visual distinction for: Claude activity
                    // status, user input echo, TodoWrite items, Task-tool
                    // sub-agent dispatches. High-confidence patterns only —
                    // anything that doesn't match falls through to the
                    // straight ANSI render.
                    let row_text: String = row.iter().map(|s| s.text.as_str()).collect();
                    let trimmed = row_text.trim();

                    // ── Pattern catalog (all tight, anchored at line start
                    //    or with multiple co-occurrences to avoid false
                    //    positives on prose) ──────────────────────────────
                    // 1. Activity status footer: "Twisting… (22s · ↓ 1.0k
                    //    tokens · thought for 2s)" and siblings. Anchored on
                    //    the "… (" verb→parenthesis seam — plain conversation
                    //    prose mentioning tokens with a stray '·' or ellipsis
                    //    was painted bold red across the row.
                    let is_activity = trimmed.contains("tokens")
                        && trimmed.contains('·')
                        && trimmed.contains("… (");
                    // 2. User input echo:
                    //    - "❯ <text>"        : Claude Code's input prompt
                    //    - "▶ You: <text>"   : AISB-master mirror format
                    //                          (~/.omega/state/aisb-conversation.log)
                    let is_user_input = (trimmed.starts_with("❯ ") && trimmed.len() > 2)
                        || trimmed.starts_with("▶ You: ");
                    // 2b. AISB-master agent reply echo: "You ▶ <text>".
                    let is_agent_reply =
                        trimmed.starts_with("You ▶ ") || trimmed.starts_with("You ▶");
                    // 3. TodoWrite items: anchored on the status glyph at
                    //    line start (after trim) so a glyph inside a table
                    //    cell or sentence doesn't match.
                    let is_todo_done = trimmed.starts_with("☑ ")
                        || trimmed.starts_with("✅ ")
                        || trimmed.starts_with("✓ ");
                    let is_todo_pending = trimmed.starts_with("☐ ");
                    let is_todo_progress = trimmed.starts_with("⏳ ");
                    let is_todo_failed = trimmed.starts_with("☒ ")
                        || trimmed.starts_with("✗ ")
                        || trimmed.starts_with("⊘ ");
                    // 4. Task-tool / sub-agent dispatch: "● Task(...)" or a
                    //    tool-echo bullet carrying "subagent_type=". The
                    //    bare contains() matched prose DISCUSSING the field
                    //    (this very kind of sentence) — require the line to
                    //    be a ● bullet.
                    let is_task_dispatch = trimmed.starts_with("● Task(")
                        || (trimmed.starts_with('●') && trimmed.contains("subagent_type="));

                    // Order matters: more-specific patterns first so a TODO
                    // line that happens to contain "tokens" isn't mis-typed
                    // as activity. Activity is checked LAST as a fallback.
                    if is_todo_done {
                        emphasized_preview_row(row, Some(th::success()), Some(Modifier::BOLD))
                    } else if is_todo_failed {
                        emphasized_preview_row(row, Some(th::error()), Some(Modifier::BOLD))
                    } else if is_todo_progress {
                        emphasized_preview_row(row, Some(th::special()), Some(Modifier::BOLD))
                    } else if is_todo_pending {
                        emphasized_preview_row(row, Some(th::accent2()), None)
                    } else if is_task_dispatch {
                        emphasized_preview_row(row, Some(th::accent()), Some(Modifier::BOLD))
                    } else if is_activity {
                        emphasized_preview_row(row, Some(th::error()), Some(Modifier::BOLD))
                    } else if is_user_input {
                        // The provider accent is a fallback only. Captured ANSI,
                        // including a Codex/Gemini composer background, wins.
                        emphasized_preview_row(row, Some(provider.accent()), Some(Modifier::BOLD))
                    } else if is_agent_reply {
                        emphasized_preview_row(row, Some(th::success()), Some(Modifier::BOLD))
                    } else {
                        let spans: Vec<Span> = row
                            .iter()
                            .map(|sp| Span::styled(sp.text.clone(), preview_span_style(sp)))
                            .collect();
                        Line::from(spans)
                    }
                }
            })
            .collect()
    } else if let Some(history) = &app.preview_history_styled {
        // Scrolled-back path: the deep capture now carries its attributes
        // (capture-pane -e, parsed by styled_rows_from_ansi), so history keeps
        // the colors of the Claude / Codex conversation instead of going
        // grayscale the moment the wheel moves. Straight ANSI render only — the
        // menu-highlight heuristic above is cursor-driven and the cursor only
        // means something on the live tail.
        history
            .iter()
            .map(|row| {
                Line::from(
                    row.iter()
                        .map(|sp| Span::styled(sp.text.clone(), preview_span_style(sp)))
                        .collect::<Vec<Span>>(),
                )
            })
            .collect()
    } else {
        app.preview_content
            .lines()
            .map(|l| Line::from(l.to_string()))
            .collect()
    };

    // Determine the source cursor before reflow. The snapshot coordinate is a
    // display column, not a byte/char index. The fallback uses ratatui's
    // Unicode-aware line width for static/plain captures.
    let source_cursor = app
        .preview_cursor
        .map(|(row, col, _)| (row, col))
        .or_else(|| {
            preview_lines
                .iter()
                .enumerate()
                .rev()
                .find(|(_, line)| {
                    line.spans
                        .iter()
                        .any(|span| !span.content.trim().is_empty())
                })
                .map(|(row, line)| {
                    (
                        row.min(u16::MAX as usize) as u16,
                        line.width().min(u16::MAX as usize) as u16,
                    )
                })
        });

    // Core intentionally drops trailing blank snapshot rows. Restore only the
    // rows needed to carry the real cursor, then hard-wrap every source row to
    // the current preview width. This covers the resize/SIGWINCH transition
    // without waiting for the provider TUI to redraw itself.
    if let Some((source_row, _)) = source_cursor {
        while preview_lines.len() <= usize::from(source_row) {
            preview_lines.push(Line::from(""));
        }
    }
    let reflowed = reflow_preview_lines(&preview_lines, app.preview_inner_width);
    let mapped_cursor = source_cursor.map(|(row, col)| {
        reflowed_cursor(
            &preview_lines,
            &reflowed.source_row_starts,
            row,
            col,
            app.preview_inner_width,
        )
    });
    preview_lines = reflowed.lines;

    // Count painted rows after Unicode-safe reflow. A stale 132-column rmux
    // frame shown inside an 80-column panel may occupy more rows than its
    // logical capture; counting source lines made the live composer disappear.
    let total_lines = preview_lines.len().min(u16::MAX as usize) as u16;
    let max_scroll = total_lines.saturating_sub(viewport_height);
    app.preview_max_scroll = max_scroll;
    // `preview_scroll` is measured from the tail; Paragraph wants a from-top
    // offset. Clamp the from-tail value, then convert.
    let from_tail = app.preview_scroll.min(max_scroll);
    let scroll = max_scroll.saturating_sub(from_tail);

    // Bottom-anchor the live terminal mirror. A real terminal keeps the
    // prompt pinned to the bottom while history scrolls up off the top. When
    // the pane content is shorter than the panel (a freshly-started Claude
    // session), pad the TOP so the input line sits at the bottom instead of
    // leaving the lower half of the panel blank.
    let mut bottom_pad: u16 = 0;
    if app.preview_styled.is_some() && max_scroll == 0 {
        let len = preview_lines.len() as u16;
        if len < viewport_height {
            bottom_pad = viewport_height - len;
            let mut padded: Vec<Line> = Vec::with_capacity(viewport_height as usize);
            for _ in 0..bottom_pad {
                padded.push(Line::from(String::new()));
            }
            padded.append(&mut preview_lines);
            preview_lines = padded;
        }
    }

    // ── Mouse drag-selection (tmux-style) ───────────────────────────────
    // Capture the viewport rows as plain text so button-release can resolve
    // the drag rectangle to real text (app.take_preview_selection_text), and
    // paint the in-flight selection REVERSED so you see what you're grabbing.
    let inner_h = area.height.saturating_sub(2) as usize;
    app.preview_screen_rows = (0..inner_h)
        .map(|r| {
            preview_lines
                .get(scroll as usize + r)
                .map(|l| {
                    l.spans
                        .iter()
                        .map(|s| s.content.as_ref())
                        .collect::<String>()
                })
                .unwrap_or_default()
        })
        .collect();
    if app.preview_select_dragging {
        if let Some(((sc, sr), (ec, er))) = app.preview_selection_viewport() {
            for r in sr..=er.min(inner_h.saturating_sub(1)) {
                let li = scroll as usize + r;
                if let Some(line) = preview_lines.get_mut(li) {
                    let from = if r == sr { sc } else { 0 };
                    let to = if r == er { ec + 1 } else { usize::MAX };
                    *line = reverse_cols(line, from, to);
                }
            }
        }
    }

    // Reading state, spelled out. A reader on a phone could not tell whether
    // the text was still going to slide away under them; PAUSED says the view
    // is theirs and names the key that gives it back to the tail.
    let scroll_indicator = if !app.preview_follow_tail && max_scroll > 0 {
        format!(" ⏸ PAUSED [{}/{}] End→live ", scroll, max_scroll)
    } else if max_scroll > 0 {
        format!(" [{}/{}] ", scroll, max_scroll)
    } else {
        String::new()
    };

    // Right side of the title: the previewed session's model + cumulative
    // token consumption (e.g. "opus-4.8 · 45.4M tok"), refreshed off the hot
    // path in main.rs. Replaces the old static key-hint text. Falls back to a
    // tiny interactive marker only when there's no meta yet (non-Claude
    // session, or first 3s before the first scan).
    let meta_suffix = app
        .selected_session()
        .and_then(|e| app.session_meta.get(&e.session.name))
        .map(|(model, tokens)| {
            format!(
                "  ⟨ {} · {} tok ⟩",
                model,
                omega_core::claude_meta::fmt_tokens(*tokens)
            )
        });
    let title = match meta_suffix {
        Some(meta) => format!("{}{}{}", preview_title, scroll_indicator, meta),
        None if chat_focused => format!("{}{}  [keys → session]", preview_title, scroll_indicator),
        None => format!("{}{}", preview_title, scroll_indicator),
    };
    let preview = Paragraph::new(preview_lines).scroll((scroll, 0)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(preview_border_style),
    );
    frame.render_widget(preview, area);

    // Visible cursor when chat is focused. Two layers (belt + suspenders):
    //   1. A painted yellow block glyph in the buffer — always visible
    //      regardless of terminal cursor settings.
    //   2. The OS-native blinking caret via set_cursor_position.
    // Positioned at the typing point: end of the last non-empty visible
    // line of the pane capture.
    if chat_focused {
        let inner_w = area.width.saturating_sub(2);
        let inner_h = area.height.saturating_sub(2);

        // Prefer the REAL cursor (row, col) reported by the pane snapshot.
        // That's exactly where the agent's input caret is. Fall back to the
        // last-non-empty-line heuristic only if the snapshot didn't carry a
        // cursor (e.g. history-browsing mode).
        let (cur_row, cur_col) = mapped_cursor.unwrap_or((0, 0));

        // Map the snapshot row to a viewport row using the same from-top
        // `scroll` offset the Paragraph uses, plus the bottom-anchor padding
        // we prepended so the caret tracks the content down to the bottom.
        let viewport_row = (cur_row + bottom_pad).saturating_sub(scroll);
        if viewport_row < inner_h {
            // Saturating add: a large `area.x` near u16::MAX could otherwise
            // wrap, falsifying the `cursor_x < area.x + area.width` bounds check.
            let cursor_x = area
                .x
                .saturating_add(1)
                .saturating_add(cur_col.min(inner_w.saturating_sub(1)));
            let cursor_y = area.y + 1 + viewport_row;
            if cursor_x < area.x + area.width && cursor_y < area.y + area.height {
                // Layer 1: painted glyph (always visible).
                let buf = frame.buffer_mut();
                if let Some(cell) = buf.cell_mut((cursor_x, cursor_y)) {
                    cell.set_symbol("▏");
                    cell.set_style(
                        Style::default()
                            .fg(th::accent2())
                            .add_modifier(Modifier::RAPID_BLINK),
                    );
                }
                // Layer 2: OS-native caret.
                frame.set_cursor_position((cursor_x, cursor_y));
            }
        }
    }
    let _ = fullscreen;
}

fn render_session_item(
    entry: &SessionEntry,
    selected: bool,
    active: bool,
    badge: Option<DoneStatus>,
) -> ListItem<'static> {
    let icon = match entry.session.role {
        SessionRole::Oracle => "◆",
        SessionRole::Worker => "●",
        SessionRole::Home => "⌂",
        SessionRole::System => "⚙",
    };

    let icon_color = match entry.session.role {
        SessionRole::Oracle => th::accent2(),
        SessionRole::Worker => th::success(),
        SessionRole::Home => th::info(),
        SessionRole::System => th::dim(),
    };

    let progress_str = match &entry.progress {
        Some(p) => format!(" {} {:.0}%", p.bar(8), p.percentage()),
        None => String::new(),
    };

    let prefix = if selected || active {
        "▶ ".to_string()
    } else {
        "  ".to_string()
    };

    let name_style = if selected {
        Style::default()
            .fg(th::sel_fg())
            .bg(th::accent())
            .add_modifier(Modifier::BOLD)
    } else if active {
        // Connected session, but the list isn't focused (you're typing in the
        // chat panel). A softer cue than the solid selection bar — underlined
        // cyan name + "▶" — so you can always tell WHICH session you're inside
        // without implying the list has keyboard focus.
        Style::default()
            .fg(th::accent())
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else {
        // Passive: plain body text, NO bold — bold on every row carries zero
        // hierarchy. Bold stays reserved for the active session and the
        // selection bar (doctrine: actif = vif + gras, passif = normal).
        Style::default().fg(th::text())
    };

    let protect_marker = if entry.is_protected { "§ " } else { "" };

    // Done/blocked badge from the worker's done.json / worker-blocked signal.
    let (badge_glyph, badge_color) = match badge {
        Some(DoneStatus::DoneClean) => ("+ ", th::success()),
        Some(DoneStatus::Pending) => ("~ ", th::accent2()),
        Some(DoneStatus::Failed) => ("x ", th::error()),
        Some(DoneStatus::Blocked) => ("! ", th::warn()),
        None => ("", th::text()),
    };

    let line = Line::from(vec![
        Span::styled(prefix, Style::default().fg(th::accent())),
        Span::raw(entry.tree_prefix.clone()),
        Span::styled(format!("{} ", icon), Style::default().fg(icon_color)),
        Span::styled(protect_marker, Style::default().fg(th::special())),
        Span::styled(
            badge_glyph,
            Style::default()
                .fg(badge_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(entry.session.name.clone(), name_style),
        Span::styled(progress_str, Style::default().fg(th::accent())),
    ]);

    ListItem::new(line)
}

fn menu_group(action: &MenuAction) -> &'static str {
    match action {
        MenuAction::NewClaude
        | MenuAction::NewCodex
        | MenuAction::NewGemini
        | MenuAction::NewAntigravity
        | MenuAction::NewPi
        | MenuAction::NewHermes
        | MenuAction::NewGlm
        | MenuAction::NewKimi => "New agent sessions",
        MenuAction::NewTerminal => "Terminal",
        MenuAction::NewProject | MenuAction::DispatchOracle => "Orchestration",
        MenuAction::Refresh | MenuAction::ToggleProtection | MenuAction::KillSelected => {
            "Session actions"
        }
        MenuAction::KillAll | MenuAction::NuclearCleanup => "Danger zone",
        MenuAction::Restart | MenuAction::Quit => "OmegaOS",
    }
}

fn draw_menu(frame: &mut Frame, app: &mut App, area: Rect) {
    // Build items with section headers so the menu reads as grouped sections.
    let mut items: Vec<ListItem> = Vec::new();
    // Parallel to `items`: rendered-row → action index (None for header/blank
    // rows). Lets a mouse click hit-test which action was clicked.
    let mut rendered_actions: Vec<Option<usize>> = Vec::new();
    let mut last_group: Option<&'static str> = None;
    for (i, action) in MenuAction::all().iter().enumerate() {
        let group = menu_group(action);
        if last_group != Some(group) {
            if last_group.is_some() {
                items.push(ListItem::new(Line::from("")));
                rendered_actions.push(None);
            }
            items.push(ListItem::new(Line::from(Span::styled(
                format!("  ─── {} ───", group),
                Style::default()
                    .fg(th::accent2())
                    .add_modifier(Modifier::BOLD),
            ))));
            rendered_actions.push(None);
            last_group = Some(group);
        }
        rendered_actions.push(Some(i));
        let selected = i == app.menu_selected;
        let prefix = if selected { "▶ " } else { "  " };
        let label_style = if selected {
            Style::default()
                .fg(th::sel_fg())
                .bg(th::accent())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        // Agent launchers carry their install state. Pressing one of these
        // when the CLI is missing already refuses with an install hint — but
        // only AFTER the press. Showing it on the row means you can see which
        // tools are actually usable before trying them.
        let (state_glyph, state_style) = match action.agent() {
            Some(agent) if !matches!(agent, omega_core::agents::Agent::Shell) => {
                if crate::app::agent_available_cached(agent) {
                    ("● ", Style::default().fg(th::success()))
                } else {
                    ("○ ", Style::default().fg(th::dim()))
                }
            }
            _ => ("", Style::default()),
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(prefix, Style::default().fg(th::accent())),
            Span::styled(state_glyph, state_style),
            Span::styled(
                format!("[{}] ", action.shortcut()),
                Style::default()
                    .fg(th::accent2())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(action.label(), label_style),
            // Say it in words too: a glyph alone is not a state anyone can read
            // on a first visit (and colour alone fails on mono themes).
            Span::styled(
                match action.agent() {
                    Some(agent)
                        if !matches!(agent, omega_core::agents::Agent::Shell)
                            && !crate::app::agent_available_cached(agent) =>
                    {
                        "   not installed".to_string()
                    }
                    _ => String::new(),
                },
                Style::default().fg(th::dim()),
            ),
        ])));
    }

    // Record layout for mouse hit-testing (input.rs handle_mouse). `menu_fits`
    // is true only when the whole list shows without scrolling (inner height =
    // area minus the 1-row top+bottom border) → click row maps 1:1 to a row.
    app.menu_area = area;
    app.menu_fits = items.len() <= area.height.saturating_sub(2) as usize;
    app.menu_rendered_actions = rendered_actions;

    // Compute rendered row index for the selected action (accounting for
    // header rows + blank separator rows between groups).
    let rendered_selected = {
        let mut idx: usize = 0;
        let mut last: Option<&'static str> = None;
        for (i, action) in MenuAction::all().iter().enumerate() {
            let g = menu_group(action);
            if last != Some(g) {
                if last.is_some() {
                    idx += 1;
                } // blank line
                idx += 1; // header line
                last = Some(g);
            }
            if i == app.menu_selected {
                break;
            }
            idx += 1;
        }
        idx
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Actions — ↑/↓ or click · Enter runs · Ctrl-T text-select/copy ")
                .border_style(Style::default().fg(th::accent())),
        )
        .highlight_style(Style::default()); // selection visual is already baked into items

    let mut state = ListState::default().with_selected(Some(rendered_selected));
    frame.render_stateful_widget(list, area, &mut state);
}

/// Build the unified Settings-tab left list: a top-padding blank, the Monitor
/// group, a blank gap, then the Settings group. Returns (items, flat_selected)
/// where `flat_selected` is the rendered row index of the cursor (for the
/// `ListState` so it scrolls to keep the cursor visible).
fn build_settings_list(app: &App) -> (Vec<ListItem<'static>>, usize) {
    let list_focused = !app.detail_focused;
    let mut items: Vec<ListItem> = Vec::new();
    let mut flat_selected = 0usize;

    // Top padding.
    items.push(ListItem::new(Line::from("")));

    // ── Group 1: Monitor ────────────────────────────────────────────────────
    items.push(group_header("Monitor"));
    for (i, sec) in MonitorSection::all().iter().enumerate() {
        let current = app.settings_group == 0 && i == app.monitor_selected;
        if current {
            flat_selected = items.len();
        }
        items.push(section_row(
            sec.label().to_string(),
            current,
            current && list_focused,
        ));
    }

    // Gap between groups.
    items.push(ListItem::new(Line::from("")));

    // ── Group 2: Settings ───────────────────────────────────────────────────
    items.push(group_header("Settings"));
    for (i, sec) in SettingsSection::all().iter().enumerate() {
        let current = app.settings_group == 1 && i == app.settings_selected;
        if current {
            flat_selected = items.len();
        }
        items.push(section_row(
            sec.label().to_string(),
            current,
            current && list_focused,
        ));
    }

    (items, flat_selected)
}

/// Build the right-panel lines for the currently selected Monitor section.
/// Returns (lines, selected_line) — `selected_line` is the line of the
/// highlighted action (Actions section, focused); 0 otherwise.
fn render_monitor_detail(app: &App) -> (Vec<Line<'static>>, usize) {
    // Inline sub-header for a merged section's second block.
    let sub = |label: &str| {
        Line::from(Span::styled(
            format!("  ─── {} ───", label),
            Style::default()
                .fg(th::accent2())
                .add_modifier(Modifier::BOLD),
        ))
    };
    match app.selected_monitor_section() {
        // Connected account + live billing, one panel.
        MonitorSection::AccountBilling => {
            let mut lines = render_monitor_account(app);
            lines.push(Line::from(""));
            lines.push(sub("Billing (live)"));
            lines.extend(render_monitor_billing());
            (lines, 0)
        }
        // Telegram bot config + the project group, one panel.
        MonitorSection::Telegram => {
            let mut lines = render_monitor_telegram();
            lines.push(Line::from(""));
            lines.push(sub("Project group"));
            lines.extend(render_monitor_projects());
            (lines, 0)
        }
        MonitorSection::Actions => render_monitor_actions(app),
    }
}

/// Split a long unbreakable string into fixed-width chunks.
///
/// Used for the OAuth authorize URL: the Monitor detail Paragraph scrolls
/// instead of wrapping, so anything wider than the panel is clipped and lost.
/// Chunking on CHARACTERS (not bytes) keeps multi-byte input intact; a URL is
/// ASCII, but this must never be able to split a char and panic.
fn fold_for_panel(s: &str, width: usize) -> Vec<String> {
    if s.is_empty() {
        return vec![String::new()];
    }
    let chars: Vec<char> = s.chars().collect();
    chars
        .chunks(width.max(1))
        .map(|c| c.iter().collect())
        .collect()
}

fn render_monitor_account(app: &App) -> Vec<Line<'static>> {
    use crate::app::ReauthStatus;
    use omega_core::monitor;
    let mut lines: Vec<Line> = vec![Line::from("")];
    if let Some(acc) = monitor::connected_account() {
        lines.push(Line::from(vec![
            Span::raw("    Email:          "),
            Span::styled(
                acc.email.clone(),
                Style::default()
                    .fg(th::success())
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::raw("    Plan:           "),
            Span::styled(
                format!("Claude {}", acc.plan.to_uppercase()),
                Style::default()
                    .fg(th::accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("   ({})", acc.auth_method)),
        ]));
    } else {
        lines.push(Line::from(Span::styled(
            "    (no connected account)",
            Style::default().fg(th::dim()),
        )));
    }
    lines.push(Line::from(""));

    // OAuth re-login engine state — drives the guided flow in-place.
    match &app.reauth_status {
        ReauthStatus::Idle => {
            lines.push(Line::from(Span::styled(
                "    ▶ Press Enter to re-auth Claude (guided OAuth — captures the login URL)",
                Style::default()
                    .fg(th::success())
                    .add_modifier(Modifier::BOLD),
            )));
        }
        ReauthStatus::Generating => {
            lines.push(Line::from(Span::styled(
                "    ⏳ Starting login session and capturing the authorize URL… (~15s)",
                Style::default()
                    .fg(th::accent2())
                    .add_modifier(Modifier::BOLD),
            )));
        }
        ReauthStatus::ShowUrl(url) => {
            lines.push(Line::from(Span::styled(
                "    1) Open this URL in your browser and authorize:",
                Style::default()
                    .fg(th::accent())
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            // The detail Paragraph deliberately does not wrap (it scrolls), so a
            // ~400-char authorize URL on ONE Line is clipped at the panel edge —
            // the operator sees a cut link and cannot select the rest of it. Fold
            // it ourselves into panel-safe chunks so the WHOLE URL is on screen
            // and copyable. Telegram's Account card is the one-tap path (it sends
            // the URL as a native button); this is the terminal fallback.
            for chunk in fold_for_panel(url, 56) {
                lines.push(Line::from(Span::styled(
                    format!("    {}", chunk),
                    Style::default()
                        .fg(th::info())
                        .add_modifier(Modifier::UNDERLINED),
                )));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "    2) Press Enter to paste the code you get back.",
                Style::default()
                    .fg(th::success())
                    .add_modifier(Modifier::BOLD),
            )));
        }
        ReauthStatus::Validating => {
            lines.push(Line::from(Span::styled(
                "    ⏳ Submitting code and waiting for credentials to refresh… (~20s)",
                Style::default()
                    .fg(th::accent2())
                    .add_modifier(Modifier::BOLD),
            )));
        }
        ReauthStatus::Done(msg) => {
            lines.push(Line::from(Span::styled(
                format!("    ✓ {}", msg),
                Style::default()
                    .fg(th::success())
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "    ▶ Press Enter to re-auth again.",
                Style::default().fg(th::dim()),
            )));
        }
        ReauthStatus::Error(msg) => {
            lines.push(Line::from(Span::styled(
                format!("    ✗ {}", msg),
                Style::default()
                    .fg(th::error())
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "    ▶ Press Enter to retry.",
                Style::default().fg(th::dim()),
            )));
        }
    }
    lines
}

fn render_monitor_billing() -> Vec<Line<'static>> {
    use omega_core::monitor;
    // Runs every frame while the Billing section is open — memoize the disk
    // reads + process scan (see RENDER_TTL above).
    thread_local! {
        #[allow(clippy::type_complexity)]
        static BILLING_MEMO: std::cell::RefCell<
            Option<(
                std::time::Instant,
                (
                    UsageSnapshotRead,
                    Option<u64>,
                    omega_core::monitor::AisbBotStatus,
                ),
            )>,
        > = const { std::cell::RefCell::new(None) };
    }
    let (snap, cache_age, bot_status) = render_memo(&BILLING_MEMO, || {
        (
            monitor::UsageSnapshot::read().map_err(|error| error.to_string()),
            monitor::UsageSnapshot::cache_age_secs(),
            monitor::aisb_bot_status(),
        )
    });
    let mut lines: Vec<Line> = vec![Line::from("")];

    if let Ok(Some(snap)) = &snap {
        let cache_label = match cache_age {
            Some(s) if s < 60 => format!("{}s ago", s),
            Some(s) if s < 3600 => format!("{}m ago", s / 60),
            Some(s) => format!("{}h ago", s / 3600),
            None => "?".to_string(),
        };
        lines.push(Line::from(format!(
            "    Account:        {}  ({})",
            if !snap.active_account.is_empty() {
                snap.active_account.as_str()
            } else {
                "—"
            },
            if !snap.email.is_empty() {
                snap.email.as_str()
            } else {
                "—"
            }
        )));
        lines.push(Line::from(format!(
            "    Source:         {}    Cache: {}",
            snap.source, cache_label
        )));
        lines.push(Line::from(""));

        for (label, pct, tokens, budget) in [
            (
                "5h session",
                snap.precise_5h(),
                snap.tokens_5h,
                snap.budget_5h,
            ),
            (
                "Week",
                snap.precise_week(),
                snap.tokens_7d,
                snap.budget_week,
            ),
        ] {
            lines.push(Line::from(vec![
                Span::raw(format!("    {:11} ", label)),
                Span::styled(render_bar(pct, 30), Style::default().fg(pct_color(pct))),
                Span::raw(format!(" {:5.1}%  ", pct)),
                Span::styled(
                    format!("{} / {} tok", short_num(tokens), short_num(budget)),
                    Style::default().fg(th::dim()),
                ),
            ]));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(format!(
            "    Sonnet:         {}%        Extra: {}%",
            snap.sonnet_pct, snap.extra_pct
        )));
    } else if let Err(error) = &snap {
        lines.push(Line::from(Span::styled(
            format!("    INVALID usage snapshot: {error}"),
            Style::default()
                .fg(th::error())
                .add_modifier(Modifier::BOLD),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "    (no usage snapshot yet)",
            Style::default().fg(th::dim()),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "    ▶ Press Enter to refresh billing now (live OAuth usage check)",
        Style::default()
            .fg(th::success())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "    ── Usage cache (AISB legacy Python bot, separate from OmegaOS) ──",
        Style::default().fg(th::accent2()),
    )));
    lines.push(Line::from(Span::styled(
        format!(
            "    Billing reads {} (omega usage --check, native OAuth).",
            omega_core::config::omega_dir()
                .join("state/usage.json")
                .display()
        ),
        Style::default().fg(th::dim()),
    )));
    let (bot_icon, bot_color, bot_text) = if bot_status.bot_alive {
        ("●", th::success(), "running")
    } else {
        ("○", th::error(), "not detected")
    };
    lines.push(Line::from(vec![
        Span::raw("    Process status: "),
        Span::styled(
            format!("{} {}", bot_icon, bot_text),
            Style::default().fg(bot_color),
        ),
    ]));
    let cache_text = match bot_status.cache_status {
        monitor::CacheStatus::Fresh(s) => format!("fresh ({}s ago)", s),
        monitor::CacheStatus::Stale(s) => format!("stale ({}s ago)", s),
        monitor::CacheStatus::Missing => "missing".to_string(),
    };
    lines.push(Line::from(format!("    Usage cache:    {}", cache_text)));
    lines
}

fn render_monitor_telegram() -> Vec<Line<'static>> {
    use omega_core::monitor;
    // Per-frame TOML read → RENDER_TTL memo (see above).
    thread_local! {
        static TG_MEMO: std::cell::RefCell<
            Option<(std::time::Instant, TelegramConfigRead)>,
        > = const { std::cell::RefCell::new(None) };
    }
    let tg_config = render_memo(&TG_MEMO, || {
        monitor::OmegaTelegramConfig::try_read().map_err(|error| error.to_string())
    });
    render_monitor_telegram_from(tg_config)
}

fn render_monitor_telegram_from(tg_config: TelegramConfigRead) -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = vec![Line::from("")];

    lines.push(Line::from(Span::styled(
        "    Omega Telegram Bot (Rust — this system)",
        Style::default()
            .fg(th::accent())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "      Omega's OWN Telegram bot (no Python, no AISB-Python dependency).",
        Style::default().fg(th::text()),
    )));
    lines.push(Line::from(Span::styled(
        "      Once configured, text you send via Telegram reaches Atlas",
        Style::default().fg(th::text()),
    )));
    lines.push(Line::from(Span::styled(
        "      — your phone-side control surface for the system.",
        Style::default().fg(th::text()),
    )));
    lines.push(Line::from(""));
    if let Ok(Some(cfg)) = tg_config {
        let state = if cfg.enabled {
            "enabled"
        } else {
            "configured (disabled)"
        };
        let color = if cfg.enabled {
            th::success()
        } else {
            th::accent2()
        };
        lines.push(Line::from(vec![
            Span::raw("    Status:         "),
            Span::styled(state.to_string(), Style::default().fg(color)),
        ]));
        if !cfg.label.is_empty() {
            lines.push(Line::from(format!("    Label:          {}", cfg.label)));
        }
        lines.push(Line::from(format!("    Chat ID:        {}", cfg.chat_id)));
        let sender = format!("user_ids {:?}", cfg.allow_user_ids);
        lines.push(Line::from(format!("    Sender filter:  {}", sender)));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "    ▶ Press Enter to DISCONNECT the bot (two-press confirm)",
            Style::default()
                .fg(th::error())
                .add_modifier(Modifier::BOLD),
        )));
    } else if let Err(error) = tg_config {
        lines.push(Line::from(Span::styled(
            format!("    Status:         INVALID configuration ({error})"),
            Style::default()
                .fg(th::error())
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "    Refusing to replace malformed authority. Repair the file or disconnect explicitly.",
            Style::default().fg(th::error()),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "    Status:         (not configured)",
            Style::default().fg(th::dim()),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "    Before you start: create a bot via @BotFather (save the token) and",
            Style::default().fg(th::text()),
        )));
        lines.push(Line::from(Span::styled(
            "    get your chat id from @userinfobot. The wizard asks for them step by step.",
            Style::default().fg(th::text()),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "    ▶ Press Enter to set up the bot (guided, no command needed)",
            Style::default()
                .fg(th::success())
                .add_modifier(Modifier::BOLD),
        )));
    }
    lines
}

#[cfg(test)]
mod monitor_render_tests {
    use super::*;

    fn text(lines: &[Line<'static>]) -> String {
        lines
            .iter()
            .flat_map(|line| line.spans.iter().map(|span| span.content.as_ref()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn malformed_telegram_authority_is_never_rendered_as_absent() {
        let invalid = text(&render_monitor_telegram_from(Err(
            "strict schema rejected".to_string()
        )));
        assert!(invalid.contains("INVALID configuration"));
        assert!(invalid.contains("strict schema rejected"));
        assert!(!invalid.contains("(not configured)"));

        let absent = text(&render_monitor_telegram_from(Ok(None)));
        assert!(absent.contains("(not configured)"));
        assert!(!absent.contains("INVALID configuration"));
    }

    #[test]
    fn malformed_telegram_group_authority_is_never_rendered_as_absent() {
        let invalid = text(&render_monitor_projects_from(Err(
            "duplicate topic id".to_string()
        )));
        assert!(invalid.contains("INVALID group state"));
        assert!(invalid.contains("duplicate topic id"));
        assert!(!invalid.contains("Not configured"));

        let absent = text(&render_monitor_projects_from(Ok(None)));
        assert!(absent.contains("Not configured"));
        assert!(!absent.contains("INVALID group state"));
    }
}

fn render_monitor_projects() -> Vec<Line<'static>> {
    // Per-frame JSON read → RENDER_TTL memo (see above).
    thread_local! {
        static GROUP_MEMO: std::cell::RefCell<
            Option<(std::time::Instant, TelegramGroupRead)>,
        > = const { std::cell::RefCell::new(None) };
    }
    let group = render_memo(&GROUP_MEMO, || {
        omega_core::telegram_group::TelegramGroupConfig::try_load()
            .map_err(|error| error.to_string())
    });
    render_monitor_projects_from(group)
}

fn render_monitor_projects_from(group: TelegramGroupRead) -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(
            "    Project group (auto-detected)",
            Style::default()
                .fg(th::accent())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    match group {
        Ok(Some(gcfg)) => {
            lines.push(Line::from(vec![
                Span::raw("    Status:         "),
                Span::styled(
                    "● Connected",
                    Style::default()
                        .fg(th::success())
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(format!(
                "    Group:          {}  ({})",
                if gcfg.group_name.is_empty() {
                    "—".to_string()
                } else {
                    gcfg.group_name.clone()
                },
                gcfg.group_id
            )));
            lines.push(Line::from(format!(
                "    Topics:         {} mapped",
                gcfg.topics.len()
            )));
            if !gcfg.topics.is_empty() {
                let names: Vec<String> = gcfg.topics.keys().cloned().collect();
                let preview = names.join(", ");
                let trimmed: String = preview.chars().take(80).collect();
                lines.push(Line::from(format!(
                    "                    {}{}",
                    trimmed,
                    if preview.len() > 80 { " …" } else { "" }
                )));
            }
            lines.push(Line::from(format!("    Set up at:      {}", gcfg.setup_at)));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "    ▶ Press Enter to change the group id",
                Style::default()
                    .fg(th::success())
                    .add_modifier(Modifier::BOLD),
            )));
        }
        Err(error) => {
            lines.push(Line::from(Span::styled(
                format!("    Status:         INVALID group state ({error})"),
                Style::default()
                    .fg(th::error())
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "    Automatic topic reconciliation is blocked until the state is repaired.",
                Style::default().fg(th::error()),
            )));
        }
        Ok(None) => {
            lines.push(Line::from(Span::styled(
                "    Status:         ○ Not configured",
                Style::default().fg(th::dim()),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "    Setup is automatic:",
                Style::default().fg(th::accent()),
            )));
            lines.push(Line::from(Span::styled(
                "      1. Create a Telegram supergroup, enable Topics in its settings",
                Style::default().fg(th::text()),
            )));
            lines.push(Line::from(Span::styled(
                "      2. Add the bot to the group and make it admin",
                Style::default().fg(th::text()),
            )));
            lines.push(Line::from(Span::styled(
                "      3. That's it — the bot auto-detects the promotion, persists the",
                Style::default().fg(th::text()),
            )));
            lines.push(Line::from(Span::styled(
                "         group, creates one topic per project, and DMs you a confirmation.",
                Style::default().fg(th::text()),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "    ▶ Press Enter to set the group id manually (guided, no command needed)",
                Style::default()
                    .fg(th::success())
                    .add_modifier(Modifier::BOLD),
            )));
        }
    }
    lines
}

fn render_monitor_actions(app: &App) -> (Vec<Line<'static>>, usize) {
    let detail_active = app.detail_focused;
    let mut lines: Vec<Line> = vec![Line::from("")];
    let mut selected_line: usize = 0;

    if detail_active {
        lines.push(Line::from(Span::styled(
            "  ↑/↓ navigate · Enter runs · Tab → back to list",
            Style::default()
                .fg(th::accent2())
                .add_modifier(Modifier::BOLD),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "  Tab → focus this panel to run actions (every section is also Enter-actionable)",
            Style::default().fg(th::dim()),
        )));
    }
    lines.push(Line::from(""));

    for (i, action) in MonitorAction::all().iter().enumerate() {
        let selected = detail_active && i == app.monitor_action_selected;
        if selected {
            selected_line = lines.len();
        }
        let prefix = if selected { "  ▶ " } else { "    " };
        let label_style = if selected {
            Style::default()
                .fg(th::sel_fg())
                .bg(th::accent())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(th::accent())),
            Span::styled(
                format!("[{}] ", action.shortcut()),
                Style::default()
                    .fg(th::accent2())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(action.label(), label_style),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  This tab refreshes every 5s.",
        Style::default().fg(th::dim()),
    )));

    (lines, selected_line)
}

fn render_bar(pct: f32, width: usize) -> String {
    let filled = ((pct / 100.0) * width as f32).round() as usize;
    let filled = filled.min(width);
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

fn pct_color(pct: f32) -> Color {
    if pct < 50.0 {
        th::success()
    } else if pct < 80.0 {
        th::accent2()
    } else {
        th::error()
    }
}

fn short_num(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1}G", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// Build the System tab's left section list.
fn build_system_list(app: &App) -> (Vec<ListItem<'static>>, usize) {
    let list_focused = !app.detail_focused;
    let mut items: Vec<ListItem> = Vec::new();
    let mut flat_selected = 0usize;

    items.push(ListItem::new(Line::from("")));
    items.push(group_header("OmegaOS"));
    for (i, sec) in InfoSection::all().iter().enumerate() {
        let current = i == app.info_section_selected;
        if current {
            flat_selected = items.len();
        }
        items.push(section_row(sec.label(), current, current && list_focused));
    }
    (items, flat_selected)
}

/// Build the Projects-tab left list. Returns (items, flat_selected) — the
/// rendered row index of the cursor for `ListState` scroll tracking.
fn build_projects_list(app: &App) -> (Vec<ListItem<'static>>, usize) {
    let list_focused = !app.detail_focused;
    let mut items: Vec<ListItem> = Vec::new();
    let mut flat_selected = 0usize;

    // Top padding.
    items.push(ListItem::new(Line::from("")));

    // Pinned quick-access row (always selection index 0): jump straight to the
    // AgentikOS OS suite tab without cycling tabs. Visually distinct from a real
    // project (a grid glyph + accent), and it participates in arrow-nav so the
    // cursor can land on it. Projects therefore live at selection index 1..=N.
    {
        let current = app.projects_selected == 0;
        if current {
            flat_selected = items.len();
        }
        let selected_here = current && list_focused;
        let style = if selected_here {
            Style::default()
                .fg(th::accent())
                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default()
                .fg(th::accent())
                .add_modifier(Modifier::BOLD)
        };
        items.push(ListItem::new(Line::from(Span::styled(
            "  ▦ OS System   → the AgentikOS suite".to_string(),
            style,
        ))));
        items.push(ListItem::new(Line::from("")));
    }

    // Grouped under thematic sub-headers derived per-machine from each project's
    // folder under the user's configured projects root (or an explicit category).
    // Selection is by registry index + 1 (the pinned row above holds index 0),
    // so the registry is kept sorted in this same category order (see
    // ManagedProject::category_rank) and arrow-nav flows top-to-bottom.
    items.push(group_header("Projects"));
    if app.project_registry.poisoned {
        items.push(section_row(
            "INVALID registry — repair projects.json".to_string(),
            false,
            false,
        ));
    } else if app.project_registry.projects.is_empty() {
        // The pinned OS row above owns the selection on an empty registry, so
        // this placeholder is informational only (never marked current).
        items.push(section_row(
            "(no projects — press n to add)".to_string(),
            false,
            false,
        ));
    } else {
        use omega_core::project_manager::ManagedProject;
        let root = app.config.projects_dir.as_path();
        // Distinct categories present, sorted (named alphabetically, Other last).
        let mut cats: Vec<String> = Vec::new();
        for p in &app.project_registry.projects {
            let c = p.display_category(root);
            if !cats.contains(&c) {
                cats.push(c);
            }
        }
        cats.sort_by_key(|c| ManagedProject::category_rank(c));

        for cat in &cats {
            // Sub-header (dim, indented one level under the "Projects" header).
            items.push(ListItem::new(Line::from(Span::styled(
                format!("    ── {} ──", cat),
                Style::default().fg(th::dim()).add_modifier(Modifier::BOLD),
            ))));
            for (i, project) in app.project_registry.projects.iter().enumerate() {
                if &project.display_category(root) != cat {
                    continue;
                }
                // Selection index 0 is the pinned OS row, so project `i` is
                // selected at projects_selected == i + 1.
                let current = i + 1 == app.projects_selected;
                if current {
                    flat_selected = items.len();
                }
                // 🔕 marks a project whose Telegram toggle is OFF.
                let tg_mark = if project.telegram_enabled() {
                    ""
                } else {
                    " 🔕"
                };
                items.push(section_row(
                    format!("{}{}", project.name, tg_mark),
                    current,
                    current && list_focused,
                ));
            }
        }
    }

    (items, flat_selected)
}

fn render_project_detail(app: &App) -> Vec<Line<'static>> {
    if app.project_registry.poisoned {
        return vec![
            Line::from(""),
            Line::from(Span::styled(
                "  INVALID project registry",
                Style::default()
                    .fg(th::error())
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!(
                    "  Repair {} before registering, deleting, or dispatching projects.",
                    omega_core::project_manager::ProjectRegistry::registry_path().display()
                ),
                Style::default().fg(th::error()),
            )),
        ];
    }
    let Some(project) = app.selected_project() else {
        return vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No project selected.",
                Style::default().fg(th::dim()),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  ▶ Press n to add a project (register an existing folder).",
                Style::default()
                    .fg(th::success())
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "    Or press Enter on this empty list to do the same.",
                Style::default().fg(th::dim()),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  For a brand-new project (scaffold + provision), use the Menu tab → New project.",
                Style::default().fg(th::dim()),
            )),
        ];
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  ".to_string()),
            Span::styled(
                project.name.clone(),
                Style::default()
                    .fg(th::accent())
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    // Path
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {:20}", "Path"),
            Style::default().fg(th::accent2()),
        ),
        Span::raw(project.path.to_string_lossy().to_string()),
    ]));

    // Git email
    if let Some(ref email) = project.git_email {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:20}", "Git email"),
                Style::default().fg(th::accent2()),
            ),
            Span::raw(email.clone()),
        ]));
    }

    // Telegram topic
    if let Some(topic_id) = project.telegram_topic_id {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:20}", "Telegram topic"),
                Style::default().fg(th::accent2()),
            ),
            Span::raw(topic_id.to_string()),
        ]));
    }

    // Oracle session
    if let Some(ref oracle) = project.oracle_session {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:20}", "Oracle session"),
                Style::default().fg(th::accent2()),
            ),
            Span::raw(oracle.clone()),
        ]));
    }

    // Created at
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {:20}", "Created"),
            Style::default().fg(th::accent2()),
        ),
        Span::raw(project.created_at.clone()),
    ]));

    lines.push(Line::from(""));

    // Planner + bootstrap status (if .planner/tracker.json etc. exist).
    // Per-frame JSON read + parse → RENDER_TTL memo, keyed by the project
    // path so switching the selection refreshes immediately.
    thread_local! {
        #[allow(clippy::type_complexity)]
        static PROJECT_MEMO: std::cell::RefCell<
            Option<(
                std::time::Instant,
                std::path::PathBuf,
                (
                    Option<omega_core::planner::PlanTracker>,
                    Option<omega_core::bootstrap::BootstrapState>,
                ),
            )>,
        > = const { std::cell::RefCell::new(None) };
    }
    let (tracker, bootstrap) = PROJECT_MEMO.with(|cell| {
        let mut cached = cell.borrow_mut();
        if let Some((at, path, v)) = cached.as_ref() {
            if *path == project.path && at.elapsed() < RENDER_TTL {
                return v.clone();
            }
        }
        let v = (
            omega_core::planner::PlanTracker::load(&project.path),
            omega_core::bootstrap::BootstrapState::load(&project.path),
        );
        *cached = Some((std::time::Instant::now(), project.path.clone(), v.clone()));
        v
    });
    if let Some(ref tracker) = tracker {
        let status = tracker.status();
        lines.push(Line::from(Span::styled(
            "  ── Planner ──",
            Style::default()
                .fg(th::accent2())
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(format!(
            "    Phase {}/{} — {:.0}% complete ({}/{} steps)",
            status.active_phase,
            status.total_phases,
            status.progress_pct(),
            status.done,
            status.total,
        )));
        if status.in_progress > 0 {
            lines.push(Line::from(Span::styled(
                format!("    {} in progress", status.in_progress),
                Style::default().fg(th::accent2()),
            )));
        }
        if status.failed > 0 {
            lines.push(Line::from(Span::styled(
                format!("    {} failed", status.failed),
                Style::default().fg(th::error()),
            )));
        }
        if status.ready > 0 {
            lines.push(Line::from(Span::styled(
                format!("    {} ready to start", status.ready),
                Style::default().fg(th::success()),
            )));
        }

        // Show phases
        lines.push(Line::from(""));
        for phase in &tracker.phases {
            let phase_done = phase.step_ids.iter().all(|sid| {
                tracker
                    .get_step(sid)
                    .map(|s| s.status == omega_core::planner::StepStatus::Done)
                    .unwrap_or(false)
            });
            let phase_icon = if phase_done {
                "[+]"
            } else if phase.id == status.active_phase {
                "[~]"
            } else {
                "[ ]"
            };
            lines.push(Line::from(format!(
                "    {} Phase {}: {}",
                phase_icon, phase.id, phase.name
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "  No planner active — press p or run /omg-planner to create a plan.",
            Style::default().fg(th::dim()),
        )));
    }

    // Bootstrap status (read in the PROJECT_MEMO block above)
    if let Some(ref state) = bootstrap {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  ── Bootstrap Pipeline ──",
            Style::default()
                .fg(th::accent2())
                .add_modifier(Modifier::BOLD),
        )));
        for phase in omega_core::bootstrap::BootstrapPhase::all() {
            let done = state.is_done(*phase);
            let active = *phase == state.current_phase && !done;
            let icon = if done {
                "[+]"
            } else if active {
                "[~]"
            } else {
                "[ ]"
            };
            lines.push(Line::from(format!(
                "    {} {} {}",
                icon,
                phase.icon(),
                phase.label()
            )));
        }
    }

    lines.push(Line::from(""));
    // ── ONE actions menu — every project feature, one line each, [key] style. ──
    let tg_on = project.telegram_enabled();
    lines.push(Line::from(Span::styled(
        "  ── Actions ──",
        Style::default()
            .fg(th::accent2())
            .add_modifier(Modifier::BOLD),
    )));
    let action = |key: &str, label: String| -> Line<'static> {
        Line::from(Span::styled(
            format!(
                "    [{}]{}{}",
                key,
                " ".repeat(7usize.saturating_sub(key.len())),
                label
            ),
            Style::default().fg(th::accent()),
        ))
    };
    lines.push(action("Enter", "Open in terminal".to_string()));
    lines.push(action("d", "Dispatch oracle".to_string()));
    lines.push(action("p", "Create plan with /omg-planner".to_string()));
    lines.push(action(
        "T",
        format!(
            "Telegram topic: {} → toggle {}",
            if tg_on { "🔔 ON" } else { "🔕 OFF" },
            if tg_on { "OFF" } else { "ON" },
        ),
    ));
    lines.push(action(
        "x",
        "Delete… (1 OmegaOS · 2 + local folder · 3 + GitHub)".to_string(),
    ));
    lines.push(action(
        "D",
        "Quick delete local machine (press twice)".to_string(),
    ));
    lines.push(action("n", "Add another project".to_string()));

    lines
}

fn draw_settings(frame: &mut Frame, app: &mut App, area: Rect) {
    // Detail panel + the focused-line for auto-scroll depend on which group the
    // cursor sits in: the Monitor group renders the monitor detail, the Settings
    // group renders the provider fields.
    let on_monitor = app.settings_on_monitor();
    let providers = app.providers();
    // Inner text width of the detail panel — gallery rows ellipsize their
    // blurbs against it instead of hard-clipping mid-word at the border.
    // (-2 borders, -1 margin for percentage-split rounding.)
    let detail_inner_w = if app.detail_fullscreen || area.width < TWO_COLUMN_MIN_WIDTH {
        area.width.saturating_sub(2)
    } else {
        (area.width.saturating_mul(75) / 100).saturating_sub(3)
    } as usize;
    let (lines, selected_field_line) = if on_monitor {
        render_monitor_detail(app)
    } else {
        render_settings_detail(app, &providers, detail_inner_w)
    };
    let section_label: String = if on_monitor {
        app.selected_monitor_section().label().to_string()
    } else {
        app.selected_settings_section().label().to_string()
    };

    // Auto-scroll to keep the selected field visible
    if app.detail_focused {
        let panel_height = area.height.saturating_sub(2);
        let field_line = selected_field_line as u16;
        if field_line < app.detail_scroll {
            app.detail_scroll = field_line.saturating_sub(1);
        } else if field_line >= app.detail_scroll + panel_height {
            app.detail_scroll = field_line.saturating_sub(panel_height.saturating_sub(2));
        }
    }

    // Publish the real clamp bound (the preview_max_scroll contract) and pin
    // the offset to it: End / wheel-overscroll stop at the last content line
    // instead of scrolling the Paragraph into blank space. The horizontal
    // split below keeps the full height, so area.height is the panel height
    // in both the split and fullscreen paths.
    app.detail_max_scroll = (lines.len() as u16).saturating_sub(area.height.saturating_sub(2));
    app.detail_scroll = app.detail_scroll.min(app.detail_max_scroll);

    // Fullscreen detail mode: skip the left list, detail takes 100% width
    if app.detail_fullscreen {
        let title = format!(" {}  [FULLSCREEN — Tab/Tab-Tab to exit] ", section_label);
        let paragraph = Paragraph::new(lines).scroll((app.detail_scroll, 0)).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(th::accent2())),
        );
        frame.render_widget(paragraph, area);
        return;
    }

    let list_focused = !app.detail_focused;
    let (items, rendered_selected) = build_settings_list(app);

    // At phone widths the split leaves an unusable 15-column settings list.
    // Match the other list/detail tabs: render only the focused half and use
    // Tab to move between them.
    if area.width < TWO_COLUMN_MIN_WIDTH {
        if list_focused {
            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} — Tab → read ", section_label))
                    .border_style(Style::default().fg(th::accent())),
            );
            let mut state = ListState::default().with_selected(Some(rendered_selected));
            frame.render_stateful_widget(list, area, &mut state);
        } else {
            let paragraph = Paragraph::new(lines).scroll((app.detail_scroll, 0)).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} — Tab → list ", section_label))
                    .border_style(Style::default().fg(th::accent2())),
            );
            frame.render_widget(paragraph, area);
        }
        return;
    }

    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(area);

    let list_border = if list_focused {
        th::accent()
    } else {
        th::dim()
    };
    let detail_border = if app.detail_focused {
        th::accent2()
    } else {
        th::dim()
    };

    // ── Left: grouped section list (Monitor group + Settings group) ──────────
    let list_title = if list_focused {
        " ▶ FOCUSED Settings — ↑/↓ select, Tab → focus detail "
    } else {
        " Settings — Tab to focus list "
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(list_title)
                .border_style(Style::default().fg(list_border)),
        )
        .highlight_style(Style::default());

    let mut settings_list_state = ListState::default().with_selected(Some(rendered_selected));
    frame.render_stateful_widget(list, split[0], &mut settings_list_state);

    // ── Right: details panel ────────────────────────────────────────────────
    let detail_title = if app.detail_focused {
        format!(
            " {}  [FOCUSED — ↑/↓ navigate, Tab → list, Tab-Tab → fullscreen] ",
            section_label
        )
    } else {
        format!(" {} ", section_label)
    };

    let paragraph = Paragraph::new(lines).scroll((app.detail_scroll, 0)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(detail_title)
            .border_style(Style::default().fg(detail_border)),
    );
    frame.render_widget(paragraph, split[1]);
}

/// Returns (lines, selected_field_line) — the line index where the selected field starts.
/// `inner_w` is the detail panel's inner text width (gallery blurb budget).
fn render_settings_detail(
    app: &App,
    providers: &omega_core::providers::ProvidersConfig,
    inner_w: usize,
) -> (Vec<Line<'static>>, usize) {
    use crate::app::{fields_for_section, SettingsField};
    let fields = fields_for_section(app.selected_settings_section(), providers, &app.config);
    let mut lines: Vec<Line> = vec![Line::from("")];
    let mut selected_line: usize = 0;

    let detail_active = app.detail_focused;

    // Top hint
    if detail_active {
        lines.push(Line::from(Span::styled(
            "  ↑/↓ navigate · Enter activates · x clear field (2×) · Tab → back to list",
            Style::default()
                .fg(th::accent2())
                .add_modifier(Modifier::BOLD),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "  Tab → focus this panel to interact (Install/Uninstall/Edit)",
            Style::default().fg(th::dim()),
        )));
    }
    lines.push(Line::from(""));

    for (i, field) in fields.iter().enumerate() {
        let is_selected = detail_active && i == app.settings_field_selected;
        if is_selected {
            selected_line = lines.len();
        }
        let prefix = if is_selected { "  ▶ " } else { "    " };
        match field {
            SettingsField::Action { label, command, .. } => {
                let label_style = if is_selected {
                    Style::default()
                        .fg(th::sel_fg())
                        .bg(th::accent())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(th::accent())
                        .add_modifier(Modifier::BOLD)
                };
                lines.push(Line::from(vec![
                    Span::raw(prefix.to_string()),
                    Span::styled(label.clone(), label_style),
                ]));
                if is_selected {
                    let cmd_preview = if command.chars().count() > 100 {
                        // Truncate by chars, not bytes: `&command[..100]` panics
                        // if byte 100 splits a multi-byte UTF-8 char.
                        format!("{}…", command.chars().take(100).collect::<String>())
                    } else {
                        command.clone()
                    };
                    lines.push(Line::from(Span::styled(
                        format!("      → Enter runs:  {}", cmd_preview),
                        Style::default().fg(th::dim()),
                    )));
                }
            }
            SettingsField::EditText {
                label,
                current_value,
                masked,
                ..
            } => {
                let display = if current_value.is_empty() {
                    "(not set)".to_string()
                } else if *masked {
                    mask_key(current_value)
                } else {
                    current_value.clone()
                };
                let label_style = if is_selected {
                    Style::default()
                        .fg(th::sel_fg())
                        .bg(th::accent2())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(th::text())
                };
                lines.push(Line::from(vec![
                    Span::raw(prefix.to_string()),
                    Span::styled(format!("{:38}", label), label_style),
                    Span::styled(display, Style::default().fg(th::accent())),
                ]));
                if is_selected {
                    lines.push(Line::from(Span::styled(
                        "      → Enter to edit (opens input modal)",
                        Style::default().fg(th::dim()),
                    )));
                }
            }
            SettingsField::Toggle { label, current, .. } => {
                let badge = if *current { "● on" } else { "○ off" };
                let badge_color = if *current { th::success() } else { th::dim() };
                let label_style = if is_selected {
                    Style::default()
                        .fg(th::sel_fg())
                        .bg(th::accent2())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(th::text())
                };
                lines.push(Line::from(vec![
                    Span::raw(prefix.to_string()),
                    Span::styled(format!("{:38}", label), label_style),
                    Span::styled(
                        badge.to_string(),
                        Style::default()
                            .fg(badge_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
                if is_selected {
                    lines.push(Line::from(Span::styled(
                        "      → Enter to toggle",
                        Style::default().fg(th::dim()),
                    )));
                }
            }
            SettingsField::Select {
                label,
                options,
                current_index,
                ..
            } => {
                let display = options
                    .get(*current_index)
                    .cloned()
                    .unwrap_or_else(|| "(not set)".to_string());
                let label_style = if is_selected {
                    Style::default()
                        .fg(th::sel_fg())
                        .bg(th::accent2())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(th::text())
                };
                lines.push(Line::from(vec![
                    Span::raw(prefix.to_string()),
                    Span::styled(format!("{:38}", label), label_style),
                    Span::styled(
                        display,
                        Style::default()
                            .fg(th::accent())
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
                if is_selected {
                    lines.push(Line::from(Span::styled(
                        "      → Enter to choose (↑/↓ selector, no typing)",
                        Style::default().fg(th::dim()),
                    )));
                }
            }
            SettingsField::Info(text) => {
                if text.is_empty() {
                    lines.push(Line::from(""));
                } else {
                    lines.push(Line::from(Span::styled(
                        format!("    {}", text),
                        Style::default().fg(th::dim()),
                    )));
                }
            }
        }
    }

    // Theme gallery: one swatch line per theme, each painted with its OWN
    // palette so the user sees every option at a glance. The Select overlay
    // (Enter on "Active theme") live-previews the whole TUI while arrowing.
    if app.selected_settings_section() == crate::app::SettingsSection::Theme {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Gallery — Enter on \"Active theme\" opens the selector; \u{2191}/\u{2193} previews live",
            Style::default().fg(th::dim()),
        )));
        lines.push(Line::from(""));
        let active = crate::theme::active();
        for id in crate::theme::ThemeId::all() {
            let p = id.palette();
            // Paint each row on ITS OWN theme background so the gallery shows
            // the real contrast between palettes, not just accent swaps.
            let row_bg = p.bg.unwrap_or(Color::Reset);
            let on = |c: Color| Style::default().fg(c).bg(row_bg);
            let marker = if *id == active { "  \u{25b6} " } else { "    " };
            let mut spans: Vec<Span> = vec![
                Span::styled(
                    marker.to_string(),
                    on(p.accent).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:28}", id.label()),
                    if *id == active {
                        on(p.accent).add_modifier(Modifier::BOLD)
                    } else {
                        on(p.text)
                    },
                ),
            ];
            // warn replaces success in the strip: on mono themes success IS
            // the accent (duplicate swatch), while warn is a real distinct role.
            for c in [
                p.accent, p.accent2, p.warn, p.error, p.info, p.special, p.dim,
            ] {
                spans.push(Span::styled("\u{2588}\u{2588}", on(c)));
            }
            // Char-safe ellipsis on the blurb: 4 marker + 28 label +
            // 14 swatches + 2 gap = 48 fixed columns before it.
            let blurb_budget = inner_w.saturating_sub(48);
            let blurb = if id.blurb().chars().count() > blurb_budget && blurb_budget > 1 {
                format!(
                    "{}…",
                    id.blurb()
                        .chars()
                        .take(blurb_budget - 1)
                        .collect::<String>()
                )
            } else {
                id.blurb().to_string()
            };
            spans.push(Span::styled(format!("  {}", blurb), on(p.dim)));
            lines.push(Line::from(spans));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "  Config files: {}  {}",
            omega_core::config::omega_dir()
                .join("config.toml")
                .display(),
            omega_core::config::omega_dir()
                .join("providers.toml")
                .display()
        ),
        Style::default().fg(th::dim()),
    )));
    (lines, selected_line)
}

/// Mask sensitive characters in an inline input (show prefix/suffix only).
fn mask_inline(s: &str) -> String {
    // Count chars, not bytes: byte slicing/length panics or miscounts on
    // multi-byte UTF-8 (emoji, accented, CJK).
    let char_count = s.chars().count();
    if char_count <= 6 {
        "•".repeat(char_count)
    } else {
        // char-boundary-safe: `&s[..3]` panics if byte 3 splits a multi-byte
        // char. Take 3 chars, not 3 bytes.
        let prefix: String = s.chars().take(3).collect();
        format!("{}…{}", prefix, "•".repeat(char_count - 3))
    }
}

fn mask_key(key: &str) -> String {
    // Count/slice by chars, not bytes: `&key[..4]` and `&key[key.len()-4..]`
    // panic if a byte index splits a multi-byte UTF-8 char (emoji, CJK).
    let char_count = key.chars().count();
    if key.is_empty() {
        "(not set)".to_string()
    } else if char_count <= 8 {
        "•".repeat(char_count)
    } else {
        let prefix: String = key.chars().take(4).collect();
        let suffix: String = key.chars().skip(char_count - 4).collect();
        format!("{}…{}", prefix, suffix)
    }
}

/// Projects tab — the project list and the selected project's detail.
fn draw_projects(frame: &mut Frame, app: &mut App, area: Rect) {
    let lines = render_project_detail(app);
    let label = app
        .selected_project()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "Projects".to_string());
    let (items, rendered_selected) = build_projects_list(app);
    draw_two_column(
        frame,
        app,
        area,
        TwoColumn {
            items,
            rendered_selected,
            lines,
            scroll_target: 0,
            section_label: label,
            list_focused_title: " ▶ FOCUSED Projects — ↑/↓ select, Tab → focus detail ",
            list_title: " Projects — Tab to focus list ",
        },
    );
}

/// System tab — the doctrine surface: what OmegaOS is, the Laws and Rules that
/// bind every agent, the AI agent roster, the orchestration layers, the
/// installed skills, and the whole manual.
fn draw_system(frame: &mut Frame, app: &mut App, area: Rect) {
    let (lines, scroll_target) = match app.selected_info_section() {
        InfoSection::Overview => (render_info_overview(app), 0),
        InfoSection::Laws => (render_info_laws(), 0),
        InfoSection::Rules => (render_info_rules(), 0),
        InfoSection::AisbAgents => render_info_aisb_agents(app),
        InfoSection::Atlas => (render_info_atlas(), 0),
        InfoSection::Oracle => (render_info_oracle(), 0),
        InfoSection::Workers => (render_info_workers(), 0),
        InfoSection::Skills => (render_info_skills(app), 0),
        InfoSection::Docs => render_info_docs(app),
    };
    let label = app.selected_info_section().label();
    let (items, rendered_selected) = build_system_list(app);
    draw_two_column(
        frame,
        app,
        area,
        TwoColumn {
            items,
            rendered_selected,
            lines,
            scroll_target,
            section_label: label,
            list_focused_title: " ▶ FOCUSED System — ↑/↓ select, Tab → focus detail ",
            list_title: " System — Tab to focus list ",
        },
    );
}

/// Everything the shared 25/75 list+detail shell needs to render one frame.
struct TwoColumn<'a> {
    items: Vec<ListItem<'a>>,
    /// Rendered row index of the left cursor, for `ListState` scroll tracking.
    rendered_selected: usize,
    lines: Vec<Line<'a>>,
    /// Line the right panel must keep visible (a sub-list cursor); 0 = none.
    scroll_target: usize,
    section_label: String,
    list_focused_title: &'a str,
    list_title: &'a str,
}

/// The 25/75 list+detail shell shared by the Projects, System, and OS tabs,
/// with the fullscreen mode, responsive collapse, scroll-bound contract, and
/// focus borders.
fn draw_two_column(frame: &mut Frame, app: &mut App, area: Rect, col: TwoColumn) {
    let TwoColumn {
        items,
        rendered_selected,
        lines,
        scroll_target,
        section_label,
        list_focused_title,
        list_title,
    } = col;

    // Auto-scroll to keep the just-moved sub-cursor visible (agent list,
    // document list) — once, on the frame after the move. Re-snapping every
    // frame pinned the panel to the list and made a document's BODY, which
    // renders below it, impossible to scroll into.
    if app.detail_focused && app.detail_follow_cursor && scroll_target > 0 {
        let panel_h = area.height.saturating_sub(2);
        let target = scroll_target as u16;
        if target < app.detail_scroll {
            app.detail_scroll = target.saturating_sub(1);
        } else if target >= app.detail_scroll + panel_h {
            app.detail_scroll = target.saturating_sub(panel_h.saturating_sub(2));
        }
        app.detail_follow_cursor = false;
    }

    // The panel wraps (below), so the scroll bound must count the rows actually
    // painted, not the logical lines. Counting logical lines let End stop short
    // of the end of a wrapped document — the tail was unreachable.
    let detail_outer_width = if app.detail_fullscreen || area.width < TWO_COLUMN_MIN_WIDTH {
        area.width
    } else {
        // Mirrors the 25/75 split below.
        (area.width as u32 * 75 / 100) as u16
    };
    // The paragraph's border consumes exactly two columns. The narrow and
    // fullscreen branches used to subtract those columns both above and here,
    // overestimating wrapped rows and allowing scroll into blank space.
    let detail_width = detail_outer_width.saturating_sub(2).max(1);
    let painted_rows = wrapped_row_count(&lines, detail_width);

    // Publish + pin the scroll bound (same contract as draw_settings): End
    // and wheel-overscroll stop at the content edge, never a blank panel.
    app.detail_max_scroll = painted_rows.saturating_sub(area.height.saturating_sub(2));
    app.detail_scroll = app.detail_scroll.min(app.detail_max_scroll);

    // Fullscreen detail
    if app.detail_fullscreen {
        let title = format!(" {}  [FULLSCREEN — Tab/Tab-Tab to exit] ", section_label);
        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((app.detail_scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(th::accent2())),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    // Below this, 25% of the width is ~15 columns: every section label was
    // clipped mid-word ("Documentati", "AI Agents (") and the detail was just
    // as cramped. Narrow terminals — a phone in portrait is ~60 columns — get
    // ONE column: whichever half has focus, at full width. Tab already toggles
    // that focus, so it becomes the way to move between them, no new key.
    if area.width < TWO_COLUMN_MIN_WIDTH {
        let list_focused = !app.detail_focused;
        if list_focused {
            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} — Tab → read ", section_label))
                    .border_style(Style::default().fg(th::accent())),
            );
            let mut state = ListState::default().with_selected(Some(rendered_selected));
            frame.render_stateful_widget(list, area, &mut state);
        } else {
            let paragraph = Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .scroll((app.detail_scroll, 0))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(" {} — Tab → list ", section_label))
                        .border_style(Style::default().fg(th::accent2())),
                );
            frame.render_widget(paragraph, area);
        }
        return;
    }

    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(area);

    let list_focused = !app.detail_focused;
    let list_border = if list_focused {
        th::accent()
    } else {
        th::dim()
    };
    let detail_border = if app.detail_focused {
        th::accent2()
    } else {
        th::dim()
    };

    let list_title = if list_focused {
        list_focused_title
    } else {
        list_title
    };
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(list_title)
                .border_style(Style::default().fg(list_border)),
        )
        .highlight_style(Style::default());

    let mut info_list_state = ListState::default().with_selected(Some(rendered_selected));
    frame.render_stateful_widget(list, split[0], &mut info_list_state);

    let detail_title = if app.detail_focused {
        format!(
            " {}  [FOCUSED — ↑/↓ scroll, Tab → list, Tab-Tab → fullscreen] ",
            section_label
        )
    } else {
        format!(" {} ", section_label)
    };

    let paragraph = Paragraph::new(lines)
        // Wrapped, not clipped: a doc line, a rule's reason or a long path used
        // to lose its tail at the panel edge with no way to see it.
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(detail_title)
                .border_style(Style::default().fg(detail_border)),
        );
    frame.render_widget(paragraph, split[1]);
}

/// Rows a set of lines actually occupies once wrapped to `width` — the count
/// the scroll bound has to use, since a wrapped panel paints more rows than it
/// has logical lines.
fn wrapped_row_count(lines: &[Line<'_>], width: u16) -> u16 {
    use std::collections::VecDeque;
    use unicode_width::UnicodeWidthChar;

    let max_width = usize::from(width.max(1));
    let mut total_rows = 0usize;

    for line in lines {
        // Count the same word-boundary wrapping used by `Wrap { trim: false }`.
        // Ratatui's exact `Paragraph::line_count` is feature-gated/private in
        // this build, so keep the small width-only state machine here instead
        // of guessing with `display_width.div_ceil(panel_width)`.
        let mut rows = 0usize;
        let mut line_width = 0usize;
        let mut word_width = 0usize;
        let mut whitespace_width = 0usize;
        let mut pending_line_has_symbols = false;
        let mut pending_word_has_symbols = false;
        let mut pending_whitespace = VecDeque::new();
        let mut previous_was_non_whitespace = false;

        for ch in line.spans.iter().flat_map(|span| span.content.chars()) {
            let symbol_width = ch.width().unwrap_or(0);
            if symbol_width > max_width {
                continue;
            }
            let is_whitespace = ch.is_whitespace();
            let word_found = previous_was_non_whitespace && is_whitespace;
            let untrimmed_overflow = !pending_line_has_symbols
                && word_width + whitespace_width + symbol_width > max_width;

            if word_found || untrimmed_overflow {
                pending_line_has_symbols |= !pending_whitespace.is_empty();
                line_width += whitespace_width;
                pending_line_has_symbols |= pending_word_has_symbols;
                line_width += word_width;
                pending_whitespace.clear();
                whitespace_width = 0;
                word_width = 0;
                pending_word_has_symbols = false;
            }

            let line_full = line_width >= max_width;
            let pending_word_overflow =
                symbol_width > 0 && line_width + whitespace_width + word_width >= max_width;
            if line_full || pending_word_overflow {
                let mut remaining_width = max_width.saturating_sub(line_width);
                rows += 1;
                line_width = 0;
                pending_line_has_symbols = false;

                while let Some(front_width) = pending_whitespace.front().copied() {
                    if front_width > remaining_width {
                        break;
                    }
                    whitespace_width = whitespace_width.saturating_sub(front_width);
                    remaining_width -= front_width;
                    pending_whitespace.pop_front();
                }
                if is_whitespace && pending_whitespace.is_empty() {
                    previous_was_non_whitespace = false;
                    continue;
                }
            }

            if is_whitespace {
                whitespace_width += symbol_width;
                pending_whitespace.push_back(symbol_width);
            } else {
                word_width += symbol_width;
                pending_word_has_symbols = true;
            }
            previous_was_non_whitespace = !is_whitespace;
        }

        if !pending_line_has_symbols && !pending_word_has_symbols && !pending_whitespace.is_empty()
        {
            rows += 1;
        }
        pending_line_has_symbols |= !pending_whitespace.is_empty();
        pending_line_has_symbols |= pending_word_has_symbols;
        if pending_line_has_symbols {
            rows += 1;
        }
        total_rows = total_rows.saturating_add(rows.max(1));
        if total_rows >= usize::from(u16::MAX) {
            return u16::MAX;
        }
    }

    total_rows as u16
}

/// OS tab — 25/75 split: left = the AgentikOS
/// operative-systems suite (glyph + name), right = the selected OS's detail
/// (tagline, status, path, integration pipeline + actions). Registry + fs stat
/// only (see `omega_core::os_products`) — no network, safe per tab entry / F5.
fn draw_os(frame: &mut Frame, app: &mut App, area: Rect) {
    let list_focused = !app.detail_focused;

    // ── Left: the suite, grouped — Personal, Build chain (01..08), Growth, Systems ─
    let mut items: Vec<ListItem> = Vec::new();
    items.push(ListItem::new(Line::from("")));
    let mut rendered_selected = 1usize;
    if app.os_entries.is_empty() {
        items.push(group_header("AgentikOS suite"));
        items.push(section_row(
            "(loading — F5 to rescan)".to_string(),
            true,
            list_focused,
        ));
        rendered_selected = items.len() - 1;
    } else {
        let mut last_group: Option<omega_core::os_products::OsGroup> = None;
        for (i, e) in app.os_entries.iter().enumerate() {
            if last_group != Some(e.product.group) {
                if last_group.is_some() {
                    items.push(ListItem::new(Line::from("")));
                }
                items.push(group_header(match e.product.group {
                    omega_core::os_products::OsGroup::Personal => "Personal",
                    omega_core::os_products::OsGroup::BuildChain => "Build chain",
                    omega_core::os_products::OsGroup::Growth => "Growth",
                    omega_core::os_products::OsGroup::Systems => "Systems & AI",
                }));
                last_group = Some(e.product.group);
            }
            let current = i == app.os_selected;
            if current {
                rendered_selected = items.len();
            }
            let label = match e.product.chain_position() {
                Some(n) => format!("{} {:02} · {}", e.glyph(), n, e.product.name),
                None => format!("{} {}", e.glyph(), e.product.name),
            };
            items.push(section_row(label, current, current && list_focused));
        }
    }

    let lines = render_os_detail(app);
    let section_label = app
        .selected_os_entry()
        .map(|e| e.product.name.to_string())
        .unwrap_or_else(|| "OS".to_string());
    draw_two_column(
        frame,
        app,
        area,
        TwoColumn {
            items,
            rendered_selected,
            lines,
            scroll_target: 0,
            section_label,
            list_focused_title: " ▶ FOCUSED OS — ↑/↓ select, Tab → focus detail ",
            list_title: " OS — Tab to focus list ",
        },
    );
}

/// Right-pane detail for the selected operative system (status + ACTIONS).
fn render_os_detail(app: &App) -> Vec<Line<'static>> {
    let Some(e) = app.selected_os_entry() else {
        return vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No operative system selected.",
                Style::default().fg(th::dim()),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Press F5 to load the AgentikOS suite.",
                Style::default()
                    .fg(th::success())
                    .add_modifier(Modifier::BOLD),
            )),
        ];
    };

    let field = |label: &str| {
        Span::styled(
            format!("  {:20}", label),
            Style::default().fg(th::accent2()),
        )
    };
    use omega_core::os_products::OsReadinessLevel;
    let readiness_color = match e.readiness.level {
        OsReadinessLevel::Scaffold => th::dim(),
        OsReadinessLevel::Reference => th::accent(),
        OsReadinessLevel::Runnable => th::accent2(),
        OsReadinessLevel::Testable => th::special(),
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw(format!("  {} ", e.glyph())),
            Span::styled(
                e.product.name.to_string(),
                Style::default()
                    .fg(th::accent())
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                e.product.tagline.to_string(),
                Style::default().fg(th::text()),
            ),
        ]),
        Line::from(""),
        Line::from(vec![field("Slug"), Span::raw(e.product.slug.to_string())]),
        Line::from(vec![
            field("Readiness"),
            Span::styled(
                e.status_label().to_string(),
                Style::default()
                    .fg(readiness_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            field("Manifest"),
            Span::raw(e.readiness.manifest_label().to_string()),
        ]),
        Line::from(vec![
            field("Master prompt"),
            Span::raw(if e.readiness.master_present {
                "present"
            } else {
                "missing"
            }),
        ]),
        Line::from(vec![
            field("Runtime surface"),
            Span::raw(if e.readiness.runtime_present {
                "present"
            } else {
                "not found"
            }),
        ]),
        Line::from(vec![
            field("Test surface"),
            Span::raw(if e.readiness.tests_present {
                "present (not executed)"
            } else {
                "not found"
            }),
        ]),
        Line::from(vec![
            field("Event schema"),
            Span::raw(
                e.readiness
                    .event_schema_status
                    .clone()
                    .unwrap_or_else(|| "not declared".to_string()),
            ),
        ]),
        Line::from(vec![
            field("Path"),
            match &e.path {
                Some(p) => Span::raw(p.to_string_lossy().to_string()),
                None => Span::styled(
                    "— (no OS/ root found on this machine)".to_string(),
                    Style::default().fg(th::dim()),
                ),
            },
        ]),
        Line::from(vec![
            field("Telegram bot"),
            if e.bot_linked {
                Span::styled(
                    "🤖 linked — DM it, the master agent answers",
                    Style::default()
                        .fg(th::success())
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    "not linked — press T to connect one".to_string(),
                    Style::default().fg(th::dim()),
                )
            },
        ]),
        Line::from(""),
    ];

    // Commands are registry declarations. Presence of a runtime surface makes
    // them plausible to execute, but this static scan never calls them.
    if !e.product.commands.is_empty() {
        lines.push(Line::from(Span::styled(
            if e.readiness.runtime_present {
                "  ─── Declared commands ─── (runtime present; not executed here)"
            } else {
                "  ─── Declared commands ─── (reference only; runtime not found)"
            },
            Style::default()
                .fg(th::accent2())
                .add_modifier(Modifier::BOLD),
        )));
        for cmd in e.product.commands {
            // Sub-lines (indented, starting with a space or an arrow) render
            // dim; command lines render bright so the verbs stand out.
            let dim = cmd.starts_with(' ') || cmd.starts_with('→');
            lines.push(Line::from(Span::styled(
                format!("  {}", cmd),
                Style::default().fg(if dim { th::dim() } else { th::text() }),
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  ─── Readiness gaps ───",
        Style::default()
            .fg(th::accent2())
            .add_modifier(Modifier::BOLD),
    )));
    if !e.readiness.directory_present {
        lines.push(Line::from("  - OS directory is missing on this machine."));
    }
    if !e.readiness.master_present {
        lines.push(Line::from("  - MASTER.md prompt is missing."));
    }
    match e.readiness.manifest {
        omega_core::os_products::OsManifestStatus::Missing => {
            lines.push(Line::from("  - MANIFEST.json is missing."));
        }
        omega_core::os_products::OsManifestStatus::Invalid => {
            lines.push(Line::from(
                "  - MANIFEST.json is invalid or does not match the product contract.",
            ));
        }
        omega_core::os_products::OsManifestStatus::Valid => {}
    }
    if !e.readiness.runtime_present {
        lines.push(Line::from("  - No runtime entrypoint/directory was found."));
    }
    if !e.readiness.tests_present {
        lines.push(Line::from("  - No test surface was found."));
    }
    if e.readiness.event_schema_status.as_deref() == Some("stub") {
        lines.push(Line::from("  - Event schema is explicitly marked stub."));
    }
    if e.readiness.master_present
        && e.readiness.runtime_present
        && e.readiness.tests_present
        && matches!(
            e.readiness.manifest,
            omega_core::os_products::OsManifestStatus::Valid
        )
        && e.readiness.event_schema_status.as_deref() != Some("stub")
    {
        lines.push(Line::from(
            "  No static surface gaps found. Runtime verification is still required.",
        ));
    }

    lines.extend([
        Line::from(""),
        Line::from(Span::styled(
            "  ─── Actions ───",
            Style::default()
                .fg(th::accent2())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "  Enter  ",
                Style::default()
                    .fg(th::accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(if e.readiness.master_present {
                "Open this OS's MASTER.md prompt in an agent session"
            } else {
                "Open the generic OS integration prompt (MASTER.md missing)"
            }),
        ]),
        Line::from(vec![
            Span::styled(
                "  T      ",
                Style::default()
                    .fg(th::accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(if e.bot_linked {
                "🤖 Telegram bot: relink / replace its token"
            } else {
                "🤖 Connect a Telegram bot (the OS master agent answers DMs)"
            }),
        ]),
        Line::from(vec![
            Span::styled(
                "  F5     ",
                Style::default()
                    .fg(th::accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("refresh statuses"),
        ]),
    ]);
    lines.push(Line::from(""));
    lines
}

/// Returns (lines, selected_agent_line) for auto-scroll.
fn render_info_aisb_agents(app: &App) -> (Vec<Line<'static>>, usize) {
    use omega_core::aisb_agents::AisbAgent;
    let agents = AisbAgent::all();
    let selected_def = agents[app.info_agent_selected].definition();

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            // Count derived from the roster (see InfoSection::label) so the
            // blurb can't drift when an agent joins.
            format!(
                "  AISB = AI Super Brain — {} Matrix roles available to the Atlas service.",
                agents.len()
            ),
            Style::default()
                .fg(th::accent())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  ↑/↓ navigate the agent list below.",
            Style::default().fg(th::dim()),
        )),
        Line::from(""),
    ];

    let mut selected_agent_line: usize = 4;
    // Compact list
    for (i, agent) in agents.iter().enumerate() {
        let def = agent.definition();
        let selected = i == app.info_agent_selected;
        if selected {
            selected_agent_line = lines.len();
        }
        let prefix = if selected { "▶ " } else { "  " };
        let name_style = if selected {
            Style::default()
                .fg(th::sel_fg())
                .bg(th::accent2())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(th::accent2())
                .add_modifier(Modifier::BOLD)
        };
        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(th::accent())),
            Span::styled(format!("{:13}", def.name), name_style),
            Span::styled(
                format!(" {} ", def.model.name()),
                Style::default().fg(th::special()),
            ),
            Span::raw(format!("· {}", def.role)),
        ]));
    }

    // Detail card for the selected agent
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  ── {} ──", selected_def.name),
        Style::default()
            .fg(th::accent())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        format!("  \"{}\"", selected_def.tagline),
        Style::default().fg(th::dim()),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(format!("  Role:    {}", selected_def.role)));
    lines.push(Line::from(format!(
        "  Model:   {}",
        selected_def.model.name()
    )));
    lines.push(Line::from(format!(
        "  Tools:   {}",
        selected_def.tools.join(", ")
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Responsibilities:",
        Style::default().fg(th::accent2()),
    )));
    for r in selected_def.responsibilities {
        lines.push(Line::from(format!("    • {}", r)));
    }
    (lines, selected_agent_line)
}

fn render_info_atlas() -> Vec<Line<'static>> {
    let master = omega_core::aisb::MASTER_SESSION_NAME;
    vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Ω  ATLAS — the Director brain (omega-tg-bot.ts)",
            Style::default()
                .fg(th::special())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  Atlas is the single brain reached over Telegram: you message it,"),
        Line::from("  it classifies intent and dispatches to the right oracle, agent or"),
        Line::from("  skill. The 15 Matrix agents (see 'AISB Agents') are its faculties."),
        Line::from("  One conversation, many agents, shared evolution."),
        Line::from(""),
        Line::from(Span::styled(
            "  Live session",
            Style::default().fg(th::accent()),
        )),
        Line::from(format!(
            "    {}  — live viewer of the Atlas Telegram conversation",
            master
        )),
        Line::from("    Select it in the Sessions tab + Tab to watch the live conversation."),
        Line::from(""),
        Line::from(Span::styled(
            "  Telegram bridge",
            Style::default().fg(th::accent()),
        )),
        Line::from("    Set up:  Settings tab → Telegram & projects → Enter (or press 'T')"),
        Line::from("             to connect Telegram (guided wizard — no command needed)."),
        Line::from("    Once set, Atlas streams its replies + accepts voice / documents /"),
        Line::from("    photos (transcribed + analysed). Per-project topics on sync."),
        Line::from(""),
        Line::from(Span::styled(
            "  One brain (Atlas), one Telegram channel — all 15 AISB agents.",
            Style::default().fg(th::dim()),
        )),
    ]
}

fn render_info_oracle() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        Line::from(Span::styled(
            "  ORACLE — the brain of every dispatched mission",
            Style::default()
                .fg(th::accent2())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  An Oracle is spawned by `omega dispatch <Project> \"<mission>\"`."),
        Line::from("  Its job is to:"),
        Line::from("    1. CLASSIFY complexity (SIMPLE / MEDIUM / COMPLEX / EPIC)"),
        Line::from("    2. SCALE the Lab plan — a typo is Understand|Build|Verify, not Deploy"),
        Line::from("    3. WRITE a real R-RUBRIC into every worker brief (no auto-fill)"),
        Line::from("    4. DISPATCH writers (claude|codex|glm) via rmux, file-scope locked"),
        Line::from("    5. TREAT worker done.json as a candidate — re-run Verify Command"),
        Line::from("    6. REPORT — oracle done.json is still not self-approval"),
        Line::from(""),
        Line::from(Span::styled("  Naming", Style::default().fg(th::accent()))),
        Line::from("    Sessions: oracle-<Project>     (1st)"),
        Line::from("              oracle-<Project>-2   (parallel oracle)"),
        Line::from(""),
        Line::from(Span::styled(
            "  Rules enforced",
            Style::default().fg(th::accent()),
        )),
        Line::from("    R-RUBRIC — measurable Done Criteria + Verify Command before spawn"),
        Line::from("    R-VERIFY — a delegate's done is an input, never the verdict"),
        Line::from("    R-BUDGET — 500K token cap; escalate, do not silently overrun"),
        Line::from("    L3       — dispatched sessions decide and proceed"),
        Line::from(""),
        Line::from(Span::styled(
            "  Oracles supervise. Writers cannot self-approve. Fake-done is forbidden.",
            Style::default().fg(th::dim()),
        )),
    ]
}

fn render_info_workers() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        Line::from(Span::styled(
            "  WORKERS — ephemeral execution sessions",
            Style::default()
                .fg(th::accent2())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  A worker is one rmux session running a writer (claude, codex, or glm)"),
        Line::from("  with a specific task prompt. Workers are short-lived: spawned by an"),
        Line::from("  Oracle, they execute one task, signal done, and the patrol cleans up."),
        Line::from(""),
        Line::from(Span::styled("  Naming", Style::default().fg(th::accent()))),
        Line::from("    <Project>-worker-<task>    e.g. Causio-worker-auth"),
        Line::from(""),
        Line::from(Span::styled(
            "  Lifecycle",
            Style::default().fg(th::accent()),
        )),
        Line::from("    1. Oracle writes Done Criteria + Verify Command (R-RUBRIC) — no auto-fill"),
        Line::from("    2. Spawn: omega spawn-worker auth \"<brief>\" --dir <project> --files a,b"),
        Line::from("    3. Scope-claim: files_owned locked in ~/.omega/state/scope-*.json"),
        Line::from("    4. Worker starts in the project dir (never rmux `.` / $HOME)"),
        Line::from("    5. omega done done_clean re-runs the Verify Command"),
        Line::from("    6. Patrol acks; the oracle still verifies (R-VERIFY)"),
        Line::from(""),
        Line::from(Span::styled(
            "  Rules enforced",
            Style::default().fg(th::accent()),
        )),
        Line::from("    L1 / L4      — runtime evidence; done means 100%"),
        Line::from("    R-VERIFY     — worker done.json is a candidate, not a verdict"),
        Line::from("    SCOPE-CLAIM  — no two workers may edit the same file"),
        Line::from(""),
        Line::from("  Workers can run in PARALLEL when their file scopes are disjoint."),
        Line::from("  When they overlap, the dispatcher serializes them automatically."),
    ]
}

/// The Laws — their own section, because they outrank every rule and reading
/// them should not mean scrolling past forty rules first.
fn render_info_laws() -> Vec<Line<'static>> {
    use omega_core::rules::laws;
    let law_list = laws();
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  THE LAWS — inviolable, bind every agent, override every rule and every task.",
            Style::default()
                .fg(th::special())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!(
                "  {} laws · injected into every dispatched agent · `omega rules list`",
                law_list.len()
            ),
            Style::default().fg(th::dim()),
        )),
        Line::from(""),
    ];

    for r in &law_list {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:14}", r.id),
                Style::default()
                    .fg(th::special())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                r.title.to_string(),
                Style::default().fg(th::text()).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            format!("    {}", r.description),
            Style::default().fg(th::text()),
        )));
        let applies = if r.applies_to.is_empty() {
            "all agents".to_string()
        } else {
            r.applies_to
                .iter()
                .map(|a| a.name())
                .collect::<Vec<_>>()
                .join(", ")
        };
        lines.push(Line::from(Span::styled(
            format!("    Applies to: {}  ·  Added: {}", applies, r.added_at),
            Style::default().fg(th::dim()),
        )));
        lines.push(Line::from(Span::styled(
            format!("    Why: {}", r.reason),
            Style::default().fg(th::dim()),
        )));
        lines.push(Line::from(""));
    }
    lines
}

fn render_info_rules() -> Vec<Line<'static>> {
    use omega_core::rules::{all_rules, RuleCategory, RuleKind};
    let mut lines =
        vec![
        Line::from(""),
        Line::from(Span::styled(
            "  System invariants — every rule has a reason, a date, and who it binds.",
            Style::default().fg(th::accent()).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  Rules implement the Laws in practice. See the Laws section for what outranks them.",
            Style::default().fg(th::dim()),
        )),
        Line::from(""),
    ];

    let categories = [
        RuleCategory::Universal,
        RuleCategory::QualityGate,
        RuleCategory::Orchestration,
        RuleCategory::Reporting,
        RuleCategory::Safety,
    ];

    for cat in &categories {
        let rules: Vec<_> = all_rules()
            .into_iter()
            .filter(|r| r.kind == RuleKind::Rule && r.category == *cat)
            .collect();
        if rules.is_empty() {
            continue;
        }
        lines.push(Line::from(Span::styled(
            format!("  ── {} ──", cat.label()),
            Style::default()
                .fg(th::accent2())
                .add_modifier(Modifier::BOLD),
        )));
        for r in rules {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:14}", r.id),
                    Style::default()
                        .fg(th::accent())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(r.title.to_string()),
            ]));
            lines.push(Line::from(Span::styled(
                format!("    {}", r.description),
                Style::default().fg(th::text()),
            )));
            let applies = if r.applies_to.is_empty() {
                "all agents".to_string()
            } else {
                r.applies_to
                    .iter()
                    .map(|a| a.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            lines.push(Line::from(Span::styled(
                format!("    Applies to: {}  ·  Added: {}", applies, r.added_at),
                Style::default().fg(th::dim()),
            )));
            lines.push(Line::from(Span::styled(
                format!("    Why: {}", r.reason),
                Style::default().fg(th::dim()),
            )));
            lines.push(Line::from(""));
        }
    }
    lines
}

/// Overview — the one screen that answers "what am I running?": the four
/// orchestration levels, every registry's live count, and where things live.
fn render_info_overview(app: &App) -> Vec<Line<'static>> {
    use omega_core::rules::{all_rules, laws, RuleKind};

    let head = Style::default()
        .fg(th::accent())
        .add_modifier(Modifier::BOLD);
    let sub = Style::default()
        .fg(th::accent2())
        .add_modifier(Modifier::BOLD);
    let dim = Style::default().fg(th::dim());

    let rule_count = all_rules()
        .iter()
        .filter(|r| r.kind == RuleKind::Rule)
        .count();
    let agent_count = omega_core::aisb_agents::AisbAgent::all().len();
    let omega_dir = omega_core::config::omega_dir();

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  Ω  OmegaOS v{}", env!("CARGO_PKG_VERSION")),
            head,
        )),
        Line::from(Span::styled(
            "  An agentic terminal operating system — a multi-agent development platform",
            Style::default().fg(th::text()),
        )),
        Line::from(Span::styled(
            "  running on rmux, driven by Laws and Rules that bind every agent it dispatches.",
            Style::default().fg(th::text()),
        )),
        Line::from(""),
        Line::from(Span::styled("  ── The four levels ──", sub)),
        Line::from(""),
    ];

    for (level, title, detail) in [
        (
            "Level 1",
            "Human interface",
            "Telegram · CLI · this TUI — you state intent",
        ),
        (
            "Level 2",
            "Atlas / AISB",
            "the Director brain classifies and dispatches",
        ),
        (
            "Level 3",
            "Oracle",
            "one per project, strategic — decomposes and delegates",
        ),
        (
            "Level 4",
            "Workers",
            "ephemeral, parallel, file-scoped — execute, verify, report",
        ),
    ] {
        lines.push(Line::from(vec![
            Span::styled(format!("  {:9}", level), Style::default().fg(th::special())),
            Span::styled(
                format!("{:18}", title),
                Style::default().fg(th::text()).add_modifier(Modifier::BOLD),
            ),
            Span::styled(detail.to_string(), dim),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  ── What is loaded right now ──",
        sub,
    )));
    lines.push(Line::from(""));

    // Counts come from the live registries — a hardcoded number here is a
    // number that goes stale the next time a law or a skill is added.
    let skills_value = app
        .skills_error
        .as_ref()
        .map(|_| "INVALID".to_string())
        .unwrap_or_else(|| app.skills.len().to_string());
    let projects_value = if app.project_registry.poisoned {
        "INVALID".to_string()
    } else {
        app.project_registry.projects.len().to_string()
    };
    for (label, value, hint) in [
        (
            "Laws",
            laws().len().to_string(),
            "inviolable, injected into every agent",
        ),
        (
            "Rules",
            rule_count.to_string(),
            "operational, scoped per agent level",
        ),
        (
            "AI agents",
            agent_count.to_string(),
            "the AISB Matrix roster",
        ),
        ("Skills", skills_value, "installed under ~/.omega/skills"),
        (
            "Documents",
            app.docs.len().to_string(),
            "the manual, under ~/.omega/docs",
        ),
        ("Projects", projects_value, "registered in the Projects tab"),
    ] {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:12}", label),
                Style::default().fg(th::accent2()),
            ),
            Span::styled(
                format!("{:>5}  ", value),
                Style::default().fg(th::text()).add_modifier(Modifier::BOLD),
            ),
            Span::styled(hint.to_string(), dim),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  ── Where it lives ──", sub)));
    lines.push(Line::from(""));
    for (label, path) in [
        ("State + secrets", omega_dir.to_string_lossy().to_string()),
        (
            "Skills",
            omega_dir.join("skills").to_string_lossy().to_string(),
        ),
        (
            "Manual",
            omega_core::docs::docs_dir().to_string_lossy().to_string(),
        ),
        (
            "Projects root",
            app.config.projects_dir.to_string_lossy().to_string(),
        ),
    ] {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:16}", label),
                Style::default().fg(th::accent2()),
            ),
            Span::styled(path, dim),
        ]));
    }

    // ── Staying current ───────────────────────────────────────────────────
    // What the nightly update cron is allowed to do and what it last did. This
    // is the only place in the UI that answers "is this box keeping itself up
    // to date?", so it names the policy, the schedule and the last outcome.
    use omega_core::config::AutoUpdatePolicy;
    let st = &app.auto_update_state;
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  ── Staying current ──", sub)));
    lines.push(Line::from(""));

    let (policy_text, policy_color) = match app.config.auto_update {
        AutoUpdatePolicy::Apply => ("apply — checks daily at 03:30 and installs", th::success()),
        AutoUpdatePolicy::Check => ("check — checks daily at 03:30, alerts only", th::accent2()),
        AutoUpdatePolicy::Off => ("off — no automatic check", th::dim()),
    };
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {:16}", "Auto-update"),
            Style::default().fg(th::accent2()),
        ),
        Span::styled(
            policy_text.to_string(),
            Style::default()
                .fg(policy_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

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

    lines.push(Line::from(vec![
        Span::styled(
            format!("  {:16}", "Last check"),
            Style::default().fg(th::accent2()),
        ),
        Span::styled(
            match st.last_check {
                Some(t) => ago(t),
                // Never having run is a real state: the cron may not be
                // installed on this box at all.
                None => "never — run `omega update --auto` to check now".to_string(),
            },
            dim,
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled(
            format!("  {:16}", "Last installed"),
            Style::default().fg(th::accent2()),
        ),
        Span::styled(
            match (&st.last_applied_commit, st.last_applied) {
                (Some(c), Some(t)) => format!("{}  ({})", c, ago(t)),
                _ => "nothing yet — this install is what you started with".to_string(),
            },
            dim,
        ),
    ]));

    if let Some(outcome) = &st.last_outcome {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:16}", "Last outcome"),
                Style::default().fg(th::accent2()),
            ),
            Span::styled(truncate_chars(outcome, 92), dim),
        ]));
    }

    // A stuck update is the one state worth shouting about: it means this box
    // has silently stopped receiving fixes.
    if st.consecutive_failures > 0 {
        let stuck = st.consecutive_failures >= omega_core::auto_update::FAILURE_CAP;
        lines.push(Line::from(Span::styled(
            format!(
                "  {:16}{} failed install(s) of {}{}",
                "",
                st.consecutive_failures,
                st.failing_commit.as_deref().unwrap_or("?"),
                if stuck {
                    " — STOPPED, needs you: run `omega update`"
                } else {
                    ""
                }
            ),
            Style::default()
                .fg(if stuck { th::error() } else { th::accent2() })
                .add_modifier(Modifier::BOLD),
        )));
    }

    lines.push(Line::from(Span::styled(
        "                  Change it: omega config set auto_update apply | check | off",
        dim,
    )));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  ↑/↓ pick a section on the left · Tab focuses this panel · [ and ] jump sections",
        dim,
    )));
    lines.push(Line::from(Span::styled(
        "  Same doctrine on the CLI: `omega rules list` · `omega guide` · `omega doctor`",
        dim,
    )));
    lines
}

/// Skills — the installed arsenal, grouped by category, straight from
/// ~/.omega/skills. Empty is a real state worth explaining, not a blank panel.
fn render_info_skills(app: &App) -> Vec<Line<'static>> {
    use omega_core::skill_registry::SkillCategory;

    let dim = Style::default().fg(th::dim());
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!(
                "  {} skills installed — invoked by name from any agent session.",
                app.skills.len()
            ),
            Style::default()
                .fg(th::accent())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!(
                "  {}",
                omega_core::config::omega_dir().join("skills").display()
            ),
            dim,
        )),
        Line::from(""),
    ];

    if let Some(error) = &app.skills_error {
        lines.push(Line::from(Span::styled(
            format!("  INVALID skill catalog: {error}"),
            Style::default()
                .fg(th::error())
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  Repair the catalog or rerun the installer; no partial catalog is shown.",
            Style::default().fg(th::error()),
        )));
        return lines;
    }

    if app.skills.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No skills found. Run `omega update` (or ./install.sh) to install the arsenal.",
            Style::default().fg(th::accent2()),
        )));
        return lines;
    }

    let categories = [
        SkillCategory::Audit,
        SkillCategory::Build,
        SkillCategory::Design,
        SkillCategory::Orchestration,
        SkillCategory::Marketing,
        SkillCategory::Utility,
        SkillCategory::Custom,
    ];
    for cat in categories {
        let mut in_cat: Vec<_> = app.skills.iter().filter(|s| s.category == cat).collect();
        if in_cat.is_empty() {
            continue;
        }
        in_cat.sort_by(|a, b| a.name.cmp(&b.name));
        lines.push(Line::from(Span::styled(
            format!("  ── {} ({}) ──", cat.label(), in_cat.len()),
            Style::default()
                .fg(th::accent2())
                .add_modifier(Modifier::BOLD),
        )));
        for skill in in_cat {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  /{:28}", skill.name),
                    Style::default().fg(th::accent()),
                ),
                Span::styled(
                    truncate_chars(&skill.description, 90),
                    Style::default().fg(th::text()),
                ),
            ]));
        }
        lines.push(Line::from(""));
    }
    lines
}

/// Documentation — the installed manual. The left half of the panel is a
/// document list with its own ↑/↓ cursor, the rest is the selected document.
fn render_info_docs(app: &mut App) -> (Vec<Line<'static>>, usize) {
    let dim = Style::default().fg(th::dim());
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!(
                "  {} documents — ↑/↓ to pick one, it opens below.",
                app.docs.len()
            ),
            Style::default()
                .fg(th::accent())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {}", omega_core::docs::docs_dir().display()),
            dim,
        )),
        Line::from(""),
    ];

    if app.docs.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No documentation installed yet.",
            Style::default()
                .fg(th::accent2())
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Run `omega update` — the installer mirrors the OmegaOS manual into",
            Style::default().fg(th::text()),
        )));
        lines.push(Line::from(Span::styled(
            format!(
                "  {} so it reads offline, with no checkout.",
                omega_core::docs::docs_dir().display()
            ),
            Style::default().fg(th::text()),
        )));
        return (lines, 0);
    }

    let selected = app.info_doc_selected.min(app.docs.len() - 1);
    let mut selected_line = 4usize;
    let mut current_group = String::new();
    for (i, doc) in app.docs.iter().enumerate() {
        if doc.group != current_group {
            current_group = doc.group.clone();
            lines.push(Line::from(Span::styled(
                format!("  ── {} ──", current_group),
                Style::default()
                    .fg(th::accent2())
                    .add_modifier(Modifier::BOLD),
            )));
        }
        let is_selected = i == selected;
        if is_selected {
            selected_line = lines.len();
        }
        let title_style = if is_selected {
            Style::default()
                .fg(th::sel_fg())
                .bg(th::accent2())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(th::text())
        };
        lines.push(Line::from(vec![
            Span::styled(
                if is_selected { "▶ " } else { "  " },
                Style::default().fg(th::accent()),
            ),
            Span::styled(
                format!("{:34}", truncate_chars(&doc.title, 34)),
                title_style,
            ),
            Span::styled(format!(" {:>5}  ", human_size(doc.bytes)), dim),
            Span::styled(truncate_chars(&doc.summary, 70), dim),
        ]));
    }

    let (title, rel_path) = {
        let doc = &app.docs[selected];
        (doc.title.clone(), doc.rel_path.clone())
    };
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  ═══ {} ═══", title),
        Style::default()
            .fg(th::accent())
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(format!("  {}", rel_path), dim)));
    lines.push(Line::from(""));

    // The body is plain text, deliberately: a terminal-width markdown renderer
    // is a different feature. Headings are the one thing worth colouring —
    // without them a 900-line spec is an undifferentiated wall.
    let body = app.selected_doc_body().unwrap_or("").to_string();
    for raw in body.lines() {
        let line = raw.trim_end();
        if line.starts_with("#### ") || line.starts_with("### ") {
            lines.push(Line::from(Span::styled(
                format!("  {}", line),
                Style::default().fg(th::accent2()),
            )));
        } else if line.starts_with("## ") || line.starts_with("# ") {
            lines.push(Line::from(Span::styled(
                format!("  {}", line),
                Style::default()
                    .fg(th::accent())
                    .add_modifier(Modifier::BOLD),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                format!("  {}", line),
                Style::default().fg(th::text()),
            )));
        }
    }
    (lines, selected_line)
}

/// Char-safe truncation — byte slicing panics on the emoji and accented text
/// that project names, skill descriptions and doc summaries are full of.
fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let cut: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{}…", cut.trim_end())
}

fn human_size(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1}M", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{}K", bytes / 1024)
    } else {
        format!("{}B", bytes)
    }
}

fn draw_help(frame: &mut Frame, app: &mut App, area: Rect) {
    let cy = Style::default()
        .fg(th::accent())
        .add_modifier(Modifier::BOLD);
    let yl = Style::default()
        .fg(th::accent2())
        .add_modifier(Modifier::BOLD);
    let wh = Style::default().fg(th::text());
    let gr = Style::default().fg(th::dim());
    let mg = Style::default().fg(th::special());

    let section = |title: &str| -> Line<'static> {
        Line::from(Span::styled(format!("  ─── {} ───", title), yl))
    };
    let key = |k: &str, desc: &str| -> Line<'static> {
        Line::from(vec![
            Span::styled(format!("    {:22}", k), cy),
            Span::styled(desc.to_string(), wh),
        ])
    };

    let mut lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Ω  ",
                Style::default()
                    .fg(th::accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "OmegaOS",
                Style::default().fg(th::text()).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  —  Agentic Terminal Operating System", gr),
            Span::styled(concat!("   v", env!("CARGO_PKG_VERSION")), cy),
        ]),
        Line::from(""),
        section("Navigation"),
        key("← / →", "Switch tabs"),
        key("Shift+Tab", "Previous tab"),
        key("F1", "Open this Help tab"),
        key(
            "Drag",
            "Select text in the session view → copies to clipboard (OSC 52)",
        ),
        key(
            "Ctrl+T",
            "Toggle mouse capture (off = native text selection)",
        ),
        key("Ctrl+L", "Redraw screen (fix corrupted view)"),
        key("Esc", "Back (detail → list → Sessions → quit)"),
        key("q", "Quit OmegaOS"),
        Line::from(""),
        section("Tab Behavior (Enter & Tab)"),
        Line::from(vec![
            Span::styled("    Tab", cy),
            Span::styled("  = focus right panel  ", wh),
            Span::styled("Tab-Tab", cy),
            Span::styled("  = fullscreen  ", wh),
            Span::styled("Esc", cy),
            Span::styled("  = back", wh),
        ]),
        Line::from(Span::styled(
            "    Same pattern on Sessions, Settings, Projects, System, and OS.",
            gr,
        )),
        Line::from(""),
        section("Sessions"),
        // DESIGN-016: j/k are deliberately swallowed on Sessions (operator
        // preference) — only the arrows navigate; don't document dead keys.
        key("↑ / ↓", "Navigate sessions"),
        key("Enter / Tab", "Focus chat (talk to agent)"),
        key("Tab-Tab", "Chat fullscreen (hide list)"),
        key("r  /  R", "Rename selected session"),
        key("x  /  X", "Kill session (skip if locked)"),
        key(".", "Toggle lock/protection"),
        key("/", "Filter the session list (empty+Enter clears)"),
        key("b", "Jump to next blocked/failed session"),
        key("PgUp / PgDn", "Scroll preview"),
        key("Home / End", "Top / bottom (tail-follow)"),
        Line::from(""),
        section("Menu — Agent Launchers"),
    ];

    let launchers = [
        ("c", "Claude"),
        ("o", "Codex"),
        ("g", "Gemini"),
        ("a", "Antigravity"),
        ("p", "Pi"),
        ("h", "Hermes"),
        ("G", "GLM"),
        ("K", "Kimi"),
        ("t", "Terminal"),
    ];
    let mut row = vec![Span::raw("    ")];
    for (i, (k, name)) in launchers.iter().enumerate() {
        row.push(Span::styled(format!("[{}]", k), yl));
        row.push(Span::styled(format!(" {:10}", name), wh));
        if (i + 1) % 4 == 0 {
            lines.push(Line::from(std::mem::take(&mut row)));
            row.push(Span::raw("    "));
        }
    }
    if row.len() > 1 {
        lines.push(Line::from(row));
    }

    lines.extend([
        key("d", "Dispatch oracle → project + mission"),
        key("F5", "Refresh sessions (r/R = Rename, everywhere)"),
        Line::from(""),
        section("Settings → Monitor group"),
        key("↑ / ↓ + Enter", "Run highlighted action"),
        key("L", "Login Claude (OAuth)"),
        key("T / D", "Telegram setup / disconnect"),
        key("P", "Set up provisioning keys"),
        key("B", "Refresh billing"),
        key("U", "Update OmegaOS"),
        Line::from(""),
        section("Settings"),
        key("↑ / ↓", "Browse sections"),
        key("Enter / Tab", "Focus detail panel → edit fields"),
        key("Enter (on field)", "Activate (install/edit/toggle)"),
        key("x", "Clear selected text field (press twice)"),
        Line::from(""),
        section("Projects"),
        key("↑ / ↓", "Browse projects"),
        key("Enter", "Focus detail; Enter again → open in terminal"),
        key("d", "Dispatch oracle to selected project"),
        key(
            "p",
            "Create a plan with /omg-planner for the selected project",
        ),
        key("n", "Register an existing folder as a project"),
        key("T", "Toggle the project's Telegram topic"),
        key("x", "Delete… (1 OmegaOS · 2 + local folder · 3 + GitHub)"),
        key("D", "Quick delete local (press twice)"),
        Line::from(""),
        section("OS"),
        key("↑ / ↓", "Browse operative systems"),
        key(
            "Enter",
            "Focus detail; Enter again → open its master or integration prompt",
        ),
        key("T", "Link a Telegram bot to the selected OS"),
        key("F5", "Rescan OS readiness from local runtime surfaces"),
        Line::from(""),
        section("System — the doctrine, the agents, the manual"),
        key(
            "↑ / ↓",
            "Pick a section; on Agents or Docs, opens that submenu immediately",
        ),
        key("Tab", "Focus the right panel to read or scroll prose"),
        key(
            "↑ / ↓ (Agents/Docs)",
            "Move the agent or manual-page cursor (Enter is not required)",
        ),
        key(
            "[  /  ]",
            "Previous / next section while the detail is focused",
        ),
        key("PgUp/PgDn Home/End", "Page and jump through long documents"),
        key("Tab-Tab", "Fullscreen the panel (a whole doc, full width)"),
        Line::from(""),
        section("Chat (Sessions, when chat-focused)"),
        key("Tab", "Return to session list"),
        key(
            "Shift+Tab",
            "Forward Shift+Tab to the focused agent session",
        ),
        key(
            "Esc",
            "Sent to the agent (escape vim/less/prompts) — Tab goes back",
        ),
        key("Ctrl+X", "Close (kill) the focused session"),
        key("Ctrl+R", "Reload the TUI (re-exec the binary)"),
        key("Alt+↑ / Alt+↓", "Scroll preview"),
        key("Ctrl+W / Alt+Bksp", "Delete word backwards"),
        key("Shift+Del / Alt+Del", "Delete word forwards"),
        key("Opt+< / Opt+>", "Jump to start / end of input (M-< / M->)"),
        key("Mouse scroll", "Scroll the panel under the cursor"),
        Line::from(""),
        section("Integrated Tools"),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("Hermes  ", cy),
            Span::styled("Nous Research multi-agent coordinator", gr),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("Pi      ", cy),
            Span::styled("earendil-works coding agent (standalone)", gr),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("PDF Gen ", cy),
            Span::styled("Whitepaper/audit/marketing reports → Telegram", gr),
        ]),
        Line::from(Span::styled(
            "    Install: omega install hermes | omega install pi",
            mg,
        )),
        Line::from(Span::styled(
            "    Generate: omega pdf --template=whitepaper --demo --send",
            mg,
        )),
        Line::from(""),
        section("CLI Commands"),
    ]);

    let cmds = [
        ("omega", "Launch TUI"),
        ("omega aisb-view", "Open read-only AISB conversation viewer"),
        ("omega aisb-chat", "Open interactive Telegram chat REPL"),
        (
            "omega plan-create [dir]",
            "Create a project plan with /omg-planner",
        ),
        ("omega list", "List sessions"),
        ("omega pdf --demo --send", "Generate + send PDF to Telegram"),
        ("omega orchestrate <P> <M>", "Full mission pipeline"),
        ("omega telegram setup …", "Bot setup"),
        ("omega config set …", "Provider config"),
        ("omega projects", "Auto-discover projects"),
        ("omega install <agent>", "Install an agent CLI"),
    ];
    for (cmd, desc) in cmds {
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(format!("{:30}", cmd), mg),
            Span::styled(desc.to_string(), gr),
        ]));
    }

    lines.extend([
        Line::from(""),
        section("Status Icons"),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("◆ ", cy),
            Span::styled("Oracle   ", gr),
            Span::styled("● ", cy),
            Span::styled("Worker   ", gr),
            Span::styled("⌂ ", cy),
            Span::styled("Home   ", gr),
            Span::styled("⚙ ", cy),
            Span::styled("System   ", gr),
            Span::styled("§ ", yl),
            Span::styled("Locked", gr),
        ]),
        Line::from(""),
    ]);

    // Publish + pin the scroll bound (same contract as draw_settings/draw_info)
    // so wheel-overscroll stops at the last line instead of a blank panel.
    let detail_width = area.width.saturating_sub(2).max(1);
    app.detail_max_scroll =
        wrapped_row_count(&lines, detail_width).saturating_sub(area.height.saturating_sub(2));
    app.detail_scroll = app.detail_scroll.min(app.detail_max_scroll);

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help — OmegaOS  (↑/↓ scroll) ")
                .border_style(Style::default().fg(th::accent())),
        );

    frame.render_widget(paragraph, area);
}

fn draw_status_bar(frame: &mut Frame, app: &mut App, area: Rect) {
    // fix6-T8: expire an async sticky notice render-side — the keypress TTL
    // only runs on input, so without this an idle operator's status bar held
    // the stale notice (masking the git segment) indefinitely.
    app.expire_sticky_status();
    // Input mode: show a prompt line (no stats)
    if !matches!(app.input_mode, InputMode::Normal) {
        let (prompt, value) = match &app.input_mode {
            InputMode::Normal => unreachable!(),
            InputMode::NewNamedSession(agent) => {
                ("Session name", format!("[{}] {}", agent, app.input_buffer))
            }
            InputMode::NewSessionPromptDirect(name, agent) => (
                "Initial prompt (optional, Esc to skip)",
                format!("[{}/{}] {}", name, agent, app.input_buffer),
            ),
            InputMode::DispatchProject(projects, sel) => (
                "Dispatch — pick project (↑/↓, Enter)",
                projects.get(*sel).cloned().unwrap_or_default(),
            ),
            InputMode::DispatchMission(p) => (
                "Dispatch — mission",
                format!("[{}] {}", p, app.input_buffer),
            ),
            InputMode::RenameSession(old) => (
                "Rename session",
                format!("[{} →] {}", old, app.input_buffer),
            ),
            InputMode::SessionFilter => ("Filter sessions", app.input_buffer.clone()),
            InputMode::NewProjectName => ("New project — name", app.input_buffer.clone()),
            InputMode::NewProjectCategory(name, _) => (
                "New project — category (overlay ↑/↓)",
                format!("[{}]", name),
            ),
            InputMode::NewProjectStack(name, category, _) => (
                "New project — stack (overlay ↑/↓)",
                format!("[{}/{}]", category, name),
            ),
            InputMode::NewProjectCredGroup(..) => {
                ("New project — credential group", app.input_buffer.clone())
            }
            InputMode::NewProjectLaunchPrompt(..) => {
                ("New project — kickoff (optional)", app.input_buffer.clone())
            }
            InputMode::NewProjectLaunchDocs(..) => {
                ("New project — docs (optional)", app.input_buffer.clone())
            }
            InputMode::SelectModel(config_key, ..) => {
                // The same overlay also drives the theme picker — don't tell
                // the user they're selecting a "model" there.
                if config_key == "general.theme" {
                    (
                        "Select theme — ↑/↓ live preview, Enter saves, Esc reverts",
                        String::new(),
                    )
                } else {
                    ("Select model — ↑/↓, Enter, Esc", String::new())
                }
            }
            InputMode::ProjectDelete(..) => {
                ("Delete project — ↑/↓ or 1/2/3, Enter, Esc", String::new())
            }
            InputMode::ProjectOpenLane(..) => (
                "Open project — Coding / Marketing / Oracle — ↑/↓ or 1/2/3, Enter, Esc",
                String::new(),
            ),
            InputMode::ProjectOpenAgentPick { .. } => (
                "Pick the LLM (installed only) — ↑/↓ or digit, Enter, Esc back",
                String::new(),
            ),
            InputMode::ProvisioningSetup { step, .. } => {
                let f = crate::app::PROVISIONING_FIELDS.get(*step);
                let key = f.map(|x| x.0).unwrap_or("");
                let masked = f.map(|x| x.2).unwrap_or(true);
                let echo = if masked {
                    mask_inline(&app.input_buffer)
                } else {
                    app.input_buffer.clone()
                };
                ("Provisioning keys", format!("[{}] {}", key, echo))
            }
            InputMode::TelegramSetupToken => (
                "Telegram setup 1/3 — BOT_TOKEN",
                mask_inline(&app.input_buffer),
            ),
            InputMode::TelegramSetupChatId(_) => (
                "Telegram setup 2/3 — CHAT_ID (numeric)",
                app.input_buffer.clone(),
            ),
            InputMode::TelegramSetupUserId(_, chat) => (
                "Telegram setup 3/3 — user_id (Esc to skip)",
                format!("[chat={}] {}", chat, app.input_buffer),
            ),
            InputMode::EditSettingsField { config_key, masked } => (
                "Edit setting",
                if *masked {
                    format!("[{}] {}", config_key, mask_inline(&app.input_buffer))
                } else {
                    format!("[{}] {}", config_key, app.input_buffer)
                },
            ),
            InputMode::GroupSetupId => ("Telegram group id", app.input_buffer.clone()),
            InputMode::AddProjectPath => ("Add project — folder path", app.input_buffer.clone()),
            InputMode::ReauthCode => ("Claude re-login — authorize code", app.input_buffer.clone()),
        };

        let status = Paragraph::new(Line::from(vec![
            Span::styled(" ▶ ", Style::default().fg(th::sel_fg()).bg(th::accent2())),
            Span::styled(
                format!(" {}: ", prompt),
                Style::default()
                    .fg(th::accent2())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(value),
            Span::styled("█", Style::default().fg(th::accent2())),
        ]));
        frame.render_widget(status, area);
        return;
    }

    // Normal mode: status bar with live system stats (UX inspired by tmux-claude,
    // re-implemented against the rmux SDK). Memoized: SystemStats::read forks
    // a `df` subprocess — at 15-60 FPS that's a fork+exec storm per TUI.
    thread_local! {
        static STATS_MEMO: std::cell::RefCell<
            Option<(std::time::Instant, omega_core::sysinfo::SystemStats)>,
        > = const { std::cell::RefCell::new(None) };
        static USAGE_MEMO: std::cell::RefCell<
            Option<(std::time::Instant, UsageSnapshotRead)>,
        > = const { std::cell::RefCell::new(None) };
    }
    let stats = render_memo(&STATS_MEMO, omega_core::sysinfo::SystemStats::read);

    let cpu = format!("CPU {:.2}", stats.cpu_load);
    let ram = format!("RAM {}%", stats.ram_pct);
    let disk = format!("DSK {}%", stats.disk_used_pct);
    let n_sessions = format!("{} sess", app.sessions.len());
    // 5h token-budget snapshot (written by the `omega usage --check` cron) —
    // the real "before the hard stop" signal. None until the first check.
    let usage = render_memo(&USAGE_MEMO, || {
        omega_core::monitor::UsageSnapshot::read().map_err(|error| error.to_string())
    });

    // Localized to `config.timezone` (IANA) so the headless-VPS UTC clock shows
    // the operator's wall time; falls back to $TZ, then system local. See
    // omega_core::clock.
    let time_str = omega_core::clock::hm(app.config.timezone.as_deref());

    let session_info = app
        .selected_session()
        .map(|e| {
            let icon = match e.session.role {
                omega_core::session::SessionRole::Oracle => "◆",
                omega_core::session::SessionRole::Worker => "●",
                omega_core::session::SessionRole::Home => "⌂",
                omega_core::session::SessionRole::System => "⚙",
            };
            format!("{} {}", icon, e.session.name)
        })
        .unwrap_or_else(|| "—".to_string());

    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(70)])
        .split(area);

    // Transport / staleness banners — the two silent killers of "scroll ne
    // marche pas" / "le fix ne marche pas" reports. (1) Under mosh the mouse
    // handshake never reaches the terminal (mobile-shell/mosh#101): say so on
    // the bar instead of letting the operator debug rmux for the third time.
    // (2) After a deploy the running TUI keeps executing the DELETED binary —
    // fixes silently absent until a reload; /proc/self/exe gains a
    // " (deleted)" suffix the moment the file is replaced.
    thread_local! {
        static ENV_WARN_MEMO: std::cell::RefCell<Option<(std::time::Instant, Option<&'static str>)>> =
            const { std::cell::RefCell::new(None) };
    }
    let env_warn: Option<&'static str> = render_memo(&ENV_WARN_MEMO, || {
        if std::fs::read_link("/proc/self/exe")
            .map(|p| p.to_string_lossy().ends_with(" (deleted)"))
            .unwrap_or(false)
        {
            return Some("⟳ omega UPDATED — Ctrl+R to reload");
        }
        if under_mosh() {
            return Some("⚠ mosh — mouse OFF (use plain SSH for wheel/select)");
        }
        None
    });

    // (3) Mouse capture OFF is the single most confusing state this TUI can be
    // in: the wheel does nothing, clicks do nothing, and NOTHING on screen says
    // why. Ctrl-T toggles it globally (handle_key checks it before anything
    // else, so it fires even from chat focus) — and Ctrl-T is exactly the key
    // Claude uses for its todo panel and Codex for its transcript, so an
    // operator reaching for those silently disarms their own mouse and then
    // reports "the scroll is completely broken". The toggle only ever printed a
    // transient status line that the next refresh wiped. Make the state
    // PERSISTENT on the bar and name the way out. Not memoized: it must flip
    // the instant Ctrl-T is pressed, and it outranks the two warnings above
    // because it is the one the operator is actively fighting.
    let env_warn = if app.mouse_capture {
        env_warn
    } else {
        Some("🖱 mouse OFF — Ctrl-T to re-enable wheel + click")
    };

    // Left side: Ω badge (no bg, bold) + selected session + status message
    let left = Paragraph::new(Line::from(vec![
        Span::styled(
            " Ω ",
            Style::default().fg(th::text()).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            env_warn.map(|w| format!(" {} ", w)).unwrap_or_default(),
            Style::default().fg(th::warn()).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            session_info,
            Style::default().fg(th::text()).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        // FIX-A (fix5): an armed two-press confirm renders STATE-DRIVEN with
        // top priority — the warning comes from `armed_confirm_warning()`
        // (the state itself), not from `status_message`, so it is TTL-immune
        // and overwrite-immune by construction: launcher prompts, Ctrl-T,
        // paste acks and sticky forwarder errors can no longer hide a live
        // armed confirm (R-1/R-2/D-3/D-4). Below that, a live status notice
        // wins (F-7): kill/create/vanish/focus confirmations and forwarder
        // errors were built for this channel, and the event loop clears the
        // message on the next keypress — so on the Sessions tab the bar falls
        // back to the selected session's compact git status (`↑4h • main`)
        // once the notice is consumed. The keyboard hints that used to live
        // here are all under the Help tab.
        {
            let on_sessions = app.tab == crate::app::Tab::Sessions;
            let git_text = if on_sessions {
                app.selected_session()
                    .and_then(|e| app.session_git_status.get(&e.session.name).cloned())
            } else {
                None
            };
            let armed = app.armed_confirm_warning();
            let (text, style) = match (armed, app.status_message.as_deref(), git_text) {
                (Some(warn), _, _) => (
                    warn,
                    Style::default()
                        .fg(th::error())
                        .add_modifier(Modifier::BOLD),
                ),
                (None, Some(msg), _) => (msg.to_string(), Style::default().fg(th::dim())),
                (None, None, Some(g)) => (g, Style::default().fg(th::success())),
                (None, None, None) => (String::new(), Style::default()),
            };
            Span::styled(text, style)
        },
    ]));
    frame.render_widget(left, split[0]);

    // Right side: system stats (n_sessions in BOLD white so it pops)
    let stat_color = |pct: u8| -> Color {
        match pct {
            0..=60 => th::success(),
            61..=85 => th::accent2(),
            _ => th::error(),
        }
    };

    // Token-budget meter color escalates toward the 80/90% usage alerts.
    let usage_color = |pct: u32| -> Color {
        match pct {
            0..=69 => th::success(),
            70..=89 => th::accent2(),
            _ => th::error(),
        }
    };
    let mut right_spans: Vec<Span> = Vec::new();
    if let Ok(Some(ref u)) = usage {
        right_spans.push(Span::styled(
            format!("TKN {}%", u.session_pct),
            Style::default()
                .fg(usage_color(u.session_pct))
                .add_modifier(Modifier::BOLD),
        ));
        right_spans.push(Span::raw("  "));
    } else if usage.is_err() {
        right_spans.push(Span::styled(
            "TKN ERR",
            Style::default()
                .fg(th::error())
                .add_modifier(Modifier::BOLD),
        ));
        right_spans.push(Span::raw("  "));
    }
    right_spans.push(Span::styled(
        cpu,
        Style::default().fg(stat_color(((stats.cpu_load * 25.0) as u8).min(99))),
    ));
    right_spans.push(Span::raw("  "));
    right_spans.push(Span::styled(
        ram,
        Style::default().fg(stat_color(stats.ram_pct)),
    ));
    right_spans.push(Span::raw("  "));
    right_spans.push(Span::styled(
        disk,
        Style::default().fg(stat_color(stats.disk_used_pct)),
    ));
    right_spans.push(Span::raw("  "));
    right_spans.push(Span::styled(
        n_sessions,
        Style::default().fg(th::text()).add_modifier(Modifier::BOLD),
    ));
    right_spans.push(Span::raw("  "));
    right_spans.push(Span::styled(
        time_str,
        Style::default()
            .fg(th::accent())
            .add_modifier(Modifier::BOLD),
    ));
    right_spans.push(Span::raw(" "));
    let right =
        Paragraph::new(Line::from(right_spans)).alignment(ratatui::layout::Alignment::Right);
    frame.render_widget(right, split[1]);
}

#[cfg(test)]
mod url_fold_tests {
    use super::fold_for_panel;

    /// The whole URL must survive folding — that is the entire point: a clipped
    /// link is what the operator could not copy.
    #[test]
    fn fold_preserves_every_character() {
        let url = "https://claude.com/cai/oauth/authorize?code=true&client_id=abc\
&redirect_uri=https%3A%2F%2Fconsole.anthropic.com%2Foauth%2Fcode%2Fcallback&state=xyz";
        let folded = fold_for_panel(url, 56);
        assert!(folded.len() > 1, "a 150-char URL must fold");
        assert!(folded.iter().all(|c| c.chars().count() <= 56));
        assert_eq!(
            folded.concat(),
            url,
            "rejoining the chunks must give the URL back"
        );
    }

    #[test]
    fn fold_handles_short_and_empty() {
        assert_eq!(fold_for_panel("abc", 56), vec!["abc".to_string()]);
        assert_eq!(fold_for_panel("", 56), vec![String::new()]);
    }
}

#[cfg(test)]
mod system_overview_tests {
    use super::*;
    use crate::app::App;
    use omega_core::config::{AutoUpdatePolicy, OmegaConfig};

    /// Flatten rendered lines to plain text so assertions read like the screen.
    fn text(lines: &[Line<'static>]) -> String {
        lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The Overview must answer "is this box keeping itself up to date?".
    /// Building the auto-updater without surfacing it here is exactly the gap
    /// this test exists to catch.
    #[test]
    fn overview_reports_the_auto_update_policy_and_last_run() {
        let mut app = App::new(OmegaConfig::default());
        app.config.auto_update = AutoUpdatePolicy::Apply;
        app.auto_update_state = omega_core::auto_update::AutoUpdateState::default();

        let out = text(&render_info_overview(&app));
        assert!(out.contains("Auto-update"), "the policy must be named");
        assert!(out.contains("03:30"), "the schedule must be visible");
        assert!(out.contains("Last check"));
        assert!(
            out.contains("omega config set auto_update"),
            "the opt-out must be discoverable"
        );
        // Never-run is a real state, not a blank.
        assert!(
            out.contains("never"),
            "a box that never checked must say so"
        );
    }

    #[test]
    fn overview_shows_each_policy_distinctly() {
        let mut app = App::new(OmegaConfig::default());
        for (policy, expected) in [
            (AutoUpdatePolicy::Apply, "installs"),
            (AutoUpdatePolicy::Check, "alerts only"),
            (AutoUpdatePolicy::Off, "no automatic check"),
        ] {
            app.config.auto_update = policy;
            let out = text(&render_info_overview(&app));
            assert!(
                out.contains(expected),
                "policy {:?} must render {:?}",
                policy,
                expected
            );
        }
    }

    /// A stuck auto-update means the box has silently stopped getting fixes —
    /// it has to be loud, and it has to say what to do.
    #[test]
    fn a_stuck_auto_update_is_shouted_not_whispered() {
        let mut app = App::new(OmegaConfig::default());
        app.config.auto_update = AutoUpdatePolicy::Apply;
        for _ in 0..omega_core::auto_update::FAILURE_CAP {
            app.auto_update_state.record_failure("deadbee");
        }
        let out = text(&render_info_overview(&app));
        assert!(out.contains("deadbee"), "name the commit that is failing");
        assert!(out.contains("STOPPED"), "say that it gave up");
        assert!(out.contains("omega update"), "say how to unstick it");
    }

    /// One failure is worth reporting but is NOT the stuck state — tomorrow's
    /// run retries by itself.
    #[test]
    fn a_single_failure_is_reported_without_crying_wolf() {
        let mut app = App::new(OmegaConfig::default());
        app.auto_update_state.record_failure("deadbee");
        let out = text(&render_info_overview(&app));
        assert!(out.contains("deadbee"));
        assert!(!out.contains("STOPPED"), "one failure is not a stop");
    }

    #[test]
    fn invalid_skill_catalog_is_distinct_from_an_empty_catalog() {
        let mut app = App::new(OmegaConfig::default());
        app.skills.clear();
        app.skills_error = Some("catalog digest mismatch".to_string());

        let skills = text(&render_info_skills(&app));
        assert!(skills.contains("INVALID skill catalog"));
        assert!(skills.contains("catalog digest mismatch"));
        assert!(!skills.contains("No skills found"));

        let overview = text(&render_info_overview(&app));
        assert!(overview.contains("INVALID"));
    }

    #[test]
    fn invalid_project_registry_is_never_rendered_as_empty() {
        let mut app = App::new(OmegaConfig::default());
        app.project_registry.projects.clear();
        app.project_registry.poisoned = true;

        let detail = text(&render_project_detail(&app));
        assert!(detail.contains("INVALID project registry"));
        assert!(!detail.contains("No project selected"));

        let overview = text(&render_info_overview(&app));
        assert!(overview.contains("INVALID"));
    }
}

#[cfg(test)]
mod menu_state_tests {
    use crate::app::{agent_available_cached, MenuAction};
    use omega_core::agents::Agent;

    /// Every agent launcher in the Menu is a tool you might go and test, so
    /// each one has to say whether it is actually installed BEFORE you press
    /// it — the launch guard only refuses after the press.
    #[test]
    fn every_agent_launcher_resolves_an_install_state() {
        let launchers: Vec<Agent> = MenuAction::all()
            .iter()
            .filter_map(|a| a.agent())
            .filter(|a| !matches!(a, Agent::Shell))
            .collect();
        assert!(
            launchers.len() >= 5,
            "the Menu should carry the agent launchers, found {}",
            launchers.len()
        );
        // The lookup must answer for each, and must not panic or hang.
        for agent in launchers {
            let _: bool = agent_available_cached(agent);
        }
    }

    /// Non-agent rows (Refresh, Kill, Quit…) must NOT grow a state dot — a
    /// state glyph on "Quit" would be meaningless.
    #[test]
    fn only_agent_rows_carry_a_state() {
        assert!(MenuAction::Quit.agent().is_none());
        assert!(MenuAction::KillAll.agent().is_none());
        assert!(MenuAction::NewClaude.agent().is_some());
    }
}

#[cfg(test)]
mod responsive_tests {
    use super::*;
    use omega_core::config::OmegaConfig;
    use ratatui::{backend::TestBackend, Terminal};

    fn rendered(terminal: &Terminal<TestBackend>) -> String {
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    /// A phone in portrait is ~60 columns. The full tab bar needs far more, and
    /// ratatui clips it mid-word rather than adapting — at 70 columns the last
    /// tab read "Settin", at 60 it was gone entirely, so Settings did not appear
    /// to exist at all. The compact bar exists for exactly that range.
    #[test]
    fn the_tab_bar_knows_when_it_does_not_fit() {
        let needed = tab_bar_width();
        assert!(
            needed > 60,
            "the full bar cannot fit a phone; it must collapse"
        );
        // And the threshold must be honest about the real labels, not a guess.
        let longest_possible: usize = Tab::ORDER
            .iter()
            .map(|t| t.title().chars().count())
            .sum::<usize>();
        assert!(
            needed > longest_possible,
            "separators and borders count too"
        );
    }

    /// The scroll bound is computed from painted rows. Counting logical lines
    /// instead left the tail of a wrapped document unreachable by End.
    #[test]
    fn wrapped_rows_are_counted_not_logical_lines() {
        let lines = vec![
            Line::from("short"),
            // 30 chars at width 10 = 3 rows.
            Line::from("x".repeat(30)),
        ];
        assert_eq!(wrapped_row_count(&lines, 10), 4);
        // Wide enough for everything = one row each.
        assert_eq!(wrapped_row_count(&lines, 200), 2);
    }

    /// An empty line still occupies a row — otherwise every blank spacer would
    /// silently shorten the scroll bound.
    #[test]
    fn an_empty_line_still_takes_a_row() {
        assert_eq!(wrapped_row_count(&[Line::from("")], 40), 1);
        assert_eq!(wrapped_row_count(&[], 40), 0);
    }

    /// Width 0 must not divide by zero — terminals do report degenerate sizes
    /// mid-resize.
    #[test]
    fn a_zero_width_panel_does_not_panic() {
        assert_eq!(wrapped_row_count(&[Line::from("abc")], 0), 3);
    }

    /// A multi-span line is measured across ALL its spans — the System tab
    /// builds nearly every line from several styled spans.
    #[test]
    fn multi_span_lines_measure_their_whole_width() {
        let line = Line::from(vec![
            Span::raw("12345"),
            Span::raw("67890"),
            Span::raw("abcde"),
        ]);
        assert_eq!(wrapped_row_count(std::slice::from_ref(&line), 5), 3);
    }

    #[test]
    fn wrapped_rows_measure_terminal_columns_for_wide_glyphs() {
        assert_eq!(wrapped_row_count(&[Line::from("界界")], 2), 2);
    }

    #[test]
    fn wrapped_rows_match_word_boundary_wrapping() {
        assert_eq!(wrapped_row_count(&[Line::from("aaa aaa aaa aaa")], 5), 4);
        assert_eq!(
            wrapped_row_count(&[Line::from("AAAAAAAAAAAAAAAAAAAA    AAA")], 20),
            2
        );
        assert_eq!(
            wrapped_row_count(&[Line::from("AAA AAA AAAAA AA AAAAAA")], 10),
            3
        );
        assert_eq!(
            wrapped_row_count(&[Line::from("               4 Indent")], 10),
            3
        );
    }

    #[test]
    fn settings_collapses_to_the_focused_column_on_phone_widths() {
        let mut app = App::new(OmegaConfig::default());
        app.tab = Tab::Settings;
        let mut terminal = Terminal::new(TestBackend::new(60, 16)).unwrap();

        terminal
            .draw(|frame| {
                let area = frame.area();
                draw_settings(frame, &mut app, area);
            })
            .unwrap();
        assert!(rendered(&terminal).contains("Tab → read"));

        app.detail_focused = true;
        terminal
            .draw(|frame| {
                let area = frame.area();
                draw_settings(frame, &mut app, area);
            })
            .unwrap();
        assert!(rendered(&terminal).contains("Tab → list"));
    }

    #[test]
    fn every_tab_survives_degenerate_terminal_sizes() {
        for (width, height) in [(1, 1), (20, 5), (60, 16)] {
            for tab in Tab::ORDER {
                let mut app = App::new(OmegaConfig::default());
                app.tab = tab;
                let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
                terminal
                    .draw(|frame| draw(frame, &mut app))
                    .unwrap_or_else(|error| panic!("{tab:?} failed at {width}x{height}: {error}"));
            }
        }
    }
}

#[cfg(test)]
mod help_contract_tests {
    use super::*;
    use omega_core::config::OmegaConfig;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn help_exposes_update_and_provider_neutral_chat_without_hidden_master_role() {
        let mut app = App::new(OmegaConfig::default());
        let mut terminal = Terminal::new(TestBackend::new(180, 140)).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                draw_help(frame, &mut app, area);
            })
            .unwrap();
        let rendered = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(
            rendered.contains("Update OmegaOS"),
            "Monitor U action is missing"
        );
        assert!(rendered.contains("focused agent session"));
        assert!(rendered.contains("AISB conversation viewer"));
        assert!(rendered.contains("Link a Telegram bot to the selected OS"));
        assert!(!rendered.contains("AISB Master"));
        assert!(!rendered.contains("Forward to Claude"));
    }

    #[test]
    fn help_wraps_long_rows_instead_of_clipping_their_tail() {
        let mut app = App::new(OmegaConfig::default());
        let mut terminal = Terminal::new(TestBackend::new(60, 20)).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                draw_help(frame, &mut app, area);
            })
            .unwrap();
        let rendered = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(
            rendered.contains("(OSC 52)"),
            "the tail of a long help row must wrap onto a visible line"
        );
    }
}

#[cfg(test)]
mod os_readiness_render_tests {
    use super::*;
    use omega_core::config::OmegaConfig;
    use omega_core::os_products::{
        OsEntry, OsManifestStatus, OsProduct, OsReadiness, OsReadinessLevel,
    };
    use ratatui::{backend::TestBackend, Terminal};

    fn text(lines: &[Line<'static>]) -> String {
        lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn os_detail_reports_static_evidence_without_claiming_tests_passed() {
        let mut app = App::new(OmegaConfig::default());
        app.os_entries = vec![OsEntry {
            product: OsProduct::all()[0],
            readiness: OsReadiness {
                level: OsReadinessLevel::Testable,
                directory_present: true,
                master_present: true,
                payload_present: true,
                manifest: OsManifestStatus::Valid,
                runtime_present: true,
                tests_present: true,
                event_schema_status: Some("stub".to_string()),
            },
            path: Some(std::path::PathBuf::from("/tmp/os")),
            bot_linked: false,
        }];

        let rendered = text(&render_os_detail(&app));
        assert!(rendered.contains("runtime + tests present (not executed)"));
        assert!(rendered.contains("present (not executed)"));
        assert!(rendered.contains("Event schema"));
        assert!(rendered.contains("stub"));
        assert!(!rendered.contains("integrated"));
        assert!(!rendered.contains("tests passed"));
    }

    #[test]
    fn os_tab_uses_the_responsive_wrapped_detail_contract() {
        let mut app = App::new(OmegaConfig::default());
        app.os_entries = vec![OsEntry {
            product: OsProduct::all()[0],
            readiness: OsReadiness {
                level: OsReadinessLevel::Reference,
                directory_present: true,
                master_present: false,
                payload_present: true,
                manifest: OsManifestStatus::Missing,
                runtime_present: false,
                tests_present: false,
                event_schema_status: None,
            },
            path: Some(std::path::PathBuf::from(
                "/tmp/a-deliberately-long-operative-system-path-for-wrapping",
            )),
            bot_linked: false,
        }];

        let mut list_terminal = Terminal::new(TestBackend::new(60, 10)).unwrap();
        list_terminal
            .draw(|frame| {
                let area = frame.area();
                draw_os(frame, &mut app, area);
            })
            .unwrap();
        let list_rendered = list_terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(
            list_rendered.contains("Tab → read"),
            "narrow OS view must show one focused column"
        );

        app.detail_focused = true;
        let detail_lines = render_os_detail(&app);
        let mut detail_terminal = Terminal::new(TestBackend::new(60, 10)).unwrap();
        detail_terminal
            .draw(|frame| {
                let area = frame.area();
                draw_os(frame, &mut app, area);
            })
            .unwrap();
        let detail_rendered = detail_terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(detail_rendered.contains("Tab → list"));
        assert!(detail_rendered.contains("Readiness"));
        assert_eq!(
            app.detail_max_scroll,
            wrapped_row_count(&detail_lines, 58).saturating_sub(8),
            "scroll bound must count the rows actually painted after wrapping"
        );
    }
}
