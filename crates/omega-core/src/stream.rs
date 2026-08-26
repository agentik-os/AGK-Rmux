//! `omega stream` — mirror a live rmux session, local or over ssh, into a
//! local viewer session.
//!
//! # The mechanism
//!
//! A viewer session runs exactly one thing: the shell loop
//! `~/.omega/bin/omega-stream.sh <target> <session> <interval> <lines>`, which
//! SNAPSHOTS the rendered screen of the source session on a timer
//! (`rmux capture-pane -p -t <session> -S -<lines>`, clear, print, sleep).
//! This module owns the Rust half: parsing the target coordinate, enumerating
//! ssh aliases, naming the viewer, and probing a box before anything is
//! created.
//!
//! # Five constraints, each one paid for by a real failure
//!
//! These are requirements, not preferences. They are also written into the
//! doctrine as R-STREAM so they are never re-derived.
//!
//! 1. NEVER replay raw bytes. `rmux pipe-pane -O` piped into `tail -f` renders
//!    as garbage: a full-screen TUI emits cursor moves and partial redraws
//!    that only mean something against a live screen buffer. `capture-pane`
//!    returns the RENDERED text, which is what a human actually reads.
//! 2. PULL, never push. The viewer box reaches out and fetches; the source box
//!    ships nothing. A push-based shipper died once and the mirror FROZE while
//!    the source kept growing, and a frozen mirror is indistinguishable from a
//!    quiet one. Pulling puts the liveness of the stream on the box that can
//!    notice it stopped.
//! 3. The puller must be a CHILD OF THE VIEWER SESSION. `nohup setsid ... &`
//!    inside an ssh command does not survive the ssh exiting. This design
//!    satisfies that by construction: the loop IS the session's command. The
//!    corollary binds the shell half: the loop must never exit on error, or
//!    the session dies and the operator sees nothing at all. Errors are
//!    rendered, never fatal.
//! 4. rmux is not tmux, and it is not on the non-interactive PATH. Use the
//!    absolute `$HOME/.local/bin/rmux` (see [`rmux_bin`]), and test `$RMUX`:
//!    rmux exports `RMUX` and `RMUX_PANE`, never `$TMUX`.
//! 5. QUOTING KILLS THIS SILENTLY. A `$VAR` inside a double-quoted remote ssh
//!    command expands LOCALLY. The remote rmux path must reach the REMOTE
//!    shell unexpanded, which is why [`probe_argv`] hands ssh the literal
//!    `$HOME/...` string and never a locally interpolated path. Getting this
//!    wrong once told a Linux box to read /Users/hacker/... .
//!
//! # Coordinates
//!
//! Hosts are ssh CONFIG ALIASES. We pass the alias to ssh and let ssh resolve
//! HostName / Port / User / IdentityFile, so a coordinate is never hardcoded
//! here (one host on this tailnet answers on port 42820, not 22, and only ssh
//! knows that). `~/.ssh/config` is parsed for exactly two reasons: to
//! enumerate aliases for `omega stream list`, and to give a clean
//! "unknown host" error instead of a raw ssh failure.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{bail, Context, Result};

use crate::session::{sanitize_session_name, MAX_SESSION_NAME_LEN};

/// Default seconds between screen snapshots.
pub const DEFAULT_INTERVAL_SECS: u32 = 3;

/// Default scrollback lines captured per snapshot.
pub const DEFAULT_LINES: u32 = 120;

/// Seconds handed to ssh's own `ConnectTimeout`. Lower than [`PROBE_TIMEOUT`]
/// so ssh usually fails first with a precise message, and our wall clock is
/// only the backstop.
pub const SSH_CONNECT_TIMEOUT_SECS: u32 = 8;

/// Hard wall clock on any single probe. A down host must never hang
/// `omega stream list`.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(12);

/// Where a streamed session lives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamTarget {
    /// A session on this box.
    Local { session: String },
    /// A session on an ssh host alias (resolved by ssh, not by us).
    Remote { host: String, session: String },
}

impl StreamTarget {
    /// The source session name on its box.
    pub fn session(&self) -> &str {
        match self {
            StreamTarget::Local { session } => session,
            StreamTarget::Remote { session, .. } => session,
        }
    }

    /// The ssh alias, or `None` when the target is this box.
    pub fn host(&self) -> Option<&str> {
        match self {
            StreamTarget::Local { .. } => None,
            StreamTarget::Remote { host, .. } => Some(host),
        }
    }

    /// First argument of the shell loop: an ssh alias, or the literal `local`.
    pub fn host_arg(&self) -> &str {
        self.host().unwrap_or("local")
    }

    /// How the operator wrote it: `session` or `host:session`.
    pub fn label(&self) -> String {
        match self {
            StreamTarget::Local { session } => session.clone(),
            StreamTarget::Remote { host, session } => format!("{host}:{session}"),
        }
    }
}

/// Split `host:session` from a bare `session`.
///
/// A bare name means local. The FIRST `:` is the separator, unambiguously: an
/// rmux session name can contain `-`, but never `:` (rmux rewrites it), so
/// there is no second reading to disambiguate.
///
/// Degenerate forms (`:name`, `name:`) have no valid remote reading, so the
/// non-empty side is taken as a local session name and preflight reports it
/// honestly as "not found" rather than us inventing a host.
pub fn parse_target(arg: &str) -> StreamTarget {
    let arg = arg.trim();
    match arg.split_once(':') {
        Some((host, session)) if !host.is_empty() && !session.is_empty() => StreamTarget::Remote {
            host: host.to_string(),
            session: session.to_string(),
        },
        Some((host, session)) => StreamTarget::Local {
            session: if host.is_empty() {
                session.to_string()
            } else {
                host.to_string()
            },
        },
        None => StreamTarget::Local {
            session: arg.to_string(),
        },
    }
}

/// What `~/.ssh/config` told us.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SshConfig {
    /// Concrete `Host` aliases, in file order, deduped. Wildcard patterns are
    /// dropped: `Host *` is a defaults block, not a box you can reach.
    pub hosts: Vec<String>,
    /// The file pulls in other files we deliberately do not expand, so
    /// `hosts` is a LOWER BOUND. Callers must not hard-reject an unknown
    /// alias when this is set, or a perfectly valid included host is blocked.
    pub has_include: bool,
}

/// Parse ssh_config text into concrete `Host` aliases.
///
/// Tolerant by design: comments, blank lines, leading whitespace, any case of
/// the `Host` keyword, several aliases on one line (`Host a b c`), and the
/// `Host=alias` form. Wildcard patterns (`*`, `?`) and negations (`!foo`) are
/// skipped, since neither names a reachable box.
pub fn parse_ssh_config(text: &str) -> SshConfig {
    let mut out = SshConfig::default();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // `Key=value` is legal ssh_config; normalize it to whitespace so one
        // tokenizer handles both spellings.
        let normalized = line.replace('=', " ");
        let mut tokens = normalized.split_whitespace();
        let Some(keyword) = tokens.next() else {
            continue;
        };
        if keyword.eq_ignore_ascii_case("include") {
            out.has_include = true;
            continue;
        }
        if !keyword.eq_ignore_ascii_case("host") {
            continue;
        }
        for alias in tokens {
            if alias.starts_with('#') {
                break; // trailing comment
            }
            if alias.contains('*') || alias.contains('?') || alias.starts_with('!') {
                continue;
            }
            if !out.hosts.iter().any(|h| h == alias) {
                out.hosts.push(alias.to_string());
            }
        }
    }
    out
}

