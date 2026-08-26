//! Execution graph types — the persisted SHAPE of a mission (R-GRAPH).
//!
//! [`crate::mission::PlanContract`] already models a mission as a list of tasks
//! plus `depends_on`, which is a DAG and nothing more. That is enough to answer
//! "what must run before what" and nothing else: it cannot say that a branch is
//! taken on a classification string, and it cannot say that a stage loops until
//! it runs dry, because a `depends_on` cycle is unconditionally rejected there.
//! Both of those are real, daily shapes — the router that sends a finding to the
//! right verifier, and the loop-until-dry sweep that keeps fanning out finders
//! until K rounds surface nothing new.
//!
//! This module is the persisted vocabulary for those shapes, and deliberately
//! ONLY the vocabulary: types, validation, and pure lookups. No executor, no
//! risk gate, no ledger write, no process spawn, no network. A separate task
//! owns each of those, and a type file that quietly grew an executor would be a
//! second writer on their surface.
//!
//! What it reuses rather than re-invents, because a second vocabulary for the
//! same concept is how two halves of one system drift apart:
//!
//! - [`crate::mission::TaskAttemptState`] IS the node lifecycle. A node does not
//!   get its own `Pending | Done | Error` enum; it moves through the exact same
//!   machine an attempt does, so a node's state and the ledger's state are the
//!   same word and [`crate::mission::InvalidTransition`] is the same error.
//! - [`crate::mission::RetryPolicy`] and [`crate::mission::VerifierCheck`] are
//!   attached per node, so R-LOOP's bounded retries and R-VERIFY's checks are
//!   carried by the graph itself instead of re-derived by whoever runs it.
//! - [`crate::mission::TaskId`] links a node back to the task contract that
//!   authorized it, when there is one. Optional, because a reduce node is plain
//!   code and never had a contract.
//!
//! Two invariants shape every struct here:
//!
//! 1. AN OLD FILE STILL LOADS AND AN UNKNOWN FIELD STILL SURVIVES. Every added
//!    field carries `#[serde(default)]` and every struct carries a flattened
//!    `extra` map. A graph written by a newer OmegaOS, read here, and written
//!    back keeps the fields this version has never heard of. There is no
//!    destructive migration anywhere in this file: nothing rewrites, drops, or
//!    renames a key it does not own.
//! 2. THE BRANCH IS DATA, NOT A MODEL CALL. A [`Router`] resolves a
//!    classification string through a `BTreeMap` and a default. The same string
//!    always lands on the same node, on every machine, in every replay. A model
//!    may PRODUCE the classification string somewhere else; the branch it takes
//!    is a table lookup, so there is no "it decided to skip the audit" surprise
//!    and a run is reproducible from the persisted graph alone.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

use crate::mission::{
    InvalidTransition, MissionId, PlanContract, PlanId, RetryPolicy, TaskAttemptState, TaskId,
    VerifierCheck,
};

/// Schema version of a persisted [`Graph`] / [`GraphState`].
///
/// Same idiom as [`crate::mission::CONTRACT_SCHEMA_VERSION`]: a bare `u32`
/// written into the document and re-supplied by serde when an older file omits
/// it, so a file that predates the field is readable rather than a parse error.
pub const GRAPH_SCHEMA_VERSION: u32 = 1;

fn graph_schema_version() -> u32 {
    GRAPH_SCHEMA_VERSION
}

/// Secret runtime authority used to authenticate execution receipts. The key is
/// intentionally never serializable: callers persist it separately from the
/// editable GraphState document with owner-only permissions.
#[derive(Clone)]
pub struct GraphExecutionAuthority([u8; 32]);

impl GraphExecutionAuthority {
    pub fn from_key(key: [u8; 32]) -> Self {
        Self(key)
    }

    /// Public identifier safe to persist beside the state. It selects a key but
    /// cannot be used to forge a MAC.
    pub fn authority_id(&self) -> String {
        blake3::hash(&self.0).to_hex().to_string()
    }

    pub(crate) fn mac(&self, domain: &str, fields: &[&[u8]]) -> String {
        let mut hasher = blake3::Hasher::new_keyed(&self.0);
        hasher.update(&(domain.len() as u64).to_le_bytes());
        hasher.update(domain.as_bytes());
        for field in fields {
            hasher.update(&(field.len() as u64).to_le_bytes());
            hasher.update(field);
        }
        hasher.finalize().to_hex().to_string()
    }

    /// Authenticate one entry in the CLI's append-only graph journal without
    /// exposing a generic MAC oracle. The typed fields and fixed domain keep a
    /// journal signature from being reusable as a reservation, output, gate,
    /// or acceptance receipt.
    pub fn sign_journal_record(
        &self,
        sequence: u64,
        previous_hash: &str,
        payload_digest: &str,
    ) -> String {
        self.mac(
            "omega.graph.journal-record.v1",
            &[
                &sequence.to_le_bytes(),
                previous_hash.as_bytes(),
                payload_digest.as_bytes(),
            ],
        )
    }

    /// Verify a signature produced by [`Self::sign_journal_record`].
    pub fn verify_journal_record(
        &self,
        sequence: u64,
        previous_hash: &str,
        payload_digest: &str,
        supplied_mac: &str,
    ) -> bool {
        supplied_mac == self.sign_journal_record(sequence, previous_hash, payload_digest)
    }

    /// Stable digest of the canonical serialized journal payload supplied by
    /// the CLI. Kept here so every driver uses the same hash primitive.
    pub fn journal_payload_digest(payload: &[u8]) -> String {
        blake3::hash(payload).to_hex().to_string()
    }

    /// Public chain hash for one authenticated journal envelope. This is not an
    /// authority proof; it links the next entry to the exact signed envelope.
    pub fn journal_record_hash(
        sequence: u64,
        previous_hash: &str,
        payload_digest: &str,
        authority_mac: &str,
    ) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"omega.graph.journal-chain.v1");
        hasher.update(&sequence.to_le_bytes());
        for value in [previous_hash, payload_digest, authority_mac] {
            hasher.update(&(value.len() as u64).to_le_bytes());
            hasher.update(value.as_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }
}

impl fmt::Debug for GraphExecutionAuthority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GraphExecutionAuthority")
            .field("authority_id", &self.authority_id())
            .finish_non_exhaustive()
    }
}

impl Drop for GraphExecutionAuthority {
    fn drop(&mut self) {
        self.0.fill(0);
    }
}

// ---------------------------------------------------------------------------
// NodeId — the key everything else points at
// ---------------------------------------------------------------------------

/// Identity of one node, unique inside one graph.
///
/// `Ord` is derived on purpose: routers and per-node state are `BTreeMap`s keyed
/// by this type, so iteration order, serialized key order and therefore error
/// messages are stable across runs. A `HashMap` here would make two validations
/// of the same broken graph report a different node first.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for NodeId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

// ---------------------------------------------------------------------------
// NodeKind — what a node IS, which decides what it costs
// ---------------------------------------------------------------------------

/// The role a node plays in the graph.
///
/// The split that matters is [`NodeKind::Reduce`] versus everything else: a
/// flatten, a dedupe, a filter or a rank between a fan-out and a synthesis is
/// plain code and costs zero model tokens (R-GRAPH). Naming it as its own kind
/// is what stops the next author from burning an agent on an edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// One bounded model call: one input in, one output out.
    Agent,
    /// Pure code between stages. No model call, no tokens.
    Reduce,
    /// Hosts a [`Router`]: classifies, then branches through the table.
    Router,
    /// Sits on an edge and tries to REFUTE a finding before it moves downstream.
    Verifier,
    /// Merges the fan-in into the answer that leaves the graph.
    Synthesis,
}

// ---------------------------------------------------------------------------
// Node
// ---------------------------------------------------------------------------

fn default_node_state() -> TaskAttemptState {
    TaskAttemptState::Queued
}

/// One node: one bounded job, one input, one output.
///
/// `state` is a [`TaskAttemptState`] rather than a graph-local enum so a node
/// and the attempt ledger speak one vocabulary; see the module doc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    /// Bounded retries for this node (R-LOOP). Defaulted so a graph written
    /// before this field existed loads with the mission default rather than
    /// failing to parse.
    #[serde(default)]
    pub retry: RetryPolicy,
    /// The checks that must pass before this node's output is accepted
    /// (R-VERIFY). Empty is legal: a reduce node has nothing to verify.
    #[serde(default)]
    pub checks: Vec<VerifierCheck>,
    #[serde(default = "default_node_state")]
    pub state: TaskAttemptState,
    /// The task contract that authorized this node, when one exists. A reduce
    /// node is plain code and has none, so this is optional rather than a
    /// synthetic id nobody can resolve.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<TaskId>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Node {
    /// A queued node of `kind`, with the mission's default retry policy.
    pub fn new(id: impl Into<NodeId>, kind: NodeKind) -> Self {
        Self {
            id: id.into(),
            kind,
            retry: RetryPolicy::default(),
            checks: Vec::new(),
            state: default_node_state(),
            task_id: None,
            extra: Map::new(),
        }
    }

    pub fn with_task(mut self, task_id: TaskId) -> Self {
        self.task_id = Some(task_id);
        self
    }

    pub fn with_checks(mut self, checks: Vec<VerifierCheck>) -> Self {
        self.checks = checks;
        self
    }

    pub fn with_retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    /// Move this node's lifecycle, through the mission state machine.
    ///
    /// Delegating instead of re-implementing is the whole point: an illegal move
    /// here fails with the same [`InvalidTransition`] the ledger raises, so a
    /// caller has one error type to handle rather than two that disagree.
    pub fn transition(&mut self, next: TaskAttemptState) -> Result<(), InvalidTransition> {
        self.state = self.state.transition(next)?;
        Ok(())
    }
}

impl From<NodeId> for Node {
    fn from(id: NodeId) -> Self {
        Self::new(id, NodeKind::Agent)
    }
}

// ---------------------------------------------------------------------------
// Edge
// ---------------------------------------------------------------------------

/// An edge exists ONLY where data actually moves from `from` to `to`.
///
/// This is the "and then" test made structural (R-GRAPH): if the next node does
/// not READ the previous node's output, there is no edge and the wait it implies
/// is pure latency. Encoding sequencing-that-is-not-dataflow as an edge is the
/// single most common way a graph gets needlessly serial.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Edge {
    pub fn new(from: impl Into<NodeId>, to: impl Into<NodeId>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            extra: Map::new(),
        }
    }

    fn same_ends(&self, from: &NodeId, to: &NodeId) -> bool {
        self.from == *from && self.to == *to
    }
}

// ---------------------------------------------------------------------------
// Router — the branch, as deterministic data
// ---------------------------------------------------------------------------

/// A deterministic branch: a classification string in, one node out.
///
/// `on` is the exact top-level field read from the host's authenticated
/// structured output. Its value must be a string. Resolution is
/// [`Router::resolve`] and nothing else: an exact lookup in `routes`, then
/// `default`. No model call, no fuzzy match, no ordering sensitivity: the same
/// authenticated classification always resolves to the same node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Router {
    /// Exact top-level structured-output field containing the classification.
    #[serde(default)]
    pub on: String,
    /// Exact classification -> destination. `BTreeMap` so the persisted key
    /// order, and therefore any digest taken over it, is stable.
    #[serde(default)]
    pub routes: BTreeMap<String, NodeId>,
    /// Where an unmatched classification goes. `None` means an unmatched
    /// classification has NOWHERE to go, which is a legal graph: the caller is
    /// then required to handle the miss instead of silently drifting into a
    /// branch nobody chose.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<NodeId>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Router {
    pub fn new(on: impl Into<String>) -> Self {
        Self {
            on: on.into(),
            routes: BTreeMap::new(),
            default: None,
            extra: Map::new(),
        }
    }

    pub fn with_route(mut self, case: impl Into<String>, target: impl Into<NodeId>) -> Self {
        self.routes.insert(case.into(), target.into());
        self
    }

    pub fn with_default(mut self, target: impl Into<NodeId>) -> Self {
        self.default = Some(target.into());
        self
    }

    /// Resolve a classification. Exact match first, then the default.
    pub fn resolve(&self, classification: &str) -> Option<&NodeId> {
        self.routes.get(classification).or(self.default.as_ref())
    }

    /// Every node this router can reach, deduplicated and ordered.
    fn targets(&self) -> impl Iterator<Item = (&str, &NodeId)> {
        self.routes
            .iter()
            .map(|(case, target)| (case.as_str(), target))
    }
}

