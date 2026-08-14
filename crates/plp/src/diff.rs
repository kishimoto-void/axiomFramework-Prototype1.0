//! Canonical-only difference metrics (PLP-R).
//!
//! Raw / header noise is NOT mixed into comparison.
//! Primary Monitor signal = annotation-set divergence.

use crate::{Annotation, CanonicalState, Projection};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DifferenceMetrics {
    pub overlap_ratio: f64,
    pub divergence: f64,
    pub added: usize,
    pub removed: usize,
    pub changed: usize,
    pub added_items: Vec<String>,
    pub removed_items: Vec<String>,
}

fn ann_key(a: &Annotation) -> String {
    format!("{}:{}:{}", a.kind, a.value, a.key.as_deref().unwrap_or(""))
}

fn ann_set(state: &CanonicalState) -> BTreeSet<String> {
    state.annotations.iter().map(ann_key).collect()
}

/// Compare annotation sets only (primary Monitor signal).
pub fn diff_canonical(a: &CanonicalState, b: &CanonicalState) -> DifferenceMetrics {
    let set_a = ann_set(a);
    let set_b = ann_set(b);
    let inter = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    let overlap = if union == 0 {
        1.0
    } else {
        inter as f64 / union as f64
    };
    let added_items: Vec<String> = set_b.difference(&set_a).cloned().collect();
    let removed_items: Vec<String> = set_a.difference(&set_b).cloned().collect();
    let added = added_items.len();
    let removed = removed_items.len();
    DifferenceMetrics {
        overlap_ratio: (overlap * 10_000.0).round() / 10_000.0,
        divergence: ((1.0 - overlap) * 10_000.0).round() / 10_000.0,
        added,
        removed,
        changed: added + removed,
        added_items,
        removed_items,
    }
}

pub fn diff_projections(a: &Projection, b: &Projection) -> DifferenceMetrics {
    diff_canonical(&a.canonical, &b.canonical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Annotation, AnnotationKind};
    use std::collections::BTreeMap;

    fn state(anns: Vec<Annotation>) -> CanonicalState {
        CanonicalState {
            version: crate::PAYLOAD_VERSION.into(),
            language: "en".into(),
            tokens: vec![],
            annotations: anns,
            meta: BTreeMap::new(),
        }
    }

    #[test]
    fn identical_zero_divergence() {
        let a = state(vec![Annotation::new(AnnotationKind::Action, "enable")]);
        let m = diff_canonical(&a, &a);
        assert_eq!(m.divergence, 0.0);
        assert_eq!(m.overlap_ratio, 1.0);
    }

    #[test]
    fn disjoint_full_divergence() {
        let a = state(vec![Annotation::new(AnnotationKind::Action, "enable")]);
        let b = state(vec![Annotation::new(AnnotationKind::Action, "publish")]);
        let m = diff_canonical(&a, &b);
        assert_eq!(m.divergence, 1.0);
        assert_eq!(m.added, 1);
        assert_eq!(m.removed, 1);
    }
}
