//! Monitor — contract / state divergence decisions (PLP-R).
//!
//! Primary signal: Canonical annotation divergence (not header noise).
//! Aligns with ObserverVerdict in Round Consensus Protocol.

use crate::diff::DifferenceMetrics;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum MonitorDecisionKind {
    Continue,
    AskUser,
    Abort,
}

impl MonitorDecisionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Continue => "Continue",
            Self::AskUser => "AskUser",
            Self::Abort => "Abort",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonitorDecision {
    pub kind: MonitorDecisionKind,
    pub reason: Option<String>,
    pub candidates: Option<Vec<String>>,
    pub divergence: f64,
    pub integrity_ok: bool,
}

/// Rules:
/// - integrity failure → Abort
/// - annotation divergence == 0 → Continue (header noise ignored)
/// - divergence > threshold → AskUser (default: any divergence)
pub fn monitor_decide(
    metrics: &DifferenceMetrics,
    integrity_ok: bool,
    divergence_threshold: f64,
) -> MonitorDecision {
    if !integrity_ok {
        return MonitorDecision {
            kind: MonitorDecisionKind::Abort,
            reason: Some("integrity check failed".into()),
            candidates: None,
            divergence: metrics.divergence,
            integrity_ok: false,
        };
    }
    if metrics.divergence <= divergence_threshold {
        return MonitorDecision {
            kind: MonitorDecisionKind::Continue,
            reason: Some("canonical annotations identical or below threshold".into()),
            candidates: None,
            divergence: metrics.divergence,
            integrity_ok: true,
        };
    }
    let mut candidates = metrics.added_items.clone();
    candidates.extend(metrics.removed_items.iter().cloned());
    MonitorDecision {
        kind: MonitorDecisionKind::AskUser,
        reason: Some(format!("canonical divergence={}", metrics.divergence)),
        candidates: Some(candidates),
        divergence: metrics.divergence,
        integrity_ok: true,
    }
}

/// Default: ask on any positive divergence.
pub fn monitor_decide_default(metrics: &DifferenceMetrics, integrity_ok: bool) -> MonitorDecision {
    monitor_decide(metrics, integrity_ok, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::DifferenceMetrics;

    fn m(div: f64) -> DifferenceMetrics {
        DifferenceMetrics {
            overlap_ratio: 1.0 - div,
            divergence: div,
            added: if div > 0.0 { 1 } else { 0 },
            removed: if div > 0.0 { 1 } else { 0 },
            changed: if div > 0.0 { 2 } else { 0 },
            added_items: if div > 0.0 {
                vec!["ACTION:publish".into()]
            } else {
                vec![]
            },
            removed_items: if div > 0.0 {
                vec!["ACTION:enable".into()]
            } else {
                vec![]
            },
        }
    }

    #[test]
    fn continue_on_zero() {
        let d = monitor_decide_default(&m(0.0), true);
        assert_eq!(d.kind, MonitorDecisionKind::Continue);
    }

    #[test]
    fn ask_on_divergence() {
        let d = monitor_decide_default(&m(1.0), true);
        assert_eq!(d.kind, MonitorDecisionKind::AskUser);
        assert!(d.candidates.as_ref().unwrap().len() >= 1);
    }

    #[test]
    fn abort_on_integrity() {
        let d = monitor_decide_default(&m(0.0), false);
        assert_eq!(d.kind, MonitorDecisionKind::Abort);
    }
}
