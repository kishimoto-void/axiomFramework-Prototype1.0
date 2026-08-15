//! AXIOM Common Protocol (ACP) — Production Reference Implementation v1.2.0
//!
//! Specification Version: 1.2.0
//!
//! Features:
//! - Zero-allocation stream JCS hashing (RFC 8785)
//! - Domain separation tags (const)
//! - BTreeMap for deterministic key order
//! - `thiserror` structured errors
//! - `time` crate RFC 3339 timestamps
//! - Prototype seal API retained for Phase 1–2 pipeline compatibility
//!
//! License: AXIOM Framework Research License v1.0

use std::collections::BTreeMap;
#[cfg(feature = "std")]
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

// ---------------------------------------------------------------------------
// Prototype 1.0 compatibility (PSS→PLP→Capsule→ACP→DCK pipeline)
// ---------------------------------------------------------------------------

use axiom_capsule::Capsule;

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
    bytes_to_hex(&h.finalize())
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

// ---------------------------------------------------------------------------
// Protocol Constants & Domain Tags
// ---------------------------------------------------------------------------

pub const AXIOM_PROTOCOL_NAME: &str = "AXIOM";
pub const AXIOM_PROTOCOL_ID: &str = "acp";
pub const AXIOM_SPEC_VERSION: &str = "1.2.0";
pub const AXIOM_ENCODING: &str = "rfc8785-jcs";

