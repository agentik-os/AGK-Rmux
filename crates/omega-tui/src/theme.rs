//! TUI theme engine — semantic color roles + selectable palettes.
//!
//! Every chrome color in `ui.rs` goes through a semantic accessor here
//! (`th::accent()`, `th::success()`, …) instead of a hardcoded
//! `Color::Cyan`. The active theme is a process-global (lock-free
//! `AtomicU8`) so switching it in Settings re-skins the whole TUI on the
//! very next frame — no plumbing through every render function.
//!
//! Mostly NOT themed on purpose: the session-pane preview passthrough
//! (`preview_to_color` in ui.rs) — that is the agent's own output and keeps
//! the terminal's real palette. ONE deliberate exception: a small
//! high-confidence emphasis catalog in `draw_sessions_right` (the selection
//! bar, todo states, user-input echo, task dispatch, the activity footer)
//! repaints whole matched rows with the active theme's semantic roles so
//! those cues stay readable on every theme. Everything that doesn't match
//! the catalog falls through to the raw ANSI render.
//!
//! # Contrast contract (test-enforced, WCAG 2.x AA)
//!
//! Every theme that paints its own background (`bg: Some(..)`) must hold,
//! measured against that bg:
//!
//! | roles                                            | minimum ratio |
//! |---------------------------------------------------|---------------|
//! | text, dim, info, error, warn, bright              | 4.5 : 1       |
//! | accent, accent2, success, special (text-bearing)  | 4.5 : 1       |
//! | dim2 (dead role — hierarchy guard only)           | 3.0 : 1       |
//! | sel_fg on the accent (selection bars)             | 4.5 : 1       |
//! | sel_fg on accent2 (selected fields)               | 4.5 : 1       |
//! | warn vs accent, error vs accent (distinctness)    | CIE76 ΔE ≥ 30 |
//!
//! The last row is the role-vs-role axis: an alert must read as a DIFFERENT
//! state than the active accent, not just clear the bg. Noir and Paper are
//! exempt — mono by design, the badge glyphs (`+ ~ x !`) carry the state.
//!
//! plus the gray hierarchy `ratio(dim2) < ratio(dim) < ratio(text)` so the
//! three quiet levels stay visually ordered. A theme that fails is fixed by
//! tuning the palette VALUE, never by relaxing the threshold — see
//! `tests::contrast_contract`.
//!
//! Omega is exempt: `bg = None` and its roles are named ANSI colors that
//! delegate to the terminal's own palette, so no fixed luminance exists to
//! audit. Every other theme is truecolor by design — literal RGB renders
//! identically on every truecolor emulator (incl. Termius); only Omega
//! adapts to the terminal scheme.

use ratatui::style::Color;
use std::sync::atomic::{AtomicU8, Ordering};

/// Semantic color roles for the TUI chrome. Mapping from the legacy
/// hardcoded palette: Cyan→accent, Yellow→accent2, Green→success,
/// Red→error, Blue→info, Magenta→special, Gray→dim, DarkGray→dim2,
/// White→bright, Black→sel_fg (text on accent-colored selection bars).
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub accent: Color,
    pub accent2: Color,
    pub success: Color,
    pub error: Color,
    /// Attention-but-not-failure states (blocked badge, stale markers) —
    /// semantically between accent2 and error.
    pub warn: Color,
    pub info: Color,
    pub special: Color,
    pub dim: Color,
    pub dim2: Color,
    pub bright: Color,
    pub sel_fg: Color,
    /// Full-screen background. `None` = keep the terminal's own background
    /// (the Omega default). `Some(..)` is what makes a theme unmistakably
    /// different at a glance — Dracula's purple, Paper's white, etc.
    pub bg: Option<Color>,
    /// Body text color. `Color::Reset` (terminal default) for bg-less themes;
    /// themes that paint a background MUST set an explicit readable text
    /// color (the terminal's default fg may be invisible on the painted bg).
    pub text: Color,
    // Bright variants (legacy Light* colors — rare call sites).
    pub accent_hi: Color,
    pub accent2_hi: Color,
    pub success_hi: Color,
    pub error_hi: Color,
    pub warn_hi: Color,
    pub info_hi: Color,
    pub special_hi: Color,
}

/// Compact const constructor: the `*_hi` variants default to their base
/// role; themes that want distinct brights override via struct update.
#[allow(clippy::too_many_arguments)]
const fn mk(
    accent: Color,
    accent2: Color,
    success: Color,
    error: Color,
    warn: Color,
    info: Color,
    special: Color,
    dim: Color,
    dim2: Color,
    bright: Color,
    sel_fg: Color,
) -> Theme {
    Theme {
        accent,
        accent2,
        success,
        error,
        warn,
        info,
        special,
        dim,
        dim2,
        bright,
        sel_fg,
        bg: None,
        text: Color::Reset,
        accent_hi: accent,
        accent2_hi: accent2,
        success_hi: success,
        error_hi: error,
        warn_hi: warn,
        info_hi: info,
        special_hi: special,
    }
}

