//! `omega monitor <session>`: watch a live rmux session, classify what it is
//! doing, and answer each kind of stop differently.
//!
//! NOT to be confused with [`crate::monitor`], which is the billing / accounts /
//! bot view behind the bare `omega monitor`. This module is the SESSION monitor:
//! `omega monitor <session> | <host>:<session> | list`. They share a command
//! name and nothing else, which is why the module is `session_monitor` and the
//! CLI routes on whether a target was given.
//!
//! # Where the logic lives
//!
//! [`classify`] is a PURE function over a captured pane. Pure means unit
//! testable with zero ssh and zero rmux, which is the only way all four states
//! ever get proven. The shell loop under `~/.omega/bin/omega-monitor.sh` does
//! the poll, the ssh, the sleep and the send, and delegates every JUDGEMENT
//! back here through `omega monitor classify`. The reference watcher this
//! module replaces put all of it in bash greps that could not be tested, and
//! that is the one thing we deliberately improve on.
//!
//! # The substrate
//!
//! The whole coordinate layer is [`crate::stream`] and is reused, never
//! reimplemented: [`crate::stream::StreamTarget`], `parse_target`,
//! `is_safe_coordinate`, `read_ssh_config`, `probe_target`, `ProbeOutcome`,
//! `rmux_bin`, `session_exists`. Its five constraints bind here too: rmux is
//! not tmux and is not on the non-interactive PATH; rmux exports `RMUX`, never
//! `$TMUX`; a `$VAR` inside a double-quoted remote ssh command expands LOCALLY,
//! so the remote rmux path must reach the remote shell unexpanded; a `#S`
//! format stays quoted or the remote shell reads `#` as a comment; and
//! send-keys needs its Enter as a SEPARATE call.
//!
//! # Four states, not two
//!
//! A stop has more than one shape and each shape needs a different answer.
//!
//! * [`MonitorState::Question`]: the agent is asking and will not move. Needs
//!   JUDGEMENT, so it goes to a human. NEVER auto-answered.
//! * [`MonitorState::Stalled`]: the turn finished with work still available.
//!   Mechanical, so it gets a mechanical nudge.
//! * [`MonitorState::Blocked`]: nothing is runnable. NEVER nudged, because a
//!   nudge here is not persistence, it is manufactured thrash.
//! * [`MonitorState::Working`]: busy. Say nothing. Silence is the correct
//!   output most of the time.
//!
//! # The seven defects
//!
//! Every one of them was a false positive on a live build, and a false positive
//! trains the operator to ignore alerts, which is the only thing an alerting
//! system has. Each fix is built in here rather than left to the caller.
//!
//! 1. THE WATCHER MATCHED ITS OWN MESSAGES. Text sent into a session echoes in
//!    the pane. [`sent_slices`] records what was sent as SHORT slices (the pane
//!    hard-wraps, so a whole-line match never fires) and [`failure_signals`]
//!    excludes any line carrying one. See also [`question_ui_visible`], where
//!    the same echo arrived as a shell command and satisfied the question test.
//! 2. TWO SIGNALS SHARED ONE STATE VARIABLE and ping-ponged forever.
//!    [`SignalLatches`] has ONE FIELD PER SIGNAL.
//! 3. CLEARING THE FAILURE LATCH ON RESUME made frozen scrollback re-fire.
//!    [`SignalLatches::note_working`] deliberately does not clear it.
//! 4. WORDS WERE MATCHED WHERE STRUCTURES WERE MEANT. [`FAILURE_PATTERNS`]
//!    carries the punctuation.
//! 5. DROPPING A FIXED TAIL DID NOT CUT THE INPUT BOX, because a long message
//!    is longer than the drop. [`strip_input_box`] cuts to the prompt marker.
//! 6. COMPLETED TASK LINES DROWNED THE IN-PROGRESS TRANSITIONS.
//!    [`task_transitions`] drops them before diffing.
//! 7. ONLY RUNNABLE STEPS WERE COUNTED, so a build reported BLOCKED while steps
//!    awaited a sign-off. Awaiting judgement IS work: a work probe counts work
//!    in any form, and [`WorkSignal::from_probe_output`] treats anything it
//!    cannot read as work rather than stalling silently.

use std::path::PathBuf;

use crate::session::MAX_SESSION_NAME_LEN;
use crate::stream::{self, StreamTarget};

/// Seconds between watcher polls. The watcher is the CHEAP layer: one capture,
/// one classification, usually one line of output or none at all.
pub const DEFAULT_INTERVAL_SECS: u32 = 60;

/// Scrollback lines captured per poll.
pub const DEFAULT_LINES: u32 = 120;

/// Watcher polls between two runs of the deep audit team. The audit is the
/// EXPENSIVE layer (parallel read-only sub-agents), so it runs on a much slower
/// cadence than the state classification.
pub const DEFAULT_AUDIT_EVERY: u32 = 30;

/// Nudges that may be spent WITHOUT the progress metric advancing before the
/// budget is exhausted and a human is asked for. See [`NudgeBudget`].
pub const DEFAULT_NUDGE_BUDGET: u32 = 5;

/// Hard wall clock on one probe run. A probe that reaches a box which answers
/// TCP and then never replies would otherwise park the poll forever, and a
/// frozen monitor is indistinguishable from a quiet one.
pub const PROBE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);

// ── The four states ─────────────────────────────────────────────────────────

/// What a captured pane says the session is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorState {
    /// The agent is asking and will not move without an answer.
    Question,
    /// The turn finished and there is still work to do.
    Stalled,
    /// The turn finished and nothing is runnable.
    Blocked,
    /// The activity indicator is on screen.
    Working,
}

impl MonitorState {
    /// The stable wire word, which is what the shell loop reads back.
    pub fn as_str(&self) -> &'static str {
        match self {
            MonitorState::Question => "QUESTION",
            MonitorState::Stalled => "STALLED",
            MonitorState::Blocked => "BLOCKED",
            MonitorState::Working => "WORKING",
        }
    }

    /// Parse the wire word. Case-insensitive, because the shell half is written
    /// by humans who will type it in either case.
    pub fn parse(s: &str) -> Option<MonitorState> {
        match s.trim().to_ascii_uppercase().as_str() {
            "QUESTION" => Some(MonitorState::Question),
            "STALLED" => Some(MonitorState::Stalled),
            "BLOCKED" => Some(MonitorState::Blocked),
            "WORKING" => Some(MonitorState::Working),
            _ => None,
        }
    }

    /// May the watcher answer this state mechanically?
    ///
    /// Only [`MonitorState::Stalled`] may. A question needs judgement and a
    /// blocked session has nothing to run, so nudging either one manufactures
    /// thrash instead of progress.
    pub fn should_nudge(&self) -> bool {
        matches!(self, MonitorState::Stalled)
    }

    /// Does this state have to reach a person?
    pub fn needs_human(&self) -> bool {
        matches!(self, MonitorState::Question | MonitorState::Blocked)
    }

    /// One line an operator can act on.
    pub fn describe(&self) -> &'static str {
        match self {
            MonitorState::Question => {
                "asking a question and waiting for an answer (judgement, never auto-answered)"
            }
            MonitorState::Stalled => "the turn finished with work still available (nudge it)",
            MonitorState::Blocked => {
                "the turn finished and nothing is runnable (never nudge, this needs a human)"
            }
            MonitorState::Working => "working (say nothing)",
        }
    }
}

/// What the per-target work probe reported.
///
/// DEFECT 7 lives in the meaning of these variants: a work probe counts work in
/// ANY form, including steps awaiting a sign-off. A probe that counted only
/// steps marked runnable reported BLOCKED on a build whose remaining steps were
/// all waiting for a human, and awaiting judgement is work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkSignal {
    /// The probe printed a positive integer.
    Available,
    /// The probe printed zero.
    None,
    /// The probe could not be read: no probe configured, it failed, it timed
    /// out, or it printed something that is not an integer.
    Unknown,
}

impl WorkSignal {
    /// Read a work probe's stdout.
    ///
    /// Unreadable output is [`WorkSignal::Unknown`], and Unknown is treated as
    /// work by [`classify`]. Never stall silently: a probe we cannot read is
    /// not evidence that a build has nothing left to do.
    pub fn from_probe_output(out: &str) -> WorkSignal {
        let token = out
            .lines()
            .map(str::trim)
            .rfind(|l| !l.is_empty())
            .unwrap_or("");
        match token.parse::<i64>() {
            Ok(n) if n > 0 => WorkSignal::Available,
            Ok(0) => WorkSignal::None,
            // A negative count is not a count. Treat it like any other garbage.
            Ok(_) | Err(_) => WorkSignal::Unknown,
        }
    }

    /// Read a work count the CALLER already measured.
    ///
    /// The watcher loop runs the probe once per poll for its own decisions and
    /// then asks for a classification, so the count arrives as a number rather
    /// than as a command to re-run. Re-running it would cost a second ssh round
    /// trip and, worse, could disagree with the count the caller acted on.
    ///
    /// The mapping is deliberately IDENTICAL to [`WorkSignal::from_probe_output`],
    /// negative included: the two entry points must never disagree about what a
    /// number means, or the same build would classify differently depending on
    /// which half of the feature asked.
    pub fn from_count(n: i64) -> WorkSignal {
        match n {
            n if n > 0 => WorkSignal::Available,
            0 => WorkSignal::None,
            _ => WorkSignal::Unknown,
        }
    }
}

/// Classify a captured pane. PURE: no ssh, no rmux, no clock.
///
/// The order is the contract, not an implementation detail.
///
/// 1. QUESTION FIRST, because a question that falls through to Stalled gets
///    NUDGED, and nudging a question is auto-answering it.
/// 2. WORKING next, because a busy session is the common case and the correct
///    output for it is silence.
/// 3. Otherwise the turn stopped, and only the work probe can say whether that
///    stop is a stall or a block.
pub fn classify(pane: &str, work: WorkSignal) -> MonitorState {
    if question_ui_visible(pane) {
        return MonitorState::Question;
    }
    if working_indicator_visible(pane) {
        return MonitorState::Working;
    }
    match work {
        WorkSignal::None => MonitorState::Blocked,
        // DEFECT 7 again: Unknown means we could not read the probe, not that
        // there is nothing to do. Stalled is the answer that keeps the build
        // moving; Blocked would park it on a probe bug.
        WorkSignal::Available | WorkSignal::Unknown => MonitorState::Stalled,
    }
}

// ── Reading a pane ──────────────────────────────────────────────────────────

/// Both halves of the question UI's hint line. They must appear on the SAME
/// line: either one alone fires on ordinary prose (a code review containing
/// "there is nothing to navigate to while it is open" was read as a pending
/// question).
pub const QUESTION_MARKER_SELECT: &str = "Enter to select";
/// See [`QUESTION_MARKER_SELECT`].
pub const QUESTION_MARKER_NAVIGATE: &str = "to navigate";

/// The activity indicator, spelled two ways by the agent CLI.
pub const WORKING_MARKER_INTERRUPT: &str = "esc to interrupt";
/// The other spelling: a spinner glyph opening a line that reports token counts.
pub const WORKING_MARKER_TOKENS: &str = "tokens";

/// Glyphs the agent CLI cycles through at the head of its activity line.
const ACTIVITY_GLYPHS: &[char] = &['✻', '✽', '✢', '✳', '✶', '∗', '·'];

/// Box-drawing characters that open a horizontal rule.
const RULE_CHARS: &[char] = &['─', '━'];

/// How many rule characters in a row make a line a rule rather than prose.
const MIN_RULE_RUN: usize = 8;

/// Characters the agent CLI uses for its composer prompt. The `❯` is followed
/// by a NO-BREAK SPACE in a real capture, not an ordinary one, so the marker is
/// tested on the trimmed line and never as the literal two characters `"❯ "`.
const PROMPT_MARKERS: &[char] = &['❯', '>'];

/// Non-blank lines at the bottom of a pane that count as LIVE UI when no
/// composer is drawn. A modal (a question, a permission prompt) renders at the
/// very bottom of the screen by definition, so a generous bound costs nothing
/// and keeps a marker-shaped line deep in the scrollback from firing.
pub const LIVE_TAIL_LINES: usize = 12;

/// Lines above the composer's opening rule that still count as live chrome.
/// The activity line renders directly above the input box, so a small window
/// covers it, and keeping the window small is what stops a stale
/// `esc to interrupt` deep in the scrollback from pinning a stopped session to
/// Working forever (defect 1 wearing different clothes).
const ACTIVITY_LOOKBACK: usize = 8;

/// Is this line a horizontal rule (the border of the composer or of a panel)?
fn is_horizontal_rule(line: &str) -> bool {
    let mut run = 0usize;
    for c in line.trim_start().chars() {
        if !RULE_CHARS.contains(&c) {
            return false;
        }
        run += 1;
        if run >= MIN_RULE_RUN {
            return true;
        }
    }
    false
}

/// Is this line the composer's prompt line?
fn is_prompt_marker_line(line: &str) -> bool {
    line.trim()
        .chars()
        .next()
        .is_some_and(|c| PROMPT_MARKERS.contains(&c))
}

