//! PLP — State Projection (Prototype 1.0) + PLP-R research contracts
//!
//! Incorporates **PLP-R** from Axiom-Framework research line:
//! - PLP does **not** parse meaning
//! - Annotation = Canonical Projection Candidate (not Semantic Truth)
//! - Dual Hash: raw_hash (HashA) / canonical_hash (HashB)
//! - TokenOnlyProjector (baseline) + MinimalProjector (demo)
//! - Deterministic canonical serialization for Golden locks
//! - DifferenceMetrics (Canonical-only) + Monitor (Continue / AskUser / Abort)
//!
//! Payload version (hash-relevant): `0.1.1`
//! Research package version: `0.1.2`
//!
//! 実験は忠実に実際行って

mod hash_ser;
mod project;
mod diff;
mod monitor;

pub use hash_ser::{build_canonical_payload, dual_hash, sha256_hex};
pub use project::{
    project_minimal, project_text_minimal, project_text_token_only, project_token_only,
    ProjectOptions,
};
pub use diff::{diff_canonical, diff_projections, DifferenceMetrics};
pub use monitor::{
    monitor_decide, monitor_decide_default, MonitorDecision, MonitorDecisionKind,
};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Payload version frozen for Golden vectors (≠ crate version).
pub const PAYLOAD_VERSION: &str = "0.1.1";
pub const PROTOCOL: &str = "PLP-R/0.1";
/// Research package version (docs / demos).
pub const RESEARCH_VERSION: &str = "0.1.2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AnnotationKind {
    Entity,
    Action,
    Location,
    Relation,
    Attribute,
    Constraint,
}

impl AnnotationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Entity => "ENTITY",
            Self::Action => "ACTION",
            Self::Location => "LOCATION",
            Self::Relation => "RELATION",
            Self::Attribute => "ATTRIBUTE",
            Self::Constraint => "CONSTRAINT",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Annotation {
    pub kind: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

impl Annotation {
    pub fn new(kind: AnnotationKind, value: impl Into<String>) -> Self {
        Self {
            kind: kind.as_str().into(),
            value: value.into(),
            key: None,
        }
    }

    pub fn with_key(
        kind: AnnotationKind,
        value: impl Into<String>,
        key: impl Into<String>,
    ) -> Self {
        Self {
            kind: kind.as_str().into(),
            value: value.into(),
            key: Some(key.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalState {
    pub version: String,
    pub language: String,
    pub tokens: Vec<String>,
    pub annotations: Vec<Annotation>,
    pub meta: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectionHeader {
    pub protocol: String,
    pub version: String,
    pub capsule_id: String,
    pub parent_id: Option<String>,
    pub clock: u64,
    pub sequence: u64,
    pub timestamp_ns: u64,
    pub source: String,
    pub hash_algorithm: String,
}

impl Default for ProjectionHeader {
    fn default() -> Self {
        Self {
            protocol: PROTOCOL.into(),
            version: PAYLOAD_VERSION.into(),
            capsule_id: String::new(),
            parent_id: None,
            clock: 0,
            sequence: 0,
            timestamp_ns: 0,
            source: "plp-r".into(),
            hash_algorithm: "sha256".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Projection {
    pub header: ProjectionHeader,
    pub raw_text: String,
    pub canonical: CanonicalState,
    pub raw_hash: String,
    pub canonical_hash: String,
}

impl Projection {
    pub fn hash_a(&self) -> &str {
        &self.raw_hash
    }
    pub fn hash_b(&self) -> &str {
        &self.canonical_hash
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PlpError {
    #[error("empty input")]
    EmptyInput,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axiom_pss::normalize;

    #[test]
    fn token_only_deterministic() {
        let n = normalize("Hello World").unwrap();
        let opts = ProjectOptions::with_id("t1");
        let a = project_token_only(&n, opts.clone()).unwrap();
        let b = project_token_only(&n, opts).unwrap();
        assert_eq!(a.raw_hash, b.raw_hash);
        assert_eq!(a.canonical_hash, b.canonical_hash);
        assert!(a.canonical.annotations.is_empty());
        assert_eq!(
            a.canonical.meta.get("annotation_status").map(|s| s.as_str()),
            Some("none")
        );
        assert_eq!(a.canonical.version, PAYLOAD_VERSION);
    }

    #[test]
    fn token_only_en_tokens_lowercased() {
        let n = normalize("Enable Review").unwrap();
        let p = project_token_only(&n, ProjectOptions::with_id("x")).unwrap();
        assert_eq!(p.canonical.tokens, vec!["enable", "review"]);
        assert_eq!(p.canonical.language, "en");
    }

    #[test]
    fn ja_language_and_entity_candidate() {
        let n = normalize("猫が机の上で寝ている").unwrap();
        let p = project_minimal(&n, ProjectOptions::with_id("ja1")).unwrap();
        assert_eq!(p.canonical.language, "ja");
        assert!(!p.canonical.tokens.is_empty());
        assert_eq!(
            p.canonical.meta.get("annotation_status").map(|s| s.as_str()),
            Some("canonical_projection_candidate")
        );
    }

    #[test]
    fn minimal_action_candidate() {
        let n = normalize("Enable review bot").unwrap();
        let p = project_minimal(&n, ProjectOptions::with_id("m1")).unwrap();
        assert!(
            p.canonical
                .annotations
                .iter()
                .any(|a| a.kind == "ACTION" && a.value == "enable"),
            "expected ACTION enable, got {:?}",
            p.canonical.annotations
        );
    }

    #[test]
    fn different_capsule_id_same_raw_semantic_class() {
        let n = normalize("same text").unwrap();
        let a = project_token_only(&n, ProjectOptions::with_id("a")).unwrap();
        let b = project_token_only(&n, ProjectOptions::with_id("b")).unwrap();
        assert_eq!(a.raw_hash, b.raw_hash);
        assert_ne!(a.canonical_hash, b.canonical_hash);
    }

    #[test]
    fn empty_errors() {
        let n = axiom_pss::NormalizedInput {
            text: "  ".into(),
            language_hint: Some("en".into()),
            encoding: "utf-8".into(),
        };
        assert_eq!(
            project_token_only(&n, ProjectOptions::default()).unwrap_err(),
            PlpError::EmptyInput
        );
    }

    #[test]
    fn one_shot_helpers() {
        let p = project_text_token_only("hello", ProjectOptions::with_id("z")).unwrap();
        assert_eq!(p.hash_a().len(), 64);
        assert_eq!(p.hash_b().len(), 64);
    }

    #[test]
    fn monitor_on_projection_diff() {
        let n = normalize("Enable review bot").unwrap();
        let a = project_minimal(&n, ProjectOptions::with_id("a")).unwrap();
        let b = project_token_only(&n, ProjectOptions::with_id("a")).unwrap();
        let m = diff_projections(&a, &b);
        let d = monitor_decide_default(&m, true);
        if m.divergence > 0.0 {
            assert_eq!(d.kind, MonitorDecisionKind::AskUser);
        }
    }
}