/// Pale tint of an RGB color (2/3 toward white) — used for the roles that
/// must stand out from the gray chrome without competing with the accent
/// itself (separators, selected-field background, hints).
const fn lighten(c: Color) -> Color {
    match c {
        Color::Rgb(r, g, b) => Color::Rgb(
            ((r as u16 + 2 * 255) / 3) as u8,
            ((g as u16 + 2 * 255) / 3) as u8,
            ((b as u16 + 2 * 255) / 3) as u8,
        ),
        other => other,
    }
}

/// The "Monogram model" (user-validated): a quiet grayscale chrome + ONE
/// signature accent color. Every theme except Omega (terminal-native),
/// Noir and Paper (pure mono by design) is built from this template — the
/// palettes differ only by their accent, background and text tint.
///
/// Visual hierarchy (mirrors what Omega's multicolor chrome conveys):
/// - accent          → selection bars, focus, borders of the active panel
/// - lighten(accent) → accent2: section separators (`─── x ───`), the
///   selected settings-field background, hints — clearly
///   NOT body gray, clearly NOT the full accent
/// - accent          → success/active states (`● on`, running markers):
///   the signature color marks everything alive
///
/// `dim`/`dim2` are per-theme: the quietest grays that still clear the
/// contrast contract on THIS theme's bg (dim ≥ 4.5:1, dim2 ≥ 3.0:1) —
/// literal values here, the WCAG math lives in the tests.
const fn mono(accent: Color, bg: Color, text: Color, dim: Color, dim2: Color) -> Theme {
    Theme {
        bg: Some(bg),
        text,
        ..mk(
            accent,
            lighten(accent),           // accent2 — separators / selected-field bg
            accent,                    // success — active states wear the accent
            Color::Rgb(255, 110, 110), // error — alert red, distinct from bg AND actives
            Color::Rgb(255, 165, 0),   // warn — blocked/attention badge
            Color::Rgb(180, 180, 180), // info
            lighten(accent),           // special — subtle but themed
            dim,
            dim2,
            Color::White, // bright
            Color::Black, // sel_fg
        )
    }
}

// ── Palettes ────────────────────────────────────────────────────────────────

/// Omega (default) — the classic cyan/yellow chrome, 16-color named ANSI so
/// it adapts to the terminal's own scheme (light and dark).
const OMEGA: Theme = Theme {
    accent_hi: Color::LightCyan,
    accent2_hi: Color::LightYellow,
    success_hi: Color::LightGreen,
    error_hi: Color::LightRed,
    info_hi: Color::LightBlue,
    special_hi: Color::LightMagenta,
    ..mk(
        Color::Cyan,
        Color::Yellow,
        Color::Green,
        Color::Red,
        Color::LightRed, // warn — named ANSI like every other Omega role, so
        // it adapts to light terminal schemes (a fixed orange
        // went near-invisible there); red-orange alert family,
        // distinct from pending (Yellow) and error (Red)
        Color::Blue,
        Color::Magenta,
        Color::Gray,
        Color::DarkGray,
        Color::White,
        Color::Black,
    )
};

/// Matrix — mono chrome + neon Matrix green.
const MATRIX: Theme = mono(
    Color::Rgb(0, 255, 65),
    Color::Rgb(5, 15, 5),
    Color::Rgb(210, 230, 210),
    Color::Rgb(122, 122, 122),
    Color::Rgb(94, 94, 94),
);

/// Terminal — mono chrome + soft phosphor green.
const TERMINAL: Theme = mono(
    Color::Rgb(102, 255, 102),
    Color::Rgb(0, 18, 0),
    Color::Rgb(200, 225, 200),
    Color::Rgb(122, 122, 122),
    Color::Rgb(95, 95, 95),
);

/// Amber — mono chrome + retro amber phosphor.
/// Its accent IS orange — the mono warn orange would be the same state color
/// (ΔE76 6.5), so warn moves to the alert-red family (glyphs disambiguate
/// warn from error within the Alerte class).
const AMBER: Theme = Theme {
    warn: Color::Rgb(255, 90, 90),
    warn_hi: Color::Rgb(255, 90, 90),
    ..mono(
        Color::Rgb(255, 176, 0),
        Color::Rgb(22, 13, 0),
        Color::Rgb(235, 220, 200),
        Color::Rgb(123, 123, 123),
        Color::Rgb(95, 95, 95),
    )
};

