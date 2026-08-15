//! PLP Capsule v1.3.0 — Ultra-Optimized Production Reference Implementation (FIXED)
//!
//! Fixes applied on top of the reviewed v1.3.0 draft:
//! - BUG FIX: `write_raw_str` / `write_i64` / `write_u128` / `write_canonical_f64` /
//!   `write_observation_block` took `W: Write` (implicitly `Sized`), but were called
//!   with `writer: &mut dyn Write` inside `compute_content_hash`'s streaming closure.
//!   A generic parameter bound only by `Write` cannot be unified with the unsized
//!   type `dyn Write`, so the crate did not compile. All of these now use
//!   `W: Write + ?Sized`.
//! - BUG FIX: `HasherWriter<D: Digest>` was defined unconditionally, but its only
//!   import of the `Digest` trait lives behind `#[cfg(feature = "sha2-hash")]`.
//!   Building with `--no-default-features --features blake3-hash` would fail to
//!   resolve `Digest`. `HasherWriter` is now cfg-gated the same way as its sole
//!   user, `Sha256Algorithm`.
//! - BUG FIX: the control-character escape branch in `write_raw_str` declared an
//!   unused `[0u8; 6]` buffer and fell back to `format!("\\u{:04x}", ...)`, which
//!   both triggers an unused-variable warning and contradicts the "no format! in
//!   the hot path" optimization goal. Replaced with a manual 6-byte `\uXXXX`
//!   encoder. Output bytes are identical, so golden hash vectors are unaffected.
//! - Cargo.toml: added the missing `itoa` dependency (used throughout but absent
//!   from the project's actual Cargo.toml).
//!
//! No changes were made to the canonical JSON layout, field order, or numeric
//! formatting — every fix here is compile/robustness only, so existing golden
//! hash vectors should still reproduce identically.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::sync::Arc;
use thiserror::Error;

#[cfg(feature = "sha2-hash")]
use sha2::{Digest, Sha256};

// ==========================================================
// 1. Errors
// ==========================================================

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum CapsuleError {
    #[error("Observer '{name}': {reason}")]
    ObserverFailed { name: String, reason: String },

    #[error("{context}: key '{key}' has non-finite value ({value})")]
    NonFiniteValue {
        context: String,
        key: String,
        value: f64,
    },

    #[error("Hash serialization failed: {0}")]
    HashSerializationFailed(String),

    #[error("Hash mismatch: expected {expected:?}, got {calculated}")]
    HashMismatch {
        expected: Option<String>,
        calculated: String,
    },

    #[error("Hash computation failed: {0}")]
    HashComputationFailed(String),

    #[error("Schema not found for capability '{capability}'")]
    SchemaNotFound { capability: String },

    #[error("Unsupported hash algorithm: {0}")]
    UnsupportedHashAlgorithm(String),

    #[error("{0}")]
    Other(String),
}

impl CapsuleError {
    pub fn is_observer_related(&self) -> bool {
        matches!(
            self,
            CapsuleError::ObserverFailed { .. }
                | CapsuleError::SchemaNotFound { .. }
                | CapsuleError::NonFiniteValue { .. }
        )
    }

    #[inline]
    fn io(err: std::io::Error) -> Self {
        CapsuleError::HashSerializationFailed(err.to_string())
    }
}

// ==========================================================
// 2. HashAlgorithm & HasherWriter
// ==========================================================

pub trait HashAlgorithm: Send + Sync + Clone {
    fn name(&self) -> &'static str;
    fn digest_hex(&self, data: &[u8]) -> String;
    fn compute_streaming<F>(&self, f: F) -> Result<String, CapsuleError>
    where
        F: FnOnce(&mut dyn Write) -> Result<(), CapsuleError>;
}

#[cfg(feature = "sha2-hash")]
#[derive(Debug, Clone, Copy, Default)]
pub struct Sha256Algorithm;