/// Index of the line that OPENS the composer box (its top rule), or `None` when
/// no composer is on screen.
///
/// DEFECT 5. The composer is found STRUCTURALLY, by the rule line immediately
/// above the prompt marker, never by dropping a fixed number of tail lines: a
/// long message pasted into the composer is longer than any fixed drop, and the
/// part that survived the drop was then matched as if the agent had said it.
///
/// The scan runs from the bottom because the composer is the LAST such pair on
/// screen, and the immediate adjacency of rule and marker is what keeps a
/// question option list (`❯ 1. Option A`, which follows prose, not a rule) from
/// being mistaken for it.
///
/// NOTE that this locates a rule-then-marker pair ANYWHERE, which is right for
/// cutting the composer out of a stopped pane but is NOT on its own evidence
/// that the operator can type: a question modal draws its own rule and its own
/// selected option, `────` then `❯ 1. Option A`, which is the same shape. See
/// [`question_ui_visible`] for why that distinction is load-bearing and where
/// it is actually decided.
pub fn find_input_box(pane: &str) -> Option<usize> {
    let lines: Vec<&str> = pane.lines().collect();
    for i in (1..lines.len()).rev() {
        if is_prompt_marker_line(lines[i]) && is_horizontal_rule(lines[i - 1]) {
            return Some(i - 1);
        }
    }
    None
}

/// The pane with the composer box, everything the operator typed into it, and
/// the status bar under it removed. What is left is the transcript.
pub fn strip_input_box(pane: &str) -> String {
    let lines: Vec<&str> = pane.lines().collect();
    let cut = find_input_box(pane).unwrap_or(lines.len());
    lines[..cut].join("\n")
}

/// The last `n` NON-BLANK lines, in screen order.
fn tail_lines(pane: &str, n: usize) -> Vec<&str> {
    let mut out: Vec<&str> = pane
        .lines()
        .rev()
        .filter(|l| !l.trim().is_empty())
        .take(n)
        .collect();
    out.reverse();
    out
}

/// The region of the pane that is LIVE CHROME: the composer, the status bar
/// under it, and the handful of lines above it where the activity indicator
/// renders. Falls back to the bottom of the pane when no composer is drawn,
/// which is what a modal UI looks like.
fn live_chrome(pane: &str) -> Vec<&str> {
    let lines: Vec<&str> = pane.lines().collect();
    match find_input_box(pane) {
        Some(rule) => lines[rule.saturating_sub(ACTIVITY_LOOKBACK)..].to_vec(),
        None => tail_lines(pane, LIVE_TAIL_LINES),
    }
}

/// Does one line carry BOTH halves of the question hint?
fn is_question_hint_line(line: &str) -> bool {
    line.contains(QUESTION_MARKER_SELECT) && line.contains(QUESTION_MARKER_NAVIGATE)
}

/// Is the question UI on screen right now?
///
/// DEFECT 1, in the form that is hardest to see. The both-markers-on-one-line
/// test is necessary and NOT sufficient: the watcher's own probe is a shell
/// command containing that very pattern (`grep -cE 'Enter to select.*to
/// navigate'`), it echoes into the pane as a tool call, and it satisfies the
/// structural test perfectly. The capture
/// `tests/fixtures/GOLDEN-self-echo-false-question.txt` is that exact pane.
///
/// So the hint alone never decides it. What does is WHERE the hint sits
/// relative to the place the operator can type.
///
/// The question UI takes over the bottom of the screen: its hint line is the
/// last thing drawn, and there is nothing typeable below it. An echoed command
/// is transcript, so the composer is drawn UNDER it. That single relation
/// separates the two cleanly and, unlike a depth bound, it does not care how
/// tall the modal is or how long a pasted message was.
///
/// A NEARLY-SHIPPED BUG LIVES HERE. This used to bail the moment
/// [`find_input_box`] found any rule-then-marker pair, on the reasoning that
/// the modal REPLACES the composer. But a modal draws `────` then
/// `❯ 1. Option A`, which is that same pair, so the project's own real question
/// capture passed only because one blank line happened to separate them. Move
/// the option line up by that one line and a LIVE QUESTION classified STALLED,
/// which the watcher NUDGES, which ANSWERS the question the operator was meant
/// to answer. Auto-answering a question is the single worst thing this feature
/// could do, so the test is now a relation between positions, never a shape.
pub fn question_ui_visible(pane: &str) -> bool {
    let lines: Vec<&str> = pane.lines().collect();
    let Some(hint) = lines.iter().rposition(|l| is_question_hint_line(l)) else {
        return false;
    };
    // The hint must be LIVE, not scrollback a `clear` has not yet swept away.
    let last_content = match lines.iter().rposition(|l| !l.trim().is_empty()) {
        Some(i) => i,
        None => return false,
    };
    if last_content.saturating_sub(hint) >= LIVE_TAIL_LINES {
        return false;
    }
    // Nothing typeable below the hint. A composer under it means the hint is
    // transcript, which is exactly where our own echoed command lands.
    !lines[hint + 1..].iter().any(|l| is_prompt_marker_line(l))
}

/// Is the activity indicator on screen?
///
/// Two spellings, both anchored in the live chrome. `esc to interrupt` lives on
/// the status bar UNDER the composer, which is why this test runs on the
/// unstripped pane while the question test runs on the stripped one.
///
/// The token spelling additionally requires a spinner glyph at the head of the
/// line. Bare `tokens` appears on the status bar of a session that has stopped
/// (it reports the turn's total), and `✻ Cooked for 3s` is the FINISHED form of
/// the spinner line with no token count at all: neither is activity.
pub fn working_indicator_visible(pane: &str) -> bool {
    live_chrome(pane)
        .into_iter()
        .any(|line| line.contains(WORKING_MARKER_INTERRUPT) || is_activity_line(line))
}

fn is_activity_line(line: &str) -> bool {
    let t = line.trim_start();
    t.chars()
        .next()
        .is_some_and(|c| ACTIVITY_GLYPHS.contains(&c))
        && t.contains(WORKING_MARKER_TOKENS)
}

// ── Is a composer safe to PASTE INTO? ───────────────────────────────────────
//
// [`find_input_box`] answers "where does the composer box start", which is the
// right question when CUTTING it out of a transcript and the wrong one when
// deciding whether a block of text may be TYPED there. This section answers the
// second question, and it is written as a WHITELIST: the pane must positively
// exhibit the agent's own live chrome, rather than merely fail to exhibit one
// known modal.
//
// Every rejection below was reproduced in runtime by a forensic audit of the
// followup delivery path:
//   * a LIVE bash shell. Oracles run as `bash -c '<agent> …; exec bash'`
//     (agents.rs:452), so when the agent dies the session STAYS UP with its
//     last frame — composer included — and a shell prompt under it. Every line
//     of a mission pasted there executes as a command.
//   * a modal whose hint hard-wraps onto two lines (a narrow pane in a split
//     layout), or a numbered permission dialog that draws no hint at all. Enter
//     picks the highlighted option.
//   * a composer holding the operator's own unsubmitted draft: a paste
//     CONCATENATES onto it and submits the lot as one turn.

/// Markers only the agent's own status bar draws, on the line(s) directly under
/// the composer. This is the POSITIVE evidence that the box on screen belongs
/// to a live agent TUI rather than to a shell, a pager, or a modal.
pub const COMPOSER_STATUS_MARKERS: &[&str] = &[
    "⏵⏵",
    "bypass permissions",
    "shift+tab to cycle",
    "accept edits on",
    "plan mode on",
    "for shortcuts",
    WORKING_MARKER_INTERRUPT,
];

/// How far under the prompt marker the status bar may sit. A real capture puts
/// it two lines down (the composer's closing rule, then the bar).
const COMPOSER_STATUS_LOOKAHEAD: usize = 4;

/// Glyphs that OPEN a row of the agent's own footer block — the task panel it
/// draws under the status bar when `ctrl+t to hide tasks` is on.
const COMPOSER_FOOTER_GLYPHS: &[char] = &['◯', '◻', '◼', '○', '●', '✔', '✓'];

/// Does this line belong to the agent's own footer — the block it draws BELOW
/// the composer's closing rule?
///
/// THE PREDICATE THIS REPLACES TRIED TO RECOGNISE A SHELL, AND A RE-AUDIT
/// PROVED IT CANNOT. An oracle runs as `bash -c '<agent> …; exec bash'`
/// (agents.rs:452), so a dead agent leaves its last frame on screen with a live
/// shell prompt under it — and that prompt is drawn from the USER'S `~/.bashrc`,
/// not from anything OmegaOS controls. Six realistic `PS1` forms were replayed
/// into six real rmux sessions and FOUR were accepted (`\u@\h \w ❯`, `\w >`,
/// `➜  omegaos git:(main)`, a bare `omegaos`); the mission body then executed
/// line by line as commands, `$()` live.
///
/// So the proof is inverted. Nothing is asked of the foreign line any more:
/// what follows the composer must POSITIVELY be the agent's own footer, which
/// is a blank line, the status bar itself, a token-count row, or a task-panel
/// row. Every one of the six prompts fails that, and so does a seventh nobody
/// has thought of yet.
///
/// The cost of being wrong runs one way only: an unrecognised footer row costs
/// a spawned oracle (the caller falls back), while an unrecognised shell prompt
/// runs a mission as shell commands. A new footer row in a future agent release
/// therefore switches the feature off rather than pointing it at a shell.
fn is_composer_footer_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return true;
    }
    if COMPOSER_STATUS_MARKERS.iter().any(|m| trimmed.contains(m)) {
        return true;
    }
    if is_token_count_row(trimmed) {
        return true;
    }
    trimmed
        .chars()
        .next()
        .is_some_and(|c| COMPOSER_FOOTER_GLYPHS.contains(&c))
}

/// The right-aligned rows the agent puts under its status bar: `400384 tokens`
/// and `new task? /clear to save 250k tokens`.
///
/// Matched by SHAPE, not by the bare word: a count then the word, so a shell
/// whose working directory merely ends in `tokens` is not read as chrome.
fn is_token_count_row(trimmed: &str) -> bool {
    let Some(head) = trimmed.strip_suffix(WORKING_MARKER_TOKENS) else {
        return false;
    };
    let Some(count) = head.split_whitespace().next_back() else {
        return false;
    };
    head.ends_with(char::is_whitespace)
        && count.starts_with(|c: char| c.is_ascii_digit())
        && count
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == 'k' || c == 'K' || c == 'M')
}

/// Index of the composer's PROMPT MARKER line, bounded to the live tail.
///
/// Same structural pair as [`find_input_box`] (a rule with a marker directly
/// under it) with one added constraint: the pair must be inside the live tail,
/// exactly as [`question_ui_visible`] bounds its hint. The composer is by
/// construction the LAST thing drawn, so a pair sitting `LIVE_TAIL_LINES` or
/// more above the final line of content is scrollback, not a place to type.
pub fn find_live_composer_marker(pane: &str) -> Option<usize> {
    let lines: Vec<&str> = pane.lines().collect();
    let last_content = lines.iter().rposition(|l| !l.trim().is_empty())?;
    for i in (1..lines.len()).rev() {
        if is_prompt_marker_line(lines[i]) && is_horizontal_rule(lines[i - 1]) {
            // Bottom-most pair first: if this one is already scrollback, every
            // pair above it is further away still.
            if last_content.saturating_sub(i) >= LIVE_TAIL_LINES {
                return None;
            }
            return Some(i);
        }
    }
    None
}

/// Index of the composer box's CLOSING rule, scanning DOWN from the marker.
///
/// The composer is a BOX between two rules, not a line: the marker opens it and
/// the text wraps onto the lines under the marker. Everything that reads the
/// composer therefore needs both edges, and a box whose closing rule is not on
/// screen is a box this module cannot bound — the caller treats that as "not
/// empty", which costs a spawned oracle and never a paste into the unknown.
fn composer_closing_rule(lines: &[&str], marker: usize) -> Option<usize> {
    lines
        .iter()
        .enumerate()
        .skip(marker + 1)
        .find(|(_, l)| is_horizontal_rule(l))
        .map(|(i, _)| i)
}

/// Is the composer EMPTY — the whole BOX, marker line to closing rule?
///
/// A followup pastes WITHOUT clearing, because clearing would destroy text the
/// operator typed and has not sent. So a draft is not a target: the caller
/// falls back to spawning. Two live sessions on the audited machine were
/// holding a draft at that moment, and one of them read `❯ ferme la session
/// OmegaOS` — the followup would have submitted "close the session" plus the
/// mission as a single turn.
///
/// TESTING THE MARKER LINE ALONE WAS NOT ENOUGH, and a re-audit reproduced it
/// on a live session: any draft whose FIRST line is blank — a paste that opens
/// with a newline, a `\`+Enter, an Option+Enter typed first — leaves the marker
/// bare and the text intact one line BELOW it. The probe read that pane as an
/// empty composer, and the paste came back as the single turn
/// `fais aussi le refactor avant de fermerMISSION SUIVI: audite le module
/// dispatch`. So the whole box is judged: one non-blank line anywhere between
/// the marker and the closing rule means a draft is being held.
pub fn composer_is_empty(pane: &str, marker: usize) -> bool {
    let lines: Vec<&str> = pane.lines().collect();
    let Some(marker_line) = lines.get(marker) else {
        return false;
    };
    let trimmed = marker_line.trim();
    let mut chars = trimmed.chars();
    match chars.next() {
        // `trim` covers the NO-BREAK SPACE the real capture puts after `❯`.
        Some(c) if PROMPT_MARKERS.contains(&c) => {
            if !chars.as_str().trim().is_empty() {
                return false;
            }
        }
        _ => return false,
    }
    let Some(close) = composer_closing_rule(&lines, marker) else {
        return false;
    };
    lines[marker + 1..close].iter().all(|l| l.trim().is_empty())
}