/// Noir — full black & white, pure grayscale on dark terminals.
const NOIR: Theme = Theme {
    bg: Some(Color::Rgb(0, 0, 0)),
    text: Color::Rgb(220, 220, 220),
    ..mk(
        Color::White,
        Color::White, // accent2 — separators / selected-field bg pop above the 220 text
        Color::Rgb(235, 235, 235), // success — actives brighter than body text
        Color::Rgb(255, 255, 255),
        Color::Rgb(255, 165, 0), // warn — the one non-gray: blocked must read at a glance
        Color::Rgb(170, 170, 170),
        Color::Rgb(215, 215, 215),
        Color::Rgb(120, 120, 120),
        Color::Rgb(90, 90, 90),
        Color::White,
        Color::Black,
    )
};

/// Paper — full white: ink-on-paper for LIGHT terminal backgrounds.
const PAPER: Theme = Theme {
    bg: Some(Color::Rgb(245, 245, 240)),
    text: Color::Rgb(20, 20, 20),
    ..mk(
        Color::Rgb(20, 20, 20),
        Color::Rgb(0, 0, 0), // accent2 — separators / selected-field bg: pure ink
        Color::Rgb(30, 30, 30), // success — actives darker than body ink
        Color::Rgb(0, 0, 0),
        Color::Rgb(150, 75, 0), // warn — burnt orange, legible on the paper bg
        Color::Rgb(90, 90, 90),
        Color::Rgb(60, 60, 60),
        Color::Rgb(112, 112, 112),
        Color::Rgb(141, 141, 141),
        Color::Rgb(0, 0, 0),
        Color::Rgb(255, 255, 255),
    )
};

/// Transparent Dark — NO painted background: the terminal's own bg (and any
/// transparency/blur it has) shows through. White-ink mono chrome — Noir's
/// hierarchy, for dark or translucent terminal backgrounds.
const TRANSPARENT_DARK: Theme = Theme {
    bg: None,
    text: Color::Rgb(220, 220, 220),
    ..mk(
        Color::White,
        Color::White, // accent2 — separators / selected-field bg pop above the 220 text
        Color::Rgb(235, 235, 235), // success — actives brighter than body text
        Color::Rgb(255, 255, 255),
        Color::Rgb(255, 165, 0), // warn — the one non-gray: blocked must read at a glance
        Color::Rgb(170, 170, 170),
        Color::Rgb(215, 215, 215),
        Color::Rgb(120, 120, 120),
        Color::Rgb(90, 90, 90),
        Color::White,
        Color::Black,
    )
};

/// Transparent Light — NO painted background, black-ink mono chrome —
/// Paper's hierarchy, for light terminal backgrounds.
const TRANSPARENT_LIGHT: Theme = Theme {
    bg: None,
    text: Color::Rgb(20, 20, 20),
    ..mk(
        Color::Rgb(20, 20, 20),
        Color::Rgb(0, 0, 0), // accent2 — separators / selected-field bg: pure ink
        Color::Rgb(30, 30, 30), // success — actives darker than body ink
        Color::Rgb(0, 0, 0),
        Color::Rgb(150, 75, 0), // warn — burnt orange, legible on a light bg
        Color::Rgb(90, 90, 90),
        Color::Rgb(60, 60, 60),
        Color::Rgb(112, 112, 112),
        Color::Rgb(141, 141, 141),
        Color::Rgb(0, 0, 0),
        Color::Rgb(255, 255, 255),
    )
};

/// Monogram — the original: mono chrome + cyan.
const MONOGRAM: Theme = mono(
    Color::Rgb(0, 255, 255),
    Color::Rgb(10, 10, 12),
    Color::Rgb(225, 225, 225),
    Color::Rgb(121, 121, 121),
    Color::Rgb(93, 93, 93),
);

/// Dracula — mono chrome + Dracula purple.
const DRACULA: Theme = mono(
    Color::Rgb(189, 147, 249),
    Color::Rgb(40, 42, 54),
    Color::Rgb(235, 235, 240),
    Color::Rgb(145, 145, 145),
    Color::Rgb(115, 115, 115),
);

/// Nord — mono chrome + arctic ice blue.
const NORD: Theme = mono(
    Color::Rgb(136, 192, 208),
    Color::Rgb(46, 52, 64),
    Color::Rgb(222, 228, 235),
    Color::Rgb(156, 156, 156),
    Color::Rgb(125, 125, 125),
);