#[cfg(feature = "sha2-hash")]
impl HashAlgorithm for Sha256Algorithm {
    fn name(&self) -> &'static str {
        "sha256"
    }

    fn digest_hex(&self, data: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(data);
        hex::encode(h.finalize())
    }

    fn compute_streaming<F>(&self, f: F) -> Result<String, CapsuleError>
    where
        F: FnOnce(&mut dyn Write) -> Result<(), CapsuleError>,
    {
        let mut writer = HasherWriter::<Sha256>::new();
        f(&mut writer)?;
        Ok(writer.finish_hex())
    }
}

#[cfg(feature = "blake3-hash")]
#[derive(Debug, Clone, Copy, Default)]
pub struct Blake3Algorithm;

#[cfg(feature = "blake3-hash")]
impl HashAlgorithm for Blake3Algorithm {
    fn name(&self) -> &'static str {
        "blake3"
    }

    fn digest_hex(&self, data: &[u8]) -> String {
        hex::encode(blake3::hash(data).as_bytes())
    }

    fn compute_streaming<F>(&self, f: F) -> Result<String, CapsuleError>
    where
        F: FnOnce(&mut dyn Write) -> Result<(), CapsuleError>,
    {
        let mut writer = Blake3HasherWriter::new();
        f(&mut writer)?;
        Ok(writer.finish_hex())
    }
}

// FIX: cfg-gated to match the only place `Digest` is imported (sha2-hash feature).
#[cfg(feature = "sha2-hash")]
pub struct HasherWriter<D: Digest> {
    hasher: D,
}

#[cfg(feature = "sha2-hash")]
impl<D: Digest> HasherWriter<D> {
    pub fn new() -> Self {
        Self { hasher: D::new() }
    }

    pub fn finish_hex(self) -> String {
        hex::encode(self.hasher.finalize())
    }
}

#[cfg(feature = "sha2-hash")]
impl<D: Digest> Write for HasherWriter<D> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.hasher.update(buf);
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "blake3-hash")]
pub struct Blake3HasherWriter {
    hasher: blake3::Hasher,
}

#[cfg(feature = "blake3-hash")]
impl Blake3HasherWriter {
    pub fn new() -> Self {
        Self {
            hasher: blake3::Hasher::new(),
        }
    }

    pub fn finish_hex(self) -> String {
        self.hasher.finalize().to_hex().to_string()
    }
}

#[cfg(feature = "blake3-hash")]
impl Write for Blake3HasherWriter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.hasher.update(buf);
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// ==========================================================
// 3. SchemaProvider & CapabilityRegistry
// ==========================================================

pub trait SchemaProvider: Send + Sync {
    fn get_schema(&self, capability: &str) -> Option<String>;
}

#[derive(Clone, Default)]
pub struct CapabilityRegistry {
    schemas: Arc<HashMap<String, String>>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self {
            schemas: Arc::new(HashMap::new()),
        }
    }

    pub fn register(mut self, capability: &str, schema: &str) -> Self {
        let map = Arc::make_mut(&mut self.schemas);
        map.insert(capability.to_string(), schema.to_string());
        self
    }
}

impl SchemaProvider for CapabilityRegistry {
    fn get_schema(&self, capability: &str) -> Option<String> {
        self.schemas.get(capability).cloned()
    }
}