/// Is a selection list on screen — `❯ 1. Option A`?
///
/// The blacklist half of the modal test, and the one that survives a hint the
/// pane hard-wrapped. A permission dialog draws this shape with a footer that
/// is not the question hint at all, so [`question_ui_visible`] misses it while
/// Enter would still pick the highlighted option.
fn is_numbered_option_line(line: &str) -> bool {
    let trimmed = line.trim();
    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !PROMPT_MARKERS.contains(&first) {
        return false;
    }
    let rest = chars.as_str().trim_start();
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    digits > 0 && rest[digits..].starts_with('.')
}

/// Does a shell prompt appear on this line?
///
/// Deliberately generous. This predicate guards a paste that would otherwise
/// EXECUTE, so a false rejection costs a spawned oracle while a false
/// acceptance runs a mission body as shell commands, `$()` and redirections
/// live.
fn is_shell_prompt_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    if matches!(trimmed.chars().last(), Some('$') | Some('#') | Some('%')) {
        return true;
    }
    // `vibe@station:~/Station/SideBusiness/OmegaOS$ some command`
    trimmed.contains('@') && (trimmed.contains(":~") || trimmed.contains(":/"))
}

/// May a mission body be pasted into this pane?
///
/// The whole predicate, as six independent reasons to say no. The caller adds
/// [`question_ui_visible`] on top; each check is kept separate so a regression
/// names itself in the failing test.
pub fn composer_ready_for_paste(pane: &str) -> bool {
    let lines: Vec<&str> = pane.lines().collect();
    let Some(marker) = find_live_composer_marker(pane) else {
        return false;
    };
    // The whole BOX, not the marker line: a draft whose first line is blank
    // hides under a bare marker.
    if !composer_is_empty(pane, marker) {
        return false;
    }
    let Some(close) = composer_closing_rule(&lines, marker) else {
        return false;
    };
    // POSITIVE evidence of a live agent, within the composer's own chrome.
    let status_end = (marker + 1 + COMPOSER_STATUS_LOOKAHEAD).min(lines.len());
    let has_status_bar = lines[(marker + 1).min(lines.len())..status_end]
        .iter()
        .any(|l| COMPOSER_STATUS_MARKERS.iter().any(|m| l.contains(m)));
    if !has_status_bar {
        return false;
    }
    // A selection list anywhere in the live tail: Enter would answer it.
    if tail_lines(pane, LIVE_TAIL_LINES)
        .into_iter()
        .any(is_numbered_option_line)
    {
        return false;
    }
    // NOTHING typeable below the composer. A shell prompt there means the frame
    // above it is a corpse the agent left on screen when it exited; a second
    // prompt marker means the pair we matched was not the bottom-most one.
    // Kept as the blacklist half, under the whitelist that now decides: it
    // names the two shapes that were actually reproduced in runtime.
    if lines[(marker + 1).min(lines.len())..]
        .iter()
        .any(|l| is_shell_prompt_line(l) || is_prompt_marker_line(l))
    {
        return false;
    }
    // ONLY the agent's own footer below the box. This is the check that closes
    // the dead-agent shell for every `PS1`, including the four realistic ones
    // the blacklist above accepted. See [`is_composer_footer_line`].
    lines[(close + 1).min(lines.len())..]
        .iter()
        .all(|l| is_composer_footer_line(l))
}

// ── Was a paste ACCEPTED? ───────────────────────────────────────────────────
//
// The section above answers "may text be typed here". This one answers the
// question that comes one second later, and it is a DIFFERENT question with a
// different failure mode: a followup that was accepted but not RECOGNISED as
// accepted used to be re-delivered as a second oracle.

/// What the agent draws on the composer's marker line while it is holding
/// messages it has not consumed yet.
///
/// THE NOMINAL CASE, NOT AN ANOMALY, and the exact string was taken from a
/// capture of a live agent rather than guessed. A followup goes to a BUSY
/// oracle by definition — that is the whole point of the feature — and a busy
/// agent QUEUES the paste instead of consuming it:
///
/// ```text
///   ❯ SENTINEL-QUEUED-FOLLOWUP alpha
///     bravo charlie
/// ────────────────────────────────────────────────────────
/// ❯ Press up to edit queued messages
/// ────────────────────────────────────────────────────────
///   ⏵⏵ bypass permissions on (shift+tab to cycle) · esc to interrupt
/// ```
///
/// The queued text is echoed ABOVE the box as a turn, and the box itself holds
/// this placeholder. Read as "the composer is not empty", that pane says the
/// paste was buffered and never submitted — which is exactly backwards, and is
/// what made the acceptance probe fail on its main use case.
const COMPOSER_QUEUED_PLACEHOLDER: &str = "Press up to edit queued messages";

/// Is the composer showing the agent's own QUEUED-MESSAGES placeholder rather
/// than text somebody typed?
///
/// Matched as the WHOLE content of the marker line, not as a substring of the
/// pane: a placeholder is chrome the agent draws by itself, and requiring it to
/// be the entire line keeps an operator's message that happens to quote it from
/// reading as chrome (DEFECT 4 — match the structure, not the words).
pub fn composer_shows_queued_messages(pane: &str) -> bool {
    let Some(marker) = find_live_composer_marker(pane) else {
        return false;
    };
    let lines: Vec<&str> = pane.lines().collect();
    let Some(line) = lines.get(marker) else {
        return false;
    };
    let mut chars = line.trim().chars();
    match chars.next() {
        Some(c) if PROMPT_MARKERS.contains(&c) => {
            chars.as_str().trim() == COMPOSER_QUEUED_PLACEHOLDER
        }
        _ => false,
    }
}

/// Did the text we sent reach the TRANSCRIPT — the part of the pane that is not
/// the composer?
///
/// POSITIVE evidence of acceptance, and the only kind that covers both shapes a
/// live agent produces: a consumed turn and a queued one both echo the message
/// above the box, and neither depends on the composer being empty afterwards.
/// It has to be, because the composer is NOT reliably empty after a successful
/// followup — a capture taken one second after an accepted paste showed the
/// agent's own suggested next prompt sitting in it (`❯ did it finish?`), which
/// is indistinguishable from a draft in captured text.
///
/// `sent` is what [`sent_slices`] recorded AT SEND TIME, for the same reason
/// the watcher records it: the pane hard-wraps, so only short overlapping
/// windows survive (DEFECT 1). The composer box is stripped BEFORE the test and
/// never after, and a pane whose box cannot be located answers `false` rather
/// than matching text that may still be sitting unsent in it.
pub fn sent_text_reached_the_transcript(pane: &str, sent: &[String]) -> bool {
    if sent.is_empty() || find_input_box(pane).is_none() {
        return false;
    }
    let transcript = strip_input_box(pane);
    sent.iter().any(|slice| transcript.contains(slice.as_str()))
}

// ── Failure signals ─────────────────────────────────────────────────────────

/// EXACT failure strings, WITH their punctuation.
///
/// DEFECT 4: the first watcher matched WORDS where STRUCTURES were meant. A
/// bare `MISSING` fired on the prose "missing redirect" describing an
/// already-fixed defect, and `is RED` fired on "is REDundant". A runner emits
/// its verdicts in a fixed shape, so match that shape.
pub const FAILURE_PATTERNS: &[&str] = &[
    "MISSING ROW:",
    "is RED.",
    "Traceback (most recent call last):",
    "panicked at",
    "error: could not compile",
    "npm ERR!",
    "FAILED:",
    "Segmentation fault",
    "command not found",
];

/// Prefixes whose next character decides the verdict: `exit code 0` is a pass
/// and `exit code 1` is a failure, so the digit is part of the pattern.
const EXIT_CODE_PREFIXES: &[&str] = &["exit code ", "exit status "];

/// Most failure lines reported from one poll. A runner that fails everything
/// would otherwise turn one alert into a wall of text nobody reads.
pub const MAX_FAILURE_SIGNALS: usize = 5;

/// Characters kept from a matched line. Enough to identify it, short enough for
/// a Telegram line.
pub const MAX_SIGNAL_CHARS: usize = 200;

/// Words per recorded slice of an outgoing message.
const SLICE_WORDS: usize = 3;

/// Characters per recorded slice. A slice has to be SHORT: the pane hard-wraps
/// an outgoing message, so a whole-line match never fires.
const MAX_SLICE_CHARS: usize = 24;

/// A slice this short is not distinctive enough to exclude a line by; it would
/// blank real signal instead of the echo.
const MIN_SLICE_CHARS: usize = 12;

/// Most slices kept per message.
const MAX_SENT_SLICES: usize = 32;

/// Split an outgoing message into SHORT, OVERLAPPING slices to exclude its echo
/// by CONTENT.
///
/// DEFECT 1. Text sent into a session echoes in the pane, and the first watcher
/// matched its own nudges as if the agent had said them. Excluding the whole
/// message never worked, because the pane hard-wraps it and no single captured
/// line ever contains it. Short overlapping windows survive the wrap: whatever
/// column the wrap lands on, some window is still intact on some line.
///
/// An empty message yields no slices, and no slices means NOTHING is excluded.
/// That is the Rust half of the sentinel lesson: the shell watcher passed its
/// exclusion list to `grep -f`, and `grep -f` against an EMPTY file matches
/// nothing, silently blanking the entire stream, so the shell half must pass a
/// sentinel that matches nothing rather than an empty list. Here the empty case
/// is expressed directly and cannot invert.
pub fn sent_slices(msg: &str) -> Vec<String> {
    let normalized = msg.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return Vec::new();
    }
    let words: Vec<&str> = normalized.split(' ').collect();

    let mut out: Vec<String> = Vec::new();
    if words.len() > SLICE_WORDS {
        for window in words.windows(SLICE_WORDS) {
            let slice = truncate_chars(&window.join(" "), MAX_SLICE_CHARS);
            if slice.chars().count() < MIN_SLICE_CHARS {
                continue;
            }
            if !out.contains(&slice) {
                out.push(slice);
            }
            if out.len() >= MAX_SENT_SLICES {
                break;
            }
        }
    }
    if out.is_empty() {
        // A short message, or one made entirely of tiny words. Record it whole
        // rather than nothing: an empty exclusion list is what let the watcher
        // match its own text in the first place.
        out.push(truncate_chars(&normalized, MAX_SLICE_CHARS));
    }
    out
}

/// The failure lines visible in a pane, with the watcher's own echoed text
/// excluded.
///
/// `sent` is what [`sent_slices`] recorded AT SEND TIME. Any line carrying one
/// of those slices is our own message coming back, never the agent's verdict.
/// Results are deduped (the same line re-renders on every frame) and capped.
pub fn failure_signals(pane: &str, sent: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in pane.lines() {
        let line = raw.trim();
        if line.is_empty() || !matches_failure(line) {
            continue;
        }
        // DEFECT 1: our own message, echoed back.
        if sent
            .iter()
            .any(|slice| !slice.is_empty() && line.contains(slice.as_str()))
        {
            continue;
        }
        let evidence = truncate_chars(line, MAX_SIGNAL_CHARS);
        if !out.contains(&evidence) {
            out.push(evidence);
        }
        if out.len() >= MAX_FAILURE_SIGNALS {
            break;
        }
    }
    out
}

/// Does this line carry one of the exact failure structures?
fn matches_failure(line: &str) -> bool {
    if FAILURE_PATTERNS.iter().any(|p| line.contains(p)) {
        return true;
    }
    // `exit code [1-9]`: the digit is the verdict, so `exit code 0` is a pass.
    for prefix in EXIT_CODE_PREFIXES {
        let mut rest = line;
        while let Some(at) = rest.find(prefix) {
            let after = &rest[at + prefix.len()..];
            if after.starts_with(['1', '2', '3', '4', '5', '6', '7', '8', '9']) {
                return true;
            }
            rest = &rest[at + prefix.len()..];
        }
    }
    false
}

/// Cut to `max` CHARACTERS, never bytes: a pane is full of box drawing and
/// arrows, and a byte cut through one of them panics.
fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    s.chars().take(max).collect()
}

// ── Task transitions ────────────────────────────────────────────────────────

/// Glyphs the agent CLI puts in front of a COMPLETED task item.
const COMPLETED_TASK_GLYPHS: &[char] = &['✔', '✓', '☑', '☒', '✅'];

/// Glyphs in front of a task item that is in progress or still pending.
const ACTIVE_TASK_GLYPHS: &[char] = &['◼', '◻', '■', '□', '☐', '▪', '▫'];

/// Gutter characters that precede a task list inside a tool result block.
const TASK_GUTTER: &[char] = &['⎿', ' ', '\u{a0}', '│'];

/// Most transitions reported from one poll.
pub const MAX_TASK_TRANSITIONS: usize = 8;

/// Task lines that are NEW in `now` compared with `prev`.
///
/// DEFECT 6: completed task lines re-render constantly and drowned the
/// in-progress transitions, which are the only ones that say what is happening
/// NOW. They are dropped from BOTH sides before the diff, so a task that
/// completes between two polls does not register as a transition either, and
/// the summary lines (`… +10 completed`) go with them.
pub fn task_transitions(prev: &str, now: &str) -> Vec<String> {
    let before = active_task_lines(prev);
    let mut out: Vec<String> = Vec::new();
    for line in active_task_lines(now) {
        if !before.contains(&line) && !out.contains(&line) {
            out.push(line);
        }
        if out.len() >= MAX_TASK_TRANSITIONS {
            break;
        }
    }
    out
}

/// The in-progress and pending task lines of a pane, normalized so the same
/// item captured twice compares equal.
pub fn active_task_lines(pane: &str) -> Vec<String> {
    pane.lines().filter_map(normalize_task_line).collect()
}

