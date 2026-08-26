//! Smart router: routes parsed intents to the correct oracle/project.
//!
//! Mirrors the VPS routing logic: keyword detection, topic_id mapping,
//! multi-project parallel dispatch, DM auto-dispatch, and prompt enhancement.

use crate::intent::{IntentJson, IntentParser};
use crate::routing::RoutingDecision;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A registered project that the router can dispatch to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEntry {
    pub name: String,
    pub path: PathBuf,
    pub aliases: Vec<String>,
    pub topic_id: Option<i64>,
    pub oracle_session: String,
    pub git_email: Option<String>,
}

impl ProjectEntry {
    pub fn new(name: &str, path: PathBuf) -> Self {
        let oracle_session = format!("oracle-{}", name);
        Self {
            name: name.to_string(),
            path,
            aliases: Vec::new(),
            topic_id: None,
            oracle_session,
            git_email: None,
        }
    }

    pub fn with_aliases(mut self, aliases: Vec<String>) -> Self {
        self.aliases = aliases;
        self
    }

    pub fn with_topic_id(mut self, topic_id: i64) -> Self {
        self.topic_id = Some(topic_id);
        self
    }
}

/// Registry of all known projects, loadable from config or auto-discovered.
#[derive(Debug, Clone)]
pub struct ProjectRegistry {
    projects: Vec<ProjectEntry>,
    topic_map: HashMap<i64, usize>,
    name_map: HashMap<String, usize>,
}

impl ProjectRegistry {
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
            topic_map: HashMap::new(),
            name_map: HashMap::new(),
        }
    }

    pub fn register(&mut self, entry: ProjectEntry) {
        let idx = self.projects.len();
        if let Some(tid) = entry.topic_id {
            self.topic_map.insert(tid, idx);
        }
        self.name_map.insert(entry.name.to_lowercase(), idx);
        for alias in &entry.aliases {
            self.name_map.insert(alias.to_lowercase(), idx);
        }
        self.projects.push(entry);
    }

    pub fn by_name(&self, name: &str) -> Option<&ProjectEntry> {
        self.name_map
            .get(&name.to_lowercase())
            .map(|&idx| &self.projects[idx])
    }

    pub fn by_topic_id(&self, topic_id: i64) -> Option<&ProjectEntry> {
        self.topic_map
            .get(&topic_id)
            .map(|&idx| &self.projects[idx])
    }

    pub fn all(&self) -> &[ProjectEntry] {
        &self.projects
    }

    pub fn len(&self) -> usize {
        self.projects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.projects.is_empty()
    }

    /// Load from a projects.json file (compatible with VPS format).
    pub fn load_from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let entries: Vec<ProjectFileEntry> = serde_json::from_str(&content)?;
        let mut registry = Self::new();
        for e in entries {
            let mut entry = ProjectEntry::new(&e.name, PathBuf::from(&e.path));
            if let Some(aliases) = e.aliases {
                entry = entry.with_aliases(aliases);
            }
            if let Some(tid) = e.topic_id {
                entry = entry.with_topic_id(tid);
            }
            entry.git_email = e.git_email;
            registry.register(entry);
        }
        Ok(registry)
    }

    /// Build registry from OmegaConfig project list.
    pub fn from_config(projects: &[crate::config::ProjectConfig]) -> Self {
        let mut registry = Self::new();
        for p in projects {
            let entry = ProjectEntry::new(&p.name, p.path.clone());
            registry.register(entry);
        }
        registry
    }
}

impl Default for ProjectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON shape for projects.json file entries.
#[derive(Debug, Deserialize)]
struct ProjectFileEntry {
    name: String,
    path: String,
    aliases: Option<Vec<String>>,
    topic_id: Option<i64>,
    git_email: Option<String>,
}

/// The routing decision produced by SmartRouter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResult {
    pub intent: IntentJson,
    pub routing: RoutingDecision,
    pub targets: Vec<RouteTarget>,
    pub enhanced_prompt: Option<String>,
    pub source: RouteSource,
}

/// Where the message originated from — determines routing strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteSource {
    DirectMessage,
    TopicMessage,
    SlashCommand,
    ReplyToReport,
}

/// A specific dispatch target (project + oracle session).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteTarget {
    pub project_name: String,
    pub oracle_session: String,
    pub project_path: PathBuf,
}

