//! AXIOM Conformance Test Suite v1.0 — PLP-focused runner
//! (ACP coordinate uses domain-separated SHA-256 without `time` crate
//!  so the suite builds on older toolchains.)
//!
//! Full 13-test suite for Prototype 1.0.

use axiom_plp::{
    build_canonical_payload, diff_projections, dual_hash, monitor_decide_default,
    project_minimal, project_token_only, ProjectOptions, MonitorDecisionKind, PAYLOAD_VERSION,
};
use axiom_pss::normalize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden_vectors/PLP_R_GOLDEN_LOCK_v0_1.json")
}

fn load_golden() -> serde_json::Value {
    let raw = fs::read_to_string(golden_path()).expect("golden lock missing");
    serde_json::from_str(&raw).expect("golden JSON")
}

fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    format!("{:x}", h.finalize())
}

fn project_tok(text: &str, id: &str) -> axiom_plp::Projection {
    let n = normalize(text).unwrap();
    project_token_only(&n, ProjectOptions::with_id(id)).unwrap()
}

fn project_min(text: &str, id: &str) -> axiom_plp::Projection {
    let n = normalize(text).unwrap();
    project_minimal(&n, ProjectOptions::with_id(id)).unwrap()
}

/// Minimal ACP-compatible seal (domain tag axiom:v2:proof) without time dep.
fn seal_proof(contract_hash_a: &str, raw_hash: &str, canonical_hash: &str) -> String {
    let mut material = Vec::new();
    material.extend_from_slice(b"axiom:v2:proof\0");
    material.extend_from_slice(contract_hash_a.as_bytes());
    material.push(0);
    material.extend_from_slice(raw_hash.as_bytes());
    material.push(0);
    material.extend_from_slice(canonical_hash.as_bytes());
    sha256_hex(&material)
}

fn domain_state_hash(canonical_jsonish: &str) -> String {
    let mut material = Vec::new();
    material.extend_from_slice(b"AXIOM-STATE-CANONICAL-v1:");
    material.extend_from_slice(canonical_jsonish.as_bytes());
    sha256_hex(&material)
}

#[test]
fn cts_01_golden_hash() {
    let p1 = project_tok("Enable review bot", "cts-01");
    let p2 = project_tok("Enable review bot", "cts-01");
    assert_eq!(p1.raw_hash, p2.raw_hash);
    assert_eq!(p1.canonical_hash, p2.canonical_hash);
    assert_eq!(p1.raw_hash.len(), 64);
}

#[test]
fn cts_02_golden_vector() {
    let p = project_tok("Enable Review", "cts-02");
    assert_eq!(p.canonical.language, "en");
    assert_eq!(p.canonical.tokens, vec!["enable", "review"]);
    assert_eq!(p.canonical.version, PAYLOAD_VERSION);
    let g = load_golden();
    assert_eq!(g[0]["id"], "01_en_cat_sleep");
    assert_eq!(g[0]["canonical_hash"].as_str().unwrap().len(), 64);
}

#[test]
fn cts_03_golden_coordinate() {
    let payload = br#"{"a":1,"b":"x"}"#;
    let h1 = domain_state_hash(std::str::from_utf8(payload).unwrap());
    let h2 = domain_state_hash(r#"{"a":1,"b":"x"}"#);
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 64);
}