/// Gruvbox — mono chrome + warm gruvbox orange.
/// Like Amber, the orange accent collides with the mono warn orange
/// (ΔE76 22.5) — warn moves to the alert-red family.
const GRUVBOX: Theme = Theme {
    warn: Color::Rgb(255, 90, 90),
    warn_hi: Color::Rgb(255, 90, 90),
    ..mono(
        Color::Rgb(254, 128, 25),
        Color::Rgb(40, 40, 40),
        Color::Rgb(230, 220, 200),
        Color::Rgb(142, 142, 142),
        Color::Rgb(113, 113, 113),
    )
};

/// Solarized — mono chrome + solarized teal.
const SOLARIZED: Theme = mono(
    Color::Rgb(42, 161, 152),
    Color::Rgb(0, 43, 54),
    Color::Rgb(200, 210, 205),
    Color::Rgb(141, 141, 141),
    Color::Rgb(112, 112, 112),
);

/// Tokyo Night — mono chrome + Tokyo Night blue.
const TOKYO_NIGHT: Theme = mono(
    Color::Rgb(122, 162, 247),
    Color::Rgb(26, 27, 38),
    Color::Rgb(215, 220, 240),
    Color::Rgb(131, 131, 131),
    Color::Rgb(103, 103, 103),
);

/// Synthwave — mono chrome + neon pink.
const SYNTHWAVE: Theme = mono(
    Color::Rgb(255, 113, 206),
    Color::Rgb(26, 11, 46),
    Color::Rgb(235, 225, 245),
    Color::Rgb(125, 125, 125),
    Color::Rgb(98, 98, 98),
);

/// Ocean — mono chrome + deep sea blue.
const OCEAN: Theme = mono(
    Color::Rgb(0, 170, 255),
    Color::Rgb(10, 25, 40),
    Color::Rgb(210, 225, 240),
    Color::Rgb(129, 129, 129),
    Color::Rgb(100, 100, 100),
);

/// Crimson — mono chrome + alert red.
/// Exception to the mono error red: Crimson's accent IS red — a red error
/// would collide with the active state, so error stays White here.
const CRIMSON: Theme = Theme {
    error: Color::White,
    error_hi: Color::White,
    ..mono(
        Color::Rgb(255, 70, 85),
        Color::Rgb(26, 5, 8),
        Color::Rgb(240, 215, 215),
        Color::Rgb(121, 121, 121),
        Color::Rgb(94, 94, 94),
    )
};

/// Glass family — the mono template with NO painted background, tuned for
/// translucent / "glassy" terminals over a dark desktop (blur, acrylic, a
/// background image). Same hierarchy as `mono`, but every ink is brighter:
/// a blurred glass background is LIGHTER than a painted near-black canvas,
/// so text/dim/dim2 carry extra luminance to stay readable through the
/// transparency. Audited against pure black in the contrast tests — the
/// darkest the glass can ever be; any lighter blur only gains contrast on
/// these bright inks. (For mono inks without an accent, see the existing
/// Transparent Dark / Transparent Light pair.)
const fn glass(accent: Color) -> Theme {
    Theme {
        bg: None,
        text: Color::Rgb(235, 235, 235),
        ..mk(
            accent,
            lighten(accent),           // accent2 — separators / selected-field bg
            accent,                    // success — active states wear the accent
            Color::Rgb(255, 110, 110), // error — alert red
            Color::Rgb(255, 165, 0),   // warn — blocked/attention badge
            Color::Rgb(190, 190, 190), // info
            lighten(accent),           // special
            Color::Rgb(150, 150, 150), // dim — brighter than painted-bg themes
            Color::Rgb(115, 115, 115), // dim2
            Color::White,              // bright
            Color::Black,              // sel_fg
        )
    }
}

/// Glass Cyan — transparent + electric cyan inks.
const GLASS_CYAN: Theme = glass(Color::Rgb(0, 255, 255));

/// Glass Green — transparent + neon green inks.
const GLASS_GREEN: Theme = glass(Color::Rgb(0, 255, 65));

/// Glass Amber — transparent + amber inks. Like Amber/Gruvbox, the orange
/// accent collides with the mono warn orange — warn moves to the alert-red
/// family (glyphs disambiguate warn from error within the Alerte class).
const GLASS_AMBER: Theme = Theme {
    warn: Color::Rgb(255, 90, 90),
    warn_hi: Color::Rgb(255, 90, 90),
    ..glass(Color::Rgb(255, 176, 0))
};

/// Glass Purple — transparent + Dracula-purple inks.
const GLASS_PURPLE: Theme = glass(Color::Rgb(189, 147, 249));

/// Glass Pink — transparent + neon pink inks.
const GLASS_PINK: Theme = glass(Color::Rgb(255, 113, 206));

/// Glass Blue — transparent + azure inks.
const GLASS_BLUE: Theme = glass(Color::Rgb(0, 170, 255));

