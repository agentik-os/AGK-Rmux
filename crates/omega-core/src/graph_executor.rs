//! Graph executor — the pure decision core that turns a [`Graph`] plus a
//! [`GraphState`] into "what may run now, and what will never run" (R-GRAPH).
//!
//! [`crate::graph`] owns the persisted VOCABULARY and validates that a graph is
//! structurally sound. It deliberately stops there: it cannot say which nodes are
//! runnable right now, it does not count a retry against a ceiling, and it has no
//! opinion on what happens to the dependents of a node that will never finish.
//! This module owns exactly those three answers and nothing else.
//!
//! WHAT THIS IS NOT, and the boundary is the point: no process spawn, no network,
//! no filesystem, no clock. Every function here is a pure computation over its
//! two arguments, so a whole mission can be replayed off a persisted state file
//! and reach the same decisions on any machine. The CLI, the daemon or a test
//! harness drives it: this core says WHAT to run, the caller runs it and hands
//! the results back through [`advance`].
//!
//! The four contracts it implements:
//!
//! 1. FAN-OUT IS A SET, NOT A SEQUENCE. [`ready_nodes`] returns EVERY node whose
//!    incoming edges are satisfied, so the caller dispatches them concurrently.
//!    A node with no incoming edges is ready from the first call. Returning one
//!    node at a time would silently serialize a diamond, which is the exact
//!    wasted wall clock R-GRAPH exists to stop.
//! 2. BOUNDED RETRIES, THEN ESCALATE (R-LOOP). A failed node consults its own
//!    [`crate::mission::RetryPolicy`]. Attempts are counted in [`GraphState`], so
//!    the ceiling survives a restart: a run resumed from disk cannot hand a
//!    thrashing node a fresh budget. When the cap is spent the node is TERMINALLY
//!    failed and the graph either falls back or reports the stranded work.
//! 3. FAILURE PROPAGATES, IT NEVER HANGS. A terminally failed node with no
//!    fallback strands every dependent that can only be reached through it. Those
//!    dependents are reported as unreachable rather than left queued forever,
//!    because a caller that cannot tell "nothing ready right now" from "nothing
//!    will ever be ready" has no choice but to poll a dead graph.
//! 4. IT CONVERGES. Repeated calls strictly progress or terminate. Every retry
//!    spends a per-node budget that is never refunded, and every back edge spends
//!    a per-edge traversal budget that is never refunded, so no sequence of calls
//!    can cycle forever. See [`advance`] for the argument in full.
//!
//! WHERE THE EXECUTOR'S OWN BOOKKEEPING LIVES. Loop traversal counts, failure
//! reasons and the unreachable set are the executor's state, not the graph
//! vocabulary's, so they are written into the forward-compatible `extra` bag of
//! [`GraphState`] under one namespaced key rather than by adding fields to
//! `graph.rs`. Two reasons, and they point the same way: `graph.rs` is owned by
//! the vocabulary and a second writer on it is how two halves of one system
//! drift apart, and an `extra` key round-trips through an older or newer OmegaOS
//! untouched, so a state file written here still loads there.
//!
//! JOIN SEMANTICS ARE AND, ROUTING IS AUTHENTICATED DATA. A node waits on all of
//! its live incoming edges. A router host must return a signed structured output;
//! the executor reads the exact field declared by [`crate::graph::Router::on`],
//! persists the selected route, cancels only branch-exclusive nodes, and treats
//! those cancelled inputs as neutral at a later join. The caller cannot select a
//! branch by merely editing the state document.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

use crate::graph::{
    Graph, GraphError, GraphExecutionAuthority, GraphState, GraphStateError, Node, NodeId,
    NodeReservation,
};
use crate::mission::{
    InvalidTransition, TaskAttemptState, VerifierCheck, VerifierCheckKind, CONTRACT_SCHEMA_VERSION,
};

// ---------------------------------------------------------------------------
// Fallback declaration
// ---------------------------------------------------------------------------

/// Key under which a node declares its fallback, inside [`Node::extra`].
///
/// Declared through the forward-compatible `extra` bag rather than as a new
/// field on [`Node`] for the same reason the bookkeeping below lives in
/// `GraphState::extra`: the fallback is an EXECUTION concern and `graph.rs` is
/// the vocabulary, so this module does not become a second writer there. Because
/// `extra` is `#[serde(flatten)]`, a declared fallback persists in the graph
/// document exactly like a first-class field and survives a round trip through
/// any other OmegaOS version.
pub const FALLBACK_KEY: &str = "fallback";

/// Declare `fallback` as the node taken when `node` exhausts its retries.
///
/// A free function rather than a `Node` method because `Node` belongs to
/// `graph.rs`; adding an inherent method there would put this module's semantics
/// inside the vocabulary file.
pub fn with_fallback(mut node: Node, fallback: impl Into<NodeId>) -> Node {
    let id: NodeId = fallback.into();
    node.extra
        .insert(FALLBACK_KEY.to_string(), Value::String(id.0));
    node
}

/// The fallback declared by `node`, if any. A non-string value is treated as no
/// declaration rather than an error: this reads data that may have been written
/// by another version, and a panic or a hard failure over one malformed optional
/// key would take a whole mission down.
pub fn fallback_of(node: &Node) -> Option<NodeId> {
    node.extra
        .get(FALLBACK_KEY)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(NodeId::new)
}

// ---------------------------------------------------------------------------
// Results the caller hands back
// ---------------------------------------------------------------------------

/// What one dispatched node reported.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "result")]
pub enum NodeResult {
    Succeeded,
    /// Failed with a human-readable reason. The reason is recorded in the state
    /// so a later [`ExecutionOutcome::Failed`] can name the cause instead of
    /// telling the operator only that something, somewhere, went wrong.
    Failed {
        reason: String,
    },
}

/// One node's result, as handed to [`advance`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeReport {
    pub node: NodeId,
    pub result: NodeResult,
    /// Exact dispatch authority this report answers. `None` is deserializable for
    /// source compatibility but is rejected by [`advance`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reservation: Option<NodeReservation>,
    /// Independently observed results for every check declared on the node.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checks: Vec<NodeCheckResult>,
    /// Authenticated machine-readable output. Required on successful router
    /// hosts and on sources of dry-bounded loops; optional elsewhere.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<NodeOutputReceipt>,
}

/// Structured output bound to exactly one dispatch reservation.
///
/// The worker may print JSON, but only the authority-holding driver can turn it
/// into this receipt. Copying a reservation is therefore insufficient to forge
/// a classification or a `changed` signal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeOutputReceipt {
    pub reservation_id: String,
    #[serde(default)]
    pub fields: BTreeMap<String, Value>,
    pub output_digest: String,
    pub receipt_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub authority_mac: String,
}

impl NodeOutputReceipt {
    pub fn new(
        reservation: &NodeReservation,
        fields: BTreeMap<String, Value>,
        authority: &GraphExecutionAuthority,
    ) -> Result<Self, ExecutorError> {
        let output_digest = structured_output_digest(&fields)?;
        let receipt_id = output_receipt_id(&reservation.reservation_id, &output_digest);
        let authority_mac = authority.mac(
            "omega.graph.node-output.v1",
            &[
                receipt_id.as_bytes(),
                reservation.reservation_id.as_bytes(),
                output_digest.as_bytes(),
            ],
        );
        Ok(Self {
            reservation_id: reservation.reservation_id.clone(),
            fields,
            output_digest,
            receipt_id,
            authority_mac,
        })
    }

    pub fn field(&self, name: &str) -> Option<&Value> {
        self.fields.get(name)
    }

    fn authenticate(
        &self,
        reservation: &NodeReservation,
        authority: &GraphExecutionAuthority,
    ) -> Result<bool, ExecutorError> {
        let digest = structured_output_digest(&self.fields)?;
        let receipt_id = output_receipt_id(&reservation.reservation_id, &digest);
        Ok(self.reservation_id == reservation.reservation_id
            && self.output_digest == digest
            && self.receipt_id == receipt_id
            && self.authority_mac
                == authority.mac(
                    "omega.graph.node-output.v1",
                    &[
                        receipt_id.as_bytes(),
                        reservation.reservation_id.as_bytes(),
                        digest.as_bytes(),
                    ],
                ))
    }
}

fn canonicalize_output(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<String, Value> = map
                .into_iter()
                .map(|(key, value)| (key, canonicalize_output(value)))
                .collect();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize_output).collect()),
        other => other,
    }
}

fn structured_output_digest(fields: &BTreeMap<String, Value>) -> Result<String, ExecutorError> {
    let value = canonicalize_output(Value::Object(
        fields
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    ));
    let bytes = serde_json::to_vec(&value).map_err(|error| ExecutorError::OutputRejected {
        node: "<output>".to_string(),
        reason: format!("cannot fingerprint structured output: {error}"),
    })?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn output_receipt_id(reservation_id: &str, output_digest: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    for value in [reservation_id, output_digest] {
        hasher.update(&(value.len() as u64).to_le_bytes());
        hasher.update(value.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeCheckResult {
    pub check_id: String,
    pub passed: bool,
    pub detail: String,
    /// Cryptographic correlation to the exact declared check, dispatch
    /// reservation and concrete observation. Legacy results deserialize but are
    /// rejected at the execution boundary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt: Option<CheckReceipt>,
}

/// What the verifier actually observed. Inputs are repeated deliberately: the
/// executor compares them to the immutable [`VerifierCheck`] instead of trusting
/// a caller-supplied boolean about a different command, URL, path or object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum CheckObservation {
    Command {
        argv: Vec<String>,
        #[serde(default)]
        cwd: Option<String>,
        exit_code: i32,
    },
    Http {
        url: String,
        status: u16,
    },
    FileExists {
        path: String,
        exists: bool,
    },
    GitObject {
        sha: String,
        exists: bool,
    },
}

/// Tamper-evident correlation receipt for one verifier observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckReceipt {
    #[serde(default = "contract_schema_version")]
    pub schema_version: u32,
    pub check_id: String,
    pub check_contract_digest: String,
    pub reservation_id: String,
    pub observation: CheckObservation,
    pub receipt_id: String,
}

fn contract_schema_version() -> u32 {
    CONTRACT_SCHEMA_VERSION
}

impl CheckReceipt {
    pub fn new(
        check: &VerifierCheck,
        reservation: &NodeReservation,
        observation: CheckObservation,
        authority: &GraphExecutionAuthority,
    ) -> Result<Self, ExecutorError> {
        let check_contract_digest = verifier_check_digest(check)?;
        let receipt_id = receipt_digest(
            check.check_id.as_str(),
            &check_contract_digest,
            reservation.reservation_id.as_str(),
            &observation,
            authority,
        )?;
        Ok(Self {
            schema_version: CONTRACT_SCHEMA_VERSION,
            check_id: check.check_id.clone(),
            check_contract_digest,
            reservation_id: reservation.reservation_id.clone(),
            observation,
            receipt_id,
        })
    }
}

impl NodeCheckResult {
    /// Build a result from the concrete observation. `passed` is derived, never
    /// accepted from the worker as an independent assertion.
    pub fn observed(
        check: &VerifierCheck,
        reservation: &NodeReservation,
        observation: CheckObservation,
        detail: impl Into<String>,
        authority: &GraphExecutionAuthority,
    ) -> Result<Self, ExecutorError> {
        let passed = observation_passes(check, &observation);
        let receipt = CheckReceipt::new(check, reservation, observation, authority)?;
        Ok(Self {
            check_id: check.check_id.clone(),
            passed,
            detail: detail.into(),
            receipt: Some(receipt),
        })
    }
}

impl NodeReport {
    #[deprecated(note = "use NodeReport::succeeded_for with the dispatch reservation")]
    pub fn succeeded(node: impl Into<NodeId>) -> Self {
        Self {
            node: node.into(),
            result: NodeResult::Succeeded,
            reservation: None,
            checks: Vec::new(),
            output: None,
        }
    }

    pub fn succeeded_for(reservation: &NodeReservation) -> Self {
        Self {
            node: reservation.node.clone(),
            result: NodeResult::Succeeded,
            reservation: Some(reservation.clone()),
            checks: Vec::new(),
            output: None,
        }
    }

    #[deprecated(note = "use NodeReport::failed_for with the dispatch reservation")]
    pub fn failed(node: impl Into<NodeId>, reason: impl Into<String>) -> Self {
        Self {
            node: node.into(),
            result: NodeResult::Failed {
                reason: reason.into(),
            },
            reservation: None,
            checks: Vec::new(),
            output: None,
        }
    }

    pub fn failed_for(reservation: &NodeReservation, reason: impl Into<String>) -> Self {
        Self {
            node: reservation.node.clone(),
            result: NodeResult::Failed {
                reason: reason.into(),
            },
            reservation: Some(reservation.clone()),
            checks: Vec::new(),
            output: None,
        }
    }

    #[deprecated(note = "use NodeCheckResult::observed and with_check_result")]
    pub fn with_check(
        mut self,
        check_id: impl Into<String>,
        passed: bool,
        detail: impl Into<String>,
    ) -> Self {
        self.checks.push(NodeCheckResult {
            check_id: check_id.into(),
            passed,
            detail: detail.into(),
            receipt: None,
        });
        self
    }

    pub fn with_check_result(mut self, result: NodeCheckResult) -> Self {
        self.checks.push(result);
        self
    }

    pub fn with_output(mut self, output: NodeOutputReceipt) -> Self {
        self.output = Some(output);
        self
    }

    fn is_failure(&self) -> bool {
        matches!(self.result, NodeResult::Failed { .. })
    }
}

// ---------------------------------------------------------------------------
// Outcome
// ---------------------------------------------------------------------------

/// Where the graph stands after a step.
///
/// The variants exist to make ONE distinction impossible to miss: `Progressing`
/// means work is available now, `Blocked` means no work is available and none
/// ever will be for the listed nodes. A caller that cannot tell those apart can
/// only poll forever.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionOutcome {
    /// These nodes may be dispatched now, concurrently.
    Progressing { ready: Vec<NodeId> },
    /// Nothing is runnable and these nodes will never become runnable, because
    /// every path to them runs through a terminally failed node.
    Blocked { unreachable: Vec<NodeId> },
    /// Every node has settled and nothing failed unrecovered.
    Complete,
    /// Everything that could run has run, and this node failed terminally
    /// without stranding anyone. Reported after `Blocked` because `Blocked`
    /// carries strictly more information when both apply; the reason for any
    /// failed node stays retrievable through [`failure_reason`].
    Failed { node: NodeId, reason: String },
}

/// The full result of one [`advance`] call: the outcome plus what the step
/// actually did, so a caller can log a run without diffing two state snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepOutcome {
    pub outcome: ExecutionOutcome,
    /// Reports that were applied to the state this step.
    pub applied: Vec<NodeId>,
    /// Nodes that failed but still had budget, and will be offered again.
    pub retrying: Vec<NodeId>,
    /// Nodes that failed with their retry budget spent. Terminal.
    pub exhausted: Vec<NodeId>,
    /// Fallback nodes unlocked by an exhausted principal this step.
    pub fallbacks: Vec<NodeId>,
    /// Back edges traversed this step, as `(from, to)`.
    pub loops_taken: Vec<(NodeId, NodeId)>,
    /// Authenticated branch decisions committed by router reports this step.
    pub routes_taken: Vec<RouteDecisionReceipt>,
    /// Nodes newly proven unreachable this step.
    pub newly_unreachable: Vec<NodeId>,
    /// Durable authorities corresponding one-for-one with `ready()`.
    pub reservations: Vec<NodeReservation>,
}