/// Reply-based routing tracker: maps message IDs to project names.
#[derive(Debug, Clone, Default)]
pub struct ReplyRouter {
    report_map: HashMap<i64, String>,
}

impl ReplyRouter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn track_report(&mut self, message_id: i64, project: &str) {
        self.report_map.insert(message_id, project.to_string());
    }

    pub fn lookup(&self, reply_to_message_id: i64) -> Option<&str> {
        self.report_map
            .get(&reply_to_message_id)
            .map(|s| s.as_str())
    }
}

/// The main smart router that ties together intent parsing, project
/// registry, and routing decisions.
pub struct SmartRouter {
    parser: IntentParser,
    registry: ProjectRegistry,
    reply_router: ReplyRouter,
}

impl SmartRouter {
    pub fn new(registry: ProjectRegistry) -> Self {
        let mut parser = IntentParser::new();
        // Register project keywords with the intent parser
        for proj in registry.all() {
            parser.register_project(&proj.name, proj.aliases.clone());
        }

        Self {
            parser,
            registry,
            reply_router: ReplyRouter::new(),
        }
    }

    /// Route a direct message (DM) — detect projects from keywords.
    pub fn route_dm(&self, text: &str) -> RouteResult {
        let intent = self.parser.parse(text);
        let routing = crate::routing::classify_mission(text);

        let targets = self.resolve_targets(&intent);
        let enhanced_prompt = if !targets.is_empty() {
            Some(PromptEnhancer::enhance(text, &intent, targets.first()))
        } else {
            None
        };

        RouteResult {
            intent,
            routing,
            targets,
            enhanced_prompt,
            source: RouteSource::DirectMessage,
        }
    }

    /// Route a topic message — project is determined by topic_id.
    pub fn route_topic(&self, text: &str, topic_id: i64) -> RouteResult {
        let intent = self.parser.parse(text);
        let routing = crate::routing::classify_mission(text);

        let targets = if let Some(proj) = self.registry.by_topic_id(topic_id) {
            vec![RouteTarget {
                project_name: proj.name.clone(),
                oracle_session: proj.oracle_session.clone(),
                project_path: proj.path.clone(),
            }]
        } else {
            Vec::new()
        };

        let enhanced_prompt = if !targets.is_empty() {
            Some(PromptEnhancer::enhance(text, &intent, targets.first()))
        } else {
            None
        };

        RouteResult {
            intent,
            routing,
            targets,
            enhanced_prompt,
            source: RouteSource::TopicMessage,
        }
    }

    /// Route a slash command (e.g. /causio fix auth).
    pub fn route_command(&self, command: &str, args: &str) -> RouteResult {
        let full_text = if args.is_empty() {
            command.to_string()
        } else {
            format!("{} {}", command, args)
        };

        let intent = self.parser.parse(&full_text);
        let routing = crate::routing::classify_mission(&full_text);

        // Try to match command name to a project
        let cmd_lower = command.trim_start_matches('/').to_lowercase();
        let targets = if let Some(proj) = self.registry.by_name(&cmd_lower) {
            vec![RouteTarget {
                project_name: proj.name.clone(),
                oracle_session: proj.oracle_session.clone(),
                project_path: proj.path.clone(),
            }]
        } else {
            self.resolve_targets(&intent)
        };

        let enhanced_prompt = if !targets.is_empty() {
            Some(PromptEnhancer::enhance(
                &full_text,
                &intent,
                targets.first(),
            ))
        } else {
            None
        };

        RouteResult {
            intent,
            routing,
            targets,
            enhanced_prompt,
            source: RouteSource::SlashCommand,
        }
    }

    /// Route a reply to a previous report — uses reply_router lookup.
    pub fn route_reply(&self, text: &str, reply_to_message_id: i64) -> RouteResult {
        let intent = self.parser.parse(text);
        let routing = crate::routing::classify_mission(text);

        let targets = if let Some(project_name) = self.reply_router.lookup(reply_to_message_id) {
            if let Some(proj) = self.registry.by_name(project_name) {
                vec![RouteTarget {
                    project_name: proj.name.clone(),
                    oracle_session: proj.oracle_session.clone(),
                    project_path: proj.path.clone(),
                }]
            } else {
                Vec::new()
            }
        } else {
            // Fallback to DM-style keyword detection
            self.resolve_targets(&intent)
        };

        let enhanced_prompt = if !targets.is_empty() {
            Some(PromptEnhancer::enhance(text, &intent, targets.first()))
        } else {
            None
        };

        RouteResult {
            intent,
            routing,
            targets,
            enhanced_prompt,
            source: RouteSource::ReplyToReport,
        }
    }