/// Path of the operator's ssh config.
pub fn ssh_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/"))
        .join(".ssh")
        .join("config")
}

/// Read `~/.ssh/config` and return its concrete `Host` aliases. Empty when the
/// file is missing or unreadable, which callers treat as "cannot enumerate",
/// never as "no hosts exist".
pub fn ssh_hosts() -> Vec<String> {
    read_ssh_config().hosts
}

/// Read and parse `~/.ssh/config`, keeping the `Include` flag.
pub fn read_ssh_config() -> SshConfig {
    match std::fs::read_to_string(ssh_config_path()) {
        Ok(text) => parse_ssh_config(&text),
        Err(_) => SshConfig::default(),
    }
}

/// Is this coordinate safe to interpolate into the viewer's shell command?
///
/// Everything handed to [`viewer_command`] ends up on a shell command line, so
/// anything outside the slug alphabet is REFUSED rather than quoted. An rmux
/// session name and an ssh alias are both slugs; a sentence is not either of
/// them.
pub fn is_safe_coordinate(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// Prefix every viewer name carries, and the marker `omega stream list` reads.
const VIEWER_PREFIX: &str = "stream-";

/// Length of the disambiguating fingerprint. Base-36 over 6 characters is ~2.2
/// billion values, against a population of at most a few dozen live viewers.
const FINGERPRINT_LEN: usize = 6;

/// The name a target gets when nothing is ambiguous: exactly the documented
/// `stream-<session>` / `stream-<host>-<session>` contract.
///
/// Both `:` and `.` are neutralized BEFORE sanitizing, because rmux does not
/// reject them, it silently REWRITES them to `_` (verified: a session created
/// as `a.b` comes back as `a_b`). A name we compute but rmux does not key on
/// would break `has-session` idempotency and attach, so the name must already
/// be one rmux keeps verbatim.
fn plain_viewer_name(target: &StreamTarget) -> String {
    let raw = match target {
        StreamTarget::Local { session } => format!("{VIEWER_PREFIX}{session}"),
        StreamTarget::Remote { host, session } => format!("{VIEWER_PREFIX}{host}-{session}"),
    };
    // Space, not '-': sanitize collapses any run of disallowed characters into
    // a single '-', so this avoids "a--b" on an already-hyphenated coordinate.
    let neutral = raw.replace([':', '.'], " ");
    sanitize_session_name(&neutral)
}

/// Is the plain name a FAITHFUL encoding of this coordinate — one that no other
/// coordinate could have produced?
///
/// The viewer name is the identity of the mirror, so it has to be reversible.
/// Two mechanisms used to destroy that, and both are decidable from the target
/// alone:
///
/// * FOLDING. `sanitize_session_name` collapses anything outside the slug
///   alphabet to `-`, so `box.local` and `box-local` land on one name.
/// * TRUNCATION at [`MAX_SESSION_NAME_LEN`]. Reproduced live: two 43-character
///   sessions differing only in the last character produced ONE viewer, and the
///   operator was told they were watching B while looking at A.
///
/// Plus one that is decidable for the remote form: a host containing `-` makes
/// the host/session boundary unreadable (`stream-a-b-c` is `a` + `b-c` and
/// `a-b` + `c` at once).
///
/// Comparing against the un-sanitized composition catches folding and
/// truncation in one test: if sanitizing changed anything at all, the plain
/// name is not a faithful encoding and the fingerprint takes over.
fn encodes_faithfully(target: &StreamTarget, plain: &str) -> bool {
    let verbatim = match target {
        StreamTarget::Local { session } => format!("{VIEWER_PREFIX}{session}"),
        StreamTarget::Remote { host, session } => {
            // A dashed host cannot be told from a dashed session afterwards.
            if host.contains('-') {
                return false;
            }
            format!("{VIEWER_PREFIX}{host}-{session}")
        }
    };
    plain == verbatim
}

/// A short, stable, machine-independent fingerprint of a coordinate.
///
/// FNV-1a rather than [`std::collections::hash_map::DefaultHasher`] on purpose:
/// the std hasher's output is explicitly not guaranteed stable across Rust
/// releases, and a viewer name that changes under the operator when they
/// upgrade the toolchain is a name that finds nothing.
fn fingerprint(label: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in label.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    const ALPHABET: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut out = [b'0'; FINGERPRINT_LEN];
    for slot in out.iter_mut().rev() {
        *slot = ALPHABET[(hash % 36) as usize];
        hash /= 36;
    }
    String::from_utf8(out.to_vec()).expect("the alphabet is ascii")
}

/// The unambiguous fallback name: the plain name, cut to make room, plus a
/// fingerprint of [`StreamTarget::label`].
///
/// `label()` is the coordinate exactly as the operator wrote it, and it is
/// injective over valid targets (a session name may not contain `:`, so
/// `host:session` can never be a local session name). Two different sources
/// therefore never share a fingerprinted name.
pub fn fingerprinted_viewer_name(target: &StreamTarget) -> String {
    let plain = plain_viewer_name(target);
    let head_budget = MAX_SESSION_NAME_LEN - FINGERPRINT_LEN - 1;
    // `plain` is ASCII by construction (sanitize guarantees it), so a byte cut
    // is a character cut.
    let head = plain
        .get(..head_budget)
        .unwrap_or(&plain)
        .trim_end_matches(['-', '.']);
    format!("{head}-{}", fingerprint(&target.label()))
}

/// The viewer session name for a target.
///
/// `local` gives `stream-<session>`, a remote gives `stream-<host>-<session>`,
/// and anything the plain form cannot encode faithfully (see
/// [`encodes_faithfully`]) gets `<cut>-<fingerprint>` instead. The documented
/// forms are unchanged for every coordinate that is already a clean, in-budget
/// slug, which is the overwhelming majority and every live viewer on this box.
///
/// ONE structural overlap survives this function, deliberately: a local session
/// literally named `<alias>-<session>` composes to the same plain name as the
/// remote coordinate `<alias>:<session>`. Removing it by name alone would mean
/// fingerprinting one WHOLE family (every local, or every remote), renaming
/// viewers that are correct today and contradicting the `stream-<host>-<session>`
/// contract in R-STREAM. It is closed one layer up instead, by
/// [`choose_viewer`], which reads what a live viewer is ACTUALLY streaming and
/// moves to the fingerprinted name on a mismatch — so the two sources still get
/// two viewers, and neither is ever shown under the other's label.
pub fn viewer_name(target: &StreamTarget) -> String {
    let plain = plain_viewer_name(target);
    if encodes_faithfully(target, &plain) {
        plain
    } else {
        fingerprinted_viewer_name(target)
    }
}

/// What a live session standing on a candidate viewer name turns out to be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewerSlot {
    /// No session under that name.
    Free,
    /// A viewer, and this is the coordinate it is really pulling — read from
    /// the command rmux started it with, never inferred from its name.
    Mirroring(StreamTarget),
    /// A session exists under that name and it is not a viewer we can read
    /// (an operator's own session, or a viewer started by an older build).
    Taken,
}

