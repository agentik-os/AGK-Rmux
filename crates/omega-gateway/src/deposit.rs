//! Pure deposit logic — the native Rust reimplementation of the Telegram
//! DEPOSIT bot's fan-out (`~/.omega/telegram-bot/inbox-bot.ts`), read in full
//! as the ground-truth spec for this module. No HTTP here; `routes_deposit.rs`
//! is the thin HTTP wrapper around [`deposit`].
//!
//! Storage layout under `deposit_home` (mirrors the real `~/.omega` layout):
//! `<deposit_home>/inbox/<timestamp>_<uniq>_<sanitized-name>`   (the original, always)
//! `<deposit_home>/deposit/<Box>/<timestamp>_<uniq>_<sanitized-name>`  (hard
//!   link, or a copy when the link fails, e.g. cross-filesystem)
//! `<deposit_home>/deposit/index.jsonl`  (one JSON line per deposit, held or not)
//! `<deposit_home>/deposit.toml`  (`DepositConfig`)
//!
//! `<uniq>` is a short random hex suffix (bug B2): the timestamp alone has
//! only 1-second granularity, so two deposits of the same original filename
//! arriving in the same wall-clock second would otherwise collide onto the
//! identical final path.

use crate::util::random_hex;
use serde::Deserialize;
use std::path::Path;

/// `boxes = [...]` / `fanout_secrets = true|false` in `<deposit_home>/deposit.toml`.
/// Same load idiom as `GatewayConfig::load`: missing or invalid file both
/// yield defaults (a corrupt file warns, never panics).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DepositConfig {
    pub boxes: Vec<String>,
    pub fanout_secrets: bool,
}

impl Default for DepositConfig {
    fn default() -> Self {
        Self {
            boxes: vec![
                "Home".into(),
                "AltReality".into(),
                "Omega".into(),
                "Box".into(),
            ],
            fanout_secrets: false,
        }
    }
}

impl DepositConfig {
    pub fn load(deposit_home: &Path) -> Self {
        let path = deposit_home.join("deposit.toml");
        match std::fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text).unwrap_or_else(|e| {
                tracing::warn!("invalid {}: {e}; using defaults", path.display());
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }
}

/// Outcome of one [`deposit`] call.
#[derive(Debug)]
pub struct DepositOutcome {
    /// The final timestamped filename written to the inbox.
    pub file: String,
    /// Boxes the deposit actually reached (empty when `held` is true).
    pub boxes: Vec<String>,
    /// True when the upload looked like a credential and was kept in the
    /// inbox only (see [`looks_secret`]).
    pub held: bool,
}

/// Sanitizes `original_name`, writes `bytes` into the inbox under a
/// timestamped filename, then (unless secret-looking and not forced)
/// hard-links (copy-fallback) that same file into each target box under
/// `<deposit_home>/deposit/`, and appends one line to
/// `<deposit_home>/deposit/index.jsonl`.
///
/// `box_tag`: `None` fans out to every configured box; `Some(name)` narrows
/// to that one box (case-insensitive match against the configured box
/// names). An unrecognized `box_tag` is rejected with `Err` BEFORE anything
/// is written to disk — not even the inbox original — matching the
/// path-traversal-guard discipline `valid_chat_id`/`valid_session_name`
/// already use elsewhere in this crate: validate first, touch the
/// filesystem second.
pub fn deposit(
    deposit_home: &Path,
    original_name: &str,
    bytes: &[u8],
    box_tag: Option<&str>,
    share: bool,
) -> anyhow::Result<DepositOutcome> {
    let config = DepositConfig::load(deposit_home);

    // Validate the box tag BEFORE touching the filesystem at all.
    let requested_box: Option<String> = match box_tag {
        Some(tag) => match config.boxes.iter().find(|b| b.eq_ignore_ascii_case(tag)) {
            Some(b) => Some(b.clone()),
            None => anyhow::bail!("unknown box: {tag}"),
        },
        None => None,
    };

    let sanitized = sanitize_filename(original_name);

    // B3: secret-detection MUST run against the sanitized-but-NOT-YET-
    // truncated name. `looks_secret`'s regex end-anchors several patterns
    // (e.g. `\.pem$`); truncating first can slice off the very suffix that
    // makes a name look secret (110 `a`s + `.pem` truncated to 100 chars
    // loses the `.pem`), silently defeating the credential-detection gate.
    // Live-reproduced: `held:false`, boxes populated. NEVER move this check
    // after `truncate_for_filesystem` below.
    let held = looks_secret(&sanitized) && !share && !config.fanout_secrets;
    let truncated = truncate_for_filesystem(&sanitized);

    let inbox_dir = deposit_home.join("inbox");
    std::fs::create_dir_all(&inbox_dir)?;
    let filename = write_unique_inbox_file(&inbox_dir, &timestamp(), &truncated, bytes)?;
    let inbox_path = inbox_dir.join(&filename);

    let boxes_dir = deposit_home.join("deposit");
    let mut reached: Vec<String> = Vec::new();
    if !held {
        let targets: Vec<String> = match requested_box {
            Some(b) => vec![b],
            None => config.boxes.clone(),
        };
        std::fs::create_dir_all(&boxes_dir)?;
        for b in &targets {
            let bdir = boxes_dir.join(b);
            std::fs::create_dir_all(&bdir)?;
            let dst = bdir.join(&filename);
            // Zero-extra-disk fan-out: hard-link, falling back to a copy when
            // the link fails — mirrors inbox-bot.ts's `place()`. B2 fix:
            // match the error KIND explicitly instead of a bare `.is_err()`
            // catch-all, so a genuine unexpected error (permissions, disk
            // full) propagates instead of being silently swallowed into a
            // wrong-looking "fell back to copy" success path. Only fall back
            // when linking genuinely isn't possible: `CrossesDevices` (the
            // box lives on a different filesystem) or `AlreadyExists` (the
            // well-understood case `write_unique_inbox_file`'s uniquifier
            // now prevents at the source — if it ever fires, a copy is still
            // the right recovery, but any other error is a real anomaly).
            match std::fs::hard_link(&inbox_path, &dst) {
                Ok(()) => {}
                Err(e)
                    if e.kind() == std::io::ErrorKind::AlreadyExists
                        || e.kind() == std::io::ErrorKind::CrossesDevices =>
                {
                    std::fs::copy(&inbox_path, &dst)?;
                }
                Err(e) => return Err(e.into()),
            }
            reached.push(b.clone());
        }
    }

    // One index line per deposit, held or not, so the log is a complete
    // record of everything that arrived.
    std::fs::create_dir_all(&boxes_dir)?;
    let index_path = boxes_dir.join("index.jsonl");
    let line = serde_json::json!({
        "ts": timestamp(),
        "file": filename,
        "boxes": reached,
        "held": held,
    });
    {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&index_path)?;
        writeln!(f, "{line}")?;
    }