fn normalize_task_line(raw: &str) -> Option<String> {
    let line = raw.trim_start_matches(TASK_GUTTER).trim();
    let first = line.chars().next()?;
    if COMPLETED_TASK_GLYPHS.contains(&first) {
        return None; // DEFECT 6
    }
    if !ACTIVE_TASK_GLYPHS.contains(&first) {
        return None;
    }
    Some(line.to_string())
}

// ── The nudge bound ─────────────────────────────────────────────────────────

/// What [`NudgeBudget::try_nudge`] decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NudgeDecision {
    /// Spend a nudge. `attempt` counts from 1 within the current run of
    /// no-progress nudges, not over the lifetime of the monitor.
    Nudge { attempt: u32, of: u32 },
    /// The budget is spent and the build has produced nothing across all of it.
    /// Its own outcome, deliberately: this is where the watcher stops acting
    /// and a person is asked for.
    Exhausted { without_progress: u32 },
}

impl NudgeDecision {
    /// One line an operator can act on.
    pub fn describe(&self) -> String {
        match self {
            NudgeDecision::Nudge { attempt, of } => {
                format!("nudging ({attempt}/{of} since the last advance)")
            }
            NudgeDecision::Exhausted { without_progress } => format!(
                "{without_progress} nudges produced no progress at all. \
                 This needs a human, not another nudge"
            ),
        }
    }
}

/// Bound the ABSENCE OF PROGRESS, never the work.
///
/// A flat cap of N restarts stops a healthy long run for no reason: a cap of 25
/// ran out around step 40 of 157 on a build that was moving the whole time. So
/// the counter RESETS every time the build's own progress metric advances, and
/// only exhausts after N nudges that produced NOTHING. That is what R-LOOP
/// actually asks for, bounded retries on the SAME failure, rather than a
/// ceiling on success.
///
/// The metric comes from a `--progress-probe` command that prints a monotonic
/// integer (steps done, tests passing, rows signed). Only an ADVANCE resets:
/// a metric that goes backwards, which happens when a probe is edited or a
/// build is rewound, must not hand out a fresh budget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NudgeBudget {
    limit: u32,
    spent: u32,
    best: Option<i64>,
}

impl NudgeBudget {
    /// A budget of `limit` nudges per run of no progress. A limit of 0 means
    /// never nudge, which is a legitimate read-only monitor.
    pub fn new(limit: u32) -> Self {
        NudgeBudget {
            limit,
            spent: 0,
            best: None,
        }
    }

    /// Record the progress metric. Returns true when it ADVANCED, which is
    /// exactly when the budget resets.
    ///
    /// The first reading only establishes the baseline: it is not an advance,
    /// or every monitor would start by announcing a reset it did not earn.
    pub fn record_progress(&mut self, metric: i64) -> bool {
        match self.best {
            Some(best) if metric <= best => false,
            Some(_) => {
                self.best = Some(metric);
                self.spent = 0;
                true
            }
            None => {
                self.best = Some(metric);
                false
            }
        }
    }

    /// Ask for a nudge.
    pub fn try_nudge(&mut self) -> NudgeDecision {
        if self.spent >= self.limit {
            return NudgeDecision::Exhausted {
                without_progress: self.limit,
            };
        }
        self.spent += 1;
        NudgeDecision::Nudge {
            attempt: self.spent,
            of: self.limit,
        }
    }

    /// Nudges spent since the last advance.
    pub fn spent(&self) -> u32 {
        self.spent
    }

    /// Nudges left before a human is needed.
    pub fn remaining(&self) -> u32 {
        self.limit.saturating_sub(self.spent)
    }

    /// The highest progress metric seen so far, if any.
    pub fn best(&self) -> Option<i64> {
        self.best
    }

    /// Is the budget spent? Once true, the watcher stops nudging and escalates.
    pub fn is_exhausted(&self) -> bool {
        self.spent >= self.limit
    }
}

// ── Signal latches ──────────────────────────────────────────────────────────

/// One latch PER SIGNAL, so two signals can never overwrite each other.
///
/// DEFECT 2: the first watcher kept one `last_reported` variable for every
/// signal. The failure branch wrote it, the idle branch overwrote it, and on
/// the next poll the UNCHANGED failure text no longer matched what was stored,
/// so it fired again. Forever. One field per signal removes the interference by
/// construction.
///
/// A latch holds the KEY of what was last reported, not a boolean, so genuinely
/// new content of the same kind still gets through.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SignalLatches {
    question: Option<String>,
    stalled: Option<String>,
    blocked: Option<String>,
    /// DEFECT 3, and it is the reason this field is documented on its own:
    /// clearing the failure latch when the session resumed made the FROZEN
    /// SCROLLBACK re-fire. The pane keeps its scrollback, so the same failure
    /// text is still on screen after the resume and gets reported a second and
    /// a third time. It is therefore NEVER cleared by
    /// [`SignalLatches::note_working`]. Genuinely new failure text differs from
    /// the latched key and gets through on its own merits, which is the only
    /// path that should exist.
    failure: Option<String>,
}

impl SignalLatches {
    /// Empty latches: nothing reported yet.
    pub fn new() -> Self {
        SignalLatches::default()
    }

    /// Should this state be reported? True exactly once per distinct `key`.
    ///
    /// [`MonitorState::Working`] is never reported: silence is the correct
    /// output for a busy session. Calling it here is still meaningful, because
    /// it is what tells the latches the session resumed.
    pub fn fire_state(&mut self, state: MonitorState, key: &str) -> bool {
        let slot = match state {
            MonitorState::Question => &mut self.question,
            MonitorState::Stalled => &mut self.stalled,
            MonitorState::Blocked => &mut self.blocked,
            MonitorState::Working => {
                self.note_working();
                return false;
            }
        };
        latch(slot, key)
    }

    /// Should this failure be reported?
    pub fn fire_failure(&mut self, key: &str) -> bool {
        latch(&mut self.failure, key)
    }

    /// The session started moving again.
    ///
    /// Clears the three STOP latches so that a later stop is reported as the
    /// new event it is. Deliberately does NOT clear [`SignalLatches::failure`]:
    /// see the field's own note (defect 3).
    pub fn note_working(&mut self) {
        self.question = None;
        self.stalled = None;
        self.blocked = None;
    }

    /// The key currently latched for a state, for tests and for rendering.
    pub fn latched(&self, state: MonitorState) -> Option<&str> {
        match state {
            MonitorState::Question => self.question.as_deref(),
            MonitorState::Stalled => self.stalled.as_deref(),
            MonitorState::Blocked => self.blocked.as_deref(),
            MonitorState::Working => None,
        }
    }

    /// The key currently latched for the failure signal.
    pub fn latched_failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }
}

fn latch(slot: &mut Option<String>, key: &str) -> bool {
    if slot.as_deref() == Some(key) {
        return false;
    }
    *slot = Some(key.to_string());
    true
}

// ── Viewer naming ───────────────────────────────────────────────────────────

/// Prefix every monitor session carries. Distinct from stream.rs's `stream-`,
/// so a monitor and a stream of the SAME target never collide: they differ in
/// the first character of the name, whatever the coordinate is.
pub const MONITOR_PREFIX: &str = "monitor-";

/// stream.rs's prefix, which is swapped out rather than re-derived.
const STREAM_PREFIX: &str = "stream-";

/// Length of the disambiguating fingerprint stream.rs appends. Pinned here
/// because that constant is private there; `fingerprint_shape_matches_stream`
/// fails the build if the two ever drift.
const FINGERPRINT_LEN: usize = 6;

/// The monitor session name for a target.
///
/// The injective machinery is stream.rs's, reused rather than reimplemented:
/// [`crate::stream::viewer_name`] already neutralizes what rmux silently
/// REWRITES (`:` and `.` become `_`), already refuses to fold two coordinates
/// onto one name, and already falls back to a fingerprint when the plain form
/// cannot encode a coordinate faithfully. Swapping the prefix inherits all of
/// it, so a monitor name is injective for exactly the reasons a stream name is.
pub fn monitor_viewer_name(target: &StreamTarget) -> String {
    let candidate = with_monitor_prefix(&stream::viewer_name(target));
    if candidate.len() <= MAX_SESSION_NAME_LEN {
        return candidate;
    }
    // `monitor-` is exactly one byte longer than `stream-`, so a stem that
    // filled stream.rs's budget to the last character overflows this one.
    // Recomposing from the FINGERPRINTED form rather than truncating the plain
    // one is what keeps the name injective: the 6-character tail is a hash of
    // the coordinate label, so two coordinates a truncation would fold onto one
    // name still differ here.
    let fingerprinted = stream::fingerprinted_viewer_name(target);
    let stem = fingerprinted
        .strip_prefix(STREAM_PREFIX)
        .unwrap_or(&fingerprinted);
    let over = (MONITOR_PREFIX.len() + stem.len()).saturating_sub(MAX_SESSION_NAME_LEN);
    // The tail is `-<fingerprint>` and must survive intact; the head pays.
    let split = stem.len().saturating_sub(FINGERPRINT_LEN + 1);
    let (head, tail) = stem.split_at(split);
    let head = &head[..head.len().saturating_sub(over)];
    format!(
        "{MONITOR_PREFIX}{}{}",
        head.trim_end_matches(['-', '.']),
        tail
    )
}

fn with_monitor_prefix(stream_name: &str) -> String {
    let stem = stream_name
        .strip_prefix(STREAM_PREFIX)
        .unwrap_or(stream_name);
    format!("{MONITOR_PREFIX}{stem}")
}

// ── The shell seam ──────────────────────────────────────────────────────────

/// The poll loop the monitor session runs. Installed by `install.sh` from
/// `scripts/omega-monitor.sh`.
pub fn monitor_script_path() -> PathBuf {
    crate::config::omega_dir()
        .join("bin")
        .join("omega-monitor.sh")
}

/// The exact command string handed to `rmux new-session`.
///
/// `$HOME` is expanded HERE (Rust spawns rmux directly, with no shell of its
/// own), so the loop gets an absolute script path. The two probes are arbitrary
/// shell, so they are single-quoted; the coordinates never are, because
/// [`crate::stream::is_safe_coordinate`] has already refused anything that is
/// not a slug.
pub fn monitor_command(
    target: &StreamTarget,
    interval: u32,
    lines: u32,
    work_probe: &str,
    progress_probe: &str,
    audit_every: u32,
) -> String {
    format!(
        "{} {} {} {} {} {} {} {}",
        monitor_script_path().display(),
        target.host_arg(),
        target.session(),
        interval,
        lines,
        shell_single_quote(work_probe),
        shell_single_quote(progress_probe),
        audit_every
    )
}

/// Full argv of the session-creating call, split out so a test can pin the seam
/// without spawning rmux.
pub fn monitor_viewer_argv(
    viewer: &str,
    target: &StreamTarget,
    interval: u32,
    lines: u32,
    work_probe: &str,
    progress_probe: &str,
    audit_every: u32,
) -> Vec<String> {
    vec![
        "new-session".to_string(),
        "-d".to_string(),
        "-s".to_string(),
        viewer.to_string(),
        monitor_command(
            target,
            interval,
            lines,
            work_probe,
            progress_probe,
            audit_every,
        ),
    ]
}

/// Recover the coordinate a monitor is watching from the command it was started
/// with.
///
/// Only the first three tokens are read (`<script> <host> <session>`), which is
/// all the identity needs and is why the quoted probes further along cannot
/// confuse it. Returns `None` for anything that is not the monitor loop, which
/// is the safe direction: an unreadable session is treated as occupied, never
/// as a monitor of whatever its name suggests.
pub fn parse_monitor_command(cmd: &str) -> Option<StreamTarget> {
    let mut tokens = cmd.split_whitespace();
    let script = tokens.next()?;
    if !script.ends_with("omega-monitor.sh") {
        return None;
    }
    let host_arg = tokens.next()?;
    let session = tokens.next()?;
    if !stream::is_safe_coordinate(session) {
        return None;
    }
    if host_arg == "local" {
        Some(StreamTarget::Local {
            session: session.to_string(),
        })
    } else if stream::is_safe_coordinate(host_arg) {
        Some(StreamTarget::Remote {
            host: host_arg.to_string(),
            session: session.to_string(),
        })
    } else {
        None
    }
}

/// Parse `rmux list-panes -a -F '#S|#{pane_start_command}'` into the live
/// monitors and what each one is really watching.
pub fn parse_monitor_index(listing: &str) -> Vec<(String, StreamTarget)> {
    let mut out: Vec<(String, StreamTarget)> = Vec::new();
    for line in listing.lines() {
        let Some((name, cmd)) = line.split_once('|') else {
            continue;
        };
        let name = name.trim();
        if !name.starts_with(MONITOR_PREFIX) {
            continue;
        }
        if let Some(target) = parse_monitor_command(cmd.trim()) {
            if !out.iter().any(|(n, _)| n == name) {
                out.push((name.to_string(), target));
            }
        }
    }
    out
}

/// Full argv of one pane capture.
///
/// The remote form carries the LITERAL string `$HOME/.local/bin/rmux`: Rust
/// performs no expansion, ssh concatenates its command arguments and hands them
/// to the REMOTE login shell, so `$HOME` resolves on the remote box. Expanding
/// it locally once told a Linux box to read a macOS path, silently.
pub fn capture_pane_argv(target: &StreamTarget, lines: u32) -> Vec<String> {
    match target.host() {
        None => vec![
            stream::rmux_bin().display().to_string(),
            "capture-pane".to_string(),
            "-p".to_string(),
            "-t".to_string(),
            target.session().to_string(),
            "-S".to_string(),
            format!("-{lines}"),
        ],
        Some(host) => vec![
            "ssh".to_string(),
            "-o".to_string(),
            "BatchMode=yes".to_string(),
            "-o".to_string(),
            format!("ConnectTimeout={}", stream::SSH_CONNECT_TIMEOUT_SECS),
            host.to_string(),
            format!(
                "$HOME/.local/bin/rmux capture-pane -p -t {} -S -{lines}",
                shell_single_quote(target.session())
            ),
        ],
    }
}