impl StepOutcome {
    /// The ready set, or an empty slice when the graph is not progressing.
    pub fn ready(&self) -> &[NodeId] {
        match &self.outcome {
            ExecutionOutcome::Progressing { ready } => ready,
            _ => &[],
        }
    }

    pub fn is_terminal(&self) -> bool {
        !matches!(self.outcome, ExecutionOutcome::Progressing { .. })
    }

    pub fn reservation_for(&self, node: &NodeId) -> Option<&NodeReservation> {
        self.reservations
            .iter()
            .find(|reservation| reservation.node == *node)
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Every way a step can be rejected. A typed enum and never a panic: this core
/// consumes a graph and a state that arrived from disk or from another machine,
/// and a panic there takes down a daemon over one bad file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutorError {
    /// The graph itself is structurally invalid. Re-checked on every step rather
    /// than trusted once, because the caller may have mutated it between calls
    /// and a dangling edge silently strands every node behind it.
    InvalidGraph(GraphError),
    /// The persisted mutable half is inconsistent with the graph or with its
    /// own reservation identities.
    InvalidGraphState(GraphStateError),
    /// A report naming a node this graph does not contain.
    UnknownNode(String),
    /// The same node reported twice in one step. Rejected rather than
    /// last-one-wins: two results for one attempt means the caller lost track of
    /// a dispatch, and silently picking one hides that.
    DuplicateReport(String),
    /// State belongs to another graph document.
    GraphDigestMismatch { expected: String, actual: String },
    /// The node was never dispatched, or its reservation was already consumed.
    ReportNotReserved {
        node: String,
        state: TaskAttemptState,
    },
    /// A node-only report cannot prove which run or generation produced it.
    UnboundReport(String),
    /// A report carries an authority from another run, generation or version.
    StaleReport {
        node: String,
        expected: String,
        supplied: String,
    },
    /// Monotone reservation identity can no longer advance without wrapping.
    ReservationCounterExhausted(String),
    /// The graph declared a verifier contract that cannot be evaluated safely.
    InvalidCheckContract { node: String, reason: String },
    /// A success report did not prove every declared verifier check passed.
    CheckRejected {
        node: String,
        check: String,
        reason: String,
    },
    /// A router or dry-loop report did not carry valid structured evidence.
    OutputRejected { node: String, reason: String },
    /// Persisted executor routing/loop state failed authentication or no longer
    /// matches the immutable graph/run/reservation it was created for.
    InvalidExecutorState { subject: String, reason: String },
    /// A lifecycle move the mission state machine forbids, surfaced verbatim so
    /// the executor and the ledger report the same error for the same mistake.
    IllegalTransition(InvalidTransition),
}

impl fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGraph(err) => write!(f, "invalid graph: {err}"),
            Self::InvalidGraphState(err) => write!(f, "invalid graph state: {err}"),
            Self::UnknownNode(id) => write!(f, "result reported for unknown node {id}"),
            Self::DuplicateReport(id) => write!(f, "node {id} reported twice in one step"),
            Self::GraphDigestMismatch { expected, actual } => write!(
                f,
                "graph state digest mismatch: expected {expected}, actual {actual}"
            ),
            Self::ReportNotReserved { node, state } => {
                write!(
                    f,
                    "node {node} reported from {state:?} without an active reservation"
                )
            }
            Self::UnboundReport(node) => write!(
                f,
                "node {node} report has no run/generation reservation binding"
            ),
            Self::StaleReport {
                node,
                expected,
                supplied,
            } => write!(
                f,
                "node {node} report reservation is stale: expected {expected}, supplied {supplied}"
            ),
            Self::ReservationCounterExhausted(node) => {
                write!(f, "node {node} reservation counter is exhausted")
            }
            Self::InvalidCheckContract { node, reason } => {
                write!(f, "node {node} has an invalid verifier contract: {reason}")
            }
            Self::CheckRejected {
                node,
                check,
                reason,
            } => write!(f, "node {node} verifier {check} rejected success: {reason}"),
            Self::OutputRejected { node, reason } => {
                write!(f, "node {node} structured output rejected: {reason}")
            }
            Self::InvalidExecutorState { subject, reason } => {
                write!(f, "executor state for {subject} is invalid: {reason}")
            }
            Self::IllegalTransition(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for ExecutorError {}

impl From<GraphError> for ExecutorError {
    fn from(err: GraphError) -> Self {
        Self::InvalidGraph(err)
    }
}

impl From<GraphStateError> for ExecutorError {
    fn from(err: GraphStateError) -> Self {
        match err {
            GraphStateError::InvalidGraph(graph) => Self::InvalidGraph(graph),
            GraphStateError::GraphDigestMismatch { expected, actual } => {
                Self::GraphDigestMismatch { expected, actual }
            }
            other => Self::InvalidGraphState(other),
        }
    }
}

impl From<InvalidTransition> for ExecutorError {
    fn from(err: InvalidTransition) -> Self {
        Self::IllegalTransition(err)
    }
}

// ---------------------------------------------------------------------------
// Executor bookkeeping inside GraphState::extra
// ---------------------------------------------------------------------------

/// One namespaced key rather than three top-level ones, so this module cannot
/// collide with another writer's `extra` entries.
const EXEC_KEY: &str = "graph_executor";
const LOOPS_KEY: &str = "loop_traversals";
const LOOP_PROGRESS_KEY: &str = "loop_progress";
const ROUTES_KEY: &str = "route_decisions";
const FAILURES_KEY: &str = "failures";
const UNREACHABLE_KEY: &str = "unreachable";

/// Durable authenticated selection made by one router-host report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteDecisionReceipt {
    pub host: NodeId,
    pub classification_field: String,
    pub classification: String,
    pub target: NodeId,
    #[serde(default)]
    pub skipped: Vec<NodeId>,
    pub run_id: String,
    pub graph_digest: String,
    pub reservation_id: String,
    pub generation: u64,
    pub output_receipt_id: String,
    pub decision_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub authority_mac: String,
}

/// Last authenticated dry-progress observation for one bounded back edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoopProgressReceipt {
    pub from: NodeId,
    pub to: NodeId,
    pub run_id: String,
    pub graph_digest: String,
    pub reservation_id: String,
    pub generation: u64,
    pub output_receipt_id: String,
    pub changed: bool,
    pub dry_streak: u32,
    pub traversals_after: u32,
    pub progress_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub authority_mac: String,
}

fn edge_key(from: &NodeId, to: &NodeId) -> String {
    format!("{from}->{to}")
}

fn bag(state: &GraphState) -> Option<&Map<String, Value>> {
    state.extra.get(EXEC_KEY).and_then(Value::as_object)
}

fn bag_mut(state: &mut GraphState) -> &mut Map<String, Value> {
    state
        .extra
        .entry(EXEC_KEY.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    // A non-object value written by something else is replaced rather than
    // fought over: this is our namespace, and refusing to proceed over a
    // malformed key would strand a whole run.
    let slot = state.extra.get_mut(EXEC_KEY).expect("just inserted");
    if !slot.is_object() {
        *slot = Value::Object(Map::new());
    }
    slot.as_object_mut().expect("object")
}

fn sub_map<'a>(state: &'a GraphState, key: &str) -> Option<&'a Map<String, Value>> {
    bag(state)?.get(key).and_then(Value::as_object)
}

fn sub_map_mut<'a>(state: &'a mut GraphState, key: &str) -> &'a mut Map<String, Value> {
    let bag = bag_mut(state);
    let slot = bag
        .entry(key.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    if !slot.is_object() {
        *slot = Value::Object(Map::new());
    }
    slot.as_object_mut().expect("object")
}

