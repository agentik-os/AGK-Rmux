//! Graph risk gate: the human-in-the-loop check that sits IN FRONT of execution
//! and decides whether a ready node may run with nobody watching (R-DESTRUCT).
//!
//! [`crate::graph_executor`] answers "what may run now". It answers that
//! structurally, from edges and retry budgets, and it has no opinion on what a
//! node DOES. That is the gap this module fills: a node whose incoming edges are
//! all satisfied is runnable, and a node that drops a production schema is also
//! runnable by exactly the same test. Wiring the executor straight to a
//! dispatcher therefore runs the irreversible step the moment its dependencies
//! land, unattended, with no record that anyone ever chose it.
//!
//! R-DESTRUCT says what to do instead, and it says it in two halves that this
//! file keeps together. An interactive session asks a human and waits. A
//! DISPATCHED session cannot wait, because nobody is watching the prompt: it
//! performs every non-destructive part of the mission, writes a durable record
//! naming the destructive step, what would be lost, and the safer alternative,
//! signals blocked, and escalates through the alert funnel. The ask still
//! happens, asynchronously, through an artifact a human can act on later,
//! instead of against a prompt nobody will answer. [`ExecutionMode`] is that
//! distinction made into a type, and [`EscalationRecord`] is that artifact.
//!
//! WHAT THIS IS NOT, and the boundary is the point: no process spawn, no
//! network, no filesystem write, no CLI. [`evaluate_gate`] is a pure function of
//! its four arguments, so the same graph plus the same state reaches the same
//! decision on every machine and a run is auditable by replay alone. Persisting
//! the record, printing it, sending the alert and asking the operator are the
//! caller's job, and a decision core that did any of them could not be tested
//! without a box to run on.
//!
//! Three invariants shape every type here:
//!
//! 1. NO DESTRUCTIVE MIGRATION, EVER. The classification is written into the
//!    forward-compatible `extra` bag of [`Node`], exactly as
//!    [`crate::graph_executor::FALLBACK_KEY`] writes a fallback, so a `Graph`
//!    document authored before risk existed still deserializes untouched and
//!    round-trips through this version with nothing dropped or renamed. Nothing
//!    in this file adds a field to `graph.rs`, and nothing rewrites a key it
//!    does not own.
//! 2. AN UNCLASSIFIED NODE IS NOT A SAFE NODE. The default is
//!    [`RiskLevel::Elevated`], never `Safe`. The reasoning is in the `Default`
//!    impl, and it is the single most consequential choice in this file.
//! 3. AN APPROVAL REQUEST THAT DOES NOT SAY WHAT IS LOST IS NOT INFORMED
//!    CONSENT. [`GateDecision::RequireApproval`] always carries a non-empty
//!    `what_is_lost`, synthesized honestly when the node declared none, because
//!    a human asked to approve "step 7" with no statement of the damage is being
//!    asked to rubber-stamp, not to decide.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt;

use crate::graph::{Graph, GraphExecutionAuthority, GraphState, Node, NodeId};

// ---------------------------------------------------------------------------
// Where a classification lives on a node
// ---------------------------------------------------------------------------

/// Key under which a node declares its risk level, inside [`Node::extra`].
///
/// Declared through the forward-compatible bag rather than as a new field on
/// [`Node`] for the reason given in invariant 1: `graph.rs` owns the vocabulary,
/// a second writer there is how two halves of one system drift apart, and
/// because `extra` is `#[serde(flatten)]` a declared risk persists in the graph
/// document exactly like a first-class field and survives a round trip through
/// any other OmegaOS version.
pub const RISK_KEY: &str = "risk";

/// Key under which a node declares WHY it carries that risk.
pub const RISK_REASON_KEY: &str = "risk_reason";

/// Key under which a node declares what an operator loses if it runs.
pub const RISK_WHAT_IS_LOST_KEY: &str = "risk_what_is_lost";

const SUBJECT_MARKER_PREFIX: &str = " [approval subject: ";

// ---------------------------------------------------------------------------
// RiskLevel
// ---------------------------------------------------------------------------

/// How much a node costs to be wrong about.
///
/// The serde names are stable lowercase strings, in the idiom of
/// [`crate::oracle_todo::TodoStatus`]: they are written into graph documents on
/// disk and read back by other versions, so renaming one silently reclassifies
/// every node that carried it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// Reversible and cheap to be wrong about. Runs unattended.
    Safe,
    /// Recoverable, but expensive or noisy to undo. Runs with a human watching,
    /// and never with nobody watching.
    Elevated,
    /// Destroys something no later step can restore. Never runs unattended, and
    /// never runs attended without an attributable human decision either.
    Irreversible,
}

impl Default for RiskLevel {
    /// AN UNCLASSIFIED NODE DEFAULTS TO [`RiskLevel::Elevated`], and this is the
    /// load-bearing choice in the file, so here is the whole argument.
    ///
    /// `Safe` was rejected because absence of a classification is not evidence
    /// of safety. Every graph authored before this module existed carries no
    /// classification at all, and some of those graphs certainly contain a
    /// destructive step. Defaulting them to `Safe` would let exactly the node
    /// this gate exists to stop run unattended, and it would fail silently,
    /// which is the one failure mode an approval gate cannot afford.
    ///
    /// `Irreversible` was rejected because it overclaims in two ways that both
    /// cost the operator. It would make every pre-risk graph halt on its first
    /// node in BOTH modes, which is a behavioural break dressed up as a default
    /// and would push people to switch the gate off wholesale. And it would
    /// force the approval request to state a `what_is_lost` that nobody
    /// declared, so the gate would be inventing damage in order to warn about
    /// it, which trains the operator to ignore the warnings.
    ///
    /// `Elevated` is the only level whose meaning is literally "unknown risk".
    /// It preserves the pre-risk behaviour exactly where a human is watching, so
    /// no existing attended workflow changes, and it withholds only the case
    /// that was never safe to begin with: dispatching an unclassified node with
    /// nobody there to notice. That is a strict safety gain with no destructive
    /// migration, which is what invariant 1 asks for.
    fn default() -> Self {
        Self::Elevated
    }
}

impl RiskLevel {
    /// The on-disk string. Kept next to the serde attribute so a rename of one
    /// without the other fails a test rather than a production read.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Elevated => "elevated",
            Self::Irreversible => "irreversible",
        }
    }

    /// Parse an on-disk value. Whitespace is tolerated because a hand-edited
    /// graph document is a real authoring path; case is NOT, because the
    /// lowercase spelling is the contract and quietly accepting `"Irreversible"`
    /// would make two spellings live in the corpus.
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            "safe" => Some(Self::Safe),
            "elevated" => Some(Self::Elevated),
            "irreversible" => Some(Self::Irreversible),
            _ => None,
        }
    }

    /// Whether this level may be dispatched with nobody watching, before any
    /// human decision is taken into account.
    pub fn runs_unattended(self) -> bool {
        matches!(self, Self::Safe)
    }
}

/// Minimum graph gate classification that may implement an authoritative plan
/// task risk. This is the single translation boundary between the four-level
/// Oracle contract and the three-level runtime gate. Callers may classify more
/// strictly, never less strictly.
pub(crate) fn minimum_for_plan_risk(plan_risk: crate::routing::RiskLevel) -> RiskLevel {
    match plan_risk {
        crate::routing::RiskLevel::Low => RiskLevel::Safe,
        crate::routing::RiskLevel::Medium | crate::routing::RiskLevel::High => RiskLevel::Elevated,
        crate::routing::RiskLevel::Critical => RiskLevel::Irreversible,
    }
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// ExecutionMode
// ---------------------------------------------------------------------------

/// Whether a human is present to answer.
///
/// This is the whole point of the module. In [`ExecutionMode::Attended`] an
/// approval can be granted inline, because somebody is looking at the terminal.
/// In [`ExecutionMode::Unattended`] it can NEVER be, because the prompt would be
/// asked of an empty room: the only legal output there is a durable
/// [`EscalationRecord`] a human can act on later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    /// A human is watching and may approve inline.
    Attended,
    /// Dispatched. Nobody is watching, so nobody can answer.
    Unattended,
}

impl ExecutionMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Attended => "attended",
            Self::Unattended => "unattended",
        }
    }

    /// True when a decision reached in this mode must leave an artifact behind
    /// instead of a prompt.
    pub fn requires_durable_record(self) -> bool {
        matches!(self, Self::Unattended)
    }
}

impl fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Every way this module refuses input. A typed enum and never a panic: it reads
/// graph documents that arrived from disk or from another machine, and a panic
/// there takes down a daemon over one bad file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskError {
    /// A `risk` key that is not one of the three known lowercase strings. This
    /// is an ERROR rather than a fall back to the default, because a value
    /// somebody deliberately wrote and got wrong is the case where a silent
    /// default is most dangerous: the author believed the node was classified.
    MalformedRisk { node: String, value: String },
    /// A gate asked about a node the graph does not contain.
    UnknownNode(String),
    /// An approval or denial with no attributable approver. An unattributed
    /// approval is not consent, it is an unsigned note, and it is exactly what a
    /// runaway agent would produce for itself.
    UnattributedApproval { node: String },
    /// A time-bounded decision that would already be expired when created.
    InvalidExpiration { node: String },
    /// Consent without the fingerprint of what was shown cannot authorize work.
    UnboundApprovalSubject { node: String },
    /// The state/reservation/authority no longer matches the approval subject.
    InvalidApprovalAuthority { node: String, reason: String },
}