/// Full argv of the keystroke half of a nudge.
///
/// `-l` sends the text LITERALLY, so a message containing something that looks
/// like a key name (`Enter`, `C-c`) is typed rather than pressed.
pub fn send_keys_argv(target: &StreamTarget, text: &str) -> Vec<String> {
    match target.host() {
        None => vec![
            stream::rmux_bin().display().to_string(),
            "send-keys".to_string(),
            "-t".to_string(),
            target.session().to_string(),
            "-l".to_string(),
            text.to_string(),
        ],
        Some(host) => vec![
            "ssh".to_string(),
            "-o".to_string(),
            "BatchMode=yes".to_string(),
            "-o".to_string(),
            format!("ConnectTimeout={}", stream::SSH_CONNECT_TIMEOUT_SECS),
            host.to_string(),
            format!(
                "$HOME/.local/bin/rmux send-keys -t {} -l {}",
                shell_single_quote(target.session()),
                shell_single_quote(text)
            ),
        ],
    }
}

/// Full argv of the Enter half of a nudge.
///
/// SEPARATE CALL, always. Appending Enter to the text argument of the same
/// send-keys types the word "Enter" into the composer instead of submitting,
/// and with `-l` in play it is guaranteed to.
pub fn send_enter_argv(target: &StreamTarget) -> Vec<String> {
    match target.host() {
        None => vec![
            stream::rmux_bin().display().to_string(),
            "send-keys".to_string(),
            "-t".to_string(),
            target.session().to_string(),
            "Enter".to_string(),
        ],
        Some(host) => vec![
            "ssh".to_string(),
            "-o".to_string(),
            "BatchMode=yes".to_string(),
            "-o".to_string(),
            format!("ConnectTimeout={}", stream::SSH_CONNECT_TIMEOUT_SECS),
            host.to_string(),
            format!(
                "$HOME/.local/bin/rmux send-keys -t {} Enter",
                shell_single_quote(target.session())
            ),
        ],
    }
}

/// Wrap a string so a POSIX shell receives it verbatim, including a literal
/// single quote.
pub fn shell_single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

// ── Live rmux reads ─────────────────────────────────────────────────────────

/// Every live monitor on this box and the coordinate it is really watching.
///
/// One rmux call for the whole index. An empty result means "nothing readable",
/// never "nothing monitored", so callers must not treat it as proof.
pub async fn live_monitor_index() -> Vec<(String, StreamTarget)> {
    let mut cmd = tokio::process::Command::new(stream::rmux_bin());
    cmd.args(["list-panes", "-a", "-F", "#S|#{pane_start_command}"])
        .stdin(std::process::Stdio::null())
        .kill_on_drop(true);
    match tokio::time::timeout(stream::PROBE_TIMEOUT, cmd.output()).await {
        Ok(Ok(out)) if out.status.success() => {
            parse_monitor_index(&String::from_utf8_lossy(&out.stdout))
        }
        _ => Vec::new(),
    }
}

/// Create the detached monitor session that runs the poll loop.
///
/// Verifies afterwards that a session under exactly this name exists: rmux
/// rewrites characters it dislikes instead of refusing them, so "created" is
/// not proof that the name we will attach to is the name it stored.
#[allow(clippy::too_many_arguments)]
pub async fn create_monitor(
    viewer: &str,
    target: &StreamTarget,
    interval: u32,
    lines: u32,
    work_probe: &str,
    progress_probe: &str,
    audit_every: u32,
) -> anyhow::Result<()> {
    let script = monitor_script_path();
    if !script.is_file() {
        anyhow::bail!(
            "the monitor loop is not installed at {}. Run install.sh (or `omega sync`) first: \
             creating a monitor without it would leave a session that dies instantly",
            script.display()
        );
    }
    let argv = monitor_viewer_argv(
        viewer,
        target,
        interval,
        lines,
        work_probe,
        progress_probe,
        audit_every,
    );
    let out = tokio::process::Command::new(stream::rmux_bin())
        .args(&argv)
        .stdin(std::process::Stdio::null())
        .output()
        .await?;
    if !out.status.success() {
        let detail = String::from_utf8_lossy(&out.stderr)
            .lines()
            .map(str::trim)
            .rfind(|l| !l.is_empty())
            .unwrap_or("(no error output)")
            .to_string();
        anyhow::bail!(
            "rmux new-session failed (exit {:?}): {detail}",
            out.status.code()
        );
    }
    if !stream::session_exists(viewer).await {
        anyhow::bail!(
            "rmux reported success but no session named {viewer} exists. \
             The name was probably rewritten, so there is nothing to attach to"
        );
    }
    Ok(())
}

