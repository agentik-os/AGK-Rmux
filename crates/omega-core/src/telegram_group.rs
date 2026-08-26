//! OmegaOS Telegram supergroup + per-project topics.
//!
//! When a supergroup is set up via `/setupgroup <id>` from the bot, the
//! config is stored here. The bridge then auto-creates a forum topic per
//! project on first dispatch, maps `project_name → message_thread_id`,
//! and routes that project's oracle reports (PDF + text) to its topic.
//!
//! Storage: `~/.omega/telegram-groups.json`, shared with the Telegram bot and
//! operational alert scripts. The former `telegram-group.toml` remains a
//! read-only fallback for installations that have not migrated yet.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct TelegramGroupConfig {
    /// Supergroup chat_id (negative integer, e.g. -1001234567890).
    pub group_id: i64,
    /// Human-friendly group name (cached from getChat for the bridge's
    /// status display — not required for routing).
    #[serde(default)]
    pub group_name: String,
    /// project_name → message_thread_id (Telegram forum topic id).
    #[serde(default)]
    pub topics: BTreeMap<String, i64>,
    /// ISO timestamp of last successful setup.
    #[serde(default)]
    pub setup_at: String,
}

impl TelegramGroupConfig {
    pub fn path() -> PathBuf {
        crate::config::omega_dir().join("telegram-groups.json")
    }

    fn legacy_path() -> PathBuf {
        crate::config::omega_dir().join("telegram-group.toml")
    }

    fn from_storage(root: &JsonMap<String, JsonValue>, path: &Path) -> Result<Option<Self>> {
        for field in ["isForum"] {
            if let Some(value) = root.get(field) {
                anyhow::ensure!(
                    value.is_boolean(),
                    "Telegram group state {} field {field} must be boolean",
                    path.display()
                );
            }
        }
        for field in ["atlas_topic", "alerts_topic"] {
            if let Some(value) = root.get(field) {
                anyhow::ensure!(
                    value.as_i64().is_some_and(|value| value > 0),
                    "Telegram group state {} field {field} must be a positive integer",
                    path.display()
                );
            }
        }
        for field in ["group_name", "setup_at"] {
            if let Some(value) = root.get(field) {
                anyhow::ensure!(
                    value.is_string(),
                    "Telegram group state {} field {field} must be a string",
                    path.display()
                );
            }
        }

        let mut topics = BTreeMap::new();
        let mut topic_owners = BTreeSet::new();
        if let Some(value) = root.get("topics") {
            let stored = value.as_object().with_context(|| {
                format!(
                    "Telegram group state {} field topics must be an object",
                    path.display()
                )
            })?;
            for (topic, project) in stored {
                let topic_id = topic
                    .parse::<i64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .with_context(|| {
                        format!(
                            "Telegram group state {} has invalid topic id {topic:?}",
                            path.display()
                        )
                    })?;
                let project = project
                    .as_str()
                    .filter(|project| !project.is_empty())
                    .with_context(|| {
                        format!(
                            "Telegram group state {} topic {topic} has an invalid project name",
                            path.display()
                        )
                    })?;
                anyhow::ensure!(
                    topic_owners.insert(project.to_ascii_lowercase()),
                    "Telegram group state {} maps topic owner {project:?} more than once",
                    path.display()
                );
                if !matches!(project.to_ascii_lowercase().as_str(), "atlas" | "alerts") {
                    anyhow::ensure!(
                        topics.insert(project.to_string(), topic_id).is_none(),
                        "Telegram group state {} maps project {project:?} more than once",
                        path.display()
                    );
                }
            }
        }

        let Some(group_id) = root.get("hub") else {
            return Ok(None);
        };
        let group_id = group_id.as_i64().with_context(|| {
            format!(
                "Telegram group state {} field hub must be an integer",
                path.display()
            )
        })?;
        anyhow::ensure!(
            group_id < 0,
            "Telegram forum group id must be negative, got {group_id}"
        );
        Ok(Some(Self {
            group_id,
            group_name: root
                .get("group_name")
                .and_then(JsonValue::as_str)
                .unwrap_or_default()
                .to_string(),
            topics,
            setup_at: root
                .get("setup_at")
                .and_then(JsonValue::as_str)
                .unwrap_or_default()
                .to_string(),
        }))
    }

    fn load_storage(path: &Path) -> Result<Option<JsonMap<String, JsonValue>>> {
        let Some(bytes) = crate::config::read_private_optional(path)? else {
            return Ok(None);
        };
        let value: JsonValue = serde_json::from_slice(&bytes).with_context(|| {
            format!("refusing malformed Telegram group state {}", path.display())
        })?;
        let root = value.as_object().cloned().with_context(|| {
            format!(
                "Telegram group state {} must be a JSON object",
                path.display()
            )
        })?;
        // Validate known fields even when no hub has been registered yet.
        let _ = Self::from_storage(&root, path)?;
        Ok(Some(root))
    }