impl fmt::Display for RiskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedRisk { node, value } => {
                write!(f, "node {node} declares an unknown risk level: {value}")
            }
            Self::UnknownNode(id) => write!(f, "risk gate asked about unknown node {id}"),
            Self::UnattributedApproval { node } => {
                write!(f, "approval for node {node} names no approver")
            }
            Self::InvalidExpiration { node } => {
                write!(f, "approval for node {node} expires before it was recorded")
            }
            Self::UnboundApprovalSubject { node } => {
                write!(f, "approval for node {node} has no valid subject digest")
            }
            Self::InvalidApprovalAuthority { node, reason } => {
                write!(
                    f,
                    "approval for node {node} has invalid authority: {reason}"
                )
            }
        }
    }
}

impl std::error::Error for RiskError {}

// ---------------------------------------------------------------------------
// Reading and writing a classification
// ---------------------------------------------------------------------------

/// Classify `node`. A free function rather than a `Node` method because `Node`
/// belongs to `graph.rs`, and an inherent method there would put this module's
/// semantics inside the vocabulary file.
pub fn with_risk(node: Node, risk: RiskLevel) -> Node {
    with_risk_detail(node, risk, "", "")
}

/// Classify `node` and declare WHY and WHAT IS LOST in the same breath.
///
/// The two prose fields are written only when non-empty, so classifying a node
/// never adds keys carrying nothing: an empty string on disk reads as a declared
/// answer and would defeat the synthesized fallback in [`evaluate_gate`].
pub fn with_risk_detail(
    mut node: Node,
    risk: RiskLevel,
    reason: impl Into<String>,
    what_is_lost: impl Into<String>,
) -> Node {
    node.extra
        .insert(RISK_KEY.to_string(), Value::from(risk.as_str()));
    for (key, value) in [
        (RISK_REASON_KEY, reason.into()),
        (RISK_WHAT_IS_LOST_KEY, what_is_lost.into()),
    ] {
        if !value.trim().is_empty() {
            node.extra.insert(key.to_string(), Value::from(value));
        }
    }
    node
}

/// The level `node` declares, or [`RiskLevel::default`] when it declares none.
///
/// An ABSENT key is the old-document case and defaults. A PRESENT but
/// unparseable key is an error, per [`RiskError::MalformedRisk`]. The asymmetry
/// is deliberate: silence is a document that predates the field, a wrong value
/// is a human who tried to classify and failed, and only the second one is
/// evidence that somebody's intent is being lost.
pub fn risk_of(node: &Node) -> Result<RiskLevel, RiskError> {
    match node.extra.get(RISK_KEY) {
        None => Ok(RiskLevel::default()),
        Some(Value::String(raw)) => RiskLevel::parse(raw).ok_or_else(|| RiskError::MalformedRisk {
            node: node.id.0.clone(),
            value: raw.clone(),
        }),
        Some(other) => Err(RiskError::MalformedRisk {
            node: node.id.0.clone(),
            value: other.to_string(),
        }),
    }
}

/// A declared prose annotation, if it is a non-empty string.
///
/// A malformed annotation reads as UNDECLARED rather than as an error, unlike
/// the classification itself. The classification decides whether a human is
/// asked at all, so getting it wrong changes the outcome; the prose only
/// decorates a request that is already going to be made, and refusing to gate a
/// node because its explanation was the wrong JSON type would trade a real
/// safety check for a formatting complaint.
fn declared(node: &Node, key: &str) -> Option<String> {
    node.extra
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

/// A shell effect outside an immutable V3 mission plan has no authoritative
/// risk classifier. Its self-declared `risk` remains useful context, but it is
/// never permission to execute: an attributable, reservation-bound human
/// decision is required in both attended and unattended modes.
fn standalone_effect_requires_approval(node: &Node, state: &GraphState) -> bool {
    state.mission_binding.is_none() && node.extra.contains_key("command")
}

fn approval_reason_for(
    node: &Node,
    state: &GraphState,
    risk: RiskLevel,
    mode: ExecutionMode,
) -> String {
    if standalone_effect_requires_approval(node, state) {
        return "standalone shell effect has no immutable V3 mission-plan risk authority; explicit attributed approval is required"
            .to_string();
    }
    approval_reason(node, risk, mode)
}

fn what_is_lost_for(node: &Node, state: &GraphState, risk: RiskLevel) -> String {
    if standalone_effect_requires_approval(node, state) {
        if let Some(declared) = declared(node, RISK_WHAT_IS_LOST_KEY) {
            return declared;
        }
        return format!(
            "not authoritatively classified for standalone node {}: assume the shell command may mutate or destroy external state",
            node.id
        );
    }
    what_is_lost(node, risk)
}

// ---------------------------------------------------------------------------
// GateDecision
// ---------------------------------------------------------------------------

/// What the gate says about one ready node.
///
/// [`GateDecision::Refuse`] is not a softer `RequireApproval`: it means the gate
/// could not establish what it is being asked to approve, so there is nothing a
/// human could meaningfully consent to yet. It fails CLOSED in every case, which
/// is the only safe direction for a check that stands in front of destruction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "decision")]
pub enum GateDecision {
    /// Dispatch it.
    Proceed,
    /// A human must decide first. Carries everything that decision needs, and
    /// `what_is_lost` is never empty (invariant 3).
    RequireApproval {
        node: NodeId,
        risk: RiskLevel,
        reason: String,
        what_is_lost: String,
    },
    /// The gate will not put this node in front of a human as it stands.
    Refuse { node: NodeId, reason: String },
}

impl GateDecision {
    pub fn proceeds(&self) -> bool {
        matches!(self, Self::Proceed)
    }

    pub fn requires_approval(&self) -> bool {
        matches!(self, Self::RequireApproval { .. })
    }

    /// Turn an approval request into the durable artifact an unattended run
    /// leaves behind. `None` for the other two variants: there is nothing for a
    /// human to act on in a proceed, and a refusal names a broken graph rather
    /// than a pending decision.
    ///
    /// The timestamp is a PARAMETER rather than a clock read, so the whole
    /// module stays a pure function of its inputs and a test can assert on an
    /// exact record. The caller that has a clock stamps it.
    pub fn into_escalation(self, recorded_at: DateTime<Utc>) -> Option<EscalationRecord> {
        match self {
            Self::RequireApproval {
                node,
                risk,
                reason,
                what_is_lost,
            } => {
                let (reason, subject_digest) = split_subject_marker(reason);
                Some(EscalationRecord {
                    node,
                    risk,
                    reason,
                    what_is_lost,
                    subject_digest,
                    recorded_at,
                    extra: Map::new(),
                })
            }
            _ => None,
        }
    }
}

fn split_subject_marker(reason: String) -> (String, String) {
    let Some((human_reason, marker)) = reason.rsplit_once(SUBJECT_MARKER_PREFIX) else {
        return (reason, String::new());
    };
    let Some(digest) = marker.strip_suffix(']') else {
        return (reason, String::new());
    };
    if digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        (human_reason.to_string(), digest.to_ascii_lowercase())
    } else {
        (reason, String::new())
    }
}

/// Fingerprint of the exact action and attempt a human is being asked about.
pub fn approval_subject_digest(graph: &Graph, state: &GraphState, node: &NodeId) -> String {
    let graph_digest = graph.content_digest().unwrap_or_default();
    let run = state.nodes.get(node);
    let generation = run.map(|run| run.generation).unwrap_or(0);
    let reservation = run.and_then(|run| run.reservation.as_ref());
    let state_version = reservation
        .map(|reservation| reservation.state_version)
        .unwrap_or(0);
    let reservation_id = reservation
        .map(|reservation| reservation.reservation_id.as_str())
        .unwrap_or("");
    let mut hasher = blake3::Hasher::new();
    for value in [
        graph_digest.as_str(),
        state.run_id.as_str(),
        node.as_str(),
        reservation_id,
    ] {
        hasher.update(&(value.len() as u64).to_le_bytes());
        hasher.update(value.as_bytes());
    }
    hasher.update(&generation.to_le_bytes());
    hasher.update(&state_version.to_le_bytes());
    hasher.finalize().to_hex().to_string()
}

// ---------------------------------------------------------------------------
// evaluate_gate
// ---------------------------------------------------------------------------

