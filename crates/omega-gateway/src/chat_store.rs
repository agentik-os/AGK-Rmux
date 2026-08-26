//! Persisted chat metadata + transcripts.
//!
//! Storage layout under the gateway data dir:
//! `<gateway_dir>/chats/<id>/meta.json`        (0600, ChatMeta)
//! `<gateway_dir>/chats/<id>/transcript.jsonl`  (0600, one ChatMessage per line)
//! Each `<gateway_dir>/chats/<id>/` dir is hardened to 0700.

use crate::fsperm::{harden_dir, harden_file};
use crate::protocol::{ChatAgent, ChatMessage, ChatMeta};
use crate::util::random_hex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct ChatStore {
    chats_dir: PathBuf,
    /// I-5 guard: chat ids with a turn currently in flight. `chat_permits`
    /// (see `server::AppState`) only caps the GLOBAL concurrent turn count,
    /// so two different WebSocket connections to the SAME chat id could
    /// each acquire a permit and each spawn a turn concurrently, racing the
    /// transcript append and `set_provider_session` writes. This set makes
    /// "a turn is active on chat X" an explicit, checkable fact instead.
    active_turns: Mutex<HashSet<String>>,
}

impl ChatStore {
    /// Opens (creating if needed) the chat store rooted at `<gateway_dir>/chats`.
    pub fn open(gateway_dir: &Path) -> Self {
        let chats_dir = gateway_dir.join("chats");
        std::fs::create_dir_all(&chats_dir).ok();
        harden_dir(&chats_dir);
        Self {
            chats_dir,
            active_turns: Mutex::new(HashSet::new()),
        }
    }

    /// Marks chat `id` as having a turn in flight. Returns `true` and
    /// records it when no turn was already active for this chat; returns
    /// `false`, changing nothing, when one already is. The caller is
    /// expected to call [`Self::end_turn`] on every exit path from the turn
    /// it started (an RAII guard is the safe way to guarantee that).
    pub fn try_start_turn(&self, id: &str) -> bool {
        let mut set = self.active_turns.lock().unwrap_or_else(|e| e.into_inner());
        set.insert(id.to_string())
    }

    /// Marks chat `id` as no longer having a turn in flight. A no-op if no
    /// turn was recorded as active for this id.
    pub fn end_turn(&self, id: &str) {
        let mut set = self.active_turns.lock().unwrap_or_else(|e| e.into_inner());
        set.remove(id);
    }

    fn dir_for(&self, id: &str) -> PathBuf {
        self.chats_dir.join(id)
    }

    fn meta_path(&self, id: &str) -> PathBuf {
        self.dir_for(id).join("meta.json")
    }

    fn transcript_path(&self, id: &str) -> PathBuf {
        self.dir_for(id).join("transcript.jsonl")
    }

    fn write_meta(&self, meta: &ChatMeta) {
        let dir = self.dir_for(&meta.id);
        std::fs::create_dir_all(&dir).ok();
        harden_dir(&dir);
        let path = self.meta_path(&meta.id);
        match serde_json::to_string_pretty(meta) {
            Ok(text) => {
                // Atomic write: write to a temp sibling then rename, so a crash
                // mid-write or a concurrent turn never leaves a torn meta.json
                // that get() would read as None (the chat would transiently
                // vanish from list()). rename(2) is atomic on the same fs.
                let tmp = path.with_extension("json.tmp");
                if let Err(e) = std::fs::write(&tmp, text) {
                    tracing::error!("failed to write {}: {e}", tmp.display());
                } else {
                    harden_file(&tmp);
                    if let Err(e) = std::fs::rename(&tmp, &path) {
                        tracing::error!(
                            "failed to rename {} -> {}: {e}",
                            tmp.display(),
                            path.display()
                        );
                    } else {
                        harden_file(&path);
                    }
                }
            }
            Err(e) => tracing::error!("failed to serialize chat meta {}: {e}", meta.id),
        }
    }

    /// Creates a new chat, persists its metadata, and returns it.
    /// `account_slug` is the account slot this chat's turns should run
    /// under, if the caller chose one at creation (`None` means "resolve
    /// the kind's default account per turn", per routes_chat.rs).
    pub fn create(
        &self,
        agent: ChatAgent,
        cwd: String,
        title: Option<String>,
        account_slug: Option<String>,
    ) -> ChatMeta {
        let now = chrono::Utc::now().to_rfc3339();
        let meta = ChatMeta {
            id: random_hex(8),
            title,
            agent,
            cwd,
            created_at: now.clone(),
            updated_at: now,
            provider_session_id: None,
            account_slug,
        };
        self.write_meta(&meta);
        meta
    }