impl ViewerSlot {
    /// Is this slot already the mirror we were about to create?
    pub fn mirrors(&self, target: &StreamTarget) -> bool {
        matches!(self, ViewerSlot::Mirroring(t) if t == target)
    }

    /// What it mirrors, when it mirrors anything.
    pub fn target(&self) -> Option<&StreamTarget> {
        match self {
            ViewerSlot::Mirroring(t) => Some(t),
            _ => None,
        }
    }
}

/// Which viewer session `omega stream <target>` should end up attached to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewerChoice {
    /// A viewer for exactly this target is already running.
    Reuse(String),
    /// Nothing is streaming this target yet; create it under this name.
    Create(String),
    /// Both candidate names are held by something else. Refuse rather than
    /// attach the operator to a mirror of a different session.
    Conflict {
        name: String,
        held_by: Option<StreamTarget>,
    },
}

impl ViewerChoice {
    /// The viewer session name this choice lands on, if any.
    pub fn name(&self) -> Option<&str> {
        match self {
            ViewerChoice::Reuse(n) | ViewerChoice::Create(n) => Some(n),
            ViewerChoice::Conflict { .. } => None,
        }
    }
}

/// Decide between reusing, creating, and refusing, given what each candidate
/// name currently holds. Pure, so the whole decision table is unit-testable
/// without an rmux server.
///
/// `preferred` is the state of [`viewer_name`]; `fallback` is the state of
/// [`fingerprinted_viewer_name`] and is only consulted when the preferred name
/// is held by something that is not this target.
pub fn choose_viewer(
    target: &StreamTarget,
    preferred: &ViewerSlot,
    fallback: &ViewerSlot,
) -> ViewerChoice {
    let name = viewer_name(target);
    match preferred {
        ViewerSlot::Free => return ViewerChoice::Create(name),
        slot if slot.mirrors(target) => return ViewerChoice::Reuse(name),
        _ => {}
    }

    // The preferred name is taken by a DIFFERENT stream. This is the live
    // failure F-002 reproduced: the old code printed "Already streaming <B>"
    // and attached the operator to A. Move to a name only this coordinate can
    // produce.
    let alt = fingerprinted_viewer_name(target);
    if alt == name {
        // Already the fingerprinted form, so there is no further escape: only a
        // fingerprint collision or a foreign session squatting that exact name
        // gets here. Say so instead of pretending.
        return ViewerChoice::Conflict {
            name,
            held_by: preferred.target().cloned(),
        };
    }
    match fallback {
        ViewerSlot::Free => ViewerChoice::Create(alt),
        slot if slot.mirrors(target) => ViewerChoice::Reuse(alt),
        other => ViewerChoice::Conflict {
            name: alt,
            held_by: other.target().cloned(),
        },
    }
}

/// Recover the coordinate a viewer is streaming from the command it was started
/// with (`<script> <target> <session> <interval> <lines>` — the seam contract).
///
/// Returns `None` for anything that is not that command, which is the safe
/// direction: an unreadable session is treated as occupied, never as a mirror
/// of what its name suggests.
pub fn parse_viewer_command(cmd: &str) -> Option<StreamTarget> {
    let mut tokens = cmd.split_whitespace();
    let script = tokens.next()?;
    if !script.ends_with("omega-stream.sh") {
        return None;
    }
    let host_arg = tokens.next()?;
    let session = tokens.next()?;
    if !is_safe_coordinate(session) {
        return None;
    }
    if host_arg == "local" {
        Some(StreamTarget::Local {
            session: session.to_string(),
        })
    } else if is_safe_coordinate(host_arg) {
        Some(StreamTarget::Remote {
            host: host_arg.to_string(),
            session: session.to_string(),
        })
    } else {
        None
    }
}

/// Parse `rmux list-panes -a -F '#S|#{pane_start_command}'` into the live
/// viewers and what each one is really streaming.
///
/// This is the honest "already mirrored" index. Matching on the NAME instead
/// (what `omega stream list` did) marks two colliding sources as mirrored while
/// only one viewer exists.
pub fn parse_viewer_index(listing: &str) -> Vec<(String, StreamTarget)> {
    let mut out = Vec::new();
    for line in listing.lines() {
        let Some((name, cmd)) = line.split_once('|') else {
            continue;
        };
        let name = name.trim();
        if !name.starts_with(VIEWER_PREFIX) {
            continue;
        }
        if let Some(target) = parse_viewer_command(cmd.trim()) {
            if !out.iter().any(|(n, _): &(String, StreamTarget)| n == name) {
                out.push((name.to_string(), target));
            }
        }
    }
    out
}

/// Absolute rmux binary (lesson 4: rmux is NOT on the non-interactive PATH).
/// Falls back to a bare `rmux` only when the canonical install path is absent.
pub fn rmux_bin() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        let installed = home.join(".local").join("bin").join("rmux");
        if installed.is_file() {
            return installed;
        }
    }
    PathBuf::from("rmux")
}

/// The shell loop the viewer session runs. Installed by `install.sh` from
/// `scripts/omega-stream.sh`.
pub fn stream_script_path() -> PathBuf {
    crate::config::omega_dir()
        .join("bin")
        .join("omega-stream.sh")
}

/// The exact command string handed to `rmux new-session`.
///
/// `$HOME` is expanded HERE (Rust spawns rmux directly, with no shell of its
/// own), so the viewer gets an absolute script path.
pub fn viewer_command(target: &StreamTarget, interval: u32, lines: u32) -> String {
    format!(
        "{} {} {} {} {}",
        stream_script_path().display(),
        target.host_arg(),
        target.session(),
        interval,
        lines
    )
}

/// Full argv of the session-creating call, split out so a test can pin the
/// seam without spawning rmux.
///
/// Equivalent to:
/// `rmux new-session -d -s <viewer> "<script> <target> <session> <interval> <lines>"`
pub fn viewer_argv(viewer: &str, target: &StreamTarget, interval: u32, lines: u32) -> Vec<String> {
    vec![
        "new-session".to_string(),
        "-d".to_string(),
        "-s".to_string(),
        viewer.to_string(),
        viewer_command(target, interval, lines),
    ]
}

/// Full argv of the session-listing probe for a box (`None` = this box).
///
/// LESSON 5 LIVES HERE. The remote element is the literal string
/// `$HOME/.local/bin/rmux list-sessions -F '#S'`. Rust performs no expansion,
/// ssh concatenates its command arguments and hands them to the REMOTE login
/// shell, so `$HOME` resolves on the remote box, which is the only correct
/// answer. The `#S` MUST stay quoted: unquoted, a remote shell reads `#` as
/// the start of a comment and the format silently vanishes.
pub fn probe_argv(host: Option<&str>) -> Vec<String> {
    match host {
        None => vec![
            rmux_bin().display().to_string(),
            "list-sessions".to_string(),
            "-F".to_string(),
            "#S".to_string(),
        ],
        Some(host) => vec![
            "ssh".to_string(),
            "-o".to_string(),
            "BatchMode=yes".to_string(),
            "-o".to_string(),
            format!("ConnectTimeout={SSH_CONNECT_TIMEOUT_SECS}"),
            host.to_string(),
            "$HOME/.local/bin/rmux list-sessions -F '#S'".to_string(),
        ],
    }
}