    Ok(DepositOutcome {
        file: filename,
        boxes: reached,
        held,
    })
}

/// The exact secret-detection regex the real Telegram DEPOSIT bot uses
/// (`SECRETISH` in `inbox-bot.ts`), case-insensitive, matched against the
/// sanitized (but NOT length-truncated — see `truncate_for_filesystem`'s doc
/// comment, bug B3) base filename only.
fn looks_secret(filename: &str) -> bool {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(
            r"(?i)(\.(p8|pem|key|env|p12|jks|keystore|ppk|crt|pfx)$)|(^|[._-])(id_rsa|id_ed25519)|credential|secret|token|passwd|private[._-]?key",
        )
        .expect("static secret-detection regex is valid")
    });
    re.is_match(filename)
}

/// Strips anything outside `[A-Za-z0-9._-]` to `_`. Never empty — falls back
/// to `"file"` when sanitization leaves nothing. Because `/` (and every other
/// separator) is replaced, the result is always a single path component:
/// joined onto a directory it can never path-traverse out of it, even when
/// `original_name` is `../../etc/passwd`-shaped.
///
/// Deliberately does NOT truncate to a filesystem-safe length — that is
/// `truncate_for_filesystem`'s job, and it must run strictly AFTER
/// `looks_secret` has already decided `held` (bug B3).
fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "file".to_string()
    } else {
        sanitized
    }
}

/// Caps an already-sanitized name at 100 chars, for constructing the final
/// on-disk path only.
///
/// MUST be called only AFTER `looks_secret` has already run against the
/// untruncated `sanitize_filename` output. `looks_secret`'s regex
/// end-anchors several patterns (e.g. `\.pem$`); truncating first can slice
/// off the exact suffix that makes a name look secret — 110 `a`s followed by
/// `.pem`, capped at 100 chars, loses the `.pem` entirely and the file is
/// classified as NOT secret. Live-reproduced by a runtime review:
/// `held:false`, boxes populated (bug B3). If this ordering is ever
/// "simplified" back to truncate-then-check, that credential-leak bypass
/// comes back with it — don't.
///
/// `sanitize_filename`'s output is always non-empty and pure ASCII, so
/// `truncate` here can never land on a non-UTF-8 boundary or produce an
/// empty string.
fn truncate_for_filesystem(sanitized: &str) -> String {
    let mut truncated = sanitized.to_string();
    truncated.truncate(100);
    truncated
}