    /// All known chats, most recently updated first.
    pub fn list(&self) -> Vec<ChatMeta> {
        let mut metas = Vec::new();
        let Ok(entries) = std::fs::read_dir(&self.chats_dir) else {
            return metas;
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_dir() {
                continue;
            }
            let Some(id) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            if let Some(meta) = self.get(&id) {
                metas.push(meta);
            }
        }
        metas.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        metas
    }

    /// Reads one chat's metadata by id.
    pub fn get(&self, id: &str) -> Option<ChatMeta> {
        let text = std::fs::read_to_string(self.meta_path(id)).ok()?;
        match serde_json::from_str(&text) {
            Ok(meta) => Some(meta),
            Err(e) => {
                tracing::warn!("corrupted meta.json for chat {id}: {e}");
                None
            }
        }
    }

    /// Appends a message to the chat's transcript and bumps `updated_at`.
    pub fn append_message(&self, id: &str, msg: &ChatMessage) {
        let dir = self.dir_for(id);
        std::fs::create_dir_all(&dir).ok();
        harden_dir(&dir);
        let path = self.transcript_path(id);
        let Ok(line) = serde_json::to_string(msg) else {
            tracing::error!("failed to serialize chat message for {id}");
            return;
        };
        use std::io::Write;
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path);
        match file {
            Ok(mut f) => {
                if let Err(e) = writeln!(f, "{line}") {
                    tracing::error!("failed to append to {}: {e}", path.display());
                }
                harden_file(&path);
            }
            Err(e) => tracing::error!("failed to open {}: {e}", path.display()),
        }

