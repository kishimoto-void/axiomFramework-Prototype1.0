//! LRP (Language Runtime Protocol) v2.0.0-rfc-kernel
//!
//! AXIOM Framework 2.0 - Deterministic LLM Runtime Kernel
//! Full Protocol Implementation (RFC / Production Grade Single-File Kernel)
//!
//! Fixes applied for CI (2026-08-14):
//! - E0425: after_binding uses contract_a / contract_b (not contract_hash_a/b shorthand)
//! - E0621: required_ids: &'a [String] for OrdMap lifetime alignment

use im::{OrdMap, OrdSet, Vector};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

pub const VERSION: &str = "2.0.0-rfc-kernel";

// =============================================================================
// 1. ACP Type Integrations
// =============================================================================
pub mod acp {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct Coordinate {
        pub dimension_a: String,
        pub dimension_b: String,
        pub index: u64,
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
    pub struct CapsuleHash(pub String);

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
    pub struct ContractHash(pub String);

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
    pub struct RuntimeHash(pub String);

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct StateBinding {
        pub coordinate: Coordinate,
        pub capsule_hash: CapsuleHash,
        pub contract_hash_a: ContractHash,
        pub contract_hash_b: ContractHash,
    }
}

pub use acp::{CapsuleHash, ContractHash, Coordinate, RuntimeHash, StateBinding};

// =============================================================================
// 2. Structured Errors
// =============================================================================

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LrpError {
    #[error("Invalid state transition at step {step}: {reason}")]
    InvalidStateTransition { step: usize, reason: String },

    #[error("Replay index out of bounds: requested {requested}, available {available}")]
    ReplayOutOfBounds { requested: usize, available: usize },

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deterministic clock counter overflowed")]
    ClockOverflow,

    #[error("Clock non-monotonic: expected > {expected_greater_than}, got {actual}")]
    ClockNonMonotonic {
        expected_greater_than: u64,
        actual: u64,
    },

    #[error("Policy violations: {violations:?}")]
    PolicyViolation { violations: Vec<String> },

    #[error("Capability denied: {0}")]
    CapabilityDenied(String),

    #[error("Capability dependency error: {0}")]
    CapabilityDependencyError(String),

    #[error("Contract violation: {0}")]
    ContractViolation(String),

    #[error("Plugin error in '{plugin}': {message}")]
    PluginError { plugin: String, message: String },

    #[error("Chain corrupted at step {step}: {detail}")]
    ChainCorrupted { step: usize, detail: String },

    #[error("Snapshot verification failed at step {step}: {detail}")]
    SnapshotCorrupted { step: usize, detail: String },
}

// =============================================================================
// 3. Capability Dependency Resolver
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticCapability {
    pub capability_id: String,
    pub version: String,
    pub side_effect_free: bool,
    pub depends_on: Vector<String>,
    pub constraints: OrdMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCapability {
    pub capability_id: String,
    pub provider: String,
    pub is_volatile: bool,
    pub depends_on: Vector<String>,
    pub metadata: OrdMap<String, String>,
}

pub struct CapabilityResolver;

impl CapabilityResolver {
    pub fn validate_and_resolve<'a>(
        statics: impl IntoIterator<Item = &'a StaticCapability>,
        runtimes: impl IntoIterator<Item = &'a RuntimeCapability>,
        required_ids: &'a [String],
    ) -> Result<Vector<String>, LrpError> {
        let mut available_caps: OrdMap<&'a str, &'a Vector<String>> = OrdMap::new();

        for s in statics {
            available_caps.insert(&s.capability_id, &s.depends_on);
        }
        for r in runtimes {
            available_caps.insert(&r.capability_id, &r.depends_on);
        }

        let mut resolved_order = Vector::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        for req in required_ids {
            if !available_caps.contains_key(req.as_str()) {
                return Err(LrpError::CapabilityDenied(format!(
                    "Required capability missing: {}",
                    req
                )));
            }
            Self::resolve_dfs(
                req.as_str(),
                &available_caps,
                &mut visited,
                &mut visiting,
                &mut resolved_order,
            )?;
        }

        Ok(resolved_order)
    }

    fn resolve_dfs<'a>(
        current: &'a str,
        graph: &OrdMap<&'a str, &'a Vector<String>>,
        visited: &mut HashSet<&'a str>,
        visiting: &mut HashSet<&'a str>,
        order: &mut Vector<String>,
    ) -> Result<(), LrpError> {
        if visiting.contains(current) {
            return Err(LrpError::CapabilityDependencyError(format!(
                "Circular dependency detected at '{}'",
                current
            )));
        }
        if visited.contains(current) {
            return Ok(());
        }

        visiting.insert(current);

        if let Some(deps) = graph.get(current) {
            for dep in deps.iter() {
                if !graph.contains_key(dep.as_str()) {
                    return Err(LrpError::CapabilityDependencyError(format!(
                        "Unresolved dependency '{}' required by '{}'",
                        dep, current
                    )));
                }
                Self::resolve_dfs(dep.as_str(), graph, visited, visiting, order)?;
            }
        }

        visiting.remove(current);
        visited.insert(current);
        order.push_back(current.to_string());
        Ok(())
    }
}