/// Number of random-uniquifier retries before giving up (bug B2). A
/// collision on the very first attempt already requires two deposits with
/// the identical sanitized name landing in the identical wall-clock second
/// AND rolling the identical 3-byte random suffix; this ceiling exists only
/// to turn a theoretical infinite loop into a bounded, diagnosable error.
const MAX_FILENAME_ATTEMPTS: u32 = 5;

/// Writes `bytes` into a NEW file under `inbox_dir`, named
/// `<ts>_<uniq>_<truncated>`, where `uniq` is a few random hex chars.
///
/// Two fixes for bug B2 in one function: (1) the random `uniq` suffix makes
/// the final filename collision-safe even when two deposits of the same
/// original filename land in the same wall-clock second (`ts` alone has only
/// 1-second granularity); (2) the file is opened with `create_new(true)`,
/// which atomically FAILS with `ErrorKind::AlreadyExists` if the target path
/// already exists, rather than `std::fs::write`'s silent in-place
/// truncate+rewrite of an existing file. That in-place truncate was the
/// actual corruption vector: if the existing file at that path was already
/// hard-linked into box directories, truncating it in place corrupted every
/// box copy too, and the shared-inode `AlreadyExists` on the *box* hard-link
/// step then fell back to copying from the just-clobbered inbox file. On the
/// vanishingly unlikely chance of a genuine `uniq` collision, retries once
/// with a fresh suffix rather than ever falling back to an in-place
/// overwrite.
fn write_unique_inbox_file(
    inbox_dir: &Path,
    ts: &str,
    truncated: &str,
    bytes: &[u8],
) -> anyhow::Result<String> {
    for _ in 0..MAX_FILENAME_ATTEMPTS {
        let uniq = random_hex(3);
        let filename = format!("{ts}_{uniq}_{truncated}");
        let path = inbox_dir.join(&filename);
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut f) => {
                use std::io::Write;
                f.write_all(bytes)?;
                return Ok(filename);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.into()),
        }
    }
    anyhow::bail!(
        "could not allocate a unique inbox filename for {truncated:?} after {MAX_FILENAME_ATTEMPTS} attempts"
    );
}