/// What a probe found. Typed, never stringly, because the caller's next move
/// differs per kind: retry, name the alias list, or list what IS on the box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeOutcome {
    /// The box answered: these rmux sessions exist on it (possibly none).
    Sessions(Vec<String>),
    /// SSH ITSELF failed. THE DISCRIMINATOR: ssh exits 255 for its own
    /// failures (host down, DNS, auth, wrong port), and passes the remote
    /// command's status through for anything else.
    Unreachable { detail: String },
    /// The command RAN on the box and failed (any non-zero that is not the
    /// ssh-255 signal): no rmux server, rmux not installed, nothing to list.
    RmuxFailed { code: Option<i32>, detail: String },
    /// The bounded wall clock elapsed. A hung box never blocks the caller.
    TimedOut { secs: u64 },
    /// The probe could not even be launched locally (no ssh, no rmux binary).
    SpawnFailed { detail: String },
}

impl ProbeOutcome {
    /// The session list, when the box answered.
    pub fn sessions(&self) -> Option<&[String]> {
        match self {
            ProbeOutcome::Sessions(s) => Some(s),
            _ => None,
        }
    }

    /// Did the box answer at all?
    pub fn reachable(&self) -> bool {
        matches!(self, ProbeOutcome::Sessions(_))
    }

    /// One line an operator can act on.
    pub fn describe(&self) -> String {
        match self {
            ProbeOutcome::Sessions(s) => format!("{} session(s)", s.len()),
            ProbeOutcome::Unreachable { detail } => format!("unreachable over ssh: {detail}"),
            ProbeOutcome::RmuxFailed { code, detail } => match code {
                Some(c) => format!("rmux failed on the box (exit {c}): {detail}"),
                None => format!("rmux failed on the box (killed by signal): {detail}"),
            },
            ProbeOutcome::TimedOut { secs } => format!("no answer within {secs}s"),
            ProbeOutcome::SpawnFailed { detail } => format!("could not run the probe: {detail}"),
        }
    }
}