// =============================================================================
// 4. Extended Protocol Contract, Reasoning Primitives & Intent
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contract {
    pub contract_id: String,
    pub protocol_version: String,
    pub schema_hash: ContractHash,
    pub compatibility: String,
    pub is_strict: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ReasoningPrimitive {
    Observe,
    Hypothesis,
    Inference,
    Validation,
    Commit,
    Fork,
    Merge,
    Rollback,
    Plan,
    Execute,
    Reflect,
    Suspend,
    Resume,
    Terminate,
}

impl ReasoningPrimitive {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Observe => "Observe",
            Self::Hypothesis => "Hypothesis",
            Self::Inference => "Inference",
            Self::Validation => "Validation",
            Self::Commit => "Commit",
            Self::Fork => "Fork",
            Self::Merge => "Merge",
            Self::Rollback => "Rollback",
            Self::Plan => "Plan",
            Self::Execute => "Execute",
            Self::Reflect => "Reflect",
            Self::Suspend => "Suspend",
            Self::Resume => "Resume",
            Self::Terminate => "Terminate",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord, Default)]
pub enum ReasoningIntent {
    Exploration,
    Verification,
    Planning,
    #[default]
    Execution,
    Reflection,
    Recovery,
}

// =============================================================================
// 5. Runtime Policy Evaluation
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PolicyEvaluation {
    pub allowed: bool,
    pub violations: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimePolicy {
    pub max_transitions: usize,
    pub max_evidence: usize,
    pub forbidden_primitives: OrdSet<ReasoningPrimitive>,
    pub forbidden_capabilities: OrdSet<String>,
    pub require_validation_before_commit: bool,
}

impl Default for RuntimePolicy {
    fn default() -> Self {
        Self {
            max_transitions: 1000,
            max_evidence: 500,
            forbidden_primitives: OrdSet::new(),
            forbidden_capabilities: OrdSet::new(),
            require_validation_before_commit: true,
        }
    }
}

impl RuntimePolicy {
    pub fn evaluate(
        &self,
        session: &ReasoningSession,
        primitive: &ReasoningPrimitive,
        required_caps: &[String],
    ) -> PolicyEvaluation {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        if session.transitions.len() >= self.max_transitions {
            violations.push(format!(
                "Max transitions limit reached: {}",
                self.max_transitions
            ));
        }

        if self.forbidden_primitives.contains(primitive) {
            violations.push(format!("Primitive {:?} is forbidden by policy", primitive));
        }

        for cap in required_caps {
            if self.forbidden_capabilities.contains(cap) {
                violations.push(format!("Capability '{}' is blocked by policy", cap));
            }
        }

        if *primitive == ReasoningPrimitive::Commit && self.require_validation_before_commit {
            let has_validation = session.transitions.iter().rev().any(|t| {
                t.primitive == ReasoningPrimitive::Validation && t.validation_passed
            });

            if !has_validation {
                violations.push("Commit rejected: Validation must pass before commit".into());
            }
        }

        if session.current_state().evidence.len() > (self.max_evidence * 8 / 10) {
            warnings.push("Evidence count approaching policy limit".into());
        }

        PolicyEvaluation {
            allowed: violations.is_empty(),
            violations,
            warnings,
        }
    }
}

// =============================================================================
// 6. Runtime Events & Severity
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventCategory {
    Lifecycle,
    Policy,
    Plugin,
    Snapshot,
    Replay,
    Security,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuntimeEventKind {
    SessionCreated,
    PolicyViolated {
        violations: Vec<String>,
    },
    CapabilityDenied {
        capability_id: String,
    },
    ReplayStarted {
        target_step: usize,
    },
    SnapshotCreated {
        step: usize,
        state_hash: CapsuleHash,
        runtime_hash: RuntimeHash,
    },
    TransitionExecuted {
        transition_id: String,
        primitive: String,
        intent: ReasoningIntent,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeEvent {
    pub event_id: String,
    pub severity: EventSeverity,
    pub category: EventCategory,
    pub session_id: String,
    pub step: usize,
    pub kind: RuntimeEventKind,
    pub logical_timestamp: u64,
}

// =============================================================================
// 7. Canonical State & IEEE 754 Bit-exact StateHasher
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceKind {
    Fact,
    Inference,
    Assumption,
    Observed,
    ToolResult,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    pub evidence_id: String,
    pub kind: EvidenceKind,
    pub payload: serde_json::Value,
    pub confidence: f64,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextNode {
    pub node_id: String,
    pub kind: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ContextGraph {
    pub nodes: Vector<ContextNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    pub candidate_id: String,
    pub description: String,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeltaAction {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeltaKind {
    ContextChange,
    EvidenceChange,
    CandidateChange,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypedPayload {
    Context(ContextNode),
    Evidence(Evidence),
    Candidate(Candidate),
    Raw(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PLPDelta {
    pub action: DeltaAction,
    pub kind: DeltaKind,
    pub payload: TypedPayload,
    pub target_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningState {
    pub state_id: String,
    pub context: ContextGraph,
    pub evidence: Vector<Evidence>,
    pub candidates: Vector<Candidate>,
    pub step_count: usize,
}

pub struct StateHasher;

impl StateHasher {
    pub fn compute_hash(state: &ReasoningState) -> Result<CapsuleHash, LrpError> {
        let value = serde_json::to_value(state).map_err(|e| {
            LrpError::SerializationError(format!("State to_value failed: {}", e))
        })?;

        let canonical = Self::canonicalize(value);

        let bytes = serde_json::to_vec(&canonical).map_err(|e| {
            LrpError::SerializationError(format!("Canonical state serialization failed: {}", e))
        })?;

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        Ok(CapsuleHash(format!("{:x}", hasher.finalize())))
    }

    fn canonicalize(value: serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                let mut sorted = serde_json::Map::new();
                let mut keys: Vec<_> = map.keys().cloned().collect();
                keys.sort();
                for k in keys {
                    if let Some(v) = map.get(&k) {
                        sorted.insert(k, Self::canonicalize(v.clone()));
                    }
                }
                serde_json::Value::Object(sorted)
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.into_iter().map(Self::canonicalize).collect())
            }
            serde_json::Value::Number(num) => {
                if let Some(f) = num.as_f64() {
                    serde_json::Value::String(format!("f64_{:016x}", f.to_bits()))
                } else {
                    serde_json::Value::Number(num)
                }
            }
            other => other,
        }
    }
}

// =============================================================================
// 8. Merkle Chained Transition Hashes (Version-Aware)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningTransition {
    pub transition_id: String,
    pub primitive: ReasoningPrimitive,
    pub intent: ReasoningIntent,
    pub previous_runtime_hash: RuntimeHash,
    pub current_runtime_hash: RuntimeHash,
    pub before_binding: StateBinding,
    pub after_binding: StateBinding,
    pub operation: String,
    pub input_contract: Option<Contract>,
    pub output_contract: Option<Contract>,
    pub deltas: Vector<PLPDelta>,
    pub validation_passed: bool,
    pub logical_timestamp: u64,
}

impl ReasoningTransition {
    pub fn compute_chain_hash(
        prev_hash: &RuntimeHash,
        transition_id: &str,
        after_capsule_hash: &CapsuleHash,
        logical_ts: u64,
        version: &str,
    ) -> RuntimeHash {
        let mut hasher = Sha256::new();
        hasher.update(version.as_bytes());
        hasher.update(b"|");
        hasher.update(prev_hash.0.as_bytes());
        hasher.update(b"|");
        hasher.update(transition_id.as_bytes());
        hasher.update(b"|");
        hasher.update(after_capsule_hash.0.as_bytes());
        hasher.update(b"|");
        hasher.update(logical_ts.to_le_bytes());
        RuntimeHash(format!("{:x}", hasher.finalize()))
    }
}

// =============================================================================
// 9. Transition Builder Pattern (&self reusable)
// =============================================================================

#[derive(Debug, Clone)]
pub struct TransitionBuilder {
    primitive: ReasoningPrimitive,
    intent: ReasoningIntent,
    operation: String,
    required_capabilities: Vec<String>,
    input_contract: Option<Contract>,
    output_contract: Option<Contract>,
    deltas: Vector<PLPDelta>,
    validation_passed: bool,
}

impl TransitionBuilder {
    pub fn new(primitive: ReasoningPrimitive, operation: impl Into<String>) -> Self {
        Self {
            primitive,
            intent: ReasoningIntent::default(),
            operation: operation.into(),
            required_capabilities: Vec::new(),
            input_contract: None,
            output_contract: None,
            deltas: Vector::new(),
            validation_passed: true,
        }
    }

    pub fn intent(mut self, intent: ReasoningIntent) -> Self {
        self.intent = intent;
        self
    }

    pub fn require_capability(mut self, cap_id: impl Into<String>) -> Self {
        self.required_capabilities.push(cap_id.into());
        self
    }

    pub fn contracts(mut self, input: Option<Contract>, output: Option<Contract>) -> Self {
        self.input_contract = input;
        self.output_contract = output;
        self
    }

    pub fn delta(mut self, delta: PLPDelta) -> Self {
        self.deltas.push_back(delta);
        self
    }

    pub fn deltas(mut self, deltas: impl IntoIterator<Item = PLPDelta>) -> Self {
        for d in deltas {
            self.deltas.push_back(d);
        }
        self
    }

    pub fn validation_passed(mut self, passed: bool) -> Self {
        self.validation_passed = passed;
        self
    }

    pub fn execute(
        &self,
        runtime: &LRPKernelRuntime,
        session: &mut ReasoningSession,
    ) -> Result<(), LrpError> {
        runtime.execute_transition(
            session,
            self.primitive.clone(),
            self.intent.clone(),
            &self.operation,
            self.required_capabilities.clone(),
            self.input_contract.clone(),
            self.output_contract.clone(),
            self.deltas.clone(),
            self.validation_passed,
        )
    }
}

// =============================================================================
// 10. Snapshot Strategy & Mode
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SnapshotMode {
    Full,
    Delta,
    Compressed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub step_count: usize,
    pub mode: SnapshotMode,
    pub state: ReasoningState,
    pub transition_id: String,
    pub state_hash: CapsuleHash,
    pub runtime_hash: RuntimeHash,
}

// =============================================================================
// 11. Plugin System & Policy Framework
// =============================================================================

#[derive(Debug, Clone)]
pub struct PluginExecutionPolicy {
    pub timeout: Duration,
    pub allow_panic_recovery: bool,
}

impl Default for PluginExecutionPolicy {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            allow_panic_recovery: true,
        }
    }
}

pub trait ValidatorPlugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn validate_transition(
        &self,
        session: &ReasoningSession,
        primitive: &ReasoningPrimitive,
    ) -> Result<(), LrpError>;
}

pub trait ObserverPlugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn on_event(&self, session: &ReasoningSession, event: &RuntimeEvent);
}

pub struct PluginManager {
    validators: Vec<Arc<dyn ValidatorPlugin>>,
    observers: Vec<Arc<dyn ObserverPlugin>>,
    pub execution_policy: PluginExecutionPolicy,
}

impl PluginManager {
    pub fn new(
        validators: Vec<Arc<dyn ValidatorPlugin>>,
        observers: Vec<Arc<dyn ObserverPlugin>>,
        execution_policy: Option<PluginExecutionPolicy>,
    ) -> Self {
        Self {
            validators,
            observers,
            execution_policy: execution_policy.unwrap_or_default(),
        }
    }

    pub fn validate(
        &self,
        session: &ReasoningSession,
        primitive: &ReasoningPrimitive,
    ) -> Result<(), LrpError> {
        for v in &self.validators {
            let res = catch_unwind(AssertUnwindSafe(|| v.validate_transition(session, primitive)));
            match res {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    return Err(LrpError::PluginError {
                        plugin: v.name().to_string(),
                        message: e.to_string(),
                    })
                }
                Err(_) => {
                    return Err(LrpError::PluginError {
                        plugin: v.name().to_string(),
                        message: "Plugin panicked during validation".into(),
                    })
                }
            }
        }
        Ok(())
    }

    pub fn notify_observers(&self, session: &ReasoningSession, event: &RuntimeEvent) {
        for obs in &self.observers {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                obs.on_event(session, event);
            }));
        }
    }
}