/// `YYYY-MM-DD_HHMMSS`, matching the Telegram bot's `stamp()` shape exactly.
fn timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%d_%H%M%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    fn ino(path: &Path) -> u64 {
        use std::os::unix::fs::MetadataExt;
        std::fs::metadata(path).unwrap().ino()
    }

    #[test]
    fn no_box_tag_fans_out_to_all_four_default_boxes_as_hard_links() {
        let dir = tempfile::tempdir().unwrap();
        let outcome = deposit(dir.path(), "notes.txt", b"hello", None, false).unwrap();

        assert!(!outcome.held);
        let mut boxes = outcome.boxes.clone();
        boxes.sort();
        assert_eq!(boxes, vec!["AltReality", "Box", "Home", "Omega"]);

        let inbox_path = dir.path().join("inbox").join(&outcome.file);
        assert!(inbox_path.exists());

        #[cfg(unix)]
        {
            let inbox_ino = ino(&inbox_path);
            for b in ["Home", "AltReality", "Omega", "Box"] {
                let box_path = dir.path().join("deposit").join(b).join(&outcome.file);
                assert!(box_path.exists());
                assert_eq!(ino(&box_path), inbox_ino, "{b} copy is not a hard link");
            }
        }
    }

    #[test]
    fn specific_box_tag_lands_only_in_that_box() {
        let dir = tempfile::tempdir().unwrap();
        let outcome = deposit(dir.path(), "notes.txt", b"hello", Some("omega"), false).unwrap();

        assert_eq!(outcome.boxes, vec!["Omega".to_string()]);
        assert!(dir
            .path()
            .join("deposit")
            .join("Omega")
            .join(&outcome.file)
            .exists());
        for b in ["Home", "AltReality", "Box"] {
            assert!(!dir
                .path()
                .join("deposit")
                .join(b)
                .join(&outcome.file)
                .exists());
        }
    }

    #[test]
    fn unrecognized_box_tag_errors_before_writing_anything() {
        let dir = tempfile::tempdir().unwrap();
        let err = deposit(dir.path(), "notes.txt", b"hello", Some("Nowhere"), false).unwrap_err();
        assert!(err.to_string().contains("Nowhere"));
        assert!(
            !dir.path().join("inbox").exists(),
            "inbox must stay untouched on validation failure"
        );
        assert!(
            !dir.path().join("deposit").exists(),
            "no box dir may be created on validation failure"
        );
    }

    #[test]
    fn secret_looking_file_is_held_by_default() {
        let dir = tempfile::tempdir().unwrap();
        let outcome = deposit(dir.path(), "id_rsa", b"private", None, false).unwrap();

        assert!(outcome.held);
        assert!(outcome.boxes.is_empty());
        assert!(dir.path().join("inbox").join(&outcome.file).exists());
        for b in ["Home", "AltReality", "Omega", "Box"] {
            let box_dir = dir.path().join("deposit").join(b);
            let has_files =
                box_dir.exists() && std::fs::read_dir(&box_dir).unwrap().next().is_some();
            assert!(!has_files, "{b} must not receive a held file");
        }
    }

    #[test]
    fn secret_looking_file_shares_when_share_is_true() {
        let dir = tempfile::tempdir().unwrap();
        let outcome = deposit(dir.path(), "service.pem", b"cert", None, true).unwrap();

        assert!(!outcome.held);
        assert_eq!(outcome.boxes.len(), 4);
        for b in ["Home", "AltReality", "Omega", "Box"] {
            assert!(dir
                .path()
                .join("deposit")
                .join(b)
                .join(&outcome.file)
                .exists());
        }
    }

    #[test]
    fn secret_looking_file_shares_when_fanout_secrets_configured() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("deposit.toml"), "fanout_secrets = true\n").unwrap();

        let outcome = deposit(dir.path(), ".env", b"SECRET=1", None, false).unwrap();

        assert!(!outcome.held);
        assert_eq!(outcome.boxes.len(), 4);
    }

    #[test]
    fn filename_sanitization_prevents_path_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let outcome = deposit(dir.path(), "../../etc/passwd", b"x", None, false).unwrap();

        // The written file must live directly under inbox/, never having
        // escaped it via a slash surviving sanitization.
        let inbox_dir = dir.path().join("inbox");
        let entries: Vec<_> = std::fs::read_dir(&inbox_dir).unwrap().collect();
        assert_eq!(entries.len(), 1);
        assert!(!outcome.file.contains('/'));
        assert!(inbox_dir.join(&outcome.file).exists());
        // Nothing was written outside the tempdir root.
        assert!(!dir.path().join("etc").exists());
    }

    #[test]
    fn sanitize_filename_never_empty() {
        assert_eq!(sanitize_filename(""), "file");
        // Disallowed characters map to '_', which is itself a legal filename
        // character — that's a valid (if odd) non-empty result, not a case
        // for the "file" fallback.
        assert_eq!(sanitize_filename("///"), "___");
    }

    #[test]
    fn truncate_for_filesystem_caps_at_100_chars() {
        let long = "a".repeat(150);
        let truncated = truncate_for_filesystem(&long);
        assert_eq!(truncated.len(), 100);
    }

    /// Bug B3 regression test: a filename long enough that the 100-char
    /// filesystem cap would slice off a trailing secret-marking suffix
    /// (`.pem`) must still be classified as secret, because the check now
    /// runs BEFORE truncation. Before the fix this deposited with
    /// `held:false` and fanned out into every box.
    #[test]
    fn long_secret_shaped_filename_is_held_even_though_truncation_would_hide_it() {
        let dir = tempfile::tempdir().unwrap();
        // 110 'a's + ".pem" = 114 chars; truncated to 100 chars, the
        // trailing ".pem" is entirely gone, so a truncate-first check would
        // see only "aaa...a" (100 a's) and `looks_secret`'s `\.pem$` anchor
        // would never match.
        let original_name = format!("{}.pem", "a".repeat(110));
        let outcome = deposit(
            dir.path(),
            &original_name,
            b"-----BEGIN KEY-----",
            None,
            false,
        )
        .unwrap();

        assert!(
            outcome.held,
            "a long filename ending in .pem must still be detected as secret"
        );
        assert!(
            outcome.boxes.is_empty(),
            "a held file must reach zero boxes"
        );
        for b in ["Home", "AltReality", "Omega", "Box"] {
            let box_dir = dir.path().join("deposit").join(b);
            let has_files =
                box_dir.exists() && std::fs::read_dir(&box_dir).unwrap().next().is_some();
            assert!(!has_files, "{b} must not have received the held file");
        }
    }

    /// Bug B2 regression test: two DIFFERENT-content deposits that sanitize
    /// to the SAME base name, issued back-to-back within the same function
    /// call (so almost certainly the same wall-clock second, no sleep),
    /// must land as two distinct files with both payloads independently
    /// intact — never a collision that truncates/corrupts one or both.
    #[test]
    fn same_second_same_name_deposits_never_collide_or_corrupt_each_other() {
        let dir = tempfile::tempdir().unwrap();
        let payload_a = b"first payload, must survive intact".to_vec();
        let payload_b = b"second payload, completely different bytes".to_vec();

        let outcome_a = deposit(dir.path(), "note.txt", &payload_a, None, false).unwrap();
        let outcome_b = deposit(dir.path(), "note.txt", &payload_b, None, false).unwrap();

        assert_ne!(
            outcome_a.file, outcome_b.file,
            "two same-second same-name deposits must not collide onto one filename"
        );

        let inbox_dir = dir.path().join("inbox");
        let bytes_a = std::fs::read(inbox_dir.join(&outcome_a.file)).unwrap();
        let bytes_b = std::fs::read(inbox_dir.join(&outcome_b.file)).unwrap();
        assert_eq!(
            bytes_a, payload_a,
            "first deposit's inbox content must be exactly its own payload"
        );
        assert_eq!(
            bytes_b, payload_b,
            "second deposit's inbox content must be exactly its own payload"
        );

        #[cfg(unix)]
        {
            for (outcome, payload) in [(&outcome_a, &payload_a), (&outcome_b, &payload_b)] {
                let inbox_path = inbox_dir.join(&outcome.file);
                let inbox_ino = ino(&inbox_path);
                for b in ["Home", "AltReality", "Omega", "Box"] {
                    let box_path = dir.path().join("deposit").join(b).join(&outcome.file);
                    assert!(box_path.exists(), "{b} must have received {}", outcome.file);
                    assert_eq!(
                        ino(&box_path),
                        inbox_ino,
                        "{b} copy of {} is not a hard link",
                        outcome.file
                    );
                    assert_eq!(
                        std::fs::read(&box_path).unwrap(),
                        *payload,
                        "{b} copy of {} has wrong content",
                        outcome.file
                    );
                }
            }
        }
    }

    #[test]
    fn looks_secret_matches_the_telegram_bot_regex() {
        for name in [
            "id_rsa",
            "id_ed25519",
            "service.pem",
            ".env",
            "my.key",
            "app.p12",
            "a_secret_note.txt",
            "TOKEN.txt",
            "private-key.bin",
        ] {
            assert!(looks_secret(name), "{name} should look secret");
        }
        for name in ["notes.txt", "photo.jpg", "report.pdf"] {
            assert!(!looks_secret(name), "{name} should not look secret");
        }
    }

    mod deposit_config {
        use super::*;

        #[test]
        fn missing_file_yields_defaults() {
            let dir = tempfile::tempdir().unwrap();
            let cfg = DepositConfig::load(dir.path());
            assert_eq!(cfg.boxes, vec!["Home", "AltReality", "Omega", "Box"]);
            assert!(!cfg.fanout_secrets);
        }

        #[test]
        fn file_overrides_fields() {
            let dir = tempfile::tempdir().unwrap();
            std::fs::write(
                dir.path().join("deposit.toml"),
                "boxes = [\"Only\"]\nfanout_secrets = true\n",
            )
            .unwrap();
            let cfg = DepositConfig::load(dir.path());
            assert_eq!(cfg.boxes, vec!["Only"]);
            assert!(cfg.fanout_secrets);
        }

        #[test]
        fn invalid_file_yields_defaults() {
            let dir = tempfile::tempdir().unwrap();
            std::fs::write(dir.path().join("deposit.toml"), "not valid toml {{{").unwrap();
            let cfg = DepositConfig::load(dir.path());
            assert_eq!(cfg.boxes, vec!["Home", "AltReality", "Omega", "Box"]);
            assert!(!cfg.fanout_secrets);
        }
    }
}
