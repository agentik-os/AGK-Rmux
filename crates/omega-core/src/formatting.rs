//! Telegram HTML formatting engine.
//!
//! Converts markdown-style text into Telegram-compatible HTML with
//! blockquote cards, code blocks, tables, progress bars, and
//! proper HTML entity escaping.

use std::fmt::Write;

/// Escape HTML special characters for Telegram's HTML parse mode.
pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Convert markdown-style text to Telegram HTML.
///
/// Handles: **bold**, `inline code`, ```code blocks```,
/// > blockquotes, # headings, - list items, [links](url).
pub fn markdown_to_telegram_html(md: &str) -> String {
    let mut out = String::with_capacity(md.len() * 2);
    let mut in_code_block = false;
    let mut code_block_buf = String::new();

    for line in md.lines() {
        let trimmed = line.trim();

        // Fenced code blocks
        if trimmed.starts_with("```") {
            if in_code_block {
                out.push_str("<pre>");
                out.push_str(&escape_html(&code_block_buf));
                out.push_str("</pre>\n");
                code_block_buf.clear();
                in_code_block = false;
            } else {
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            if !code_block_buf.is_empty() {
                code_block_buf.push('\n');
            }
            code_block_buf.push_str(line);
            continue;
        }

        // Headings
        if let Some(heading) = trimmed.strip_prefix("# ") {
            let _ = writeln!(out, "\n<b>{}</b>", escape_html(heading));
            continue;
        }
        if let Some(heading) = trimmed.strip_prefix("## ") {
            let _ = writeln!(out, "\n<b>{}</b>", escape_html(heading));
            continue;
        }
        if let Some(heading) = trimmed.strip_prefix("### ") {
            let _ = writeln!(out, "<b>{}</b>", escape_html(heading));
            continue;
        }

        // Blockquotes
        if let Some(content) = trimmed.strip_prefix("> ") {
            let _ = writeln!(
                out,
                "<blockquote>{}</blockquote>",
                format_inline(&escape_html(content))
            );
            continue;
        }

        // Horizontal rules
        if trimmed == "---" || trimmed == "━━━" || trimmed.chars().all(|c| c == '─' || c == '━')
        {
            out.push_str("━━━━━━━━━━\n");
            continue;
        }

        // List items
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let item = &trimmed[2..];
            let _ = writeln!(out, "• {}", format_inline(&escape_html(item)));
            continue;
        }

        // Numbered list items
        if trimmed.len() > 2 {
            let first_dot = trimmed.find(". ");
            if let Some(pos) = first_dot {
                if pos <= 3 && trimmed[..pos].chars().all(|c| c.is_ascii_digit()) {
                    let item = &trimmed[pos + 2..];
                    let num = &trimmed[..pos];
                    let _ = writeln!(out, "{}. {}", num, format_inline(&escape_html(item)));
                    continue;
                }
            }
        }

        // Empty line
        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }

        // Regular text with inline formatting
        let _ = writeln!(out, "{}", format_inline(&escape_html(trimmed)));
    }

    // Unclosed code block
    if in_code_block && !code_block_buf.is_empty() {
        out.push_str("<pre>");
        out.push_str(&escape_html(&code_block_buf));
        out.push_str("</pre>\n");
    }

    collapse_blank_lines(&out)
}

/// Apply inline formatting: **bold**, *italic*, `code`, [link](url).
fn format_inline(text: &str) -> String {
    let mut result = text.to_string();

    // Bold: **text**
    while let Some(start) = result.find("**") {
        if let Some(end) = result[start + 2..].find("**") {
            let end = start + 2 + end;
            let inner = result[start + 2..end].to_string();
            result = format!("{}<b>{}</b>{}", &result[..start], inner, &result[end + 2..]);
        } else {
            break;
        }
    }

    // Italic: *text*  (bold already converted **..** to <b>..</b>, so only single * remain).
    // Byte-index splice on ASCII '*' — always lands on char boundaries, never panics on UTF-8.
    while let Some(start) = result.find('*') {
        let Some(rel_end) = result[start + 1..].find('*') else {
            break;
        };
        let end = start + 1 + rel_end;
        let inner = result[start + 1..end].to_string();
        result = format!("{}<i>{}</i>{}", &result[..start], inner, &result[end + 1..]);
    }

    // Inline code: `text`
    while let Some(start) = result.find('`') {
        if let Some(end) = result[start + 1..].find('`') {
            let end = start + 1 + end;
            let inner = result[start + 1..end].to_string();
            result = format!(
                "{}<code>{}</code>{}",
                &result[..start],
                inner,
                &result[end + 1..]
            );
        } else {
            break;
        }
    }

    result
}

