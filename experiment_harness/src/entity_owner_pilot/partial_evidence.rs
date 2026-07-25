//! The evidence an attempt had already produced when it failed.

use anyhow::{ensure, Result};
use serde::Serialize;

use crate::entity_owner_pilot::pilot_params::GLOBAL_ROW_LADDER_LEN;
use crate::entity_owner_pilot::rung_evidence::RungEvidence;

/// The rungs an attempt completed before it failed — strictly fewer than the whole ladder.
///
/// The Parked Frontier requires that a terminating error "preserve[s] partial evidence", and the
/// spec requires every failed attempt to be appended "with partial evidence and diagnostics". This
/// type is that evidence, and it is deliberately a *different type* from
/// [`EvidenceArtifact`](super::evidence_artifact::EvidenceArtifact) rather than the same type with a
/// length that happens to be short. Because only a complete ladder can seal an `EvidenceArtifact`
/// and only a strictly-shorter prefix can seal a `PartialEvidence`, an analysis that accepts the
/// former can never be handed a truncated attempt's data, and the "select exactly one *complete*
/// attempt per logical slot" rule is enforced by the types rather than by a length check someone has
/// to remember to write.
///
/// A failure before the first rung finished yields an empty `PartialEvidence` — that is a real and
/// meaningful state (the attempt reached a measurable configuration and died), not an error.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct PartialEvidence {
    rungs: Vec<RungEvidence>,
}

impl PartialEvidence {
    /// Seal the rungs completed before a failure, failing loud unless they are a strict ascending
    /// prefix of the ladder. A full-length vector is rejected: an attempt that walked every rung did
    /// not fail partway, so it must be sealed as an
    /// [`EvidenceArtifact`](super::evidence_artifact::EvidenceArtifact) instead.
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

    /// The completed rungs, an ascending prefix of the ladder.
    pub(crate) fn rungs(&self) -> &[RungEvidence] {
        &self.rungs
    }

    /// How many rungs completed before the failure.
    pub(crate) fn len(&self) -> usize {
        self.rungs.len()
    }
}