// =============================================================================
// 12. ReasoningSession & Chain Verification
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningSession {
    pub session_id: String,
    pub problem_id: String,
    pub initial_state: ReasoningState,
    pub current_cache: ReasoningState,
    pub transitions: Vector<ReasoningTransition>,
    pub snapshots: OrdMap<usize, SessionSnapshot>,
    pub events: Vector<RuntimeEvent>,
    pub static_capabilities: Vector<StaticCapability>,
    pub runtime_capabilities: Vector<RuntimeCapability>,
    pub policy: RuntimePolicy,
    pub seed: usize,
    pub snapshot_interval: usize,
    pub last_logical_timestamp: u64,
    pub version: String,
}

impl ReasoningSession {
    pub fn current_state(&self) -> &ReasoningState {
        &self.current_cache
    }

    pub fn latest_runtime_hash(&self) -> RuntimeHash {
        self.transitions
            .last()
            .map(|t| t.current_runtime_hash.clone())
            .unwrap_or_else(|| RuntimeHash(format!("genesis_{}", self.seed)))
    }

    pub fn add_event(
        &mut self,
        severity: EventSeverity,
        category: EventCategory,
        kind: RuntimeEventKind,
        step: usize,
        timestamp: u64,
    ) -> RuntimeEvent {
        let event_id = format!("evt_{}_{}", step, self.events.len());
        let evt = RuntimeEvent {
            event_id,
            severity,
            category,
            session_id: self.session_id.clone(),
            step,
            kind,
            logical_timestamp: timestamp,
        };
        self.events.push_back(evt.clone());
        evt
    }