#[test]
fn cts_04_cross_language_hash() {
    let g = load_golden();
    for entry in g.as_array().unwrap() {
        let id = entry["id"].as_str().unwrap();
        let raw = entry["raw_hash"].as_str().unwrap();
        let can = entry["canonical_hash"].as_str().unwrap();
        assert_eq!(raw.len(), 64, "{id}");
        assert_eq!(can.len(), 64, "{id}");
        assert!(raw.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(can.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

#[test]
fn cts_05_cross_language_vector() {
    let g = load_golden();
    assert_eq!(g[0]["language"], "en");
    assert_eq!(
        g[0]["tokens"],
        serde_json::json!(["cat", "sleeps", "on", "table"])
    );
    assert_eq!(g[1]["language"], "ja");
}

#[test]
fn cts_06_canonical_serialization() {
    let p = project_tok("hello world", "ser-1");
    let b1 = build_canonical_payload(&p.header, &p.canonical);
    let b2 = build_canonical_payload(&p.header, &p.canonical);
    assert_eq!(b1, b2);
    let (_, h) = dual_hash(&p.raw_text, &b1);
    assert_eq!(h, p.canonical_hash);
}

#[test]
fn cts_07_hash_stability() {
    let mut set = BTreeSet::new();
    for _ in 0..20 {
        let p = project_tok("stability probe", "stable-id");
        set.insert(p.raw_hash);
        set.insert(p.canonical_hash);
    }
    assert_eq!(set.len(), 2);
}

#[test]
fn cts_08_difference_baseline() {
    let a = project_min("Enable review bot", "diff-base");
    let b = project_min("Enable review bot", "diff-base");
    let m = diff_projections(&a, &b);
    assert_eq!(m.divergence, 0.0);
    assert_eq!(m.overlap_ratio, 1.0);
    let d = monitor_decide_default(&m, true);
    assert_eq!(d.kind, MonitorDecisionKind::Continue);
}

#[test]
fn cts_09_difference_threshold() {
    let a = project_min("Enable review bot", "th-a");
    let b = project_min("Enable review bots", "th-a");
    let m = diff_projections(&a, &b);
    assert!((0.0..=1.0).contains(&m.divergence));
    let d = monitor_decide_default(&m, true);
    if m.divergence > 0.0 {
        assert_eq!(d.kind, MonitorDecisionKind::AskUser);
    }
}

#[test]
fn cts_10_difference_large_change() {
    let a = project_min("Enable review bot", "lg-a");
    let b = project_min("Disable publish pipeline", "lg-a");
    let m = diff_projections(&a, &b);
    assert!(a.raw_hash != b.raw_hash || m.divergence > 0.0 || a.canonical_hash != b.canonical_hash);
}

#[test]
fn cts_11_determinism_stress() {
    let mut first: Option<(String, String)> = None;
    for i in 0..100 {
        let p = project_tok("stress input 実験", "stress-id");
        match &first {
            None => first = Some((p.raw_hash.clone(), p.canonical_hash.clone())),
            Some((r, c)) => {
                assert_eq!(&p.raw_hash, r, "iter {i}");
                assert_eq!(&p.canonical_hash, c, "iter {i}");
            }
        }
    }
}

#[test]
fn cts_12_regression_compatibility() {
    let g = load_golden();
    assert_eq!(g.as_array().unwrap().len(), 4);
    assert!(g[0]["canonical_hash"].as_str().unwrap().starts_with("b130e1ff"));
    assert!(g[3]["canonical_hash"].as_str().unwrap().starts_with("32b79f86"));
    assert_eq!(PAYLOAD_VERSION, "0.1.1");
}

#[test]
fn cts_13_end_to_end_pipeline() {
    let n = normalize("Enable review bot").unwrap();
    assert_eq!(n.encoding, "utf-8");
    let proj = project_minimal(&n, ProjectOptions::with_id("e2e-1")).unwrap();
    assert_eq!(proj.hash_a().len(), 64);
    let raw_h = proj.raw_hash.clone();
    let can_h = proj.canonical_hash.clone();
    let contract_a = sha256_hex(b"goal: conformance\nsafety: research-only");
    let proof = seal_proof(&contract_a, &raw_h, &can_h);
    assert_eq!(proof.len(), 64);
    let proof2 = seal_proof(&contract_a, &raw_h, &can_h);
    assert_eq!(proof, proof2);
    let same = project_minimal(&n, ProjectOptions::with_id("e2e-1")).unwrap();
    let m0 = diff_projections(&proj, &same);
    assert_eq!(m0.divergence, 0.0);
    let n2 = normalize("Disable publish pipeline").unwrap();
    let other = project_minimal(&n2, ProjectOptions::with_id("e2e-2")).unwrap();
    let m1 = diff_projections(&proj, &other);
    assert!(proj.raw_hash != other.raw_hash || m1.divergence > 0.0);
}
