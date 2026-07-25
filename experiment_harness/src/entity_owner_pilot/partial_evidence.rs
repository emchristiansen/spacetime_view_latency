//! The evidence an attempt had already produced when it failed.

use anyhow::{ensure, Result};
use serde::Serialize;

use crate::entity_owner_pilot::pilot_params::GLOBAL_ROW_LADDER_LEN;
use crate::entity_owner_pilot::rung_evidence::RungEvidence;

/// The rungs an attempt completed before it failed — strictly fewer than the whole ladder.
///
/// Deliberately a different type from
/// [`EvidenceArtifact`](super::evidence_artifact::EvidenceArtifact) rather than the same type with a
/// short length: only a complete walk can seal an artifact and only a strict prefix can seal this,
/// so the two cannot be assembled from each other's data and "select one *complete* attempt per
/// logical slot" is enforced by the types rather than by a length check someone must remember.
///
/// An empty value is a real state — the attempt reached a measurable configuration and died before
/// the first rung finished — not an error.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct PartialEvidence {
    rungs: Vec<RungEvidence>,
}

impl PartialEvidence {
    /// Seal the rungs completed before a failure, failing loud unless they are a strict ascending
    /// prefix of the ladder.
    pub(crate) fn sealed(rungs: Vec<RungEvidence>) -> Result<Self> {
        ensure!(
            rungs.len() < GLOBAL_ROW_LADDER_LEN,
            "partial evidence must be a strict prefix of the {GLOBAL_ROW_LADDER_LEN}-rung ladder, \
             got {} rungs — a complete walk seals an EvidenceArtifact, not partial evidence",
            rungs.len(),
        );
        for (position, evidence) in rungs.iter().enumerate() {
            ensure!(
                evidence.rung().get() == position,
                "partial evidence must be an ascending ladder prefix; position {position} holds \
                 rung {}",
                evidence.rung().get(),
            );
        }
        Ok(Self { rungs })
    }
}