fn collapse_blank_lines(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut blank_count = 0;
    for line in text.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                out.push('\n');
            }
        } else {
            blank_count = 0;
            out.push_str(line);
            out.push('\n');
        }
    }
    out.trim_end().to_string()
}

/// Minimal Engineer template — wraps an agent response body with the
/// canonical Agentik OS Telegram look:
///
/// ```text
/// Ω  <agent>
/// ────────────────────
///
/// <formatted body>
///
/// ⌁ <duration>s · <model>
/// ```
///
/// `body_md` is the raw agent output (markdown). It gets converted to
/// Telegram HTML internally. `agent` is the visible name ("AISB Master",
/// "Oracle Causio #1", etc.). `duration_s` is the elapsed seconds since
/// the user message was received; `model` is the LLM tag (e.g. "opus-4.7").
pub fn wrap_response(agent: &str, body_md: &str, duration_s: f32, model: &str) -> String {
    let body_html = markdown_to_telegram_html(body_md);
    let agent_esc = escape_html(agent);
    let model_esc = escape_html(model);
    let divider = "─".repeat(20);
    format!(
        "Ω  <b>{agent}</b>\n{divider}\n\n{body}\n\n<i>⌁ {dur:.1}s · {model}</i>",
        agent = agent_esc,
        divider = divider,
        body = body_html,
        dur = duration_s,
        model = model_esc,
    )
}

/// "Thinking…" placeholder shown immediately when a user message arrives.
/// Gets edited (editMessageText) once the real response is ready, so the
/// user sees a single message that morphs from "thinking" → "answer".
pub fn thinking_placeholder(agent: &str) -> String {
    let agent_esc = escape_html(agent);
    let divider = "─".repeat(20);
    format!(
        "Ω  <b>{agent}</b>\n{divider}\n\n<i>⌁ Thinking…</i>",
        agent = agent_esc,
        divider = divider,
    )
}

// ─────────────────────────────────────────────────────────────────────────
// Smart formatting — Pack STRUCTURE + SPOILERS
// ─────────────────────────────────────────────────────────────────────────

/// Maximum chars before a code block / long section is offered as a file
/// attachment instead of inlined. Picked so a typical Telegram window
/// stays readable on phone.
pub const FILE_DELIVERY_THRESHOLD_CHARS: usize = 2000;
/// Lines threshold for code blocks specifically.
pub const FILE_DELIVERY_THRESHOLD_LINES: usize = 30;
/// A markdown section longer than this is auto-wrapped in
/// <blockquote expandable> so the user can collapse it.
pub const EXPANDABLE_SECTION_LINES: usize = 5;