pub const MAX_PROOF_SIZE_BYTES: usize = 64 * 1024;
pub const MAX_PROOFS_PER_FRAME: usize = 32;
pub const MAX_SIGNATURE_STRING_BYTES: usize = 16 * 1024;
pub const MAX_RECURSION_DEPTH: usize = 32;
pub const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991; // 2^53 - 1
pub const MIN_SAFE_INTEGER: i64 = -9_007_199_254_740_991;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomainTag(&'static str);

impl DomainTag {
    pub const STATE: Self = Self("AXIOM-STATE-CANONICAL-v1:");
    pub const GENESIS: Self = Self("AXIOM-GENESIS-v1:");
    pub const TRANSITION: Self = Self("AXIOM-TRANSITION-v1:");
    pub const PROOF: Self = Self("AXIOM-PROOF-v1:");
    pub const FRAME: Self = Self("AXIOM-FRAME-CANONICAL-v1:");

    #[inline]
    pub const fn prefix(&self) -> &'static str {
        self.0
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'static [u8] {
        self.0.as_bytes()
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AxiomError {
    #[error("[ACP1001][FATAL] Invalid protocol name: '{0}'")]
    InvalidProtocol(String),

    #[error("[ACP1002][FATAL] Invalid protocol_id: '{0}'")]
    InvalidProtocolId(String),

    #[error("[ACP1003][FATAL] Unsupported hash algorithm token: '{0}'")]
    UnsupportedHashAlgorithm(String),

    #[error("[ACP1004][RECOVERABLE] Timestamp '{ts}' fails RFC 3339 validation: {reason}")]
    InvalidTimestamp { ts: String, reason: String },

    #[error("[ACP2001][FATAL] Deserialization failed: {0}")]
    DeserializationFailed(String),

    #[error("[ACP2002][FATAL] Vendor namespace key '{0}' fails RFC regex requirement")]
    InvalidVendorNamespace(String),

    #[error("[ACP2003][FATAL] Invalid JCS IEEE-754 number: {0}")]
    InvalidJcsNumber(String),

    #[error("[ACP2004][FATAL] Integer exceeds RFC 8785 safe boundary 2^53 - 1: {0}")]
    IntegerPrecisionLoss(String),

    #[error("[ACP2005][FATAL] JCS recursion depth exceeded MAX_RECURSION_DEPTH ({0})")]
    RecursionLimitExceeded(usize),

    #[error("[ACP3001][FATAL] Genesis mismatch: expected '{expected}', found '{actual}'")]
    GenesisMismatch { expected: String, actual: String },

    #[error("[ACP3002][FATAL] Duplicate transition ID detected: '{0}'")]
    DuplicateTransitionId(String),

    #[error("[ACP3003][FATAL] Missing parent transition: '{0}'")]
    MissingParentTransition(String),

    #[error("[ACP4001][FATAL] Algorithm token must match [a-z][a-z0-9-]*; got '{0}'")]
    InvalidAlgorithmToken(String),

    #[error("[ACP4002][RECOVERABLE] target_type '{0}' is non-normative")]
    NonNormativeTargetType(String),

    #[error("[ACP4003][FATAL] Signature string exceeds max byte bound ({0} bytes)")]
    SignatureExceedsLimit(usize),
}

pub type Result<T> = core::result::Result<T, AxiomError>;

// ---------------------------------------------------------------------------
// Digest helpers
// ---------------------------------------------------------------------------

#[inline]
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut hex = Vec::with_capacity(bytes.len() * 2);
    for &b in bytes {
        hex.push(LUT[(b >> 4) as usize]);
        hex.push(LUT[(b & 0x0f) as usize]);
    }
    // SAFETY: LUT is ASCII hex only
    unsafe { String::from_utf8_unchecked(hex) }
}

pub trait StreamHasher {
    fn update(&mut self, data: &[u8]);
    fn finalize_32(self) -> [u8; 32];
}

pub struct Sha256StreamHasher(Sha256);

impl StreamHasher for Sha256StreamHasher {
    fn update(&mut self, data: &[u8]) {
        Digest::update(&mut self.0, data);
    }
    fn finalize_32(self) -> [u8; 32] {
        Digest::finalize(self.0).into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HashAlgorithm {
    #[serde(rename = "sha256")]
    Sha256,
}

impl HashAlgorithm {
    pub fn from_token(token: &str) -> Result<Self> {
        match token {
            "sha256" => Ok(Self::Sha256),
            _ => Err(AxiomError::UnsupportedHashAlgorithm(token.to_string())),
        }
    }

    pub const fn as_token(&self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
        }
    }

    pub fn digest32(&self, input: &[u8]) -> [u8; 32] {
        match self {
            Self::Sha256 => {
                let mut h = Sha256::new();
                h.update(input);
                h.finalize().into()
            }
        }
    }

    pub fn build_hasher(&self) -> Sha256StreamHasher {
        Sha256StreamHasher(Sha256::new())
    }

    pub fn digest_hex(&self, input: &[u8]) -> String {
        bytes_to_hex(&self.digest32(input))
    }
}

// ---------------------------------------------------------------------------
// Stream JCS Writer (RFC 8785)
// ---------------------------------------------------------------------------

pub struct JcsStreamWriter<'a, H: StreamHasher> {
    hasher: &'a mut H,
}

impl<'a, H: StreamHasher> JcsStreamWriter<'a, H> {
    pub fn new(hasher: &'a mut H) -> Self {
        Self { hasher }
    }

    #[inline]
    fn write_raw(&mut self, bytes: &[u8]) {
        self.hasher.update(bytes);
    }

    pub fn write_value(&mut self, val: &Value, depth: usize) -> Result<()> {
        if depth > MAX_RECURSION_DEPTH {
            return Err(AxiomError::RecursionLimitExceeded(MAX_RECURSION_DEPTH));
        }

        match val {
            Value::Null => self.write_raw(b"null"),
            Value::Bool(true) => self.write_raw(b"true"),
            Value::Bool(false) => self.write_raw(b"false"),
            Value::Number(n) => self.write_number(n)?,
            Value::String(s) => self.write_json_string(s)?,
            Value::Array(arr) => {
                self.write_raw(b"[");
                for (i, elem) in arr.iter().enumerate() {
                    if i > 0 {
                        self.write_raw(b",");
                    }
                    self.write_value(elem, depth + 1)?;
                }
                self.write_raw(b"]");
            }
            Value::Object(map) => {
                self.write_raw(b"{");
                let mut keys: Vec<&String> = map.keys().collect();
                // RFC 8785 §3.2.3: UTF-16 code unit order
                keys.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
                for (i, k) in keys.iter().enumerate() {
                    if i > 0 {
                        self.write_raw(b",");
                    }
                    self.write_json_string(k)?;
                    self.write_raw(b":");
                    self.write_value(&map[*k], depth + 1)?;
                }
                self.write_raw(b"}");
            }
        }
        Ok(())
    }

    fn write_number(&mut self, n: &serde_json::Number) -> Result<()> {
        if let Some(i) = n.as_i64() {
            if !(MIN_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&i) {
                return Err(AxiomError::IntegerPrecisionLoss(i.to_string()));
            }
            let s = i.to_string();
            self.write_raw(s.as_bytes());
            return Ok(());
        }
        if let Some(u) = n.as_u64() {
            if u > MAX_SAFE_INTEGER as u64 {
                return Err(AxiomError::IntegerPrecisionLoss(u.to_string()));
            }
            let s = u.to_string();
            self.write_raw(s.as_bytes());
            return Ok(());
        }

        let f = n
            .as_f64()
            .ok_or_else(|| AxiomError::InvalidJcsNumber("Not finite f64".into()))?;
        if f.is_nan() || f.is_infinite() {
            return Err(AxiomError::InvalidJcsNumber(
                "NaN/Infinity strictly forbidden".into(),
            ));
        }
        if f == 0.0 {
            self.write_raw(b"0");
            return Ok(());
        }

        let mut buf = ryu::Buffer::new();
        let formatted = buf.format_finite(f);
        // JCS: prefer "e" over "e+"
        // Clippy: sliced_string_as_bytes — prefer as_bytes() once then index
        if let Some(pos) = formatted.find("e+") {
            let bytes = formatted.as_bytes();
            self.write_raw(&bytes[..pos + 1]);
            self.write_raw(&bytes[pos + 2..]);
        } else {
            self.write_raw(formatted.as_bytes());
        }
        Ok(())
    }

    fn write_json_string(&mut self, s: &str) -> Result<()> {
        self.write_raw(b"\"");
        for ch in s.chars() {
            match ch {
                '"' => self.write_raw(b"\\\""),
                '\\' => self.write_raw(b"\\\\"),
                '\n' => self.write_raw(b"\\n"),
                '\r' => self.write_raw(b"\\r"),
                '\t' => self.write_raw(b"\\t"),
                c if (c as u32) < 0x20 => {
                    const HEX: &[u8; 16] = b"0123456789abcdef";
                    let n = c as u32;
                    let esc: [u8; 6] = [
                        b'\\',
                        b'u',
                        HEX[((n >> 12) & 0xF) as usize],
                        HEX[((n >> 8) & 0xF) as usize],
                        HEX[((n >> 4) & 0xF) as usize],
                        HEX[(n & 0xF) as usize],
                    ];
                    self.write_raw(&esc);
                }
                _ => {
                    let mut buf = [0u8; 4];
                    let encoded = ch.encode_utf8(&mut buf);
                    self.write_raw(encoded.as_bytes());
                }
            }
        }
        self.write_raw(b"\"");
        Ok(())
    }
}

/// Domain-separated stream hash (fixed [u8; 32], no intermediate Vec for digest)
pub fn compute_domain_hash(domain: DomainTag, val: &Value, alg: HashAlgorithm) -> Result<[u8; 32]> {
    let mut hasher = alg.build_hasher();
    hasher.update(domain.as_bytes());
    {
        let mut jcs = JcsStreamWriter::new(&mut hasher);
        jcs.write_value(val, 0)?;
    }
    Ok(hasher.finalize_32())
}

// ---------------------------------------------------------------------------
// Timestamp / validation helpers
// ---------------------------------------------------------------------------

pub fn validate_vendor_namespace(ns: &str) -> Result<()> {
    if ns.is_empty() {
        return Err(AxiomError::InvalidVendorNamespace(ns.to_string()));
    }
    for part in ns.split('.') {
        if part.is_empty()
            || !part
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(AxiomError::InvalidVendorNamespace(ns.to_string()));
        }
    }
    Ok(())
}

