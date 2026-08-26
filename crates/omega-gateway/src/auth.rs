use crate::fsperm::{harden_dir, harden_file};
use crate::util::random_hex;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub token_sha256: String,
    pub created_at: String,
    pub revoked: bool,
}

pub struct DeviceStore {
    path: PathBuf,
    devices: Vec<Device>,
}

pub fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    hex_string(&h.finalize())
}

fn hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

impl DeviceStore {
    pub fn open(dir: &Path) -> Self {
        std::fs::create_dir_all(dir).ok();
        harden_dir(dir);
        let path = dir.join("devices.json");
        let devices = match std::fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
                tracing::warn!(
                    "corrupted {}: {e}; starting with empty device list",
                    path.display()
                );
                Vec::new()
            }),
            Err(_) => Vec::new(),
        };
        Self { path, devices }
    }

    pub fn issue(&mut self, name: &str) -> (Device, String) {
        let token = random_hex(32);
        let device = Device {
            id: random_hex(8),
            name: name.to_string(),
            token_sha256: sha256_hex(&token),
            created_at: chrono::Utc::now().to_rfc3339(),
            revoked: false,
        };
        self.devices.push(device.clone());
        self.save();
        (device, token)
    }

    pub fn list(&self) -> &[Device] {
        &self.devices
    }

    pub fn verify(&self, token: &str) -> Option<Device> {
        let hash = sha256_hex(token);
        self.devices
            .iter()
            .find(|d| !d.revoked && d.token_sha256 == hash)
            .cloned()
    }

    pub fn revoke(&mut self, device_id: &str) -> bool {
        let mut hit = false;
        for d in self.devices.iter_mut() {
            if d.id == device_id {
                d.revoked = true;
                hit = true;
            }
        }
        if hit {
            self.save();
        }
        hit
    }

    fn save(&self) {
        match serde_json::to_string_pretty(&self.devices) {
            Ok(text) => {
                if let Err(e) = std::fs::write(&self.path, text) {
                    tracing::error!("failed to write {}: {e}", self.path.display());
                } else {
                    harden_file(&self.path);
                }
            }
            Err(e) => tracing::error!("failed to serialize devices: {e}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingCode {
    pub code: String,
    pub expires_at: String,
}

impl PairingCode {
    pub fn create(dir: &Path, ttl_secs: i64) -> anyhow::Result<Self> {
        std::fs::create_dir_all(dir).ok();
        harden_dir(dir);
        let pc = Self {
            code: random_hex(4), // 8 hex chars, human-typable
            expires_at: (chrono::Utc::now() + chrono::Duration::seconds(ttl_secs)).to_rfc3339(),
        };
        let path = dir.join("pairing.json");
        std::fs::write(&path, serde_json::to_string(&pc)?).context("write pairing.json")?;
        harden_file(&path);
        Ok(pc)
    }

    /// Returns true exactly once for a live matching code.
    /// The file removal is the atomic claim: for concurrent calls with the
    /// same valid code, remove_file succeeds for exactly one of them.
    pub fn consume(dir: &Path, code: &str) -> bool {
        let path = dir.join("pairing.json");
        let Ok(text) = std::fs::read_to_string(&path) else {
            return false;
        };
        let Ok(pc) = serde_json::from_str::<PairingCode>(&text) else {
            return false;
        };
        let live = pc.code == code
            && chrono::DateTime::parse_from_rfc3339(&pc.expires_at)
                .map(|t| t > chrono::Utc::now())
                .unwrap_or(false);
        if !live {
            return false;
        }
        std::fs::remove_file(&path).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_then_verify_roundtrip_and_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = DeviceStore::open(dir.path());
        let (device, token) = store.issue("iphone");
        assert_eq!(token.len(), 64); // 32 bytes hex
        assert_eq!(device.token_sha256, sha256_hex(&token));
        // reopen from disk: token still verifies
        let store2 = DeviceStore::open(dir.path());
        assert_eq!(store2.verify(&token).unwrap().name, "iphone");
        assert!(store2.verify("wrong-token").is_none());
    }

    #[test]
    fn revoked_device_fails_verify() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = DeviceStore::open(dir.path());
        let (device, token) = store.issue("mac");
        assert!(store.revoke(&device.id));
        assert!(store.verify(&token).is_none());
        assert!(!store.revoke("no-such-id"));
    }

    #[test]
    fn corrupted_devices_json_yields_empty_store_not_panic() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("devices.json"), "{not json").unwrap();
        let store = DeviceStore::open(dir.path());
        assert!(store.verify("anything").is_none());
    }

    #[test]
    fn wrong_code_does_not_burn_pairing_file() {
        let dir = tempfile::tempdir().unwrap();
        let pc = PairingCode::create(dir.path(), 300).unwrap();
        assert!(!PairingCode::consume(dir.path(), "deadbeef"));
        assert!(PairingCode::consume(dir.path(), &pc.code));
        assert!(!PairingCode::consume(dir.path(), &pc.code));
    }

    #[cfg(unix)]
    #[test]
    fn credential_files_are_not_group_or_world_readable() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let mut store = DeviceStore::open(dir.path());
        store.issue("iphone");

        let dir_mode = std::fs::metadata(dir.path()).unwrap().permissions().mode() & 0o777;
        assert_eq!(dir_mode, 0o700, "gateway dir must be 0700");

        let file_mode = std::fs::metadata(dir.path().join("devices.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(file_mode, 0o600, "devices.json must be 0600");
    }
}