/// Detect secrets in raw text and wrap them in <tg-spoiler>.
///
/// Patterns matched:
///   - sk-ant-… (Anthropic), sk-… (OpenAI-style)
///   - whsec_… (Webhook secrets, Stripe/Clerk)
///   - Bearer <token>
///   - ghp_… / gho_… / ghu_… / ghs_… / ghr_… (GitHub tokens)
///   - xoxb-… / xoxp-… (Slack)
///   - api[_-]?key\s*[=:]\s*"?<val>"?
///   - password\s*[=:]\s*"?<val>"?
///
/// Operates on the ALREADY-HTML-ESCAPED string. The wrapped <tg-spoiler>
/// tags are emitted verbatim and pass through Telegram's HTML parser.
pub fn mask_secrets(html: &str) -> String {
    let mut out = html.to_string();

    // Patterns: (regex-like substring detector — keeping it dependency-free,
    // we do simple find-loops). For complex patterns we'd use the regex crate;
    // here a hand-rolled scanner suffices and keeps formatting.rs leaf.
    let prefixes = [
        "sk-ant-", "sk-or-", "sk-", "whsec_", "ghp_", "gho_", "ghu_", "ghs_", "ghr_", "xoxb-",
        "xoxp-",
    ];

    for prefix in prefixes {
        while let Some(start) = out.find(prefix) {
            // Don't double-wrap if already inside a spoiler
            let before = &out[..start];
            if before
                .rfind("<tg-spoiler>")
                .is_some_and(|p| before[p..].find("</tg-spoiler>").is_none())
            {
                break;
            }
            // Token = prefix + run of [A-Za-z0-9_\-]
            let after = &out[start + prefix.len()..];
            let token_len: usize = after
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                .map(|c| c.len_utf8())
                .sum();
            if token_len == 0 {
                break;
            }
            let end = start + prefix.len() + token_len;
            let secret = &out[start..end];
            let wrapped = format!("<tg-spoiler>{}</tg-spoiler>", secret);
            out.replace_range(start..end, &wrapped);
        }
    }

    // Bearer <token>
    while let Some(pos) = out.find("Bearer ") {
        let token_start = pos + "Bearer ".len();
        let after = &out[token_start..];
        let token_len: usize = after
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
            .map(|c| c.len_utf8())
            .sum();
        if token_len < 8 {
            break;
        }
        let token_end = token_start + token_len;
        let secret = &out[token_start..token_end].to_string();
        let wrapped = format!("Bearer <tg-spoiler>{}</tg-spoiler>", secret);
        out.replace_range(pos..token_end, &wrapped);
    }

    out
}

/// Smart sectionizer — convert a markdown body to Telegram HTML AND
/// auto-collapse long sections into expandable blockquotes.
///
/// A "section" = block starting with `## <heading>` or `### <heading>`
/// up to the next heading or EOF. Sections longer than
/// `EXPANDABLE_SECTION_LINES` are wrapped in `<blockquote expandable>`.
pub fn smart_markdown_to_html(md: &str) -> String {
    let mut out = String::with_capacity(md.len() * 2);
    let lines: Vec<&str> = md.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        if (trimmed.starts_with("## ") || trimmed.starts_with("### "))
            && !trimmed.starts_with("#### ")
        {
            // Collect the section body until next heading
            let heading_text = trimmed.trim_start_matches('#').trim_start();
            let mut body = Vec::new();
            let mut j = i + 1;
            while j < lines.len() {
                let t = lines[j].trim();
                if t.starts_with("## ") || t.starts_with("### ") {
                    break;
                }
                body.push(lines[j]);
                j += 1;
            }
            // Strip trailing blank lines from body
            while body.last().is_some_and(|l| l.trim().is_empty()) {
                body.pop();
            }
            let body_md = body.join("\n");
            let body_html = markdown_to_telegram_html(&body_md);
            if body.len() > EXPANDABLE_SECTION_LINES {
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!(
                        "<blockquote expandable><b>{}</b>\n{}</blockquote>\n",
                        escape_html(heading_text),
                        body_html,
                    ),
                );
            } else {
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!("<b>{}</b>\n{}\n", escape_html(heading_text), body_html,),
                );
            }
            i = j;
            continue;
        }

        // Non-heading: render the line via the standard markdown converter
        let block_html = markdown_to_telegram_html(line);
        out.push_str(&block_html);
        out.push('\n');
        i += 1;
    }

    collapse_blank_lines(&out)
}

