//! The complete evidence of an attempt that walked the whole ladder.

use anyhow::{anyhow, ensure, Result};
use serde::Serialize;

use crate::entity_owner_pilot::pilot_params::GLOBAL_ROW_LADDER_LEN;
use crate::entity_owner_pilot::rung_evidence::RungEvidence;

/// Every rung of the frozen ladder, in ascending order — the artifact of a completed attempt.
///
/// Completeness is structural: [`Self::sealed`] converts into a fixed-size array, so a sealed
/// artifact cannot hold any other number of rungs. Mirrors
/// [`RawLatencies`](crate::observation::raw_latencies::RawLatencies)' treatment of `BATCH_SIZE`.
/// Because a partial walk can only produce
/// [`PartialEvidence`](super::partial_evidence::PartialEvidence), analysis that accepts this type
/// can never be handed a truncated attempt's data.
///
/// Sealing also proves the rungs are the ascending ladder with no gap, duplicate, or reordering, so
/// a "complete" artifact cannot be assembled from six copies of rung zero.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct EvidenceArtifact {
    rungs: Box<[RungEvidence; GLOBAL_ROW_LADDER_LEN]>,
}

impl EvidenceArtifact {
    /// Seal a completed attempt's rungs, failing loud unless they are exactly the ascending ladder.
    pub(crate) fn sealed(rungs: Vec<RungEvidence>) -> Result<Self> {
        let collected = rungs.len();
        for (position, evidence) in rungs.iter().enumerate() {
            ensure!(
                evidence.rung().get() == position,
                "a complete evidence artifact must carry the ascending ladder 0..{}; \
                 position {position} holds rung {}",
                GLOBAL_ROW_LADDER_LEN,
                evidence.rung().get(),
            );
        }
        let rungs: Box<[RungEvidence; GLOBAL_ROW_LADDER_LEN]> =
            rungs.into_boxed_slice().try_into().map_err(|_| {
                anyhow!(
                    "a complete evidence artifact must contain exactly {GLOBAL_ROW_LADDER_LEN} \
                     rungs, got {collected}"
                )
            })?;
        Ok(Self { rungs })
    }
}
