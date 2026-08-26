//! Persisted session-organization overlay: purely client-organizational
//! metadata (a folder, a custom label, a pinned flag) the gateway attaches
//! to an rmux session NAME, independent of whether that session currently
//! exists. This store never validates a session name against a live rmux
//! session — the overlay can freely reference a session that has since
//! closed, that's expected: it's a label store, not a session registry.
//!
//! Storage layout under the gateway data dir:
//! `<gateway_dir>/session_org.json` (0600, a single JSON object mapping
//! session name -> [`crate::protocol::SessionOrgEntry`]).
//!
//! Atomic-write idiom mirrors `chat_store.rs`'s `write_meta` EXACTLY: write
//! to a `.json.tmp` sibling, `harden_file` it, `std::fs::rename` (atomic on
//! the same fs) onto the real path, `harden_file` again on the final path —
//! so a crash mid-write or a concurrent write never leaves a torn
//! `session_org.json` that a reader would parse as corrupted or partial.

use crate::fsperm::{harden_dir, harden_file};
use crate::protocol::SessionOrgEntry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct SessionOrgStore {
    path: PathBuf,
    /// Guards the read-modify-write sequence in `set`: the file is small
    /// and rewritten WHOLESALE on every write (no in-memory cache, no
    /// per-key locking), so two concurrent `set` calls -- on the same key or
    /// different keys -- must not interleave their read of the whole map
    /// with each other's write, or one caller's upsert would silently
    /// vanish. `rename`'s atomicity protects readers from ever observing a
    /// torn file; this mutex protects writers from clobbering each other.
    lock: Mutex<()>,
}

impl SessionOrgStore {
    /// Opens the store rooted at `<gateway_dir>/session_org.json`. Mirrors
    /// `auth.rs::DeviceStore::open`: hardens `gateway_dir` itself to 0700
    /// defensively (it is normally already hardened via that same call path
    /// on every authenticated request, but a store should never assume
    /// another store did its job first).
    pub fn open(gateway_dir: &Path) -> Self {
        std::fs::create_dir_all(gateway_dir).ok();
        harden_dir(gateway_dir);
        let path = gateway_dir.join("session_org.json");
        Self {
            path,
            lock: Mutex::new(()),
        }
    }

    fn read_all(&self) -> HashMap<String, SessionOrgEntry> {
        let Ok(text) = std::fs::read_to_string(&self.path) else {
            return HashMap::new();
        };
        match serde_json::from_str(&text) {
            Ok(map) => map,
            Err(e) => {
                tracing::warn!(
                    "corrupted {}: {e}; treating as empty overlay",
                    self.path.display()
                );
                HashMap::new()
            }
        }
    }

    fn write_all(&self, map: &HashMap<String, SessionOrgEntry>) {
        match serde_json::to_string_pretty(map) {
            Ok(text) => {
                // Atomic write: same idiom as chat_store.rs::write_meta.
                let tmp = self.path.with_extension("json.tmp");
                if let Err(e) = std::fs::write(&tmp, text) {
                    tracing::error!("failed to write {}: {e}", tmp.display());
                } else {
                    harden_file(&tmp);
                    if let Err(e) = std::fs::rename(&tmp, &self.path) {
                        tracing::error!(
                            "failed to rename {} -> {}: {e}",
                            tmp.display(),
                            self.path.display()
                        );
                    } else {
                        harden_file(&self.path);
                    }
                }
            }
            Err(e) => tracing::error!("failed to serialize session_org.json: {e}"),
        }
    }

    /// The whole overlay map. Empty (never a panic) when the file doesn't
    /// exist yet -- e.g. a fresh gateway that has never received a
    /// `PUT /v1/session-org/{name}`.
    pub fn get_all(&self) -> HashMap<String, SessionOrgEntry> {
        let _guard = self.lock.lock().unwrap();
        self.read_all()
    }