        if let Some(mut meta) = self.get(id) {
            meta.updated_at = chrono::Utc::now().to_rfc3339();
            self.write_meta(&meta);
        }
    }

    /// The full transcript for a chat, in append order. Empty if the chat is unknown.
    pub fn transcript(&self, id: &str) -> Vec<ChatMessage> {
        let Ok(text) = std::fs::read_to_string(self.transcript_path(id)) else {
            return Vec::new();
        };
        text.lines()
            .filter_map(|line| match serde_json::from_str(line) {
                Ok(msg) => Some(msg),
                Err(e) => {
                    tracing::warn!("skipping corrupted transcript line for chat {id}: {e}");
                    None
                }
            })
            .collect()
    }

    /// Returns up to `limit` messages STRICTLY BEFORE byte offset `before`
    /// (or from end-of-file when `before` is `None`), NEWEST FIRST
    /// (reverse-chronological), reading the transcript file BACKWARD in
    /// bounded-size chunks so the amount read is proportional to `limit`
    /// (roughly `limit` lines' worth of bytes), never to the total file
    /// size — this is the fix for a 10M-token transcript blowing up
    /// memory/latency. The second return value is the cursor for the NEXT
    /// (older) page: the byte offset of the start of the oldest line
    /// returned here, or `None` when the start of the transcript was
    /// reached (nothing older exists). `limit` is clamped server-side
    /// regardless of what the caller asks, so a hostile `limit` cannot
    /// force an unbounded read.
    pub fn tail_page(
        &self,
        id: &str,
        before: Option<u64>,
        limit: usize,
    ) -> (Vec<ChatMessage>, Option<u64>) {
        const MAX_LIMIT: usize = 500;
        const CHUNK_SIZE: u64 = 64 * 1024;
        let limit = limit.min(MAX_LIMIT);

        use std::io::{Read, Seek, SeekFrom};
        let Ok(mut file) = std::fs::File::open(self.transcript_path(id)) else {
            return (Vec::new(), None);
        };
        let Ok(file_len) = file.metadata().map(|m| m.len()) else {
            return (Vec::new(), None);
        };
        let end = before.unwrap_or(file_len).min(file_len);
        if end == 0 || limit == 0 {
            return (Vec::new(), None);
        }

        // Read backward in bounded chunks until at least `limit + 2`
        // newlines have been accumulated, or the start of the file is
        // reached. The `+2` margin (not `+1`) matters: after the buffer's
        // leading partial-line fragment is discarded below, we're still
        // left with at least `limit + 1` real complete lines whenever
        // `pos != 0`, which is what lets `next_cursor` always be resolved
        // from a real line boundary rather than guessed.
        let needed = limit as u64 + 2;
        let mut pos = end;
        let mut newline_count: u64 = 0;
        let mut buf: Vec<u8> = Vec::new();
        while pos > 0 && newline_count < needed {
            let chunk_len = CHUNK_SIZE.min(pos);
            pos -= chunk_len;
            let mut chunk = vec![0u8; chunk_len as usize];
            if file.seek(SeekFrom::Start(pos)).is_err() || file.read_exact(&mut chunk).is_err() {
                break;
            }
            newline_count += chunk.iter().filter(|&&b| b == b'\n').count() as u64;
            chunk.extend_from_slice(&buf);
            buf = chunk;
        }

        // `buf` now holds exactly the bytes `[pos, end)` of the file. `end`
        // is always a line-boundary (either `file_len`, which sits right
        // after the transcript's trailing '\n', or a `before` cursor that
        // was itself returned as a line-start offset by a prior call), so
        // there is never a trailing partial fragment to handle at the
        // `end` side.
        let mut lines: Vec<(u64, &[u8])> = Vec::new();
        let mut line_start = 0usize;
        for (i, &b) in buf.iter().enumerate() {
            if b == b'\n' {
                lines.push((pos + line_start as u64, &buf[line_start..i]));
                line_start = i + 1;
            }
        }
        // If reading didn't reach the start of the file, the FIRST fragment
        // above is a partial line whose true start is earlier than `pos` —
        // discard it. If `pos == 0` every fragment is a real complete line,
        // because the file itself starts there.
        if pos != 0 && !lines.is_empty() {
            lines.remove(0);
        }

        let take = limit.min(lines.len());
        let window = &lines[lines.len() - take..];
        let next_cursor = if lines.len() > take {
            Some(window[0].0)
        } else {
            None
        };

        let mut messages = Vec::with_capacity(take);
        for (_, bytes) in window.iter().rev() {
            // newest first
            let Ok(text) = std::str::from_utf8(bytes) else {
                tracing::warn!("skipping non-utf8 transcript line for chat {id}");
                continue;
            };
            match serde_json::from_str::<ChatMessage>(text) {
                Ok(msg) => messages.push(msg),
                Err(e) => tracing::warn!("skipping corrupted transcript line for chat {id}: {e}"),
            }
        }
        (messages, next_cursor)
    }

    /// Records the provider (claude/codex) resume session id for a chat.
    pub fn set_provider_session(&self, id: &str, provider_session_id: &str) {
        if let Some(mut meta) = self.get(id) {
            meta.provider_session_id = Some(provider_session_id.to_string());
            meta.updated_at = chrono::Utc::now().to_rfc3339();
            self.write_meta(&meta);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_then_get_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(
            ChatAgent::Claude,
            "/tmp/proj".to_string(),
            Some("hi".to_string()),
            None,
        );

        let fetched = store.get(&meta.id).expect("chat should exist");
        assert_eq!(fetched.id, meta.id);
        assert_eq!(fetched.title.as_deref(), Some("hi"));
        assert_eq!(fetched.agent, ChatAgent::Claude);
        assert_eq!(fetched.cwd, "/tmp/proj");
        assert!(fetched.provider_session_id.is_none());
    }

    #[test]
    fn create_persists_account_slug() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(
            ChatAgent::Claude,
            "/tmp".to_string(),
            None,
            Some("work-1".to_string()),
        );
        assert_eq!(meta.account_slug.as_deref(), Some("work-1"));

        let fetched = store.get(&meta.id).expect("chat should exist");
        assert_eq!(fetched.account_slug.as_deref(), Some("work-1"));
    }

    #[test]
    fn create_without_account_slug_is_none() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(ChatAgent::Claude, "/tmp".to_string(), None, None);
        assert!(meta.account_slug.is_none());
    }

    #[test]
    fn old_meta_json_without_account_slug_still_deserializes() {
        // Back-compat: a meta.json written before this field existed.
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let chat_dir = dir.path().join("chats").join("legacy1");
        std::fs::create_dir_all(&chat_dir).unwrap();
        std::fs::write(
            chat_dir.join("meta.json"),
            r#"{"id":"legacy1","title":null,"agent":"claude","cwd":"/tmp","created_at":"t","updated_at":"t","provider_session_id":null}"#,
        )
        .unwrap();

        let meta = store
            .get("legacy1")
            .expect("legacy meta.json should still parse");
        assert!(meta.account_slug.is_none());
    }

    #[test]
    fn get_unknown_id_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        assert!(store.get("nonexistent").is_none());
    }

    #[test]
    fn append_two_messages_returns_both_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(ChatAgent::Codex, "/tmp".to_string(), None, None);

        store.append_message(
            &meta.id,
            &ChatMessage {
                role: "user".to_string(),
                text: "first".to_string(),
                ts: "t1".to_string(),
            },
        );
        store.append_message(
            &meta.id,
            &ChatMessage {
                role: "assistant".to_string(),
                text: "second".to_string(),
                ts: "t2".to_string(),
            },
        );

        let transcript = store.transcript(&meta.id);
        assert_eq!(transcript.len(), 2);
        assert_eq!(transcript[0].text, "first");
        assert_eq!(transcript[1].text, "second");
    }

    #[test]
    fn transcript_of_unknown_chat_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        assert!(store.transcript("nonexistent").is_empty());
    }

    #[test]
    fn create_bumps_list() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        assert!(store.list().is_empty());
        let meta = store.create(ChatAgent::Claude, "/tmp".to_string(), None, None);
        let listed = store.list();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, meta.id);
    }

    #[test]
    fn list_sorted_updated_at_desc() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let a = store.create(ChatAgent::Claude, "/tmp".to_string(), None, None);
        std::thread::sleep(std::time::Duration::from_millis(5));
        let b = store.create(ChatAgent::Claude, "/tmp".to_string(), None, None);
        std::thread::sleep(std::time::Duration::from_millis(5));
        // bump a's updated_at past b's by appending to it
        store.append_message(
            &a.id,
            &ChatMessage {
                role: "user".to_string(),
                text: "bump".to_string(),
                ts: "t".to_string(),
            },
        );

        let listed = store.list();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].id, a.id, "most recently updated chat comes first");
        assert_eq!(listed[1].id, b.id);
    }

    #[test]
    fn try_start_turn_then_second_call_is_rejected_until_end_turn() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(ChatAgent::Claude, "/tmp".to_string(), None, None);

        assert!(
            store.try_start_turn(&meta.id),
            "first call should start the turn"
        );
        assert!(
            !store.try_start_turn(&meta.id),
            "second call while active must be rejected"
        );

        store.end_turn(&meta.id);
        assert!(
            store.try_start_turn(&meta.id),
            "after end_turn, a new turn may start"
        );
    }

    #[test]
    fn end_turn_on_a_never_started_id_is_a_harmless_no_op() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        store.end_turn("never-started"); // must not panic
        assert!(store.try_start_turn("never-started"));
    }

    #[test]
    fn try_start_turn_is_independent_per_chat_id() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let a = store.create(ChatAgent::Claude, "/tmp".to_string(), None, None);
        let b = store.create(ChatAgent::Claude, "/tmp".to_string(), None, None);

        assert!(store.try_start_turn(&a.id));
        assert!(
            store.try_start_turn(&b.id),
            "a different chat id must not be blocked by a's active turn"
        );
    }

    #[test]
    fn set_provider_session_persists() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(ChatAgent::Claude, "/tmp".to_string(), None, None);
        assert!(meta.provider_session_id.is_none());

        store.set_provider_session(&meta.id, "claude-session-abc");

        let fetched = store.get(&meta.id).unwrap();
        assert_eq!(
            fetched.provider_session_id.as_deref(),
            Some("claude-session-abc")
        );
    }

    #[test]
    fn persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let meta = {
            let store = ChatStore::open(dir.path());
            let meta = store.create(
                ChatAgent::Claude,
                "/tmp".to_string(),
                Some("persisted".to_string()),
                None,
            );
            store.append_message(
                &meta.id,
                &ChatMessage {
                    role: "user".to_string(),
                    text: "hello".to_string(),
                    ts: "t1".to_string(),
                },
            );
            meta
        };

        // fresh store over the same dir
        let store2 = ChatStore::open(dir.path());
        let fetched = store2.get(&meta.id).expect("chat should survive reopen");
        assert_eq!(fetched.title.as_deref(), Some("persisted"));
        let transcript = store2.transcript(&meta.id);
        assert_eq!(transcript.len(), 1);
        assert_eq!(transcript[0].text, "hello");
        assert_eq!(store2.list().len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn chat_dir_is_0700_and_meta_is_0600() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(ChatAgent::Claude, "/tmp".to_string(), None, None);

        let chat_dir = dir.path().join("chats").join(&meta.id);
        let dir_mode = std::fs::metadata(&chat_dir).unwrap().permissions().mode() & 0o777;
        assert_eq!(dir_mode, 0o700, "chat dir must be 0700");

        let meta_mode = std::fs::metadata(chat_dir.join("meta.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(meta_mode, 0o600, "meta.json must be 0600");
    }

    #[test]
    fn tail_page_of_empty_transcript_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(ChatAgent::Claude, "/tmp".to_string(), None, None);

        // No messages appended yet -> transcript.jsonl doesn't even exist.
        let (messages, cursor) = store.tail_page(&meta.id, None, 10);
        assert!(messages.is_empty());
        assert!(cursor.is_none());
    }

    #[test]
    fn tail_page_of_unknown_chat_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let (messages, cursor) = store.tail_page("nonexistent", None, 10);
        assert!(messages.is_empty());
        assert!(cursor.is_none());
    }

    #[test]
    fn tail_page_single_message() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(ChatAgent::Claude, "/tmp".to_string(), None, None);
        store.append_message(
            &meta.id,
            &ChatMessage {
                role: "user".to_string(),
                text: "hi".to_string(),
                ts: "t1".to_string(),
            },
        );

        let (messages, cursor) = store.tail_page(&meta.id, None, 10);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].text, "hi");
        assert!(
            cursor.is_none(),
            "one message fits well under the limit -> no next page"
        );
    }

    #[test]
    fn tail_page_exact_limit_boundary_has_no_next_cursor() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(ChatAgent::Claude, "/tmp".to_string(), None, None);
        for i in 0..3 {
            store.append_message(
                &meta.id,
                &ChatMessage {
                    role: "user".to_string(),
                    text: format!("m{i}"),
                    ts: format!("t{i}"),
                },
            );
        }

        let (messages, cursor) = store.tail_page(&meta.id, None, 3);
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].text, "m2", "newest first");
        assert_eq!(messages[1].text, "m1");
        assert_eq!(messages[2].text, "m0");
        assert!(
            cursor.is_none(),
            "transcript has exactly `limit` messages -> no next page"
        );
    }

    #[test]
    fn tail_page_more_than_limit_returns_newest_window_and_a_usable_cursor() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(ChatAgent::Claude, "/tmp".to_string(), None, None);
        for i in 0..5 {
            store.append_message(
                &meta.id,
                &ChatMessage {
                    role: "user".to_string(),
                    text: format!("m{i}"),
                    ts: format!("t{i}"),
                },
            );
        }

        let (messages, cursor) = store.tail_page(&meta.id, None, 3);
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].text, "m4");
        assert_eq!(messages[1].text, "m3");
        assert_eq!(messages[2].text, "m2");
        let cursor = cursor.expect("2 older messages remain -> a next_cursor must be returned");

        let (page2, cursor2) = store.tail_page(&meta.id, Some(cursor), 3);
        assert_eq!(
            page2.len(),
            2,
            "the next older page has exactly the 2 remaining messages, no gap, no dupe"
        );
        assert_eq!(page2[0].text, "m1");
        assert_eq!(page2[1].text, "m0");
        assert!(cursor2.is_none());
    }

    #[test]
    fn tail_page_pages_through_full_transcript_with_no_gaps_or_dupes() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(ChatAgent::Claude, "/tmp".to_string(), None, None);
        let total = 9; // divides evenly into 3 pages of limit=3
        for i in 0..total {
            store.append_message(
                &meta.id,
                &ChatMessage {
                    role: "user".to_string(),
                    text: format!("m{i}"),
                    ts: format!("t{i}"),
                },
            );
        }

        let mut collected_newest_first = Vec::new();
        let mut cursor = None;
        for _ in 0..3 {
            let (page, next) = store.tail_page(&meta.id, cursor, 3);
            assert_eq!(page.len(), 3);
            collected_newest_first.extend(page.into_iter().map(|m| m.text));
            cursor = next;
        }
        assert!(
            cursor.is_none(),
            "the transcript divides evenly into exactly 3 pages of 3"
        );

        let mut chronological = collected_newest_first;
        chronological.reverse();
        let expected: Vec<String> = (0..total).map(|i| format!("m{i}")).collect();
        assert_eq!(
            chronological, expected,
            "3 pages of 3, reversed, must equal the full transcript"
        );
    }

    #[test]
    fn tail_page_skips_corrupted_line_in_the_middle() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(ChatAgent::Claude, "/tmp".to_string(), None, None);
        store.append_message(
            &meta.id,
            &ChatMessage {
                role: "user".to_string(),
                text: "one".to_string(),
                ts: "t1".to_string(),
            },
        );
        {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(store.transcript_path(&meta.id))
                .unwrap();
            writeln!(f, "{{not valid json").unwrap();
        }
        store.append_message(
            &meta.id,
            &ChatMessage {
                role: "assistant".to_string(),
                text: "three".to_string(),
                ts: "t3".to_string(),
            },
        );

        let (messages, cursor) = store.tail_page(&meta.id, None, 10);
        assert_eq!(
            messages.len(),
            2,
            "the corrupted middle line must be skipped, not panic"
        );
        assert_eq!(messages[0].text, "three", "newest first");
        assert_eq!(messages[1].text, "one");
        assert!(cursor.is_none());
    }

    /// Scale/perf proof: `tail_page`'s cost is proportional to `limit`, not
    /// to total file size. Writes ~400k messages directly to the transcript
    /// file (bypassing `append_message`'s per-call fs metadata churn, which
    /// would make this test itself slow) so the file is tens of MB, then
    /// times a single `tail_page(id, None, 20)` call against it and against
    /// a tiny 20-message transcript — both must land in the same rough
    /// latency ballpark, which a linear full-file JSON parse would not.
    #[test]
    fn tail_page_scale_latency_does_not_scale_with_file_size() {
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(ChatAgent::Claude, "/tmp".to_string(), None, None);

        const N: usize = 400_000;
        {
            use std::io::Write;
            let mut writer = std::io::BufWriter::new(
                std::fs::File::create(store.transcript_path(&meta.id)).unwrap(),
            );
            for i in 0..N {
                let msg = ChatMessage {
                    role: "user".to_string(),
                    text: format!("message number {i}"),
                    ts: format!("t{i}"),
                };
                writeln!(writer, "{}", serde_json::to_string(&msg).unwrap()).unwrap();
            }
            writer.flush().unwrap();
        }
        let file_len = std::fs::metadata(store.transcript_path(&meta.id))
            .unwrap()
            .len();
        assert!(
            file_len > 10_000_000,
            "transcript should be tens of MB, got {file_len} bytes"
        );

        let bound = std::time::Duration::from_millis(250);

        let start = std::time::Instant::now();
        let (messages, next_cursor) = store.tail_page(&meta.id, None, 20);
        let elapsed_huge = start.elapsed();
        eprintln!("tail_page on a {file_len}-byte ({N}-message) transcript took {elapsed_huge:?}");

        assert_eq!(messages.len(), 20);
        assert_eq!(
            messages[0].text,
            format!("message number {}", N - 1),
            "newest first"
        );
        assert_eq!(messages[19].text, format!("message number {}", N - 20));
        assert!(next_cursor.is_some());
        assert!(
            elapsed_huge < bound,
            "tail_page on a huge transcript took {elapsed_huge:?}, expected < {bound:?}"
        );

        // A small transcript's tail_page call should land in the same rough
        // ballpark -- proof that latency does not scale with total file size.
        let small = store.create(ChatAgent::Claude, "/tmp".to_string(), None, None);
        for i in 0..20 {
            store.append_message(
                &small.id,
                &ChatMessage {
                    role: "user".to_string(),
                    text: format!("small {i}"),
                    ts: format!("t{i}"),
                },
            );
        }
        let start2 = std::time::Instant::now();
        let (messages2, cursor2) = store.tail_page(&small.id, None, 20);
        let elapsed_small = start2.elapsed();
        eprintln!("tail_page on a 20-message transcript took {elapsed_small:?}");

        assert_eq!(messages2.len(), 20);
        assert!(cursor2.is_none());
        assert!(
            elapsed_small < bound,
            "tail_page on a small transcript took {elapsed_small:?}, expected < {bound:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn transcript_file_is_0600() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let store = ChatStore::open(dir.path());
        let meta = store.create(ChatAgent::Claude, "/tmp".to_string(), None, None);
        store.append_message(
            &meta.id,
            &ChatMessage {
                role: "user".to_string(),
                text: "hi".to_string(),
                ts: "t".to_string(),
            },
        );

        let transcript_path = dir
            .path()
            .join("chats")
            .join(&meta.id)
            .join("transcript.jsonl");
        let mode = std::fs::metadata(&transcript_path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "transcript.jsonl must be 0600");
    }
}