// ---------------------------------------------------------------------------
// LoopBound — what makes a cycle legal
// ---------------------------------------------------------------------------

/// An explicit, finite bound on one back edge.
///
/// A cycle is not forbidden here: loop-until-dry is a real and useful pattern.
/// What is forbidden is an UNBOUNDED one, because a cycle with no ceiling is a
/// machine that pays to rediscover the same dead ends until the budget dies
/// (R-LOOP, R-BUDGET). A bound declared on a back edge is the proof of
/// convergence: cut the bounded edges and what remains must be acyclic, so every
/// possible loop has to pass through a counter that runs out.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoopBound {
    pub from: NodeId,
    pub to: NodeId,
    /// Hard ceiling on traversals of this edge. Must be at least 1: a bound of 0
    /// does not describe a convergent loop, it describes an edge that can never
    /// be taken, which is almost always a defaulted field rather than an intent.
    pub max_iterations: u32,
    /// Stop after this many consecutive traversals that surfaced nothing new.
    /// Optional, and never a substitute for `max_iterations`: a dry-run counter
    /// only converges if the thing being deduplicated is deduplicated against
    /// everything SEEN, which this file cannot enforce.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_after_dry_rounds: Option<u32>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl LoopBound {
    pub fn new(from: impl Into<NodeId>, to: impl Into<NodeId>, max_iterations: u32) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            max_iterations,
            stop_after_dry_rounds: None,
            extra: Map::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// GraphError
// ---------------------------------------------------------------------------

/// Every way a graph can be structurally wrong. A typed enum, never a panic:
/// this validates data that arrives from disk and from other machines, and a
/// panic there takes down a daemon over a bad file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    UnsupportedSchema {
        version: u32,
    },
    EmptyGraph,
    EmptyNodeId,
    DuplicateNodeId(String),
    NonQueuedInitialState {
        node: String,
        state: String,
    },
    InvalidRetryPolicy(String),
    /// An edge endpoint that is not a node in this graph.
    DanglingEdge {
        from: String,
        to: String,
        missing: String,
    },
    /// A router attached to a node id that does not exist.
    UnknownRouterHost(String),
    /// A router with no machine-readable field contract.
    EmptyRouterField(String),
    /// A route pointing at a node that does not exist.
    UnknownRouterRoute {
        router: String,
        case: String,
        target: String,
    },
    /// A router default pointing at a node that does not exist.
    UnknownRouterDefault {
        router: String,
        target: String,
    },
    /// A route must select one of the host's immediate outgoing branches.
    RouterTargetNotOutgoing {
        router: String,
        target: String,
    },
    /// A loop bound declared on an edge the graph does not have.
    UnknownLoopBoundEdge {
        from: String,
        to: String,
    },
    /// Two counters on the same edge would make traversal authority ambiguous.
    DuplicateLoopBound {
        from: String,
        to: String,
    },
    /// A bound may cut only an edge that actually closes a directed cycle.
    LoopBoundIsNotBackEdge {
        from: String,
        to: String,
    },
    /// A loop bound that cannot converge to a useful traversal count.
    NonConvergentLoopBound {
        from: String,
        to: String,
    },
    /// A dry-round threshold of zero has no coherent observation semantics.
    NonConvergentDryLoopBound {
        from: String,
        to: String,
    },
    /// A cycle that survives after every bounded back edge is cut.
    UnboundedCycle {
        nodes: Vec<String>,
    },
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use GraphError::*;
        match self {
            UnsupportedSchema { version } => {
                write!(f, "unsupported graph schema version: {version}")
            }
            EmptyGraph => write!(f, "graph must contain at least one node"),
            EmptyNodeId => write!(f, "node id must not be empty"),
            DuplicateNodeId(id) => write!(f, "duplicate node id: {id}"),
            NonQueuedInitialState { node, state } => write!(
                f,
                "graph node {node} has initial state {state}; definitions must start queued"
            ),
            InvalidRetryPolicy(node) => {
                write!(f, "graph node {node} must allow at least one attempt")
            }
            DanglingEdge { from, to, missing } => {
                write!(f, "edge {from} -> {to} references unknown node {missing}")
            }
            UnknownRouterHost(id) => write!(f, "router attached to unknown node {id}"),
            EmptyRouterField(id) => {
                write!(
                    f,
                    "router {id} must declare a non-empty structured-output field"
                )
            }
            UnknownRouterRoute {
                router,
                case,
                target,
            } => write!(
                f,
                "router {router} route {case:?} points at unknown node {target}"
            ),
            UnknownRouterDefault { router, target } => {
                write!(f, "router {router} default points at unknown node {target}")
            }
            RouterTargetNotOutgoing { router, target } => write!(
                f,
                "router {router} target {target} is not an immediate outgoing branch"
            ),
            UnknownLoopBoundEdge { from, to } => {
                write!(f, "loop bound declared on missing edge {from} -> {to}")
            }
            DuplicateLoopBound { from, to } => {
                write!(f, "duplicate loop bound declared on edge {from} -> {to}")
            }
            LoopBoundIsNotBackEdge { from, to } => write!(
                f,
                "loop bound edge {from} -> {to} does not close a directed cycle"
            ),
            NonConvergentLoopBound { from, to } => write!(
                f,
                "loop bound on edge {from} -> {to} has max_iterations 0 and cannot converge"
            ),
            NonConvergentDryLoopBound { from, to } => write!(
                f,
                "loop bound on edge {from} -> {to} has stop_after_dry_rounds 0"
            ),
            UnboundedCycle { nodes } => write!(
                f,
                "graph contains an unbounded cycle through: {}",
                nodes.join(", ")
            ),
        }
    }
}

impl std::error::Error for GraphError {}

// ---------------------------------------------------------------------------
// Graph
// ---------------------------------------------------------------------------

/// The persisted shape of a mission.
///
/// Every field carries `#[serde(default)]` and the struct carries `extra`, so an
/// OLD file loads (missing fields default) and a NEW file round-trips through
/// this version without losing the keys it does not know (invariant 1 in the
/// module doc). No field here is ever rewritten or dropped on load.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Graph {
    #[serde(default = "graph_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub edges: Vec<Edge>,
    /// Routers keyed by the node that hosts them. Keying by host rather than
    /// carrying a router inline on [`Node`] keeps the branch table addressable
    /// on its own, and keeps `Node` the same shape whether or not it branches.
    #[serde(default)]
    pub routers: BTreeMap<NodeId, Router>,
    #[serde(default)]
    pub loop_bounds: Vec<LoopBound>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            schema_version: GRAPH_SCHEMA_VERSION,
            nodes: Vec::new(),
            edges: Vec::new(),
            routers: BTreeMap::new(),
            loop_bounds: Vec::new(),
            extra: Map::new(),
        }
    }
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_node(mut self, node: Node) -> Self {
        self.nodes.push(node);
        self
    }

    pub fn with_edge(mut self, from: impl Into<NodeId>, to: impl Into<NodeId>) -> Self {
        self.edges.push(Edge::new(from, to));
        self
    }

    pub fn with_router(mut self, host: impl Into<NodeId>, router: Router) -> Self {
        self.routers.insert(host.into(), router);
        self
    }

    pub fn with_loop_bound(mut self, bound: LoopBound) -> Self {
        self.loop_bounds.push(bound);
        self
    }

    pub fn node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.iter().find(|node| node.id == *id)
    }

    /// Stable fingerprint of the complete executable graph document.
    /// Object keys are canonicalized before hashing so JSON insertion order
    /// cannot create a different execution subject for the same graph.
    pub fn content_digest(&self) -> Result<String, serde_json::Error> {
        let value = canonicalize_json(serde_json::to_value(self)?);
        let bytes = serde_json::to_vec(&value)?;
        Ok(blake3::hash(&bytes).to_hex().to_string())
    }

    /// Resolve a classification at `host`. `None` means either no router lives
    /// there or the classification matched no route and there is no default.
    pub fn route(&self, host: &NodeId, classification: &str) -> Option<&NodeId> {
        self.routers.get(host)?.resolve(classification)
    }

    /// Reject every structurally invalid graph, with the reason.
    ///
    /// The order is deliberate: identity first (a duplicate id makes every later
    /// lookup ambiguous), then references, then reachability of the cycle rule,
    /// so the first error reported is the most upstream cause rather than a
    /// symptom of it.
    pub fn validate(&self) -> Result<(), GraphError> {
        if self.schema_version != GRAPH_SCHEMA_VERSION {
            return Err(GraphError::UnsupportedSchema {
                version: self.schema_version,
            });
        }
        if self.nodes.is_empty() {
            return Err(GraphError::EmptyGraph);
        }

        let mut ids: BTreeSet<&str> = BTreeSet::new();
        for node in &self.nodes {
            if node.id.0.trim().is_empty() {
                return Err(GraphError::EmptyNodeId);
            }
            if !ids.insert(node.id.as_str()) {
                return Err(GraphError::DuplicateNodeId(node.id.0.clone()));
            }
            if node.state != TaskAttemptState::Queued {
                return Err(GraphError::NonQueuedInitialState {
                    node: node.id.0.clone(),
                    state: format!("{:?}", node.state),
                });
            }
            if node.retry.max_attempts == 0 {
                return Err(GraphError::InvalidRetryPolicy(node.id.0.clone()));
            }
        }

        for edge in &self.edges {
            for endpoint in [&edge.from, &edge.to] {
                if !ids.contains(endpoint.as_str()) {
                    return Err(GraphError::DanglingEdge {
                        from: edge.from.0.clone(),
                        to: edge.to.0.clone(),
                        missing: endpoint.0.clone(),
                    });
                }
            }
        }

        for (host, router) in &self.routers {
            if !ids.contains(host.as_str()) {
                return Err(GraphError::UnknownRouterHost(host.0.clone()));
            }
            if router.on.trim().is_empty() {
                return Err(GraphError::EmptyRouterField(host.0.clone()));
            }
            for (case, target) in router.targets() {
                if !ids.contains(target.as_str()) {
                    return Err(GraphError::UnknownRouterRoute {
                        router: host.0.clone(),
                        case: case.to_string(),
                        target: target.0.clone(),
                    });
                }
                if !self.edges.iter().any(|edge| edge.same_ends(host, target)) {
                    return Err(GraphError::RouterTargetNotOutgoing {
                        router: host.0.clone(),
                        target: target.0.clone(),
                    });
                }
            }
            if let Some(target) = &router.default {
                if !ids.contains(target.as_str()) {
                    return Err(GraphError::UnknownRouterDefault {
                        router: host.0.clone(),
                        target: target.0.clone(),
                    });
                }
                if !self.edges.iter().any(|edge| edge.same_ends(host, target)) {
                    return Err(GraphError::RouterTargetNotOutgoing {
                        router: host.0.clone(),
                        target: target.0.clone(),
                    });
                }
            }
        }

        let mut bounded_edges = BTreeSet::new();
        for bound in &self.loop_bounds {
            if !self
                .edges
                .iter()
                .any(|edge| edge.same_ends(&bound.from, &bound.to))
            {
                return Err(GraphError::UnknownLoopBoundEdge {
                    from: bound.from.0.clone(),
                    to: bound.to.0.clone(),
                });
            }
            if !bounded_edges.insert((bound.from.as_str(), bound.to.as_str())) {
                return Err(GraphError::DuplicateLoopBound {
                    from: bound.from.0.clone(),
                    to: bound.to.0.clone(),
                });
            }
            if bound.max_iterations == 0 {
                return Err(GraphError::NonConvergentLoopBound {
                    from: bound.from.0.clone(),
                    to: bound.to.0.clone(),
                });
            }
            if bound.stop_after_dry_rounds == Some(0) {
                return Err(GraphError::NonConvergentDryLoopBound {
                    from: bound.from.0.clone(),
                    to: bound.to.0.clone(),
                });
            }
            if !directed_path_exists(self, &bound.to, &bound.from, |edge| {
                edge.same_ends(&bound.from, &bound.to)
            }) {
                return Err(GraphError::LoopBoundIsNotBackEdge {
                    from: bound.from.0.clone(),
                    to: bound.to.0.clone(),
                });
            }
        }

        self.check_cycles()
    }

    /// Cut every bounded back edge, then require what remains to be acyclic.
    ///
    /// Kahn's algorithm rather than a recursive walk: it needs no stack depth
    /// proportional to the graph, and the nodes it fails to emit ARE the ones
    /// caught in or fed by the cycle, which is the evidence the error carries.
    fn check_cycles(&self) -> Result<(), GraphError> {
        let bounded: BTreeSet<(&str, &str)> = self
            .loop_bounds
            .iter()
            .map(|bound| (bound.from.as_str(), bound.to.as_str()))
            .collect();

        let mut outgoing: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        let mut indegree: BTreeMap<&str, usize> = BTreeMap::new();
        for node in &self.nodes {
            outgoing.entry(node.id.as_str()).or_default();
            indegree.entry(node.id.as_str()).or_insert(0);
        }
        for edge in &self.edges {
            let pair = (edge.from.as_str(), edge.to.as_str());
            if bounded.contains(&pair) {
                continue;
            }
            outgoing.entry(pair.0).or_default().push(pair.1);
            *indegree.entry(pair.1).or_insert(0) += 1;
        }

        let mut queue: VecDeque<&str> = indegree
            .iter()
            .filter(|(_, degree)| **degree == 0)
            .map(|(id, _)| *id)
            .collect();
        let mut emitted = 0usize;
        while let Some(id) = queue.pop_front() {
            emitted += 1;
            for next in outgoing.get(id).into_iter().flatten() {
                let degree = indegree.entry(next).or_insert(0);
                *degree = degree.saturating_sub(1);
                if *degree == 0 {
                    queue.push_back(next);
                }
            }
        }

        if emitted == indegree.len() {
            return Ok(());
        }

        // Whatever still has an indegree is unreachable in topological order,
        // which is exactly the cycle plus what hangs off it.
        let nodes: Vec<String> = indegree
            .iter()
            .filter(|(_, degree)| **degree > 0)
            .map(|(id, _)| (*id).to_string())
            .collect();
        Err(GraphError::UnboundedCycle { nodes })
    }
}