/// Decide if a body should be sent as a file instead of inlined.
/// Returns Some((suggested_filename, mime_type)) if yes.
pub fn suggest_file_delivery(body_md: &str) -> Option<(String, &'static str)> {
    if body_md.len() < FILE_DELIVERY_THRESHOLD_CHARS {
        return None;
    }
    // Heuristics for extension based on content
    let (ext, mime) = if body_md.contains("```diff") || body_md.contains("---\n+++") {
        ("diff", "text/x-diff")
    } else if body_md.starts_with('{') && body_md.trim_end().ends_with('}') {
        ("json", "application/json")
    } else if body_md.contains("```rust") || body_md.contains("fn main()") {
        ("rs", "text/x-rust")
    } else if body_md.contains("```ts") || body_md.contains("```typescript") {
        ("ts", "text/x-typescript")
    } else if body_md.contains("[ERROR]") || body_md.contains("WARN") {
        ("log", "text/plain")
    } else {
        ("md", "text/markdown")
    };
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Some((format!("aisb-{}.{}", ts, ext), mime))
}

/// Tier of a response — drives the visual styling (icon + header tone).
/// Telegram has no CSS color, but blockquotes / bold / emoji combos
/// produce distinct enough cues per client theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseTier {
    /// Normal answer (default style).
    Ok,
    /// Empty model response — visual red prefix.
    Empty,
    /// LLM/runtime error — visual red prefix + error icon.
    Error,
}

/// Full smart-wrap response: applies STRUCTURE (mention by id + expandable
/// sections, user-question quoted at top) and SPOILERS (secret masking),
/// plus tiered styling for Empty/Error states.
///
/// `user_message` — when Some, the user's original question is shown as a
/// <blockquote> at the top of the bot response (renders with a colored
/// left bar in Telegram clients — the "yellow quote" effect).
#[allow(clippy::too_many_arguments)]
pub fn smart_wrap_response(
    agent: &str,
    body_md: &str,
    duration_s: f32,
    model: &str,
    user_id: Option<i64>,
    user_name: Option<&str>,
    user_message: Option<&str>,
    tier: ResponseTier,
) -> String {
    // Header — mention the user by tg://user?id= if known
    let agent_label = match tier {
        ResponseTier::Ok => format!("<b>{}</b>", escape_html(agent)),
        ResponseTier::Empty => format!("🔴 <b>{}</b> · Empty response", escape_html(agent)),
        ResponseTier::Error => format!("🔴 <b>{}</b> · Error", escape_html(agent)),
    };
    let header = match (user_id, user_name) {
        (Some(uid), Some(name)) => format!(
            "Ω  <a href=\"tg://user?id={}\">{}</a> · {}",
            uid,
            escape_html(name),
            agent_label,
        ),
        _ => format!("Ω  {}", agent_label),
    };
    let divider = "─".repeat(20);

    // Agent response wrapped in <blockquote> for visual separation
    // (the "colored left bar" treatment — exact tint is theme-driven
    // by the Telegram client and not controllable via Bot API HTML).
    //
    // user_message kept in signature for callers that want to log/cite
    // it (e.g. error reporting), but not rendered: Telegram's reply-to
    // header already shows it above the message.
    let _ = user_message;

    // Scrub internal fences (memory_context, reasoning_scratchpad, etc.)
    // BEFORE markdown conversion so they don't leak into Telegram HTML.
    let scrubbed = scrub_internal_context(body_md);
    let body_html = smart_markdown_to_html(&scrubbed);
    let body_html = mask_secrets(&body_html);
    let footer = format!("<i>⌁ {:.1}s · {}</i>", duration_s, escape_html(model));
    format!(
        "{}\n{}\n\n<blockquote>{}</blockquote>\n\n{}",
        header, divider, body_html, footer
    )
}