/// Split `rmux list-sessions -F '#S'` output into session names.
pub fn parse_session_list(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

/// Last meaningful stderr line, which is where ssh and rmux both put the
/// actual reason.
fn last_error_line(stderr: &str) -> String {
    stderr
        .lines()
        .map(str::trim)
        .rfind(|l| !l.is_empty())
        .unwrap_or("(no error output)")
        .to_string()
}

/// List the rmux sessions on a box, bounded by [`PROBE_TIMEOUT`].
///
/// `host = None` probes this box. Never returns an `Err`: every failure mode
/// is a typed [`ProbeOutcome`], because `omega stream list` must render a
/// dead host and keep going.
pub async fn probe_host(host: Option<&str>) -> ProbeOutcome {
    probe_host_bounded(host, PROBE_TIMEOUT).await
}

/// [`probe_host`] with an explicit wall clock. The bound is a parameter so a
/// test can prove that a black-holed box returns inside it without the suite
/// paying the production timeout.
pub async fn probe_host_bounded(host: Option<&str>, bound: Duration) -> ProbeOutcome {
    let argv = probe_argv(host);
    let (bin, args) = argv
        .split_first()
        .expect("probe_argv always yields a program plus arguments");

    let mut cmd = tokio::process::Command::new(bin);
    cmd.args(args)
        .stdin(std::process::Stdio::null())
        // A hung ssh must be reaped when we stop waiting, not leaked.
        .kill_on_drop(true);

    match tokio::time::timeout(bound, cmd.output()).await {
        Err(_) => ProbeOutcome::TimedOut {
            secs: bound.as_secs(),
        },
        Ok(Err(e)) => ProbeOutcome::SpawnFailed {
            detail: e.to_string(),
        },
        Ok(Ok(out)) => {
            if out.status.success() {
                return ProbeOutcome::Sessions(parse_session_list(&String::from_utf8_lossy(
                    &out.stdout,
                )));
            }
            let code = out.status.code();
            let detail = last_error_line(&String::from_utf8_lossy(&out.stderr));
            if host.is_some() && code == Some(255) {
                ProbeOutcome::Unreachable { detail }
            } else {
                ProbeOutcome::RmuxFailed { code, detail }
            }
        }
    }
}

/// List the rmux sessions on the target's box.
pub async fn probe_target(target: &StreamTarget) -> ProbeOutcome {
    probe_host(target.host()).await
}

/// Does a local session with this exact name exist?
///
/// The idempotency gate: two pullers on one viewer produced interleaved
/// garbage once, so a second one is never started.
pub async fn session_exists(name: &str) -> bool {
    let mut cmd = tokio::process::Command::new(rmux_bin());
    cmd.args(["has-session", "-t", name])
        .stdin(std::process::Stdio::null())
        .kill_on_drop(true);
    match tokio::time::timeout(PROBE_TIMEOUT, cmd.output()).await {
        Ok(Ok(out)) => out.status.success(),
        _ => false,
    }
}

/// The command rmux started a session with, or `None` when there is no such
/// session (or rmux cannot tell us).
///
/// `list-panes -F '#{pane_start_command}'` is the only thing that knows what a
/// viewer is REALLY pulling. Verified live:
/// `stream-zzv-src -> /home/vibe/.omega/bin/omega-stream.sh local zzv-src 2 60`.
async fn session_start_command(name: &str) -> Option<String> {
    let mut cmd = tokio::process::Command::new(rmux_bin());
    cmd.args(["list-panes", "-t", name, "-F", "#{pane_start_command}"])
        .stdin(std::process::Stdio::null())
        .kill_on_drop(true);
    let out = match tokio::time::timeout(PROBE_TIMEOUT, cmd.output()).await {
        Ok(Ok(out)) if out.status.success() => out,
        _ => return None,
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(str::to_string)
}

/// What is standing on this viewer name right now.
pub async fn viewer_slot(name: &str) -> ViewerSlot {
    if !session_exists(name).await {
        return ViewerSlot::Free;
    }
    match session_start_command(name).await.as_deref() {
        Some(cmd) => match parse_viewer_command(cmd) {
            Some(target) => ViewerSlot::Mirroring(target),
            None => ViewerSlot::Taken,
        },
        None => ViewerSlot::Taken,
    }
}

/// Pick the viewer session for a target, VERIFYING that anything we reuse is
/// really streaming it.
///
/// The fallback name is probed only when the preferred one is held by something
/// else, so the common path still costs a single `has-session`.
pub async fn resolve_viewer(target: &StreamTarget) -> ViewerChoice {
    let preferred = viewer_slot(&viewer_name(target)).await;
    let fallback = if matches!(preferred, ViewerSlot::Free) || preferred.mirrors(target) {
        // Unused by choose_viewer on these two branches; never probed.
        ViewerSlot::Free
    } else {
        viewer_slot(&fingerprinted_viewer_name(target)).await
    };
    choose_viewer(target, &preferred, &fallback)
}

/// Every live viewer on this box and the coordinate it is really streaming.
///
/// One rmux call for the whole index. An empty result means "nothing readable",
/// never "nothing mirrored", so callers must not treat it as proof.
pub async fn live_viewer_index() -> Vec<(String, StreamTarget)> {
    let mut cmd = tokio::process::Command::new(rmux_bin());
    cmd.args(["list-panes", "-a", "-F", "#S|#{pane_start_command}"])
        .stdin(std::process::Stdio::null())
        .kill_on_drop(true);
    match tokio::time::timeout(PROBE_TIMEOUT, cmd.output()).await {
        Ok(Ok(out)) if out.status.success() => {
            parse_viewer_index(&String::from_utf8_lossy(&out.stdout))
        }
        _ => Vec::new(),
    }
}

/// Create the detached viewer session that runs the pull loop.
///
/// Verifies afterwards that a session under exactly this name exists: rmux
/// rewrites characters it dislikes instead of refusing them, so "created" is
/// not proof that the name we will attach to is the name it stored.
pub async fn create_viewer(
    viewer: &str,
    target: &StreamTarget,
    interval: u32,
    lines: u32,
) -> Result<()> {
    let script = stream_script_path();
    if !script.is_file() {
        bail!(
            "the stream loop is not installed at {} — run install.sh (or `omega sync`) first; \
             creating a viewer without it would leave a session that dies instantly",
            script.display()
        );
    }
    let argv = viewer_argv(viewer, target, interval, lines);
    let out = tokio::process::Command::new(rmux_bin())
        .args(&argv)
        .stdin(std::process::Stdio::null())
        .output()
        .await
        .context("spawning rmux new-session")?;
    if !out.status.success() {
        bail!(
            "rmux new-session failed (exit {:?}): {}",
            out.status.code(),
            last_error_line(&String::from_utf8_lossy(&out.stderr))
        );
    }
    if !session_exists(viewer).await {
        bail!(
            "rmux reported success but no session named {viewer} exists — \
             the name was probably rewritten; nothing to attach to"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_target_splits_on_the_first_colon() {
        assert_eq!(
            parse_target("oracle-OmegaOS"),
            StreamTarget::Local {
                session: "oracle-OmegaOS".to_string()
            }
        );
        assert_eq!(
            parse_target("matrix:MAC-STREAM"),
            StreamTarget::Remote {
                host: "matrix".to_string(),
                session: "MAC-STREAM".to_string()
            }
        );
        // A session name may contain '-', so the separator must be ':' only.
        assert_eq!(
            parse_target("moonbasecapital:MoonBaseCapital-claude"),
            StreamTarget::Remote {
                host: "moonbasecapital".to_string(),
                session: "MoonBaseCapital-claude".to_string()
            }
        );
        // Surrounding whitespace is a paste artifact, not a coordinate.
        assert_eq!(
            parse_target("  matrix:MAC-STREAM  "),
            StreamTarget::Remote {
                host: "matrix".to_string(),
                session: "MAC-STREAM".to_string()
            }
        );
    }

    #[test]
    fn parse_target_degenerate_colons_stay_local() {
        // No valid remote reading exists for either of these, so preflight
        // must be allowed to report "session not found", not "host down".
        assert_eq!(
            parse_target(":MAC-STREAM"),
            StreamTarget::Local {
                session: "MAC-STREAM".to_string()
            }
        );
        assert_eq!(
            parse_target("matrix:"),
            StreamTarget::Local {
                session: "matrix".to_string()
            }
        );
    }

    #[test]
    fn parse_target_exposes_the_shell_loop_arguments() {
        let local = parse_target("oracle-OmegaOS");
        assert_eq!(local.host_arg(), "local");
        assert_eq!(local.session(), "oracle-OmegaOS");
        assert_eq!(local.host(), None);
        assert_eq!(local.label(), "oracle-OmegaOS");

        let remote = parse_target("matrix:MAC-STREAM");
        assert_eq!(remote.host_arg(), "matrix");
        assert_eq!(remote.session(), "MAC-STREAM");
        assert_eq!(remote.host(), Some("matrix"));
        assert_eq!(remote.label(), "matrix:MAC-STREAM");
    }

    /// Fixture, never the operator's real file: the parser is what is under
    /// test, and a machine-specific config would make this pass or fail for
    /// reasons that have nothing to do with the code.
    const SSH_CONFIG_FIXTURE: &str = r#"
# The operator's Mac, tailnet only.
Host matrix
    HostName matrix.example.ts.net
    User hacker

  # indented comment, and an indented keyword below
  host moonbasecapital
      Port 42820

HOST alpha beta gamma
    User shared

Host *
    ServerAliveInterval 30

Host prod-?
    User deploy

Host !staging trusted
    User ops

Host=equals-form
    User odd

Host matrix
    # duplicate alias, must not appear twice
"#;

    #[test]
    fn ssh_hosts_parses_aliases_tolerantly() {
        let cfg = parse_ssh_config(SSH_CONFIG_FIXTURE);
        assert_eq!(
            cfg.hosts,
            vec![
                "matrix",
                "moonbasecapital",
                "alpha",
                "beta",
                "gamma",
                "trusted",
                "equals-form",
            ],
            "expected concrete aliases in file order, deduped"
        );
        assert!(!cfg.has_include, "fixture has no Include directive");
    }

    #[test]
    fn ssh_hosts_drops_patterns_that_name_no_box() {
        let cfg = parse_ssh_config(SSH_CONFIG_FIXTURE);
        for pattern in ["*", "prod-?", "!staging"] {
            assert!(
                !cfg.hosts.iter().any(|h| h == pattern),
                "wildcard/negation leaked into the alias list: {pattern}"
            );
        }
    }

    #[test]
    fn ssh_hosts_flags_an_unexpanded_include() {
        let cfg = parse_ssh_config("Include ~/.ssh/config.d/*\nHost solo\n  User me\n");
        assert!(
            cfg.has_include,
            "an Include makes the alias list a lower bound; callers must not hard-reject on it"
        );
        assert_eq!(cfg.hosts, vec!["solo"]);
    }

    #[test]
    fn ssh_hosts_on_junk_is_empty_not_a_panic() {
        assert!(parse_ssh_config("").hosts.is_empty());
        assert!(parse_ssh_config("\n\n   \n# only comments\n")
            .hosts
            .is_empty());
        assert!(
            parse_ssh_config("Host\n").hosts.is_empty(),
            "bare keyword names nothing"
        );
    }

    #[test]
    fn viewer_name_follows_the_contract() {
        assert_eq!(
            viewer_name(&parse_target("oracle-OmegaOS")),
            "stream-oracle-OmegaOS"
        );
        assert_eq!(
            viewer_name(&parse_target("matrix:MAC-STREAM")),
            "stream-matrix-MAC-STREAM"
        );
    }

    #[test]
    fn viewer_name_neutralizes_what_rmux_would_rewrite() {
        // rmux does not refuse '.' or ':', it silently rewrites them to '_'
        // (verified live). A name rmux would rewrite is a name we could not
        // then find with has-session, so it must not survive this function.
        // The dotted host is ALSO a folding collision (`matrix.tail.ts.net` and
        // `matrix-tail-ts-net` fold together), so it earns a fingerprint.
        let name = viewer_name(&StreamTarget::Remote {
            host: "matrix.tail.ts.net".to_string(),
            session: "MAC-STREAM".to_string(),
        });
        assert!(
            name.starts_with("stream-matrix-tail-ts-net-MAC-STREAM-"),
            "the readable form is kept, plus what makes it unambiguous: {name}"
        );
        for n in [
            viewer_name(&parse_target("matrix:MAC-STREAM")),
            viewer_name(&StreamTarget::Local {
                session: "a.b.c".to_string(),
            }),
            viewer_name(&StreamTarget::Local {
                session: "Camélia build".to_string(),
            }),
        ] {
            assert!(
                !n.contains('.') && !n.contains(':') && !n.contains(' '),
                "viewer name would be rewritten by rmux: {n}"
            );
            assert!(
                n.starts_with("stream-"),
                "viewer names are recognizable: {n}"
            );
        }
    }

    #[test]
    fn viewer_name_never_doubles_a_separator() {
        let n = viewer_name(&StreamTarget::Remote {
            host: "box-1".to_string(),
            session: "a.b".to_string(),
        });
        assert!(
            n.starts_with("stream-box-1-a-b-"),
            "readable stem kept: {n}"
        );
        assert!(
            !n.contains("--"),
            "sanitize must not leave a doubled separator: {n}"
        );
    }

    // ── F-002 · the viewer name is the identity of the mirror ────────────────
    //
    // Every test below is a defect that was reproduced end to end: the operator
    // asked for B, was told they got B, and was shown A.

    /// The live reproduction, verbatim: two real sessions, 43 characters each,
    /// differing only in the last character. `stream-` + 43 overflows the
    /// 48-character cap, so the old code truncated both onto ONE name.
    #[test]
    fn viewer_name_survives_truncation_at_the_cap() {
        let a = parse_target("audit-stream-collision-probe-session-name-A");
        let b = parse_target("audit-stream-collision-probe-session-name-B");
        let (na, nb) = (viewer_name(&a), viewer_name(&b));

        assert_ne!(
            na, nb,
            "two different sources must never share one viewer name"
        );
        for n in [&na, &nb] {
            assert!(
                n.len() <= MAX_SESSION_NAME_LEN,
                "a name over the cap is a name rmux will truncate again: {n} ({})",
                n.len()
            );
            assert!(
                n.starts_with("stream-"),
                "viewer names stay recognizable: {n}"
            );
        }
        // The old behaviour, pinned so a regression is unmistakable.
        assert_ne!(
            na, "stream-audit-stream-collision-probe-session-name",
            "this is the truncated name both sources used to collapse onto"
        );
    }

    /// Folding collision: everything outside the slug alphabet becomes '-', so
    /// `box.local` and `box-local` used to compose the same name.
    #[test]
    fn viewer_name_survives_character_folding() {
        let pairs = [
            (
                StreamTarget::Local {
                    session: "box.local".to_string(),
                },
                StreamTarget::Local {
                    session: "box-local".to_string(),
                },
            ),
            (
                StreamTarget::Remote {
                    host: "matrix.tail.ts.net".to_string(),
                    session: "MAC-STREAM".to_string(),
                },
                StreamTarget::Remote {
                    host: "matrix-tail-ts-net".to_string(),
                    session: "MAC-STREAM".to_string(),
                },
            ),
        ];
        for (a, b) in pairs {
            assert_ne!(
                viewer_name(&a),
                viewer_name(&b),
                "{} and {} fold onto one name",
                a.label(),
                b.label()
            );
        }
    }

    /// A dashed HOST makes the boundary unreadable: `stream-a-b-c` is both
    /// `a` + `b-c` and `a-b` + `c`.
    #[test]
    fn viewer_name_survives_an_ambiguous_host_boundary() {
        let split_left = StreamTarget::Remote {
            host: "a-b".to_string(),
            session: "c".to_string(),
        };
        let split_right = StreamTarget::Remote {
            host: "a".to_string(),
            session: "b-c".to_string(),
        };
        assert_ne!(viewer_name(&split_left), viewer_name(&split_right));
    }

    /// The documented contract is NOT paid for by the fix: every coordinate
    /// that is already a clean, in-budget slug keeps the exact name it has
    /// today, including every viewer live on this box when the fix shipped.
    #[test]
    fn viewer_name_keeps_the_documented_form_for_clean_coordinates() {
        for (target, expected) in [
            ("oracle-OmegaOS", "stream-oracle-OmegaOS"),
            ("matrix:MAC-STREAM", "stream-matrix-MAC-STREAM"),
            ("matrix:MBC-STREAM", "stream-matrix-MBC-STREAM"),
            (
                "moonbasecapital:MoonBaseCapital-claude",
                "stream-moonbasecapital-MoonBaseCapital-claude",
            ),
            ("zzv-src", "stream-zzv-src"),
        ] {
            assert_eq!(
                viewer_name(&parse_target(target)),
                expected,
                "a clean coordinate must not be renamed by the collision fix"
            );
        }
    }

    /// The fingerprint is part of a session NAME: if it moves, every viewer it
    /// named becomes unfindable. Pinned so a hash change cannot be silent.
    #[test]
    fn the_fingerprint_is_stable_and_slug_safe() {
        let name = viewer_name(&parse_target("audit-stream-collision-probe-session-name-A"));
        assert_eq!(name, "stream-audit-stream-collision-probe-sessi-w2d5nm");
        assert_eq!(fingerprint("matrix:MAC-STREAM"), "brtzw8");
        assert_eq!(fingerprint(""), "j4ux45");
        for label in ["a", "matrix:MAC-STREAM", "oracle-OmegaOS", "Camélia build"] {
            let fp = fingerprint(label);
            assert_eq!(fp.len(), FINGERPRINT_LEN);
            assert!(
                fp.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
                "a fingerprint reaches rmux inside a session name: {fp}"
            );
        }
    }

    /// Sweep: nothing in a deliberately nasty population may share a name.
    #[test]
    fn viewer_names_are_injective_over_a_hostile_population() {
        let targets = [
            parse_target("a"),
            parse_target("a-b"),
            parse_target("a.b"),
            parse_target("a_b"),
            parse_target("host:a"),
            parse_target("host:a-b"),
            parse_target("host-x:a"),
            parse_target("host:a.b"),
            parse_target(&"x".repeat(60)),
            parse_target(&format!("{}A", "x".repeat(59))),
            parse_target(&format!("{}B", "x".repeat(59))),
            parse_target("audit-stream-collision-probe-session-name-A"),
            parse_target("audit-stream-collision-probe-session-name-B"),
            parse_target("Camélia build"),
            parse_target("Camelia-build"),
        ];
        let mut seen: Vec<(String, String)> = Vec::new();
        for t in &targets {
            let name = viewer_name(t);
            assert!(
                name.len() <= MAX_SESSION_NAME_LEN,
                "{name} is longer than rmux will keep"
            );
            if let Some((other, _)) = seen.iter().find(|(_, n)| n == &name) {
                panic!("{} and {} collide on {name}", other, t.label());
            }
            seen.push((t.label(), name));
        }
    }

    /// The one overlap a name-only scheme cannot remove (a local session
    /// literally named `<alias>-<session>`), pinned as a KNOWN property so it
    /// can never be mistaken for an accident — and proven closed one layer up
    /// by `choose_viewer` in the test that follows.
    #[test]
    fn the_local_versus_remote_overlap_is_known_and_resolved_above() {
        let local = parse_target("matrix-MAC-STREAM");
        let remote = parse_target("matrix:MAC-STREAM");
        assert_eq!(
            viewer_name(&local),
            viewer_name(&remote),
            "if this ever differs, the comment on viewer_name is out of date"
        );

        // A viewer for the LOCAL session is already up under that shared name.
        let taken = ViewerSlot::Mirroring(local.clone());
        let choice = choose_viewer(&remote, &taken, &ViewerSlot::Free);
        let name = choice.name().expect("the remote still gets a viewer");
        assert_ne!(
            name,
            viewer_name(&local),
            "the remote must NOT be handed the local session's mirror"
        );
        assert_eq!(choice, ViewerChoice::Create(name.to_string()));
    }

    // ── F-002 · "reusing it" is a claim about the SOURCE, not the name ───────

    #[test]
    fn choose_viewer_reuses_only_a_viewer_of_the_same_source() {
        let t = parse_target("matrix:MAC-STREAM");
        assert_eq!(
            choose_viewer(&t, &ViewerSlot::Mirroring(t.clone()), &ViewerSlot::Free),
            ViewerChoice::Reuse("stream-matrix-MAC-STREAM".to_string()),
            "the same source under the same name is the whole point of idempotency"
        );
        assert_eq!(
            choose_viewer(&t, &ViewerSlot::Free, &ViewerSlot::Free),
            ViewerChoice::Create("stream-matrix-MAC-STREAM".to_string())
        );
    }

    #[test]
    fn choose_viewer_never_reuses_a_viewer_of_a_different_source() {
        // The shape of the live repro, in the one form that still reaches a
        // shared name after the naming fix: something else is already standing
        // on the name this coordinate composes to — here the local session that
        // looks exactly like the remote coordinate. It could equally be a viewer
        // left behind by a pre-fix build under a truncated name.
        let wanted = parse_target("matrix:MAC-STREAM");
        let showing = parse_target("matrix-MAC-STREAM");
        assert_eq!(viewer_name(&wanted), viewer_name(&showing), "premise");

        let choice = choose_viewer(
            &wanted,
            &ViewerSlot::Mirroring(showing.clone()),
            &ViewerSlot::Free,
        );
        match &choice {
            ViewerChoice::Create(name) => {
                assert_ne!(
                    name,
                    &viewer_name(&showing),
                    "that name is rendering the OTHER source"
                );
                assert_eq!(name, &fingerprinted_viewer_name(&wanted));
            }
            other => panic!("the remote must get its own viewer, not {other:?}"),
        }
        assert!(
            !matches!(choice, ViewerChoice::Reuse(_)),
            "'reusing it' over another session's mirror is the F-002 defect"
        );
    }

    /// When the name a target composes to is ALREADY its fingerprinted form,
    /// there is no second name to fall back to. Refusing is the only honest
    /// answer left: attaching would show the operator someone else's session.
    #[test]
    fn choose_viewer_has_no_second_escape_from_a_fingerprinted_name() {
        let t = parse_target("audit-stream-collision-probe-session-name-B");
        assert_eq!(
            viewer_name(&t),
            fingerprinted_viewer_name(&t),
            "premise: an over-long coordinate is fingerprinted from the start"
        );
        let squatter = parse_target("something-else");
        assert_eq!(
            choose_viewer(
                &t,
                &ViewerSlot::Mirroring(squatter.clone()),
                &ViewerSlot::Free
            ),
            ViewerChoice::Conflict {
                name: viewer_name(&t),
                held_by: Some(squatter),
            }
        );
    }

    #[test]
    fn choose_viewer_treats_an_unreadable_session_as_occupied() {
        let t = parse_target("oracle-OmegaOS");
        // A session under that name whose command we cannot parse could be
        // anything, including the operator's own work. Never adopt it.
        let choice = choose_viewer(&t, &ViewerSlot::Taken, &ViewerSlot::Free);
        assert_eq!(
            choice,
            ViewerChoice::Create(fingerprinted_viewer_name(&t)),
            "an unreadable holder is stepped around, never reused"
        );
    }

    #[test]
    fn choose_viewer_refuses_rather_than_stealing_when_both_names_are_held() {
        let t = parse_target("oracle-OmegaOS");
        let other = parse_target("something-else");
        let choice = choose_viewer(
            &t,
            &ViewerSlot::Mirroring(other.clone()),
            &ViewerSlot::Mirroring(other.clone()),
        );
        assert_eq!(
            choice,
            ViewerChoice::Conflict {
                name: fingerprinted_viewer_name(&t),
                held_by: Some(other),
            },
            "with no free name left, refusing beats attaching to the wrong mirror"
        );
        assert!(
            choice.name().is_none(),
            "a conflict names no viewer to attach"
        );
    }

    #[test]
    fn choose_viewer_reuses_the_fingerprinted_viewer_when_that_is_the_one_running() {
        // Second `omega stream matrix:MAC-STREAM` after the first one had to
        // step aside: it must land back on ITS viewer, not create a third.
        let t = parse_target("matrix:MAC-STREAM");
        let other = parse_target("matrix-MAC-STREAM");
        let choice = choose_viewer(
            &t,
            &ViewerSlot::Mirroring(other),
            &ViewerSlot::Mirroring(t.clone()),
        );
        assert_eq!(
            choice,
            ViewerChoice::Reuse(fingerprinted_viewer_name(&t)),
            "idempotency must survive the fallback, or every run adds a puller"
        );
    }

    // ── F-002 · reading what a viewer is really pulling ──────────────────────

    #[test]
    fn a_viewer_command_round_trips_back_to_its_target() {
        for label in ["oracle-OmegaOS", "matrix:MAC-STREAM"] {
            let t = parse_target(label);
            let cmd = viewer_command(&t, 3, 120);
            assert_eq!(
                parse_viewer_command(&cmd),
                Some(t),
                "the seam contract is <script> <target> <session> <interval> <lines>: {cmd}"
            );
        }
    }

    #[test]
    fn parse_viewer_command_refuses_anything_that_is_not_the_loop() {
        for cmd in [
            "",
            "bash -c 'claude --dangerously-skip-permissions'",
            "/home/vibe/.omega/bin/omega-stream.sh",
            "/home/vibe/.omega/bin/omega-stream.sh local",
            "/home/vibe/.omega/bin/other-script.sh local a 3 120",
            "/home/vibe/.omega/bin/omega-stream.sh local a;id 3 120",
        ] {
            assert_eq!(
                parse_viewer_command(cmd),
                None,
                "an unparseable session must be treated as occupied, not adopted: {cmd:?}"
            );
        }
    }

    /// Fixture captured from the live rmux server
    /// (`rmux list-panes -a -F '#S|#{pane_start_command}'`), including the
    /// percent-encoded shape rmux uses for a command with shell metacharacters.
    const VIEWER_INDEX_FIXTURE: &str = concat!(
        "oracle-OmegaOS|bash -c %27claude --dangerously-skip-permissions%27\n",
        "stream-matrix-MAC-STREAM|/home/vibe/.omega/bin/omega-stream.sh matrix MAC-STREAM 3 120\n",
        "stream-zzv-src|/home/vibe/.omega/bin/omega-stream.sh local zzv-src 2 60\n",
        "stream-broken|bash\n",
        "\n",
        "not-a-viewer|/home/vibe/.omega/bin/omega-stream.sh local hidden 3 120\n",
    );

    #[test]
    fn the_mirror_index_reports_what_each_viewer_actually_pulls() {
        let index = parse_viewer_index(VIEWER_INDEX_FIXTURE);
        assert_eq!(
            index,
            vec![
                (
                    "stream-matrix-MAC-STREAM".to_string(),
                    parse_target("matrix:MAC-STREAM")
                ),
                ("stream-zzv-src".to_string(), parse_target("zzv-src")),
            ],
            "only readable stream-* viewers count, and each maps to its REAL source"
        );

        // The point of the index: a source is "already mirrored" only when a
        // viewer is really pulling it, whatever that viewer happens to be named.
        let local_lookalike = parse_target("matrix-MAC-STREAM");
        assert!(
            !index.iter().any(|(_, t)| t == &local_lookalike),
            "the colliding LOCAL session must not be reported as mirrored: only the remote is"
        );
        assert!(index
            .iter()
            .any(|(_, t)| t == &parse_target("matrix:MAC-STREAM")));
    }

    #[test]
    fn viewer_command_matches_the_shell_seam() {
        let target = parse_target("matrix:MAC-STREAM");
        let cmd = viewer_command(&target, 3, 120);
        assert!(
            cmd.ends_with(" matrix MAC-STREAM 3 120"),
            "shell loop takes <target> <session> <interval> <lines>: {cmd}"
        );
        assert!(cmd.contains("omega-stream.sh"));
        // Rust spawns rmux directly, with no shell of its own, so the script
        // path must already be absolute.
        assert!(cmd.starts_with('/'), "script path must be expanded: {cmd}");

        let local = parse_target("oracle-OmegaOS");
        assert!(
            viewer_command(&local, 5, 200).ends_with(" local oracle-OmegaOS 5 200"),
            "a local target passes the literal `local`"
        );
    }

    #[test]
    fn viewer_argv_is_the_documented_new_session_call() {
        let target = parse_target("matrix:MAC-STREAM");
        let argv = viewer_argv("stream-matrix-MAC-STREAM", &target, 3, 120);
        assert_eq!(
            argv[..4],
            ["new-session", "-d", "-s", "stream-matrix-MAC-STREAM"]
        );
        assert_eq!(argv.len(), 5, "the loop is ONE command argument");
    }

    #[test]
    fn probe_argv_keeps_home_unexpanded_for_the_remote_shell() {
        let argv = probe_argv(Some("matrix"));
        let remote_cmd = argv.last().expect("remote command is the last argument");
        // LESSON 5: expanding $HOME locally once told a Linux box to read a
        // macOS path. It must arrive at the remote shell verbatim.
        assert!(
            remote_cmd.contains("$HOME/.local/bin/rmux"),
            "remote rmux path must stay literal: {remote_cmd}"
        );
        if let Some(home) = dirs::home_dir() {
            assert!(
                !remote_cmd.contains(&home.display().to_string()),
                "the LOCAL home leaked into the remote command: {remote_cmd}"
            );
        }
        // '#' unquoted starts a comment in the remote shell, which would
        // silently drop the format and print the verbose listing instead.
        assert!(
            remote_cmd.contains("'#S'"),
            "the #S format must stay quoted: {remote_cmd}"
        );
        assert_eq!(argv[0], "ssh");
        assert!(
            argv.iter().any(|a| a == "BatchMode=yes"),
            "probes never prompt"
        );
        assert!(
            argv.iter()
                .any(|a| a == &format!("ConnectTimeout={SSH_CONNECT_TIMEOUT_SECS}")),
            "probes are bounded"
        );
        assert!(
            argv.iter().any(|a| a == "matrix"),
            "the ALIAS is passed to ssh, never a resolved coordinate"
        );
        assert!(
            !argv.iter().any(|a| a.contains("42820") || a.contains('@')),
            "ssh resolves HostName/Port/User from its config: {argv:?}"
        );
    }

    #[test]
    fn probe_argv_local_uses_the_absolute_rmux() {
        let argv = probe_argv(None);
        assert!(
            argv[0].ends_with("rmux"),
            "local probe runs rmux directly: {argv:?}"
        );
        assert_eq!(argv[1..], ["list-sessions", "-F", "#S"]);
    }

    #[test]
    fn parse_session_list_ignores_blank_lines() {
        assert_eq!(
            parse_session_list("MAC-STREAM\n\n  oracle-OmegaOS  \n"),
            vec!["MAC-STREAM", "oracle-OmegaOS"]
        );
        assert!(parse_session_list("").is_empty());
    }

    #[test]
    fn unsafe_coordinates_are_refused_not_quoted() {
        assert!(is_safe_coordinate("oracle-OmegaOS"));
        assert!(is_safe_coordinate("MoonBaseCapital-claude"));
        assert!(is_safe_coordinate("box.local"));
        for bad in [
            "",
            "a b",
            "a;rm -rf /",
            "a$(id)",
            "a`id`",
            "a|b",
            "a&b",
            "a'b",
        ] {
            assert!(
                !is_safe_coordinate(bad),
                "coordinate reaches a shell command line, must be refused: {bad:?}"
            );
        }
    }

    #[test]
    fn probe_outcomes_describe_themselves() {
        assert!(ProbeOutcome::Sessions(vec!["a".into()]).reachable());
        assert!(!ProbeOutcome::TimedOut { secs: 12 }.reachable());
        assert_eq!(
            ProbeOutcome::Sessions(vec!["a".into()]).sessions(),
            Some(&["a".to_string()][..])
        );
        assert!(ProbeOutcome::Unreachable {
            detail: "connection timed out".into()
        }
        .describe()
        .contains("unreachable"));
        assert!(ProbeOutcome::RmuxFailed {
            code: Some(1),
            detail: "no server running".into()
        }
        .describe()
        .contains("exit 1"));
    }

    #[tokio::test]
    async fn a_black_holed_host_is_bounded_not_hung() {
        // RFC 5737 TEST-NET-1: routable syntax, guaranteed no listener, so
        // the packets vanish and only the wall clock can end this. The bound
        // is passed in (1s) rather than paying PROBE_TIMEOUT in the suite;
        // what is under test is that the clock fires at all.
        let bound = Duration::from_secs(1);
        let started = std::time::Instant::now();
        let outcome = probe_host_bounded(Some("192.0.2.1"), bound).await;
        let elapsed = started.elapsed();

        assert!(
            elapsed < bound + Duration::from_secs(3),
            "a black-holed host must not hang `omega stream list`: took {elapsed:?}"
        );
        assert!(
            !outcome.reachable(),
            "a black-holed address must never report sessions: {outcome:?}"
        );
        assert!(
            matches!(
                outcome,
                ProbeOutcome::TimedOut { .. }
                    | ProbeOutcome::Unreachable { .. }
                    | ProbeOutcome::SpawnFailed { .. }
            ),
            "ssh's own failure must not be read as a missing session: {outcome:?}"
        );
    }
}