/// Directed reachability with an explicit edge filter. The seed counts as a
/// zero-length path, which is what makes a bounded self-loop a legitimate
/// cycle: cutting `a -> a` still asks whether `a` reaches `a`.
fn directed_path_exists(
    graph: &Graph,
    from: &NodeId,
    to: &NodeId,
    mut excluded: impl FnMut(&Edge) -> bool,
) -> bool {
    let mut seen = BTreeSet::from([from.clone()]);
    let mut queue = VecDeque::from([from.clone()]);
    while let Some(current) = queue.pop_front() {
        if current == *to {
            return true;
        }
        for edge in &graph.edges {
            if edge.from == current && !excluded(edge) && seen.insert(edge.to.clone()) {
                queue.push_back(edge.to.clone());
            }
        }
    }
    false
}

fn canonicalize_json(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<String, Value> = map
                .into_iter()
                .map(|(key, value)| (key, canonicalize_json(value)))
                .collect();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize_json).collect()),
        other => other,
    }
}

/// Immutable identity of the Oracle mission plan that authorized a graph run.
/// Standalone graphs carry `None`; once a run is bound, every reservation and
/// downstream receipt incorporates [`Self::binding_digest`]. `plan_digest`
/// covers the plan's `required_gates` and `required_approvals`, so neither can
/// be edited without breaking this binding. This graph layer does not invent a
/// second plan-wide gate ledger: the CLI/mission ledger must prove those named
/// requirements before accepting the mission outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphMissionBinding {
    pub mission_id: MissionId,
    pub plan_id: PlanId,
    pub plan_revision: u64,
    pub plan_digest: String,
}

impl GraphMissionBinding {
    pub fn from_plan(plan: &PlanContract) -> Result<Self, GraphStateError> {
        plan.verify_integrity()
            .map_err(|error| GraphStateError::InvalidMissionBinding {
                reason: format!("plan contract is invalid: {error}"),
            })?;
        Ok(Self {
            mission_id: plan.mission_id.clone(),
            plan_id: plan.plan_id.clone(),
            plan_revision: plan.revision,
            plan_digest: plan.content_digest.clone(),
        })
    }

    pub fn binding_digest(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        for value in [
            self.mission_id.0.as_str(),
            self.plan_id.0.as_str(),
            self.plan_digest.as_str(),
        ] {
            hasher.update(&(value.len() as u64).to_le_bytes());
            hasher.update(value.as_bytes());
        }
        hasher.update(&self.plan_revision.to_le_bytes());
        hasher.finalize().to_hex().to_string()
    }

    fn validate_shape(&self) -> Result<(), GraphStateError> {
        if self.mission_id.0.trim().is_empty()
            || self.plan_id.0.trim().is_empty()
            || self.plan_revision == 0
            || self.plan_digest.len() != 64
            || !self
                .plan_digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(GraphStateError::InvalidMissionBinding {
                reason: "mission/plan ids, revision, or plan digest are malformed".to_string(),
            });
        }
        if self.plan_id != PlanId::for_mission(&self.mission_id) {
            return Err(GraphStateError::InvalidMissionBinding {
                reason: "plan_id is not derived from mission_id".to_string(),
            });
        }
        Ok(())
    }
}

fn mission_binding_digest(binding: Option<&GraphMissionBinding>) -> String {
    binding
        .map(GraphMissionBinding::binding_digest)
        .unwrap_or_default()
}