    fn load_legacy() -> Result<Option<Self>> {
        let path = Self::legacy_path();
        let Some(bytes) = crate::config::read_private_optional(&path)? else {
            return Ok(None);
        };
        let content = std::str::from_utf8(&bytes)
            .with_context(|| format!("Telegram group config {} is not UTF-8", path.display()))?;
        let config: Self = toml::from_str(content).map_err(|error: toml::de::Error| {
            crate::config::private_toml_error("Telegram group config", &path, error.span())
        })?;
        config.validate()?;
        Ok(Some(config))
    }

    fn validate(&self) -> Result<()> {
        anyhow::ensure!(
            self.group_id < 0,
            "Telegram forum group id must be negative, got {}",
            self.group_id
        );
        anyhow::ensure!(
            self.topics
                .iter()
                .all(|(project, topic)| !project.is_empty() && *topic > 0),
            "Telegram topics require non-empty project names and positive ids"
        );
        anyhow::ensure!(
            self.topics.values().copied().collect::<BTreeSet<_>>().len() == self.topics.len(),
            "Telegram projects cannot share one topic id"
        );
        Ok(())
    }

    fn write_storage(path: &Path, root: &JsonMap<String, JsonValue>) -> Result<()> {
        let mut bytes = serde_json::to_vec_pretty(&JsonValue::Object(root.clone()))?;
        bytes.push(b'\n');
        crate::config::atomic_write_private(path, &bytes)
    }

    pub fn try_load() -> Result<Option<Self>> {
        let path = Self::path();
        match Self::load_storage(&path)? {
            Some(root) => Self::from_storage(&root, &path),
            None => Self::load_legacy(),
        }
    }

    /// Load the saved group config. Returns None if no group has ever been set
    /// up. Callers that can display errors should prefer [`Self::try_load`].
    pub fn load() -> Option<Self> {
        match Self::try_load() {
            Ok(config) => config,
            Err(error) => {
                eprintln!("omega: Telegram group state is invalid: {error:#}");
                None
            }
        }
    }

    /// Persist to the bot's canonical `telegram-groups.json` authority.
    pub fn save(&self) -> Result<()> {
        self.save_at(&Self::path())
    }

    fn save_at(&self, path: &Path) -> Result<()> {
        self.validate()?;
        let _guard = crate::project_manager::JsonDirLock::acquire(path)?;
        let mut root = Self::load_storage(path)?.unwrap_or_default();
        root.insert("hub".to_string(), JsonValue::from(self.group_id));
        if !self.group_name.is_empty() {
            root.insert(
                "group_name".to_string(),
                JsonValue::String(self.group_name.clone()),
            );
        }
        if !self.setup_at.is_empty() {
            root.insert(
                "setup_at".to_string(),
                JsonValue::String(self.setup_at.clone()),
            );
        }
        let mut topics = root
            .get("topics")
            .and_then(JsonValue::as_object)
            .cloned()
            .unwrap_or_default();
        topics.retain(|_, project| {
            project.as_str().is_some_and(|project| {
                matches!(project.to_ascii_lowercase().as_str(), "atlas" | "alerts")
            })
        });
        for (project, topic) in &self.topics {
            anyhow::ensure!(
                !topics.contains_key(&topic.to_string()),
                "Telegram project {project:?} conflicts with reserved topic id {topic}"
            );
            topics.insert(topic.to_string(), JsonValue::String(project.clone()));
        }
        root.insert("topics".to_string(), JsonValue::Object(topics));
        Self::write_storage(path, &root)
    }

    /// Update only the group identity while preserving every existing topic.
    /// Parsing and writing occur under one lock so a malformed authority is
    /// never replaced with defaults and two Rust writers cannot lose updates.
    pub fn update_group_id(group_id: i64, setup_at: String) -> Result<Self> {
        Self::update_group_id_at(&Self::path(), group_id, setup_at)
    }

    fn update_group_id_at(path: &Path, group_id: i64, setup_at: String) -> Result<Self> {
        if group_id >= 0 {
            anyhow::bail!("Telegram forum group id must be negative, got {group_id}");
        }
        let _guard = crate::project_manager::JsonDirLock::acquire(path)?;
        let mut root = Self::load_storage(path)?.unwrap_or_default();
        root.insert("hub".to_string(), JsonValue::from(group_id));
        root.insert("setup_at".to_string(), JsonValue::String(setup_at));
        Self::write_storage(path, &root)?;
        Self::from_storage(&root, path)?.context("saved Telegram group state has no hub")
    }

    pub fn topic_for(&self, project: &str) -> Option<i64> {
        self.topics.get(project).copied()
    }

    pub fn set_topic(&mut self, project: &str, topic_id: i64) {
        self.topics.insert(project.to_string(), topic_id);
    }

    /// Remove a project's topic mapping. Returns the removed thread id, or
    /// None if the project had no topic. Used by `/project → delete`.
    pub fn remove_topic(&mut self, project: &str) -> Option<i64> {
        self.topics.remove(project)
    }

