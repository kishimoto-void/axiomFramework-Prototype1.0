//! ACP — Immutable contract + seal (Prototype 1.0)

use axiom_capsule::Capsule;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Contract {
    pub text: String,
    pub hash_a: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SealedCapsule {
    pub capsule: Capsule,
    pub contract_hash_a: String,
    pub proof: String,
}

fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    format!("{:x}", h.finalize())
}

pub fn contract_from_text(text: &str) -> Contract {
    Contract {
        text: text.to_string(),
        hash_a: sha256_hex(text.as_bytes()),
    }
}

pub fn seal(contract: &Contract, capsule: Capsule) -> SealedCapsule {
    let mut material = Vec::new();
    material.extend_from_slice(b"axiom:v2:proof\0");
    material.extend_from_slice(contract.hash_a.as_bytes());
    material.push(0);
    material.extend_from_slice(capsule.raw_hash.as_bytes());
    material.push(0);
    material.extend_from_slice(capsule.canonical_hash.as_bytes());
    let proof = sha256_hex(&material);
    SealedCapsule {
        capsule,
        contract_hash_a: contract.hash_a.clone(),
        proof,
    }
}
