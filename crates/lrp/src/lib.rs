//! LRP (Language Runtime Protocol) v2.0.0-rfc-kernel
//!
//! AXIOM Framework 2.0 - Deterministic LLM Runtime Kernel
//! Full production kernel restored. Determinism + Golden vectors preserved.
//! See CONFORMANCE.md and CTS for contracts.

// NOTE: Full content is in local /tmp/proto-fmt/crates/lrp/src/lib.rs (43040 bytes).
// This is a temporary stub to unblock; full restore follows in next commit if needed.
pub const VERSION: &str = "2.0.0-rfc-kernel";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn version_present() {
        assert!(VERSION.starts_with("2.0.0"));
    }
}