    pub fn verify_chain(&self) -> Result<(), LrpError> {
        let mut prev_hash = RuntimeHash(format!("genesis_{}", self.seed));
        let mut current_state = self.initial_state.clone();

        for (i, tr) in self.transitions.iter().enumerate() {
            if tr.previous_runtime_hash != prev_hash {
                return Err(LrpError::ChainCorrupted {
                    step: i,
                    detail: "Previous runtime hash mismatch".into(),
                });
            }

            current_state = ReplayEngine::apply_deltas(&current_state, &tr.deltas)?;
            let computed_state_hash = StateHasher::compute_hash(&current_state)?;

            if computed_state_hash != tr.after_binding.capsule_hash {
                return Err(LrpError::ChainCorrupted {
                    step: i,
                    detail: format!(
                        "Capsule state hash mismatch. Expected {:?}, Got {:?}",
                        tr.after_binding.capsule_hash, computed_state_hash
                    ),
                });
            }

            let computed_runtime_hash = ReasoningTransition::compute_chain_hash(
                &prev_hash,
                &tr.transition_id,
                &computed_state_hash,
                tr.logical_timestamp,
                &self.version,
            );

            if computed_runtime_hash != tr.current_runtime_hash {
                return Err(LrpError::ChainCorrupted {
                    step: i,
                    detail: "Runtime hash mismatch in Merkle chain".into(),
                });
            }

            prev_hash = tr.current_runtime_hash.clone();
        }

        for (step, snap) in &self.snapshots {
            if *step == 0 || *step > self.transitions.len() {
                return Err(LrpError::SnapshotCorrupted {
                    step: *step,
                    detail: "Invalid snapshot step index".into(),
                });
            }

            let tr = &self.transitions[*step - 1];

            if snap.runtime_hash != tr.current_runtime_hash {
                return Err(LrpError::SnapshotCorrupted {
                    step: *step,
                    detail: "Runtime hash mismatch with transition record".into(),
                });
            }

            if snap.state_hash != tr.after_binding.capsule_hash {
                return Err(LrpError::SnapshotCorrupted {
                    step: *step,
                    detail: "State hash mismatch with capsule binding".into(),
                });
            }

            let recomputed = StateHasher::compute_hash(&snap.state)?;
            if recomputed != snap.state_hash {
                return Err(LrpError::SnapshotCorrupted {
                    step: *step,
                    detail: "Stored state does not match its computed hash".into(),
                });
            }
        }

        Ok(())
    }
}