/// Run a probe command through the user's shell, bounded by its OWN wall clock.
///
/// The bound is not a nicety. `ConnectTimeout` bounds only the CONNECT: a probe
/// that reaches a box which answers TCP and then never replies parks the poll
/// forever, no new frame is drawn, and a frozen monitor is indistinguishable
/// from a quiet one. Returns `None` on anything that is not a clean run, and
/// [`WorkSignal::from_probe_output`] reads `None` as work rather than as zero.
pub async fn run_probe(command: &str, bound: std::time::Duration) -> Option<String> {
    if command.trim().is_empty() {
        return None;
    }
    let mut cmd = tokio::process::Command::new("sh");
    cmd.arg("-c")
        .arg(command)
        .stdin(std::process::Stdio::null())
        .kill_on_drop(true);
    match tokio::time::timeout(bound, cmd.output()).await {
        Ok(Ok(out)) if out.status.success() => {
            Some(String::from_utf8_lossy(&out.stdout).to_string())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── The real captures ────────────────────────────────────────────────────
    //
    // These four are genuine `rmux capture-pane` output from live sessions, not
    // synthetic text, and they are committed under tests/fixtures/ so a fresh
    // clone proves the classifier exactly as this box does. Synthetic panes
    // agree with whatever the classifier happens to do; real ones do not.

    const PANE_QUESTION: &str = include_str!("../tests/fixtures/GOLDEN-question-real.txt");
    const PANE_STOPPED: &str = include_str!("../tests/fixtures/GOLDEN-stalled-real.txt");
    const PANE_WORKING: &str = include_str!("../tests/fixtures/MoonBaseCapital-claude.txt");
    const PANE_SELF_ECHO: &str =
        include_str!("../tests/fixtures/GOLDEN-self-echo-false-question.txt");

    // The adversarial corpus. Each of these is the REAL question capture with one
    // ordinary thing added above the modal, and each one classified STALLED or
    // WORKING before COMPOSER_TAIL_SPAN existed. STALLED is the dangerous answer:
    // the watcher nudges a stall, and nudging a question answers it.
    const PANE_Q_OPTION_UNDER_RULE: &str =
        include_str!("../tests/fixtures/adv-question-option-under-rule.txt");
    const PANE_Q_RULE_ARROW: &str =
        include_str!("../tests/fixtures/adv-question-with-rule-arrow.txt");
    const PANE_Q_RULE_BLOCKQUOTE: &str =
        include_str!("../tests/fixtures/adv-question-with-rule-blockquote.txt");
    const PANE_Q_STALE_INTERRUPT: &str =
        include_str!("../tests/fixtures/adv-question-stale-interrupt.txt");

    /// A question modal draws `rule` then `❯ 1. Option A`, the same shape as a
    /// composer. The real capture survived only because ONE blank line sat
    /// between them. These four panes are that capture with an ordinary line
    /// added above the modal, and every one of them answered STALLED or WORKING
    /// before the composer was anchored to the bottom of the screen.
    ///
    /// STALLED is the outcome that matters: the watcher nudges a stall, and a
    /// nudge typed into a question ANSWERS it. A monitor that auto-answers the
    /// question it was supposed to escalate is worse than no monitor, so this
    /// test is the one that must never go green by accident.
    #[test]
    fn an_ordinary_line_above_a_modal_never_downgrades_a_live_question() {
        for (name, pane) in [
            (
                "option list directly under its own rule",
                PANE_Q_OPTION_UNDER_RULE,
            ),
            ("a rule then a quoted arrow line", PANE_Q_RULE_ARROW),
            ("a rule then a markdown blockquote", PANE_Q_RULE_BLOCKQUOTE),
        ] {
            assert_eq!(
                classify(pane, WorkSignal::Available),
                MonitorState::Question,
                "{name}: a live question was downgraded, and a downgrade to Stalled gets it nudged"
            );
        }
    }

    /// The same root cause wearing different clothes. When the composer was
    /// found high in the pane, `live_chrome` widened to nearly the whole screen
    /// and any stale `esc to interrupt` still sitting in scrollback pinned the
    /// session to Working, which reports NOTHING at all. A question that is
    /// never reported is a session that waits forever.
    #[test]
    fn a_stale_interrupt_above_a_modal_never_silences_a_live_question() {
        assert_eq!(
            classify(PANE_Q_STALE_INTERRUPT, WorkSignal::Available),
            MonitorState::Question,
            "a stale activity marker in scrollback silenced a live question"
        );
    }

    /// The bound must not swallow the thing it is bounding: a real composer sits
    /// within a few lines of the bottom and must still be found, or every
    /// stopped session would read as a modal.
    #[test]
    fn the_composer_is_still_found_at_the_bottom_of_a_real_capture() {
        assert!(
            find_input_box(PANE_STOPPED).is_some(),
            "the real stopped capture draws a composer and it must still be located"
        );
        assert_eq!(
            classify(PANE_STOPPED, WorkSignal::Available),
            MonitorState::Stalled
        );
        assert_eq!(
            classify(PANE_STOPPED, WorkSignal::None),
            MonitorState::Blocked
        );
    }

    #[test]
    fn a_real_question_pane_is_a_question() {
        // Line 24 of the capture is
        // `Enter to select · ↑/↓ to navigate · Esc to cancel`.
        assert!(
            PANE_QUESTION.contains("Enter to select"),
            "fixture premise: the capture carries the question hint"
        );
        for work in [WorkSignal::Available, WorkSignal::None, WorkSignal::Unknown] {
            assert_eq!(
                classify(PANE_QUESTION, work),
                MonitorState::Question,
                "a question outranks every work signal: nudging it would auto-answer it"
            );
        }
        assert!(!classify(PANE_QUESTION, WorkSignal::Available).should_nudge());
        assert!(classify(PANE_QUESTION, WorkSignal::Available).needs_human());
    }

    #[test]
    fn the_same_session_after_the_turn_is_stalled_or_blocked_by_the_probe() {
        // GOLDEN-stalled-real is the SAME session as GOLDEN-question-real, one
        // turn later: the question was answered and neither marker is on
        // screen. Only the work probe can tell a stall from a block.
        assert_eq!(
            classify(PANE_STOPPED, WorkSignal::Available),
            MonitorState::Stalled
        );
        assert_eq!(
            classify(PANE_STOPPED, WorkSignal::None),
            MonitorState::Blocked
        );
        assert_eq!(
            classify(PANE_STOPPED, WorkSignal::Unknown),
            MonitorState::Stalled,
            "an unreadable probe must never stall the build silently"
        );
        assert!(classify(PANE_STOPPED, WorkSignal::Available).should_nudge());
        assert!(
            !classify(PANE_STOPPED, WorkSignal::None).should_nudge(),
            "nudging a blocked session is manufactured thrash, not persistence"
        );
    }

    #[test]
    fn a_real_working_pane_is_working_whatever_the_probe_says() {
        for work in [WorkSignal::Available, WorkSignal::None, WorkSignal::Unknown] {
            assert_eq!(
                classify(PANE_WORKING, work),
                MonitorState::Working,
                "a busy session is never nudged and never escalated"
            );
        }
        assert!(!classify(PANE_WORKING, WorkSignal::None).needs_human());
    }

    /// THE HARD CASE, and the reason the strip runs before the marker test.
    ///
    /// This capture contains the watcher's OWN probe echoed as a tool call:
    /// `q=$(grep -cE 'Enter to select.*to navigate' "panes/$s.txt")`. That one
    /// line satisfies a both-markers-on-one-line test perfectly, so the
    /// structural test alone is NOT sufficient.
    #[test]
    fn our_own_echoed_grep_is_not_a_question() {
        let echoed = PANE_SELF_ECHO
            .lines()
            .filter(|l| is_question_hint_line(l))
            .count();
        assert!(
            echoed > 0,
            "fixture premise: a naive both-markers-on-one-line test DOES fire here"
        );

        assert!(
            !question_ui_visible(PANE_SELF_ECHO),
            "the watcher matched its own probe command as a pending question"
        );
        for work in [WorkSignal::Available, WorkSignal::None, WorkSignal::Unknown] {
            assert_ne!(
                classify(PANE_SELF_ECHO, work),
                MonitorState::Question,
                "self-echo must never escalate to a human as a question"
            );
        }
        // That session was in fact busy, which is what it should be called.
        assert_eq!(
            classify(PANE_SELF_ECHO, WorkSignal::Available),
            MonitorState::Working
        );
    }

    // ── The question test ────────────────────────────────────────────────────

    #[test]
    fn either_marker_alone_is_not_a_question() {
        // The prose that trained this rule: a code review saying there is
        // nothing to navigate to was reported as a pending question.
        let prose = "\
● The panel has no list, so there is nothing to navigate to while it is open.
  Press Enter to select the default and move on, they said, but there is no list.
";
        assert!(!question_ui_visible(prose));
        assert_eq!(
            classify(prose, WorkSignal::Available),
            MonitorState::Stalled
        );
    }

    #[test]
    fn both_markers_on_one_line_with_no_composer_is_a_question() {
        let pane = "● Ready.\n\nEnter to select · ↑/↓ to navigate · Esc to cancel\n";
        assert!(question_ui_visible(pane));
    }

    #[test]
    fn a_marker_line_buried_in_scrollback_is_not_live_ui() {
        // No composer at all (a plain shell, or a pane cleared of its chrome),
        // with the echoed probe far above the bottom of the screen. A modal
        // renders at the BOTTOM by definition, so this cannot be one.
        let mut pane = String::from("$ grep -cE 'Enter to select.*to navigate' pane.txt\n");
        for i in 0..40 {
            pane.push_str(&format!("line {i}\n"));
        }
        assert!(
            !question_ui_visible(&pane),
            "an echoed command deep in the scrollback is not a live question"
        );
    }

    // ── The input box ────────────────────────────────────────────────────────

    #[test]
    fn the_input_box_is_found_structurally_in_a_real_capture() {
        let rule = find_input_box(PANE_STOPPED).expect("the composer is on screen");
        let lines: Vec<&str> = PANE_STOPPED.lines().collect();
        assert!(
            is_horizontal_rule(lines[rule]),
            "the cut lands on the composer's top rule: {:?}",
            lines[rule]
        );
        assert!(is_prompt_marker_line(lines[rule + 1]));

        let stripped = strip_input_box(PANE_STOPPED);
        assert!(
            !stripped.contains("bypass permissions on"),
            "the status bar under the composer must go with it"
        );
        assert!(
            stripped.contains("User answered Claude's questions"),
            "the transcript above the composer must survive"
        );
    }

    /// DEFECT 5, reproduced: the composer holds a message far longer than any
    /// fixed tail drop, and the part that survived the drop was matched as if
    /// the agent had said it.
    #[test]
    fn a_long_composer_message_is_cut_whole_not_by_a_fixed_tail() {
        let mut pane = String::from("● The build is green.\n");
        pane.push_str(&"─".repeat(60));
        pane.push('\n');
        pane.push_str("❯ nudge: continue with the plan\n");
        for i in 0..60 {
            pane.push_str(&format!("  and also step {i}: MISSING ROW: nothing\n"));
        }
        pane.push_str(&"─".repeat(60));
        pane.push('\n');
        pane.push_str("  ⏵⏵ bypass permissions on\n");

        let stripped = strip_input_box(&pane);
        assert!(
            !stripped.contains("MISSING ROW:"),
            "60 lines of composer content survived a fixed-size tail drop; \
             the cut must be anchored on the prompt marker"
        );
        assert!(stripped.contains("The build is green."));
    }

    #[test]
    fn a_question_option_list_is_not_mistaken_for_the_composer() {
        // `❯ 1. Option A` in the real question capture follows prose, not a
        // rule. Only the rule-then-marker adjacency identifies the composer.
        assert_eq!(
            find_input_box(PANE_QUESTION),
            None,
            "the question UI REPLACES the composer; finding one here would \
             silence every real question"
        );
        assert!(PANE_QUESTION.contains("❯ 1. Option A"), "fixture premise");
    }

    #[test]
    fn a_pane_with_no_composer_strips_to_itself() {
        let pane = "● one\n● two\n";
        assert_eq!(strip_input_box(pane), "● one\n● two");
        assert_eq!(find_input_box(""), None);
        assert_eq!(strip_input_box(""), "");
    }

    // ── The working test ─────────────────────────────────────────────────────

    #[test]
    fn a_finished_spinner_line_is_not_activity() {
        // `✻ Cooked for 3s` is the spinner line AFTER the turn ended: the glyph
        // is still there and the token count is gone.
        assert!(PANE_STOPPED.contains("✻ Cooked for 3s"), "fixture premise");
        assert!(!working_indicator_visible(PANE_STOPPED));
        assert!(working_indicator_visible(PANE_WORKING));
    }

    #[test]
    fn a_stale_interrupt_marker_in_scrollback_does_not_pin_a_stopped_session() {
        let mut pane = String::from("● an old frame said: esc to interrupt\n");
        for i in 0..40 {
            pane.push_str(&format!("● step {i} done\n"));
        }
        pane.push_str(&"─".repeat(60));
        pane.push('\n');
        pane.push_str("❯ \n");
        pane.push_str(&"─".repeat(60));
        pane.push('\n');
        pane.push_str("  ⏵⏵ bypass permissions on · ← 2 agents\n");
        assert!(
            !working_indicator_visible(&pane),
            "a stopped session would never be nudged again"
        );
        assert_eq!(
            classify(&pane, WorkSignal::Available),
            MonitorState::Stalled
        );
    }

    #[test]
    fn the_status_bar_under_the_composer_still_counts_as_working() {
        // The most reliable activity marker lives BELOW the composer, which is
        // why the working test runs on the unstripped pane.
        let last = PANE_WORKING
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap();
        assert!(last.contains(WORKING_MARKER_INTERRUPT), "fixture premise");
        assert!(!strip_input_box(PANE_WORKING).contains(WORKING_MARKER_INTERRUPT));
        assert!(working_indicator_visible(PANE_WORKING));
    }

    // ── May a mission body be pasted here? ──────────────────────────────────
    //
    // Every capture below is `rmux capture-pane -p` output from a REAL session.
    // The eight shell panes are the oracle shape `bash -c '<agent …>; exec
    // bash'` (agents.rs:452) replayed eight times with a different `PS1`, each
    // one a live rmux session at 80x24 whose frame is this crate's own
    // `GOLDEN-stalled-real.txt`. Four of them were ACCEPTED as followup targets
    // before this predicate was inverted, and a re-audit pasted a mission into
    // one and watched the body run as shell commands.

    const PANE_DRAFT_BLANK_FIRST_LINE: &str =
        include_str!("../tests/fixtures/GOLDEN-draft-blank-first-line.txt");
    const PANE_OPTIONS_ABOVE_COMPOSER: &str =
        include_str!("../tests/fixtures/adv-option-list-above-composer.txt");
    /// The same dead-agent shell with a command already half-typed at its
    /// prompt, which moves the sigil off the end of the line.
    const PANE_SHELL_DISTRO_TYPED: &str =
        include_str!("../tests/fixtures/adv-shell-ps1-distro-typed.txt");

    /// name -> the real capture of a dead agent with that `PS1` under its frame.
    const DEAD_AGENT_SHELLS: &[(&str, &str)] = &[
        (
            "distro   \\u@\\h:\\w\\$",
            include_str!("../tests/fixtures/adv-shell-ps1-distro.txt"),
        ),
        (
            "starship ❯",
            include_str!("../tests/fixtures/adv-shell-ps1-starship.txt"),
        ),
        (
            "inline   \\u@\\h \\w ❯",
            include_str!("../tests/fixtures/adv-shell-ps1-inlinearrow.txt"),
        ),
        (
            "ohmybash ➜  omegaos git:(main)",
            include_str!("../tests/fixtures/adv-shell-ps1-ohmybash.txt"),
        ),
        (
            "angle    \\w >",
            include_str!("../tests/fixtures/adv-shell-ps1-angle.txt"),
        ),
        (
            "plain    omegaos",
            include_str!("../tests/fixtures/adv-shell-ps1-plainname.txt"),
        ),
        (
            "sigil    \\W \\$",
            include_str!("../tests/fixtures/adv-shell-ps1-baresigil.txt"),
        ),
        (
            "root     omegaos #",
            include_str!("../tests/fixtures/adv-shell-ps1-rootsigil.txt"),
        ),
        (
            "typed    \\u@\\h:\\w\\$ + a command",
            include_str!("../tests/fixtures/adv-shell-ps1-distro-typed.txt"),
        ),
    ];

    /// R1, REPRODUCED ON A LIVE SESSION. The operator's draft opens with a blank
    /// line, so the marker is bare and the text sits one line UNDER it. Reading
    /// the marker alone called that composer empty, and the paste came back as
    /// one turn: `fais aussi le refactor avant de fermerMISSION SUIVI: …`.
    ///
    /// This test fails the moment emptiness is judged on a line again.
    #[test]
    fn a_draft_under_a_bare_marker_is_not_an_empty_composer() {
        let marker = find_live_composer_marker(PANE_DRAFT_BLANK_FIRST_LINE)
            .expect("fixture premise: the capture draws a live composer");
        let marker_line = PANE_DRAFT_BLANK_FIRST_LINE.lines().nth(marker).unwrap();
        assert!(
            marker_line.trim().chars().count() <= 1,
            "fixture premise: the marker line itself is bare — that is the trap"
        );
        assert!(
            PANE_DRAFT_BLANK_FIRST_LINE.contains("fais aussi le refactor avant de fermer"),
            "fixture premise: the draft is one line below the marker"
        );
        assert!(
            !composer_is_empty(PANE_DRAFT_BLANK_FIRST_LINE, marker),
            "a draft below the marker is still a draft: the box is what is empty or not"
        );
        assert!(!composer_ready_for_paste(PANE_DRAFT_BLANK_FIRST_LINE));
    }

    /// R2, REPRODUCED IN RUNTIME ON SIX REAL SESSIONS. The shell that takes over
    /// when an agent dies reads the USER'S `~/.bashrc`, so its prompt is not a
    /// constant of this project. Four of these eight were accepted while the
    /// predicate tried to recognise a shell.
    ///
    /// Nothing here asks what the foreign line IS. Under the composer's closing
    /// rule only the agent's own footer may appear, so every prompt fails, and
    /// so does the ninth `PS1` nobody has captured yet.
    #[test]
    fn no_ps1_turns_a_dead_agent_shell_into_a_paste_target() {
        for (ps1, pane) in DEAD_AGENT_SHELLS {
            assert!(
                find_live_composer_marker(pane).is_some(),
                "fixture premise ({ps1}): the dead agent's frame still draws its composer"
            );
            let marker = find_live_composer_marker(pane).unwrap();
            assert!(
                composer_is_empty(pane, marker),
                "fixture premise ({ps1}): the frozen composer IS empty — the box is not what saves us here"
            );
            assert!(
                !composer_ready_for_paste(pane),
                "PS1 {ps1}: a live shell under a dead agent's frame accepted a mission body"
            );
        }
    }

    /// The other half of the same rule: the agent's real footer must survive it.
    /// `GOLDEN-self-echo-false-question.txt` draws a task-panel row under its
    /// status bar and two live oracles drew a token-count row, so a rule that
    /// demanded blank lines under the bar would refuse the nominal path it
    /// exists to protect.
    #[test]
    fn the_agents_own_footer_rows_are_not_foreign_lines() {
        assert!(
            composer_ready_for_paste(PANE_SELF_ECHO),
            "the real capture with a task row under its status bar must stay a target"
        );
        assert!(composer_ready_for_paste(PANE_STOPPED));
        assert!(composer_ready_for_paste(PANE_WORKING));
        // Real footer rows, from live oracle captures.
        for row in [
            "                                                        400384 tokens",
            "                        new task? /clear to save 250k tokens",
            "  ◯ build-omega-monitor  Build /monitor",
            "",
        ] {
            assert!(
                is_composer_footer_line(row),
                "real footer row read as foreign: {row:?}"
            );
        }
        // A count is what makes the token row chrome, not the bare word: a shell
        // sitting in a directory called `tokens` is not the agent's footer.
        for foreign in ["/home/vibe/tokens", "vibe@host ~/src/tokens $", "omegaos"] {
            assert!(
                !is_composer_footer_line(foreign),
                "a foreign line was read as the agent's own footer: {foreign:?}"
            );
        }
    }

    /// The selection-list guard, EXECUTED. A permission dialog draws
    /// `❯ 1. Yes` with a footer that is not the question hint, so the question
    /// test never sees it while Enter would still pick the highlighted option.
    ///
    /// This pane is the real stopped capture with the real Write-permission
    /// dialog's own option block above its composer. Every other reason to
    /// refuse is switched off on purpose — the composer below is genuinely
    /// empty and its footer is genuinely the agent's — so the numbered-option
    /// guard is the only thing standing between this pane and a paste, and
    /// deleting that guard turns this test red.
    #[test]
    fn a_selection_list_above_a_live_composer_is_never_a_paste_target() {
        let marker = find_live_composer_marker(PANE_OPTIONS_ABOVE_COMPOSER)
            .expect("fixture premise: the pane draws a live composer");
        assert!(
            composer_is_empty(PANE_OPTIONS_ABOVE_COMPOSER, marker),
            "fixture premise: the composer is empty, so emptiness cannot be what refuses this"
        );
        assert!(
            PANE_OPTIONS_ABOVE_COMPOSER
                .lines()
                .any(|l| l.trim() == "❯ 1. Yes"),
            "fixture premise: a real dialog's selected option is on screen"
        );
        assert!(
            !question_ui_visible(PANE_OPTIONS_ABOVE_COMPOSER),
            "fixture premise: this dialog draws no question hint — that is why it is dangerous"
        );
        assert!(
            !composer_ready_for_paste(PANE_OPTIONS_ABOVE_COMPOSER),
            "Enter on a selection list picks the highlighted option"
        );
    }

    /// The shell-prompt blacklist, EXECUTED on its own terms. It rides UNDER the
    /// footer whitelist as defence in depth, so a pane test cannot reach it: the
    /// contract is pinned directly instead.
    ///
    /// The two sigil prompts below come from real captures (`\W \$` and a root
    /// `#`) and are caught by NOTHING but the trailing-sigil arm — no `@`, no
    /// `:~`, no `:/` — so dropping that arm turns this test red.
    #[test]
    fn a_trailing_shell_sigil_is_a_shell_prompt() {
        for (name, pane) in DEAD_AGENT_SHELLS
            .iter()
            .filter(|(n, _)| n.starts_with("sigil") || n.starts_with("root"))
        {
            let last = pane.lines().rev().find(|l| !l.trim().is_empty()).unwrap();
            assert!(
                !last.contains('@') && !last.contains(":~") && !last.contains(":/"),
                "fixture premise ({name}): only the sigil arm can catch {last:?}"
            );
            assert!(
                is_shell_prompt_line(last),
                "a real shell prompt went unrecognised: {last:?}"
            );
        }
        // The other arm, and the shape that needs it: a prompt with a command
        // already half-typed at it puts the sigil in the MIDDLE of the line.
        // Real capture, distro `PS1` with `omega status …` typed and not sent.
        let typed = PANE_SHELL_DISTRO_TYPED
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap();
        assert!(
            !matches!(
                typed.trim().chars().last(),
                Some('$') | Some('#') | Some('%')
            ),
            "fixture premise: the sigil is no longer last, so only the user@host arm can catch it"
        );
        assert!(
            is_shell_prompt_line(typed),
            "a prompt with a half-typed command went unrecognised: {typed:?}"
        );
        // And it stays a predicate about prompts, not about prose.
        assert!(!is_shell_prompt_line("● Ready."));
        assert!(!is_shell_prompt_line(
            "  ⏵⏵ bypass permissions on (shift+tab to cycle)"
        ));
    }

    // ── The work probe ───────────────────────────────────────────────────────

    #[test]
    fn the_work_probe_reads_an_integer_and_distrusts_everything_else() {
        assert_eq!(WorkSignal::from_probe_output("7\n"), WorkSignal::Available);
        assert_eq!(
            WorkSignal::from_probe_output("  3  "),
            WorkSignal::Available
        );
        assert_eq!(WorkSignal::from_probe_output("0\n"), WorkSignal::None);
        for junk in [
            "",
            "  ",
            "none",
            "bash: jq: command not found",
            "3 steps",
            "-1",
        ] {
            assert_eq!(
                WorkSignal::from_probe_output(junk),
                WorkSignal::Unknown,
                "unreadable probe output must not be read as zero: {junk:?}"
            );
        }
        // A probe that logs before it answers: the LAST line is the answer.
        assert_eq!(
            WorkSignal::from_probe_output("reading plan...\n4\n"),
            WorkSignal::Available
        );
    }

    #[test]
    fn awaiting_a_sign_off_is_work() {
        // DEFECT 7 as a contract test: the probe counts work in any form, so a
        // build whose remaining steps all await a human still reports a count,
        // and that count must produce Stalled rather than Blocked.
        assert_eq!(WorkSignal::from_probe_output("12\n"), WorkSignal::Available);
        assert_eq!(
            classify(PANE_STOPPED, WorkSignal::Available),
            MonitorState::Stalled
        );
    }

    // ── Failure signals ──────────────────────────────────────────────────────

    /// DEFECT 4: the two prose lines that trained the punctuation rule.
    #[test]
    fn failure_matching_is_structural_not_lexical() {
        let prose = "\
● Fixed the missing redirect that the audit flagged.
● That second helper is REDundant now, so it went.
● The runner exited with exit code 0, everything is green.
";
        assert!(
            failure_signals(prose, &[]).is_empty(),
            "prose describing a fixed defect is not a failure: {:?}",
            failure_signals(prose, &[])
        );

        let real = "\
  MISSING ROW: 12 of the ledger
  gate 4 is RED.
  runner finished with exit code 3
  Traceback (most recent call last):
";
        let signals = failure_signals(real, &[]);
        assert_eq!(
            signals.len(),
            4,
            "each exact structure fires once: {signals:?}"
        );
        assert!(signals.iter().any(|s| s.contains("MISSING ROW: 12")));
        assert!(signals.iter().any(|s| s.contains("is RED.")));
        assert!(signals.iter().any(|s| s.contains("exit code 3")));
    }

    /// DEFECT 1: the watcher's own nudge, echoed by the pane, read back as the
    /// agent's verdict.
    #[test]
    fn our_own_message_is_excluded_by_content_in_short_slices() {
        let sent = sent_slices(
            "check the ledger again, the gate is RED. and MISSING ROW: 12 still stands",
        );
        // The pane hard-wraps, so no captured line holds the whole message.
        let pane = "\
❯ check the ledger again, the gate is RED. and MISSING ROW: 12 still
  stands
● Reading the ledger.
  MISSING ROW: 41 of the ledger
";
        let signals = failure_signals(pane, &sent);
        assert!(
            !signals.iter().any(|s| s.contains("MISSING ROW: 12")),
            "our own echoed nudge was reported as a failure: {signals:?}"
        );
        assert!(
            signals.iter().any(|s| s.contains("MISSING ROW: 41")),
            "the agent's REAL failure must still get through: {signals:?}"
        );
    }

    #[test]
    fn sent_slices_are_short_enough_to_survive_a_hard_wrap() {
        let msg = "continue with the plan and report the ledger rows that are still unsigned";
        let slices = sent_slices(msg);
        assert!(!slices.is_empty());
        for s in &slices {
            assert!(
                s.chars().count() <= MAX_SLICE_CHARS,
                "a whole-line match never fires against a wrapped pane: {s:?}"
            );
            assert!(msg.contains(s.as_str()), "a slice must be verbatim: {s:?}");
        }
        // Overlapping windows: whatever column the wrap lands on, one survives.
        assert!(slices.len() > 3, "windows must overlap: {slices:?}");
    }

    #[test]
    fn an_empty_exclusion_list_excludes_nothing_rather_than_everything() {
        // The shell watcher fed this list to `grep -f`, and `grep -f` against
        // an EMPTY file matches NOTHING, which silently blanked the whole
        // stream. Here the empty case cannot invert.
        assert!(sent_slices("").is_empty());
        assert!(sent_slices("   \n  ").is_empty());
        let pane = "  MISSING ROW: 7 of the ledger\n";
        assert_eq!(failure_signals(pane, &[]).len(), 1);
        assert_eq!(failure_signals(pane, &sent_slices("")).len(), 1);
    }

    #[test]
    fn a_short_message_is_still_recorded_whole() {
        let slices = sent_slices("go on");
        assert_eq!(slices, vec!["go on".to_string()]);
    }

    #[test]
    fn failure_signals_are_deduped_and_capped() {
        let mut pane = String::new();
        for _ in 0..4 {
            pane.push_str("  MISSING ROW: 12 of the ledger\n");
        }
        assert_eq!(
            failure_signals(&pane, &[]).len(),
            1,
            "the same line re-renders on every frame"
        );

        let mut many = String::new();
        for i in 0..20 {
            many.push_str(&format!("  MISSING ROW: {i} of the ledger\n"));
        }
        assert_eq!(failure_signals(&many, &[]).len(), MAX_FAILURE_SIGNALS);
    }

    #[test]
    fn exit_code_zero_is_a_pass_and_the_digit_is_part_of_the_pattern() {
        assert!(failure_signals("  done, exit code 0\n", &[]).is_empty());
        assert!(!failure_signals("  died, exit code 1\n", &[]).is_empty());
        assert!(!failure_signals("  died, exit code 137\n", &[]).is_empty());
        assert!(!failure_signals("  died, exit status 2\n", &[]).is_empty());
        // Both forms on one line, the harmless one first: the scan must not
        // stop at the first prefix it finds.
        assert!(!failure_signals("  exit code 0 then exit code 4\n", &[]).is_empty());
    }

    #[test]
    fn a_signal_line_is_cut_by_characters_never_by_bytes() {
        let line = format!("  MISSING ROW: {}\n", "─↑↓✻".repeat(200));
        let signals = failure_signals(&line, &[]);
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].chars().count(), MAX_SIGNAL_CHARS);
    }

    // ── Task transitions ─────────────────────────────────────────────────────

    /// DEFECT 6, on the real capture: the MoonBase pane carries ten completed
    /// items that re-render on every frame beside the four that say what is
    /// happening now.
    #[test]
    fn completed_task_lines_are_dropped_before_diffing() {
        let active = active_task_lines(PANE_WORKING);
        assert!(!active.is_empty(), "the capture has a live task list");
        assert!(
            !active.iter().any(|l| l.starts_with('✔')),
            "a completed item is churn, not a transition: {active:?}"
        );
        assert!(
            !active.iter().any(|l| l.contains("+10 completed")),
            "the roll-up line is churn too: {active:?}"
        );
        assert!(
            active
                .iter()
                .any(|l| l.contains("Record the specimen-inherited exemption")),
            "the in-progress item is the whole point: {active:?}"
        );
    }

    #[test]
    fn only_new_active_items_are_reported_as_transitions() {
        let prev = "\
  ⎿  ◼ Design the classifier
     ◻ Build the watcher loop
     ✔ Read the contract
";
        let now = "\
  ⎿  ✔ Design the classifier
     ◼ Build the watcher loop
     ◻ Ship the skill
     ✔ Read the contract
      … +2 completed
";
        assert_eq!(
            task_transitions(prev, now),
            vec![
                // The promotion from pending to in-progress: this is the line
                // that says what is happening NOW, and it is exactly what the
                // completed items used to drown.
                "◼ Build the watcher loop".to_string(),
                "◻ Ship the skill".to_string(),
            ],
            "a promotion and a new item are transitions; merely COMPLETING is not"
        );
        assert!(
            !task_transitions(prev, now)
                .iter()
                .any(|l| l.contains("Design the classifier")),
            "the item that only completed between the two frames is churn"
        );
        assert!(
            task_transitions(now, now).is_empty(),
            "no churn on a still frame"
        );
    }

    #[test]
    fn task_transitions_are_capped() {
        let mut now = String::new();
        for i in 0..30 {
            now.push_str(&format!("     ◻ task {i}\n"));
        }
        assert_eq!(task_transitions("", &now).len(), MAX_TASK_TRANSITIONS);
    }

    // ── The nudge bound ──────────────────────────────────────────────────────

    /// The lesson in one test: a flat cap of N stops a HEALTHY long run. The
    /// bound is on the ABSENCE of progress, so a build that keeps advancing may
    /// be nudged far more than N times over its life.
    #[test]
    fn progress_resets_the_budget_so_a_healthy_long_run_is_never_stopped() {
        let mut budget = NudgeBudget::new(3);
        budget.record_progress(0); // baseline, not an advance

        let mut nudges = 0;
        for step in 1..=40 {
            for _ in 0..2 {
                assert!(
                    matches!(budget.try_nudge(), NudgeDecision::Nudge { .. }),
                    "a cap of 3 ran out around step 40 of 157 on a build that was moving"
                );
                nudges += 1;
            }
            assert!(budget.record_progress(step), "the metric advanced");
            assert_eq!(budget.spent(), 0, "an advance resets the budget");
        }
        assert_eq!(nudges, 80);
    }

    #[test]
    fn the_budget_exhausts_only_after_nudges_that_produced_nothing() {
        let mut budget = NudgeBudget::new(3);
        budget.record_progress(10);
        assert_eq!(
            budget.try_nudge(),
            NudgeDecision::Nudge { attempt: 1, of: 3 }
        );
        assert_eq!(
            budget.try_nudge(),
            NudgeDecision::Nudge { attempt: 2, of: 3 }
        );
        // The metric did not move, and going BACKWARDS is not an advance.
        assert!(!budget.record_progress(10));
        assert!(!budget.record_progress(4));
        assert_eq!(
            budget.spent(),
            2,
            "a stalled or rewound metric earns nothing"
        );
        assert_eq!(
            budget.try_nudge(),
            NudgeDecision::Nudge { attempt: 3, of: 3 }
        );
        assert_eq!(
            budget.try_nudge(),
            NudgeDecision::Exhausted {
                without_progress: 3
            }
        );
        assert!(budget.is_exhausted());
        assert_eq!(budget.remaining(), 0);
        assert!(
            budget.try_nudge().describe().contains("needs a human"),
            "exhaustion is an escalation, not a silent stop"
        );
    }

    #[test]
    fn the_first_reading_is_a_baseline_not_an_advance() {
        let mut budget = NudgeBudget::new(2);
        assert!(
            !budget.record_progress(99),
            "announcing a reset the build did not earn is a false positive too"
        );
        assert_eq!(budget.best(), Some(99));
    }

    #[test]
    fn a_zero_budget_never_nudges() {
        let mut budget = NudgeBudget::new(0);
        assert_eq!(
            budget.try_nudge(),
            NudgeDecision::Exhausted {
                without_progress: 0
            },
            "a read-only monitor is a legitimate configuration"
        );
    }

    // ── Signal latches ───────────────────────────────────────────────────────

    /// DEFECT 2, reproduced: one shared variable, two signals, forever.
    #[test]
    fn two_signals_never_overwrite_each_other() {
        let mut latches = SignalLatches::new();
        assert!(latches.fire_failure("gate 4 is RED."));
        assert!(latches.fire_state(MonitorState::Stalled, "turn ended, 12 rows left"));

        // The old watcher wrote both into one variable, so this second poll,
        // with NOTHING changed, re-fired the failure. And the next. And so on.
        assert!(
            !latches.fire_failure("gate 4 is RED."),
            "the unchanged failure re-fired because the idle branch had \
             overwritten the shared variable"
        );
        assert!(!latches.fire_state(MonitorState::Stalled, "turn ended, 12 rows left"));

        // Genuinely new content of the same kind still gets through.
        assert!(latches.fire_failure("gate 7 is RED."));
        assert!(latches.fire_state(MonitorState::Stalled, "turn ended, 9 rows left"));
    }

    /// DEFECT 3: the pane KEEPS its scrollback, so the old failure text is
    /// still on screen after the session resumes.
    #[test]
    fn the_failure_latch_is_never_cleared_on_resume() {
        let mut latches = SignalLatches::new();
        assert!(latches.fire_failure("gate 4 is RED."));
        assert!(latches.fire_state(MonitorState::Blocked, "nothing runnable"));

        latches.note_working();

        assert!(
            !latches.fire_failure("gate 4 is RED."),
            "clearing the failure latch on resume made frozen scrollback re-fire"
        );
        assert_eq!(latches.latched_failure(), Some("gate 4 is RED."));
        assert!(
            latches.fire_state(MonitorState::Blocked, "nothing runnable"),
            "a stop AFTER a resume is a new event and must be reported"
        );
    }

    #[test]
    fn working_reports_nothing_and_is_what_clears_the_stop_latches() {
        let mut latches = SignalLatches::new();
        assert!(latches.fire_state(MonitorState::Question, "A or B?"));
        assert!(
            !latches.fire_state(MonitorState::Working, "busy"),
            "silence is the correct output for a busy session"
        );
        assert_eq!(latches.latched(MonitorState::Question), None);
        assert!(latches.fire_state(MonitorState::Question, "A or B?"));
    }

    // ── Naming ───────────────────────────────────────────────────────────────

    #[test]
    fn a_monitor_and_a_stream_of_one_target_never_collide() {
        for label in [
            "oracle-OmegaOS",
            "matrix:MAC-STREAM",
            "moonbasecapital:MoonBaseCapital-claude",
            "a.b.c",
            "Camélia build",
            &"x".repeat(60),
        ] {
            let t = stream::parse_target(label);
            let mon = monitor_viewer_name(&t);
            assert_ne!(mon, stream::viewer_name(&t), "{label}");
            assert!(
                mon.starts_with(MONITOR_PREFIX),
                "monitors are recognizable: {mon}"
            );
            assert!(
                mon.len() <= MAX_SESSION_NAME_LEN,
                "a name over the cap is a name rmux truncates again: {mon} ({})",
                mon.len()
            );
            assert!(
                !mon.contains('.') && !mon.contains(':') && !mon.contains(' '),
                "rmux would REWRITE this name and we could not find it again: {mon}"
            );
        }
        assert_eq!(
            monitor_viewer_name(&stream::parse_target("oracle-OmegaOS")),
            "monitor-oracle-OmegaOS"
        );
        assert_eq!(
            monitor_viewer_name(&stream::parse_target("matrix:MAC-STREAM")),
            "monitor-matrix-MAC-STREAM"
        );
    }

    #[test]
    fn monitor_names_inherit_stream_injectivity_over_a_hostile_population() {
        let targets = [
            "a",
            "a-b",
            "a.b",
            "a_b",
            "host:a",
            "host:a-b",
            "host-x:a",
            "host:a.b",
            "audit-stream-collision-probe-session-name-A",
            "audit-stream-collision-probe-session-name-B",
            &"x".repeat(60),
            &format!("{}A", "x".repeat(59)),
            &format!("{}B", "x".repeat(59)),
        ];
        let mut seen: Vec<(String, String)> = Vec::new();
        for label in targets {
            let name = monitor_viewer_name(&stream::parse_target(label));
            assert!(
                name.len() <= MAX_SESSION_NAME_LEN,
                "{name} is longer than rmux will keep ({})",
                name.len()
            );
            if let Some((other, _)) = seen.iter().find(|(_, n)| n == &name) {
                panic!("{other} and {label} collide on {name}");
            }
            seen.push((label.to_string(), name));
        }
    }

    /// The fingerprint length is pinned locally because stream.rs keeps it
    /// private. If that ever drifts, the head/tail split above would cut into
    /// the hash and monitor names would stop being injective, so the drift has
    /// to fail the build here instead.
    #[test]
    fn fingerprint_shape_matches_stream() {
        let t = stream::parse_target("audit-stream-collision-probe-session-name-A");
        let fp = stream::fingerprinted_viewer_name(&t);
        let stem = fp.strip_prefix(STREAM_PREFIX).expect("stream prefix");
        let tail = &stem[stem.len() - FINGERPRINT_LEN - 1..];
        assert!(
            tail.starts_with('-'),
            "the tail is `-<fingerprint>`: {tail:?}"
        );
        assert!(
            tail[1..]
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            "the fingerprint is base-36: {tail:?}"
        );
    }

    // ── The shell seam ───────────────────────────────────────────────────────

    #[test]
    fn the_monitor_script_is_the_documented_path() {
        let p = monitor_script_path();
        assert!(p.ends_with("bin/omega-monitor.sh"), "{}", p.display());
        assert!(
            !p.ends_with("bin/omega-stream.sh"),
            "the monitor loop is its own script"
        );
    }

    #[test]
    fn a_monitor_command_round_trips_back_to_its_target() {
        for label in ["oracle-OmegaOS", "matrix:MAC-STREAM"] {
            let t = stream::parse_target(label);
            let cmd = monitor_command(&t, 60, 120, "make count", "make progress", 30);
            assert_eq!(
                parse_monitor_command(&cmd),
                Some(t),
                "the seam contract is <script> <host> <session> …: {cmd}"
            );
        }
    }

    #[test]
    fn probes_reach_the_loop_as_single_arguments() {
        let t = stream::parse_target("oracle-OmegaOS");
        let cmd = monitor_command(&t, 60, 120, "jq '.left' plan.json", "", 30);
        assert!(
            cmd.contains(r"'jq '\''.left'\'' plan.json'"),
            "a probe with spaces and quotes must arrive as ONE argument: {cmd}"
        );
        assert!(
            cmd.ends_with(" '' 30"),
            "an absent probe is still a positional: {cmd}"
        );
        assert!(
            cmd.starts_with('/'),
            "the script path must be expanded: {cmd}"
        );

        let argv = monitor_viewer_argv("monitor-oracle-OmegaOS", &t, 60, 120, "", "", 30);
        assert_eq!(
            argv[..4],
            ["new-session", "-d", "-s", "monitor-oracle-OmegaOS"]
        );
        assert_eq!(argv.len(), 5, "the loop is ONE command argument");
    }

    #[test]
    fn parse_monitor_command_refuses_anything_that_is_not_the_loop() {
        for cmd in [
            "",
            "bash -c 'claude --dangerously-skip-permissions'",
            "/home/vibe/.omega/bin/omega-stream.sh local a 3 120",
            "/home/vibe/.omega/bin/omega-monitor.sh",
            "/home/vibe/.omega/bin/omega-monitor.sh local",
            "/home/vibe/.omega/bin/omega-monitor.sh local a;id 60 120 '' '' 30",
        ] {
            assert_eq!(
                parse_monitor_command(cmd),
                None,
                "an unreadable session must be treated as occupied: {cmd:?}"
            );
        }
    }

    #[test]
    fn the_monitor_index_reports_what_each_monitor_actually_watches() {
        let listing = concat!(
            "oracle-OmegaOS|bash -c %27claude --dangerously-skip-permissions%27\n",
            "stream-matrix-MAC-STREAM|/home/vibe/.omega/bin/omega-stream.sh matrix MAC-STREAM 3 120\n",
            "monitor-matrix-MAC-STREAM|/home/vibe/.omega/bin/omega-monitor.sh matrix MAC-STREAM 60 120 '' '' 30\n",
            "monitor-broken|bash\n",
            "\n",
            "not-a-monitor|/home/vibe/.omega/bin/omega-monitor.sh local hidden 60 120 '' '' 30\n",
        );
        assert_eq!(
            parse_monitor_index(listing),
            vec![(
                "monitor-matrix-MAC-STREAM".to_string(),
                stream::parse_target("matrix:MAC-STREAM")
            )],
            "a stream viewer is not a monitor, and neither is an unreadable session"
        );
    }

    #[test]
    fn the_remote_capture_keeps_home_unexpanded_for_the_remote_shell() {
        let argv = capture_pane_argv(&stream::parse_target("matrix:MAC-STREAM"), 120);
        let remote = argv.last().expect("remote command is the last argument");
        assert!(
            remote.contains("$HOME/.local/bin/rmux"),
            "expanding $HOME locally once told a Linux box to read a macOS path: {remote}"
        );
        if let Some(home) = dirs::home_dir() {
            assert!(
                !remote.contains(&home.display().to_string()),
                "the LOCAL home leaked into the remote command: {remote}"
            );
        }
        assert!(remote.contains("capture-pane -p -t 'MAC-STREAM' -S -120"));
        assert_eq!(argv[0], "ssh");
        assert!(
            argv.iter().any(|a| a == "BatchMode=yes"),
            "captures never prompt"
        );
        assert!(
            argv.iter().any(|a| a == "matrix"),
            "the ALIAS is passed to ssh"
        );
    }

    #[test]
    fn the_local_capture_uses_the_absolute_rmux() {
        let argv = capture_pane_argv(&stream::parse_target("oracle-OmegaOS"), 90);
        assert!(
            argv[0].ends_with("rmux"),
            "rmux is not on the non-interactive PATH: {argv:?}"
        );
        assert_eq!(
            argv[1..],
            ["capture-pane", "-p", "-t", "oracle-OmegaOS", "-S", "-90"]
        );
    }

    #[test]
    fn enter_is_always_a_separate_send_keys_call() {
        let t = stream::parse_target("oracle-OmegaOS");
        let keys = send_keys_argv(&t, "continue with the plan");
        assert!(
            !keys.iter().any(|a| a == "Enter"),
            "appending Enter to the text types the word instead of submitting: {keys:?}"
        );
        assert!(
            keys.iter().any(|a| a == "-l"),
            "the text is sent literally: {keys:?}"
        );
        assert_eq!(keys.last().unwrap(), "continue with the plan");

        let enter = send_enter_argv(&t);
        assert_eq!(enter.last().unwrap(), "Enter");
        assert!(
            !enter.iter().any(|a| a == "-l"),
            "Enter is a KEY, not literal text"
        );
    }

    #[test]
    fn a_remote_nudge_is_quoted_for_the_remote_shell() {
        let t = stream::parse_target("matrix:MAC-STREAM");
        let remote = send_keys_argv(&t, "it's green; run `make`").pop().unwrap();
        assert!(
            remote.contains(r"'it'\''s green; run `make`'"),
            "an unquoted nudge would run on the remote box: {remote}"
        );
        assert!(remote.contains("$HOME/.local/bin/rmux send-keys"));
        let enter = send_enter_argv(&t).pop().unwrap();
        assert!(enter.ends_with("-t 'MAC-STREAM' Enter"), "{enter}");
    }

    #[test]
    fn shell_single_quote_survives_a_quote() {
        assert_eq!(shell_single_quote("plain"), "'plain'");
        assert_eq!(shell_single_quote("it's"), r"'it'\''s'");
        assert_eq!(shell_single_quote(""), "''");
    }

    // ── Wire words ───────────────────────────────────────────────────────────

    #[test]
    fn the_state_word_round_trips_for_the_shell_half() {
        for state in [
            MonitorState::Question,
            MonitorState::Stalled,
            MonitorState::Blocked,
            MonitorState::Working,
        ] {
            assert_eq!(MonitorState::parse(state.as_str()), Some(state));
            assert_eq!(
                MonitorState::parse(&state.as_str().to_lowercase()),
                Some(state)
            );
        }
        assert_eq!(MonitorState::parse("idle"), None);
    }

    #[test]
    fn only_a_stall_is_answered_mechanically() {
        assert!(MonitorState::Stalled.should_nudge());
        for state in [
            MonitorState::Question,
            MonitorState::Blocked,
            MonitorState::Working,
        ] {
            assert!(!state.should_nudge(), "{state:?} must never be nudged");
        }
        assert!(MonitorState::Question.needs_human());
        assert!(MonitorState::Blocked.needs_human());
    }

    #[tokio::test]
    async fn a_probe_is_bounded_and_an_absent_one_is_not_zero() {
        assert_eq!(run_probe("", std::time::Duration::from_secs(5)).await, None);
        assert_eq!(
            WorkSignal::from_probe_output(
                &run_probe("echo 4", std::time::Duration::from_secs(5))
                    .await
                    .unwrap_or_default()
            ),
            WorkSignal::Available
        );
        // A probe that never answers must not park the poll forever.
        let started = std::time::Instant::now();
        let out = run_probe("sleep 30", std::time::Duration::from_millis(300)).await;
        assert!(out.is_none(), "a hung probe returns nothing, not a count");
        assert!(
            started.elapsed() < std::time::Duration::from_secs(5),
            "the capture's own wall clock must fire: {:?}",
            started.elapsed()
        );
        assert_eq!(
            WorkSignal::from_probe_output(&out.unwrap_or_default()),
            WorkSignal::Unknown,
            "a probe we could not read is not evidence that work ran out"
        );
    }
}
