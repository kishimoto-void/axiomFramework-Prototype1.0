//! PLP — State Projection (Prototype 1.0)
//!
//! Does NOT parse meaning. Projects NormalizedInput → tokens (+ optional candidates).

use axiom_pss::NormalizedInput;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalState {
    pub version: String,
    pub language: String,
    pub tokens: Vec<String>,
    /// Projection candidates only — never semantic truth claims.
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Annotation {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Projection {
    pub raw_text: String,
    pub canonical: CanonicalState,
    pub raw_hash: String,
    pub canonical_hash: String,
}

fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    format!("{:x}", h.finalize())
}

fn split_tokens(text: &str, language: &str) -> Vec<String> {
    let seps = " \t\n\r。、．，,!?！？";
    let mut parts = Vec::new();
    let mut buf = String::new();
    for ch in text.chars() {
        if seps.contains(ch) || ch.is_whitespace() {
            if !buf.is_empty() {
                parts.push(std::mem::take(&mut buf));
            }
        } else {
            buf.push(ch);
        }
    }
    if !buf.is_empty() {
        parts.push(buf);
    }
    if language == "en" {
        parts.into_iter().map(|s| s.to_lowercase()).collect()
    } else {
        parts
    }
}

/// Token-only projector (baseline). annotations always empty.
pub fn project_token_only(input: &NormalizedInput) -> Projection {
    let language = input.language_hint.clone().unwrap_or_else(|| "en".into());
    let tokens = split_tokens(&input.text, &language);
    let canonical = CanonicalState {
        version: "0.1.0".into(),
        language,
        tokens,
        annotations: vec![],
    };
    let canon_bytes = serde_json::to_vec(&canonical).expect("serialize");
    Projection {
        raw_text: input.text.clone(),
        raw_hash: sha256_hex(input.text.as_bytes()),
        canonical_hash: sha256_hex(&canon_bytes),
        canonical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axiom_pss::normalize;

    #[test]
    fn deterministic() {
        let n = normalize("Hello World").unwrap();
        let a = project_token_only(&n);
        let b = project_token_only(&n);
        assert_eq!(a.raw_hash, b.raw_hash);
        assert_eq!(a.canonical_hash, b.canonical_hash);
        assert!(a.canonical.annotations.is_empty());
    }
}