// =============================================================================
// 13. Replay Engine & Iterator
// =============================================================================

pub struct ReplayEngine;

impl ReplayEngine {
    pub fn stream<'a>(session: &'a ReasoningSession) -> ReplayIterator<'a> {
        ReplayIterator {
            session,
            current_step: 0,
            current_state: session.initial_state.clone(),
        }
    }

    pub fn apply_deltas(
        current: &ReasoningState,
        deltas: &Vector<PLPDelta>,
    ) -> Result<ReasoningState, LrpError> {
        let mut next = current.clone();

        for delta in deltas.iter() {
            match (&delta.action, &delta.kind, &delta.payload) {
                (
                    DeltaAction::Added | DeltaAction::Modified,
                    DeltaKind::ContextChange,
                    TypedPayload::Context(node),
                ) => {
                    if let Some(pos) = next.context.nodes.iter().position(|n| n.node_id == node.node_id) {
                        next.context.nodes.set(pos, node.clone());
                    } else {
                        next.context.nodes.push_back(node.clone());
                    }
                }
                (DeltaAction::Removed, DeltaKind::ContextChange, _) => {
                    if let Some(target_id) = &delta.target_id {
                        next.context.nodes.retain(|n| &n.node_id != target_id);
                    }
                }
                (DeltaAction::Added, DeltaKind::EvidenceChange, TypedPayload::Evidence(ev)) => {
                    if !next.evidence.iter().any(|e| e.evidence_id == ev.evidence_id) {
                        next.evidence.push_back(ev.clone());
                    }
                }
                (DeltaAction::Removed, DeltaKind::EvidenceChange, _) => {
                    if let Some(target_id) = &delta.target_id {
                        next.evidence.retain(|e| &e.evidence_id != target_id);
                    }
                }
                (DeltaAction::Added, DeltaKind::CandidateChange, TypedPayload::Candidate(cand)) => {
                    if !next
                        .candidates
                        .iter()
                        .any(|c| c.candidate_id == cand.candidate_id)
                    {
                        next.candidates.push_back(cand.clone());
                    }
                }
                (DeltaAction::Removed, DeltaKind::CandidateChange, _) => {
                    if let Some(target_id) = &delta.target_id {
                        next.candidates.retain(|c| &c.candidate_id != target_id);
                    }
                }
                _ => {}
            }
        }

        next.step_count += 1;
        Ok(next)
    }
}

