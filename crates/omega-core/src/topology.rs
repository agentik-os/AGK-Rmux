//! Topology manager — logical profiles vs physical isolation.
//!
//! Operator / Agentik / Mission / Private are **product objects**. Linux
//! users, containers, and remotes are **deployment metadata**. Explicit
//! `topology.yaml` always wins. Detection never migrates an existing setup.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::tenancy::AgkScope;

pub const TOPOLOGY_FILE: &str = "topology.yaml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TopologyMode {
    Auto,
    #[serde(rename = "single-user")]
    SingleUser,
    #[serde(rename = "multi-user")]
    MultiUser,
    Custom,
}

impl TopologyMode {
    pub fn id(self) -> &'static str {
        match self {
            TopologyMode::Auto => "auto",
            TopologyMode::SingleUser => "single-user",
            TopologyMode::MultiUser => "multi-user",
            TopologyMode::Custom => "custom",
        }
    }
}

/// How a profile is physically isolated. Extensible — do not switch on this
/// in domain logic; ask [`Topology::binding`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeKind {
    #[serde(rename = "logical-profile")]
    LogicalProfile,
    #[serde(rename = "linux-user")]
    LinuxUser,
    Container,
    Remote,
}

impl RuntimeKind {
    pub fn isolation_label(self) -> &'static str {
        match self {
            RuntimeKind::LogicalProfile => "logical",
            RuntimeKind::LinuxUser => "linux-user",
            RuntimeKind::Container => "container",
            RuntimeKind::Remote => "remote",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileBinding {
    pub profile: AgkScope,
    pub runtime: RuntimeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linux_user: Option<String>,
    pub home: PathBuf,
    pub workspace_root: PathBuf,
    pub hermes_home: PathBuf,
    pub secrets_root: PathBuf,
    pub rmux_namespace: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyFile {
    pub mode: TopologyMode,
    #[serde(default)]
    pub profiles: BTreeMap<String, ProfileFileEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProfileFileEntry {
    #[serde(default)]
    pub runtime: Option<RuntimeKind>,
    #[serde(default)]
    pub linux_user: Option<String>,
    #[serde(default)]
    pub home: Option<PathBuf>,
    #[serde(default)]
    pub workspace_root: Option<PathBuf>,
    #[serde(default)]
    pub hermes_home: Option<PathBuf>,
    #[serde(default)]
    pub secrets_root: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topology {
    pub mode: TopologyMode,
    pub source: TopologySource,
    pub bindings: BTreeMap<AgkScope, ProfileBinding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopologySource {
    Config,
    Detected,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathRequest<'a> {
    pub profile: AgkScope,
    pub client: Option<&'a str>,
    pub object: PathObject,
    pub slug: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathObject {
    Workspace,
    Client,
    Project,
    Mission,
    Knowledge,
    Artifacts,
}

/// Host accounts that look like the four AGK Linux users.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostUser {
    pub name: String,
    pub home: PathBuf,
}

impl Topology {
    pub fn binding(&self, profile: AgkScope) -> &ProfileBinding {
        self.bindings
            .get(&profile)
            .expect("topology missing a logical profile")
    }

    pub fn resolve(&self, req: PathRequest<'_>) -> PathBuf {
        let bind = self.binding(req.profile);
        let mut path = bind.workspace_root.clone();
        if req.profile == AgkScope::Mission {
            if let Some(client) = req.client {
                path.push("clients");
                path.push(client);
            }
        }
        match req.object {
            PathObject::Workspace => {}
            PathObject::Client => {
                if req.profile == AgkScope::Mission && req.client.is_none() {
                    path.push("clients");
                }
            }
            PathObject::Project => {
                path.push("projects");
                if let Some(slug) = req.slug {
                    path.push(slug);
                }
            }
            PathObject::Mission => {
                path.push("missions");
                if let Some(slug) = req.slug {
                    path.push(slug);
                }
            }
            PathObject::Knowledge => path.push("knowledge"),
            PathObject::Artifacts => path.push("artifacts"),
        }
        path
    }
}

pub fn config_path(omega: &Path) -> PathBuf {
    omega.join(TOPOLOGY_FILE)
}

pub fn load(omega: &Path) -> Result<Option<TopologyFile>> {
    let path = config_path(omega);
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let parsed: TopologyFile = serde_yaml::from_str(&raw).context("parsing topology.yaml")?;
    Ok(Some(parsed))
}

/// Config file if present, otherwise single-user under `home`.
/// Does **not** scan the host — that belongs to [`init_if_missing`] / CLI detect.
pub fn load_resolved(home: &Path) -> Topology {
    match load(&home.join(".omega")) {
        Ok(Some(file)) => materialize(home, file, TopologySource::Config),
        _ => single_user(home, TopologySource::Default),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitOutcome {
    Preserved,
    Wrote(TopologyMode),
}

/// Write topology.yaml only when missing. Host scan is an argument so tests
/// never flip because `/home/mission` exists on the build machine.
pub fn init_if_missing(home: &Path, host_users: &[HostUser]) -> Result<InitOutcome> {
    let omega = home.join(".omega");
    if config_path(&omega).is_file() {
        return Ok(InitOutcome::Preserved);
    }
    let file = if detect_candidate(host_users) == TopologyMode::MultiUser {
        multi_user_file(host_users)
    } else {
        TopologyFile {
            mode: TopologyMode::SingleUser,
            profiles: BTreeMap::new(),
        }
    };
    let mode = file.mode;
    write_if_missing(&omega, &file)?;
    Ok(InitOutcome::Wrote(mode))
}

/// Explicit config wins. Otherwise default single-user under `home`.
/// `host_users` is the detector input — never inferred from random /home names
/// inside domain code.
pub fn resolve_topology(home: &Path, omega: &Path, host_users: &[HostUser]) -> Result<Topology> {
    if let Some(file) = load(omega)? {
        return Ok(materialize(home, file, TopologySource::Config));
    }
    let candidate = detect_candidate(host_users);
    match candidate {
        TopologyMode::MultiUser => Ok(materialize(
            home,
            multi_user_file(host_users),
            TopologySource::Detected,
        )),
        _ => Ok(single_user(home, TopologySource::Default)),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionTarget {
    pub profile: AgkScope,
    pub runtime: RuntimeKind,
    pub linux_user: Option<String>,
    pub rmux_namespace: String,
    pub workspace: PathBuf,
    pub hermes_home: PathBuf,
}

pub fn execution_target(topo: &Topology, profile: AgkScope) -> ExecutionTarget {
    let bind = topo.binding(profile);
    ExecutionTarget {
        profile,
        runtime: bind.runtime,
        linux_user: bind.linux_user.clone(),
        rmux_namespace: bind.rmux_namespace.clone(),
        workspace: bind.workspace_root.clone(),
        hermes_home: bind.hermes_home.clone(),
    }
}

/// Global → profile → client → project → mission/run. More specific wins.
pub fn inherit_runtime(layers: &[Option<RuntimeKind>]) -> RuntimeKind {
    layers
        .iter()
        .rev()
        .find_map(|layer| *layer)
        .unwrap_or(RuntimeKind::LogicalProfile)
}

/// All four dedicated users must exist. Extra Unix accounts do not count.
pub fn detect_candidate(host_users: &[HostUser]) -> TopologyMode {
    let have: Vec<&str> = host_users.iter().map(|u| u.name.as_str()).collect();
    let needed = ["operator", "agentik", "mission", "private"];
    if needed.iter().all(|n| have.contains(n)) {
        TopologyMode::MultiUser
    } else {
        TopologyMode::SingleUser
    }
}

pub fn single_user(home: &Path, source: TopologySource) -> Topology {
    materialize(
        home,
        TopologyFile {
            mode: TopologyMode::SingleUser,
            profiles: BTreeMap::new(),
        },
        source,
    )
}

fn multi_user_file(host_users: &[HostUser]) -> TopologyFile {
    let mut profiles = BTreeMap::new();
    for scope in AgkScope::ALL {
        if let Some(user) = host_users.iter().find(|u| u.name == scope.id()) {
            profiles.insert(
                scope.id().to_string(),
                ProfileFileEntry {
                    runtime: Some(RuntimeKind::LinuxUser),
                    linux_user: Some(user.name.clone()),
                    home: Some(user.home.clone()),
                    workspace_root: Some(user.home.join("workspace")),
                    hermes_home: Some(user.home.join(".hermes")),
                    secrets_root: Some(
                        user.home
                            .join(".omega")
                            .join("scopes")
                            .join(scope.id())
                            .join("secrets"),
                    ),
                },
            );
        }
    }
    TopologyFile {
        mode: TopologyMode::MultiUser,
        profiles: profiles,
    }
}

pub fn materialize(fallback_home: &Path, file: TopologyFile, source: TopologySource) -> Topology {
    let resolved_mode = match file.mode {
        TopologyMode::Auto => {
            if file
                .profiles
                .values()
                .any(|p| p.runtime == Some(RuntimeKind::LinuxUser))
            {
                TopologyMode::MultiUser
            } else if file.profiles.is_empty() {
                TopologyMode::SingleUser
            } else {
                TopologyMode::Custom
            }
        }
        other => other,
    };
    let mut bindings = BTreeMap::new();
    for scope in AgkScope::ALL {
        let entry = file.profiles.get(scope.id());
        bindings.insert(
            scope,
            bind_profile(fallback_home, scope, resolved_mode, entry),
        );
    }
    Topology {
        mode: resolved_mode,
        source,
        bindings,
    }
}

fn bind_profile(
    fallback_home: &Path,
    scope: AgkScope,
    mode: TopologyMode,
    entry: Option<&ProfileFileEntry>,
) -> ProfileBinding {
    let runtime = entry.and_then(|e| e.runtime).unwrap_or(match mode {
        TopologyMode::MultiUser => RuntimeKind::LinuxUser,
        _ => RuntimeKind::LogicalProfile,
    });
    let home = entry
        .and_then(|e| e.home.clone())
        .unwrap_or_else(|| match runtime {
            RuntimeKind::LinuxUser => PathBuf::from("/home").join(scope.id()),
            _ => fallback_home.to_path_buf(),
        });
    let workspace_root = entry
        .and_then(|e| e.workspace_root.clone())
        .unwrap_or_else(|| match runtime {
            RuntimeKind::LogicalProfile => home.join("workspace").join(scope.id()),
            _ => home.join("workspace"),
        });
    let hermes_home = entry
        .and_then(|e| e.hermes_home.clone())
        .unwrap_or_else(|| match runtime {
            RuntimeKind::LogicalProfile => home.join(".hermes").join("profiles").join(scope.id()),
            _ => home.join(".hermes"),
        });
    let secrets_root = entry
        .and_then(|e| e.secrets_root.clone())
        .unwrap_or_else(|| {
            home.join(".omega")
                .join("scopes")
                .join(scope.id())
                .join("secrets")
        });
    ProfileBinding {
        profile: scope,
        runtime,
        linux_user: entry
            .and_then(|e| e.linux_user.clone())
            .or_else(|| (runtime == RuntimeKind::LinuxUser).then(|| scope.id().to_string())),
        home,
        workspace_root,
        hermes_home,
        secrets_root,
        rmux_namespace: scope.id().to_string(),
    }
}

/// Write topology.yaml only when missing. Never overwrite a working config.
pub fn write_if_missing(omega: &Path, file: &TopologyFile) -> Result<bool> {
    let path = config_path(omega);
    if path.is_file() {
        return Ok(false);
    }
    fs::create_dir_all(omega)?;
    let body = serde_yaml::to_string(file).context("serializing topology.yaml")?;
    let header = "# Agentik topology — explicit config wins over auto-detect.\n\
                  # mode: auto | single-user | multi-user | custom\n\
                  # Profiles are logical. linux_user / home are deployment metadata.\n";
    fs::write(&path, format!("{header}{body}"))?;
    Ok(true)
}

pub fn plan(mode: TopologyMode, home: &Path, host_users: &[HostUser]) -> Result<TopologyFile> {
    match mode {
        TopologyMode::SingleUser | TopologyMode::Auto => Ok(TopologyFile {
            mode: TopologyMode::SingleUser,
            profiles: BTreeMap::new(),
        }),
        TopologyMode::MultiUser => {
            if detect_candidate(host_users) != TopologyMode::MultiUser {
                bail!(
                    "multi-user plan needs Linux users operator, agentik, mission, and private — \
                     detected {}. This is a plan only; nothing was moved.",
                    host_users
                        .iter()
                        .map(|u| u.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            let _ = home;
            Ok(multi_user_file(host_users))
        }
        TopologyMode::Custom => bail!("custom topology is written by hand, not planned"),
    }
}

/// Best-effort scan. Extra accounts are ignored. Used by CLI/install only.
pub fn scan_host_users() -> Vec<HostUser> {
    let mut found = Vec::new();
    for name in ["operator", "agentik", "mission", "private"] {
        for root in [PathBuf::from("/home"), PathBuf::from("/Users")] {
            let home = root.join(name);
            if home.is_dir() {
                found.push(HostUser {
                    name: name.to_string(),
                    home,
                });
                break;
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn users(root: &Path) -> Vec<HostUser> {
        AgkScope::ALL
            .iter()
            .map(|s| HostUser {
                name: s.id().to_string(),
                home: root.join(s.id()),
            })
            .collect()
    }

    #[test]
    fn random_unix_accounts_are_not_multi_user() {
        let hosts = vec![HostUser {
            name: "git".into(),
            home: PathBuf::from("/home/git"),
        }];
        assert_eq!(detect_candidate(&hosts), TopologyMode::SingleUser);
    }

    #[test]
    fn four_dedicated_users_are_multi_user() {
        let hosts = users(Path::new("/home"));
        assert_eq!(detect_candidate(&hosts), TopologyMode::MultiUser);
    }

    #[test]
    fn same_project_id_resolves_different_paths() {
        let single = single_user(Path::new("/home/agk"), TopologySource::Default);
        let multi = materialize(
            Path::new("/home/agk"),
            multi_user_file(&users(Path::new("/home"))),
            TopologySource::Detected,
        );
        let req = PathRequest {
            profile: AgkScope::Mission,
            client: Some("moonbase"),
            object: PathObject::Project,
            slug: Some("ceo-dashboard"),
        };
        assert_eq!(
            single.resolve(req.clone()),
            PathBuf::from("/home/agk/workspace/mission/clients/moonbase/projects/ceo-dashboard")
        );
        assert_eq!(
            multi.resolve(req),
            PathBuf::from("/home/mission/workspace/clients/moonbase/projects/ceo-dashboard")
        );
        assert_eq!(
            single.binding(AgkScope::Mission).runtime,
            RuntimeKind::LogicalProfile
        );
        assert_eq!(
            multi.binding(AgkScope::Mission).runtime,
            RuntimeKind::LinuxUser
        );
        assert_eq!(
            multi.binding(AgkScope::Mission).hermes_home,
            PathBuf::from("/home/mission/.hermes")
        );
        assert_eq!(
            single.binding(AgkScope::Mission).hermes_home,
            PathBuf::from("/home/agk/.hermes/profiles/mission")
        );
    }

    #[test]
    fn explicit_config_wins_over_four_users() {
        let tmp = tempfile::TempDir::new().unwrap();
        let omega = tmp.path().join(".omega");
        fs::create_dir_all(&omega).unwrap();
        let file = TopologyFile {
            mode: TopologyMode::SingleUser,
            profiles: BTreeMap::new(),
        };
        assert!(write_if_missing(&omega, &file).unwrap());
        assert!(!write_if_missing(&omega, &file).unwrap());
        let hosts = users(Path::new("/home"));
        let topo = resolve_topology(tmp.path(), &omega, &hosts).unwrap();
        assert_eq!(topo.mode, TopologyMode::SingleUser);
        assert_eq!(topo.source, TopologySource::Config);
        assert!(topo
            .binding(AgkScope::Mission)
            .workspace_root
            .ends_with("workspace/mission"));
    }

    #[test]
    fn inherit_runtime_more_specific_wins() {
        assert_eq!(
            inherit_runtime(&[
                Some(RuntimeKind::LogicalProfile),
                None,
                Some(RuntimeKind::Container),
                None,
            ]),
            RuntimeKind::Container
        );
    }

    #[test]
    fn plan_is_non_destructive() {
        let err = plan(TopologyMode::MultiUser, Path::new("/tmp"), &[]).unwrap_err();
        assert!(err.to_string().contains("plan only"));
    }
}