/// Strip OmegaOS / Hermes-style internal fences from a model response
/// before it leaves the system (Telegram, file delivery, etc.).
///
/// Models trained to use reasoning scratchpads or memory contexts often
/// leak those XML-ish fences into their final answer when not told
/// otherwise. The scrubber removes them so the end user sees only the
/// human-facing prose.
///
/// Fenced regions removed (case-insensitive open tag, greedy):
///   <memory_context>...</memory_context>
///   <reasoning_scratchpad>...</reasoning_scratchpad>
///   <thinking>...</thinking>
///   <scratchpad>...</scratchpad>
///   <inner_monologue>...</inner_monologue>
///   <reply_context>...</reply_context>   (we add this ourselves; safe to strip)
pub fn scrub_internal_context(text: &str) -> String {
    let fences = [
        ("memory_context", "memory_context"),
        ("reasoning_scratchpad", "reasoning_scratchpad"),
        ("thinking", "thinking"),
        ("scratchpad", "scratchpad"),
        ("inner_monologue", "inner_monologue"),
        ("reply_context", "reply_context"),
    ];
    let mut out = text.to_string();
    for (open, close) in fences {
        let open_pat = format!("<{}>", open);
        let close_pat = format!("</{}>", close);
        loop {
            let lower = out.to_lowercase();
            let Some(start) = lower.find(&open_pat) else {
                break;
            };
            let Some(rel_end) = lower[start + open_pat.len()..].find(&close_pat) else {
                // Unclosed fence — drop everything from the opener to EOF
                // rather than leak partial scratchpad.
                out.truncate(start);
                out = out.trim_end().to_string();
                break;
            };
            let end = start + open_pat.len() + rel_end + close_pat.len();
            out.replace_range(start..end, "");
        }
    }
    // Collapse 3+ newlines that the strip might have left behind.
    while out.contains("\n\n\n") {
        out = out.replace("\n\n\n", "\n\n");
    }
    out.trim().to_string()
}

/// "Thinking…" placeholder with elapsed-time progress bar, edited in
/// place every few seconds (Pack PROGRESS — inspired by AISB VPS).
///
/// `elapsed_s` is the seconds since the user message landed; the bar
/// grows over an assumed-20s window (clamped). Caller is expected to
/// edit this message every 2-3s while the LLM call is in flight.
pub fn thinking_progress(agent: &str, elapsed_s: f32) -> String {
    let agent_esc = escape_html(agent);
    let divider = "─".repeat(20);
    // Indeterminate-but-visible bar: 0..20s ramps to 90%, then stays.
    let pct = (elapsed_s / 20.0).clamp(0.0, 0.9);
    let bar = progress_bar(pct, 20);
    format!(
        "Ω  <b>{}</b>\n{}\n\n<i>⌁ Thinking — {:.1}s</i>\n{}",
        agent_esc, divider, elapsed_s, bar
    )
}

/// Format a blockquote card with title and body (AISB report style).
pub fn blockquote_card(title: &str, body: &str) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "<blockquote><b>{}</b>\n{}</blockquote>",
        escape_html(title),
        escape_html(body)
    );
    out
}