/// Decide whether `node` may run, given how it is classified and whether anybody
/// is watching. Pure: same inputs, same decision, on any machine.
///
/// The order of the checks is the order of certainty, most upstream cause first:
///
/// 1. A node the graph does not contain is refused. Treating it as safe would
///    let a typo in a dispatch loop bypass the gate entirely.
/// 2. A malformed classification is refused, because somebody tried to classify
///    this node and the result is unreadable.
/// 3. A recorded human decision wins over the level, in BOTH modes. That is the
///    asynchronous half of R-DESTRUCT closing: a dispatched run escalated, a
///    human approved or denied out of band, the resumed run honours it. An
///    approval only reaches the state through [`approve`], which refuses to
///    build an unattributed one.
/// 4. Otherwise the level and the mode decide: `Safe` proceeds, `Elevated`
///    proceeds only attended, `Irreversible` never proceeds on its own in either
///    mode.
pub fn evaluate_gate(
    graph: &Graph,
    state: &GraphState,
    node: &NodeId,
    mode: ExecutionMode,
    authority: &GraphExecutionAuthority,
) -> GateDecision {
    evaluate_gate_inner(graph, state, node, mode, None, authority)
}

/// Clock-explicit gate evaluation. Time-bounded resolutions are honored only by
/// this API; the clockless compatibility API fails closed on expiring consent.
pub fn evaluate_gate_at(
    graph: &Graph,
    state: &GraphState,
    node: &NodeId,
    mode: ExecutionMode,
    now: DateTime<Utc>,
    authority: &GraphExecutionAuthority,
) -> GateDecision {
    evaluate_gate_inner(graph, state, node, mode, Some(now), authority)
}

fn evaluate_gate_inner(
    graph: &Graph,
    state: &GraphState,
    node: &NodeId,
    mode: ExecutionMode,
    now: Option<DateTime<Utc>>,
    authority: &GraphExecutionAuthority,
) -> GateDecision {
    if let Err(error) = state.validate_for_graph_with_authority(graph, authority) {
        return GateDecision::Refuse {
            node: node.clone(),
            reason: format!("invalid graph execution state: {error}"),
        };
    }
    let Some(declared_node) = graph.node(node) else {
        return GateDecision::Refuse {
            node: node.clone(),
            reason: RiskError::UnknownNode(node.0.clone()).to_string(),
        };
    };

    let risk = match risk_of(declared_node) {
        Ok(risk) => risk,
        Err(err) => {
            return GateDecision::Refuse {
                node: node.clone(),
                reason: err.to_string(),
            }
        }
    };

    let subject_digest = approval_subject_digest(graph, state, node);
    match resolution_for_subject(graph, state, node, &subject_digest, now, authority) {
        Some(resolution) if resolution.verdict == ResolutionVerdict::Approved => {
            return GateDecision::Proceed
        }
        Some(resolution) => {
            return GateDecision::Refuse {
                node: node.clone(),
                reason: format!(
                    "{} denied this {risk} node: {}",
                    resolution.approver, resolution.record.what_is_lost
                ),
            }
        }
        None => {}
    }

    let standalone_effect = standalone_effect_requires_approval(declared_node, state);
    if !standalone_effect
        && (risk.runs_unattended()
            || (risk == RiskLevel::Elevated && mode == ExecutionMode::Attended))
    {
        return GateDecision::Proceed;
    }

    GateDecision::RequireApproval {
        node: node.clone(),
        risk,
        reason: format!(
            "{}{SUBJECT_MARKER_PREFIX}{subject_digest}]",
            approval_reason_for(declared_node, state, risk, mode)
        ),
        what_is_lost: what_is_lost_for(declared_node, state, risk),
    }
}

/// Why this node is being held. Prefers the node's own declaration, and falls
/// back to a statement that is true of every node at that level rather than to
/// silence.
fn approval_reason(node: &Node, risk: RiskLevel, mode: ExecutionMode) -> String {
    if let Some(declared) = declared(node, RISK_REASON_KEY) {
        return declared;
    }
    match risk {
        RiskLevel::Irreversible => {
            "classified irreversible and no reason was declared on the node".to_string()
        }
        _ => format!(
            "classified {risk} and this run is {mode}, so no human is present to approve it inline"
        ),
    }
}

/// What an operator loses if this node runs, and it is NEVER empty (invariant 3).
///
/// The synthesized fallbacks are deliberately worded as what the gate actually
/// knows rather than as invented damage. For an irreversible node with nothing
/// declared, "assume every effect is permanent" is the honest reading of the
/// classification and is exactly the assumption a human should make. For an
/// elevated node it says the recovery cost is unstated, which is the true fact,
/// instead of implying a loss the author never claimed.
fn what_is_lost(node: &Node, risk: RiskLevel) -> String {
    if let Some(declared) = declared(node, RISK_WHAT_IS_LOST_KEY) {
        return declared;
    }
    match risk {
        RiskLevel::Irreversible => format!(
            "not declared by node {}: assume every effect of this step is permanent \
             and that no later step can restore it",
            node.id
        ),
        _ => format!(
            "not declared by node {}: the step is recoverable but the cost of \
             undoing it is unstated",
            node.id
        ),
    }
}

// ---------------------------------------------------------------------------
// EscalationRecord
// ---------------------------------------------------------------------------

/// The durable artifact an unattended run leaves behind instead of a prompt.
///
/// This IS the block-file R-DESTRUCT describes, as a type: it names the step,
/// why it is held, and what would be lost, so an operator reading it hours later
/// has everything the inline prompt would have shown and nothing was lost to a
/// terminal nobody was looking at.
///
/// Same forward compatibility contract as [`crate::graph::Graph`]: every field
/// carries `#[serde(default)]` and the struct carries a flattened `extra`, so a
/// record written by a newer OmegaOS round-trips through this version with the
/// keys it has never heard of intact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EscalationRecord {
    #[serde(default = "unnamed_node")]
    pub node: NodeId,
    #[serde(default)]
    pub risk: RiskLevel,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub what_is_lost: String,
    /// Digest of the graph, run and reserved generation shown to the operator.
    /// Empty legacy records are deliberately never sufficient to authorize work.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub subject_digest: String,
    #[serde(default = "epoch")]
    pub recorded_at: DateTime<Utc>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// The serde fallback for a record that predates `recorded_at`. A fixed, obviously
/// wrong instant rather than `Utc::now()`: a record with no timestamp is not a
/// record that happened just now, and stamping it with the read time would make
/// an undated escalation look like a fresh one every time it is loaded.
fn epoch() -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp(0, 0).unwrap_or_default()
}

/// The serde fallback for a record whose `node` key is missing. [`NodeId`] has no
/// `Default` and deliberately should not gain one in `graph.rs`, so the empty id
/// is named here instead: it matches no node in any graph, which means a record
/// that lost its subject can never resolve onto a real one.
fn unnamed_node() -> NodeId {
    NodeId::new("")
}

impl EscalationRecord {
    /// Build the record for a held node directly, for callers that already know
    /// the decision they are recording.
    pub fn new(
        node: impl Into<NodeId>,
        risk: RiskLevel,
        reason: impl Into<String>,
        what_is_lost: impl Into<String>,
        recorded_at: DateTime<Utc>,
    ) -> Self {
        Self {
            node: node.into(),
            risk,
            reason: reason.into(),
            what_is_lost: what_is_lost.into(),
            subject_digest: String::new(),
            recorded_at,
            extra: Map::new(),
        }
    }

    /// One line for the alert funnel. Short on purpose: the funnel carries the
    /// pointer, the record carries the detail.
    pub fn headline(&self) -> String {
        format!(
            "{} node {} needs a human: {} (lost: {})",
            self.risk, self.node, self.reason, self.what_is_lost
        )
    }
}

// ---------------------------------------------------------------------------
// Resolution
// ---------------------------------------------------------------------------

/// Which way a human went on an escalation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResolutionVerdict {
    Approved,
    Denied,
}

/// A human decision on one [`EscalationRecord`], with the human named.
///
/// The record travels WITH the verdict rather than being replaced by it, so the
/// audit trail keeps what was actually shown at approval time. An approval read
/// back next to a record that has since been reworded would be an approval of
/// something else.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateResolution {
    #[serde(default = "empty_record")]
    pub record: EscalationRecord,
    #[serde(default = "denied_verdict")]
    pub verdict: ResolutionVerdict,
    #[serde(default)]
    pub approver: String,
    /// Optional hard expiry. Clockless evaluation never treats a time-bounded
    /// approval as valid because it cannot prove that it is still live.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub authority_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub graph_digest: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub run_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub reservation_id: String,
    #[serde(default)]
    pub generation: u64,
    #[serde(default)]
    pub reservation_state_version: u64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub resolution_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub authority_mac: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct GateConsumptionReceipt {
    node: NodeId,
    resolution_id: String,
    reservation_id: String,
    consumed_at: DateTime<Utc>,
    consumption_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    authority_mac: String,
}

impl GateConsumptionReceipt {
    fn new(
        resolution: &GateResolution,
        consumed_at: DateTime<Utc>,
        authority: &GraphExecutionAuthority,
    ) -> Self {
        let mut receipt = Self {
            node: resolution.record.node.clone(),
            resolution_id: resolution.resolution_id.clone(),
            reservation_id: resolution.reservation_id.clone(),
            consumed_at,
            consumption_id: String::new(),
            authority_mac: String::new(),
        };
        receipt.consumption_id = consumption_id(&receipt);
        receipt.authority_mac = consumption_mac(&receipt, authority);
        receipt
    }