// ── Registry ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ThemeId {
    Omega = 0,
    Matrix = 1,
    Terminal = 2,
    Amber = 3,
    Noir = 4,
    Paper = 5,
    Monogram = 6,
    Dracula = 7,
    Nord = 8,
    Gruvbox = 9,
    Solarized = 10,
    TokyoNight = 11,
    Synthwave = 12,
    Ocean = 13,
    Crimson = 14,
    TransparentDark = 15,
    TransparentLight = 16,
    GlassCyan = 17,
    GlassGreen = 18,
    GlassAmber = 19,
    GlassPurple = 20,
    GlassPink = 21,
    GlassBlue = 22,
}

impl ThemeId {
    pub fn all() -> &'static [ThemeId] {
        &[
            ThemeId::Omega,
            ThemeId::Matrix,
            ThemeId::Terminal,
            ThemeId::Amber,
            ThemeId::Noir,
            ThemeId::Paper,
            ThemeId::Monogram,
            ThemeId::Dracula,
            ThemeId::Nord,
            ThemeId::Gruvbox,
            ThemeId::Solarized,
            ThemeId::TokyoNight,
            ThemeId::Synthwave,
            ThemeId::Ocean,
            ThemeId::Crimson,
            ThemeId::TransparentDark,
            ThemeId::TransparentLight,
            ThemeId::GlassCyan,
            ThemeId::GlassGreen,
            ThemeId::GlassAmber,
            ThemeId::GlassPurple,
            ThemeId::GlassPink,
            ThemeId::GlassBlue,
        ]
    }

    /// Stable slug persisted in config.toml (`theme = "..."`).
    pub fn slug(self) -> &'static str {
        match self {
            ThemeId::Omega => "omega",
            ThemeId::Matrix => "matrix",
            ThemeId::Terminal => "terminal",
            ThemeId::Amber => "amber",
            ThemeId::Noir => "noir",
            ThemeId::Paper => "paper",
            ThemeId::Monogram => "monogram",
            ThemeId::Dracula => "dracula",
            ThemeId::Nord => "nord",
            ThemeId::Gruvbox => "gruvbox",
            ThemeId::Solarized => "solarized",
            ThemeId::TokyoNight => "tokyo-night",
            ThemeId::Synthwave => "synthwave",
            ThemeId::Ocean => "ocean",
            ThemeId::Crimson => "crimson",
            ThemeId::TransparentDark => "transparent-dark",
            ThemeId::TransparentLight => "transparent-light",
            ThemeId::GlassCyan => "glass-cyan",
            ThemeId::GlassGreen => "glass-green",
            ThemeId::GlassAmber => "glass-amber",
            ThemeId::GlassPurple => "glass-purple",
            ThemeId::GlassPink => "glass-pink",
            ThemeId::GlassBlue => "glass-blue",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ThemeId::Omega => "Omega (default)",
            ThemeId::Matrix => "Matrix",
            ThemeId::Terminal => "Terminal (green phosphor)",
            ThemeId::Amber => "Amber (retro phosphor)",
            ThemeId::Noir => "Noir (full black)",
            ThemeId::Paper => "Paper (full white)",
            ThemeId::Monogram => "Monogram (mono + cyan)",
            ThemeId::Dracula => "Dracula",
            ThemeId::Nord => "Nord",
            ThemeId::Gruvbox => "Gruvbox",
            ThemeId::Solarized => "Solarized Dark",
            ThemeId::TokyoNight => "Tokyo Night",
            ThemeId::Synthwave => "Synthwave",
            ThemeId::Ocean => "Ocean",
            ThemeId::Crimson => "Crimson",
            ThemeId::TransparentDark => "Transparent Dark",
            ThemeId::TransparentLight => "Transparent Light",
            ThemeId::GlassCyan => "Glass Cyan",
            ThemeId::GlassGreen => "Glass Green",
            ThemeId::GlassAmber => "Glass Amber",
            ThemeId::GlassPurple => "Glass Purple",
            ThemeId::GlassPink => "Glass Pink",
            ThemeId::GlassBlue => "Glass Blue",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            ThemeId::Omega => "Classic cyan/yellow — adapts to your terminal's own scheme",
            ThemeId::Matrix => "Mono chrome + neon Matrix green",
            ThemeId::Terminal => "Mono chrome + soft phosphor green",
            ThemeId::Amber => "Mono chrome + retro amber",
            ThemeId::Noir => "Pure grayscale for dark terminals",
            ThemeId::Paper => "Ink on paper — for LIGHT terminal backgrounds",
            ThemeId::Monogram => "Monochrome chrome, one cyan accent",
            ThemeId::Dracula => "Mono chrome + Dracula purple",
            ThemeId::Nord => "Mono chrome + arctic ice blue",
            ThemeId::Gruvbox => "Mono chrome + warm gruvbox orange",
            ThemeId::Solarized => "Mono chrome + solarized teal",
            ThemeId::TokyoNight => "Mono chrome + Tokyo Night blue",
            ThemeId::Synthwave => "Mono chrome + neon pink",
            ThemeId::Ocean => "Mono chrome + deep sea blue",
            ThemeId::Crimson => "Mono chrome + alert red",
            ThemeId::TransparentDark => "NEEDS A DARK TERMINAL — white ink, no painted bg",
            ThemeId::TransparentLight => "NEEDS A LIGHT TERMINAL — black ink, no painted bg",
            ThemeId::GlassCyan => "NEEDS A DARK TERMINAL — cyan accent, no painted bg",
            ThemeId::GlassGreen => "NEEDS A DARK TERMINAL — neon green, no painted bg",
            ThemeId::GlassAmber => "NEEDS A DARK TERMINAL — amber, no painted bg",
            ThemeId::GlassPurple => "NEEDS A DARK TERMINAL — purple, no painted bg",
            ThemeId::GlassPink => "NEEDS A DARK TERMINAL — neon pink, no painted bg",
            ThemeId::GlassBlue => "NEEDS A DARK TERMINAL — azure, no painted bg",
        }
    }

    pub fn palette(self) -> &'static Theme {
        match self {
            ThemeId::Omega => &OMEGA,
            ThemeId::Matrix => &MATRIX,
            ThemeId::Terminal => &TERMINAL,
            ThemeId::Amber => &AMBER,
            ThemeId::Noir => &NOIR,
            ThemeId::Paper => &PAPER,
            ThemeId::Monogram => &MONOGRAM,
            ThemeId::Dracula => &DRACULA,
            ThemeId::Nord => &NORD,
            ThemeId::Gruvbox => &GRUVBOX,
            ThemeId::Solarized => &SOLARIZED,
            ThemeId::TokyoNight => &TOKYO_NIGHT,
            ThemeId::Synthwave => &SYNTHWAVE,
            ThemeId::Ocean => &OCEAN,
            ThemeId::Crimson => &CRIMSON,
            ThemeId::TransparentDark => &TRANSPARENT_DARK,
            ThemeId::TransparentLight => &TRANSPARENT_LIGHT,
            ThemeId::GlassCyan => &GLASS_CYAN,
            ThemeId::GlassGreen => &GLASS_GREEN,
            ThemeId::GlassAmber => &GLASS_AMBER,
            ThemeId::GlassPurple => &GLASS_PURPLE,
            ThemeId::GlassPink => &GLASS_PINK,
            ThemeId::GlassBlue => &GLASS_BLUE,
        }
    }

    pub fn from_slug(s: &str) -> Option<ThemeId> {
        ThemeId::all().iter().copied().find(|t| t.slug() == s)
    }

    fn from_u8(n: u8) -> ThemeId {
        ThemeId::all()
            .get(n as usize)
            .copied()
            .unwrap_or(ThemeId::Omega)
    }
}