    /// Upserts one session's overlay entry (a FULL REPLACE of that key --
    /// the caller, `routes_session_org::set`, already resolved PUT's
    /// full-replace-vs-merge semantics before calling this), then
    /// atomic-writes the whole map back. Read-modify-write, so a second
    /// `set` on a DIFFERENT key must not lose the first key's entry -- that
    /// is exactly what `lock` exists to guarantee under concurrent callers.
    pub fn set(&self, session_name: &str, entry: SessionOrgEntry) {
        let _guard = self.lock.lock().unwrap();
        let mut map = self.read_all();
        map.insert(session_name.to_string(), entry);
        self.write_all(&map);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(label: Option<&str>, folder: Option<&str>, pinned: bool) -> SessionOrgEntry {
        SessionOrgEntry {
            label: label.map(str::to_string),
            folder: folder.map(str::to_string),
            pinned,
        }
    }

    #[test]
    fn get_all_on_missing_file_is_empty_no_panic() {
        let dir = tempfile::tempdir().unwrap();
        let store = SessionOrgStore::open(dir.path());
        assert!(store.get_all().is_empty());
    }

    #[test]
    fn set_then_get_all_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        let store = SessionOrgStore::open(dir.path());
        store.set("oracle-Foo-1", entry(Some("Sprint 3"), Some("/work"), true));

        let all = store.get_all();
        assert_eq!(all.len(), 1);
        let got = &all["oracle-Foo-1"];
        assert_eq!(got.label.as_deref(), Some("Sprint 3"));
        assert_eq!(got.folder.as_deref(), Some("/work"));
        assert!(got.pinned);
    }

    #[test]
    fn second_set_on_a_different_key_preserves_the_first() {
        let dir = tempfile::tempdir().unwrap();
        let store = SessionOrgStore::open(dir.path());
        store.set("oracle-Foo-1", entry(Some("first"), None, false));
        store.set("oracle-Bar-2", entry(Some("second"), None, true));

        let all = store.get_all();
        assert_eq!(
            all.len(),
            2,
            "a read-modify-write over the whole map, not an overwrite of it"
        );
        assert_eq!(all["oracle-Foo-1"].label.as_deref(), Some("first"));
        assert_eq!(all["oracle-Bar-2"].label.as_deref(), Some("second"));
        assert!(all["oracle-Bar-2"].pinned);
    }

    #[test]
    fn set_on_the_same_key_replaces_it() {
        let dir = tempfile::tempdir().unwrap();
        let store = SessionOrgStore::open(dir.path());
        store.set("oracle-Foo-1", entry(Some("old"), Some("/a"), false));
        store.set("oracle-Foo-1", entry(Some("new"), None, true));

        let all = store.get_all();
        assert_eq!(all.len(), 1);
        let got = &all["oracle-Foo-1"];
        assert_eq!(got.label.as_deref(), Some("new"));
        assert_eq!(
            got.folder, None,
            "full replace: an omitted field must not survive from the old entry"
        );
        assert!(got.pinned);
    }

    #[test]
    fn persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        {
            let store = SessionOrgStore::open(dir.path());
            store.set("oracle-Foo-1", entry(Some("persisted"), None, false));
        }
        let store2 = SessionOrgStore::open(dir.path());
        let all = store2.get_all();
        assert_eq!(all["oracle-Foo-1"].label.as_deref(), Some("persisted"));
    }

    #[cfg(unix)]
    #[test]
    fn session_org_json_is_0600_after_a_write() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let store = SessionOrgStore::open(dir.path());
        store.set("oracle-Foo-1", entry(None, None, false));

        let path = dir.path().join("session_org.json");
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "session_org.json must be 0600");
    }

    #[cfg(unix)]
    #[test]
    fn gateway_dir_is_0700_after_open() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let _store = SessionOrgStore::open(dir.path());
        let mode = std::fs::metadata(dir.path()).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o700, "gateway dir must be 0700");
    }
}