/// Validate the exact executable projection of an already authenticated plan.
/// Callers cross both trust boundaries first: [`Graph::validate`] through state
/// validation, and [`PlanContract::verify_integrity`] through
/// [`GraphMissionBinding::from_plan`].
fn validate_graph_plan_tasks(graph: &Graph, plan: &PlanContract) -> Result<(), GraphStateError> {
    let plan_tasks: BTreeMap<&str, &crate::mission::TaskContract> = plan
        .tasks
        .iter()
        .map(|task| (task.task_id.as_str(), task))
        .collect();
    let mut task_nodes: BTreeMap<&str, &Node> = BTreeMap::new();
    for node in &graph.nodes {
        let executable_lifecycle = matches!(
            node.kind,
            NodeKind::Agent | NodeKind::Verifier | NodeKind::Synthesis
        );
        let Some(task_id) = node.task_id.as_ref() else {
            if executable_lifecycle {
                return Err(GraphStateError::InvalidMissionBinding {
                    reason: format!(
                        "graph node {} ({:?}) has no task_id in a plan-bound run",
                        node.id, node.kind
                    ),
                });
            }
            continue;
        };
        let Some(task) = plan_tasks.get(task_id.as_str()) else {
            return Err(GraphStateError::InvalidMissionBinding {
                reason: format!(
                    "graph node {} references task {} absent from plan {}",
                    node.id, task_id.0, plan.plan_id.0
                ),
            });
        };
        if let Some(previous) = task_nodes.insert(task_id.as_str(), node) {
            return Err(GraphStateError::InvalidMissionBinding {
                reason: format!(
                    "plan task {} is mapped by both graph nodes {} and {}",
                    task_id.0, previous.id, node.id
                ),
            });
        }
        if !executable_lifecycle {
            return Err(GraphStateError::InvalidMissionBinding {
                reason: format!(
                    "graph node {} ({:?}) cannot own authoritative plan task {}",
                    node.id, node.kind, task_id.0
                ),
            });
        }
        if node.retry != task.retry_policy {
            return Err(GraphStateError::InvalidMissionBinding {
                reason: format!(
                    "graph node {} retry policy differs from immutable task {}",
                    node.id, task_id.0
                ),
            });
        }
        if node.checks != task.verifier_checks {
            return Err(GraphStateError::InvalidMissionBinding {
                reason: format!(
                    "graph node {} verifier checks differ from immutable task {}",
                    node.id, task_id.0
                ),
            });
        }
        let graph_risk = crate::graph_risk::risk_of(node).map_err(|error| {
            GraphStateError::InvalidMissionBinding {
                reason: format!(
                    "graph node {} has invalid risk for immutable task {}: {error}",
                    node.id, task_id.0
                ),
            }
        })?;
        let minimum = crate::graph_risk::minimum_for_plan_risk(task.risk);
        if graph_risk < minimum {
            return Err(GraphStateError::InvalidMissionBinding {
                reason: format!(
                    "graph node {} risk {} weakens immutable task {} risk {:?}; minimum is {}",
                    node.id, graph_risk, task_id.0, task.risk, minimum
                ),
            });
        }
    }

    for task in &plan.tasks {
        if !task_nodes.contains_key(task.task_id.as_str()) {
            return Err(GraphStateError::InvalidMissionBinding {
                reason: format!(
                    "authoritative plan task {} has no executable graph node",
                    task.task_id.0
                ),
            });
        }
    }

    for task in &plan.tasks {
        let task_node = task_nodes[task.task_id.as_str()];
        for dependency in &task.depends_on {
            let dependency_node = task_nodes[dependency.as_str()];
            let reachable =
                directed_path_exists(graph, &dependency_node.id, &task_node.id, |edge| {
                    graph
                        .loop_bounds
                        .iter()
                        .any(|bound| edge.same_ends(&bound.from, &bound.to))
                });
            if !reachable {
                return Err(GraphStateError::InvalidMissionBinding {
                    reason: format!(
                        "plan dependency {} -> {} is not preserved by graph reachability",
                        dependency.0, task.task_id.0
                    ),
                });
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// GraphState — the mutable half
// ---------------------------------------------------------------------------

/// Runtime state of one node: where it is, and how many attempts it has spent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeRunState {
    #[serde(default = "default_node_state")]
    pub state: TaskAttemptState,
    /// Attempts already spent. Compared against the node's
    /// [`RetryPolicy::max_attempts`] by whoever runs the graph; this file only
    /// records it, because counting is the executor's job and a type file that
    /// enforced the ceiling would be a second authority on R-LOOP.
    #[serde(default)]
    pub attempts: u32,
    /// Monotone identity of the dispatch reservation for this node. A retry or a
    /// new loop iteration gets a new generation, so a prior worker result cannot
    /// be mistaken for the current attempt.
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub generation: u64,
    /// The reservation currently authorized to produce a report for this node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reservation: Option<NodeReservation>,
    /// Durable proof that the accepted lifecycle was reached through a report
    /// answering an exact reservation, with every declared check receipt bound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance: Option<NodeAcceptanceReceipt>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for NodeRunState {
    fn default() -> Self {
        Self {
            state: default_node_state(),
            attempts: 0,
            generation: 0,
            reservation: None,
            acceptance: None,
            extra: Map::new(),
        }
    }
}

impl NodeRunState {
    /// Move this node's lifecycle through the mission state machine, for the
    /// same reason [`Node::transition`] does.
    pub fn transition(&mut self, next: TaskAttemptState) -> Result<(), InvalidTransition> {
        self.state = self.state.transition(next)?;
        Ok(())
    }
}

fn is_zero_u64(value: &u64) -> bool {
    *value == 0
}

/// Durable dispatch authority for exactly one node attempt in exactly one run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeReservation {
    pub node: NodeId,
    pub run_id: String,
    pub graph_digest: String,
    /// Empty for standalone runs; otherwise the typed Oracle plan binding.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub mission_binding_digest: String,
    pub generation: u64,
    pub state_version: u64,
    pub reservation_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub authority_mac: String,
}

impl NodeReservation {
    fn new(
        node: NodeId,
        run_id: String,
        graph_digest: String,
        mission_binding_digest: String,
        generation: u64,
        state_version: u64,
        authority: &GraphExecutionAuthority,
    ) -> Self {
        let reservation_id = Self::compute_id(
            &node,
            &run_id,
            &graph_digest,
            &mission_binding_digest,
            generation,
            state_version,
        );
        let authority_mac = authority.mac(
            "omega.graph.reservation.v1",
            &[
                reservation_id.as_bytes(),
                node.as_str().as_bytes(),
                run_id.as_bytes(),
                graph_digest.as_bytes(),
                mission_binding_digest.as_bytes(),
                &generation.to_le_bytes(),
                &state_version.to_le_bytes(),
            ],
        );
        Self {
            node,
            run_id,
            graph_digest,
            mission_binding_digest,
            generation,
            state_version,
            reservation_id,
            authority_mac,
        }
    }

    fn compute_id(
        node: &NodeId,
        run_id: &str,
        graph_digest: &str,
        mission_binding_digest: &str,
        generation: u64,
        state_version: u64,
    ) -> String {
        let mut hasher = blake3::Hasher::new();
        for value in [node.as_str(), run_id, graph_digest, mission_binding_digest] {
            hasher.update(&(value.len() as u64).to_le_bytes());
            hasher.update(value.as_bytes());
        }
        hasher.update(&generation.to_le_bytes());
        hasher.update(&state_version.to_le_bytes());
        hasher.finalize().to_hex().to_string()
    }

    /// Recompute the authority identifier from every field it binds.
    pub fn expected_id(&self) -> String {
        Self::compute_id(
            &self.node,
            &self.run_id,
            &self.graph_digest,
            &self.mission_binding_digest,
            self.generation,
            self.state_version,
        )
    }

    pub fn authenticate(&self, authority: &GraphExecutionAuthority) -> bool {
        self.authority_mac
            == authority.mac(
                "omega.graph.reservation.v1",
                &[
                    self.reservation_id.as_bytes(),
                    self.node.as_str().as_bytes(),
                    self.run_id.as_bytes(),
                    self.graph_digest.as_bytes(),
                    self.mission_binding_digest.as_bytes(),
                    &self.generation.to_le_bytes(),
                    &self.state_version.to_le_bytes(),
                ],
            )
    }
}

/// Persisted acceptance authority. It deliberately stores the consumed
/// reservation, because `NodeRunState::reservation` is cleared after applying a
/// report and a bare `Accepted` enum is not evidence that a worker ever ran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeAcceptanceReceipt {
    pub reservation: NodeReservation,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub check_receipt_ids: Vec<String>,
    pub acceptance_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub authority_mac: String,
}

impl NodeAcceptanceReceipt {
    fn new(
        reservation: NodeReservation,
        mut check_receipt_ids: Vec<String>,
        authority: &GraphExecutionAuthority,
    ) -> Self {
        check_receipt_ids.sort();
        let acceptance_id = Self::compute_id(&reservation, &check_receipt_ids);
        let mut fields: Vec<&[u8]> = vec![
            acceptance_id.as_bytes(),
            reservation.reservation_id.as_bytes(),
        ];
        fields.extend(check_receipt_ids.iter().map(|id| id.as_bytes()));
        let authority_mac = authority.mac("omega.graph.acceptance.v1", &fields);
        Self {
            reservation,
            check_receipt_ids,
            acceptance_id,
            authority_mac,
        }
    }

    fn compute_id(reservation: &NodeReservation, check_receipt_ids: &[String]) -> String {
        let mut hasher = blake3::Hasher::new();
        for value in std::iter::once(reservation.reservation_id.as_str())
            .chain(check_receipt_ids.iter().map(String::as_str))
        {
            hasher.update(&(value.len() as u64).to_le_bytes());
            hasher.update(value.as_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }

    pub fn expected_id(&self) -> String {
        Self::compute_id(&self.reservation, &self.check_receipt_ids)
    }

    pub fn authenticate(&self, authority: &GraphExecutionAuthority) -> bool {
        let mut fields: Vec<&[u8]> = vec![
            self.acceptance_id.as_bytes(),
            self.reservation.reservation_id.as_bytes(),
        ];
        fields.extend(self.check_receipt_ids.iter().map(|id| id.as_bytes()));
        self.authority_mac == authority.mac("omega.graph.acceptance.v1", &fields)
            && self.reservation.authenticate(authority)
    }
}

/// Why a persisted run state cannot be trusted for a graph definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphStateError {
    InvalidGraph(GraphError),
    UnsupportedSchema { version: u32 },
    EmptyRunId,
    EmptyGraphDigest,
    GraphDigestMismatch { expected: String, actual: String },
    EmptyNodeSet,
    MissingNode(String),
    UnknownNode(String),
    InvalidNodeState { node: String, reason: String },
    InvalidReservation { node: String, reason: String },
    InvalidMissionBinding { reason: String },
    MissionBindingMismatch { expected: String, actual: String },
}

impl fmt::Display for GraphStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGraph(error) => write!(f, "invalid graph: {error}"),
            Self::UnsupportedSchema { version } => {
                write!(f, "unsupported graph state schema version: {version}")
            }
            Self::EmptyRunId => write!(f, "graph state run_id must not be empty"),
            Self::EmptyGraphDigest => write!(f, "graph state graph_digest must not be empty"),
            Self::GraphDigestMismatch { expected, actual } => write!(
                f,
                "graph state digest mismatch: expected {expected}, actual {actual}"
            ),
            Self::EmptyNodeSet => write!(f, "graph state must contain every graph node"),
            Self::MissingNode(node) => write!(f, "graph state is missing node {node}"),
            Self::UnknownNode(node) => write!(f, "graph state contains unknown node {node}"),
            Self::InvalidNodeState { node, reason } => {
                write!(f, "graph state node {node} is invalid: {reason}")
            }
            Self::InvalidReservation { node, reason } => {
                write!(f, "graph state reservation for {node} is invalid: {reason}")
            }
            Self::InvalidMissionBinding { reason } => {
                write!(f, "graph mission binding is invalid: {reason}")
            }
            Self::MissionBindingMismatch { expected, actual } => write!(
                f,
                "graph mission binding mismatch: expected {expected}, actual {actual}"
            ),
        }
    }
}

impl std::error::Error for GraphStateError {}

impl From<GraphError> for GraphStateError {
    fn from(error: GraphError) -> Self {
        Self::InvalidGraph(error)
    }
}

/// Per-node execution state, persisted beside the [`Graph`] rather than inside
/// it: the graph is the immutable shape, this is what moves. Same forward
/// compatibility contract as [`Graph`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphState {
    #[serde(default = "graph_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub nodes: BTreeMap<NodeId, NodeRunState>,
    /// Unique identity of this execution, persisted across process restarts.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub run_id: String,
    /// Graph fingerprint this state was created for. A different graph must not
    /// inherit attempts, reservations, or approvals from this state document.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub graph_digest: String,
    /// Public identifier of the separately persisted authentication key.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub authority_id: String,
    /// Optional immutable Oracle mission/plan authority for this run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mission_binding: Option<GraphMissionBinding>,
    /// Monotone version of the mutable run state.
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub version: u64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for GraphState {
    fn default() -> Self {
        Self {
            schema_version: GRAPH_SCHEMA_VERSION,
            nodes: BTreeMap::new(),
            run_id: String::new(),
            graph_digest: String::new(),
            authority_id: String::new(),
            mission_binding: None,
            version: 0,
            extra: Map::new(),
        }
    }
}

impl GraphState {
    pub fn new() -> Self {
        Self::default()
    }

    /// A state map seeded from a graph: every node queued, zero attempts.
    pub fn for_graph(graph: &Graph) -> Self {
        Self::for_graph_with_run_id(graph, MissionId::new().0)
    }

    /// Deterministic constructor for callers that already own a durable run id.
    pub fn for_graph_with_run_id(graph: &Graph, run_id: impl Into<String>) -> Self {
        let mut state = Self::new();
        let supplied_run_id = run_id.into();
        state.run_id = if supplied_run_id.trim().is_empty() {
            MissionId::new().0
        } else {
            supplied_run_id
        };
        state.graph_digest = graph.content_digest().unwrap_or_default();
        for node in &graph.nodes {
            state.nodes.insert(
                node.id.clone(),
                NodeRunState {
                    state: node.state,
                    attempts: 0,
                    generation: 0,
                    reservation: None,
                    acceptance: None,
                    extra: Map::new(),
                },
            );
        }
        state
    }

    pub fn for_graph_with_authority(
        graph: &Graph,
        run_id: impl Into<String>,
        authority: &GraphExecutionAuthority,
    ) -> Self {
        let mut state = Self::for_graph_with_run_id(graph, run_id);
        state.authority_id = authority.authority_id();
        state
    }

    pub fn for_graph_with_plan(
        graph: &Graph,
        run_id: impl Into<String>,
        plan: &PlanContract,
    ) -> Result<Self, GraphStateError> {
        let mut state = Self::for_graph_with_run_id(graph, run_id);
        state.bind_to_plan(graph, plan)?;
        Ok(state)
    }

    pub fn for_graph_with_plan_and_authority(
        graph: &Graph,
        run_id: impl Into<String>,
        plan: &PlanContract,
        authority: &GraphExecutionAuthority,
    ) -> Result<Self, GraphStateError> {
        let mut state = Self::for_graph_with_authority(graph, run_id, authority);
        state.bind_to_plan(graph, plan)?;
        Ok(state)
    }

    /// Bind a pristine standalone run to one immutable plan revision. Rebinding
    /// to the same plan is idempotent; changing or removing it requires a new
    /// run, so existing reservations can never cross Oracle plan revisions.
    pub fn bind_to_plan(
        &mut self,
        graph: &Graph,
        plan: &PlanContract,
    ) -> Result<(), GraphStateError> {
        self.validate_for_graph(graph)?;
        let binding = GraphMissionBinding::from_plan(plan)?;
        validate_graph_plan_tasks(graph, plan)?;
        match &self.mission_binding {
            Some(existing) if existing == &binding => return Ok(()),
            Some(existing) => {
                return Err(GraphStateError::MissionBindingMismatch {
                    expected: existing.binding_digest(),
                    actual: binding.binding_digest(),
                })
            }
            None => {}
        }
        let pristine = self.nodes.values().all(|run| {
            run.state == TaskAttemptState::Queued
                && run.attempts == 0
                && run.generation == 0
                && run.reservation.is_none()
                && run.acceptance.is_none()
        });
        if !pristine {
            return Err(GraphStateError::InvalidMissionBinding {
                reason: "cannot bind a plan after graph execution has started".to_string(),
            });
        }
        self.mission_binding = Some(binding);
        self.version = self.version.saturating_add(1);
        Ok(())
    }

