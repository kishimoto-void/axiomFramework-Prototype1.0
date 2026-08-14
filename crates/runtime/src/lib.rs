//! Runtime — wire PSS → PLP → Capsule → ACP → DCK (Prototype 1.0)

use axiom_acp::{contract_from_text, seal, Contract, SealedCapsule};
use axiom_capsule::Capsule;
use axiom_dck::{report, DifferenceReport};
use axiom_plp::{project_token_only, PlpError, ProjectOptions};
use axiom_pss::{normalize, NormalizedInput, PssError};

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error(transparent)]
    Pss(#[from] PssError),
    #[error(transparent)]
    Plp(#[from] PlpError),
}

pub struct Pipeline {
    pub contract: Contract,
}

impl Pipeline {
    pub fn new(contract_text: &str) -> Self {
        Self {
            contract: contract_from_text(contract_text),
        }
    }

    pub fn project_sealed(
        &self,
        raw: &str,
        capsule_id: &str,
    ) -> Result<SealedCapsule, RuntimeError> {
        let norm: NormalizedInput = normalize(raw)?;
        let opts = ProjectOptions::with_id(capsule_id);
        let projection = project_token_only(&norm, opts)?;
        let capsule = Capsule::from_projection(capsule_id, projection);
        Ok(seal(&self.contract, capsule))
    }

    pub fn compare(&self, left: &SealedCapsule, right: &SealedCapsule) -> DifferenceReport {
        report(&left.capsule, &right.capsule)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_to_end_same_input() {
        let pipe = Pipeline::new("goal: demo\nsafety: no secrets");
        let a = pipe.project_sealed("hello world", "c1").unwrap();
        let b = pipe.project_sealed("hello world", "c2").unwrap();
        let r = pipe.compare(&a, &b);
        // Same raw text → same HashA class; different capsule_id may affect HashB
        // but DCK annotation divergence baseline for identical token projection is 0
        assert_eq!(r.metrics.divergence, 0.0);
    }
}
