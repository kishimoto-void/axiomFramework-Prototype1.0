//! PSS — Problem / input normalization (Prototype 1.0 Phase 1)
//!
//! Responsibility: normalize raw input for PLP.
//! Does **not** invent semantics.
//!
//! 実験は忠実に実際行って

use serde::{Deserialize, Serialize};

/// Normalized input handed to PLP.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NormalizedInput {
    /// Trimmed text (ends only; internal whitespace preserved except strip of CR).
    pub text: String,
    /// Heuristic language hint: `"ja"` | `"en"` | other future tags.
    pub language_hint: Option<String>,
    /// Always `"utf-8"` in Prototype 1.0.
    pub encoding: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PssError {
    #[error("empty input after normalization")]
    EmptyInput,
}

/// Detect JP script (hiragana / katakana / CJK unified).
pub fn detect_language_hint(text: &str) -> &'static str {
    if text.chars().any(|c| {
        let o = c as u32;
        (0x3040..=0x30FF).contains(&o) || (0x4E00..=0x9FFF).contains(&o)
    }) {
        "ja"
    } else {
        "en"
    }
}

/// Normalize raw input.
///
/// Policy (Prototype 1.0):
/// - UTF-8 string in, UTF-8 out
/// - Trim leading/trailing Unicode whitespace
/// - Strip `\r` (CRLF → LF family)
/// - Reject empty after trim
/// - Attach language_hint (heuristic only — not truth)
pub fn normalize(raw: &str) -> Result<NormalizedInput, PssError> {
    let text: String = raw.chars().filter(|c| *c != '\r').collect();
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err(PssError::EmptyInput);
    }
    let language_hint = Some(detect_language_hint(&text).to_string());
    Ok(NormalizedInput {
        text,
        language_hint,
        encoding: "utf-8".into(),
    })
}

/// Normalize with an explicit language override (still no semantics).
pub fn normalize_with_language(
    raw: &str,
    language_hint: impl Into<String>,
) -> Result<NormalizedInput, PssError> {
    let mut n = normalize(raw)?;
    n.language_hint = Some(language_hint.into());
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty() {
        assert_eq!(normalize("   ").unwrap_err(), PssError::EmptyInput);
        assert_eq!(normalize("").unwrap_err(), PssError::EmptyInput);
    }

    #[test]
    fn trims_ends() {
        let n = normalize("  hello  ").unwrap();
        assert_eq!(n.text, "hello");
        assert_eq!(n.encoding, "utf-8");
    }

    #[test]
    fn strips_cr() {
        let n = normalize("a\r\nb").unwrap();
        assert_eq!(n.text, "a\nb");
    }

    #[test]
    fn detects_ja() {
        let n = normalize("猫が机の上で寝ている").unwrap();
        assert_eq!(n.language_hint.as_deref(), Some("ja"));
    }

    #[test]
    fn detects_en() {
        let n = normalize("Enable review bot").unwrap();
        assert_eq!(n.language_hint.as_deref(), Some("en"));
    }

    #[test]
    fn override_language() {
        let n = normalize_with_language("hello", "zz").unwrap();
        assert_eq!(n.language_hint.as_deref(), Some("zz"));
    }
}