    /// Track a sent report for reply-based routing.
    pub fn track_report(&mut self, message_id: i64, project: &str) {
        self.reply_router.track_report(message_id, project);
    }

    pub fn registry(&self) -> &ProjectRegistry {
        &self.registry
    }

    /// Resolve IntentJson's detected_projects into RouteTargets.
    fn resolve_targets(&self, intent: &IntentJson) -> Vec<RouteTarget> {
        intent
            .detected_projects
            .iter()
            .filter_map(|name| {
                self.registry.by_name(name).map(|proj| RouteTarget {
                    project_name: proj.name.clone(),
                    oracle_session: proj.oracle_session.clone(),
                    project_path: proj.path.clone(),
                })
            })
            .collect()
    }
}

/// Transforms casual user text into a structured oracle dispatch prompt.
/// Mirrors VPS `enhance_prompt` + `_build_oracle_dispatch_prompt`.
pub struct PromptEnhancer;

impl PromptEnhancer {
    /// Enhance raw user text into a structured oracle prompt.
    pub fn enhance(raw_text: &str, intent: &IntentJson, target: Option<&RouteTarget>) -> String {
        let mut prompt = String::with_capacity(512);

        // Header with project context
        if let Some(t) = target {
            prompt.push_str(&format!(
                "## Project: {}\nPath: {}\nSession: {}\n\n",
                t.project_name,
                t.project_path.display(),
                t.oracle_session,
            ));
        }

        // Technical objective
        prompt.push_str("## Objective\n");
        prompt.push_str(&format!(
            "Action: {} | Confidence: {:.0}%\n",
            intent.action.label(),
            intent.confidence * 100.0,
        ));
        if let Some(ref tgt) = intent.target {
            prompt.push_str(&format!("Target: {}\n", tgt));
        }
        prompt.push('\n');

        // Original request
        prompt.push_str("## Original Request\n");
        prompt.push_str(raw_text);
        prompt.push_str("\n\n");

        // Success criteria
        if !intent.success_criteria.is_empty() {
            prompt.push_str("## Success Criteria\n");
            for c in &intent.success_criteria {
                prompt.push_str(&format!("- {}\n", c));
            }
            prompt.push('\n');
        }

        // Verify commands
        if !intent.verify_commands.is_empty() {
            prompt.push_str("## Verify Commands\n```bash\n");
            for cmd in &intent.verify_commands {
                prompt.push_str(cmd);
                prompt.push('\n');
            }
            prompt.push_str("```\n\n");
        }

        // Audit skills
        if !intent.detected_audits.is_empty() {
            prompt.push_str("## Audit Skills to Invoke\n");
            for audit in &intent.detected_audits {
                prompt.push_str(&format!("- /{}\n", audit));
            }
            prompt.push('\n');
        }

        // Signal file instruction
        if let Some(t) = target {
            prompt.push_str(&format!(
                "## Signal File\nWrite result to: /tmp/aisb-oracle-result-{}.md\n",
                t.project_name.to_lowercase(),
            ));
        }

        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_registry() -> ProjectRegistry {
        let mut reg = ProjectRegistry::new();
        reg.register(
            ProjectEntry::new("Causio", PathBuf::from("/home/user/projects/Causio"))
                .with_aliases(vec!["causio".into()])
                .with_topic_id(28),
        );
        reg.register(
            ProjectEntry::new("Kommu", PathBuf::from("/home/user/projects/Kommu"))
                .with_aliases(vec!["kommu".into()])
                .with_topic_id(31),
        );
        reg.register(
            ProjectEntry::new(
                "DentistryGPT",
                PathBuf::from("/home/user/projects/DentistryGPT"),
            )
            .with_aliases(vec!["dent".into(), "dentistry".into()])
            .with_topic_id(27),
        );
        reg
    }

    #[test]
    fn dm_routes_to_project_by_keyword() {
        let router = SmartRouter::new(test_registry());
        let result = router.route_dm("fix le login sur Causio");
        assert_eq!(result.targets.len(), 1);
        assert_eq!(result.targets[0].project_name, "Causio");
        assert_eq!(result.source, RouteSource::DirectMessage);
    }

    #[test]
    fn dm_multi_project_dispatch() {
        let router = SmartRouter::new(test_registry());
        let result = router.route_dm("deploy Causio et Kommu en prod");
        assert_eq!(result.targets.len(), 2);
        assert!(result.intent.is_multi_project);
    }

    #[test]
    fn dm_no_project_empty_targets() {
        let router = SmartRouter::new(test_registry());
        let result = router.route_dm("what is the weather today?");
        assert!(result.targets.is_empty());
    }

    #[test]
    fn topic_routes_by_id() {
        let router = SmartRouter::new(test_registry());
        let result = router.route_topic("fix the sidebar", 28);
        assert_eq!(result.targets.len(), 1);
        assert_eq!(result.targets[0].project_name, "Causio");
        assert_eq!(result.source, RouteSource::TopicMessage);
    }

    #[test]
    fn topic_unknown_id_empty_targets() {
        let router = SmartRouter::new(test_registry());
        let result = router.route_topic("do something", 99999);
        assert!(result.targets.is_empty());
    }

    #[test]
    fn slash_command_routes_to_project() {
        let router = SmartRouter::new(test_registry());
        let result = router.route_command("/causio", "fix auth flow");
        assert_eq!(result.targets.len(), 1);
        assert_eq!(result.targets[0].project_name, "Causio");
        assert_eq!(result.source, RouteSource::SlashCommand);
    }

    #[test]
    fn reply_routing() {
        let mut router = SmartRouter::new(test_registry());
        router.track_report(12345, "Kommu");

        let result = router.route_reply("continue with the auth fix", 12345);
        assert_eq!(result.targets.len(), 1);
        assert_eq!(result.targets[0].project_name, "Kommu");
        assert_eq!(result.source, RouteSource::ReplyToReport);
    }

    #[test]
    fn reply_unknown_msg_falls_back_to_keywords() {
        let router = SmartRouter::new(test_registry());
        let result = router.route_reply("fix Causio sidebar", 99999);
        assert_eq!(result.targets.len(), 1);
        assert_eq!(result.targets[0].project_name, "Causio");
    }

    #[test]
    fn enhanced_prompt_contains_project_context() {
        let router = SmartRouter::new(test_registry());
        let result = router.route_dm("fix le login sur Causio");
        let prompt = result.enhanced_prompt.unwrap();
        assert!(prompt.contains("Causio"));
        assert!(prompt.contains("Objective"));
        assert!(prompt.contains("Success Criteria"));
    }

    #[test]
    fn enhanced_prompt_includes_audit_skills() {
        let router = SmartRouter::new(test_registry());
        let result = router.route_dm("fais un audit UX et code audit sur Causio");
        let prompt = result.enhanced_prompt.unwrap();
        assert!(prompt.contains("/uiuxaudit"));
        assert!(prompt.contains("/codeaudit"));
    }

    #[test]
    fn registry_lookup_by_name() {
        let reg = test_registry();
        assert!(reg.by_name("Causio").is_some());
        assert!(reg.by_name("causio").is_some()); // case-insensitive
        assert!(reg.by_name("dent").is_some()); // alias
        assert!(reg.by_name("nonexistent").is_none());
    }

    #[test]
    fn registry_lookup_by_topic() {
        let reg = test_registry();
        let proj = reg.by_topic_id(28).unwrap();
        assert_eq!(proj.name, "Causio");
        assert!(reg.by_topic_id(99999).is_none());
    }

    #[test]
    fn registry_from_config() {
        use crate::config::{ProjectCategory, ProjectConfig};
        let configs = vec![ProjectConfig {
            name: "TestProject".into(),
            path: PathBuf::from("/tmp/test"),
            category: ProjectCategory::Work,
        }];
        let reg = ProjectRegistry::from_config(&configs);
        assert_eq!(reg.len(), 1);
        assert!(reg.by_name("testproject").is_some());
    }
}
