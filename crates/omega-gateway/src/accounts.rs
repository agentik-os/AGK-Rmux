//! Isolated per-account credential slots + metadata registry.
//!
//! Storage layout under the gateway data dir:
//! `<gateway_dir>/accounts/accounts.json`  (0600, registry: `Vec<Account>` — METADATA only)
//! `<gateway_dir>/accounts/<slug>/`        (0700, one hardened dir per slot — the
//!                                          credential material a provider CLI writes
//!                                          there is opaque to this module and never
//!                                          touched or serialized by it)

use crate::fsperm::{harden_dir, harden_file};
use crate::protocol::{Account, AccountKind};
use crate::util::random_hex;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Slug charset: `[a-z0-9-]{1,32}`, non-empty. Deliberately excludes `.`/`/`
/// so a slug can never traverse out of the accounts dir (e.g. `../x`).
pub fn valid_slug(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Handle over `<gateway_dir>/accounts`: every read/write goes straight to
/// disk (no in-memory cache), so cloning (cheap: one `PathBuf` plus one
/// `Arc`) is safe to share across async tasks (e.g. `AppState`,
/// `spawn_blocking` closures). `lock` guards the read-modify-write sequence
/// in `create_slot` / `remove` / `set_default`: those three read the whole
/// registry, mutate it in memory, then write it back, so two concurrent
/// mutators (e.g. two `POST /v1/accounts` for different slugs) could
/// otherwise both read before either writes and silently drop one write
/// (I-6). It is wrapped in `Arc` deliberately, not a bare field: this
/// struct derives `Clone` and is cloned into every handler closure, and a
/// bare `Mutex<()>` field would give each clone its own independent lock,
/// defeating the whole point, so every clone must share the SAME mutex
/// instance.
#[derive(Clone)]
pub struct AccountStore {
    accounts_dir: PathBuf,
    lock: Arc<Mutex<()>>,
}

impl AccountStore {
    /// Opens (creating if needed) the account store rooted at `<gateway_dir>/accounts`.
    pub fn open(gateway_dir: &Path) -> Self {
        let accounts_dir = gateway_dir.join("accounts");
        std::fs::create_dir_all(&accounts_dir).ok();
        harden_dir(&accounts_dir);
        Self {
            accounts_dir,
            lock: Arc::new(Mutex::new(())),
        }
    }

    fn registry_path(&self) -> PathBuf {
        self.accounts_dir.join("accounts.json")
    }

    /// The hardened slot directory for `slug` — where a provider CLI's
    /// credential material for that slot lives. This module never reads or
    /// writes inside it beyond creating/removing the directory itself.
    pub fn slot_dir(&self, slug: &str) -> PathBuf {
        self.accounts_dir.join(slug)
    }

    /// Reads and parses the registry. A MISSING file is normal first-run and
    /// silently returns empty. A file that exists but fails to PARSE is a
    /// different case entirely: returning empty here without quarantining it
    /// would let the next `write_registry` overwrite the bad file with an
    /// empty list, silently orphaning every account slot on disk. So a parse
    /// failure instead quarantines the bad bytes (see
    /// `quarantine_corrupt_registry`) before returning empty.
    fn read_registry(&self) -> Vec<Account> {
        let path = self.registry_path();
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Vec::new();
        };
        match serde_json::from_str(&text) {
            Ok(list) => list,
            Err(e) => {
                tracing::error!(
                    "corrupted accounts.json ({e}); quarantining instead of overwriting"
                );
                self.quarantine_corrupt_registry(&path);
                Vec::new()
            }
        }
    }

    /// Renames an unparseable `accounts.json` out of the way so it is never
    /// silently clobbered by the next write: to `accounts.json.corrupt`, or
    /// `.corrupt.1`, `.corrupt.2`, ... if that name is already taken (e.g. a
    /// prior quarantine from an earlier corruption).
    fn quarantine_corrupt_registry(&self, path: &Path) {
        let mut dest = self.accounts_dir.join("accounts.json.corrupt");
        let mut n = 1u32;
        while dest.exists() {
            dest = self.accounts_dir.join(format!("accounts.json.corrupt.{n}"));
            n += 1;
        }
        if let Err(e) = std::fs::rename(path, &dest) {
            tracing::error!("failed to quarantine corrupt {}: {e}", path.display());
        }
    }

    fn write_registry(&self, accounts: &[Account]) {
        let path = self.registry_path();
        match serde_json::to_string_pretty(accounts) {
            Ok(text) => {
                // Atomic write: unique temp sibling + rename, same discipline
                // as chat_store's meta.json (temp file, then atomic rename)
                // plus a per-write random suffix as defense-in-depth on top
                // of `lock`: `lock` serializes mutators WITHIN this process,
                // but cannot help two separate gateway PROCESSES sharing the
                // same accounts dir, and a unique temp filename means one
                // process's in-flight write can never collide with
                // another's on the same fixed `.json.tmp` path.
                let tmp = path.with_extension(format!("json.tmp.{}", random_hex(4)));
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
            Err(e) => tracing::error!("failed to serialize accounts registry: {e}"),
        }
    }

    /// Creates a new isolated credential slot: validates the slug, makes its
    /// hardened (0700) directory, and persists the metadata registry. The
    /// first account created for a given `kind` becomes that kind's default.
    pub fn create_slot(&self, slug: &str, label: &str, kind: AccountKind) -> Result<Account> {
        if !valid_slug(slug) {
            bail!("invalid slug: {slug:?} (must be [a-z0-9-]{{1,32}})");
        }
        // Held across the whole read-modify-write below: two concurrent
        // `create_slot` calls (e.g. two `POST /v1/accounts` for different
        // slugs) must not both read the registry before either writes, or
        // one of them silently vanishes from the persisted list (I-6).
        let _guard = self.lock.lock().unwrap();
        let mut accounts = self.read_registry();
        if accounts.iter().any(|a| a.slug == slug) {
            bail!("account slug already exists: {slug}");
        }
        let is_default = !accounts.iter().any(|a| a.kind == kind);

        let dir = self.slot_dir(slug);
        std::fs::create_dir_all(&dir)?;
        harden_dir(&dir);

        let account = Account {
            slug: slug.to_string(),
            label: label.to_string(),
            kind,
            created_at: chrono::Utc::now().to_rfc3339(),
            is_default,
        };
        accounts.push(account.clone());
        self.write_registry(&accounts);
        Ok(account)
    }

    /// All known accounts (any kind), in registry order.
    pub fn list(&self) -> Vec<Account> {
        self.read_registry()
    }

    /// Reads one account's metadata by slug.
    pub fn get(&self, slug: &str) -> Option<Account> {
        self.read_registry().into_iter().find(|a| a.slug == slug)
    }

    /// Removes an account: deletes its registry entry AND its slot directory
    /// (credentials and all). Returns false if the slug was unknown.
    pub fn remove(&self, slug: &str) -> bool {
        // Same RMW race as `create_slot`, guarded the same way (I-6).
        let _guard = self.lock.lock().unwrap();
        let mut accounts = self.read_registry();
        let before = accounts.len();
        accounts.retain(|a| a.slug != slug);
        if accounts.len() == before {
            return false;
        }
        self.write_registry(&accounts);
        if let Err(e) = std::fs::remove_dir_all(self.slot_dir(slug)) {
            tracing::warn!("failed to remove slot dir for {slug}: {e}");
        }
        true
    }

    /// Makes `slug` the default for its kind, clearing the default flag on
    /// every other account of the SAME kind (accounts of other kinds are
    /// untouched). Returns false if the slug is unknown.
    pub fn set_default(&self, slug: &str) -> bool {
        // Same RMW race as `create_slot`, guarded the same way (I-6).
        let _guard = self.lock.lock().unwrap();
        let mut accounts = self.read_registry();
        let Some(kind) = accounts.iter().find(|a| a.slug == slug).map(|a| a.kind) else {
            return false;
        };
        for a in accounts.iter_mut().filter(|a| a.kind == kind) {
            a.is_default = a.slug == slug;
        }
        self.write_registry(&accounts);
        true
    }

    /// The current default account for `kind`, if any.
    pub fn default_for(&self, kind: AccountKind) -> Option<Account> {
        self.read_registry()
            .into_iter()
            .find(|a| a.kind == kind && a.is_default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_slug_rejects_bad_input() {
        assert!(!valid_slug("../x"), "path traversal must be rejected");
        assert!(!valid_slug("UP"), "uppercase must be rejected");
        assert!(!valid_slug(""), "empty must be rejected");
        assert!(!valid_slug(&"a".repeat(33)), ">32 chars must be rejected");
    }

    #[test]
    fn valid_slug_accepts_good_input() {
        assert!(valid_slug("work-1"));
        assert!(valid_slug("a"));
        assert!(valid_slug(&"a".repeat(32)));
    }

    #[test]
    fn create_slot_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        let account = store
            .create_slot("work-1", "Work account", AccountKind::Claude)
            .unwrap();

        assert_eq!(account.slug, "work-1");
        assert_eq!(account.label, "Work account");
        assert_eq!(account.kind, AccountKind::Claude);
        assert!(account.is_default, "first account of a kind is its default");

        let fetched = store.get("work-1").expect("account should exist");
        assert_eq!(fetched.slug, account.slug);
        assert_eq!(fetched.label, account.label);
    }

    #[test]
    fn create_slot_rejects_invalid_slug() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        assert!(store
            .create_slot("../x", "bad", AccountKind::Claude)
            .is_err());
        assert!(store.create_slot("UP", "bad", AccountKind::Claude).is_err());
    }

    #[test]
    fn create_slot_rejects_duplicate_slug() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        store
            .create_slot("work-1", "Work", AccountKind::Claude)
            .unwrap();
        assert!(store
            .create_slot("work-1", "Again", AccountKind::Claude)
            .is_err());
    }

    #[test]
    fn get_unknown_slug_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        assert!(store.get("nonexistent").is_none());
    }

    #[test]
    fn list_returns_all_accounts() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        assert!(store.list().is_empty());
        store.create_slot("a", "A", AccountKind::Claude).unwrap();
        store.create_slot("b", "B", AccountKind::Codex).unwrap();
        let listed = store.list();
        assert_eq!(listed.len(), 2);
    }

    #[test]
    fn first_of_kind_is_default_independently_per_kind() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        let claude1 = store
            .create_slot("c1", "Claude 1", AccountKind::Claude)
            .unwrap();
        assert!(claude1.is_default);

        let codex1 = store
            .create_slot("x1", "Codex 1", AccountKind::Codex)
            .unwrap();
        assert!(
            codex1.is_default,
            "first Codex account is its own kind's default"
        );

        let claude2 = store
            .create_slot("c2", "Claude 2", AccountKind::Claude)
            .unwrap();
        assert!(!claude2.is_default, "second Claude account is not default");
    }

    #[test]
    fn set_default_moves_default_within_kind_other_kind_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        store
            .create_slot("c1", "Claude 1", AccountKind::Claude)
            .unwrap();
        store
            .create_slot("c2", "Claude 2", AccountKind::Claude)
            .unwrap();
        let codex1 = store
            .create_slot("x1", "Codex 1", AccountKind::Codex)
            .unwrap();

        assert!(store.set_default("c2"));

        let c1 = store.get("c1").unwrap();
        let c2 = store.get("c2").unwrap();
        let x1 = store.get("x1").unwrap();
        assert!(!c1.is_default, "loser is cleared");
        assert!(c2.is_default, "winner is set");
        assert!(x1.is_default, "other kind's default untouched");
        assert_eq!(codex1.slug, x1.slug);
    }

    #[test]
    fn set_default_unknown_slug_returns_false() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        assert!(!store.set_default("nonexistent"));
    }

    #[test]
    fn default_for_returns_current_default_per_kind() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        assert!(store.default_for(AccountKind::Claude).is_none());
        store
            .create_slot("c1", "Claude 1", AccountKind::Claude)
            .unwrap();
        store
            .create_slot("c2", "Claude 2", AccountKind::Claude)
            .unwrap();
        store.set_default("c2");
        assert_eq!(store.default_for(AccountKind::Claude).unwrap().slug, "c2");
    }

    #[test]
    fn remove_deletes_dir_and_registry_entry() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        store
            .create_slot("work-1", "Work", AccountKind::Claude)
            .unwrap();
        let slot_dir = store.slot_dir("work-1");
        assert!(slot_dir.exists());

        assert!(store.remove("work-1"));
        assert!(store.get("work-1").is_none());
        assert!(!slot_dir.exists(), "slot dir must be deleted");
    }

    #[test]
    fn remove_unknown_slug_returns_false() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        assert!(!store.remove("nonexistent"));
    }

    #[test]
    fn persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        {
            let store = AccountStore::open(dir.path());
            store
                .create_slot("work-1", "Work", AccountKind::Claude)
                .unwrap();
            store
                .create_slot("work-2", "Work 2", AccountKind::Claude)
                .unwrap();
            store.set_default("work-2");
        }

        let store2 = AccountStore::open(dir.path());
        assert_eq!(store2.list().len(), 2);
        assert_eq!(
            store2.default_for(AccountKind::Claude).unwrap().slug,
            "work-2"
        );
    }

    #[cfg(unix)]
    #[test]
    fn slot_dir_is_0700_and_registry_is_0600() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        store
            .create_slot("work-1", "Work", AccountKind::Claude)
            .unwrap();

        let slot_dir = dir.path().join("accounts").join("work-1");
        let dir_mode = std::fs::metadata(&slot_dir).unwrap().permissions().mode() & 0o777;
        assert_eq!(dir_mode, 0o700, "slot dir must be 0700");

        let registry_path = dir.path().join("accounts").join("accounts.json");
        let registry_mode = std::fs::metadata(&registry_path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(registry_mode, 0o600, "accounts.json must be 0600");
    }

    #[test]
    fn corrupt_registry_is_quarantined_not_silently_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        let registry_path = dir.path().join("accounts").join("accounts.json");
        let bad_bytes = b"{ this is not valid json at all";
        std::fs::write(&registry_path, bad_bytes).unwrap();

        // (a) reading a corrupt registry returns empty/usable rather than
        // panicking or propagating the parse error.
        let listed = store.list();
        assert!(
            listed.is_empty(),
            "a corrupt registry must read back as an empty, usable list"
        );

        // (b) the original bad bytes now live at accounts.json.corrupt,
        // untouched, instead of being clobbered by the next write.
        let corrupt_path = dir.path().join("accounts").join("accounts.json.corrupt");
        assert!(
            corrupt_path.exists(),
            "the corrupt file must be quarantined"
        );
        assert_eq!(
            std::fs::read(&corrupt_path).unwrap(),
            bad_bytes,
            "quarantined bytes must be untouched"
        );
        assert!(
            !registry_path.exists(),
            "the corrupt path is vacated by the rename"
        );

        // The store keeps working normally afterward (a fresh write does not
        // collide with the quarantined file).
        store
            .create_slot("work-1", "Work", AccountKind::Claude)
            .unwrap();
        assert_eq!(store.list().len(), 1);
    }

    /// I-6 regression: two threads calling `create_slot` concurrently, for
    /// DIFFERENT slugs, must never lose either write. Pre-fix, `create_slot`
    /// does a plain read-modify-write with no lock: thread A reads the
    /// registry, thread B reads the same (still missing A's entry), A
    /// writes, then B writes its own version, silently erasing A's entry
    /// from `accounts.json` (A's slot directory still exists on disk,
    /// orphaned and invisible to `list()`). A per-iteration `Barrier`
    /// forces both threads to enter `create_slot` at (as close to) the same
    /// instant on every round, which is what makes an otherwise
    /// timing-sensitive race reproduce reliably across many iterations
    /// instead of depending on luck within a single run.
    #[test]
    fn concurrent_create_slot_never_loses_an_account() {
        let dir = tempfile::tempdir().unwrap();
        let store = AccountStore::open(dir.path());
        let iterations = 150usize;
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));

        let store_a = store.clone();
        let barrier_a = barrier.clone();
        let handle_a = std::thread::spawn(move || {
            for i in 0..iterations {
                barrier_a.wait();
                store_a
                    .create_slot(&format!("race-a-{i}"), "A", AccountKind::Claude)
                    .unwrap();
            }
        });

        let store_b = store.clone();
        let barrier_b = barrier.clone();
        let handle_b = std::thread::spawn(move || {
            for i in 0..iterations {
                barrier_b.wait();
                store_b
                    .create_slot(&format!("race-b-{i}"), "B", AccountKind::Codex)
                    .unwrap();
            }
        });

        handle_a.join().unwrap();
        handle_b.join().unwrap();

        let listed = store.list();
        for i in 0..iterations {
            assert!(
                listed.iter().any(|a| a.slug == format!("race-a-{i}")),
                "lost race-a-{i}: concurrent create_slot dropped an account (I-6)"
            );
            assert!(
                listed.iter().any(|a| a.slug == format!("race-b-{i}")),
                "lost race-b-{i}: concurrent create_slot dropped an account (I-6)"
            );
        }
        assert_eq!(
            listed.len(),
            iterations * 2,
            "registry must contain every created account, no lost updates"
        );
    }

    #[test]
    fn second_corrupt_registry_gets_a_numeric_suffix() {
        let dir = tempfile::tempdir().unwrap();
        let accounts_dir = dir.path().join("accounts");
        std::fs::create_dir_all(&accounts_dir).unwrap();
        // Pre-seed an existing quarantine file, as if an earlier corruption
        // was already quarantined.
        std::fs::write(
            accounts_dir.join("accounts.json.corrupt"),
            b"first corruption",
        )
        .unwrap();

        let store = AccountStore::open(dir.path());
        let registry_path = accounts_dir.join("accounts.json");
        std::fs::write(&registry_path, b"{ second corruption").unwrap();

        store.list();

        assert_eq!(
            std::fs::read(accounts_dir.join("accounts.json.corrupt")).unwrap(),
            b"first corruption",
            "the pre-existing quarantine file must not be clobbered"
        );
        assert_eq!(
            std::fs::read(accounts_dir.join("accounts.json.corrupt.1")).unwrap(),
            b"{ second corruption",
            "the newer corruption gets a numeric suffix instead of overwriting"
        );
    }
}