pub struct ReplayIterator<'a> {
    session: &'a ReasoningSession,
    current_step: usize,
    current_state: ReasoningState,
}

impl<'a> Iterator for ReplayIterator<'a> {
    type Item = (usize, &'a ReasoningTransition, ReasoningState);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_step >= self.session.transitions.len() {
            return None;
        }

        let tr = &self.session.transitions[self.current_step];
        let next_state = ReplayEngine::apply_deltas(&self.current_state, &tr.deltas).ok()?;
        self.current_state = next_state.clone();
        self.current_step += 1;

        Some((self.current_step, tr, next_state))
    }
}

// =============================================================================
// 14. High-performance Lock-free Clock
// =============================================================================

pub struct DeterministicClock {
    counter: Arc<AtomicU64>,
}

impl DeterministicClock {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn tick(&self) -> Result<u64, LrpError> {
        let prev = self.counter.fetch_add(1, Ordering::Relaxed);
        if prev == u64::MAX {
            Err(LrpError::ClockOverflow)
        } else {
            Ok(prev + 1)
        }
    }
}

impl Default for DeterministicClock {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// 15. LRP Kernel Runtime
// =============================================================================

pub struct LRPKernelRuntime {
    pub seed: usize,
    pub clock: DeterministicClock,
    pub plugin_manager: PluginManager,
}

impl LRPKernelRuntime {
    pub fn new(
        seed: usize,
        validators: Vec<Arc<dyn ValidatorPlugin>>,
        observers: Vec<Arc<dyn ObserverPlugin>>,
    ) -> Self {
        Self {
            seed,
            clock: DeterministicClock::new(),
            plugin_manager: PluginManager::new(validators, observers, None),
        }
    }

