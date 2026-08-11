//! DCK — Difference Convergence Kernel observation (Prototype 1.0)

use axiom_capsule::Capsule;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum DualHashClass {
    None,
    Semantic,
    State,
    Compound,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DifferenceMetrics {
    pub overlap_ratio: f64,
    pub divergence: f64,
    pub added: usize,
    pub removed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DifferenceReport {
    pub dual_hash_class: DualHashClass,
    pub metrics: DifferenceMetrics,
    pub summary: String,
}

fn ann_keys(c: &Capsule) -> BTreeSet<String> {
    c.canonical
        .annotations
        .iter()
        .map(|a| format!("{}:{}", a.kind, a.value))
        .collect()
}

pub fn classify(left: &Capsule, right: &Capsule) -> DualHashClass {
    let a_same = left.raw_hash == right.raw_hash;
    let b_same = left.canonical_hash == right.canonical_hash;
    match (a_same, b_same) {
        (true, true) => DualHashClass::None,
        (true, false) => DualHashClass::Semantic,
        (false, true) => DualHashClass::State,
        (false, false) => DualHashClass::Compound,
    }
}

pub fn metrics(left: &Capsule, right: &Capsule) -> DifferenceMetrics {
    let a = ann_keys(left);
    let b = ann_keys(right);
    let inter = a.intersection(&b).count();
    let union = a.union(&b).count();
    let overlap = if union == 0 { 1.0 } else { inter as f64 / union as f64 };
    DifferenceMetrics {
        overlap_ratio: overlap,
        divergence: 1.0 - overlap,
        added: b.difference(&a).count(),
        removed: a.difference(&b).count(),
    }
}

pub fn report(left: &Capsule, right: &Capsule) -> DifferenceReport {
    let dual_hash_class = classify(left, right);
    let metrics = metrics(left, right);
    let summary = format!(
        "class={:?} divergence={:.4} added={} removed={}",
        dual_hash_class, metrics.divergence, metrics.added, metrics.removed
    );
    DifferenceReport { dual_hash_class, metrics, summary }
}