    /// Prove this state is bound to the exact supplied immutable plan revision.
    pub fn validate_plan_binding(
        &self,
        graph: &Graph,
        plan: &PlanContract,
    ) -> Result<(), GraphStateError> {
        self.validate_for_graph(graph)?;
        let expected = GraphMissionBinding::from_plan(plan)?;
        validate_graph_plan_tasks(graph, plan)?;
        match &self.mission_binding {
            Some(actual) if actual == &expected => Ok(()),
            Some(actual) => Err(GraphStateError::MissionBindingMismatch {
                expected: expected.binding_digest(),
                actual: actual.binding_digest(),
            }),
            None => Err(GraphStateError::MissionBindingMismatch {
                expected: expected.binding_digest(),
                actual: "standalone".to_string(),
            }),
        }
    }

    pub fn state_of(&self, id: &NodeId) -> Option<TaskAttemptState> {
        self.nodes.get(id).map(|node| node.state)
    }

    pub fn attempts_of(&self, id: &NodeId) -> u32 {
        self.nodes.get(id).map(|node| node.attempts).unwrap_or(0)
    }

    pub fn reservation_of(&self, id: &NodeId) -> Option<&NodeReservation> {
        self.nodes.get(id)?.reservation.as_ref()
    }

    /// Validate a deserialized run before it can authorize execution.
    pub fn validate_for_graph(&self, graph: &Graph) -> Result<(), GraphStateError> {
        graph.validate()?;
        if self.schema_version != GRAPH_SCHEMA_VERSION {
            return Err(GraphStateError::UnsupportedSchema {
                version: self.schema_version,
            });
        }
        if self.run_id.trim().is_empty() {
            return Err(GraphStateError::EmptyRunId);
        }
        if self.graph_digest.trim().is_empty() {
            return Err(GraphStateError::EmptyGraphDigest);
        }
        if self.nodes.is_empty() {
            return Err(GraphStateError::EmptyNodeSet);
        }
        if let Some(binding) = &self.mission_binding {
            binding.validate_shape()?;
        }
        let expected_binding_digest = mission_binding_digest(self.mission_binding.as_ref());

        let actual_digest =
            graph
                .content_digest()
                .map_err(|error| GraphStateError::InvalidNodeState {
                    node: "<graph>".to_string(),
                    reason: format!("cannot fingerprint graph: {error}"),
                })?;
        if self.graph_digest != actual_digest {
            return Err(GraphStateError::GraphDigestMismatch {
                expected: self.graph_digest.clone(),
                actual: actual_digest,
            });
        }

        let graph_ids: BTreeSet<&NodeId> = graph.nodes.iter().map(|node| &node.id).collect();
        for node in &graph.nodes {
            let run = self
                .nodes
                .get(&node.id)
                .ok_or_else(|| GraphStateError::MissingNode(node.id.0.clone()))?;
            if run.attempts > node.retry.max_attempts {
                return Err(GraphStateError::InvalidNodeState {
                    node: node.id.0.clone(),
                    reason: format!(
                        "attempt count {} exceeds retry ceiling {}",
                        run.attempts, node.retry.max_attempts
                    ),
                });
            }
            if run.state == TaskAttemptState::CorrectionRequired
                && run.attempts >= node.retry.max_attempts
            {
                return Err(GraphStateError::InvalidNodeState {
                    node: node.id.0.clone(),
                    reason: "correction_required has no retry budget remaining".to_string(),
                });
            }

            match &run.reservation {
                Some(reservation) => {
                    if run.state != TaskAttemptState::Running {
                        return Err(GraphStateError::InvalidReservation {
                            node: node.id.0.clone(),
                            reason: format!(
                                "reservation exists while lifecycle is {:?}",
                                run.state
                            ),
                        });
                    }
                    if run.generation == 0
                        || reservation.node != node.id
                        || reservation.run_id != self.run_id
                        || reservation.graph_digest != self.graph_digest
                        || reservation.mission_binding_digest != expected_binding_digest
                        || reservation.generation != run.generation
                        || reservation.state_version == 0
                        || reservation.state_version > self.version
                    {
                        return Err(GraphStateError::InvalidReservation {
                            node: node.id.0.clone(),
                            reason: "identity, run, generation, or state version mismatch"
                                .to_string(),
                        });
                    }
                    let expected = reservation.expected_id();
                    if reservation.reservation_id != expected {
                        return Err(GraphStateError::InvalidReservation {
                            node: node.id.0.clone(),
                            reason: format!(
                                "reservation_id mismatch: expected {expected}, supplied {}",
                                reservation.reservation_id
                            ),
                        });
                    }
                }
                None if run.state == TaskAttemptState::Running => {
                    return Err(GraphStateError::InvalidReservation {
                        node: node.id.0.clone(),
                        reason: "running node has no active reservation".to_string(),
                    });
                }
                None => {}
            }
            match &run.acceptance {
                Some(acceptance) => {
                    if run.state != TaskAttemptState::Accepted {
                        return Err(GraphStateError::InvalidNodeState {
                            node: node.id.0.clone(),
                            reason: format!(
                                "acceptance receipt exists while lifecycle is {:?}",
                                run.state
                            ),
                        });
                    }
                    let reservation = &acceptance.reservation;
                    if reservation.node != node.id
                        || reservation.run_id != self.run_id
                        || reservation.graph_digest != self.graph_digest
                        || reservation.mission_binding_digest != expected_binding_digest
                        || reservation.generation != run.generation
                        || reservation.state_version == 0
                        || reservation.state_version > self.version
                        || reservation.reservation_id != reservation.expected_id()
                    {
                        return Err(GraphStateError::InvalidNodeState {
                            node: node.id.0.clone(),
                            reason: "acceptance receipt reservation is inconsistent".to_string(),
                        });
                    }
                    let unique: BTreeSet<&str> = acceptance
                        .check_receipt_ids
                        .iter()
                        .map(String::as_str)
                        .collect();
                    if unique.len() != acceptance.check_receipt_ids.len()
                        || unique.iter().any(|id| id.trim().is_empty())
                        || unique.len() != node.checks.len()
                    {
                        return Err(GraphStateError::InvalidNodeState {
                            node: node.id.0.clone(),
                            reason: "acceptance check receipt set does not match declared checks"
                                .to_string(),
                        });
                    }
                    if acceptance.acceptance_id != acceptance.expected_id() {
                        return Err(GraphStateError::InvalidNodeState {
                            node: node.id.0.clone(),
                            reason: "acceptance_id does not match its contents".to_string(),
                        });
                    }
                }
                None if run.state == TaskAttemptState::Accepted => {
                    return Err(GraphStateError::InvalidNodeState {
                        node: node.id.0.clone(),
                        reason: "accepted node has no durable acceptance receipt".to_string(),
                    });
                }
                None => {}
            }
        }
        if let Some(unknown) = self.nodes.keys().find(|id| !graph_ids.contains(id)) {
            return Err(GraphStateError::UnknownNode(unknown.0.clone()));
        }
        Ok(())
    }