    /// Reverse lookup: which project owns this forum topic? Lets the bridge
    /// route a message typed in a topic straight to that project's oracle.
    pub fn project_for_topic(&self, thread_id: i64) -> Option<String> {
        self.topics
            .iter()
            .find(|(_, &t)| t == thread_id)
            .map(|(p, _)| p.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut cfg = TelegramGroupConfig {
            group_id: -1001234567890,
            group_name: "OmegaOS Projects".to_string(),
            topics: BTreeMap::new(),
            setup_at: "2026-05-28T18:00:00Z".to_string(),
        };
        cfg.set_topic("DentistryGPT", 42);
        cfg.set_topic("Causio", 47);
        let toml_str = toml::to_string_pretty(&cfg).unwrap();
        let back: TelegramGroupConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(back.group_id, -1001234567890);
        assert_eq!(back.topic_for("Causio"), Some(47));
        assert_eq!(back.topic_for("Missing"), None);
    }

    #[test]
    fn remove_topic_roundtrip() {
        let mut cfg = TelegramGroupConfig::default();
        cfg.set_topic("Causio", 47);
        cfg.set_topic("Kommu", 51);
        assert_eq!(cfg.remove_topic("Causio"), Some(47));
        assert_eq!(cfg.topic_for("Causio"), None);
        assert_eq!(cfg.topic_for("Kommu"), Some(51));
        // Idempotent: removing again returns None.
        assert_eq!(cfg.remove_topic("Causio"), None);
        // Unknown project: None.
        assert_eq!(cfg.remove_topic("Ghost"), None);
    }

    #[test]
    fn reverse_topic_lookup() {
        let mut cfg = TelegramGroupConfig::default();
        cfg.set_topic("DentistryGPT", 6);
        cfg.set_topic("OmegaOS", 7);
        // a message in thread 7 → routes to OmegaOS' oracle
        assert_eq!(cfg.project_for_topic(7).as_deref(), Some("OmegaOS"));
        assert_eq!(cfg.project_for_topic(6).as_deref(), Some("DentistryGPT"));
        // General topic / unmapped thread → None → falls through to Master
        assert_eq!(cfg.project_for_topic(999), None);
    }

    #[test]
    fn strict_group_update_preserves_topics_and_refuses_malformed_authority() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("telegram-groups.json");
        crate::config::atomic_write_private(
            &path,
            &serde_json::to_vec_pretty(&serde_json::json!({
                "hub": -1001,
                "isForum": true,
                "group_name": "Projects",
                "setup_at": "old",
                "atlas_topic": 7,
                "topics": {"7": "atlas", "42": "OmegaOS"},
                "future": {"preserve": true}
            }))
            .unwrap(),
        )
        .unwrap();

        let updated =
            TelegramGroupConfig::update_group_id_at(&path, -2002, "new".to_string()).unwrap();
        assert_eq!(updated.group_id, -2002);
        assert_eq!(updated.group_name, "Projects");
        assert_eq!(updated.topic_for("OmegaOS"), Some(42));
        let stored: JsonValue = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(stored["isForum"], true);
        assert_eq!(stored["atlas_topic"], 7);
        assert_eq!(stored["topics"]["7"], "atlas");
        assert_eq!(stored["topics"]["42"], "OmegaOS");
        assert_eq!(stored["future"], serde_json::json!({"preserve": true}));

        std::fs::write(&path, b"not = [valid").unwrap();
        let before = std::fs::read(&path).unwrap();
        assert!(
            TelegramGroupConfig::update_group_id_at(&path, -3003, "later".to_string(),).is_err()
        );
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert!(
            TelegramGroupConfig::update_group_id_at(&path, 123, "invalid".to_string(),).is_err()
        );
    }

    #[test]
    fn canonical_json_save_preserves_reserved_topics_and_unknown_fields() {
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().join("telegram-groups.json");
        crate::config::atomic_write_private(
            &path,
            br#"{"hub":-1001,"isForum":true,"alerts_topic":8,"topics":{"8":"alerts","42":"Old"},"future":9}"#,
        )
        .unwrap();
        TelegramGroupConfig {
            group_id: -1001,
            group_name: "Projects".into(),
            topics: BTreeMap::from([("New".into(), 77)]),
            setup_at: "now".into(),
        }
        .save_at(&path)
        .unwrap();

        let stored: JsonValue = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(stored["future"], 9);
        assert_eq!(stored["topics"]["8"], "alerts");
        assert_eq!(stored["topics"]["77"], "New");
        assert!(stored["topics"].get("42").is_none());
        assert!(!tmp.path().join("telegram-groups.json.lock").exists());

        let before = std::fs::read(&path).unwrap();
        let conflicting = TelegramGroupConfig {
            group_id: -1001,
            group_name: String::new(),
            topics: BTreeMap::from([("Project".into(), 8)]),
            setup_at: String::new(),
        };
        assert!(conflicting.save_at(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
}
