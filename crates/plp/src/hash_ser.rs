//! Deterministic hashing + canonical serialization (PLP-R Golden compatible).

use crate::{CanonicalState, ProjectionHeader};
use sha2::{Digest, Sha256};

pub fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    format!("{:x}", h.finalize())
}

fn write_raw_str(s: &str) -> String {
    let mut out = String::from("\"");
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Deterministic canonical payload bytes (header + canonical).
pub fn build_canonical_payload(header: &ProjectionHeader, canonical: &CanonicalState) -> Vec<u8> {
    let mut parts: Vec<String> = Vec::new();
    parts.push("{\"header\":{".into());
    parts.push(format!("{}:{}", write_raw_str("protocol"), write_raw_str(&header.protocol)));
    parts.push(format!(",{}:{}", write_raw_str("version"), write_raw_str(&header.version)));
    parts.push(format!(",{}:{}", write_raw_str("capsule_id"), write_raw_str(&header.capsule_id)));
    parts.push(format!(",{}:", write_raw_str("parent_id")));
    match &header.parent_id {
        None => parts.push("null".into()),
        Some(p) => parts.push(write_raw_str(p)),
    }
    parts.push(format!(",{}:{}", write_raw_str("clock"), header.clock));
    parts.push(format!(",{}:{}", write_raw_str("sequence"), header.sequence));
    parts.push(format!(
        ",{}:{}",
        write_raw_str("timestamp_ns"),
        write_raw_str(&header.timestamp_ns.to_string())
    ));
    parts.push(format!(",{}:{}", write_raw_str("source"), write_raw_str(&header.source)));
    parts.push(format!(
        ",{}:{}",
        write_raw_str("hash_algorithm"),
        write_raw_str(&header.hash_algorithm)
    ));
    parts.push("}".into());

    parts.push(",\"canonical\":{".into());
    parts.push(format!("{}:{}", write_raw_str("version"), write_raw_str(&canonical.version)));
    parts.push(format!(",{}:{}", write_raw_str("language"), write_raw_str(&canonical.language)));

    parts.push(",\"tokens\":[".into());
    for (i, t) in canonical.tokens.iter().enumerate() {
        if i > 0 {
            parts.push(",".into());
        }
        parts.push(write_raw_str(t));
    }
    parts.push("]".into());

    let mut anns = canonical.annotations.clone();
    anns.sort_by(|a, b| {
        (&a.kind, &a.value, a.key.as_deref().unwrap_or("")).cmp(&(
            &b.kind,
            &b.value,
            b.key.as_deref().unwrap_or(""),
        ))
    });

    parts.push(",\"annotations\":[".into());
    for (i, a) in anns.iter().enumerate() {
        if i > 0 {
            parts.push(",".into());
        }
        parts.push("{".into());
        parts.push(format!("{}:{}", write_raw_str("kind"), write_raw_str(&a.kind)));
        parts.push(format!(",{}:{}", write_raw_str("value"), write_raw_str(&a.value)));
        parts.push(format!(",{}:", write_raw_str("key")));
        match &a.key {
            None => parts.push("null".into()),
            Some(k) => parts.push(write_raw_str(k)),
        }
        parts.push("}".into());
    }
    parts.push("]".into());

    parts.push(",\"meta\":{".into());
    for (i, (k, v)) in canonical.meta.iter().enumerate() {
        if i > 0 {
            parts.push(",".into());
        }
        parts.push(write_raw_str(k));
        parts.push(":".into());
        parts.push(write_raw_str(v));
    }
    parts.push("}}".into());

    parts.concat().into_bytes()
}

pub fn dual_hash(raw_text: &str, payload: &[u8]) -> (String, String) {
    (sha256_hex(raw_text.as_bytes()), sha256_hex(payload))
}