    /// Authenticate every execution authority after structural validation.
    /// Callers must use this boundary before resuming an editable state file.
    pub fn validate_for_graph_with_authority(
        &self,
        graph: &Graph,
        authority: &GraphExecutionAuthority,
    ) -> Result<(), GraphStateError> {
        self.validate_for_graph(graph)?;
        if self.authority_id != authority.authority_id() {
            return Err(GraphStateError::InvalidNodeState {
                node: "<graph>".to_string(),
                reason: "execution authority does not match persisted authority_id".to_string(),
            });
        }
        for (id, run) in &self.nodes {
            if let Some(reservation) = &run.reservation {
                if !reservation.authenticate(authority) {
                    return Err(GraphStateError::InvalidReservation {
                        node: id.0.clone(),
                        reason: "reservation authority MAC is invalid".to_string(),
                    });
                }
            }
            if let Some(acceptance) = &run.acceptance {
                if !acceptance.authenticate(authority) {
                    return Err(GraphStateError::InvalidNodeState {
                        node: id.0.clone(),
                        reason: "acceptance authority MAC is invalid".to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    pub(crate) fn bind_authority_if_pristine(
        &mut self,
        authority: &GraphExecutionAuthority,
    ) -> Result<(), GraphStateError> {
        let expected = authority.authority_id();
        if self.authority_id == expected {
            return Ok(());
        }
        if !self.authority_id.is_empty() {
            return Err(GraphStateError::InvalidNodeState {
                node: "<graph>".to_string(),
                reason: "execution authority does not match persisted authority_id".to_string(),
            });
        }
        let pristine = self.version == 0
            && self.nodes.values().all(|run| {
                run.state == TaskAttemptState::Queued
                    && run.attempts == 0
                    && run.generation == 0
                    && run.reservation.is_none()
                    && run.acceptance.is_none()
            });
        if !pristine {
            return Err(GraphStateError::InvalidNodeState {
                node: "<graph>".to_string(),
                reason: "cannot bind an authority to a non-pristine run".to_string(),
            });
        }
        self.authority_id = expected;
        self.version = self.version.saturating_add(1);
        Ok(())
    }

    /// Count one more attempt at `id`, creating the entry if the node was never
    /// seen before. `saturating_add` because a wrapped counter would silently
    /// hand a thrashing node a fresh budget.
    pub fn record_attempt(&mut self, id: &NodeId) -> u32 {
        let entry = self.nodes.entry(id.clone()).or_default();
        entry.attempts = entry.attempts.saturating_add(1);
        self.version = self.version.saturating_add(1);
        entry.attempts
    }

    /// Move `id` through the mission state machine.
    pub fn transition(
        &mut self,
        id: &NodeId,
        next: TaskAttemptState,
    ) -> Result<TaskAttemptState, InvalidTransition> {
        let entry = self.nodes.entry(id.clone()).or_default();
        entry.transition(next)?;
        self.version = self.version.saturating_add(1);
        Ok(entry.state)
    }

    pub(crate) fn reserve(
        &mut self,
        id: &NodeId,
        authority: &GraphExecutionAuthority,
    ) -> Result<NodeReservation, InvalidTransition> {
        let next_version = self.version.saturating_add(1);
        let binding_digest = mission_binding_digest(self.mission_binding.as_ref());
        let entry = self.nodes.entry(id.clone()).or_default();
        entry.transition(TaskAttemptState::Running)?;
        entry.generation = entry.generation.saturating_add(1);
        entry.acceptance = None;
        let reservation = NodeReservation::new(
            id.clone(),
            self.run_id.clone(),
            self.graph_digest.clone(),
            binding_digest,
            entry.generation,
            next_version,
            authority,
        );
        entry.reservation = Some(reservation.clone());
        self.version = next_version;
        Ok(reservation)
    }

    pub(crate) fn clear_reservation(&mut self, id: &NodeId) {
        if let Some(entry) = self.nodes.get_mut(id) {
            if entry.reservation.take().is_some() {
                self.version = self.version.saturating_add(1);
            }
        }
    }

    pub(crate) fn record_acceptance(
        &mut self,
        id: &NodeId,
        check_receipt_ids: Vec<String>,
        authority: &GraphExecutionAuthority,
    ) -> Result<(), GraphStateError> {
        let entry = self
            .nodes
            .get_mut(id)
            .ok_or_else(|| GraphStateError::InvalidNodeState {
                node: id.0.clone(),
                reason: "cannot record acceptance for missing node".to_string(),
            })?;
        if entry.state != TaskAttemptState::Accepted {
            return Err(GraphStateError::InvalidNodeState {
                node: id.0.clone(),
                reason: "acceptance can only be recorded in accepted state".to_string(),
            });
        }
        let reservation =
            entry
                .reservation
                .clone()
                .ok_or_else(|| GraphStateError::InvalidNodeState {
                    node: id.0.clone(),
                    reason: "acceptance requires the consumed reservation".to_string(),
                })?;
        entry.acceptance = Some(NodeAcceptanceReceipt::new(
            reservation,
            check_receipt_ids,
            authority,
        ));
        self.version = self.version.saturating_add(1);
        Ok(())
    }

    pub(crate) fn mark_updated(&mut self) {
        self.version = self.version.saturating_add(1);
    }

    pub(crate) fn reseed(&mut self, id: &NodeId) {
        let entry = self.nodes.entry(id.clone()).or_default();
        entry.state = TaskAttemptState::Queued;
        entry.reservation = None;
        entry.acceptance = None;
        self.version = self.version.saturating_add(1);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mission::{TaskContract, VerifierCheckKind, CONTRACT_SCHEMA_VERSION};

    /// fan-out -> reduce -> synthesis, the workhorse diamond, with a router in
    /// front of the two finders.
    fn valid_graph() -> Graph {
        Graph::new()
            .with_node(Node::new("classify", NodeKind::Router))
            .with_node(Node::new("find_bugs", NodeKind::Agent))
            .with_node(Node::new("find_perf", NodeKind::Agent))
            .with_node(Node::new("dedupe", NodeKind::Reduce))
            .with_node(Node::new("synthesize", NodeKind::Synthesis))
            .with_edge("classify", "find_bugs")
            .with_edge("classify", "find_perf")
            .with_edge("find_bugs", "dedupe")
            .with_edge("find_perf", "dedupe")
            .with_edge("dedupe", "synthesize")
            .with_router(
                "classify",
                Router::new("finding_kind")
                    .with_route("bug", "find_bugs")
                    .with_route("perf", "find_perf")
                    .with_default("find_bugs"),
            )
    }

    fn task_contract(
        task_id: &str,
        risk: crate::routing::RiskLevel,
        depends_on: &[&str],
    ) -> TaskContract {
        TaskContract {
            schema_version: CONTRACT_SCHEMA_VERSION,
            task_id: TaskId::new(task_id),
            name: task_id.to_string(),
            prompt: format!("execute {task_id}"),
            acceptance_criteria: vec!["runtime verified".to_string()],
            verifier_checks: vec![VerifierCheck {
                schema_version: CONTRACT_SCHEMA_VERSION,
                check_id: "verify".to_string(),
                kind: VerifierCheckKind::FileExists {
                    path: "result.json".to_string(),
                },
                timeout_secs: 10,
            }],
            required_capabilities: Vec::new(),
            scope: Vec::new(),
            risk,
            retry_policy: RetryPolicy::default(),
            depends_on: depends_on.iter().map(|id| TaskId::new(*id)).collect(),
        }
    }

    fn plan_with_tasks(mission: &str, revision: u64, tasks: Vec<TaskContract>) -> PlanContract {
        PlanContract::new(
            MissionId(mission.to_string()),
            revision,
            revision,
            tasks,
            Vec::new(),
            Vec::new(),
        )
        .unwrap()
    }

    fn plan(mission: &str, revision: u64, task_id: &str) -> PlanContract {
        plan_with_tasks(
            mission,
            revision,
            vec![task_contract(
                task_id,
                crate::routing::RiskLevel::Medium,
                &[],
            )],
        )
    }

    fn bound_node(id: &str, kind: NodeKind, task: &TaskContract) -> Node {
        Node::new(id, kind)
            .with_task(task.task_id.clone())
            .with_retry(task.retry_policy.clone())
            .with_checks(task.verifier_checks.clone())
    }

    #[test]
    fn valid_graph_validates_clean() {
        let graph = valid_graph();
        assert_eq!(graph.validate(), Ok(()));
        assert_eq!(graph.schema_version, GRAPH_SCHEMA_VERSION);
    }

    #[test]
    fn unknown_future_field_survives_round_trip() {
        // A graph written by a LATER OmegaOS: fields this version has never
        // heard of, at the graph level, on a node, on an edge, on a router, and
        // in the state document.
        let json = serde_json::json!({
            "schema_version": GRAPH_SCHEMA_VERSION,
            // Every known field is spelled out so the equality below is exact:
            // a defaulted field that serde re-emits is not a dropped key, and
            // omitting them here would test the wrong thing. The old-file case
            // (fields absent entirely) is `old_file_without_added_fields_loads`.
            "nodes": [{
                "id": "a",
                "kind": "agent",
                "retry": {"max_attempts": 3, "backoff_secs": 5},
                "checks": [],
                "state": "queued",
                "budget_tokens": 50_000,
            }],
            "edges": [{"from": "a", "to": "a", "carries": "findings"}],
            "routers": {
                "a": {"on": "verdict", "routes": {}, "weighting": "uniform"}
            },
            "loop_bounds": [{
                "from": "a",
                "to": "a",
                "max_iterations": 3,
                "dry_streak_policy": "strict",
            }],
            "provenance": {"authored_by": "oracle-OmegaOS-9"},
        });

        let graph: Graph =
            serde_json::from_value(json.clone()).expect("old reader parses new file");
        assert_eq!(graph.validate(), Ok(()));

        let round_tripped = serde_json::to_value(&graph).expect("serializes");
        assert_eq!(round_tripped, json, "no key may be dropped on round trip");

        // And the unknown keys are reachable, not merely preserved by accident.
        assert_eq!(
            graph.nodes[0].extra.get("budget_tokens"),
            Some(&Value::from(50_000))
        );
        assert!(graph.extra.contains_key("provenance"));
        assert!(graph.edges[0].extra.contains_key("carries"));
        assert!(graph.routers[&NodeId::new("a")]
            .extra
            .contains_key("weighting"));

        // Same contract for the state document.
        let state_json = serde_json::json!({
            "schema_version": GRAPH_SCHEMA_VERSION,
            "nodes": {"a": {"state": "running", "attempts": 2, "last_worker": "w-1"}},
            "run_id": "run-7",
        });
        let state: GraphState = serde_json::from_value(state_json.clone()).expect("state parses");
        assert_eq!(
            serde_json::to_value(&state).expect("serializes"),
            state_json
        );
    }

    #[test]
    fn old_file_without_added_fields_loads() {
        // No schema_version, no retry, no checks, no state, no routers: the
        // shape a graph had before those fields existed.
        let graph: Graph = serde_json::from_value(serde_json::json!({
            "nodes": [{"id": "a", "kind": "agent"}, {"id": "b", "kind": "reduce"}],
            "edges": [{"from": "a", "to": "b"}],
        }))
        .expect("old file parses");

        assert_eq!(graph.schema_version, GRAPH_SCHEMA_VERSION);
        assert_eq!(graph.nodes[0].retry, RetryPolicy::default());
        assert_eq!(graph.nodes[0].state, TaskAttemptState::Queued);
        assert!(graph.nodes[0].checks.is_empty());
        assert_eq!(graph.validate(), Ok(()));
    }

    #[test]
    fn duplicate_node_id_rejected() {
        let graph = Graph::new()
            .with_node(Node::new("a", NodeKind::Agent))
            .with_node(Node::new("a", NodeKind::Reduce));

        assert_eq!(
            graph.validate(),
            Err(GraphError::DuplicateNodeId("a".to_string()))
        );
    }

    #[test]
    fn graph_definition_must_be_nonempty_and_start_every_node_queued() {
        assert_eq!(Graph::new().validate(), Err(GraphError::EmptyGraph));

        let mut node = Node::new("already-running", NodeKind::Agent);
        node.state = TaskAttemptState::Running;
        assert_eq!(
            Graph::new().with_node(node).validate(),
            Err(GraphError::NonQueuedInitialState {
                node: "already-running".to_string(),
                state: "Running".to_string(),
            })
        );
    }

    #[test]
    fn dangling_edge_rejected() {
        let graph = Graph::new()
            .with_node(Node::new("a", NodeKind::Agent))
            .with_edge("a", "ghost");

        assert_eq!(
            graph.validate(),
            Err(GraphError::DanglingEdge {
                from: "a".to_string(),
                to: "ghost".to_string(),
                missing: "ghost".to_string(),
            })
        );
    }

    #[test]
    fn router_route_to_missing_node_rejected() {
        let graph = Graph::new()
            .with_node(Node::new("classify", NodeKind::Router))
            .with_node(Node::new("known", NodeKind::Agent))
            .with_router(
                "classify",
                Router::new("verdict")
                    .with_route("keep", "known")
                    .with_route("escalate", "ghost"),
            );

        assert_eq!(
            graph.validate(),
            Err(GraphError::UnknownRouterRoute {
                router: "classify".to_string(),
                case: "escalate".to_string(),
                target: "ghost".to_string(),
            })
        );
    }

    #[test]
    fn router_requires_a_structured_field_and_direct_branch_targets() {
        let empty_field = Graph::new()
            .with_node(Node::new("classify", NodeKind::Router))
            .with_router("classify", Router::new("   "));
        assert_eq!(
            empty_field.validate(),
            Err(GraphError::EmptyRouterField("classify".to_string()))
        );

        let indirect = Graph::new()
            .with_node(Node::new("classify", NodeKind::Router))
            .with_node(Node::new("middle", NodeKind::Reduce))
            .with_node(Node::new("branch", NodeKind::Agent))
            .with_edge("classify", "middle")
            .with_edge("middle", "branch")
            .with_router("classify", Router::new("kind").with_route("a", "branch"));
        assert_eq!(
            indirect.validate(),
            Err(GraphError::RouterTargetNotOutgoing {
                router: "classify".to_string(),
                target: "branch".to_string(),
            })
        );
    }

    #[test]
    fn unbounded_cycle_rejected() {
        let graph = Graph::new()
            .with_node(Node::new("find", NodeKind::Agent))
            .with_node(Node::new("verify", NodeKind::Verifier))
            .with_edge("find", "verify")
            .with_edge("verify", "find");

        match graph.validate() {
            Err(GraphError::UnboundedCycle { nodes }) => {
                assert_eq!(nodes, vec!["find".to_string(), "verify".to_string()]);
            }
            other => panic!("expected an unbounded cycle, got {other:?}"),
        }
    }

    #[test]
    fn bounded_cycle_accepted() {
        // Loop-until-dry: the back edge carries an explicit finite ceiling, so
        // the loop is provably convergent and therefore legal.
        let graph = Graph::new()
            .with_node(Node::new("find", NodeKind::Agent))
            .with_node(Node::new("dedupe", NodeKind::Reduce))
            .with_node(Node::new("verify", NodeKind::Verifier))
            .with_edge("find", "dedupe")
            .with_edge("dedupe", "verify")
            .with_edge("verify", "find")
            .with_loop_bound(LoopBound::new("verify", "find", 3));

        assert_eq!(graph.validate(), Ok(()));
    }

    #[test]
    fn forward_edge_cannot_be_misdeclared_as_a_loop_bound() {
        let graph = Graph::new()
            .with_node(Node::new("a", NodeKind::Agent))
            .with_node(Node::new("b", NodeKind::Agent))
            .with_edge("a", "b")
            .with_loop_bound(LoopBound::new("a", "b", 1));

        assert_eq!(
            graph.validate(),
            Err(GraphError::LoopBoundIsNotBackEdge {
                from: "a".to_string(),
                to: "b".to_string(),
            })
        );
    }

    #[test]
    fn duplicate_loop_bounds_are_rejected() {
        let graph = Graph::new()
            .with_node(Node::new("a", NodeKind::Agent))
            .with_node(Node::new("b", NodeKind::Agent))
            .with_edge("a", "b")
            .with_edge("b", "a")
            .with_loop_bound(LoopBound::new("b", "a", 2))
            .with_loop_bound(LoopBound::new("b", "a", 3));

        assert_eq!(
            graph.validate(),
            Err(GraphError::DuplicateLoopBound {
                from: "b".to_string(),
                to: "a".to_string(),
            })
        );
    }

    #[test]
    fn zero_iteration_bound_does_not_legalize_a_cycle() {
        let graph = Graph::new()
            .with_node(Node::new("find", NodeKind::Agent))
            .with_node(Node::new("verify", NodeKind::Verifier))
            .with_edge("find", "verify")
            .with_edge("verify", "find")
            .with_loop_bound(LoopBound::new("verify", "find", 0));

        assert_eq!(
            graph.validate(),
            Err(GraphError::NonConvergentLoopBound {
                from: "verify".to_string(),
                to: "find".to_string(),
            })
        );
    }

    #[test]
    fn zero_dry_round_threshold_is_rejected() {
        let mut bound = LoopBound::new("verify", "find", 3);
        bound.stop_after_dry_rounds = Some(0);
        let graph = Graph::new()
            .with_node(Node::new("find", NodeKind::Agent))
            .with_node(Node::new("verify", NodeKind::Verifier))
            .with_edge("find", "verify")
            .with_edge("verify", "find")
            .with_loop_bound(bound);
        assert_eq!(
            graph.validate(),
            Err(GraphError::NonConvergentDryLoopBound {
                from: "verify".to_string(),
                to: "find".to_string(),
            })
        );
    }

    #[test]
    fn loop_bound_on_a_missing_edge_rejected() {
        let graph = Graph::new()
            .with_node(Node::new("a", NodeKind::Agent))
            .with_node(Node::new("b", NodeKind::Agent))
            .with_edge("a", "b")
            .with_loop_bound(LoopBound::new("b", "a", 2));

        assert_eq!(
            graph.validate(),
            Err(GraphError::UnknownLoopBoundEdge {
                from: "b".to_string(),
                to: "a".to_string(),
            })
        );
    }

    #[test]
    fn router_resolves_deterministically_and_falls_back_to_default() {
        let graph = valid_graph();
        let host = NodeId::new("classify");

        // Same classification, same node, every time.
        for _ in 0..8 {
            assert_eq!(graph.route(&host, "perf"), Some(&NodeId::new("find_perf")));
        }
        assert_eq!(graph.route(&host, "bug"), Some(&NodeId::new("find_bugs")));

        // Unknown classification falls back to the declared default.
        assert_eq!(
            graph.route(&host, "never_seen"),
            Some(&NodeId::new("find_bugs"))
        );

        // A router with no default reports the miss instead of guessing.
        let strict = Router::new("verdict").with_route("keep", "known");
        assert_eq!(strict.resolve("keep"), Some(&NodeId::new("known")));
        assert_eq!(strict.resolve("drop"), None);

        // A node with no router is a miss, not a panic.
        assert_eq!(graph.route(&NodeId::new("dedupe"), "bug"), None);
    }

    #[test]
    fn node_state_reuses_the_mission_attempt_machine() {
        let mut node = Node::new("a", NodeKind::Agent);
        assert_eq!(node.state, TaskAttemptState::Queued);
        node.transition(TaskAttemptState::Running).expect("legal");
        assert!(node.transition(TaskAttemptState::Accepted).is_err());
        assert_eq!(node.state, TaskAttemptState::Running);
    }

    #[test]
    fn graph_state_tracks_attempts_per_node() {
        let graph = valid_graph();
        let mut state = GraphState::for_graph(&graph);
        let id = NodeId::new("find_bugs");

        assert_eq!(state.state_of(&id), Some(TaskAttemptState::Queued));
        assert_eq!(state.record_attempt(&id), 1);
        assert_eq!(state.record_attempt(&id), 2);
        assert_eq!(state.attempts_of(&id), 2);
        assert_eq!(
            state.transition(&id, TaskAttemptState::Running),
            Ok(TaskAttemptState::Running)
        );
        assert!(state.transition(&id, TaskAttemptState::Accepted).is_err());
        assert_eq!(state.attempts_of(&NodeId::new("dedupe")), 0);
    }

    #[test]
    fn mission_plan_binding_is_typed_immutable_and_bound_into_reservations() {
        let plan_v1 = plan("m-bound", 1, "task-a");
        let graph = Graph::new().with_node(
            Node::new("work", NodeKind::Agent)
                .with_task(TaskId::new("task-a"))
                .with_retry(plan_v1.tasks[0].retry_policy.clone())
                .with_checks(plan_v1.tasks[0].verifier_checks.clone()),
        );
        let authority = GraphExecutionAuthority::from_key([0x61; 32]);
        let mut state = GraphState::for_graph_with_plan_and_authority(
            &graph,
            "run-bound",
            &plan_v1,
            &authority,
        )
        .unwrap();
        state.validate_plan_binding(&graph, &plan_v1).unwrap();
        let binding_digest = state.mission_binding.as_ref().unwrap().binding_digest();
        let reservation = state.reserve(&NodeId::new("work"), &authority).unwrap();
        assert_eq!(reservation.mission_binding_digest, binding_digest);
        assert!(reservation.authenticate(&authority));
        state
            .validate_for_graph_with_authority(&graph, &authority)
            .unwrap();

        let encoded = serde_json::to_value(&state).unwrap();
        let restored: GraphState = serde_json::from_value(encoded).unwrap();
        restored.validate_plan_binding(&graph, &plan_v1).unwrap();

        let plan_v2 = plan("m-bound", 2, "task-a");
        assert!(matches!(
            restored.validate_plan_binding(&graph, &plan_v2),
            Err(GraphStateError::MissionBindingMismatch { .. })
        ));
        let mut cannot_rebind = restored.clone();
        assert!(matches!(
            cannot_rebind.bind_to_plan(&graph, &plan_v2),
            Err(GraphStateError::MissionBindingMismatch { .. })
        ));
    }

    #[test]
    fn mission_binding_digest_covers_plan_wide_gates_and_approvals() {
        let tasks = vec![task_contract(
            "task-a",
            crate::routing::RiskLevel::Medium,
            &[],
        )];
        let unguarded = PlanContract::new(
            MissionId("m-plan-gates".to_string()),
            1,
            1,
            tasks.clone(),
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
        let guarded = PlanContract::new(
            MissionId("m-plan-gates".to_string()),
            1,
            1,
            tasks,
            vec!["runtime-security-gate".to_string()],
            vec!["operator-approval".to_string()],
        )
        .unwrap();

        assert_ne!(unguarded.content_digest, guarded.content_digest);
        assert_ne!(
            GraphMissionBinding::from_plan(&unguarded)
                .unwrap()
                .binding_digest(),
            GraphMissionBinding::from_plan(&guarded)
                .unwrap()
                .binding_digest()
        );
    }

    #[test]
    fn mission_binding_tamper_and_unknown_plan_task_fail_closed() {
        let plan = plan("m-binding-tamper", 1, "task-a");
        let graph = Graph::new().with_node(
            Node::new("work", NodeKind::Agent)
                .with_task(TaskId::new("task-a"))
                .with_retry(plan.tasks[0].retry_policy.clone())
                .with_checks(plan.tasks[0].verifier_checks.clone()),
        );
        let authority = GraphExecutionAuthority::from_key([0x62; 32]);
        let mut state = GraphState::for_graph_with_plan_and_authority(
            &graph,
            "run-binding-tamper",
            &plan,
            &authority,
        )
        .unwrap();
        state.reserve(&NodeId::new("work"), &authority).unwrap();
        state.mission_binding.as_mut().unwrap().plan_revision = 9;
        assert!(matches!(
            state.validate_for_graph_with_authority(&graph, &authority),
            Err(GraphStateError::InvalidReservation { .. })
        ));

        let wrong_graph = Graph::new().with_node(
            Node::new("work", NodeKind::Agent)
                .with_task(TaskId::new("not-in-plan"))
                .with_retry(plan.tasks[0].retry_policy.clone())
                .with_checks(plan.tasks[0].verifier_checks.clone()),
        );
        assert!(matches!(
            GraphState::for_graph_with_plan(&wrong_graph, "run-wrong-task", &plan),
            Err(GraphStateError::InvalidMissionBinding { .. })
        ));

        let mut standalone = GraphState::for_graph_with_authority(
            &Graph::new().with_node(Node::new("solo", NodeKind::Agent)),
            "run-standalone",
            &authority,
        );
        let reservation = standalone
            .reserve(&NodeId::new("solo"), &authority)
            .unwrap();
        assert!(reservation.mission_binding_digest.is_empty());
    }

    #[test]
    fn plan_binding_requires_exact_task_mapping_and_dependency_reachability() {
        let plan = plan_with_tasks(
            "m-binding-topology",
            1,
            vec![
                task_contract("task-a", crate::routing::RiskLevel::Medium, &[]),
                task_contract("task-b", crate::routing::RiskLevel::Medium, &["task-a"]),
            ],
        );
        let task_a = plan
            .tasks
            .iter()
            .find(|task| task.task_id == TaskId::new("task-a"))
            .unwrap();
        let task_b = plan
            .tasks
            .iter()
            .find(|task| task.task_id == TaskId::new("task-b"))
            .unwrap();

        let omitted = Graph::new().with_node(bound_node("a", NodeKind::Agent, task_a));
        assert!(matches!(
            GraphState::for_graph_with_plan(&omitted, "run-omitted", &plan),
            Err(GraphStateError::InvalidMissionBinding { reason })
                if reason.contains("task-b has no executable graph node")
        ));

        let duplicated = Graph::new()
            .with_node(bound_node("a-one", NodeKind::Agent, task_a))
            .with_node(bound_node("a-two", NodeKind::Verifier, task_a))
            .with_node(bound_node("b", NodeKind::Synthesis, task_b));
        assert!(matches!(
            GraphState::for_graph_with_plan(&duplicated, "run-duplicate", &plan),
            Err(GraphStateError::InvalidMissionBinding { reason })
                if reason.contains("mapped by both graph nodes")
        ));

        let erased = Graph::new()
            .with_node(bound_node("a", NodeKind::Agent, task_a))
            .with_node(bound_node("b", NodeKind::Synthesis, task_b));
        assert!(matches!(
            GraphState::for_graph_with_plan(&erased, "run-erased", &plan),
            Err(GraphStateError::InvalidMissionBinding { reason })
                if reason.contains("task-a -> task-b is not preserved")
        ));

        let back_edge_only = Graph::new()
            .with_node(bound_node("a", NodeKind::Agent, task_a))
            .with_node(bound_node("b", NodeKind::Synthesis, task_b))
            .with_edge("b", "a")
            .with_edge("a", "b")
            .with_loop_bound(LoopBound::new("a", "b", 1));
        assert!(matches!(
            GraphState::for_graph_with_plan(&back_edge_only, "run-back-edge", &plan),
            Err(GraphStateError::InvalidMissionBinding { reason })
                if reason.contains("task-a -> task-b is not preserved")
        ));

        let mediated = Graph::new()
            .with_node(bound_node("a", NodeKind::Agent, task_a))
            .with_node(Node::new("route", NodeKind::Router))
            .with_node(Node::new("reduce", NodeKind::Reduce))
            .with_node(bound_node("b", NodeKind::Synthesis, task_b))
            .with_edge("a", "route")
            .with_edge("route", "reduce")
            .with_edge("reduce", "b");
        assert!(GraphState::for_graph_with_plan(&mediated, "run-mediated", &plan).is_ok());
    }

    #[test]
    fn plan_binding_risk_translation_never_weakens_authoritative_task_risk() {
        let medium = plan_with_tasks(
            "m-risk-medium",
            1,
            vec![task_contract(
                "task",
                crate::routing::RiskLevel::Medium,
                &[],
            )],
        );
        let medium_missing =
            Graph::new().with_node(bound_node("work", NodeKind::Agent, &medium.tasks[0]));
        assert!(
            GraphState::for_graph_with_plan(&medium_missing, "run-medium-missing", &medium).is_ok(),
            "missing graph risk defaults to Elevated and does not weaken Medium"
        );

        let high = plan_with_tasks(
            "m-risk-high",
            1,
            vec![task_contract("task", crate::routing::RiskLevel::High, &[])],
        );
        let high_safe = Graph::new().with_node(crate::graph_risk::with_risk(
            bound_node("work", NodeKind::Agent, &high.tasks[0]),
            crate::graph_risk::RiskLevel::Safe,
        ));
        assert!(matches!(
            GraphState::for_graph_with_plan(&high_safe, "run-high-safe", &high),
            Err(GraphStateError::InvalidMissionBinding { reason })
                if reason.contains("risk safe weakens") && reason.contains("minimum is elevated")
        ));
        let high_missing =
            Graph::new().with_node(bound_node("work", NodeKind::Agent, &high.tasks[0]));
        assert!(
            GraphState::for_graph_with_plan(&high_missing, "run-high-missing", &high).is_ok(),
            "missing graph risk defaults to Elevated and does not weaken High"
        );
        let high_stricter = Graph::new().with_node(crate::graph_risk::with_risk(
            bound_node("work", NodeKind::Agent, &high.tasks[0]),
            crate::graph_risk::RiskLevel::Irreversible,
        ));
        assert!(
            GraphState::for_graph_with_plan(&high_stricter, "run-high-stricter", &high).is_ok(),
            "a stricter graph classification must remain valid"
        );

        let critical = plan_with_tasks(
            "m-risk-critical",
            1,
            vec![task_contract(
                "task",
                crate::routing::RiskLevel::Critical,
                &[],
            )],
        );
        let critical_safe = Graph::new().with_node(crate::graph_risk::with_risk(
            bound_node("work", NodeKind::Agent, &critical.tasks[0]),
            crate::graph_risk::RiskLevel::Safe,
        ));
        assert!(matches!(
            GraphState::for_graph_with_plan(
                &critical_safe,
                "run-critical-safe",
                &critical,
            ),
            Err(GraphStateError::InvalidMissionBinding { reason })
                if reason.contains("risk safe weakens")
                    && reason.contains("minimum is irreversible")
        ));
        let critical_missing =
            Graph::new().with_node(bound_node("work", NodeKind::Agent, &critical.tasks[0]));
        assert!(matches!(
            GraphState::for_graph_with_plan(
                &critical_missing,
                "run-critical-missing",
                &critical,
            ),
            Err(GraphStateError::InvalidMissionBinding { reason })
                if reason.contains("risk elevated weakens")
        ));

        let mut malformed_node = bound_node("work", NodeKind::Agent, &medium.tasks[0]);
        malformed_node.extra.insert(
            crate::graph_risk::RISK_KEY.to_string(),
            Value::from("almost-safe"),
        );
        let malformed = Graph::new().with_node(malformed_node);
        assert!(matches!(
            GraphState::for_graph_with_plan(&malformed, "run-malformed-risk", &medium),
            Err(GraphStateError::InvalidMissionBinding { reason })
                if reason.contains("has invalid risk")
                    && reason.contains("unknown risk level")
        ));
    }

    #[test]
    fn critical_plan_task_requires_irreversible_gate_and_explicit_approval() {
        let plan = plan_with_tasks(
            "m-risk-critical-gate",
            1,
            vec![task_contract(
                "task",
                crate::routing::RiskLevel::Critical,
                &[],
            )],
        );
        let graph = Graph::new().with_node(crate::graph_risk::with_risk(
            bound_node("work", NodeKind::Agent, &plan.tasks[0]),
            crate::graph_risk::RiskLevel::Irreversible,
        ));
        let authority = GraphExecutionAuthority::from_key([0x63; 32]);
        let mut state = GraphState::for_graph_with_plan_and_authority(
            &graph,
            "run-critical-gate",
            &plan,
            &authority,
        )
        .unwrap();
        let node = NodeId::new("work");
        state.reserve(&node, &authority).unwrap();

        for mode in [
            crate::graph_risk::ExecutionMode::Attended,
            crate::graph_risk::ExecutionMode::Unattended,
        ] {
            let held = crate::graph_risk::evaluate_gate(&graph, &state, &node, mode, &authority);
            assert!(matches!(
                held,
                crate::graph_risk::GateDecision::RequireApproval {
                    risk: crate::graph_risk::RiskLevel::Irreversible,
                    ..
                }
            ));
        }

        let recorded_at = chrono::DateTime::<chrono::Utc>::from_timestamp(1_800_000_000, 0)
            .expect("valid instant");
        let escalation = crate::graph_risk::evaluate_gate(
            &graph,
            &state,
            &node,
            crate::graph_risk::ExecutionMode::Unattended,
            &authority,
        )
        .into_escalation(recorded_at)
        .expect("critical task must emit an approval request");
        let approval =
            crate::graph_risk::approve(&graph, &state, escalation, "operator", &authority).unwrap();
        crate::graph_risk::record_resolution(&graph, &mut state, &approval, &authority).unwrap();
        assert_eq!(
            crate::graph_risk::evaluate_gate(
                &graph,
                &state,
                &node,
                crate::graph_risk::ExecutionMode::Unattended,
                &authority,
            ),
            crate::graph_risk::GateDecision::Proceed
        );
    }

    #[test]
    fn plan_bound_effect_nodes_cannot_weaken_checks_or_exist_without_a_task() {
        let plan = plan("m-binding-contract", 1, "task-a");
        let exact = Node::new("work", NodeKind::Agent)
            .with_task(TaskId::new("task-a"))
            .with_retry(plan.tasks[0].retry_policy.clone())
            .with_checks(plan.tasks[0].verifier_checks.clone());
        assert!(GraphState::for_graph_with_plan(
            &Graph::new().with_node(exact.clone()),
            "run-exact",
            &plan,
        )
        .is_ok());

        let weakened_checks = Graph::new().with_node(exact.clone().with_checks(Vec::new()));
        assert!(matches!(
            GraphState::for_graph_with_plan(&weakened_checks, "run-weak-checks", &plan),
            Err(GraphStateError::InvalidMissionBinding { reason })
                if reason.contains("verifier checks differ")
        ));

        let mut retry = plan.tasks[0].retry_policy.clone();
        retry.max_attempts = retry.max_attempts.saturating_add(1);
        let weakened_retry = Graph::new().with_node(exact.clone().with_retry(retry));
        assert!(matches!(
            GraphState::for_graph_with_plan(&weakened_retry, "run-weak-retry", &plan),
            Err(GraphStateError::InvalidMissionBinding { reason })
                if reason.contains("retry policy differs")
        ));

        for kind in [NodeKind::Agent, NodeKind::Verifier, NodeKind::Synthesis] {
            let orphan = Graph::new().with_node(Node::new("orphan", kind));
            assert!(matches!(
                GraphState::for_graph_with_plan(&orphan, "run-orphan", &plan),
                Err(GraphStateError::InvalidMissionBinding { reason })
                    if reason.contains("has no task_id")
            ));
        }

        for kind in [NodeKind::Reduce, NodeKind::Router] {
            let internal = Graph::new()
                .with_node(exact.clone())
                .with_node(Node::new("internal", kind));
            assert!(
                GraphState::for_graph_with_plan(&internal, "run-internal", &plan).is_ok(),
                "{kind:?} may remain a plan-internal pure node"
            );
        }
    }

    #[test]
    fn persisted_graph_state_fails_closed_on_identity_and_reservation_tampering() {
        let graph = valid_graph();
        let state = GraphState::for_graph_with_run_id(&graph, "run-state-validation");
        assert_eq!(state.validate_for_graph(&graph), Ok(()));

        let mut missing = state.clone();
        missing.nodes.remove(&NodeId::new("find_bugs"));
        assert_eq!(
            missing.validate_for_graph(&graph),
            Err(GraphStateError::MissingNode("find_bugs".to_string()))
        );

        let mut unknown = state.clone();
        unknown
            .nodes
            .insert(NodeId::new("ghost"), NodeRunState::default());
        assert_eq!(
            unknown.validate_for_graph(&graph),
            Err(GraphStateError::UnknownNode("ghost".to_string()))
        );

        let mut empty_run = state.clone();
        empty_run.run_id.clear();
        assert_eq!(
            empty_run.validate_for_graph(&graph),
            Err(GraphStateError::EmptyRunId)
        );

        let mut wrong_schema = state.clone();
        wrong_schema.schema_version = GRAPH_SCHEMA_VERSION + 1;
        assert_eq!(
            wrong_schema.validate_for_graph(&graph),
            Err(GraphStateError::UnsupportedSchema {
                version: GRAPH_SCHEMA_VERSION + 1,
            })
        );

        let mut fabricated_acceptance = state.clone();
        fabricated_acceptance
            .nodes
            .get_mut(&NodeId::new("find_bugs"))
            .unwrap()
            .state = TaskAttemptState::Accepted;
        assert!(matches!(
            fabricated_acceptance.validate_for_graph(&graph),
            Err(GraphStateError::InvalidNodeState { node, reason })
                if node == "find_bugs" && reason.contains("no durable acceptance receipt")
        ));

        let mut reserved = state;
        let node = NodeId::new("classify");
        let authority = GraphExecutionAuthority::from_key([0x41; 32]);
        reserved.bind_authority_if_pristine(&authority).unwrap();
        reserved.reserve(&node, &authority).unwrap();
        assert_eq!(
            reserved.validate_for_graph_with_authority(&graph, &authority),
            Ok(())
        );
        reserved
            .nodes
            .get_mut(&node)
            .unwrap()
            .reservation
            .as_mut()
            .unwrap()
            .reservation_id = "fabricated".to_string();
        assert!(matches!(
            reserved.validate_for_graph(&graph),
            Err(GraphStateError::InvalidReservation { node, reason })
                if node == "classify" && reason.contains("reservation_id mismatch")
        ));
    }

    #[test]
    fn structurally_valid_forged_acceptance_fails_authenticated_validation() {
        let graph = Graph::new().with_node(Node::new("work", NodeKind::Agent));
        let authority = GraphExecutionAuthority::from_key([0x41; 32]);
        let node = NodeId::new("work");
        let mut state =
            GraphState::for_graph_with_authority(&graph, "run-forged-acceptance", &authority);

        state.record_attempt(&node);
        state.reserve(&node, &authority).unwrap();
        state
            .transition(&node, TaskAttemptState::CandidateDone)
            .unwrap();
        state
            .transition(&node, TaskAttemptState::Verifying)
            .unwrap();
        state.transition(&node, TaskAttemptState::Accepted).unwrap();
        state
            .record_acceptance(&node, Vec::new(), &authority)
            .unwrap();
        state.clear_reservation(&node);
        assert_eq!(
            state.validate_for_graph_with_authority(&graph, &authority),
            Ok(())
        );

        let acceptance = state
            .nodes
            .get_mut(&node)
            .unwrap()
            .acceptance
            .as_mut()
            .unwrap();
        acceptance.reservation.authority_mac = "fabricated".to_string();
        acceptance.authority_mac = "fabricated".to_string();

        assert_eq!(state.validate_for_graph(&graph), Ok(()));
        assert!(matches!(
            state.validate_for_graph_with_authority(&graph, &authority),
            Err(GraphStateError::InvalidNodeState { node, reason })
                if node == "work" && reason.contains("authority MAC")
        ));
    }
}