// ── Global active theme ─────────────────────────────────────────────────────

static ACTIVE: AtomicU8 = AtomicU8::new(ThemeId::Omega as u8);

/// Switch the active theme — takes effect on the next rendered frame.
pub fn set_active(id: ThemeId) {
    ACTIVE.store(id as u8, Ordering::Relaxed);
}

/// Set the active theme from a config slug; unknown slugs fall back to Omega.
pub fn set_active_slug(slug: &str) {
    set_active(ThemeId::from_slug(slug).unwrap_or(ThemeId::Omega));
}

pub fn active() -> ThemeId {
    ThemeId::from_u8(ACTIVE.load(Ordering::Relaxed))
}

fn cur() -> &'static Theme {
    active().palette()
}

// Semantic accessors — the only color API ui.rs should use for chrome.
/// Full-screen background override; `None` keeps the terminal's own bg.
pub fn bg() -> Option<Color> {
    cur().bg
}
/// Body text color (Reset for bg-less themes, explicit otherwise).
pub fn text() -> Color {
    cur().text
}
pub fn accent() -> Color {
    cur().accent
}
pub fn accent2() -> Color {
    cur().accent2
}
pub fn success() -> Color {
    cur().success
}
pub fn error() -> Color {
    cur().error
}
pub fn warn() -> Color {
    cur().warn
}
pub fn info() -> Color {
    cur().info
}
pub fn special() -> Color {
    cur().special
}
pub fn dim() -> Color {
    cur().dim
}
pub fn dim2() -> Color {
    cur().dim2
}
pub fn bright() -> Color {
    cur().bright
}
pub fn sel_fg() -> Color {
    cur().sel_fg
}
pub fn accent_hi() -> Color {
    cur().accent_hi
}
pub fn accent2_hi() -> Color {
    cur().accent2_hi
}
pub fn success_hi() -> Color {
    cur().success_hi
}
pub fn error_hi() -> Color {
    cur().error_hi
}
pub fn warn_hi() -> Color {
    cur().warn_hi
}
pub fn info_hi() -> Color {
    cur().info_hi
}
pub fn special_hi() -> Color {
    cur().special_hi
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_roundtrip() {
        for id in ThemeId::all() {
            assert_eq!(
                ThemeId::from_slug(id.slug()),
                Some(*id),
                "slug {}",
                id.slug()
            );
        }
        assert_eq!(ThemeId::from_slug("nope"), None);
    }

    #[test]
    fn set_active_slug_falls_back_to_omega() {
        set_active(ThemeId::Matrix);
        set_active_slug("does-not-exist");
        assert_eq!(active(), ThemeId::Omega);
    }

    #[test]
    fn twenty_three_themes() {
        assert_eq!(ThemeId::all().len(), 23);
    }

    /// A bg-less theme with fixed RGB inks is only readable on the terminal
    /// background it was DESIGNED for — `contrast_contract` audits it against
    /// an ASSUMED bg (black or white), which the runtime never verifies. Pick
    /// the wrong one and the chrome drops to ~1.1:1: invisible. So every such
    /// theme MUST state the terminal it needs, in the blurb the picker renders
    /// next to its name. Omega is exempt: named ANSI roles, no fixed ink, it
    /// genuinely adapts to whatever palette the terminal carries.
    #[test]
    fn bg_less_themes_declare_the_terminal_they_need() {
        for id in ThemeId::all() {
            if id.palette().bg.is_some() || *id == ThemeId::Omega {
                continue;
            }
            let blurb = id.blurb();
            assert!(
                blurb.starts_with("NEEDS A DARK TERMINAL")
                    || blurb.starts_with("NEEDS A LIGHT TERMINAL"),
                "{}: paints no background, so its blurb must open with the \
                 terminal background it requires — got {blurb:?}",
                id.slug()
            );
        }
    }

    // ── WCAG 2.x contrast contract ──────────────────────────────────────────
    // The math lives here, not in the const palettes: themes carry literal
    // Rgb values and this test is the enforcement.

    /// Channel values for the colors that appear in palettes. Named
    /// White/Black (Noir, Crimson's error) get their nominal sRGB values —
    /// close enough for a contract on themes that paint their own bg.
    fn rgb_of(c: Color) -> (u8, u8, u8) {
        match c {
            Color::Rgb(r, g, b) => (r, g, b),
            Color::White => (255, 255, 255),
            Color::Black => (0, 0, 0),
            other => panic!("contrast contract: {other:?} has no fixed luminance"),
        }
    }

    /// sRGB channel linearization (shared by luminance and Lab).
    fn srgb_lin(v: u8) -> f64 {
        let v = v as f64 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    /// WCAG 2.x relative luminance.
    fn relative_luminance(c: Color) -> f64 {
        let (r, g, b) = rgb_of(c);
        0.2126 * srgb_lin(r) + 0.7152 * srgb_lin(g) + 0.0722 * srgb_lin(b)
    }

    /// sRGB → CIE Lab (D65) — for the role-vs-role distinctness floor.
    fn lab_of(c: Color) -> (f64, f64, f64) {
        let (r, g, b) = rgb_of(c);
        let (r, g, b) = (srgb_lin(r), srgb_lin(g), srgb_lin(b));
        let x = (0.4124564 * r + 0.3575761 * g + 0.1804375 * b) / 0.95047;
        let y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
        let z = (0.0193339 * r + 0.1191920 * g + 0.9503041 * b) / 1.08883;
        fn f(t: f64) -> f64 {
            if t > 0.008856 {
                t.cbrt()
            } else {
                7.787 * t + 16.0 / 116.0
            }
        }
        (
            116.0 * f(y) - 16.0,
            500.0 * (f(x) - f(y)),
            200.0 * (f(y) - f(z)),
        )
    }

    /// CIE76 color difference — perceptual distance between two roles.
    fn delta_e76(a: Color, b: Color) -> f64 {
        let (l1, a1, b1) = lab_of(a);
        let (l2, a2, b2) = lab_of(b);
        ((l1 - l2).powi(2) + (a1 - a2).powi(2) + (b1 - b2).powi(2)).sqrt()
    }

    fn contrast_ratio(a: Color, b: Color) -> f64 {
        let (la, lb) = (relative_luminance(a), relative_luminance(b));
        let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
        (hi + 0.05) / (lo + 0.05)
    }

    /// The contract from the module doc: AA thresholds per role, plus the
    /// dim hierarchy. A failing theme is fixed by tuning the palette VALUE,
    /// never by relaxing a threshold here.
    #[test]
    fn contrast_contract() {
        for id in ThemeId::all() {
            let t = id.palette();
            let bg = match t.bg {
                Some(bg) => bg,
                // Omega is exempt: bg = None and its roles are named ANSI
                // colors that adapt to the terminal's own palette — there is
                // no fixed luminance to audit. The transparent themes also
                // paint no bg but their inks are fixed RGB, so they are
                // audited against the terminal background they are DESIGNED
                // for (dark → black, light → white).
                None => match *id {
                    ThemeId::Omega => continue,
                    ThemeId::TransparentDark => Color::Rgb(0, 0, 0),
                    ThemeId::TransparentLight => Color::Rgb(255, 255, 255),
                    // Glass family: designed for dark translucent terminals —
                    // audited against pure black, the darkest the glass can
                    // be; a lighter blur only gains contrast on bright inks.
                    ThemeId::GlassCyan
                    | ThemeId::GlassGreen
                    | ThemeId::GlassAmber
                    | ThemeId::GlassPurple
                    | ThemeId::GlassPink
                    | ThemeId::GlassBlue => Color::Rgb(0, 0, 0),
                    other => panic!("{other:?}: bg-less theme needs an assumed audit bg"),
                },
            };
            let vs_bg = |c: Color| contrast_ratio(c, bg);
            let check = |role: &str, c: Color, min: f64| {
                let r = vs_bg(c);
                assert!(
                    r >= min,
                    "{}: {role} {:?} is {r:.2}:1 vs bg, needs >= {min}:1",
                    id.slug(),
                    c
                );
            };
            // 4.5:1 — body-level legibility.
            check("text", t.text, 4.5);
            check("dim", t.dim, 4.5);
            check("info", t.info, 4.5);
            check("error", t.error, 4.5);
            check("warn", t.warn, 4.5);
            check("bright", t.bright, 4.5);
            // sel_fg renders on accent-colored selection bars, not on bg.
            let sel = contrast_ratio(t.sel_fg, t.accent);
            assert!(
                sel >= 4.5,
                "{}: sel_fg on accent is {sel:.2}:1, needs >= 4.5:1",
                id.slug()
            );
            // sel_fg also renders on accent2 backgrounds (selected settings
            // fields, AISB agent row, input-mode chip — `.bg(th::accent2())`).
            let sel2 = contrast_ratio(t.sel_fg, t.accent2);
            assert!(
                sel2 >= 4.5,
                "{}: sel_fg on accent2 is {sel2:.2}:1, needs >= 4.5:1",
                id.slug()
            );
            // 4.5:1 — these roles paint readable text at dozens of ui.rs
            // sites (labels, separators, active states), so they carry the
            // body-text floor, not the large/graphical 3.0.
            check("accent", t.accent, 4.5);
            check("accent2", t.accent2, 4.5);
            check("success", t.success, 4.5);
            check("special", t.special, 4.5);
            // 3.0:1 — dim2 has no text call sites; it only anchors the
            // gray hierarchy below.
            check("dim2", t.dim2, 3.0);
            // Role-vs-role distinctness: an alert must read as a DIFFERENT
            // state than the active accent, not just clear the bg (the axis
            // that let 'Amber warn ≡ accent' ship green). Noir and Paper are
            // exempt — mono by design, the badge glyphs (+ ~ x !) carry state.
            if !matches!(
                *id,
                ThemeId::Noir
                    | ThemeId::Paper
                    | ThemeId::TransparentDark
                    | ThemeId::TransparentLight
            ) {
                let dw = delta_e76(t.warn, t.accent);
                assert!(
                    dw >= 30.0,
                    "{}: warn vs accent ΔE76 is {dw:.1}, needs >= 30",
                    id.slug()
                );
                let de = delta_e76(t.error, t.accent);
                assert!(
                    de >= 30.0,
                    "{}: error vs accent ΔE76 is {de:.1}, needs >= 30",
                    id.slug()
                );
            }
            // Gray hierarchy: the three quiet levels stay visually ordered.
            let (r2, r1, rt) = (vs_bg(t.dim2), vs_bg(t.dim), vs_bg(t.text));
            assert!(
                r2 < r1 && r1 < rt,
                "{}: dim hierarchy broken: dim2 {r2:.2} < dim {r1:.2} < text {rt:.2}",
                id.slug()
            );
        }
    }
}
