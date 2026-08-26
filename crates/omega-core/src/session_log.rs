use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SessionEntry {
    #[serde(rename = "session")]
    Header(SessionHeader),
    #[serde(rename = "message")]
    Message(MessageEntry),
    #[serde(rename = "tool_call")]
    ToolCall(ToolCallEntry),
    #[serde(rename = "done")]
    Done(DoneEntry),
    #[serde(rename = "compaction")]
    Compaction(CompactionEntry),
    #[serde(rename = "event")]
    Event(EventEntry),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHeader {
    pub version: u32,
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub cwd: String,
    pub session_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_session: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEntry {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallEntry {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub tool_name: String,
    pub args: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoneEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub status: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub summary: String,
    pub entries_compacted: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub data: serde_json::Value,
}

pub struct SessionLog {
    path: PathBuf,
    session_id: String,
    entry_counter: u64,
}

impl SessionLog {
    pub fn create(sessions_dir: &Path, session_name: &str, cwd: &str) -> Result<Self> {
        std::fs::create_dir_all(sessions_dir)?;

        let session_id = generate_id();
        let filename = format!("{}-{}.jsonl", session_name, &session_id[..8]);
        let path = sessions_dir.join(&filename);

        let mut log = Self {
            path,
            session_id: session_id.clone(),
            entry_counter: 0,
        };

        let header = SessionEntry::Header(SessionHeader {
            version: 1,
            id: session_id,
            timestamp: Utc::now(),
            cwd: cwd.to_string(),
            session_name: session_name.to_string(),
            parent_session: None,
        });

        log.append(&header)?;
        Ok(log)
    }

    pub fn open(path: PathBuf) -> Result<Self> {
        let entries = Self::read_entries(&path)?;
        let session_id = entries
            .first()
            .and_then(|e| match e {
                SessionEntry::Header(h) => Some(h.id.clone()),
                _ => None,
            })
            .unwrap_or_else(generate_id);

        let entry_counter = entries.len() as u64;

        Ok(Self {
            path,
            session_id,
            entry_counter,
        })
    }

    pub fn append(&mut self, entry: &SessionEntry) -> Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        let line = serde_json::to_string(entry)?;
        writeln!(file, "{}", line)?;
        self.entry_counter += 1;
        Ok(())
    }

    pub fn append_message(&mut self, role: &str, content: &str) -> Result<String> {
        let id = self.next_id();
        let entry = SessionEntry::Message(MessageEntry {
            id: id.clone(),
            parent_id: None,
            timestamp: Utc::now(),
            role: role.to_string(),
            content: content.to_string(),
        });
        self.append(&entry)?;
        Ok(id)
    }

    pub fn append_tool_call(
        &mut self,
        tool_name: &str,
        args: serde_json::Value,
        result: Option<&str>,
    ) -> Result<String> {
        let id = self.next_id();
        let entry = SessionEntry::ToolCall(ToolCallEntry {
            id: id.clone(),
            parent_id: None,
            timestamp: Utc::now(),
            tool_name: tool_name.to_string(),
            args,
            result: result.map(|s| s.to_string()),
        });
        self.append(&entry)?;
        Ok(id)
    }

    pub fn append_event(&mut self, event_type: &str, data: serde_json::Value) -> Result<String> {
        let id = self.next_id();
        let entry = SessionEntry::Event(EventEntry {
            id: id.clone(),
            timestamp: Utc::now(),
            event_type: event_type.to_string(),
            data,
        });
        self.append(&entry)?;
        Ok(id)
    }

    pub fn read_entries(path: &Path) -> Result<Vec<SessionEntry>> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<SessionEntry>(&line) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    tracing::warn!(error = %e, "Skipping malformed JSONL entry");
                }
            }
        }

        Ok(entries)
    }

    pub fn entries(&self) -> Result<Vec<SessionEntry>> {
        Self::read_entries(&self.path)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    fn next_id(&mut self) -> String {
        self.entry_counter += 1;
        format!("{:08x}", self.entry_counter)
    }

    pub fn find_latest(sessions_dir: &Path, session_name: &str) -> Option<PathBuf> {
        let prefix = format!("{}-", session_name);
        let mut candidates: Vec<_> = std::fs::read_dir(sessions_dir)
            .ok()?
            .flatten()
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| n.starts_with(&prefix) && n.ends_with(".jsonl"))
                    .unwrap_or(false)
            })
            .collect();

        candidates
            .sort_by_key(|e| std::cmp::Reverse(e.metadata().ok().and_then(|m| m.modified().ok())));
        candidates.first().map(|e| e.path())
    }
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let rand: u32 = (ts as u32).wrapping_mul(2654435761);
    format!("{:012x}{:08x}", ts, rand)
}
