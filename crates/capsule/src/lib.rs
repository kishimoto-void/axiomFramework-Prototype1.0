//! Capsule — A/B state storage (Prototype 1.0)
//!
//! A = Raw + raw_hash (immutable layer)
//! B = Canonical + canonical_hash (projected layer)

use axiom_plp::{CanonicalState, Projection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capsule {
    pub id: String,
    pub raw_text: String,
    pub raw_hash: String,
    pub canonical: CanonicalState,
    pub canonical_hash: String,
}

impl Capsule {
    pub fn from_projection(id: impl Into<String>, p: Projection) -> Self {
        Self {
            id: id.into(),
            raw_text: p.raw_text,
            raw_hash: p.raw_hash,
            canonical: p.canonical,
            canonical_hash: p.canonical_hash,
        }
    }

    pub fn hash_a(&self) -> &str {
        &self.raw_hash
    }

    pub fn hash_b(&self) -> &str {
        &self.canonical_hash
    }
}