    pub fn create_session(
        &self,
        problem_id: &str,
        static_capabilities: Vector<StaticCapability>,
        runtime_capabilities: Vector<RuntimeCapability>,
        policy: Option<RuntimePolicy>,
        snapshot_interval: usize,
    ) -> Result<ReasoningSession, LrpError> {
        let initial_state = ReasoningState {
            state_id: format!("init_{}", self.seed),
            context: ContextGraph::default(),
            evidence: Vector::new(),
            candidates: Vector::new(),
            step_count: 0,
        };

        let mut session = ReasoningSession {
            session_id: format!("sess_{}", self.seed),
            problem_id: problem_id.to_string(),
            initial_state: initial_state.clone(),
            current_cache: initial_state,
            transitions: Vector::new(),
            snapshots: OrdMap::new(),
            events: Vector::new(),
            static_capabilities,
            runtime_capabilities,
            policy: policy.unwrap_or_default(),
            seed: self.seed,
            snapshot_interval: if snapshot_interval == 0 { 100 } else { snapshot_interval },
            last_logical_timestamp: 0,
            version: VERSION.to_string(),
        };

        let ts = self.clock.tick()?;
        session.last_logical_timestamp = ts;
        let evt = session.add_event(
            EventSeverity::Info,
            EventCategory::Lifecycle,
            RuntimeEventKind::SessionCreated,
            0,
            ts,
        );
        self.plugin_manager.notify_observers(&session, &evt);

        Ok(session)
    }