pub fn validate_alg_token(alg: &str) -> Result<()> {
    let mut chars = alg.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return Err(AxiomError::InvalidAlgorithmToken(alg.to_string())),
    }
    for c in chars {
        if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err(AxiomError::InvalidAlgorithmToken(alg.to_string()));
        }
    }
    Ok(())
}

pub fn normalize_timestamp(ts: &str) -> Result<String> {
    let ts_clean = ts.trim();
    let dt =
        OffsetDateTime::parse(ts_clean, &Rfc3339).map_err(|e| AxiomError::InvalidTimestamp {
            ts: ts_clean.to_string(),
            reason: e.to_string(),
        })?;

    let utc_dt = dt.to_offset(time::UtcOffset::UTC);
    let year = utc_dt.year();
    let month = u8::from(utc_dt.month());
    let day = utc_dt.day();
    let hour = utc_dt.hour();
    let minute = utc_dt.minute();
    let second = utc_dt.second();
    let nano = utc_dt.nanosecond();

    if nano == 0 {
        Ok(format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            year, month, day, hour, minute, second
        ))
    } else {
        let millis = nano / 1_000_000;
        if nano % 1_000_000 == 0 {
            Ok(format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
                year, month, day, hour, minute, second, millis
            ))
        } else {
            Ok(format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09}Z",
                year, month, day, hour, minute, second, nano
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// Core protocol structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxiomHeader {
    pub protocol: String,
    pub protocol_id: String,
    pub version: String,
    pub encoding: String,
    pub hash_algorithm: String,
}