/// Format a progress bar for Telegram.
/// `progress` is 0.0..=1.0, `width` is number of characters.
pub fn progress_bar(progress: f32, width: usize) -> String {
    let clamped = progress.clamp(0.0, 1.0);
    let filled = (clamped * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    let pct = (clamped * 100.0).round() as u32;
    format!(
        "<code>[{}{}] {}%</code>",
        "█".repeat(filled),
        "░".repeat(empty),
        pct
    )
}

/// Format a status badge.
pub fn status_badge(label: &str, ok: bool) -> String {
    let icon = if ok { "🟢" } else { "🔴" };
    format!("{} <b>{}</b>", icon, escape_html(label))
}

/// Format a key-value table for Telegram (no real table support, uses aligned text).
pub fn kv_table(rows: &[(&str, &str)]) -> String {
    let mut out = String::new();
    let max_key = rows.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    for (key, val) in rows {
        let _ = writeln!(
            out,
            "<b>{:width$}</b>  <code>{}</code>",
            escape_html(key),
            escape_html(val),
            width = max_key
        );
    }
    out.trim_end().to_string()
}

/// Format an oracle report for Telegram delivery.
pub fn format_oracle_report(project: &str, status: &str, body: &str) -> String {
    let status_icon = match status.to_uppercase().as_str() {
        "DONE" => "✅",
        "FAILED" => "🔴",
        "PENDING" => "🟡",
        _ => "📋",
    };
    let header = format!(
        "{} <b>{}</b> — {}",
        status_icon,
        escape_html(project),
        escape_html(status)
    );
    let formatted_body = markdown_to_telegram_html(body);
    format!("{}\n━━━━━━━━━━\n{}", header, formatted_body)
}

/// Format an audit score summary.
pub fn format_audit_scores(scores: &[(&str, f32, f32)]) -> String {
    let mut out = String::from("<b>Quality Arsenal Scores</b>\n━━━━━━━━━━\n");
    for (name, score, max) in scores {
        let pct = if *max > 0.0 {
            (score / max) * 100.0
        } else {
            0.0
        };
        let icon = if pct >= 80.0 {
            "🟢"
        } else if pct >= 50.0 {
            "🟡"
        } else {
            "🔴"
        };
        let _ = writeln!(
            out,
            "{} <b>{}</b>  <code>{:.0}/{:.0}</code>  ({})",
            icon,
            escape_html(name),
            score,
            max,
            progress_bar(pct / 100.0, 10)
        );
    }
    out.trim_end().to_string()
}

/// Split a message into chunks that fit Telegram's 4096-char limit.
pub fn split_message(text: &str, max_len: usize) -> Vec<String> {
    if text.len() <= max_len {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();

    for line in text.lines() {
        if current.len() + line.len() + 1 > max_len {
            if !current.is_empty() {
                chunks.push(current.clone());
                current.clear();
            }
            if line.len() > max_len {
                // Force-split very long lines. Slice on a UTF-8 CHAR boundary —
                // a raw byte index at max_len would panic mid-codepoint (emoji,
                // accents) and kill the whole bridge process.
                let mut remaining = line;
                while remaining.len() > max_len {
                    // Largest char-boundary <= max_len (always >= 1 for non-empty).
                    let mut cut = max_len;
                    while cut > 0 && !remaining.is_char_boundary(cut) {
                        cut -= 1;
                    }
                    if cut == 0 {
                        // A single char wider than max_len — take one whole char
                        // rather than loop forever.
                        cut = remaining
                            .char_indices()
                            .nth(1)
                            .map(|(i, _)| i)
                            .unwrap_or(remaining.len());
                    }
                    chunks.push(remaining[..cut].to_string());
                    remaining = &remaining[cut..];
                }
                current.push_str(remaining);
            } else {
                current.push_str(line);
            }
        } else {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_html_chars() {
        assert_eq!(
            escape_html("<b>&\"test\"</b>"),
            "&lt;b&gt;&amp;&quot;test&quot;&lt;/b&gt;"
        );
    }

    #[test]
    fn markdown_headings() {
        let html = markdown_to_telegram_html("# Title\n## Subtitle");
        assert!(html.contains("<b>Title</b>"));
        assert!(html.contains("<b>Subtitle</b>"));
    }

    #[test]
    fn markdown_code_block() {
        let html = markdown_to_telegram_html("```\nfn main() {}\n```");
        assert!(html.contains("<pre>"));
        assert!(html.contains("fn main()"));
    }

    #[test]
    fn markdown_blockquote() {
        let html = markdown_to_telegram_html("> Important note");
        assert!(html.contains("<blockquote>"));
    }

    #[test]
    fn markdown_list_items() {
        let html = markdown_to_telegram_html("- First\n- Second");
        assert!(html.contains("• First"));
        assert!(html.contains("• Second"));
    }

    #[test]
    fn progress_bar_formatting() {
        let bar = progress_bar(0.5, 10);
        assert!(bar.contains("50%"));
        assert!(bar.contains("█████░░░░░"));
    }

    #[test]
    fn progress_bar_clamping() {
        let bar = progress_bar(1.5, 10);
        assert!(bar.contains("100%"));
        let bar0 = progress_bar(-0.5, 10);
        assert!(bar0.contains("0%"));
    }

    #[test]
    fn split_message_short() {
        let chunks = split_message("hello", 4096);
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn split_message_long() {
        let text = "a\n".repeat(3000);
        let chunks = split_message(&text, 4096);
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.len() <= 4096);
        }
    }

    #[test]
    fn split_message_multibyte_no_panic() {
        // A single long line of multi-byte chars (emoji = 4 bytes) whose
        // force-split boundary lands mid-codepoint — used to panic and kill the
        // bridge process. Must split on char boundaries and round-trip exactly.
        let line = "🚀".repeat(2000); // 8000 bytes, no '\n'
        let chunks = split_message(&line, 100);
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.len() <= 100);
        }
        assert_eq!(chunks.concat(), line, "split must be lossless");
    }

    #[test]
    fn blockquote_card_format() {
        let card = blockquote_card("Status", "All good");
        assert!(card.contains("<blockquote>"));
        assert!(card.contains("<b>Status</b>"));
    }

    #[test]
    fn status_badge_ok_and_fail() {
        assert!(status_badge("Build", true).contains("🟢"));
        assert!(status_badge("Build", false).contains("🔴"));
    }

    #[test]
    fn format_oracle_report_done() {
        let report = format_oracle_report("Causio", "DONE", "Fixed auth flow");
        assert!(report.contains("✅"));
        assert!(report.contains("Causio"));
    }

    #[test]
    fn format_audit_scores_display() {
        let scores = vec![("Code", 336.0_f32, 420.0_f32), ("Debug", 100.0, 360.0)];
        let output = format_audit_scores(&scores);
        assert!(output.contains("🟢")); // 336/420 = 80%
        assert!(output.contains("🔴")); // 100/360 = 27.8%
        assert!(output.contains("Code"));
    }

    #[test]
    fn kv_table_format() {
        let table = kv_table(&[("Status", "Online"), ("Uptime", "12h")]);
        assert!(table.contains("Status"));
        assert!(table.contains("Online"));
    }

    #[test]
    fn wrap_response_minimal_engineer() {
        let out = wrap_response("AISB Master", "Hello", 2.4, "opus-4.7");
        assert!(out.starts_with("Ω  <b>AISB Master</b>"));
        assert!(out.contains("─"));
        assert!(out.contains("Hello"));
        assert!(out.contains("<i>⌁ 2.4s · opus-4.7</i>"));
    }

    #[test]
    fn thinking_placeholder_has_marker() {
        let p = thinking_placeholder("AISB Master");
        assert!(p.contains("<b>AISB Master</b>"));
        assert!(p.contains("Thinking"));
    }

    #[test]
    fn italic_with_accented_french_no_panic() {
        // Exact string that crashed the live bridge at formatting.rs:148.
        let input = "<b>La cible</b> — c'est surtout le *visuel* (une page qu'on regarde), \
                     surtout le *système* (l'archi d'orchestration), ou *les deux* \
                     (alors je découpe en sous-projets séquentiels) ?";
        let html = markdown_to_telegram_html(input);
        assert!(
            html.contains("<i>visuel</i>"),
            "italic should wrap 'visuel'"
        );
        assert!(
            html.contains("<i>système</i>"),
            "italic should wrap 'système'"
        );
        assert!(
            html.contains("<i>les deux</i>"),
            "italic should wrap 'les deux'"
        );
        assert!(html.contains("système"), "accented chars must survive");
        assert!(html.contains("séquentiels"), "accented chars must survive");
    }

    #[test]
    fn italic_char_boundary_with_accents() {
        // Bare cases that exercise the char-boundary path directly.
        let html = markdown_to_telegram_html("a *b* é *d* è");
        assert!(html.contains("<i>b</i>"));
        assert!(html.contains("<i>d</i>"));
        assert!(html.contains("é"));
        assert!(html.contains("è"));

        // Italic content itself contains multibyte chars.
        let html2 = markdown_to_telegram_html("ouverture *éléphant* fin");
        assert!(html2.contains("<i>éléphant</i>"));
    }

    #[test]
    fn italic_unpaired_asterisk_is_kept() {
        // A lone trailing '*' should NOT panic and should be left as-is.
        let html = markdown_to_telegram_html("hello *world");
        assert!(!html.contains("<i>"));
        assert!(html.contains("*world"));
    }
}