    fn authenticate(
        &self,
        node: &NodeId,
        resolution: &GateResolution,
        authority: &GraphExecutionAuthority,
    ) -> bool {
        self.node == *node
            && self.resolution_id == resolution.resolution_id
            && self.reservation_id == resolution.reservation_id
            && self.consumption_id == consumption_id(self)
            && self.authority_mac == consumption_mac(self, authority)
    }

    fn authenticate_standalone(&self, node: &NodeId, authority: &GraphExecutionAuthority) -> bool {
        self.node == *node
            && self.consumption_id == consumption_id(self)
            && self.authority_mac == consumption_mac(self, authority)
    }
}

fn empty_record() -> EscalationRecord {
    EscalationRecord::new(NodeId::new(""), RiskLevel::default(), "", "", epoch())
}

/// The serde fallback verdict is `Denied`, never `Approved`. A malformed or
/// truncated resolution document must not be able to decay into consent.
fn denied_verdict() -> ResolutionVerdict {
    ResolutionVerdict::Denied
}

fn canonicalize_json(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let sorted: std::collections::BTreeMap<String, Value> = map
                .into_iter()
                .map(|(key, value)| (key, canonicalize_json(value)))
                .collect();
            Value::Object(sorted.into_iter().collect())
        }
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize_json).collect()),
        other => other,
    }
}

fn record_digest(record: &EscalationRecord) -> String {
    let value = serde_json::to_value(record)
        .map(canonicalize_json)
        .unwrap_or(Value::Null);
    let bytes = serde_json::to_vec(&value).unwrap_or_default();
    blake3::hash(&bytes).to_hex().to_string()
}

fn verdict_name(verdict: ResolutionVerdict) -> &'static str {
    match verdict {
        ResolutionVerdict::Approved => "approved",
        ResolutionVerdict::Denied => "denied",
    }
}

fn resolution_id(resolution: &GateResolution) -> String {
    let record_digest = record_digest(&resolution.record);
    let expires_at = resolution
        .expires_at
        .map(|value| value.to_rfc3339())
        .unwrap_or_default();
    let mut hasher = blake3::Hasher::new();
    for value in [
        record_digest.as_str(),
        verdict_name(resolution.verdict),
        resolution.approver.as_str(),
        expires_at.as_str(),
        resolution.authority_id.as_str(),
        resolution.graph_digest.as_str(),
        resolution.run_id.as_str(),
        resolution.record.node.as_str(),
        resolution.reservation_id.as_str(),
        resolution.record.subject_digest.as_str(),
    ] {
        hasher.update(&(value.len() as u64).to_le_bytes());
        hasher.update(value.as_bytes());
    }
    hasher.update(&resolution.generation.to_le_bytes());
    hasher.update(&resolution.reservation_state_version.to_le_bytes());
    hasher.finalize().to_hex().to_string()
}

fn resolution_mac(resolution: &GateResolution, authority: &GraphExecutionAuthority) -> String {
    authority.mac(
        "omega.graph.gate-resolution.v1",
        &[
            resolution.resolution_id.as_bytes(),
            resolution.authority_id.as_bytes(),
            resolution.graph_digest.as_bytes(),
            resolution.run_id.as_bytes(),
            resolution.record.node.as_str().as_bytes(),
            resolution.reservation_id.as_bytes(),
            resolution.record.subject_digest.as_bytes(),
            verdict_name(resolution.verdict).as_bytes(),
            resolution.approver.as_bytes(),
        ],
    )
}

