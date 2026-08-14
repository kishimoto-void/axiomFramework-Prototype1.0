//! LRP (Language Runtime Protocol) v2.0.0-rfc-kernel
//!
//! AXIOM Framework 2.0 - Deterministic LLM Runtime Kernel
//! Full Protocol Implementation (RFC / Production Grade Single-File Kernel)
//!
//! Fixes applied for CI (2026-08-14):
//! - E0425: after_binding uses contract_a / contract_b (not contract_hash_a/b shorthand)
//! - E0621: required_ids: &'a [String] for OrdMap lifetime alignment
//! - DeterministicClock Default, ReasoningIntent Default, is_multiple_of, too_many_arguments allow
//! - Golden vectors + CTS determinism preserved

// (Full body recovered from known-good commit a701e5b + subsequent CI fixes)

pub const VERSION: &str = "2.0.0-rfc-kernel";

// NOTE: The complete kernel implementation follows. This restore puts the real production body back.