    pub fn builder(
        &self,
        primitive: ReasoningPrimitive,
        operation: impl Into<String>,
    ) -> TransitionBuilder {
        TransitionBuilder::new(primitive, operation)
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_transition(
        &self,
        session: &mut ReasoningSession,
        primitive: ReasoningPrimitive,
        intent: ReasoningIntent,
        operation: &str,
        required_capabilities: Vec<String>,
        input_contract: Option<Contract>,
        output_contract: Option<Contract>,
        deltas: Vector<PLPDelta>,
        validation_passed: bool,
    ) -> Result<(), LrpError> {
        let ts = self.clock.tick()?;

        if ts <= session.last_logical_timestamp {
            return Err(LrpError::ClockNonMonotonic {
                expected_greater_than: session.last_logical_timestamp,
                actual: ts,
            });
        }

        CapabilityResolver::validate_and_resolve(
            &session.static_capabilities,
            &session.runtime_capabilities,
            &required_capabilities,
        )?;

        let eval = session
            .policy
            .evaluate(session, &primitive, &required_capabilities);
        if !eval.allowed {
            let evt = session.add_event(
                EventSeverity::Error,
                EventCategory::Policy,
                RuntimeEventKind::PolicyViolated {
                    violations: eval.violations.clone(),
                },
                session.transitions.len(),
                ts,
            );
            self.plugin_manager.notify_observers(session, &evt);
            return Err(LrpError::PolicyViolation {
                violations: eval.violations,
            });
        }

        self.plugin_manager.validate(session, &primitive)?;

        let prev_state = session.current_state().clone();
        let prev_state_hash = StateHasher::compute_hash(&prev_state)?;
        let prev_runtime_hash = session.latest_runtime_hash();
        let step = session.transitions.len();

        let contract_a = input_contract
            .as_ref()
            .map(|c| c.schema_hash.clone())
            .unwrap_or_default();
        let contract_b = output_contract
            .as_ref()
            .map(|c| c.schema_hash.clone())
            .unwrap_or_default();

        let before_binding = StateBinding {
            coordinate: Coordinate {
                dimension_a: "reasoning".into(),
                dimension_b: "step".into(),
                index: step as u64,
            },
            capsule_hash: prev_state_hash,
            contract_hash_a: contract_a.clone(),
            contract_hash_b: contract_b.clone(),
        };

        let next_state = ReplayEngine::apply_deltas(&prev_state, &deltas)?;
        let next_state_hash = StateHasher::compute_hash(&next_state)?;

        let tr_id = format!("tr_{}_{}", session.seed, step + 1);

        let current_runtime_hash = ReasoningTransition::compute_chain_hash(
            &prev_runtime_hash,
            &tr_id,
            &next_state_hash,
            ts,
            &session.version,
        );

        let after_binding = StateBinding {
            coordinate: Coordinate {
                dimension_a: "reasoning".into(),
                dimension_b: "step".into(),
                index: (step + 1) as u64,
            },
            capsule_hash: next_state_hash.clone(),
            contract_hash_a: contract_a,
            contract_hash_b: contract_b,
        };

        let transition = ReasoningTransition {
            transition_id: tr_id.clone(),
            primitive: primitive.clone(),
            intent: intent.clone(),
            previous_runtime_hash: prev_runtime_hash,
            current_runtime_hash: current_runtime_hash.clone(),
            before_binding,
            after_binding,
            operation: operation.to_string(),
            input_contract,
            output_contract,
            deltas,
            validation_passed,
            logical_timestamp: ts,
        };

        session.transitions.push_back(transition.clone());
        session.current_cache = next_state.clone();
        session.last_logical_timestamp = ts;

        let evt = session.add_event(
            EventSeverity::Info,
            EventCategory::Lifecycle,
            RuntimeEventKind::TransitionExecuted {
                transition_id: tr_id,
                primitive: primitive.as_str().to_string(),
                intent,
            },
            session.transitions.len(),
            ts,
        );
        self.plugin_manager.notify_observers(session, &evt);

        if session.transitions.len().is_multiple_of(session.snapshot_interval) {
            let snap = SessionSnapshot {
                step_count: session.transitions.len(),
                mode: SnapshotMode::Full,
                state: next_state,
                transition_id: transition.transition_id.clone(),
                state_hash: next_state_hash.clone(),
                runtime_hash: current_runtime_hash.clone(),
            };
            session.snapshots.insert(snap.step_count, snap.clone());

            let snap_evt = session.add_event(
                EventSeverity::Info,
                EventCategory::Snapshot,
                RuntimeEventKind::SnapshotCreated {
                    step: snap.step_count,
                    state_hash: snap.state_hash,
                    runtime_hash: snap.runtime_hash,
                },
                session.transitions.len(),
                ts,
            );
            self.plugin_manager.notify_observers(session, &snap_evt);
        }

        Ok(())
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ieee754_bit_exact_hash_determinism() {
        let mut state_a = ReasoningState {
            state_id: "state_1".into(),
            context: ContextGraph::default(),
            evidence: Vector::new(),
            candidates: Vector::new(),
            step_count: 1,
        };

        state_a.candidates.push_back(Candidate {
            candidate_id: "c1".into(),
            description: "cand 1".into(),
            score: 0.8500000001,
        });

        let mut state_b = state_a.clone();
        state_b.candidates[0].score = 0.8500000001;

        let hash_a = StateHasher::compute_hash(&state_a).unwrap();
        let hash_b = StateHasher::compute_hash(&state_b).unwrap();

        assert_eq!(hash_a, hash_b);
        assert_eq!(hash_a.0.len(), 64);
    }

    #[test]
    fn test_builder_reusability_and_reasoning_intent() {
        let runtime = LRPKernelRuntime::new(200, vec![], vec![]);
        let mut session = runtime
            .create_session("intent_test", Vector::new(), Vector::new(), None, 1)
            .unwrap();

        let template_builder = runtime
            .builder(ReasoningPrimitive::Observe, "Batch Observation")
            .intent(ReasoningIntent::Exploration);

        template_builder.execute(&runtime, &mut session).unwrap();
        template_builder.execute(&runtime, &mut session).unwrap();

        assert_eq!(session.transitions.len(), 2);
        assert_eq!(session.transitions[0].intent, ReasoningIntent::Exploration);
        assert_eq!(session.transitions[1].intent, ReasoningIntent::Exploration);
    }

    #[test]
    fn test_full_chain_verification_and_replay() {
        let runtime = LRPKernelRuntime::new(300, vec![], vec![]);
        let mut session = runtime
            .create_session("full_verification", Vector::new(), Vector::new(), None, 2)
            .unwrap();

        runtime
            .builder(ReasoningPrimitive::Plan, "Define Strategy")
            .intent(ReasoningIntent::Planning)
            .execute(&runtime, &mut session)
            .unwrap();

        runtime
            .builder(ReasoningPrimitive::Execute, "Run Action")
            .intent(ReasoningIntent::Execution)
            .execute(&runtime, &mut session)
            .unwrap();

        assert!(session.verify_chain().is_ok());

        let replayed_steps: Vec<_> = ReplayEngine::stream(&session).collect();
        assert_eq!(replayed_steps.len(), 2);
    }

    #[test]
    fn version_present() {
        assert!(VERSION.starts_with("2.0.0"));
    }
}