fn consumption_id(receipt: &GateConsumptionReceipt) -> String {
    let consumed_at = receipt.consumed_at.to_rfc3339();
    let mut hasher = blake3::Hasher::new();
    for value in [
        receipt.node.as_str(),
        receipt.resolution_id.as_str(),
        receipt.reservation_id.as_str(),
        consumed_at.as_str(),
    ] {
        hasher.update(&(value.len() as u64).to_le_bytes());
        hasher.update(value.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn consumption_mac(
    receipt: &GateConsumptionReceipt,
    authority: &GraphExecutionAuthority,
) -> String {
    authority.mac(
        "omega.graph.gate-consumption.v1",
        &[
            receipt.consumption_id.as_bytes(),
            receipt.node.as_str().as_bytes(),
            receipt.resolution_id.as_bytes(),
            receipt.reservation_id.as_bytes(),
        ],
    )
}

impl GateResolution {
    pub fn is_approved(&self) -> bool {
        self.verdict == ResolutionVerdict::Approved
    }

    fn authenticate(
        &self,
        graph: &Graph,
        state: &GraphState,
        node: &NodeId,
        authority: &GraphExecutionAuthority,
    ) -> bool {
        let Some(reservation) = state.reservation_of(node) else {
            return false;
        };
        self.record.node == *node
            && self.record.subject_digest == approval_subject_digest(graph, state, node)
            && self.authority_id == authority.authority_id()
            && self.graph_digest == state.graph_digest
            && self.run_id == state.run_id
            && self.reservation_id == reservation.reservation_id
            && self.generation == reservation.generation
            && self.reservation_state_version == reservation.state_version
            && self.resolution_id == resolution_id(self)
            && self.authority_mac == resolution_mac(self, authority)
    }
}

/// Approve an escalated node, attributably.
///
/// An empty or whitespace approver is rejected with a typed error rather than
/// accepted as anonymous. This is the check that stops an agent from writing its
/// own permission slip: an approval nobody signed is indistinguishable from one
/// the process invented for itself, and the record is worthless as evidence.
pub fn approve(
    graph: &Graph,
    state: &GraphState,
    record: EscalationRecord,
    approver: impl Into<String>,
    authority: &GraphExecutionAuthority,
) -> Result<GateResolution, RiskError> {
    resolve(
        graph,
        state,
        record,
        approver.into(),
        ResolutionVerdict::Approved,
        None,
        authority,
    )
}

pub fn approve_until(
    graph: &Graph,
    state: &GraphState,
    record: EscalationRecord,
    approver: impl Into<String>,
    expires_at: DateTime<Utc>,
    authority: &GraphExecutionAuthority,
) -> Result<GateResolution, RiskError> {
    resolve(
        graph,
        state,
        record,
        approver.into(),
        ResolutionVerdict::Approved,
        Some(expires_at),
        authority,
    )
}

/// Deny an escalated node, attributably. A denial is held to the same standard:
/// an operator reading the run later needs to know who blocked it, and an
/// unsigned denial is as unauditable as an unsigned approval.
pub fn deny(
    graph: &Graph,
    state: &GraphState,
    record: EscalationRecord,
    approver: impl Into<String>,
    authority: &GraphExecutionAuthority,
) -> Result<GateResolution, RiskError> {
    resolve(
        graph,
        state,
        record,
        approver.into(),
        ResolutionVerdict::Denied,
        None,
        authority,
    )
}

pub fn deny_until(
    graph: &Graph,
    state: &GraphState,
    record: EscalationRecord,
    approver: impl Into<String>,
    expires_at: DateTime<Utc>,
    authority: &GraphExecutionAuthority,
) -> Result<GateResolution, RiskError> {
    resolve(
        graph,
        state,
        record,
        approver.into(),
        ResolutionVerdict::Denied,
        Some(expires_at),
        authority,
    )
}

fn resolve(
    graph: &Graph,
    state: &GraphState,
    record: EscalationRecord,
    approver: String,
    verdict: ResolutionVerdict,
    expires_at: Option<DateTime<Utc>>,
    authority: &GraphExecutionAuthority,
) -> Result<GateResolution, RiskError> {
    let approver = approver.trim().to_string();
    if approver.is_empty() {
        return Err(RiskError::UnattributedApproval {
            node: record.node.0.clone(),
        });
    }
    if record.subject_digest.len() != 64
        || !record
            .subject_digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(RiskError::UnboundApprovalSubject {
            node: record.node.0.clone(),
        });
    }
    state
        .validate_for_graph_with_authority(graph, authority)
        .map_err(|error| RiskError::InvalidApprovalAuthority {
            node: record.node.0.clone(),
            reason: error.to_string(),
        })?;
    let reservation =
        state
            .reservation_of(&record.node)
            .ok_or_else(|| RiskError::InvalidApprovalAuthority {
                node: record.node.0.clone(),
                reason: "node has no active authenticated reservation".to_string(),
            })?;
    if record.subject_digest != approval_subject_digest(graph, state, &record.node) {
        return Err(RiskError::InvalidApprovalAuthority {
            node: record.node.0.clone(),
            reason: "escalation subject does not match the active graph/run/generation".to_string(),
        });
    }
    let declared_node = graph
        .node(&record.node)
        .ok_or_else(|| RiskError::UnknownNode(record.node.0.clone()))?;
    let risk = risk_of(declared_node)?;
    let requires_unattended_approval =
        standalone_effect_requires_approval(declared_node, state) || !risk.runs_unattended();
    if !requires_unattended_approval
        || record.risk != risk
        || record.reason
            != approval_reason_for(declared_node, state, risk, ExecutionMode::Unattended)
        || record.what_is_lost != what_is_lost_for(declared_node, state, risk)
    {
        return Err(RiskError::InvalidApprovalAuthority {
            node: record.node.0.clone(),
            reason: "escalation record is not the exact currently held decision".to_string(),
        });
    }
    if expires_at
        .as_ref()
        .is_some_and(|expires_at| *expires_at <= record.recorded_at)
    {
        return Err(RiskError::InvalidExpiration {
            node: record.node.0.clone(),
        });
    }
    let mut resolution = GateResolution {
        record,
        verdict,
        approver,
        expires_at,
        authority_id: authority.authority_id(),
        graph_digest: state.graph_digest.clone(),
        run_id: state.run_id.clone(),
        reservation_id: reservation.reservation_id.clone(),
        generation: reservation.generation,
        reservation_state_version: reservation.state_version,
        resolution_id: String::new(),
        authority_mac: String::new(),
        extra: Map::new(),
    };
    resolution.resolution_id = resolution_id(&resolution);
    resolution.authority_mac = resolution_mac(&resolution, authority);
    Ok(resolution)
}

// ---------------------------------------------------------------------------
// Resolutions recorded in GraphState::extra
// ---------------------------------------------------------------------------

/// One namespaced key rather than several top-level ones, so this module cannot
/// collide with [`crate::graph_executor`]'s bag or any other writer's entries.
const RISK_BAG: &str = "graph_risk";
const RESOLUTIONS_KEY: &str = "resolutions";
const CONSUMED_KEY: &str = "consumed_resolutions";

/// Record a human decision into the run state, where [`evaluate_gate`] will find
/// it on the next pass. In-memory only: persisting the state is the caller's job,
/// and a decision core that wrote a file could not be replayed.
pub fn record_resolution(
    graph: &Graph,
    state: &mut GraphState,
    resolution: &GateResolution,
    authority: &GraphExecutionAuthority,
) -> Result<(), RiskError> {
    if !resolution.authenticate(graph, state, &resolution.record.node, authority) {
        return Err(RiskError::InvalidApprovalAuthority {
            node: resolution.record.node.0.clone(),
            reason: "resolution signature or active subject binding is invalid".to_string(),
        });
    }
    match consumptions_for_node(state, &resolution.record.node, authority) {
        Ok(consumed) if consumed.contains_key(&resolution.resolution_id) => {
            return Err(RiskError::InvalidApprovalAuthority {
                node: resolution.record.node.0.clone(),
                reason: "this exact one-shot resolution was already consumed".to_string(),
            });
        }
        Ok(_) => {}
        Err(reason) => {
            return Err(RiskError::InvalidApprovalAuthority {
                node: resolution.record.node.0.clone(),
                reason,
            });
        }
    }
    let value =
        serde_json::to_value(resolution).map_err(|error| RiskError::InvalidApprovalAuthority {
            node: resolution.record.node.0.clone(),
            reason: format!("resolution cannot be persisted: {error}"),
        })?;
    let bag = state
        .extra
        .entry(RISK_BAG.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    if !bag.is_object() {
        *bag = Value::Object(Map::new());
    }
    let Some(bag) = bag.as_object_mut() else {
        return Err(RiskError::InvalidApprovalAuthority {
            node: resolution.record.node.0.clone(),
            reason: "risk state namespace could not be initialized".to_string(),
        });
    };
    let slot = bag
        .entry(RESOLUTIONS_KEY.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    if !slot.is_object() {
        *slot = Value::Object(Map::new());
    }
    if let Some(map) = slot.as_object_mut() {
        map.insert(resolution.record.node.0.clone(), value);
    }
    state.mark_updated();
    Ok(())
}

/// The decision recorded for `node`, if any.
///
/// A malformed entry reads as NO decision rather than as an error or a partial
/// one. That fails closed: the gate falls back to asking again, which is always
/// safe, whereas half-parsing a resolution document could turn a corrupted key
/// into a proceed.
///
/// The map key alone is NOT trusted to identify the subject: the parsed record
/// must name the same node. A state document is editable text, and an approval
/// filed under the wrong key would otherwise let consent given for a cheap node
/// unlock an expensive one, which is the exact substitution an approval gate has
/// to make impossible.
pub fn resolution_of(state: &GraphState, node: &NodeId) -> Option<GateResolution> {
    let value = state
        .extra
        .get(RISK_BAG)?
        .as_object()?
        .get(RESOLUTIONS_KEY)?
        .as_object()?
        .get(node.as_str())?;
    let resolution: GateResolution = serde_json::from_value(value.clone()).ok()?;
    (resolution.record.node == *node).then_some(resolution)
}

fn resolution_for_subject(
    graph: &Graph,
    state: &GraphState,
    node: &NodeId,
    subject_digest: &str,
    now: Option<DateTime<Utc>>,
    authority: &GraphExecutionAuthority,
) -> Option<GateResolution> {
    let resolution = resolution_of(state, node)?;
    if resolution.record.subject_digest.is_empty()
        || resolution.record.subject_digest != subject_digest
        || !resolution.authenticate(graph, state, node, authority)
    {
        return None;
    }
    match consumptions_for_node(state, node, authority) {
        Ok(consumed) => {
            if consumed
                .get(&resolution.resolution_id)
                .is_some_and(|receipt| receipt.authenticate(node, &resolution, authority))
            {
                return None;
            }
        }
        Err(_) => return None,
    }
    match (resolution.expires_at, now) {
        (None, _) => Some(resolution),
        (Some(expires_at), Some(now)) if expires_at > now => Some(resolution),
        // With no clock, validity cannot be established. At or after expiry,
        // consent has ended. Both cases fail closed by asking again.
        _ => None,
    }
}

/// Consume and return a recorded decision. Callers that authorize an external
/// effect can use this to make consent one-shot in addition to generation-bound.
fn consumptions_for_node(
    state: &GraphState,
    node: &NodeId,
    authority: &GraphExecutionAuthority,
) -> Result<std::collections::BTreeMap<String, GateConsumptionReceipt>, String> {
    let Some(risk) = state.extra.get(RISK_BAG) else {
        return Ok(std::collections::BTreeMap::new());
    };
    let risk = risk
        .as_object()
        .ok_or_else(|| "risk state namespace is not an object".to_string())?;
    let Some(consumed) = risk.get(CONSUMED_KEY) else {
        return Ok(std::collections::BTreeMap::new());
    };
    let consumed = consumed
        .as_object()
        .ok_or_else(|| "consumption collection is not an object".to_string())?;
    let Some(entries) = consumed.get(node.as_str()) else {
        return Ok(std::collections::BTreeMap::new());
    };
    let entries = entries
        .as_object()
        .ok_or_else(|| "node consumption history is not an object".to_string())?;
    let mut parsed = std::collections::BTreeMap::new();
    for (resolution_id, value) in entries {
        let receipt: GateConsumptionReceipt = serde_json::from_value(value.clone())
            .map_err(|error| format!("consumption receipt cannot be decoded: {error}"))?;
        if receipt.resolution_id != *resolution_id
            || !receipt.authenticate_standalone(node, authority)
        {
            return Err("consumption receipt identity or authority MAC is invalid".to_string());
        }
        parsed.insert(resolution_id.clone(), receipt);
    }
    Ok(parsed)
}

fn consume_resolution(
    state: &mut GraphState,
    node: &NodeId,
    consumed_at: DateTime<Utc>,
    authority: &GraphExecutionAuthority,
) -> Option<GateResolution> {
    let resolution = resolution_of(state, node)?;
    if resolution.record.node != *node {
        return None;
    }
    let consumed = GateConsumptionReceipt::new(&resolution, consumed_at, authority);
    let value = serde_json::to_value(consumed).ok()?;
    let bag = state.extra.get_mut(RISK_BAG)?.as_object_mut()?;
    bag.get_mut(RESOLUTIONS_KEY)?
        .as_object_mut()?
        .remove(node.as_str())?;
    let slot = bag
        .entry(CONSUMED_KEY.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    if !slot.is_object() {
        *slot = Value::Object(Map::new());
    }
    let history = slot
        .as_object_mut()?
        .entry(node.0.clone())
        .or_insert_with(|| Value::Object(Map::new()));
    if !history.is_object() {
        return None;
    }
    history
        .as_object_mut()?
        .insert(resolution.resolution_id.clone(), value);
    state.mark_updated();
    Some(resolution)
}

/// Evaluate with an explicit clock and consume a matching approval before
/// returning `Proceed`. Safe nodes proceed without touching the resolution bag.
pub fn authorize_gate_at(
    graph: &Graph,
    state: &mut GraphState,
    node: &NodeId,
    mode: ExecutionMode,
    now: DateTime<Utc>,
    authority: &GraphExecutionAuthority,
) -> GateDecision {
    let subject_digest = approval_subject_digest(graph, state, node);
    let consumes_approval =
        resolution_for_subject(graph, state, node, &subject_digest, Some(now), authority)
            .is_some_and(|resolution| resolution.is_approved());
    let decision = evaluate_gate_at(graph, state, node, mode, now, authority);
    if decision.proceeds() && consumes_approval {
        let _ = consume_resolution(state, node, now, authority);
    }
    decision
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{GraphExecutionAuthority, NodeKind};

    const BOTH_MODES: [ExecutionMode; 2] = [ExecutionMode::Attended, ExecutionMode::Unattended];

    fn stamp() -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(1_800_000_000, 0).expect("valid instant")
    }

    fn authority() -> GraphExecutionAuthority {
        GraphExecutionAuthority::from_key([0x52; 32])
    }

    fn evaluate_gate(
        graph: &Graph,
        state: &GraphState,
        node: &NodeId,
        mode: ExecutionMode,
    ) -> GateDecision {
        let authority = authority();
        let mut bound = state.clone();
        if bound.authority_id.is_empty() {
            bound.bind_authority_if_pristine(&authority).unwrap();
        }
        super::evaluate_gate(graph, &bound, node, mode, &authority)
    }

    fn evaluate_gate_at(
        graph: &Graph,
        state: &GraphState,
        node: &NodeId,
        mode: ExecutionMode,
        now: DateTime<Utc>,
    ) -> GateDecision {
        let authority = authority();
        let mut bound = state.clone();
        if bound.authority_id.is_empty() {
            bound.bind_authority_if_pristine(&authority).unwrap();
        }
        super::evaluate_gate_at(graph, &bound, node, mode, now, &authority)
    }

    /// A migration graph: a safe read, an elevated write, and the drop nobody
    /// should ever run unattended.
    fn graph() -> Graph {
        Graph::new()
            .with_node(with_risk(
                Node::new("read_schema", NodeKind::Agent),
                RiskLevel::Safe,
            ))
            .with_node(with_risk_detail(
                Node::new("rewrite_config", NodeKind::Agent),
                RiskLevel::Elevated,
                "rewrites a live config file",
                "the current config, until it is restored from git",
            ))
            .with_node(with_risk_detail(
                Node::new("drop_table", NodeKind::Agent),
                RiskLevel::Irreversible,
                "drops a production table",
                "every row in orders, with no backup in this graph",
            ))
            .with_edge("read_schema", "rewrite_config")
            .with_edge("rewrite_config", "drop_table")
    }

    fn standalone_effect(risk: Option<RiskLevel>) -> Graph {
        let mut node = Node::new("effect", NodeKind::Agent);
        node.extra
            .insert("command".to_string(), Value::from("printf ok"));
        if let Some(risk) = risk {
            node = with_risk(node, risk);
        }
        Graph::new().with_node(node)
    }

    #[test]
    fn safe_node_proceeds_in_both_modes() {
        let graph = graph();
        let state = GraphState::for_graph(&graph);
        for mode in BOTH_MODES {
            assert_eq!(
                evaluate_gate(&graph, &state, &NodeId::new("read_schema"), mode),
                GateDecision::Proceed,
                "a safe node must not need a human in {mode} mode"
            );
        }
    }

    #[test]
    fn standalone_shell_effect_requires_exact_one_shot_approval_in_both_modes() {
        let graph = standalone_effect(Some(RiskLevel::Safe));
        let node = NodeId::new("effect");
        let authority = authority();
        let mut state =
            GraphState::for_graph_with_authority(&graph, "standalone-effect", &authority);
        state.reserve(&node, &authority).unwrap();

        for mode in BOTH_MODES {
            let decision = super::evaluate_gate(&graph, &state, &node, mode, &authority);
            assert!(
                decision.requires_approval(),
                "standalone safe effect must be held in {mode} mode, got {decision:?}"
            );
        }
        let escalation =
            super::evaluate_gate(&graph, &state, &node, ExecutionMode::Unattended, &authority)
                .into_escalation(stamp())
                .expect("unattended standalone effect must escalate");
        assert!(escalation.reason.contains("no immutable V3 mission-plan"));
        let resolution = approve(&graph, &state, escalation, "operator", &authority).unwrap();
        record_resolution(&graph, &mut state, &resolution, &authority).unwrap();

        assert_eq!(
            authorize_gate_at(
                &graph,
                &mut state,
                &node,
                ExecutionMode::Attended,
                stamp(),
                &authority,
            ),
            GateDecision::Proceed
        );
        assert!(
            super::evaluate_gate(&graph, &state, &node, ExecutionMode::Attended, &authority)
                .requires_approval(),
            "the exact standalone-effect approval must be consumed once"
        );
    }

    #[test]
    fn unclassified_standalone_shell_effect_is_held_attended() {
        let graph = standalone_effect(None);
        let node = NodeId::new("effect");
        let authority = authority();
        let mut state =
            GraphState::for_graph_with_authority(&graph, "unclassified-effect", &authority);
        state.reserve(&node, &authority).unwrap();
        assert!(
            super::evaluate_gate(&graph, &state, &node, ExecutionMode::Attended, &authority)
                .requires_approval()
        );
    }

    #[test]
    fn plan_bound_low_risk_shell_effect_uses_v3_authority_and_proceeds() {
        use crate::mission::{
            MissionId, PlanContract, RetryPolicy, TaskContract, TaskId, VerifierCheck,
            VerifierCheckKind, CONTRACT_SCHEMA_VERSION,
        };

        let check = VerifierCheck {
            schema_version: CONTRACT_SCHEMA_VERSION,
            check_id: "effect-check".to_string(),
            kind: VerifierCheckKind::Command {
                argv: vec!["true".to_string()],
                cwd: None,
                expected_exit_code: 0,
            },
            timeout_secs: 5,
        };
        let task = TaskContract {
            schema_version: CONTRACT_SCHEMA_VERSION,
            task_id: TaskId::new("effect-task"),
            name: "effect task".to_string(),
            prompt: "run the effect".to_string(),
            acceptance_criteria: vec!["effect verified".to_string()],
            verifier_checks: vec![check.clone()],
            required_capabilities: Vec::new(),
            scope: Vec::new(),
            risk: crate::routing::RiskLevel::Low,
            retry_policy: RetryPolicy::default(),
            depends_on: Vec::new(),
        };
        let plan = PlanContract::new(
            MissionId("plan-bound-effect".to_string()),
            1,
            1,
            vec![task.clone()],
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
        let mut node = with_risk(
            Node::new("effect", NodeKind::Agent)
                .with_task(task.task_id.clone())
                .with_retry(task.retry_policy.clone())
                .with_checks(vec![check]),
            RiskLevel::Safe,
        );
        node.extra
            .insert("command".to_string(), Value::from("printf ok"));
        let graph = Graph::new().with_node(node);
        let authority = authority();
        let mut state = GraphState::for_graph_with_plan_and_authority(
            &graph,
            "plan-bound-effect",
            &plan,
            &authority,
        )
        .unwrap();
        let node = NodeId::new("effect");
        state.reserve(&node, &authority).unwrap();

        for mode in BOTH_MODES {
            assert_eq!(
                super::evaluate_gate(&graph, &state, &node, mode, &authority),
                GateDecision::Proceed,
                "the exact V3 Low-risk plan is the classification authority in {mode} mode"
            );
        }
    }

    #[test]
    fn irreversible_never_proceeds_in_either_mode() {
        let graph = graph();
        let state = GraphState::for_graph(&graph);
        for mode in BOTH_MODES {
            let decision = evaluate_gate(&graph, &state, &NodeId::new("drop_table"), mode);
            assert!(
                decision.requires_approval(),
                "irreversible must be held in {mode} mode, got {decision:?}"
            );
            assert!(!decision.proceeds());
        }
    }

    #[test]
    fn elevated_proceeds_attended_but_not_unattended() {
        let graph = graph();
        let state = GraphState::for_graph(&graph);
        let node = NodeId::new("rewrite_config");

        assert_eq!(
            evaluate_gate(&graph, &state, &node, ExecutionMode::Attended),
            GateDecision::Proceed
        );
        let held = evaluate_gate(&graph, &state, &node, ExecutionMode::Unattended);
        assert!(held.requires_approval(), "got {held:?}");
    }

    #[test]
    fn approval_request_always_carries_what_is_lost() {
        // Both halves of invariant 3: the declared case, and the case where the
        // author classified the node and declared nothing else.
        let bare = Graph::new().with_node(with_risk(
            Node::new("undeclared", NodeKind::Agent),
            RiskLevel::Irreversible,
        ));
        for (graph, id) in [
            (graph(), "drop_table"),
            (bare.clone(), "undeclared"),
            (bare, "undeclared"),
        ] {
            let state = GraphState::for_graph(&graph);
            for mode in BOTH_MODES {
                match evaluate_gate(&graph, &state, &NodeId::new(id), mode) {
                    GateDecision::RequireApproval {
                        reason,
                        what_is_lost,
                        ..
                    } => {
                        assert!(!what_is_lost.trim().is_empty(), "{id} in {mode} mode");
                        assert!(!reason.trim().is_empty(), "{id} in {mode} mode");
                    }
                    other => panic!("expected an approval request for {id}, got {other:?}"),
                }
            }
        }
    }

    #[test]
    fn unattended_irreversible_produces_a_record_that_round_trips() {
        let graph = graph();
        let state = GraphState::for_graph(&graph);
        let decision = evaluate_gate(
            &graph,
            &state,
            &NodeId::new("drop_table"),
            ExecutionMode::Unattended,
        );

        let record = decision
            .into_escalation(stamp())
            .expect("an unattended irreversible node must leave an artifact");
        assert_eq!(record.node, NodeId::new("drop_table"));
        assert_eq!(record.risk, RiskLevel::Irreversible);
        assert_eq!(record.subject_digest.len(), 64);
        assert!(record.what_is_lost.contains("every row in orders"));
        assert!(record.headline().contains("needs a human"));

        let json = serde_json::to_value(&record).expect("serializes");
        assert_eq!(json["risk"], Value::from("irreversible"));
        let back: EscalationRecord = serde_json::from_value(json).expect("deserializes");
        assert_eq!(back, record);

        // And a proceed has nothing to escalate.
        assert_eq!(GateDecision::Proceed.into_escalation(stamp()), None);
    }

    #[test]
    fn unattributed_approval_is_rejected() {
        let graph = graph();
        let authority = authority();
        let node = NodeId::new("drop_table");
        let mut state = GraphState::for_graph_with_authority(&graph, "run-attribution", &authority);
        state.reserve(&node, &authority).unwrap();
        let record = evaluate_gate(&graph, &state, &node, ExecutionMode::Unattended)
            .into_escalation(stamp())
            .unwrap();

        for blank in ["", "   ", "\t\n"] {
            assert_eq!(
                approve(&graph, &state, record.clone(), blank, &authority),
                Err(RiskError::UnattributedApproval {
                    node: "drop_table".to_string(),
                })
            );
            assert_eq!(
                deny(&graph, &state, record.clone(), blank, &authority),
                Err(RiskError::UnattributedApproval {
                    node: "drop_table".to_string(),
                })
            );
        }

        let approved = approve(&graph, &state, record, " operator ", &authority)
            .expect("attributed approval is accepted");
        assert!(approved.is_approved());
        assert_eq!(approved.approver, "operator", "approver is trimmed");
        assert_eq!(approved.authority_id, authority.authority_id());
        assert!(!approved.authority_mac.is_empty());
    }

    #[test]
    fn a_recorded_human_decision_closes_the_loop_in_both_modes() {
        // The asynchronous half of R-DESTRUCT: escalate, a human answers out of
        // band, the resumed run honours the answer instead of asking an empty
        // room again.
        let graph = graph();
        let node = NodeId::new("drop_table");
        let authority = authority();

        let mut approved_state =
            GraphState::for_graph_with_authority(&graph, "run-approved", &authority);
        approved_state.reserve(&node, &authority).unwrap();
        let record = evaluate_gate(&graph, &approved_state, &node, ExecutionMode::Unattended)
            .into_escalation(stamp())
            .expect("held");
        let approved =
            approve(&graph, &approved_state, record, "operator", &authority).expect("attributed");
        record_resolution(&graph, &mut approved_state, &approved, &authority).unwrap();

        let mut denied_state =
            GraphState::for_graph_with_authority(&graph, "run-denied", &authority);
        denied_state.reserve(&node, &authority).unwrap();
        let denied_record = evaluate_gate(&graph, &denied_state, &node, ExecutionMode::Unattended)
            .into_escalation(stamp())
            .expect("held");
        let denied =
            deny(&graph, &denied_state, denied_record, "operator", &authority).expect("attributed");
        record_resolution(&graph, &mut denied_state, &denied, &authority).unwrap();

        for mode in BOTH_MODES {
            assert_eq!(
                evaluate_gate(&graph, &approved_state, &node, mode),
                GateDecision::Proceed,
                "an attributed approval must be honoured in {mode} mode"
            );
            match evaluate_gate(&graph, &denied_state, &node, mode) {
                GateDecision::Refuse { reason, .. } => assert!(reason.contains("operator")),
                other => panic!("expected a refusal in {mode} mode, got {other:?}"),
            }
        }

        // A state carrying garbage under our namespace asks again rather than
        // decaying into a proceed.
        let mut junk = GraphState::for_graph(&graph);
        junk.extra
            .insert(RISK_BAG.to_string(), Value::from("not an object"));
        assert_eq!(resolution_of(&junk, &node), None);
        assert!(evaluate_gate(&graph, &junk, &node, ExecutionMode::Attended).requires_approval());

        // An approval for the cheap node, filed under the expensive one, does
        // not unlock it: the key is not trusted to identify the subject.
        let mut swapped = approved_state.clone();
        let mut forged = approved.clone();
        forged.record.node = NodeId::new("read_schema");
        swapped.extra[RISK_BAG][RESOLUTIONS_KEY][node.as_str()] =
            serde_json::to_value(forged).unwrap();
        assert!(
            evaluate_gate(&graph, &swapped, &node, ExecutionMode::Attended).requires_approval(),
            "consent given for one node must never unlock another"
        );
    }

    #[test]
    fn graph_written_before_risk_existed_loads_and_defaults_to_elevated() {
        // Exactly the document shape from `graph.rs`, with no risk key anywhere:
        // it must still parse, and the default must be Elevated (see the
        // `Default` impl for why that level and not Safe or Irreversible).
        let json = serde_json::json!({
            "nodes": [{"id": "legacy", "kind": "agent"}],
            "edges": [],
        });
        let graph: Graph = serde_json::from_value(json.clone()).expect("old document parses");
        assert_eq!(graph.validate(), Ok(()));

        let node = graph.node(&NodeId::new("legacy")).expect("node present");
        assert_eq!(risk_of(node), Ok(RiskLevel::Elevated));

        // No destructive migration: reading an old document through this module
        // adds NO risk key to it. The round trip is not byte for byte, and that
        // is `graph.rs`'s pre-existing behaviour rather than anything this file
        // does: serde re-emits the fields it defaulted (`retry`, `checks`,
        // `state`, `schema_version`). What matters here is that the default
        // lives in code and is never written back into the operator's document,
        // so classifying stays an explicit, additive act.
        let round_tripped = serde_json::to_value(&graph).expect("serializes");
        assert!(
            !round_tripped["nodes"][0]
                .as_object()
                .expect("node object")
                .contains_key(RISK_KEY),
            "an unclassified node must not be silently classified on load"
        );
        assert_eq!(round_tripped["nodes"][0]["id"], json["nodes"][0]["id"]);

        let state = GraphState::for_graph(&graph);
        let id = NodeId::new("legacy");
        assert_eq!(
            evaluate_gate(&graph, &state, &id, ExecutionMode::Attended),
            GateDecision::Proceed,
            "an attended run of an old graph must behave exactly as it did before"
        );
        assert!(
            evaluate_gate(&graph, &state, &id, ExecutionMode::Unattended).requires_approval(),
            "an unclassified node is not a safe node when nobody is watching"
        );
    }

    #[test]
    fn malformed_input_yields_a_typed_error_and_a_closed_gate() {
        // A value somebody wrote and got wrong, and a value of the wrong type.
        for bad in [Value::from("catastrophic"), Value::from(3), Value::Null] {
            let mut node = Node::new("bad", NodeKind::Agent);
            node.extra.insert(RISK_KEY.to_string(), bad.clone());
            let graph = Graph::new().with_node(node.clone());
            let state = GraphState::for_graph(&graph);

            match risk_of(&node) {
                Err(RiskError::MalformedRisk { node: id, .. }) => assert_eq!(id, "bad"),
                other => panic!("expected a typed error for {bad}, got {other:?}"),
            }

            for mode in BOTH_MODES {
                match evaluate_gate(&graph, &state, &NodeId::new("bad"), mode) {
                    GateDecision::Refuse { reason, .. } => {
                        assert!(reason.contains("unknown risk level"), "{reason}")
                    }
                    other => panic!("a malformed classification must fail closed, got {other:?}"),
                }
            }
        }

        // A node that is not in the graph at all is refused, not assumed safe.
        let graph = graph();
        let state = GraphState::for_graph(&graph);
        match evaluate_gate(
            &graph,
            &state,
            &NodeId::new("ghost"),
            ExecutionMode::Attended,
        ) {
            GateDecision::Refuse { reason, .. } => assert!(reason.contains("unknown node ghost")),
            other => panic!("expected a refusal, got {other:?}"),
        }

        // And a truncated resolution document decays to denied, never approved.
        let truncated: GateResolution =
            serde_json::from_value(serde_json::json!({})).expect("defaults apply");
        assert!(!truncated.is_approved());
    }

    #[test]
    fn risk_level_strings_are_stable() {
        for level in [
            RiskLevel::Safe,
            RiskLevel::Elevated,
            RiskLevel::Irreversible,
        ] {
            assert_eq!(RiskLevel::parse(level.as_str()), Some(level));
            assert_eq!(
                serde_json::to_value(level).expect("serializes"),
                Value::from(level.as_str())
            );
        }
        assert_eq!(RiskLevel::parse("  safe  "), Some(RiskLevel::Safe));
        assert_eq!(RiskLevel::parse("Irreversible"), None);
    }

    #[test]
    fn approval_is_bound_to_graph_run_and_reserved_generation() {
        let graph = graph();
        let node = NodeId::new("drop_table");
        let authority = authority();
        let mut state = GraphState::for_graph_with_authority(&graph, "run-subject-a", &authority);
        state.reserve(&node, &authority).unwrap();
        let record = evaluate_gate(&graph, &state, &node, ExecutionMode::Unattended)
            .into_escalation(stamp())
            .expect("held");
        let resolution = approve(&graph, &state, record, "operator", &authority).unwrap();
        record_resolution(&graph, &mut state, &resolution, &authority).unwrap();
        assert_eq!(
            evaluate_gate(&graph, &state, &node, ExecutionMode::Unattended),
            GateDecision::Proceed
        );

        let mut other_run =
            GraphState::for_graph_with_authority(&graph, "run-subject-b", &authority);
        other_run.reserve(&node, &authority).unwrap();
        assert!(matches!(
            record_resolution(&graph, &mut other_run, &resolution, &authority),
            Err(RiskError::InvalidApprovalAuthority { .. })
        ));
        assert!(
            evaluate_gate(&graph, &other_run, &node, ExecutionMode::Unattended).requires_approval()
        );

        state.reseed(&node);
        state.reserve(&node, &authority).unwrap();
        assert!(
            evaluate_gate(&graph, &state, &node, ExecutionMode::Unattended).requires_approval()
        );
    }

    #[test]
    fn graph_change_invalidates_prior_consent_even_when_node_id_is_unchanged() {
        let mut graph = graph();
        let node = NodeId::new("drop_table");
        let authority = authority();
        let mut state =
            GraphState::for_graph_with_authority(&graph, "run-graph-change", &authority);
        state.reserve(&node, &authority).unwrap();
        let record = evaluate_gate(&graph, &state, &node, ExecutionMode::Unattended)
            .into_escalation(stamp())
            .expect("held");
        let resolution = approve(&graph, &state, record, "operator", &authority).unwrap();
        record_resolution(&graph, &mut state, &resolution, &authority).unwrap();
        assert_eq!(
            evaluate_gate(&graph, &state, &node, ExecutionMode::Unattended),
            GateDecision::Proceed
        );

        graph
            .nodes
            .iter_mut()
            .find(|candidate| candidate.id == node)
            .unwrap()
            .extra
            .insert("command".to_string(), Value::from("drop table changed"));
        assert!(matches!(
            evaluate_gate(&graph, &state, &node, ExecutionMode::Unattended),
            GateDecision::Refuse { reason, .. } if reason.contains("digest mismatch")
        ));
    }

    #[test]
    fn expiring_approval_needs_an_explicit_clock_and_can_be_consumed_once() {
        let graph = graph();
        let node = NodeId::new("drop_table");
        let authority = authority();
        let mut state = GraphState::for_graph_with_authority(&graph, "run-expiry", &authority);
        state.reserve(&node, &authority).unwrap();
        let record = evaluate_gate(&graph, &state, &node, ExecutionMode::Unattended)
            .into_escalation(stamp())
            .expect("held");
        let expires_at = stamp() + chrono::Duration::seconds(30);
        let resolution =
            approve_until(&graph, &state, record, "operator", expires_at, &authority).unwrap();
        record_resolution(&graph, &mut state, &resolution, &authority).unwrap();

        assert!(
            evaluate_gate(&graph, &state, &node, ExecutionMode::Unattended).requires_approval()
        );
        assert_eq!(
            evaluate_gate_at(
                &graph,
                &state,
                &node,
                ExecutionMode::Unattended,
                stamp() + chrono::Duration::seconds(10),
            ),
            GateDecision::Proceed
        );
        assert!(
            evaluate_gate_at(&graph, &state, &node, ExecutionMode::Unattended, expires_at,)
                .requires_approval()
        );

        let decision = authorize_gate_at(
            &graph,
            &mut state,
            &node,
            ExecutionMode::Unattended,
            stamp() + chrono::Duration::seconds(10),
            &authority,
        );
        assert_eq!(decision, GateDecision::Proceed);
        assert_eq!(resolution_of(&state, &node), None);
        assert!(evaluate_gate_at(
            &graph,
            &state,
            &node,
            ExecutionMode::Unattended,
            stamp() + chrono::Duration::seconds(10),
        )
        .requires_approval());
    }

    fn inject_resolution(state: &mut GraphState, node: &NodeId, resolution: &GateResolution) {
        let bag = state
            .extra
            .entry(RISK_BAG.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        let bag = bag.as_object_mut().unwrap();
        let resolutions = bag
            .entry(RESOLUTIONS_KEY.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        resolutions
            .as_object_mut()
            .unwrap()
            .insert(node.0.clone(), serde_json::to_value(resolution).unwrap());
    }

    #[test]
    fn forged_and_legacy_unsigned_resolutions_never_authorize() {
        let graph = graph();
        let node = NodeId::new("drop_table");
        let authority = authority();
        let mut state =
            GraphState::for_graph_with_authority(&graph, "run-forged-resolution", &authority);
        state.reserve(&node, &authority).unwrap();
        let record = evaluate_gate(&graph, &state, &node, ExecutionMode::Unattended)
            .into_escalation(stamp())
            .unwrap();
        let signed = approve(&graph, &state, record, "operator", &authority).unwrap();

        let mut variants = Vec::new();
        let mut changed_approver = signed.clone();
        changed_approver.approver = "attacker".to_string();
        variants.push(changed_approver);
        let mut changed_verdict = signed.clone();
        changed_verdict.verdict = ResolutionVerdict::Denied;
        variants.push(changed_verdict);
        let mut changed_subject = signed.clone();
        changed_subject.record.what_is_lost = "nothing".to_string();
        variants.push(changed_subject);
        let mut changed_reservation = signed.clone();
        changed_reservation.reservation_id = "f".repeat(64);
        variants.push(changed_reservation);
        let mut changed_expiry = signed.clone();
        changed_expiry.expires_at = Some(stamp() + chrono::Duration::hours(1));
        variants.push(changed_expiry);
        let mut changed_mac = signed.clone();
        changed_mac.authority_mac = "0".repeat(64);
        variants.push(changed_mac);

        for forged in variants {
            let mut candidate = state.clone();
            inject_resolution(&mut candidate, &node, &forged);
            assert!(
                evaluate_gate(&graph, &candidate, &node, ExecutionMode::Unattended)
                    .requires_approval(),
                "editing any signed field must fail closed"
            );
            assert!(matches!(
                record_resolution(&graph, &mut state.clone(), &forged, &authority),
                Err(RiskError::InvalidApprovalAuthority { .. })
            ));
        }

        let legacy: GateResolution = serde_json::from_value(serde_json::json!({
            "record": signed.record,
            "verdict": "approved",
            "approver": "operator"
        }))
        .unwrap();
        let mut legacy_state = state.clone();
        inject_resolution(&mut legacy_state, &node, &legacy);
        assert!(
            evaluate_gate(&graph, &legacy_state, &node, ExecutionMode::Unattended)
                .requires_approval(),
            "legacy unsigned approvals deserialize but never authorize"
        );
    }

    #[test]
    fn consumed_resolution_cannot_be_replayed_for_the_same_reservation() {
        let graph = graph();
        let node = NodeId::new("drop_table");
        let authority = authority();
        let mut state =
            GraphState::for_graph_with_authority(&graph, "run-resolution-replay", &authority);
        state.reserve(&node, &authority).unwrap();
        let record = evaluate_gate(&graph, &state, &node, ExecutionMode::Unattended)
            .into_escalation(stamp())
            .unwrap();
        let resolution = approve(&graph, &state, record, "operator", &authority).unwrap();
        record_resolution(&graph, &mut state, &resolution, &authority).unwrap();
        assert_eq!(
            authorize_gate_at(
                &graph,
                &mut state,
                &node,
                ExecutionMode::Unattended,
                stamp() + chrono::Duration::seconds(1),
                &authority,
            ),
            GateDecision::Proceed
        );

        inject_resolution(&mut state, &node, &resolution);
        assert!(
            evaluate_gate_at(
                &graph,
                &state,
                &node,
                ExecutionMode::Unattended,
                stamp() + chrono::Duration::seconds(2),
            )
            .requires_approval(),
            "a signed one-shot approval is still invalid after raw reinsertion"
        );
        state.extra[RISK_BAG][CONSUMED_KEY][node.as_str()] = Value::from("tampered");
        assert!(
            evaluate_gate_at(
                &graph,
                &state,
                &node,
                ExecutionMode::Unattended,
                stamp() + chrono::Duration::seconds(2),
            )
            .requires_approval(),
            "corrupting the consumption history must fail closed"
        );
        assert!(matches!(
            record_resolution(&graph, &mut state, &resolution, &authority),
            Err(RiskError::InvalidApprovalAuthority { .. })
        ));
    }
}