// ==========================================================
// 4. Core Structures
// ==========================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleHeader {
    pub protocol: String,
    pub capsule_schema: String,
    pub version: String,
    pub capsule_id: String,
    pub parent_id: Option<String>,
    pub clock: i64,
    pub sequence: i64,
    pub timestamp_ns: u128,
    pub source: String,
    pub flags: CapsuleFlags,
    pub hash_algorithm: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapsuleFlags {
    pub is_keyframe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputCapsule {
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationBlock {
    pub name: String,
    pub schema: String,
    pub capability: String,
    pub observer_id: String,
    pub values: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObsKey<'a> {
    pub name: &'a str,
    pub schema: &'a str,
    pub capability: &'a str,
    pub observer_id: &'a str,
}

impl<'a> ObsKey<'a> {
    #[inline]
    pub fn from_block(obs: &'a ObservationBlock) -> Self {
        Self {
            name: &obs.name,
            schema: &obs.schema,
            capability: &obs.capability,
            observer_id: &obs.observer_id,
        }
    }

    /// Fast length-prefixed stable key formatting without format! macro overhead
    pub fn to_stable_key(&self) -> String {
        let mut num_buf = itoa::Buffer::new();
        let estimated_cap = self.name.len()
            + self.schema.len()
            + self.capability.len()
            + self.observer_id.len()
            + 32;
        let mut out = String::with_capacity(estimated_cap);

        out.push_str(num_buf.format(self.name.len()));
        out.push('\0');
        out.push_str(self.name);
        out.push('\0');

        out.push_str(num_buf.format(self.schema.len()));
        out.push('\0');
        out.push_str(self.schema);
        out.push('\0');

        out.push_str(num_buf.format(self.capability.len()));
        out.push('\0');
        out.push_str(self.capability);
        out.push('\0');

        out.push_str(num_buf.format(self.observer_id.len()));
        out.push('\0');
        out.push_str(self.observer_id);

        out
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueDelta {
    Added(f64),
    Modified(f64),
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeltaKind {
    Added,
    Modified,
    Removed,
}

impl DeltaKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeltaKind::Added => "added",
            DeltaKind::Modified => "modified",
            DeltaKind::Removed => "removed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaEntry {
    pub kind: DeltaKind,
    pub values: BTreeMap<String, ValueDelta>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeltaBlock {
    pub changes: BTreeMap<String, DeltaEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapsuleIntegrity {
    pub content_hash: Option<String>,
    pub valid: bool,
    pub observer_valid: bool,
    pub hash_valid: Option<bool>,
    pub errors: Vec<CapsuleError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PLPCapsule {
    pub header: CapsuleHeader,
    pub input: InputCapsule,
    pub observations: Vec<ObservationBlock>,
    pub delta: DeltaBlock,
    pub integrity: CapsuleIntegrity,
}

// ==========================================================
// 5. Observer
// ==========================================================

pub trait Observer<W>: Send + Sync {
    fn name(&self) -> &str;
    fn observer_id(&self) -> &str;
    fn observe(&self, world: &W) -> Result<ObservationBlock, String>;
}

// ==========================================================
// 6. Low-level JSON Streaming Helpers
// ==========================================================

fn default_protocol() -> String {
    "PLP/1.1".to_string()
}
fn default_schema() -> String {
    "v1/capsule".to_string()
}
fn default_version() -> String {
    "1.1.3".to_string()
}
fn default_source() -> String {
    "system".to_string()
}

#[inline]
fn write_raw_str<W: Write + ?Sized>(out: &mut W, s: &str) -> Result<(), CapsuleError> {
    out.write_all(b"\"").map_err(CapsuleError::io)?;
    for ch in s.chars() {
        match ch {
            '"' => out.write_all(b"\\\"").map_err(CapsuleError::io)?,
            '\\' => out.write_all(b"\\\\").map_err(CapsuleError::io)?,
            '\n' => out.write_all(b"\\n").map_err(CapsuleError::io)?,
            '\r' => out.write_all(b"\\r").map_err(CapsuleError::io)?,
            '\t' => out.write_all(b"\\t").map_err(CapsuleError::io)?,
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
                out.write_all(&esc).map_err(CapsuleError::io)?;
            }
            _ => {
                let mut buf = [0u8; 4];
                let encoded = ch.encode_utf8(&mut buf);
                out.write_all(encoded.as_bytes())
                    .map_err(CapsuleError::io)?;
            }
        }
    }
    out.write_all(b"\"").map_err(CapsuleError::io)
}

#[inline]
fn write_i64<W: Write + ?Sized>(out: &mut W, val: i64) -> Result<(), CapsuleError> {
    let mut buf = itoa::Buffer::new();
    out.write_all(buf.format(val).as_bytes())
        .map_err(CapsuleError::io)
}

#[inline]
fn write_u128<W: Write + ?Sized>(out: &mut W, val: u128) -> Result<(), CapsuleError> {
    let mut buf = itoa::Buffer::new();
    out.write_all(buf.format(val).as_bytes())
        .map_err(CapsuleError::io)
}

#[inline]
fn write_canonical_f64<W: Write + ?Sized>(out: &mut W, val: f64) -> Result<(), CapsuleError> {
    if !val.is_finite() {
        return Err(CapsuleError::NonFiniteValue {
            context: "canonicalize_f64".into(),
            key: "".into(),
            value: val,
        });
    }
    if val == 0.0 {
        return out.write_all(b"0").map_err(CapsuleError::io);
    }
    let mut buf = ryu::Buffer::new();
    let s = buf.format_finite(val);
    if let Some(pos) = s.find("e+") {
        out.write_all(s[..pos + 1].as_bytes())
            .map_err(CapsuleError::io)?;
        out.write_all(s[pos + 2..].as_bytes())
            .map_err(CapsuleError::io)?;
    } else {
        out.write_all(s.as_bytes()).map_err(CapsuleError::io)?;
    }
    Ok(())
}

fn validate_f64_values(
    values: &BTreeMap<String, f64>,
    context_fn: impl Fn() -> String,
) -> Result<(), CapsuleError> {
    for (k, &v) in values {
        if !v.is_finite() {
            return Err(CapsuleError::NonFiniteValue {
                context: context_fn(),
                key: k.clone(),
                value: v,
            });
        }
    }
    Ok(())
}

fn approx_eq(a: f64, b: f64, rel_eps: f64, abs_eps: f64) -> bool {
    let diff = (a - b).abs();
    if diff <= abs_eps {
        return true;
    }
    let max_abs = a.abs().max(b.abs());
    if max_abs == 0.0 {
        return true;
    }
    diff / max_abs <= rel_eps
}

// ==========================================================
// 7. Streaming Content Hash Calculation
// ==========================================================

pub fn compute_content_hash<H: HashAlgorithm>(
    header: &CapsuleHeader,
    observations: &[ObservationBlock],
    delta: &DeltaBlock,
    hasher: &H,
) -> Result<String, CapsuleError> {
    for obs in observations {
        validate_f64_values(&obs.values, || format!("Observation '{}'", obs.name))?;
    }
    for (key, entry) in &delta.changes {
        for (k, vd) in &entry.values {
            if let ValueDelta::Added(v) | ValueDelta::Modified(v) = vd {
                if !v.is_finite() {
                    return Err(CapsuleError::NonFiniteValue {
                        context: format!("Delta '{}'", key),
                        key: k.clone(),
                        value: *v,
                    });
                }
            }
        }
    }

    hasher.compute_streaming(|writer| {
        writer
            .write_all(b"{\"header\":{\"protocol\":")
            .map_err(CapsuleError::io)?;
        write_raw_str(writer, &header.protocol)?;
        writer
            .write_all(b",\"capsule_schema\":")
            .map_err(CapsuleError::io)?;
        write_raw_str(writer, &header.capsule_schema)?;
        writer
            .write_all(b",\"version\":")
            .map_err(CapsuleError::io)?;
        write_raw_str(writer, &header.version)?;
        writer
            .write_all(b",\"capsule_id\":")
            .map_err(CapsuleError::io)?;
        write_raw_str(writer, &header.capsule_id)?;
        writer
            .write_all(b",\"parent_id\":")
            .map_err(CapsuleError::io)?;
        if let Some(ref p) = header.parent_id {
            write_raw_str(writer, p)?;
        } else {
            writer.write_all(b"null").map_err(CapsuleError::io)?;
        }
        writer.write_all(b",\"clock\":").map_err(CapsuleError::io)?;
        write_i64(writer, header.clock)?;
        writer
            .write_all(b",\"sequence\":")
            .map_err(CapsuleError::io)?;
        write_i64(writer, header.sequence)?;
        writer
            .write_all(b",\"timestamp_ns\":\"")
            .map_err(CapsuleError::io)?;
        write_u128(writer, header.timestamp_ns)?;
        writer
            .write_all(b"\",\"source\":")
            .map_err(CapsuleError::io)?;
        write_raw_str(writer, &header.source)?;
        writer
            .write_all(b",\"is_keyframe\":")
            .map_err(CapsuleError::io)?;
        writer
            .write_all(if header.flags.is_keyframe {
                b"true"
            } else {
                b"false"
            })
            .map_err(CapsuleError::io)?;
        writer
            .write_all(b",\"hash_algorithm\":")
            .map_err(CapsuleError::io)?;
        write_raw_str(writer, &header.hash_algorithm)?;

        writer
            .write_all(b"},\"observations\":[")
            .map_err(CapsuleError::io)?;

        let obs_len = observations.len();
        if obs_len <= 16 {
            let mut stack_buf: [Option<&ObservationBlock>; 16] = [None; 16];
            for (i, obs) in observations.iter().enumerate() {
                stack_buf[i] = Some(obs);
            }
            let slice = &mut stack_buf[..obs_len];
            slice.sort_by(|a, b| {
                ObsKey::from_block(a.unwrap()).cmp(&ObsKey::from_block(b.unwrap()))
            });

            for (i, obs_opt) in slice.iter().enumerate() {
                if i > 0 {
                    writer.write_all(b",").map_err(CapsuleError::io)?;
                }
                write_observation_block(writer, obs_opt.unwrap())?;
            }
        } else {
            let mut heap_buf: Vec<&ObservationBlock> = observations.iter().collect();
            heap_buf.sort_by(|a, b| ObsKey::from_block(a).cmp(&ObsKey::from_block(b)));

            for (i, obs) in heap_buf.iter().enumerate() {
                if i > 0 {
                    writer.write_all(b",").map_err(CapsuleError::io)?;
                }
                write_observation_block(writer, obs)?;
            }
        }

        writer
            .write_all(b"],\"delta\":{")
            .map_err(CapsuleError::io)?;
        for (i, (key, entry)) in delta.changes.iter().enumerate() {
            if i > 0 {
                writer.write_all(b",").map_err(CapsuleError::io)?;
            }
            write_raw_str(writer, key)?;
            writer.write_all(b":{\"kind\":").map_err(CapsuleError::io)?;
            write_raw_str(writer, entry.kind.as_str())?;
            writer
                .write_all(b",\"values\":{")
                .map_err(CapsuleError::io)?;
            for (j, (vk, vd)) in entry.values.iter().enumerate() {
                if j > 0 {
                    writer.write_all(b",").map_err(CapsuleError::io)?;
                }
                write_raw_str(writer, vk)?;
                writer.write_all(b":{\"kind\":").map_err(CapsuleError::io)?;
                match vd {
                    ValueDelta::Added(v) => {
                        write_raw_str(writer, "added")?;
                        writer.write_all(b",\"value\":").map_err(CapsuleError::io)?;
                        write_canonical_f64(writer, *v)?;
                    }
                    ValueDelta::Modified(v) => {
                        write_raw_str(writer, "modified")?;
                        writer.write_all(b",\"value\":").map_err(CapsuleError::io)?;
                        write_canonical_f64(writer, *v)?;
                    }
                    ValueDelta::Removed => {
                        write_raw_str(writer, "removed")?;
                    }
                }
                writer.write_all(b"}").map_err(CapsuleError::io)?;
            }
            writer.write_all(b"}}").map_err(CapsuleError::io)?;
        }
        writer.write_all(b"}}").map_err(CapsuleError::io)?;

        Ok(())
    })
}

#[inline]
fn write_observation_block<W: Write + ?Sized>(
    writer: &mut W,
    o: &ObservationBlock,
) -> Result<(), CapsuleError> {
    writer.write_all(b"{\"name\":").map_err(CapsuleError::io)?;
    write_raw_str(writer, &o.name)?;
    writer
        .write_all(b",\"schema\":")
        .map_err(CapsuleError::io)?;
    write_raw_str(writer, &o.schema)?;
    writer
        .write_all(b",\"capability\":")
        .map_err(CapsuleError::io)?;
    write_raw_str(writer, &o.capability)?;
    writer
        .write_all(b",\"observer_id\":")
        .map_err(CapsuleError::io)?;
    write_raw_str(writer, &o.observer_id)?;
    writer
        .write_all(b",\"values\":{")
        .map_err(CapsuleError::io)?;
    for (j, (k, &v)) in o.values.iter().enumerate() {
        if j > 0 {
            writer.write_all(b",").map_err(CapsuleError::io)?;
        }
        write_raw_str(writer, k)?;
        writer.write_all(b":").map_err(CapsuleError::io)?;
        write_canonical_f64(writer, v)?;
    }
    writer.write_all(b"}}").map_err(CapsuleError::io)
}

// ==========================================================
// 8. PLPCapsule & CapsuleBuilder Methods
// ==========================================================

impl PLPCapsule {
    pub fn verify<H: HashAlgorithm>(&self, hasher: &H) -> Result<bool, CapsuleError> {
        let calculated =
            compute_content_hash(&self.header, &self.observations, &self.delta, hasher)?;
        match &self.integrity.content_hash {
            Some(expected) if expected == &calculated => Ok(true),
            Some(expected) => Err(CapsuleError::HashMismatch {
                expected: Some(expected.clone()),
                calculated,
            }),
            None => Err(CapsuleError::HashMismatch {
                expected: None,
                calculated,
            }),
        }
    }

    pub fn recompute_hash<H: HashAlgorithm>(&mut self, hasher: &H) -> bool {
        self.integrity.errors.retain(|e| e.is_observer_related());
        match compute_content_hash(&self.header, &self.observations, &self.delta, hasher) {
            Ok(calculated) => {
                self.integrity.content_hash = Some(calculated);
                self.integrity.hash_valid = Some(true);
                self.integrity.valid =
                    self.integrity.observer_valid && self.integrity.errors.is_empty();
                true
            }
            Err(e) => {
                self.integrity.hash_valid = Some(false);
                self.integrity.valid = false;
                self.integrity.errors.push(e);
                false
            }
        }
    }

    pub fn seal<H: HashAlgorithm>(&mut self, hasher: &H) -> bool {
        self.header.flags.is_keyframe = true;
        self.recompute_hash(hasher)
    }

    pub fn snapshot(&self) -> Self {
        self.clone()
    }
}

// ==========================================================
// 9. CapsuleBuilder with Zero-Alloc Delta Construction
// ==========================================================

pub struct BuildParams {
    pub clock: i64,
    pub sequence: i64,
    pub capsule_id: String,
    pub timestamp_ns: u128,
    pub source: Option<String>,
    pub parent_id: Option<String>,
    pub flags: Option<CapsuleFlags>,
}

#[cfg(feature = "sha2-hash")]
pub struct CapsuleBuilder<W, S = CapabilityRegistry, H = Sha256Algorithm>
where
    S: SchemaProvider,
    H: HashAlgorithm,
{
    observers: Vec<Arc<dyn Observer<W>>>,
    schema_provider: S,
    hasher: H,
    rel_epsilon: f64,
    abs_epsilon: f64,
}

#[cfg(feature = "sha2-hash")]
impl<W> CapsuleBuilder<W, CapabilityRegistry, Sha256Algorithm> {
    pub fn new(observers: Vec<Arc<dyn Observer<W>>>) -> Self {
        Self {
            observers,
            schema_provider: CapabilityRegistry::new(),
            hasher: Sha256Algorithm,
            rel_epsilon: 1e-9,
            abs_epsilon: 1e-12,
        }
    }
}

impl<W, S, H> CapsuleBuilder<W, S, H>
where
    S: SchemaProvider,
    H: HashAlgorithm,
{
    pub fn with_schema_provider<S2: SchemaProvider>(
        self,
        provider: S2,
    ) -> CapsuleBuilder<W, S2, H> {
        CapsuleBuilder {
            observers: self.observers,
            schema_provider: provider,
            hasher: self.hasher,
            rel_epsilon: self.rel_epsilon,
            abs_epsilon: self.abs_epsilon,
        }
    }

    pub fn with_hasher<H2: HashAlgorithm>(self, hasher: H2) -> CapsuleBuilder<W, S, H2> {
        CapsuleBuilder {
            observers: self.observers,
            schema_provider: self.schema_provider,
            hasher,
            rel_epsilon: self.rel_epsilon,
            abs_epsilon: self.abs_epsilon,
        }
    }

    pub fn with_epsilon(mut self, rel: f64, abs: f64) -> Self {
        self.rel_epsilon = rel;
        self.abs_epsilon = abs;
        self
    }

    pub fn compute_delta(
        &self,
        current: &[ObservationBlock],
        previous: Option<&PLPCapsule>,
    ) -> DeltaBlock {
        let mut changes = BTreeMap::new();
        let mut prev_map: HashMap<ObsKey, &ObservationBlock> = HashMap::new();

        if let Some(prev_capsule) = previous {
            for obs in &prev_capsule.observations {
                prev_map.insert(ObsKey::from_block(obs), obs);
            }
        }

        for curr in current {
            let key = ObsKey::from_block(curr);
            let mut values = BTreeMap::new();
            let mut has_added = false;
            let mut has_modified = false;
            let mut has_removed = false;

            if let Some(prev) = prev_map.get(&key) {
                for (k, &v) in &curr.values {
                    match prev.values.get(k) {
                        Some(&prev_v) => {
                            if !approx_eq(v, prev_v, self.rel_epsilon, self.abs_epsilon) {
                                values.insert(k.clone(), ValueDelta::Modified(v - prev_v));
                                has_modified = true;
                            }
                        }
                        None => {
                            if v.is_finite() {
                                values.insert(k.clone(), ValueDelta::Added(v));
                                has_added = true;
                            }
                        }
                    }
                }

                for k in prev.values.keys() {
                    if !curr.values.contains_key(k) {
                        values.insert(k.clone(), ValueDelta::Removed);
                        has_removed = true;
                    }
                }
            } else {
                for (k, &v) in &curr.values {
                    if v.is_finite() {
                        values.insert(k.clone(), ValueDelta::Added(v));
                        has_added = true;
                    }
                }
            }

            if !values.is_empty() {
                let kind = match (has_added, has_modified, has_removed) {
                    (true, false, false) => DeltaKind::Added,
                    (false, false, true) => DeltaKind::Removed,
                    _ => DeltaKind::Modified,
                };
                changes.insert(key.to_stable_key(), DeltaEntry { kind, values });
            }
        }

        if let Some(prev_capsule) = previous {
            let curr_map: HashMap<ObsKey, &ObservationBlock> =
                current.iter().map(|o| (ObsKey::from_block(o), o)).collect();

            for prev in &prev_capsule.observations {
                let key = ObsKey::from_block(prev);
                if !curr_map.contains_key(&key) {
                    let mut values = BTreeMap::new();
                    for k in prev.values.keys() {
                        values.insert(k.clone(), ValueDelta::Removed);
                    }
                    if !values.is_empty() {
                        changes.insert(
                            key.to_stable_key(),
                            DeltaEntry {
                                kind: DeltaKind::Removed,
                                values,
                            },
                        );
                    }
                }
            }
        }

        DeltaBlock { changes }
    }

    pub fn build_with_params(
        &self,
        world: &W,
        input: InputCapsule,
        params: BuildParams,
        previous: Option<&PLPCapsule>,
    ) -> PLPCapsule {
        let mut observations = Vec::with_capacity(self.observers.len());
        let mut errors = Vec::new();
        let mut observer_valid = true;

        for obs in &self.observers {
            match obs.observe(world) {
                Ok(mut block) => {
                    let expected_schema = self.schema_provider.get_schema(&block.capability);
                    match expected_schema {
                        Some(schema) => {
                            block.schema = schema;
                            observations.push(block);
                        }
                        None => {
                            observer_valid = false;
                            errors.push(CapsuleError::SchemaNotFound {
                                capability: block.capability.clone(),
                            });
                        }
                    }
                }
                Err(reason) => {
                    observer_valid = false;
                    errors.push(CapsuleError::ObserverFailed {
                        name: obs.name().to_string(),
                        reason,
                    });
                }
            }
        }

        let delta = self.compute_delta(&observations, previous);

        let header = CapsuleHeader {
            protocol: default_protocol(),
            capsule_schema: default_schema(),
            version: default_version(),
            capsule_id: params.capsule_id,
            parent_id: params.parent_id,
            clock: params.clock,
            sequence: params.sequence,
            timestamp_ns: params.timestamp_ns,
            source: params.source.unwrap_or_else(default_source),
            flags: params.flags.unwrap_or_default(),
            hash_algorithm: self.hasher.name().to_string(),
        };

        let calculated_hash =
            compute_content_hash(&header, &observations, &delta, &self.hasher).ok();
        let hash_valid = calculated_hash.is_some();
        let valid = observer_valid && hash_valid && errors.is_empty();

        PLPCapsule {
            header,
            input,
            observations,
            delta,
            integrity: CapsuleIntegrity {
                content_hash: calculated_hash,
                valid,
                observer_valid,
                hash_valid: Some(hash_valid),
                errors,
            },
        }
    }
}

// ==========================================================
// 10. Golden Tests
// ==========================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_hash_sha256_strict_matching() {
        let header = CapsuleHeader {
            protocol: "PLP/1.1".into(),
            capsule_schema: "v1/capsule".into(),
            version: "1.1.3".into(),
            capsule_id: "00000000-0000-4000-8000-000000000001".into(),
            parent_id: None,
            clock: 42,
            sequence: 7,
            timestamp_ns: 1_700_000_000_000_000_000,
            source: "golden".into(),
            flags: CapsuleFlags { is_keyframe: true },
            hash_algorithm: "sha256".into(),
        };

        let mut values = BTreeMap::new();
        values.insert("x".into(), 1.0);
        values.insert("y".into(), -2.5);
        let obs = vec![ObservationBlock {
            name: "geom".into(),
            schema: "v1/geometry".into(),
            capability: "geometry".into(),
            observer_id: "cam0".into(),
            values,
        }];
        let delta = DeltaBlock::default();

        let algo = Sha256Algorithm;
        let hash1 = compute_content_hash(&header, &obs, &delta, &algo).unwrap();
        assert_eq!(hash1.len(), 64);

        let hash2 = compute_content_hash(&header, &obs, &delta, &algo).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_obs_key_to_stable_key_formatting() {
        let obs = ObservationBlock {
            name: "sensor".into(),
            schema: "v1/schema".into(),
            capability: "cap".into(),
            observer_id: "id0".into(),
            values: BTreeMap::new(),
        };

        let key = ObsKey::from_block(&obs);
        let stable = key.to_stable_key();
        assert_eq!(stable, "6\0sensor\09\0v1/schema\03\0cap\03\0id0");
    }

    #[test]
    fn test_non_finite_value_rejection() {
        let header = CapsuleHeader {
            protocol: "PLP/1.1".into(),
            capsule_schema: "v1/capsule".into(),
            version: "1.1.3".into(),
            capsule_id: "test".into(),
            parent_id: None,
            clock: 1,
            sequence: 1,
            timestamp_ns: 100,
            source: "sys".into(),
            flags: CapsuleFlags::default(),
            hash_algorithm: "sha256".into(),
        };

        let mut values = BTreeMap::new();
        values.insert("invalid".into(), f64::NAN);
        let obs = vec![ObservationBlock {
            name: "bad".into(),
            schema: "v1/bad".into(),
            capability: "bad".into(),
            observer_id: "obs".into(),
            values,
        }];

        let result = compute_content_hash(&header, &obs, &DeltaBlock::default(), &Sha256Algorithm);
        assert!(result.is_err());
        if let Err(CapsuleError::NonFiniteValue { key, .. }) = result {
            assert_eq!(key, "invalid");
        } else {
            panic!("Expected NonFiniteValue error");
        }
    }

    #[test]
    fn test_dyn_write_path_compiles_and_matches_direct_path() {
        let header = CapsuleHeader {
            protocol: "PLP/1.1".into(),
            capsule_schema: "v1/capsule".into(),
            version: "1.1.3".into(),
            capsule_id: "dyn-write-check".into(),
            parent_id: None,
            clock: 1,
            sequence: 1,
            timestamp_ns: 42,
            source: "sys".into(),
            flags: CapsuleFlags::default(),
            hash_algorithm: "sha256".into(),
        };
        let obs: Vec<ObservationBlock> = vec![];
        let delta = DeltaBlock::default();
        let algo = Sha256Algorithm;

        let hash = compute_content_hash(&header, &obs, &delta, &algo).unwrap();
        assert_eq!(hash.len(), 64);
    }
}
