//! Provider-aware rmux preview reflow.
//!
//! A pane and its outer TUI panel do not resize atomically. This module keeps
//! the transient frame readable without changing rmux or inventing a second
//! input buffer: captured rows are reflowed by grapheme, their ANSI styles are
//! retained, and the source cursor is mapped into the current viewport.

use crate::theme as th;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

pub(crate) struct ReflowedPreview {
    pub(crate) lines: Vec<Line<'static>>,
    pub(crate) source_row_starts: Vec<u16>,
}

fn flush_span(spans: &mut Vec<Span<'static>>, text: &mut String, style: &mut Option<Style>) {
    if let Some(span_style) = style.take() {
        if !text.is_empty() {
            spans.push(Span::styled(std::mem::take(text), span_style));
        }
    }
}

/// Hard-wrap captured terminal rows while preserving grapheme clusters and
/// styles. Every returned line is bounded by `width` display cells.
pub(crate) fn reflow_lines(lines: &[Line<'_>], width: u16) -> ReflowedPreview {
    use unicode_width::UnicodeWidthStr;

    let width = usize::from(width.max(1));
    let mut out = Vec::with_capacity(lines.len());
    let mut source_row_starts = Vec::with_capacity(lines.len());

    for line in lines {
        source_row_starts.push(out.len().min(u16::MAX as usize) as u16);
        let mut spans = Vec::new();
        let mut span_text = String::new();
        let mut span_style = None;
        let mut col = 0usize;

        for grapheme in line.styled_graphemes(Style::default()) {
            let measured = UnicodeWidthStr::width(grapheme.symbol);
            // A two-cell cluster cannot fit in a one-cell panel. Keep it atomic
            // and use a visible one-cell fallback instead of invalid Unicode.
            let (symbol, grapheme_width) = if measured > width {
                ("�", 1)
            } else {
                (grapheme.symbol, measured)
            };
            if col > 0 && col.saturating_add(grapheme_width) > width {
                flush_span(&mut spans, &mut span_text, &mut span_style);
                out.push(Line::from(std::mem::take(&mut spans)));
                col = 0;
            }
            if span_style != Some(grapheme.style) {
                flush_span(&mut spans, &mut span_text, &mut span_style);
                span_style = Some(grapheme.style);
            }
            span_text.push_str(symbol);
            col = col.saturating_add(grapheme_width);
        }

        flush_span(&mut spans, &mut span_text, &mut span_style);
        // An empty source row is still one terminal row.
        out.push(Line::from(spans));
    }

    ReflowedPreview {
        lines: out,
        source_row_starts,
    }
}

/// Translate an rmux `(row, display column)` cursor into reflowed coordinates.
pub(crate) fn reflow_cursor(
    source_lines: &[Line<'_>],
    source_row_starts: &[u16],
    source_row: u16,
    source_col: u16,
    width: u16,
) -> (u16, u16) {
    use unicode_width::UnicodeWidthStr;

    let width = usize::from(width.max(1));
    let row_index = usize::from(source_row).min(source_lines.len().saturating_sub(1));
    let row_start = source_row_starts.get(row_index).copied().unwrap_or(0);
    let target_col = usize::from(source_col);
    let mut source_cols = 0usize;
    let mut row = 0usize;
    let mut col = 0usize;

    if let Some(line) = source_lines.get(row_index) {
        for grapheme in line.styled_graphemes(Style::default()) {
            let source_width = UnicodeWidthStr::width(grapheme.symbol);
            if source_cols.saturating_add(source_width) > target_col {
                break;
            }
            let display_width = source_width.min(width);
            if col > 0 && col.saturating_add(display_width) > width {
                row = row.saturating_add(1);
                col = 0;
            }
            col = col.saturating_add(display_width);
            source_cols = source_cols.saturating_add(source_width);
        }
    }

    // Unstyled trailing blanks are absent from snapshot spans. Advance through
    // that gap so the caret stays at the true insertion column.
    for _ in source_cols..target_col {
        if col >= width {
            row = row.saturating_add(1);
            col = 0;
        }
        col = col.saturating_add(1);
    }
    if col >= width {
        row = row.saturating_add(col / width);
        col %= width;
    }

    (
        row_start.saturating_add(row.min(u16::MAX as usize) as u16),
        col.min(u16::MAX as usize) as u16,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Provider {
    Claude,
    Codex,
    Gemini,
    Other,
}

impl Provider {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Claude => "CLAUDE",
            Self::Codex => "CODEX",
            Self::Gemini => "GEMINI",
            Self::Other => "AGENT",
        }
    }

    pub(crate) fn accent(self) -> Color {
        match self {
            Self::Claude => th::special(),
            Self::Codex => th::success(),
            Self::Gemini => th::accent(),
            Self::Other => th::accent2(),
        }
    }
}

/// Prefer the provider recorded by OmegaOS at session creation. Strong local
/// signals remain as a compatibility fallback for pre-marker sessions.
pub(crate) fn provider(
    persisted: Option<&str>,
    session_name: &str,
    model: Option<&str>,
    content: &str,
) -> Provider {
    fn classify_identity(value: &str, model_signal: bool) -> Option<Provider> {
        let lower = value.to_ascii_lowercase();
        let tokens: Vec<&str> = lower
            .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '.')
            .filter(|token| !token.is_empty())
            .collect();
        if tokens.iter().any(|token| {
            *token == "claude"
                || model_signal && matches!(*token, "anthropic" | "sonnet" | "opus" | "haiku")
        }) {
            Some(Provider::Claude)
        } else if tokens.iter().any(|token| {
            *token == "codex"
                || model_signal
                    && (matches!(*token, "openai" | "o1" | "o3" | "o4") || token.starts_with("gpt"))
        }) {
            Some(Provider::Codex)
        } else if tokens.contains(&"gemini") {
            Some(Provider::Gemini)
        } else {
            None
        }
    }

    persisted
        .and_then(|value| classify_identity(value, false))
        .or_else(|| classify_identity(session_name, false))
        .or_else(|| model.and_then(|value| classify_identity(value, true)))
        .or_else(|| {
            // Footer-only signals must be specific. Ordinary agent output often
            // discusses OpenAI, Anthropic, or Google and must not recolor the
            // session just because that prose happens to sit near the tail.
            let tail = content
                .lines()
                .rev()
                .take(6)
                .collect::<Vec<_>>()
                .join(" ")
                .to_ascii_lowercase();
            if tail.contains("gpt-") || tail.contains("codex cli") {
                Some(Provider::Codex)
            } else if tail.contains("gemini-") || tail.contains("gemini cli") {
                Some(Provider::Gemini)
            } else if tail.contains("claude code") {
                Some(Provider::Claude)
            } else {
                None
            }
        })
        .unwrap_or(Provider::Other)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::{App, SessionEntry, SessionFocus},
        ui::draw_sessions_right,
    };
    use omega_core::{
        config::OmegaConfig,
        session::{OmegaSession, PreviewColor, PreviewSpan},
    };
    use ratatui::{backend::TestBackend, style::Modifier, Terminal};

    fn text(lines: &[Line<'_>]) -> String {
        lines
            .iter()
            .flat_map(|line| line.spans.iter().map(|span| span.content.as_ref()))
            .collect()
    }

    fn app(name: &str, text: &str, cursor_col: u16) -> App {
        let mut app = App::new(OmegaConfig::default());
        app.session_focus = SessionFocus::Chat;
        app.sessions = vec![SessionEntry {
            session: OmegaSession::classify(name),
            progress: None,
            is_current: false,
            is_protected: false,
            tree_prefix: String::new(),
        }];
        app.preview_content = text.to_string();
        app.preview_styled = Some(vec![vec![PreviewSpan {
            text: text.to_string(),
            fg: Some(PreviewColor::Rgb(195, 147, 255)),
            bg: Some(PreviewColor::Rgb(30, 30, 30)),
            bold: false,
            dim: false,
            italic: false,
            underline: false,
        }]]);
        app.preview_cursor = Some((0, cursor_col, true));
        app
    }

    #[test]
    fn reflow_preserves_text_graphemes_width_and_style() {
        let family = "👩‍💻";
        let value = format!("{}{}e\u{301}{}", "x".repeat(17), family, "y".repeat(29));
        let style = Style::default()
            .fg(Color::Rgb(195, 147, 255))
            .bg(Color::Rgb(30, 30, 30))
            .add_modifier(Modifier::ITALIC);
        let source = vec![Line::from(Span::styled(value.clone(), style))];
        let out = reflow_lines(&source, 18);

        assert_eq!(text(&out.lines), value);
        assert!(out.lines.iter().all(|line| line.width() <= 18));
        assert!(out.lines.iter().flat_map(|line| &line.spans).all(|span| {
            span.style.fg == Some(Color::Rgb(195, 147, 255))
                && span.style.bg == Some(Color::Rgb(30, 30, 30))
                && span.style.add_modifier.contains(Modifier::ITALIC)
        }));

        let edge = reflow_lines(&[Line::from(format!("aaaaaaaaa{family}B"))], 10);
        assert_eq!(text(&edge.lines), format!("aaaaaaaaa{family}B"));
        assert_eq!(edge.lines[0].width(), 9, "the cluster must move whole");
    }

    #[test]
    fn cursor_tracks_wraps_source_rows_and_one_cell_fallback() {
        let source = vec![Line::from("x".repeat(25)), Line::from("second row")];
        let out = reflow_lines(&source, 10);
        assert_eq!(out.source_row_starts, vec![0, 3]);
        assert_eq!(
            reflow_cursor(&source, &out.source_row_starts, 0, 23, 10),
            (2, 3)
        );
        assert_eq!(
            reflow_cursor(&source, &out.source_row_starts, 1, 6, 10),
            (3, 6)
        );

        let tiny = reflow_lines(&[Line::from("界")], 1);
        assert_eq!(text(&tiny.lines), "�");
        assert_eq!(tiny.lines[0].width(), 1);
    }

    #[test]
    fn detects_primary_providers_from_strong_identity_or_footer_signals() {
        assert_eq!(
            provider(None, "oracle", Some("claude-opus-5"), ""),
            Provider::Claude
        );
        assert_eq!(
            provider(
                None,
                "research",
                None,
                "\n gpt-5.6-sol xhigh · Main [default]"
            ),
            Provider::Codex
        );
        assert_eq!(provider(None, "project-gemini", None, ""), Provider::Gemini);
        assert_eq!(
            provider(None, "research", None, "ordinary output"),
            Provider::Other
        );
        assert_eq!(
            provider(
                None,
                "research",
                None,
                "A report comparing OpenAI, Anthropic, and Google"
            ),
            Provider::Other,
            "ordinary prose must not masquerade as a provider footer"
        );
        assert_eq!(
            provider(Some("codex"), "oracle", Some("claude-opus-5"), ""),
            Provider::Codex,
            "persisted provider must outrank heuristic model text"
        );
    }

    #[test]
    fn primary_providers_keep_ansi_and_cursor_visible_when_narrow_or_resized() {
        for (name, label) in [
            ("claude-session", "CLAUDE"),
            ("codex-session", "CODEX"),
            ("gemini-session", "GEMINI"),
        ] {
            let value = "x".repeat(150);
            let mut app = app(name, &value, 120);
            for (width, expected_inner) in [(80, 78), (40, 38)] {
                let mut terminal = Terminal::new(TestBackend::new(width, 8)).unwrap();
                terminal
                    .draw(|frame| {
                        let area = frame.area();
                        draw_sessions_right(frame, &mut app, area, true);
                    })
                    .unwrap();

                let cursor = terminal.get_cursor_position().unwrap();
                assert!(cursor.x > 0 && cursor.x < width - 1);
                assert!(cursor.y > 0 && cursor.y < 7);
                assert_eq!(
                    terminal
                        .backend()
                        .buffer()
                        .cell(cursor)
                        .map(|cell| cell.symbol()),
                    Some("▏")
                );
                assert_eq!(app.preview_inner_width, expected_inner);
                let top = (0..width)
                    .filter_map(|x| {
                        terminal
                            .backend()
                            .buffer()
                            .cell((x, 0))
                            .map(|cell| cell.symbol())
                    })
                    .collect::<String>();
                assert!(top.contains(label));
                assert!(terminal.backend().buffer().content.iter().any(|cell| {
                    cell.symbol() == "x"
                        && cell.fg == Color::Rgb(195, 147, 255)
                        && cell.bg == Color::Rgb(30, 30, 30)
                }));
            }
        }
    }
}