fn route_decision_id(receipt: &RouteDecisionReceipt) -> String {
    let mut hasher = blake3::Hasher::new();
    for value in [
        receipt.host.as_str(),
        receipt.classification_field.as_str(),
        receipt.classification.as_str(),
        receipt.target.as_str(),
        receipt.run_id.as_str(),
        receipt.graph_digest.as_str(),
        receipt.reservation_id.as_str(),
        receipt.output_receipt_id.as_str(),
    ] {
        hasher.update(&(value.len() as u64).to_le_bytes());
        hasher.update(value.as_bytes());
    }
    hasher.update(&receipt.generation.to_le_bytes());
    for id in &receipt.skipped {
        hasher.update(&(id.as_str().len() as u64).to_le_bytes());
        hasher.update(id.as_str().as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn route_decision_mac(
    receipt: &RouteDecisionReceipt,
    authority: &GraphExecutionAuthority,
) -> String {
    authority.mac(
        "omega.graph.route-decision.v1",
        &[
            receipt.decision_id.as_bytes(),
            receipt.reservation_id.as_bytes(),
            receipt.output_receipt_id.as_bytes(),
            receipt.run_id.as_bytes(),
            receipt.graph_digest.as_bytes(),
        ],
    )
}

fn loop_progress_id(receipt: &LoopProgressReceipt) -> String {
    let mut hasher = blake3::Hasher::new();
    for value in [
        receipt.from.as_str(),
        receipt.to.as_str(),
        receipt.run_id.as_str(),
        receipt.graph_digest.as_str(),
        receipt.reservation_id.as_str(),
        receipt.output_receipt_id.as_str(),
    ] {
        hasher.update(&(value.len() as u64).to_le_bytes());
        hasher.update(value.as_bytes());
    }
    hasher.update(&receipt.generation.to_le_bytes());
    hasher.update(&[u8::from(receipt.changed)]);
    hasher.update(&receipt.dry_streak.to_le_bytes());
    hasher.update(&receipt.traversals_after.to_le_bytes());
    hasher.finalize().to_hex().to_string()
}

fn loop_progress_mac(receipt: &LoopProgressReceipt, authority: &GraphExecutionAuthority) -> String {
    authority.mac(
        "omega.graph.loop-progress.v1",
        &[
            receipt.progress_id.as_bytes(),
            receipt.reservation_id.as_bytes(),
            receipt.output_receipt_id.as_bytes(),
            receipt.run_id.as_bytes(),
            receipt.graph_digest.as_bytes(),
        ],
    )
}

fn route_decisions(
    state: &GraphState,
) -> Result<BTreeMap<NodeId, RouteDecisionReceipt>, ExecutorError> {
    let Some(root) = state.extra.get(EXEC_KEY) else {
        return Ok(BTreeMap::new());
    };
    let root = root
        .as_object()
        .ok_or_else(|| ExecutorError::InvalidExecutorState {
            subject: "graph_executor".to_string(),
            reason: "namespace is not an object".to_string(),
        })?;
    let Some(value) = root.get(ROUTES_KEY) else {
        return Ok(BTreeMap::new());
    };
    let values = value
        .as_object()
        .ok_or_else(|| ExecutorError::InvalidExecutorState {
            subject: ROUTES_KEY.to_string(),
            reason: "route decision collection is not an object".to_string(),
        })?;
    values
        .iter()
        .map(|(host, value)| {
            let receipt: RouteDecisionReceipt =
                serde_json::from_value(value.clone()).map_err(|error| {
                    ExecutorError::InvalidExecutorState {
                        subject: host.clone(),
                        reason: format!("route decision cannot be decoded: {error}"),
                    }
                })?;
            if receipt.host.as_str() != host {
                return Err(ExecutorError::InvalidExecutorState {
                    subject: host.clone(),
                    reason: "route decision map key does not match its host".to_string(),
                });
            }
            Ok((receipt.host.clone(), receipt))
        })
        .collect()
}

fn loop_progress_records(
    state: &GraphState,
) -> Result<BTreeMap<String, LoopProgressReceipt>, ExecutorError> {
    let Some(root) = state.extra.get(EXEC_KEY) else {
        return Ok(BTreeMap::new());
    };
    let root = root
        .as_object()
        .ok_or_else(|| ExecutorError::InvalidExecutorState {
            subject: "graph_executor".to_string(),
            reason: "namespace is not an object".to_string(),
        })?;
    let Some(value) = root.get(LOOP_PROGRESS_KEY) else {
        return Ok(BTreeMap::new());
    };
    let values = value
        .as_object()
        .ok_or_else(|| ExecutorError::InvalidExecutorState {
            subject: LOOP_PROGRESS_KEY.to_string(),
            reason: "loop progress collection is not an object".to_string(),
        })?;
    values
        .iter()
        .map(|(edge, value)| {
            let receipt: LoopProgressReceipt =
                serde_json::from_value(value.clone()).map_err(|error| {
                    ExecutorError::InvalidExecutorState {
                        subject: edge.clone(),
                        reason: format!("loop progress cannot be decoded: {error}"),
                    }
                })?;
            if edge_key(&receipt.from, &receipt.to) != *edge {
                return Err(ExecutorError::InvalidExecutorState {
                    subject: edge.clone(),
                    reason: "loop progress map key does not match its edge".to_string(),
                });
            }
            Ok((edge.clone(), receipt))
        })
        .collect()
}

fn record_route_decision(
    state: &mut GraphState,
    receipt: &RouteDecisionReceipt,
) -> Result<(), ExecutorError> {
    let value =
        serde_json::to_value(receipt).map_err(|error| ExecutorError::InvalidExecutorState {
            subject: receipt.host.0.clone(),
            reason: format!("route decision cannot be persisted: {error}"),
        })?;
    sub_map_mut(state, ROUTES_KEY).insert(receipt.host.0.clone(), value);
    state.mark_updated();
    Ok(())
}

fn record_loop_progress(
    state: &mut GraphState,
    receipt: &LoopProgressReceipt,
) -> Result<(), ExecutorError> {
    let value =
        serde_json::to_value(receipt).map_err(|error| ExecutorError::InvalidExecutorState {
            subject: edge_key(&receipt.from, &receipt.to),
            reason: format!("loop progress cannot be persisted: {error}"),
        })?;
    sub_map_mut(state, LOOP_PROGRESS_KEY).insert(edge_key(&receipt.from, &receipt.to), value);
    state.mark_updated();
    Ok(())
}

fn route_skipped_nodes(state: &GraphState) -> Result<BTreeSet<NodeId>, ExecutorError> {
    Ok(route_decisions(state)?
        .into_values()
        .flat_map(|decision| decision.skipped)
        .collect())
}

/// Retire route receipts whose hosts are about to enter a new loop generation.
/// Their former exclusive branches become eligible again, except where another
/// still-current router decision independently keeps the same node skipped.
/// Decisions hosted outside `body` are left byte-for-byte intact.
fn reseed_set_for_loop_iteration(
    state: &mut GraphState,
    body: &BTreeSet<NodeId>,
) -> Result<BTreeSet<NodeId>, ExecutorError> {
    let decisions = route_decisions(state)?;
    let obsolete_hosts: BTreeSet<NodeId> = decisions
        .keys()
        .filter(|host| body.contains(*host))
        .cloned()
        .collect();
    if obsolete_hosts.is_empty() {
        return Ok(body.clone());
    }

    let mut obsolete_skipped = BTreeSet::new();
    let mut retained_skipped = BTreeSet::new();
    for (host, decision) in &decisions {
        if obsolete_hosts.contains(host) {
            obsolete_skipped.extend(decision.skipped.iter().cloned());
        } else {
            retained_skipped.extend(decision.skipped.iter().cloned());
        }
    }

    let routes = sub_map_mut(state, ROUTES_KEY);
    for host in &obsolete_hosts {
        routes.remove(host.as_str());
    }
    state.mark_updated();

    let mut reseed = body.clone();
    reseed.extend(obsolete_skipped);
    reseed.retain(|id| !retained_skipped.contains(id));
    Ok(reseed)
}

/// Traversals already spent on one back edge.
pub fn loop_traversals(state: &GraphState, from: &NodeId, to: &NodeId) -> u32 {
    sub_map(state, LOOPS_KEY)
        .and_then(|map| map.get(&edge_key(from, to)))
        .and_then(Value::as_u64)
        .unwrap_or(0)
        .min(u64::from(u32::MAX)) as u32
}

fn record_traversal(state: &mut GraphState, from: &NodeId, to: &NodeId) -> u32 {
    let next = loop_traversals(state, from, to).saturating_add(1);
    sub_map_mut(state, LOOPS_KEY).insert(edge_key(from, to), Value::from(next));
    state.mark_updated();
    next
}

/// The recorded reason a node failed, if it ever did.
pub fn failure_reason(state: &GraphState, id: &NodeId) -> Option<String> {
    sub_map(state, FAILURES_KEY)?
        .get(id.as_str())
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn record_failure(state: &mut GraphState, id: &NodeId, reason: &str) {
    sub_map_mut(state, FAILURES_KEY).insert(id.0.clone(), Value::from(reason));
    state.mark_updated();
}

/// Nodes already proven unreachable in this run.
pub fn unreachable_nodes(state: &GraphState) -> Vec<NodeId> {
    bag(state)
        .and_then(|bag| bag.get(UNREACHABLE_KEY))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(NodeId::new)
                .collect()
        })
        .unwrap_or_default()
}

fn record_unreachable(state: &mut GraphState, ids: &BTreeSet<NodeId>) {
    if ids.is_empty() {
        return;
    }
    let mut all: BTreeSet<NodeId> = unreachable_nodes(state).into_iter().collect();
    all.extend(ids.iter().cloned());
    let encoded: Vec<Value> = all.iter().map(|id| Value::from(id.0.clone())).collect();
    bag_mut(state).insert(UNREACHABLE_KEY.to_string(), Value::Array(encoded));
    state.mark_updated();
}

// ---------------------------------------------------------------------------
// Lifecycle driving
// ---------------------------------------------------------------------------

/// Walk a node through the mission attempt machine to a target state.
///
/// The machine in [`crate::mission`] has no shortcut from `Queued` to `Accepted`,
/// and reimplementing a looser one here would let a node's recorded lifecycle
/// disagree with the ledger's. So the executor walks the real path and skips any
/// step already taken, which makes it idempotent when a state was restored
/// mid-path from disk.
fn drive(
    state: &mut GraphState,
    id: &NodeId,
    path: &[TaskAttemptState],
) -> Result<(), ExecutorError> {
    for step in path {
        let current = state.state_of(id).unwrap_or(TaskAttemptState::Queued);
        if current == *step {
            continue;
        }
        state.transition(id, *step)?;
    }
    Ok(())
}

const ACCEPT_PATH: [TaskAttemptState; 4] = [
    TaskAttemptState::Running,
    TaskAttemptState::CandidateDone,
    TaskAttemptState::Verifying,
    TaskAttemptState::Accepted,
];

const RETRY_PATH: [TaskAttemptState; 4] = [
    TaskAttemptState::Running,
    TaskAttemptState::CandidateDone,
    TaskAttemptState::Verifying,
    TaskAttemptState::CorrectionRequired,
];

const FAIL_PATH: [TaskAttemptState; 4] = [
    TaskAttemptState::Running,
    TaskAttemptState::CandidateDone,
    TaskAttemptState::Verifying,
    TaskAttemptState::Failed,
];

/// Re-seed a node for a NEW loop iteration.
///
/// Deliberately a re-seed and not a transition: `Accepted` is terminal in the
/// mission machine, so walking backwards from it is illegal and should stay
/// illegal. A second pass through a loop body is a fresh attempt series at those
/// nodes, which the ledger models as a new run of the node, not as an undone
/// acceptance. `attempts` is NOT cleared, so a node that burned failures in
/// round one cannot buy a full new budget by going round again: that is the R-LOOP
/// ceiling holding across iterations rather than resetting with them.
fn reseed_for_iteration(state: &mut GraphState, id: &NodeId) {
    state.reseed(id);
}

// ---------------------------------------------------------------------------
// Structural queries
// ---------------------------------------------------------------------------

fn is_back_edge(graph: &Graph, from: &NodeId, to: &NodeId) -> bool {
    graph
        .loop_bounds
        .iter()
        .any(|bound| bound.from == *from && bound.to == *to)
}

fn state_of(state: &GraphState, id: &NodeId) -> TaskAttemptState {
    state.state_of(id).unwrap_or(TaskAttemptState::Queued)
}

/// The node this one substitutes for, when it was declared as a fallback.
fn principal_of(graph: &Graph, id: &NodeId) -> Option<NodeId> {
    graph
        .nodes
        .iter()
        .find(|node| fallback_of(node).as_ref() == Some(id))
        .map(|node| node.id.clone())
}

/// Has this node failed terminally with nothing left to cover for it?
///
/// A declared fallback that has not yet had its turn counts as cover. Treating a
/// failure as unrecovered the instant it happens would strand the very
/// dependents the fallback exists to serve, one step before the fallback is even
/// dispatched: the failure is only final once the fallback is final too.
fn is_unrecovered_failure(graph: &Graph, state: &GraphState, id: &NodeId) -> bool {
    if state_of(state, id) != TaskAttemptState::Failed {
        return false;
    }
    match graph.node(id).and_then(fallback_of) {
        // A fallback naming a node this graph does not contain covers nothing.
        Some(fallback) if graph.node(&fallback).is_some() => matches!(
            state_of(state, &fallback),
            TaskAttemptState::Failed | TaskAttemptState::Cancelled
        ),
        _ => true,
    }
}

/// Is the dependency carried by `from -> to` satisfied?
///
/// Three cases, and the middle one is the whole fallback mechanism: an edge from
/// a failed principal to ITS fallback fires precisely BECAUSE the principal
/// failed, which is what lets a fallback be wired downstream of the node it
/// replaces without deadlocking on it.
fn edge_satisfied(graph: &Graph, state: &GraphState, from: &NodeId, to: &NodeId) -> bool {
    match state_of(state, from) {
        TaskAttemptState::Accepted => true,
        TaskAttemptState::Failed => match graph.node(from).and_then(fallback_of) {
            Some(fallback) if fallback == *to => true,
            Some(fallback) => state_of(state, &fallback) == TaskAttemptState::Accepted,
            None => false,
        },
        _ => false,
    }
}

fn exclusive_route_nodes(graph: &Graph, host: &NodeId, chosen: &NodeId) -> BTreeSet<NodeId> {
    let Some(router) = graph.routers.get(host) else {
        return BTreeSet::new();
    };
    let chosen_reachable = reachable(graph, chosen, Direction::Forward);
    let mut unchosen_reachable = BTreeSet::new();
    for target in router
        .routes
        .values()
        .chain(router.default.iter())
        .filter(|target| *target != chosen)
    {
        unchosen_reachable.extend(reachable(graph, target, Direction::Forward));
    }
    unchosen_reachable
        .difference(&chosen_reachable)
        .filter(|id| *id != host)
        .cloned()
        .collect()
}

fn route_decision_from_report(
    graph: &Graph,
    state: &GraphState,
    report: &NodeReport,
    authority: &GraphExecutionAuthority,
) -> Result<Option<RouteDecisionReceipt>, ExecutorError> {
    let Some(router) = graph.routers.get(&report.node) else {
        return Ok(None);
    };
    if report.is_failure() {
        return Ok(None);
    }
    let output = report
        .output
        .as_ref()
        .ok_or_else(|| ExecutorError::OutputRejected {
            node: report.node.0.clone(),
            reason: format!("router requires authenticated field {:?}", router.on),
        })?;
    let classification = output
        .field(&router.on)
        .and_then(Value::as_str)
        .ok_or_else(|| ExecutorError::OutputRejected {
            node: report.node.0.clone(),
            reason: format!("router field {:?} is missing or is not a string", router.on),
        })?
        .to_string();
    let target =
        router
            .resolve(&classification)
            .cloned()
            .ok_or_else(|| ExecutorError::OutputRejected {
                node: report.node.0.clone(),
                reason: format!("classification {classification:?} matches no route or default"),
            })?;
    let reservation = report
        .reservation
        .as_ref()
        .ok_or_else(|| ExecutorError::UnboundReport(report.node.0.clone()))?;
    let mut receipt = RouteDecisionReceipt {
        host: report.node.clone(),
        classification_field: router.on.clone(),
        classification,
        target: target.clone(),
        skipped: exclusive_route_nodes(graph, &report.node, &target)
            .into_iter()
            .collect(),
        run_id: state.run_id.clone(),
        graph_digest: state.graph_digest.clone(),
        reservation_id: reservation.reservation_id.clone(),
        generation: reservation.generation,
        output_receipt_id: output.receipt_id.clone(),
        decision_id: String::new(),
        authority_mac: String::new(),
    };
    receipt.decision_id = route_decision_id(&receipt);
    receipt.authority_mac = route_decision_mac(&receipt, authority);
    Ok(Some(receipt))
}

fn apply_route_decisions(
    graph: &Graph,
    state: &mut GraphState,
    reports: &[NodeReport],
    authority: &GraphExecutionAuthority,
) -> Result<Vec<RouteDecisionReceipt>, ExecutorError> {
    let mut applied = Vec::new();
    for report in reports {
        let Some(receipt) = route_decision_from_report(graph, state, report, authority)? else {
            continue;
        };
        for id in &receipt.skipped {
            match state_of(state, id) {
                TaskAttemptState::Queued | TaskAttemptState::CorrectionRequired => {
                    state.transition(id, TaskAttemptState::Cancelled)?;
                }
                TaskAttemptState::Cancelled => {}
                other => {
                    return Err(ExecutorError::InvalidExecutorState {
                        subject: receipt.host.0.clone(),
                        reason: format!(
                            "unchosen branch node {id} was already {other:?} before route selection"
                        ),
                    });
                }
            }
        }
        record_route_decision(state, &receipt)?;
        applied.push(receipt);
    }
    Ok(applied)
}

fn validate_route_decision(
    graph: &Graph,
    state: &GraphState,
    receipt: &RouteDecisionReceipt,
    authority: &GraphExecutionAuthority,
) -> Result<(), ExecutorError> {
    let reject = |reason: String| ExecutorError::InvalidExecutorState {
        subject: receipt.host.0.clone(),
        reason,
    };
    let router = graph
        .routers
        .get(&receipt.host)
        .ok_or_else(|| reject("decision names a node without a router".to_string()))?;
    let target = router
        .resolve(&receipt.classification)
        .ok_or_else(|| reject("persisted classification no longer resolves".to_string()))?;
    let expected_skipped: Vec<NodeId> = exclusive_route_nodes(graph, &receipt.host, target)
        .into_iter()
        .collect();
    let run = state
        .nodes
        .get(&receipt.host)
        .ok_or_else(|| reject("router host is absent from run state".to_string()))?;
    let acceptance = run
        .acceptance
        .as_ref()
        .ok_or_else(|| reject("router decision has no accepted host receipt".to_string()))?;
    if run.state != TaskAttemptState::Accepted
        || receipt.classification_field != router.on
        || receipt.target != *target
        || receipt.skipped != expected_skipped
        || receipt.run_id != state.run_id
        || receipt.graph_digest != state.graph_digest
        || receipt.reservation_id != acceptance.reservation.reservation_id
        || receipt.generation != acceptance.reservation.generation
        || receipt.decision_id != route_decision_id(receipt)
        || receipt.authority_mac != route_decision_mac(receipt, authority)
    {
        return Err(reject(
            "identity, classification, skipped set, reservation, or authority MAC mismatch"
                .to_string(),
        ));
    }
    for skipped in &receipt.skipped {
        if state_of(state, skipped) != TaskAttemptState::Cancelled {
            return Err(reject(format!(
                "unchosen branch node {skipped} is not durably cancelled"
            )));
        }
    }
    Ok(())
}

fn validate_loop_progress(
    graph: &Graph,
    state: &GraphState,
    receipt: &LoopProgressReceipt,
    authority: &GraphExecutionAuthority,
) -> Result<(), ExecutorError> {
    let subject = edge_key(&receipt.from, &receipt.to);
    let reject = |reason: String| ExecutorError::InvalidExecutorState {
        subject: subject.clone(),
        reason,
    };
    let bound = graph
        .loop_bounds
        .iter()
        .find(|bound| bound.from == receipt.from && bound.to == receipt.to)
        .ok_or_else(|| reject("progress names an undeclared loop bound".to_string()))?;
    let stop_after = bound.stop_after_dry_rounds.ok_or_else(|| {
        reject("progress exists for a loop without a dry-round policy".to_string())
    })?;
    let run = state
        .nodes
        .get(&receipt.from)
        .ok_or_else(|| reject("loop source is absent from run state".to_string()))?;
    if receipt.run_id != state.run_id
        || receipt.graph_digest != state.graph_digest
        || receipt.generation == 0
        || receipt.generation > run.generation
        || receipt.dry_streak > stop_after
        || (receipt.changed && receipt.dry_streak != 0)
        || (!receipt.changed && receipt.dry_streak == 0)
        || receipt.traversals_after != loop_traversals(state, &receipt.from, &receipt.to)
        || receipt.progress_id != loop_progress_id(receipt)
        || receipt.authority_mac != loop_progress_mac(receipt, authority)
    {
        return Err(reject(
            "run, generation, dry streak, traversal count, or authority MAC mismatch".to_string(),
        ));
    }
    if run.state == TaskAttemptState::Accepted
        && run
            .acceptance
            .as_ref()
            .map(|acceptance| acceptance.reservation.reservation_id.as_str())
            != Some(receipt.reservation_id.as_str())
    {
        return Err(reject(
            "terminal dry observation does not match the source acceptance receipt".to_string(),
        ));
    }
    Ok(())
}

fn validate_executor_state(
    graph: &Graph,
    state: &GraphState,
    authority: &GraphExecutionAuthority,
) -> Result<(), ExecutorError> {
    if let Some(root) = state.extra.get(EXEC_KEY) {
        let root = root
            .as_object()
            .ok_or_else(|| ExecutorError::InvalidExecutorState {
                subject: EXEC_KEY.to_string(),
                reason: "namespace is not an object".to_string(),
            })?;
        if let Some(raw) = root.get(LOOPS_KEY) {
            let counters = raw
                .as_object()
                .ok_or_else(|| ExecutorError::InvalidExecutorState {
                    subject: LOOPS_KEY.to_string(),
                    reason: "loop traversal collection is not an object".to_string(),
                })?;
            for (edge, value) in counters {
                let Some(bound) = graph
                    .loop_bounds
                    .iter()
                    .find(|bound| edge_key(&bound.from, &bound.to) == *edge)
                else {
                    return Err(ExecutorError::InvalidExecutorState {
                        subject: edge.clone(),
                        reason: "traversal counter names an undeclared loop bound".to_string(),
                    });
                };
                let count = value
                    .as_u64()
                    .ok_or_else(|| ExecutorError::InvalidExecutorState {
                        subject: edge.clone(),
                        reason: "traversal counter is not an unsigned integer".to_string(),
                    })?;
                if count > u64::from(bound.max_iterations) {
                    return Err(ExecutorError::InvalidExecutorState {
                        subject: edge.clone(),
                        reason: "traversal counter exceeds the immutable ceiling".to_string(),
                    });
                }
            }
        }
    }

    let decisions = route_decisions(state)?;
    for receipt in decisions.values() {
        validate_route_decision(graph, state, receipt, authority)?;
    }
    for host in graph.routers.keys() {
        let accepted = state_of(state, host) == TaskAttemptState::Accepted;
        if accepted != decisions.contains_key(host) {
            return Err(ExecutorError::InvalidExecutorState {
                subject: host.0.clone(),
                reason: if accepted {
                    "accepted router host has no authenticated route decision".to_string()
                } else {
                    "route decision exists before its host was accepted".to_string()
                },
            });
        }
    }

    let progress = loop_progress_records(state)?;
    for receipt in progress.values() {
        validate_loop_progress(graph, state, receipt, authority)?;
    }
    for bound in &graph.loop_bounds {
        if bound.stop_after_dry_rounds.is_none() {
            continue;
        }
        let key = edge_key(&bound.from, &bound.to);
        let source_accepted = state_of(state, &bound.from) == TaskAttemptState::Accepted;
        if (source_accepted || loop_traversals(state, &bound.from, &bound.to) > 0)
            && !progress.contains_key(&key)
        {
            return Err(ExecutorError::InvalidExecutorState {
                subject: key,
                reason: "dry-bounded loop has no authenticated progress receipt".to_string(),
            });
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// ready_nodes
// ---------------------------------------------------------------------------

/// Every node that may be dispatched right now: the FAN-OUT set.
///
/// A node is ready when it is waiting to run (`Queued`, or `CorrectionRequired`
/// after a retryable failure) and every one of its incoming edges is satisfied.
/// Two edges are deliberately not counted:
///
/// - A declared back edge (a [`crate::graph::LoopBound`]) is ignored, otherwise
///   nothing inside a legal bounded cycle could ever start: every node in the
///   cycle would be waiting on the next one round.
/// - A fallback node stays gated until its principal has terminally failed, so a
///   standalone fallback with no incoming edges does not fire eagerly beside the
///   node it exists to replace.
///
/// Order follows the graph's own node declaration order, so two runs over the
/// same graph dispatch in the same order and a log diff means something.
/// Returns a plain `Vec` rather than a `Result`: this is a lookup over data that
/// [`Graph::validate`] has already had its chance to reject, and it treats an
/// unknown node as `Queued` rather than failing.
pub fn ready_nodes(
    graph: &Graph,
    state: &GraphState,
    authority: &GraphExecutionAuthority,
) -> Result<Vec<NodeId>, ExecutorError> {
    state.validate_for_graph_with_authority(graph, authority)?;
    validate_executor_state(graph, state, authority)?;
    let skipped = route_skipped_nodes(state)?;
    let mut ready = Vec::new();
    for node in &graph.nodes {
        let id = &node.id;
        if skipped.contains(id) {
            continue;
        }
        if !matches!(
            state_of(state, id),
            TaskAttemptState::Queued | TaskAttemptState::CorrectionRequired
        ) {
            continue;
        }
        if let Some(principal) = principal_of(graph, id) {
            if state_of(state, &principal) != TaskAttemptState::Failed {
                continue;
            }
        }
        let satisfied = graph
            .edges
            .iter()
            .filter(|edge| edge.to == *id)
            .filter(|edge| !is_back_edge(graph, &edge.from, &edge.to))
            .all(|edge| {
                skipped.contains(&edge.from) || edge_satisfied(graph, state, &edge.from, id)
            });
        if satisfied {
            ready.push(id.clone());
        }
    }
    Ok(ready)
}

// ---------------------------------------------------------------------------
// advance
// ---------------------------------------------------------------------------

/// Apply the reported results, then say what the graph can do next.
///
/// WHY IT CONVERGES, which is the contract that matters most here. Three
/// monotone counters bound every possible sequence of calls:
///
/// 1. A node's failures are counted in [`GraphState`] and never refunded, so a
///    node can be offered at most `RetryPolicy::max_attempts` times before it is
///    terminally failed (R-LOOP).
/// 2. A back edge's traversals are counted per edge and never refunded, so once
///    a loop's `max_iterations` is spent the edge is dead and what remains is the
///    acyclic graph [`Graph::validate`] already proved acyclic.
/// 3. Every other node moves only forward through the mission attempt machine
///    towards a terminal state.
///
/// So each call either applies at least one result, spends at least one budget,
/// or returns a terminal outcome. Calling it repeatedly with no results is safe
/// and idempotent: it re-reports the same decision instead of mutating.
///
/// PRECEDENCE of the outcome, and it is deliberate: `Progressing` first (work
/// beats analysis), then `Blocked` (stranded nodes are the most actionable
/// terminal fact and name what died), then `Failed`, then `Complete`.
pub fn advance(
    graph: &Graph,
    state: &mut GraphState,
    results: &[NodeReport],
    authority: &GraphExecutionAuthority,
) -> Result<StepOutcome, ExecutorError> {
    // State files are the durable checkpoint. Apply the whole step to a
    // candidate and publish it only after the final authenticated validation,
    // so every typed error is transactionally mutation-free.
    let mut candidate = state.clone();
    let outcome = advance_in_place(graph, &mut candidate, results, authority)?;
    *state = candidate;
    Ok(outcome)
}

fn advance_in_place(
    graph: &Graph,
    state: &mut GraphState,
    results: &[NodeReport],
    authority: &GraphExecutionAuthority,
) -> Result<StepOutcome, ExecutorError> {
    // Reject malformed graph/state documents before binding the previously
    // pristine state to a key. Failed input validation must be mutation-free.
    state.validate_for_graph(graph)?;
    state.bind_authority_if_pristine(authority)?;
    state.validate_for_graph_with_authority(graph, authority)?;
    validate_executor_state(graph, state, authority)?;

    let known: BTreeSet<&str> = graph.nodes.iter().map(|node| node.id.as_str()).collect();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for report in results {
        if !known.contains(report.node.as_str()) {
            return Err(ExecutorError::UnknownNode(report.node.0.clone()));
        }
        if !seen.insert(report.node.as_str()) {
            return Err(ExecutorError::DuplicateReport(report.node.0.clone()));
        }
    }
    for report in results {
        validate_report(graph, state, report, authority)?;
    }

    let mut applied = Vec::new();
    let mut retrying = Vec::new();
    let mut exhausted = Vec::new();

    for report in results {
        let id = &report.node;
        match &report.result {
            NodeResult::Succeeded => {
                drive(state, id, &ACCEPT_PATH)?;
                let check_receipt_ids = report
                    .checks
                    .iter()
                    .filter_map(|result| {
                        result
                            .receipt
                            .as_ref()
                            .map(|receipt| receipt.receipt_id.clone())
                    })
                    .collect();
                state.record_acceptance(id, check_receipt_ids, authority)?;
            }
            NodeResult::Failed { reason } => {
                // The attempt is counted BEFORE the ceiling is tested, so the
                // budget is spent by the failure that just happened rather than
                // by the one after it. A policy of `max_attempts: 3` therefore
                // runs the node exactly three times, never four.
                let spent = state.record_attempt(id);
                let cap = graph
                    .node(id)
                    .map(|node| node.retry.max_attempts)
                    .unwrap_or_default();
                record_failure(state, id, reason);
                if spent < cap {
                    drive(state, id, &RETRY_PATH)?;
                    retrying.push(id.clone());
                } else {
                    drive(state, id, &FAIL_PATH)?;
                    exhausted.push(id.clone());
                }
            }
        }
        state.clear_reservation(id);
        applied.push(id.clone());
    }

    // Fallbacks unlocked by an exhaustion in THIS step, reported so a caller can
    // log the substitution rather than infer it from a changed ready set.
    let mut fallbacks = Vec::new();
    for id in &exhausted {
        if let Some(fallback) = graph.node(id).and_then(fallback_of) {
            if graph.node(&fallback).is_some() {
                fallbacks.push(fallback);
            }
        }
    }

    let routes_taken = apply_route_decisions(graph, state, results, authority)?;
    let loops_taken = take_back_edges(graph, state, results, authority)?;
    let newly_unreachable = strand_dependents(graph, state);

    let ready = ready_nodes(graph, state, authority)?;
    let reservation_count = u64::try_from(ready.len()).unwrap_or(u64::MAX);
    if state.version.checked_add(reservation_count).is_none() {
        return Err(ExecutorError::ReservationCounterExhausted(
            ready
                .first()
                .map(|id| id.0.clone())
                .unwrap_or_else(|| "<graph>".to_string()),
        ));
    }
    for id in &ready {
        if state
            .nodes
            .get(id)
            .is_some_and(|node| node.generation == u64::MAX)
        {
            return Err(ExecutorError::ReservationCounterExhausted(id.0.clone()));
        }
    }
    for id in ready {
        state.reserve(&id, authority)?;
    }
    let reservations: Vec<NodeReservation> = graph
        .nodes
        .iter()
        .filter_map(|node| state.reservation_of(&node.id).cloned())
        .collect();
    let dispatchable: Vec<NodeId> = reservations
        .iter()
        .map(|reservation| reservation.node.clone())
        .collect();
    let outcome = classify(graph, state, dispatchable);

    // Validate our own output before handing it back as the next durable
    // checkpoint. This catches an internal mutation regression in the same call
    // that produced it, instead of after a restart.
    state.validate_for_graph_with_authority(graph, authority)?;
    validate_executor_state(graph, state, authority)?;

    Ok(StepOutcome {
        outcome,
        applied,
        retrying,
        exhausted,
        fallbacks,
        loops_taken,
        routes_taken,
        newly_unreachable,
        reservations,
    })
}

fn validate_report(
    graph: &Graph,
    state: &GraphState,
    report: &NodeReport,
    authority: &GraphExecutionAuthority,
) -> Result<(), ExecutorError> {
    let current_state = state
        .state_of(&report.node)
        .unwrap_or(TaskAttemptState::Queued);
    let reservation =
        state
            .reservation_of(&report.node)
            .ok_or_else(|| ExecutorError::ReportNotReserved {
                node: report.node.0.clone(),
                state: current_state,
            })?;
    if current_state != TaskAttemptState::Running {
        return Err(ExecutorError::ReportNotReserved {
            node: report.node.0.clone(),
            state: current_state,
        });
    }
    let supplied = report
        .reservation
        .as_ref()
        .ok_or_else(|| ExecutorError::UnboundReport(report.node.0.clone()))?;
    if supplied != reservation {
        return Err(ExecutorError::StaleReport {
            node: report.node.0.clone(),
            expected: reservation.reservation_id.clone(),
            supplied: supplied.reservation_id.clone(),
        });
    }
    if !reservation.authenticate(authority) {
        return Err(ExecutorError::StaleReport {
            node: report.node.0.clone(),
            expected: reservation.reservation_id.clone(),
            supplied: "invalid-authority-mac".to_string(),
        });
    }

    if let Some(output) = &report.output {
        if !output.authenticate(reservation, authority)? {
            return Err(ExecutorError::OutputRejected {
                node: report.node.0.clone(),
                reason: "output receipt does not authenticate for the active reservation"
                    .to_string(),
            });
        }
    }

    let node = graph
        .node(&report.node)
        .ok_or_else(|| ExecutorError::UnknownNode(report.node.0.clone()))?;
    validate_declared_checks(node)?;
    if matches!(report.result, NodeResult::Succeeded) {
        validate_check_results(node, report, authority)?;
        let _ = route_decision_from_report(graph, state, report, authority)?;
        if graph
            .loop_bounds
            .iter()
            .any(|bound| bound.from == report.node && bound.stop_after_dry_rounds.is_some())
        {
            let changed = report
                .output
                .as_ref()
                .and_then(|output| output.field("changed"))
                .and_then(Value::as_bool);
            if changed.is_none() {
                return Err(ExecutorError::OutputRejected {
                    node: report.node.0.clone(),
                    reason:
                        "dry-bounded loop source requires authenticated boolean field \"changed\""
                            .to_string(),
                });
            }
        }
    }
    Ok(())
}

fn validate_declared_checks(node: &Node) -> Result<(), ExecutorError> {
    let mut ids = BTreeSet::new();
    for check in &node.checks {
        let id = check.check_id.trim();
        let invalid = if check.schema_version != CONTRACT_SCHEMA_VERSION {
            Some(format!(
                "check {} uses unsupported schema {}",
                check.check_id, check.schema_version
            ))
        } else if id.is_empty() {
            Some("check id is empty".to_string())
        } else if !ids.insert(id) {
            Some(format!("duplicate check id {id}"))
        } else if check.timeout_secs == 0 {
            Some(format!("check {id} has a zero timeout"))
        } else {
            match &check.kind {
                VerifierCheckKind::Command { argv, .. }
                    if argv
                        .first()
                        .map(|program| program.trim().is_empty())
                        .unwrap_or(true) =>
                {
                    Some(format!("check {id} has no command program"))
                }
                VerifierCheckKind::Http {
                    url,
                    expected_status,
                } if url.trim().is_empty() || !(100..=599).contains(expected_status) => {
                    Some(format!("check {id} has an invalid HTTP expectation"))
                }
                VerifierCheckKind::FileExists { path } if path.trim().is_empty() => {
                    Some(format!("check {id} has an empty path"))
                }
                VerifierCheckKind::GitObject { sha } if sha.trim().is_empty() => {
                    Some(format!("check {id} has an empty object id"))
                }
                _ => None,
            }
        };
        if let Some(reason) = invalid {
            return Err(ExecutorError::InvalidCheckContract {
                node: node.id.0.clone(),
                reason,
            });
        }
    }
    Ok(())
}

fn validate_check_results(
    node: &Node,
    report: &NodeReport,
    authority: &GraphExecutionAuthority,
) -> Result<(), ExecutorError> {
    let expected: BTreeSet<&str> = node
        .checks
        .iter()
        .map(|check| check.check_id.as_str())
        .collect();
    let mut observed = BTreeSet::new();
    for result in &report.checks {
        if !expected.contains(result.check_id.as_str()) {
            return Err(ExecutorError::CheckRejected {
                node: node.id.0.clone(),
                check: result.check_id.clone(),
                reason: "result was not declared by the node".to_string(),
            });
        }
        if !observed.insert(result.check_id.as_str()) {
            return Err(ExecutorError::CheckRejected {
                node: node.id.0.clone(),
                check: result.check_id.clone(),
                reason: "result was reported more than once".to_string(),
            });
        }
        let check = node
            .checks
            .iter()
            .find(|check| check.check_id == result.check_id)
            .expect("expected set was derived from node checks");
        let receipt = result
            .receipt
            .as_ref()
            .ok_or_else(|| ExecutorError::CheckRejected {
                node: node.id.0.clone(),
                check: result.check_id.clone(),
                reason: "result has no contract/reservation receipt".to_string(),
            })?;
        let reservation = report
            .reservation
            .as_ref()
            .expect("validate_report rejected an unbound report");
        validate_check_receipt(node, check, reservation, result, receipt, authority)?;
        if !result.passed {
            return Err(ExecutorError::CheckRejected {
                node: node.id.0.clone(),
                check: result.check_id.clone(),
                reason: if result.detail.trim().is_empty() {
                    "check failed".to_string()
                } else {
                    result.detail.clone()
                },
            });
        }
    }
    if let Some(missing) = expected.difference(&observed).next() {
        return Err(ExecutorError::CheckRejected {
            node: node.id.0.clone(),
            check: (*missing).to_string(),
            reason: "no passing result was supplied".to_string(),
        });
    }
    Ok(())
}

fn validate_check_receipt(
    node: &Node,
    check: &VerifierCheck,
    reservation: &NodeReservation,
    result: &NodeCheckResult,
    receipt: &CheckReceipt,
    authority: &GraphExecutionAuthority,
) -> Result<(), ExecutorError> {
    let reject = |reason: String| ExecutorError::CheckRejected {
        node: node.id.0.clone(),
        check: result.check_id.clone(),
        reason,
    };
    if receipt.schema_version != CONTRACT_SCHEMA_VERSION {
        return Err(reject(format!(
            "receipt uses unsupported schema {}",
            receipt.schema_version
        )));
    }
    if receipt.check_id != result.check_id || receipt.check_id != check.check_id {
        return Err(reject(
            "receipt check identity does not match contract".to_string(),
        ));
    }
    let expected_contract = verifier_check_digest(check)?;
    if receipt.check_contract_digest != expected_contract {
        return Err(reject(
            "receipt was produced for a different verifier contract".to_string(),
        ));
    }
    if receipt.reservation_id != reservation.reservation_id {
        return Err(reject(
            "receipt was produced for a different dispatch reservation".to_string(),
        ));
    }
    let expected_receipt = receipt_digest(
        receipt.check_id.as_str(),
        receipt.check_contract_digest.as_str(),
        receipt.reservation_id.as_str(),
        &receipt.observation,
        authority,
    )?;
    if receipt.receipt_id != expected_receipt {
        return Err(reject(
            "receipt digest does not match its contents".to_string(),
        ));
    }
    let observed_pass = observation_passes(check, &receipt.observation);
    if result.passed != observed_pass {
        return Err(reject(
            "reported verdict does not match the concrete observation".to_string(),
        ));
    }
    if !observed_pass {
        return Err(reject(if result.detail.trim().is_empty() {
            "concrete observation did not satisfy the verifier contract".to_string()
        } else {
            result.detail.clone()
        }));
    }
    Ok(())
}

fn verifier_check_digest(check: &VerifierCheck) -> Result<String, ExecutorError> {
    let bytes = serde_json::to_vec(check).map_err(|error| ExecutorError::InvalidCheckContract {
        node: "<receipt>".to_string(),
        reason: format!("cannot fingerprint verifier contract: {error}"),
    })?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn receipt_digest(
    check_id: &str,
    check_contract_digest: &str,
    reservation_id: &str,
    observation: &CheckObservation,
    authority: &GraphExecutionAuthority,
) -> Result<String, ExecutorError> {
    let observation =
        serde_json::to_vec(observation).map_err(|error| ExecutorError::InvalidCheckContract {
            node: "<receipt>".to_string(),
            reason: format!("cannot fingerprint verifier observation: {error}"),
        })?;
    let fields = [
        check_id.as_bytes(),
        check_contract_digest.as_bytes(),
        reservation_id.as_bytes(),
        observation.as_slice(),
    ];
    Ok(authority.mac("omega.graph.check-receipt.v1", &fields))
}

fn observation_passes(check: &VerifierCheck, observation: &CheckObservation) -> bool {
    match (&check.kind, observation) {
        (
            VerifierCheckKind::Command {
                argv,
                cwd,
                expected_exit_code,
            },
            CheckObservation::Command {
                argv: observed_argv,
                cwd: observed_cwd,
                exit_code,
            },
        ) => argv == observed_argv && cwd == observed_cwd && expected_exit_code == exit_code,
        (
            VerifierCheckKind::Http {
                url,
                expected_status,
            },
            CheckObservation::Http {
                url: observed_url,
                status,
            },
        ) => url == observed_url && expected_status == status,
        (
            VerifierCheckKind::FileExists { path },
            CheckObservation::FileExists {
                path: observed_path,
                exists,
            },
        ) => path == observed_path && *exists,
        (
            VerifierCheckKind::GitObject { sha },
            CheckObservation::GitObject {
                sha: observed_sha,
                exists,
            },
        ) => sha == observed_sha && *exists,
        _ => false,
    }
}

/// Spend a traversal on every back edge whose source just completed, and re-seed
/// the loop body so the next round can run.
///
/// Only edges whose source was reported succeeded IN THIS STEP are considered.
/// Testing "source is Accepted" instead would re-take the same back edge on
/// every later call until the budget drained, which turns an idempotent no-op
/// call into silent budget burn.
fn take_back_edges(
    graph: &Graph,
    state: &mut GraphState,
    results: &[NodeReport],
    authority: &GraphExecutionAuthority,
) -> Result<Vec<(NodeId, NodeId)>, ExecutorError> {
    let completed: BTreeSet<&str> = results
        .iter()
        .filter(|report| !report.is_failure())
        .map(|report| report.node.as_str())
        .collect();

    let mut taken = Vec::new();
    let prior_progress = loop_progress_records(state)?;
    for bound in &graph.loop_bounds {
        if !completed.contains(bound.from.as_str()) {
            continue;
        }
        if state_of(state, &bound.from) != TaskAttemptState::Accepted {
            continue;
        }
        let report = results
            .iter()
            .find(|report| report.node == bound.from && !report.is_failure())
            .expect("completed set was derived from successful reports");
        let traversals_before = loop_traversals(state, &bound.from, &bound.to);
        let mut should_take = traversals_before < bound.max_iterations;
        let mut changed = true;
        let mut dry_streak = 0;
        if let Some(stop_after) = bound.stop_after_dry_rounds {
            let output = report
                .output
                .as_ref()
                .ok_or_else(|| ExecutorError::OutputRejected {
                    node: report.node.0.clone(),
                    reason: "dry-bounded loop report has no authenticated output".to_string(),
                })?;
            changed = output
                .field("changed")
                .and_then(Value::as_bool)
                .ok_or_else(|| ExecutorError::OutputRejected {
                    node: report.node.0.clone(),
                    reason: "dry-bounded loop field \"changed\" is not boolean".to_string(),
                })?;
            let previous = prior_progress
                .get(&edge_key(&bound.from, &bound.to))
                .map(|receipt| receipt.dry_streak)
                .unwrap_or(0);
            dry_streak = if changed {
                0
            } else {
                previous.saturating_add(1)
            };
            should_take &= dry_streak < stop_after;
        }

        if should_take {
            record_traversal(state, &bound.from, &bound.to);
            let body = loop_body(graph, &bound.from, &bound.to);
            for id in reseed_set_for_loop_iteration(state, &body)? {
                reseed_for_iteration(state, &id);
            }
            taken.push((bound.from.clone(), bound.to.clone()));
        }

        if bound.stop_after_dry_rounds.is_some() {
            let reservation = report
                .reservation
                .as_ref()
                .ok_or_else(|| ExecutorError::UnboundReport(report.node.0.clone()))?;
            let output = report.output.as_ref().expect("dry output validated above");
            let mut receipt = LoopProgressReceipt {
                from: bound.from.clone(),
                to: bound.to.clone(),
                run_id: state.run_id.clone(),
                graph_digest: state.graph_digest.clone(),
                reservation_id: reservation.reservation_id.clone(),
                generation: reservation.generation,
                output_receipt_id: output.receipt_id.clone(),
                changed,
                dry_streak,
                traversals_after: loop_traversals(state, &bound.from, &bound.to),
                progress_id: String::new(),
                authority_mac: String::new(),
            };
            receipt.progress_id = loop_progress_id(&receipt);
            receipt.authority_mac = loop_progress_mac(&receipt, authority);
            record_loop_progress(state, &receipt)?;
        }
    }
    Ok(taken)
}

/// The nodes that belong to one loop: everything forward-reachable from `to`
/// that is also backward-reachable from `from`, over ordinary (non back) edges.
///
/// That intersection is exactly the body the back edge sends round again.
/// Re-seeding the whole graph instead would throw away work outside the loop,
/// and re-seeding only `to` would leave the rest of the body stuck `Accepted`
/// and the loop would run once and stop.
fn loop_body(graph: &Graph, from: &NodeId, to: &NodeId) -> BTreeSet<NodeId> {
    let forward = reachable(graph, to, Direction::Forward);
    let backward = reachable(graph, from, Direction::Backward);
    forward.intersection(&backward).cloned().collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Forward,
    Backward,
}

/// Breadth-first reachability over non back edges, including the seed.
///
/// Iterative on purpose: a recursive walk costs stack depth proportional to the
/// graph, and this runs against graphs that arrive from disk.
fn reachable(graph: &Graph, seed: &NodeId, direction: Direction) -> BTreeSet<NodeId> {
    let mut adjacency: BTreeMap<&str, Vec<&NodeId>> = BTreeMap::new();
    for edge in &graph.edges {
        if is_back_edge(graph, &edge.from, &edge.to) {
            continue;
        }
        match direction {
            Direction::Forward => adjacency
                .entry(edge.from.as_str())
                .or_default()
                .push(&edge.to),
            Direction::Backward => adjacency
                .entry(edge.to.as_str())
                .or_default()
                .push(&edge.from),
        }
    }

    let mut seen: BTreeSet<NodeId> = BTreeSet::new();
    seen.insert(seed.clone());
    let mut queue: VecDeque<NodeId> = VecDeque::from([seed.clone()]);
    while let Some(current) = queue.pop_front() {
        for next in adjacency.get(current.as_str()).into_iter().flatten() {
            if seen.insert((*next).clone()) {
                queue.push_back((*next).clone());
            }
        }
    }
    seen
}

/// Cancel and record every node that can now only be reached through a
/// terminally failed node.
///
/// This is contract 3: failure PROPAGATES rather than hanging. Under AND-join a
/// node with even one dead incoming edge can never become ready, so leaving it
/// `Queued` would mean a caller polling a graph that is already over. The walk
/// stops at nodes that already succeeded, because their output exists and
/// nothing behind them is stranded through that path.
fn strand_dependents(graph: &Graph, state: &mut GraphState) -> Vec<NodeId> {
    let mut stranded: BTreeSet<NodeId> = BTreeSet::new();
    let already: BTreeSet<NodeId> = unreachable_nodes(state).into_iter().collect();

    for node in &graph.nodes {
        if !is_unrecovered_failure(graph, state, &node.id) {
            continue;
        }
        let skip = fallback_of(node);
        let mut queue: VecDeque<NodeId> = VecDeque::new();
        for edge in &graph.edges {
            if edge.from != node.id || is_back_edge(graph, &edge.from, &edge.to) {
                continue;
            }
            if skip.as_ref() == Some(&edge.to) {
                continue;
            }
            queue.push_back(edge.to.clone());
        }
        while let Some(current) = queue.pop_front() {
            match state_of(state, &current) {
                // Its output exists, or it is already terminal on its own
                // account: nothing beyond it is stranded through this path.
                TaskAttemptState::Accepted
                | TaskAttemptState::Failed
                | TaskAttemptState::Cancelled => continue,
                _ => {}
            }
            if !stranded.insert(current.clone()) {
                continue;
            }
            for edge in &graph.edges {
                if edge.from == current && !is_back_edge(graph, &edge.from, &edge.to) {
                    queue.push_back(edge.to.clone());
                }
            }
        }
    }

    for id in &stranded {
        // Cancelled is the mission machine's word for "this will not run". A
        // transition that the machine refuses is swallowed rather than raised:
        // the node is unreachable either way, and failing the whole step over
        // the bookkeeping would hide the far more important fact.
        let _ = state.transition(id, TaskAttemptState::Cancelled);
    }
    record_unreachable(state, &stranded);

    stranded
        .into_iter()
        .filter(|id| !already.contains(id))
        .collect()
}

/// Decide the outcome, in the precedence documented on [`advance`].
fn classify(graph: &Graph, state: &GraphState, ready: Vec<NodeId>) -> ExecutionOutcome {
    if !ready.is_empty() {
        return ExecutionOutcome::Progressing { ready };
    }

    // Anything recorded unreachable, plus anything still unsettled with nothing
    // ready. The second half is the safety net: an unsettled node that cannot be
    // dispatched is blocked by definition, and reporting it as `Complete` would
    // be a false green.
    let mut blocked: BTreeSet<NodeId> = unreachable_nodes(state).into_iter().collect();
    for node in &graph.nodes {
        if !state_of(state, &node.id).is_terminal() {
            blocked.insert(node.id.clone());
        }
    }
    blocked.retain(|id| graph.node(id).is_some());
    if !blocked.is_empty() {
        return ExecutionOutcome::Blocked {
            unreachable: blocked.into_iter().collect(),
        };
    }

    if let Some(node) = graph
        .nodes
        .iter()
        .find(|node| is_unrecovered_failure(graph, state, &node.id))
    {
        return ExecutionOutcome::Failed {
            node: node.id.clone(),
            reason: failure_reason(state, &node.id)
                .unwrap_or_else(|| "node failed without a recorded reason".to_string()),
        };
    }

    ExecutionOutcome::Complete
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use crate::graph::{LoopBound, NodeKind};
    use crate::mission::{RetryPolicy, VerifierCheck};

    fn chain() -> Graph {
        Graph::new()
            .with_node(Node::new("a", NodeKind::Agent))
            .with_node(Node::new("b", NodeKind::Agent))
            .with_node(Node::new("c", NodeKind::Synthesis))
            .with_edge("a", "b")
            .with_edge("b", "c")
    }

    /// a fans out to b and c, both feed the reduce d.
    fn diamond() -> Graph {
        Graph::new()
            .with_node(Node::new("a", NodeKind::Agent))
            .with_node(Node::new("b", NodeKind::Agent))
            .with_node(Node::new("c", NodeKind::Agent))
            .with_node(Node::new("d", NodeKind::Reduce))
            .with_edge("a", "b")
            .with_edge("a", "c")
            .with_edge("b", "d")
            .with_edge("c", "d")
    }

    fn ids(values: &[&str]) -> Vec<NodeId> {
        values.iter().map(|value| NodeId::new(*value)).collect()
    }

    fn output_report(
        reservation: &NodeReservation,
        fields: &[(&str, Value)],
        authority: &GraphExecutionAuthority,
    ) -> NodeReport {
        let fields = fields
            .iter()
            .map(|(key, value)| ((*key).to_string(), value.clone()))
            .collect();
        NodeReport::succeeded_for(reservation).with_output(
            NodeOutputReceipt::new(reservation, fields, authority).expect("output signs"),
        )
    }

    fn routed_diamond() -> Graph {
        Graph::new()
            .with_node(Node::new("classify", NodeKind::Router))
            .with_node(Node::new("branch_a", NodeKind::Agent))
            .with_node(Node::new("branch_b", NodeKind::Agent))
            .with_node(Node::new("join", NodeKind::Synthesis))
            .with_edge("classify", "branch_a")
            .with_edge("classify", "branch_b")
            .with_edge("branch_a", "join")
            .with_edge("branch_b", "join")
            .with_router(
                "classify",
                crate::graph::Router::new("kind")
                    .with_route("a", "branch_a")
                    .with_route("b", "branch_b"),
            )
    }

    fn routed_loop_with_outer_router() -> Graph {
        Graph::new()
            .with_node(Node::new("outer", NodeKind::Router))
            .with_node(Node::new("off_path", NodeKind::Agent))
            .with_node(Node::new("classify", NodeKind::Router))
            .with_node(Node::new("branch_a", NodeKind::Agent))
            .with_node(Node::new("branch_b", NodeKind::Agent))
            .with_node(Node::new("join", NodeKind::Synthesis))
            .with_edge("outer", "off_path")
            .with_edge("outer", "classify")
            .with_edge("classify", "branch_a")
            .with_edge("classify", "branch_b")
            .with_edge("branch_a", "join")
            .with_edge("branch_b", "join")
            .with_edge("join", "classify")
            .with_router(
                "outer",
                crate::graph::Router::new("path")
                    .with_route("off", "off_path")
                    .with_route("loop", "classify"),
            )
            .with_router(
                "classify",
                crate::graph::Router::new("kind")
                    .with_route("a", "branch_a")
                    .with_route("b", "branch_b"),
            )
            .with_loop_bound(LoopBound::new("join", "classify", 1))
    }

    fn authority() -> GraphExecutionAuthority {
        GraphExecutionAuthority::from_key([0x47; 32])
    }

    fn step(graph: &Graph, state: &mut GraphState, reports: &[NodeReport]) -> StepOutcome {
        let reports: Vec<NodeReport> = reports
            .iter()
            .cloned()
            .map(|mut report| {
                if report.reservation.is_none() {
                    report.reservation = state.reservation_of(&report.node).cloned();
                }
                report
            })
            .collect();
        advance(graph, state, &reports, &authority()).expect("step is legal")
    }

    #[test]
    fn linear_chain_advances_one_node_at_a_time() {
        let graph = chain();
        let mut state = GraphState::for_graph(&graph);

        let first = step(&graph, &mut state, &[]);
        assert_eq!(first.ready(), ids(&["a"]).as_slice());

        let second = step(&graph, &mut state, &[NodeReport::succeeded("a")]);
        assert_eq!(second.ready(), ids(&["b"]).as_slice());
        assert_eq!(second.applied, ids(&["a"]));

        let third = step(&graph, &mut state, &[NodeReport::succeeded("b")]);
        assert_eq!(third.ready(), ids(&["c"]).as_slice());

        let last = step(&graph, &mut state, &[NodeReport::succeeded("c")]);
        assert_eq!(last.outcome, ExecutionOutcome::Complete);
    }

    #[test]
    fn diamond_fans_out_to_two_ready_nodes_then_converges() {
        let graph = diamond();
        let mut state = GraphState::for_graph(&graph);

        assert_eq!(
            step(&graph, &mut state, &[]).ready(),
            ids(&["a"]).as_slice()
        );

        // THE fan-out: both branches are offered in one step, not one per call.
        let fanned = step(&graph, &mut state, &[NodeReport::succeeded("a")]);
        assert_eq!(fanned.ready(), ids(&["b", "c"]).as_slice());

        // One branch back: the join still waits on the other, so d is NOT
        // offered. The unreported branch stays ready, because the ready set is
        // idempotent by design: this core says what MAY run, and tracking what
        // is already in flight belongs to the caller that dispatched it.
        let half = step(&graph, &mut state, &[NodeReport::succeeded("b")]);
        assert_eq!(half.ready(), ids(&["c"]).as_slice());
        assert!(
            !half.ready().contains(&NodeId::new("d")),
            "join must wait on every input"
        );

        let joined = step(&graph, &mut state, &[NodeReport::succeeded("c")]);
        assert_eq!(joined.ready(), ids(&["d"]).as_slice());

        let done = step(&graph, &mut state, &[NodeReport::succeeded("d")]);
        assert_eq!(done.outcome, ExecutionOutcome::Complete);
    }

    #[test]
    fn authenticated_router_runs_only_selected_branch_and_preserves_join() {
        let graph = routed_diamond();
        let authority = authority();
        let mut state = GraphState::for_graph_with_authority(&graph, "run-route-a", &authority);
        let first = advance(&graph, &mut state, &[], &authority).unwrap();
        let classify = first
            .reservation_for(&NodeId::new("classify"))
            .unwrap()
            .clone();

        let routed = advance(
            &graph,
            &mut state,
            &[output_report(
                &classify,
                &[("kind", Value::from("a"))],
                &authority,
            )],
            &authority,
        )
        .unwrap();
        assert_eq!(routed.ready(), ids(&["branch_a"]));
        assert_eq!(routed.routes_taken.len(), 1);
        assert_eq!(routed.routes_taken[0].target, NodeId::new("branch_a"));
        assert_eq!(routed.routes_taken[0].skipped, ids(&["branch_b"]));
        assert_eq!(
            state.state_of(&NodeId::new("branch_b")),
            Some(TaskAttemptState::Cancelled)
        );
        assert!(state.reservation_of(&NodeId::new("branch_b")).is_none());

        let branch_a = routed
            .reservation_for(&NodeId::new("branch_a"))
            .unwrap()
            .clone();
        let joined = advance(
            &graph,
            &mut state,
            &[NodeReport::succeeded_for(&branch_a)],
            &authority,
        )
        .unwrap();
        assert_eq!(joined.ready(), ids(&["join"]));
        let join = joined
            .reservation_for(&NodeId::new("join"))
            .unwrap()
            .clone();
        assert_eq!(
            advance(
                &graph,
                &mut state,
                &[NodeReport::succeeded_for(&join)],
                &authority,
            )
            .unwrap()
            .outcome,
            ExecutionOutcome::Complete
        );
    }

    #[test]
    fn router_reentered_by_loop_can_switch_branch_without_losing_unrelated_route() {
        let graph = routed_loop_with_outer_router();
        let authority = authority();
        let mut state = GraphState::for_graph_with_authority(&graph, "run-route-loop", &authority);

        let initial = advance(&graph, &mut state, &[], &authority).unwrap();
        let outer = initial
            .reservation_for(&NodeId::new("outer"))
            .unwrap()
            .clone();
        let entered = advance(
            &graph,
            &mut state,
            &[output_report(
                &outer,
                &[("path", Value::from("loop"))],
                &authority,
            )],
            &authority,
        )
        .unwrap();
        let classify_first = entered
            .reservation_for(&NodeId::new("classify"))
            .unwrap()
            .clone();
        let first_branch = advance(
            &graph,
            &mut state,
            &[output_report(
                &classify_first,
                &[("kind", Value::from("a"))],
                &authority,
            )],
            &authority,
        )
        .unwrap();
        let branch_a = first_branch
            .reservation_for(&NodeId::new("branch_a"))
            .unwrap()
            .clone();
        let at_join = advance(
            &graph,
            &mut state,
            &[NodeReport::succeeded_for(&branch_a)],
            &authority,
        )
        .unwrap();
        let join = at_join
            .reservation_for(&NodeId::new("join"))
            .unwrap()
            .clone();

        let second_round = advance(
            &graph,
            &mut state,
            &[NodeReport::succeeded_for(&join)],
            &authority,
        )
        .unwrap();
        assert_eq!(second_round.ready(), ids(&["classify"]));
        let persisted = route_decisions(&state).unwrap();
        assert!(persisted.contains_key(&NodeId::new("outer")));
        assert!(!persisted.contains_key(&NodeId::new("classify")));
        assert_eq!(
            state.state_of(&NodeId::new("off_path")),
            Some(TaskAttemptState::Cancelled),
            "the unrelated outer router decision must remain authoritative"
        );
        assert_eq!(
            state.state_of(&NodeId::new("branch_b")),
            Some(TaskAttemptState::Queued),
            "the former skipped branch must be eligible in the new generation"
        );

        let classify_second = second_round
            .reservation_for(&NodeId::new("classify"))
            .unwrap()
            .clone();
        let switched = advance(
            &graph,
            &mut state,
            &[output_report(
                &classify_second,
                &[("kind", Value::from("b"))],
                &authority,
            )],
            &authority,
        )
        .unwrap();
        assert_eq!(switched.ready(), ids(&["branch_b"]));
        assert_eq!(
            state.state_of(&NodeId::new("branch_a")),
            Some(TaskAttemptState::Cancelled)
        );
        assert_eq!(switched.routes_taken[0].target, NodeId::new("branch_b"));
    }

    #[test]
    fn route_decision_survives_resume_and_tampering_fails_closed() {
        let graph = routed_diamond();
        let authority = authority();
        let mut state =
            GraphState::for_graph_with_authority(&graph, "run-route-resume", &authority);
        let first = advance(&graph, &mut state, &[], &authority).unwrap();
        let reservation = first.reservation_for(&NodeId::new("classify")).unwrap();
        advance(
            &graph,
            &mut state,
            &[output_report(
                reservation,
                &[("kind", Value::from("b"))],
                &authority,
            )],
            &authority,
        )
        .unwrap();

        let encoded = serde_json::to_value(&state).unwrap();
        let mut restored: GraphState = serde_json::from_value(encoded).unwrap();
        let resumed = advance(&graph, &mut restored, &[], &authority).unwrap();
        assert_eq!(resumed.ready(), ids(&["branch_b"]));
        assert_eq!(
            restored.state_of(&NodeId::new("branch_a")),
            Some(TaskAttemptState::Cancelled)
        );

        let mut forged = restored.clone();
        forged.extra[EXEC_KEY][ROUTES_KEY]["classify"]["target"] = Value::from("branch_a");
        let before = forged.clone();
        assert!(matches!(
            advance(&graph, &mut forged, &[], &authority),
            Err(ExecutorError::InvalidExecutorState { .. })
        ));
        assert_eq!(forged, before, "forged persisted routing must be inert");
    }

    #[test]
    fn forged_or_unmatched_router_output_is_rejected_without_mutation() {
        let graph = routed_diamond();
        let authority = authority();
        let mut state = GraphState::for_graph_with_authority(&graph, "run-route-forge", &authority);
        let first = advance(&graph, &mut state, &[], &authority).unwrap();
        let reservation = first
            .reservation_for(&NodeId::new("classify"))
            .unwrap()
            .clone();

        let mut forged = NodeOutputReceipt::new(
            &reservation,
            BTreeMap::from([("kind".to_string(), Value::from("a"))]),
            &authority,
        )
        .unwrap();
        forged.fields.insert("kind".to_string(), Value::from("b"));
        let before = state.clone();
        assert!(matches!(
            advance(
                &graph,
                &mut state,
                &[NodeReport::succeeded_for(&reservation).with_output(forged)],
                &authority,
            ),
            Err(ExecutorError::OutputRejected { .. })
        ));
        assert_eq!(state, before);

        let unmatched = output_report(
            &reservation,
            &[("kind", Value::from("unknown"))],
            &authority,
        );
        assert!(matches!(
            advance(&graph, &mut state, &[unmatched], &authority),
            Err(ExecutorError::OutputRejected { reason, .. }) if reason.contains("matches no route")
        ));
        assert_eq!(state, before);
    }

    #[test]
    fn failing_node_retries_exactly_the_policy_and_no_more() {
        let policy = RetryPolicy {
            max_attempts: 3,
            backoff_secs: 0,
        };
        let graph = Graph::new()
            .with_node(Node::new("flaky", NodeKind::Agent).with_retry(policy))
            .with_node(Node::new("after", NodeKind::Synthesis))
            .with_edge("flaky", "after");
        let mut state = GraphState::for_graph(&graph);

        let mut dispatches = 0;
        let mut guard = 0;
        loop {
            guard += 1;
            assert!(guard < 20, "advance must converge");
            let outcome = step(&graph, &mut state, &[]);
            match outcome.outcome {
                ExecutionOutcome::Progressing { ref ready } => {
                    assert_eq!(ready.as_slice(), ids(&["flaky"]).as_slice());
                    dispatches += 1;
                    step(&graph, &mut state, &[NodeReport::failed("flaky", "boom")]);
                }
                _ => break,
            }
        }

        assert_eq!(dispatches, 3, "exactly max_attempts runs, never a fourth");
        assert_eq!(state.attempts_of(&NodeId::new("flaky")), 3);
        assert_eq!(
            state.state_of(&NodeId::new("flaky")),
            Some(TaskAttemptState::Failed)
        );
        assert_eq!(
            failure_reason(&state, &NodeId::new("flaky")).as_deref(),
            Some("boom")
        );
    }

    #[test]
    fn exhausted_node_takes_its_fallback() {
        let policy = RetryPolicy {
            max_attempts: 1,
            backoff_secs: 0,
        };
        let graph = Graph::new()
            .with_node(with_fallback(
                Node::new("primary", NodeKind::Agent).with_retry(policy),
                "backup",
            ))
            .with_node(Node::new("backup", NodeKind::Agent))
            .with_node(Node::new("consume", NodeKind::Synthesis))
            .with_edge("primary", "backup")
            .with_edge("primary", "consume");
        let mut state = GraphState::for_graph(&graph);

        // The fallback is GATED: it does not fire beside its principal.
        assert_eq!(
            step(&graph, &mut state, &[]).ready(),
            ids(&["primary"]).as_slice()
        );

        let failed = step(
            &graph,
            &mut state,
            &[NodeReport::failed("primary", "no capacity")],
        );
        assert_eq!(failed.exhausted, ids(&["primary"]));
        assert_eq!(failed.fallbacks, ids(&["backup"]));
        assert_eq!(failed.ready(), ids(&["backup"]).as_slice());

        // The fallback's success satisfies the dependents of the node it replaced.
        let recovered = step(&graph, &mut state, &[NodeReport::succeeded("backup")]);
        assert_eq!(recovered.ready(), ids(&["consume"]).as_slice());

        let done = step(&graph, &mut state, &[NodeReport::succeeded("consume")]);
        assert_eq!(done.outcome, ExecutionOutcome::Complete);
    }

    #[test]
    fn a_fallback_that_also_dies_stops_covering_and_the_failure_escalates() {
        // Cover is not unlimited: once the fallback is terminal too there is
        // nothing left to wait for, so the dependents must be reported rather
        // than left queued behind two dead nodes.
        let policy = RetryPolicy {
            max_attempts: 1,
            backoff_secs: 0,
        };
        let graph = Graph::new()
            .with_node(with_fallback(
                Node::new("primary", NodeKind::Agent).with_retry(policy.clone()),
                "backup",
            ))
            .with_node(Node::new("backup", NodeKind::Agent).with_retry(policy))
            .with_node(Node::new("consume", NodeKind::Synthesis))
            .with_edge("primary", "backup")
            .with_edge("primary", "consume");
        let mut state = GraphState::for_graph(&graph);

        step(&graph, &mut state, &[]);
        let first = step(&graph, &mut state, &[NodeReport::failed("primary", "down")]);
        assert!(
            first.newly_unreachable.is_empty(),
            "nothing is stranded while the fallback still has its turn"
        );

        let second = step(
            &graph,
            &mut state,
            &[NodeReport::failed("backup", "down too")],
        );
        assert_eq!(second.newly_unreachable, ids(&["consume"]));
        assert_eq!(
            second.outcome,
            ExecutionOutcome::Blocked {
                unreachable: ids(&["consume"]),
            }
        );
    }

    #[test]
    fn exhausted_node_without_fallback_strands_its_dependents() {
        let policy = RetryPolicy {
            max_attempts: 1,
            backoff_secs: 0,
        };
        let graph = Graph::new()
            .with_node(Node::new("root", NodeKind::Agent).with_retry(policy))
            .with_node(Node::new("mid", NodeKind::Agent))
            .with_node(Node::new("leaf", NodeKind::Synthesis))
            .with_edge("root", "mid")
            .with_edge("mid", "leaf");
        let mut state = GraphState::for_graph(&graph);

        step(&graph, &mut state, &[]);
        let dead = step(&graph, &mut state, &[NodeReport::failed("root", "crashed")]);

        // It reports the stranded set instead of hanging on a node that will
        // never complete: that is the whole point of the variant.
        assert_eq!(dead.newly_unreachable, ids(&["leaf", "mid"]));
        assert_eq!(
            dead.outcome,
            ExecutionOutcome::Blocked {
                unreachable: ids(&["leaf", "mid"]),
            }
        );
        assert!(dead.ready().is_empty());

        // And it stays terminal: a second call neither hangs nor changes verdict.
        let again = step(&graph, &mut state, &[]);
        assert_eq!(again.outcome, dead.outcome);
        assert!(again.newly_unreachable.is_empty());
    }

    #[test]
    fn terminal_failure_with_no_dependents_reports_failed() {
        let policy = RetryPolicy {
            max_attempts: 1,
            backoff_secs: 0,
        };
        let graph = Graph::new().with_node(Node::new("solo", NodeKind::Agent).with_retry(policy));
        let mut state = GraphState::for_graph(&graph);

        step(&graph, &mut state, &[]);
        let outcome = step(&graph, &mut state, &[NodeReport::failed("solo", "nope")]);

        assert_eq!(
            outcome.outcome,
            ExecutionOutcome::Failed {
                node: NodeId::new("solo"),
                reason: "nope".to_string(),
            }
        );
    }

    #[test]
    fn bounded_cycle_terminates() {
        // Loop-until-dry: find -> verify, verify loops back to find twice.
        let graph = Graph::new()
            .with_node(Node::new("find", NodeKind::Agent))
            .with_node(Node::new("verify", NodeKind::Verifier))
            .with_edge("find", "verify")
            .with_edge("verify", "find")
            .with_loop_bound(LoopBound::new("verify", "find", 2));
        let mut state = GraphState::for_graph(&graph);

        let mut rounds = 0;
        let mut guard = 0;
        let outcome = loop {
            guard += 1;
            assert!(guard < 50, "a bounded cycle must converge");
            let outcome = step(&graph, &mut state, &[]);
            let ready = match outcome.outcome {
                ExecutionOutcome::Progressing { ref ready } => ready.clone(),
                other => break other,
            };
            if ready.contains(&NodeId::new("find")) {
                rounds += 1;
            }
            let reports: Vec<NodeReport> = ready
                .iter()
                .map(|id| NodeReport::succeeded(id.0.as_str()))
                .collect();
            step(&graph, &mut state, &reports);
        };

        assert_eq!(outcome, ExecutionOutcome::Complete);
        // The body ran once, then once per traversal of the bounded back edge.
        assert_eq!(rounds, 3);
        assert_eq!(
            loop_traversals(&state, &NodeId::new("verify"), &NodeId::new("find")),
            2,
            "the traversal budget is spent, never refunded"
        );
    }

    fn dry_loop(stop_after: u32) -> Graph {
        let mut bound = LoopBound::new("verify", "find", 8);
        bound.stop_after_dry_rounds = Some(stop_after);
        Graph::new()
            .with_node(Node::new("find", NodeKind::Agent))
            .with_node(Node::new("verify", NodeKind::Verifier))
            .with_edge("find", "verify")
            .with_edge("verify", "find")
            .with_loop_bound(bound)
    }

    fn complete_dry_round(
        graph: &Graph,
        state: &mut GraphState,
        find: &NodeReservation,
        changed: bool,
        authority: &GraphExecutionAuthority,
    ) -> StepOutcome {
        let after_find =
            advance(graph, state, &[NodeReport::succeeded_for(find)], authority).unwrap();
        let verify = after_find
            .reservation_for(&NodeId::new("verify"))
            .expect("verify becomes ready")
            .clone();
        advance(
            graph,
            state,
            &[output_report(
                &verify,
                &[("changed", Value::from(changed))],
                authority,
            )],
            authority,
        )
        .unwrap()
    }

    #[test]
    fn dry_loop_stops_after_consecutive_dry_rounds_before_hard_ceiling() {
        let graph = dry_loop(2);
        let authority = authority();
        let mut state = GraphState::for_graph_with_authority(&graph, "run-dry-stop", &authority);
        let initial = advance(&graph, &mut state, &[], &authority).unwrap();
        let first_find = initial
            .reservation_for(&NodeId::new("find"))
            .unwrap()
            .clone();
        let first_dry = complete_dry_round(&graph, &mut state, &first_find, false, &authority);
        assert_eq!(first_dry.ready(), ids(&["find"]));
        let second_find = first_dry
            .reservation_for(&NodeId::new("find"))
            .unwrap()
            .clone();
        let stopped = complete_dry_round(&graph, &mut state, &second_find, false, &authority);
        assert_eq!(stopped.outcome, ExecutionOutcome::Complete);
        assert_eq!(
            loop_traversals(&state, &NodeId::new("verify"), &NodeId::new("find")),
            1,
            "the second consecutive dry observation stops before another traversal"
        );
        let progress = loop_progress_records(&state).unwrap();
        assert_eq!(progress["verify->find"].dry_streak, 2);
        assert!(!progress["verify->find"].changed);
    }

    #[test]
    fn changed_signal_resets_persisted_dry_streak() {
        let graph = dry_loop(2);
        let authority = authority();
        let mut state = GraphState::for_graph_with_authority(&graph, "run-dry-reset", &authority);
        let mut outcome = advance(&graph, &mut state, &[], &authority).unwrap();
        for (index, changed) in [false, true, false, false].into_iter().enumerate() {
            let find = outcome
                .reservation_for(&NodeId::new("find"))
                .expect("find is ready until final dry stop")
                .clone();
            outcome = complete_dry_round(&graph, &mut state, &find, changed, &authority);
            let expected_streak = match index {
                0 => 1,
                1 => 0,
                2 => 1,
                _ => 2,
            };
            assert_eq!(
                loop_progress_records(&state).unwrap()["verify->find"].dry_streak,
                expected_streak
            );
        }
        assert_eq!(outcome.outcome, ExecutionOutcome::Complete);
        assert_eq!(
            loop_traversals(&state, &NodeId::new("verify"), &NodeId::new("find")),
            3
        );
    }

    #[test]
    fn dry_loop_missing_or_forged_changed_signal_fails_closed() {
        let graph = dry_loop(2);
        let authority = authority();
        let mut state = GraphState::for_graph_with_authority(&graph, "run-dry-missing", &authority);
        let initial = advance(&graph, &mut state, &[], &authority).unwrap();
        let find = initial
            .reservation_for(&NodeId::new("find"))
            .unwrap()
            .clone();
        let after_find = advance(
            &graph,
            &mut state,
            &[NodeReport::succeeded_for(&find)],
            &authority,
        )
        .unwrap();
        let verify = after_find
            .reservation_for(&NodeId::new("verify"))
            .unwrap()
            .clone();
        let before = state.clone();
        assert!(matches!(
            advance(
                &graph,
                &mut state,
                &[NodeReport::succeeded_for(&verify)],
                &authority,
            ),
            Err(ExecutorError::OutputRejected { reason, .. }) if reason.contains("changed")
        ));
        assert_eq!(state, before);

        let forged = NodeOutputReceipt::new(
            &verify,
            BTreeMap::from([("changed".to_string(), Value::from(false))]),
            &GraphExecutionAuthority::from_key([0x99; 32]),
        )
        .unwrap();
        assert!(matches!(
            advance(
                &graph,
                &mut state,
                &[NodeReport::succeeded_for(&verify).with_output(forged)],
                &authority,
            ),
            Err(ExecutorError::OutputRejected { .. })
        ));
        assert_eq!(state, before);
    }

    #[test]
    fn advance_on_a_complete_graph_reports_complete_and_stays_idempotent() {
        let graph = chain();
        let mut state = GraphState::for_graph(&graph);
        for id in ["a", "b", "c"] {
            step(&graph, &mut state, &[]);
            step(&graph, &mut state, &[NodeReport::succeeded(id)]);
        }

        let before = state.clone();
        let first = step(&graph, &mut state, &[]);
        assert_eq!(first.outcome, ExecutionOutcome::Complete);
        assert!(first.applied.is_empty());

        let second = step(&graph, &mut state, &[]);
        assert_eq!(second.outcome, ExecutionOutcome::Complete);
        assert_eq!(state, before, "a no-op step must not mutate the state");
    }

    #[test]
    fn a_clean_graph_never_yields_the_same_node_twice() {
        let graph = diamond();
        assert_eq!(graph.validate(), Ok(()));
        let mut state = GraphState::for_graph(&graph);

        let mut dispatched: Vec<NodeId> = Vec::new();
        let mut guard = 0;
        loop {
            guard += 1;
            assert!(guard < 50, "advance must converge");
            let outcome = step(&graph, &mut state, &[]);
            let ready = match outcome.outcome {
                ExecutionOutcome::Progressing { ref ready } => ready.clone(),
                _ => break,
            };
            for id in &ready {
                assert!(
                    !dispatched.contains(id),
                    "node {id} was offered twice in an acyclic graph"
                );
            }
            dispatched.extend(ready.iter().cloned());
            let reports: Vec<NodeReport> = ready
                .iter()
                .map(|id| NodeReport::succeeded(id.0.as_str()))
                .collect();
            step(&graph, &mut state, &reports);
        }

        assert_eq!(dispatched, ids(&["a", "b", "c", "d"]));
    }

    #[test]
    fn malformed_input_is_a_typed_error_never_a_panic() {
        let graph = chain();
        let mut state = GraphState::for_graph(&graph);
        let authority = authority();
        let pristine = state.clone();

        assert_eq!(
            advance(
                &graph,
                &mut state,
                &[NodeReport::succeeded("ghost")],
                &authority,
            ),
            Err(ExecutorError::UnknownNode("ghost".to_string()))
        );
        assert_eq!(
            state, pristine,
            "a rejected report must not bind or mutate state"
        );
        assert_eq!(
            advance(
                &graph,
                &mut state,
                &[NodeReport::succeeded("a"), NodeReport::failed("a", "x")],
                &authority,
            ),
            Err(ExecutorError::DuplicateReport("a".to_string()))
        );
        assert_eq!(
            state, pristine,
            "duplicate input must be transactionally inert"
        );

        let broken = Graph::new()
            .with_node(Node::new("a", NodeKind::Agent))
            .with_edge("a", "ghost");
        let mut broken_state = GraphState::for_graph(&broken);
        let before_broken = broken_state.clone();
        assert!(matches!(
            advance(&broken, &mut broken_state, &[], &authority),
            Err(ExecutorError::InvalidGraph(GraphError::DanglingEdge { .. }))
        ));
        assert_eq!(
            broken_state, before_broken,
            "invalid graph input must not bind or mutate state"
        );
    }

    #[test]
    fn executor_bookkeeping_round_trips_through_the_state_document() {
        // The executor's counters live in GraphState::extra, so they must
        // survive a save and a reload like any other persisted field.
        let policy = RetryPolicy {
            max_attempts: 1,
            backoff_secs: 0,
        };
        let graph = Graph::new()
            .with_node(Node::new("root", NodeKind::Agent).with_retry(policy))
            .with_node(Node::new("leaf", NodeKind::Synthesis))
            .with_edge("root", "leaf");
        let mut state = GraphState::for_graph(&graph);
        step(&graph, &mut state, &[]);
        step(&graph, &mut state, &[NodeReport::failed("root", "boom")]);

        let encoded = serde_json::to_value(&state).expect("serializes");
        let restored: GraphState = serde_json::from_value(encoded).expect("parses");
        assert_eq!(restored, state);
        assert_eq!(
            failure_reason(&restored, &NodeId::new("root")).as_deref(),
            Some("boom")
        );
        assert_eq!(unreachable_nodes(&restored), ids(&["leaf"]));
    }

    #[test]
    fn fallback_declaration_survives_a_graph_round_trip() {
        let node = with_fallback(Node::new("primary", NodeKind::Agent), "backup");
        assert_eq!(fallback_of(&node), Some(NodeId::new("backup")));

        let graph = Graph::new()
            .with_node(node)
            .with_node(Node::new("backup", NodeKind::Agent));
        let encoded = serde_json::to_value(&graph).expect("serializes");
        let restored: Graph = serde_json::from_value(encoded).expect("parses");
        assert_eq!(
            fallback_of(&restored.nodes[0]),
            Some(NodeId::new("backup")),
            "a fallback declared in extra must persist like a real field"
        );
        assert_eq!(restored.validate(), Ok(()));
    }

    #[test]
    fn reports_require_a_live_reservation_and_bound_reports_reject_old_generations() {
        let policy = RetryPolicy {
            max_attempts: 2,
            backoff_secs: 0,
        };
        let graph = Graph::new().with_node(Node::new("work", NodeKind::Agent).with_retry(policy));
        let mut state = GraphState::for_graph_with_run_id(&graph, "run-reservations");
        let authority = authority();

        assert!(matches!(
            advance(
                &graph,
                &mut state,
                &[NodeReport::succeeded("work")],
                &authority,
            ),
            Err(ExecutorError::ReportNotReserved { node, .. }) if node == "work"
        ));

        let first = advance(&graph, &mut state, &[], &authority).unwrap();
        let reservation_1 = first.reservation_for(&NodeId::new("work")).unwrap().clone();
        assert_eq!(
            state.state_of(&NodeId::new("work")),
            Some(TaskAttemptState::Running)
        );
        assert_eq!(
            advance(
                &graph,
                &mut state,
                &[NodeReport::succeeded("work")],
                &authority,
            ),
            Err(ExecutorError::UnboundReport("work".to_string()))
        );
        let retry = advance(
            &graph,
            &mut state,
            &[NodeReport::failed_for(&reservation_1, "retry")],
            &authority,
        )
        .unwrap();
        let reservation_2 = retry.reservation_for(&NodeId::new("work")).unwrap().clone();
        assert!(reservation_2.generation > reservation_1.generation);
        assert_ne!(reservation_2.reservation_id, reservation_1.reservation_id);

        assert!(matches!(
            advance(
                &graph,
                &mut state,
                &[NodeReport::succeeded_for(&reservation_1)],
                &authority,
            ),
            Err(ExecutorError::StaleReport { node, .. }) if node == "work"
        ));
        assert_eq!(
            state.reservation_of(&NodeId::new("work")),
            Some(&reservation_2),
            "rejecting an old worker must not consume the current reservation"
        );
    }

    #[test]
    fn declared_checks_must_all_be_uniquely_reported_as_passed_before_acceptance() {
        let check = VerifierCheck {
            schema_version: CONTRACT_SCHEMA_VERSION,
            check_id: "unit".to_string(),
            kind: VerifierCheckKind::Command {
                argv: vec!["cargo".to_string(), "test".to_string()],
                cwd: None,
                expected_exit_code: 0,
            },
            timeout_secs: 30,
        };
        let graph = Graph::new()
            .with_node(Node::new("checked", NodeKind::Agent).with_checks(vec![check.clone()]));
        let mut state = GraphState::for_graph_with_run_id(&graph, "run-checks");
        let authority = authority();
        let first = advance(&graph, &mut state, &[], &authority).unwrap();
        let reservation = first
            .reservation_for(&NodeId::new("checked"))
            .unwrap()
            .clone();

        assert!(matches!(
            advance(
                &graph,
                &mut state,
                &[NodeReport::succeeded_for(&reservation)],
                &authority,
            ),
            Err(ExecutorError::CheckRejected { check, .. }) if check == "unit"
        ));
        let failed_check = NodeCheckResult::observed(
            &check,
            &reservation,
            CheckObservation::Command {
                argv: vec!["cargo".to_string(), "test".to_string()],
                cwd: None,
                exit_code: 1,
            },
            "test failed",
            &authority,
        )
        .unwrap();
        assert!(matches!(
            advance(
                &graph,
                &mut state,
                &[NodeReport::succeeded_for(&reservation).with_check_result(failed_check)],
                &authority,
            ),
            Err(ExecutorError::CheckRejected { reason, .. }) if reason == "test failed"
        ));
        assert_eq!(
            state.state_of(&NodeId::new("checked")),
            Some(TaskAttemptState::Running),
            "rejected evidence must not advance lifecycle state"
        );

        let passing_check = NodeCheckResult::observed(
            &check,
            &reservation,
            CheckObservation::Command {
                argv: vec!["cargo".to_string(), "test".to_string()],
                cwd: None,
                exit_code: 0,
            },
            "ok",
            &authority,
        )
        .unwrap();
        let accepted = advance(
            &graph,
            &mut state,
            &[NodeReport::succeeded_for(&reservation).with_check_result(passing_check)],
            &authority,
        )
        .unwrap();
        assert_eq!(accepted.outcome, ExecutionOutcome::Complete);
        assert_eq!(
            state.state_of(&NodeId::new("checked")),
            Some(TaskAttemptState::Accepted)
        );
    }

    #[test]
    fn check_receipts_reject_contract_reservation_observation_and_digest_substitution() {
        let check = VerifierCheck {
            schema_version: CONTRACT_SCHEMA_VERSION,
            check_id: "unit".to_string(),
            kind: VerifierCheckKind::Command {
                argv: vec!["cargo".to_string(), "test".to_string()],
                cwd: Some("crates/omega-core".to_string()),
                expected_exit_code: 0,
            },
            timeout_secs: 30,
        };
        let policy = RetryPolicy {
            max_attempts: 2,
            backoff_secs: 0,
        };
        let graph = Graph::new().with_node(
            Node::new("checked", NodeKind::Agent)
                .with_retry(policy)
                .with_checks(vec![check.clone()]),
        );
        let mut state = GraphState::for_graph_with_run_id(&graph, "run-receipt-binding");
        let authority = authority();
        let first = advance(&graph, &mut state, &[], &authority).unwrap();
        let reservation_1 = first
            .reservation_for(&NodeId::new("checked"))
            .unwrap()
            .clone();
        let retry = advance(
            &graph,
            &mut state,
            &[NodeReport::failed_for(&reservation_1, "retry")],
            &authority,
        )
        .unwrap();
        let reservation_2 = retry
            .reservation_for(&NodeId::new("checked"))
            .unwrap()
            .clone();

        let observation = CheckObservation::Command {
            argv: vec!["cargo".to_string(), "test".to_string()],
            cwd: Some("crates/omega-core".to_string()),
            exit_code: 0,
        };
        let stale = NodeCheckResult::observed(
            &check,
            &reservation_1,
            observation.clone(),
            "old execution",
            &authority,
        )
        .unwrap();
        assert!(matches!(
            advance(
                &graph,
                &mut state,
                &[NodeReport::succeeded_for(&reservation_2).with_check_result(stale)],
                &authority,
            ),
            Err(ExecutorError::CheckRejected { reason, .. })
                if reason.contains("different dispatch reservation")
        ));

        let mut altered_check = check.clone();
        altered_check.kind = VerifierCheckKind::Command {
            argv: vec!["cargo".to_string(), "check".to_string()],
            cwd: Some("crates/omega-core".to_string()),
            expected_exit_code: 0,
        };
        let altered = NodeCheckResult::observed(
            &altered_check,
            &reservation_2,
            CheckObservation::Command {
                argv: vec!["cargo".to_string(), "check".to_string()],
                cwd: Some("crates/omega-core".to_string()),
                exit_code: 0,
            },
            "different command",
            &authority,
        )
        .unwrap();
        assert!(matches!(
            advance(
                &graph,
                &mut state,
                &[NodeReport::succeeded_for(&reservation_2).with_check_result(altered)],
                &authority,
            ),
            Err(ExecutorError::CheckRejected { reason, .. })
                if reason.contains("different verifier contract")
        ));

        let wrong_observation = NodeCheckResult::observed(
            &check,
            &reservation_2,
            CheckObservation::Command {
                argv: vec!["cargo".to_string(), "test".to_string()],
                cwd: Some("another-directory".to_string()),
                exit_code: 0,
            },
            "wrong cwd",
            &authority,
        )
        .unwrap();
        assert!(matches!(
            advance(
                &graph,
                &mut state,
                &[NodeReport::succeeded_for(&reservation_2)
                    .with_check_result(wrong_observation)],
                &authority,
            ),
            Err(ExecutorError::CheckRejected { reason, .. }) if reason == "wrong cwd"
        ));

        let mut tampered = NodeCheckResult::observed(
            &check,
            &reservation_2,
            observation.clone(),
            "ok",
            &authority,
        )
        .unwrap();
        tampered.receipt.as_mut().unwrap().receipt_id = "fabricated".to_string();
        assert!(matches!(
            advance(
                &graph,
                &mut state,
                &[NodeReport::succeeded_for(&reservation_2).with_check_result(tampered)],
                &authority,
            ),
            Err(ExecutorError::CheckRejected { reason, .. })
                if reason.contains("receipt digest")
        ));

        let valid =
            NodeCheckResult::observed(&check, &reservation_2, observation, "ok", &authority)
                .unwrap();
        assert_eq!(
            advance(
                &graph,
                &mut state,
                &[NodeReport::succeeded_for(&reservation_2).with_check_result(valid)],
                &authority,
            )
            .unwrap()
            .outcome,
            ExecutionOutcome::Complete
        );
    }

    #[test]
    fn state_bound_to_another_graph_is_rejected_before_any_report_is_applied() {
        let graph_a = Graph::new().with_node(Node::new("a", NodeKind::Agent));
        let graph_b = Graph::new().with_node(Node::new("b", NodeKind::Agent));
        let mut state = GraphState::for_graph_with_run_id(&graph_a, "run-a");
        let authority = authority();
        assert!(matches!(
            advance(&graph_b, &mut state, &[], &authority),
            Err(ExecutorError::GraphDigestMismatch { .. })
        ));
        assert_eq!(
            state.state_of(&NodeId::new("a")),
            Some(TaskAttemptState::Queued)
        );
    }
}
