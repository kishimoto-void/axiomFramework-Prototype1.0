//! PSS — Problem / input normalization (Prototype 1.0)
//!
//! Responsibility: normalize raw input. No semantics.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedInput {
    pub text: String,
    pub language_hint: Option<String>,
    pub encoding: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PssError {
    #[error("empty input")]
    EmptyInput,
}

/// Minimal normalizer: trim ends, reject empty, heuristic language hint.
pub fn normalize(raw: &str) -> Result<NormalizedInput, PssError> {
    let text = raw.trim().to_string();
    if text.is_empty() {
        return Err(PssError::EmptyInput);
    }
    let language_hint = if text.chars().any(|c| {
        let o = c as u32;
        (0x3040..=0x30FF).contains(&o) || (0x4E00..=0x9FFF).contains(&o)
    }) {
        Some("ja".into())
    } else {
        Some("en".into())
    };
    Ok(NormalizedInput {
        text,
        language_hint,
        encoding: "utf-8".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty() {
        assert!(normalize("   ").is_err());
    }

    #[test]
    fn detects_ja() {
        let n = normalize("猫がいる").unwrap();
        assert_eq!(n.language_hint.as_deref(), Some("ja"));
    }
}