impl Default for AxiomHeader {
    fn default() -> Self {
        Self {
            protocol: AXIOM_PROTOCOL_NAME.to_string(),
            protocol_id: AXIOM_PROTOCOL_ID.to_string(),
            version: AXIOM_SPEC_VERSION.to_string(),
            encoding: AXIOM_ENCODING.to_string(),
            hash_algorithm: "sha256".to_string(),
        }
    }
}

impl AxiomHeader {
    pub fn validate(&self) -> Result<()> {
        if self.protocol != AXIOM_PROTOCOL_NAME {
            return Err(AxiomError::InvalidProtocol(self.protocol.clone()));
        }
        if self.protocol_id != AXIOM_PROTOCOL_ID {
            return Err(AxiomError::InvalidProtocolId(self.protocol_id.clone()));
        }
        HashAlgorithm::from_token(&self.hash_algorithm)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Genesis {
    pub genesis_id: String,
    pub created_by: String,
    pub initial_state_hash: String,
    pub timestamp: String,
}

impl Genesis {
    pub fn new(
        genesis_id: &str,
        created_by: &str,
        initial_state_hash: &str,
        timestamp: &str,
    ) -> Result<Self> {
        Ok(Self {
            genesis_id: genesis_id.to_string(),
            created_by: created_by.to_string(),
            initial_state_hash: initial_state_hash.to_string(),
            timestamp: normalize_timestamp(timestamp)?,
        })
    }

    pub fn genesis_hash(&self, hasher: HashAlgorithm) -> Result<[u8; 32]> {
        let val = serde_json::to_value(self)
            .map_err(|e| AxiomError::DeserializationFailed(e.to_string()))?;
        compute_domain_hash(DomainTag::GENESIS, &val, hasher)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofEnvelope {
    pub algorithm: String,
    pub signer: String,
    pub signature: String,
    pub target_hash: String,
    #[serde(default = "default_target_type")]
    pub target_type: String,
}

fn default_target_type() -> String {
    "frame".to_string()
}

impl ProofEnvelope {
    pub fn validate(&self) -> Result<()> {
        validate_alg_token(&self.algorithm)?;
        let normative = ["frame", "core", "transition", "genesis"];
        if !normative.contains(&self.target_type.as_str()) {
            return Err(AxiomError::NonNormativeTargetType(self.target_type.clone()));
        }
        if self.signature.len() > MAX_SIGNATURE_STRING_BYTES {
            return Err(AxiomError::SignatureExceedsLimit(self.signature.len()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionRecord {
    pub transition_id: String,
    pub sequence_number: u64,
    pub before_states: Vec<String>,
    pub after: String,
    pub operation: String,
    pub actor: String,
    pub timestamp: String,
    #[serde(default)]
    pub parent_transitions: Vec<String>,
    pub reason: Option<String>,
    pub delta: Option<Value>,
    pub proof: Option<ProofEnvelope>,

    #[serde(skip)]
    #[cfg(feature = "std")]
    hash_cache: OnceLock<[u8; 32]>,
}

impl PartialEq for TransitionRecord {
    fn eq(&self, other: &Self) -> bool {
        self.transition_id == other.transition_id
            && self.sequence_number == other.sequence_number
            && self.before_states == other.before_states
            && self.after == other.after
            && self.operation == other.operation
            && self.actor == other.actor
            && self.timestamp == other.timestamp
            && self.parent_transitions == other.parent_transitions
            && self.reason == other.reason
            && self.delta == other.delta
            && self.proof == other.proof
    }
}

impl Eq for TransitionRecord {}

impl TransitionRecord {
    pub fn transition_hash(&self, hasher: HashAlgorithm) -> Result<[u8; 32]> {
        #[cfg(feature = "std")]
        if let Some(&cached) = self.hash_cache.get() {
            return Ok(cached);
        }

        let val = serde_json::to_value(self)
            .map_err(|e| AxiomError::DeserializationFailed(e.to_string()))?;
        let hash = compute_domain_hash(DomainTag::TRANSITION, &val, hasher)?;

        #[cfg(feature = "std")]
        let _ = self.hash_cache.set(hash);

        Ok(hash)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    pub entity: String,
    pub scope: String,
    pub domain: String,
    #[serde(default)]
    pub boundary: Vec<String>,
    pub scheme: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub current: Value,
    pub initial: Value,
    pub target: Value,
    #[serde(default)]
    pub transition: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Invariant {
    #[serde(default)]
    pub must_hold: Vec<String>,
    #[serde(default)]
    pub forbidden: Vec<String>,
    #[serde(default)]
    pub conservation: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Constraint {
    #[serde(default)]
    pub hard: Vec<String>,
    #[serde(default)]
    pub soft: Vec<String>,
    #[serde(default)]
    pub resource: Vec<String>,
    #[serde(default)]
    pub limit: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxiomCore {
    pub identity: Identity,
    pub state: State,
    pub invariant: Invariant,
    pub constraint: Constraint,
}

impl AxiomCore {
    pub fn core_hash(&self, hasher: HashAlgorithm) -> Result<[u8; 32]> {
        let val = serde_json::to_value(self)
            .map_err(|e| AxiomError::DeserializationFailed(e.to_string()))?;
        compute_domain_hash(DomainTag::STATE, &val, hasher)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AxiomExtension {
    pub geometry: Option<Value>,
    pub intent: Option<Value>,
    pub difference: Option<Value>,
    pub memory: Option<Value>,
    pub output_contract: Option<Value>,
    #[serde(rename = "$ext", default)]
    pub ext: BTreeMap<String, Value>,
}

impl AxiomExtension {
    pub fn validate(&self) -> Result<()> {
        for key in self.ext.keys() {
            validate_vendor_namespace(key)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxiomFrame {
    pub header: AxiomHeader,
    pub genesis: Genesis,
    pub core: AxiomCore,
    pub extension: Option<AxiomExtension>,
    #[serde(default)]
    pub transitions: Vec<TransitionRecord>,
    #[serde(default)]
    pub proofs: Vec<ProofEnvelope>,
}

impl AxiomFrame {
    pub fn coordinate_id(&self, hasher: HashAlgorithm) -> Result<[u8; 32]> {
        let mut content_dict = BTreeMap::new();
        content_dict.insert(
            "header".to_string(),
            serde_json::to_value(&self.header)
                .map_err(|e| AxiomError::DeserializationFailed(e.to_string()))?,
        );
        content_dict.insert(
            "genesis".to_string(),
            serde_json::to_value(&self.genesis)
                .map_err(|e| AxiomError::DeserializationFailed(e.to_string()))?,
        );
        content_dict.insert(
            "core".to_string(),
            serde_json::to_value(&self.core)
                .map_err(|e| AxiomError::DeserializationFailed(e.to_string()))?,
        );

        if let Some(ref ext) = self.extension {
            content_dict.insert(
                "extension".to_string(),
                serde_json::to_value(ext)
                    .map_err(|e| AxiomError::DeserializationFailed(e.to_string()))?,
            );
        }
        if !self.transitions.is_empty() {
            content_dict.insert(
                "transitions".to_string(),
                serde_json::to_value(&self.transitions)
                    .map_err(|e| AxiomError::DeserializationFailed(e.to_string()))?,
            );
        }

        let val = Value::Object(content_dict.into_iter().collect());
        compute_domain_hash(DomainTag::FRAME, &val, hasher)
    }

    pub fn verify_causal_chain(&self) -> Result<()> {
        let mut known_ids = BTreeMap::new();
        for t in &self.transitions {
            if known_ids.insert(t.transition_id.as_str(), t).is_some() {
                return Err(AxiomError::DuplicateTransitionId(t.transition_id.clone()));
            }
        }
        for t in &self.transitions {
            for parent_id in &t.parent_transitions {
                if !known_ids.contains_key(parent_id.as_str()) {
                    return Err(AxiomError::MissingParentTransition(parent_id.clone()));
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn prototype_seal_still_works() {
        let c = contract_from_text("goal: demo\nsafety: no secrets");
        assert_eq!(c.hash_a.len(), 64);
        let c2 = contract_from_text("goal: demo\nsafety: no secrets");
        assert_eq!(c.hash_a, c2.hash_a);
    }

    #[test]
    fn domain_hash_determinism() {
        let alg = HashAlgorithm::Sha256;
        let v = json!({"a": 1, "b": "x"});
        let h1 = compute_domain_hash(DomainTag::STATE, &v, alg).unwrap();
        let h2 = compute_domain_hash(DomainTag::STATE, &v, alg).unwrap();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 32);
    }

    #[test]
    fn genesis_timestamp_normalized() {
        let g = Genesis::new(
            "gen-1",
            "actor://admin",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "2026-08-14T15:00:00Z",
        )
        .unwrap();
        assert_eq!(g.timestamp, "2026-08-14T15:00:00Z");
        let hash = g.genesis_hash(HashAlgorithm::Sha256).unwrap();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn integer_precision_loss_rejected() {
        let alg = HashAlgorithm::Sha256;
        let v = json!(9007199254740992i64);
        let err = compute_domain_hash(DomainTag::STATE, &v, alg);
        assert!(matches!(err, Err(AxiomError::IntegerPrecisionLoss(_))));
    }

    #[test]
    fn causal_chain_detects_missing_parent() {
        let genesis = Genesis::new("g", "a", "00", "2026-08-14T00:00:00Z").unwrap();
        let frame = AxiomFrame {
            header: AxiomHeader::default(),
            genesis,
            core: AxiomCore {
                identity: Identity {
                    entity: "e".into(),
                    scope: "s".into(),
                    domain: "d".into(),
                    boundary: vec![],
                    scheme: None,
                },
                state: State {
                    current: json!({}),
                    initial: json!({}),
                    target: json!({}),
                    transition: vec![],
                },
                invariant: Invariant::default(),
                constraint: Constraint::default(),
            },
            extension: None,
            transitions: vec![TransitionRecord {
                transition_id: "t1".into(),
                sequence_number: 1,
                before_states: vec![],
                after: "s1".into(),
                operation: "op".into(),
                actor: "a".into(),
                timestamp: "2026-08-14T00:00:01Z".into(),
                parent_transitions: vec!["missing-parent".into()],
                reason: None,
                delta: None,
                proof: None,
                #[cfg(feature = "std")]
                hash_cache: OnceLock::new(),
            }],
            proofs: vec![],
        };
        assert!(matches!(
            frame.verify_causal_chain(),
            Err(AxiomError::MissingParentTransition(_))
        ));
    }
}
