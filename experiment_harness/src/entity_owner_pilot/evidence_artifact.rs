//! The complete evidence of an attempt that walked the whole ladder.

use anyhow::{anyhow, ensure, Result};
use serde::Serialize;

use crate::entity_owner_pilot::global_row_rung::GlobalRowRung;
use crate::entity_owner_pilot::pilot_params::GLOBAL_ROW_LADDER_LEN;
use crate::entity_owner_pilot::rung_evidence::RungEvidence;

/// Every rung of the frozen ladder, in ascending ladder order — the artifact of an attempt that
/// completed.
///
/// Completeness is proven *structurally*: the sole constructor [`Self::sealed`] converts into a
/// fixed-size `[RungEvidence; GLOBAL_ROW_LADDER_LEN]`, so after sealing an `EvidenceArtifact` cannot
/// hold any other number of rungs. This mirrors
/// [`RawLatencies`](crate::observation::raw_latencies::RawLatencies)' treatment of `BATCH_SIZE`:
/// "complete" is the type, not a property enforced by hand at each call site. An attempt that walked
/// only part of the ladder cannot produce one of these — it produces
/// [`PartialEvidence`](super::partial_evidence::PartialEvidence) instead, which is exactly why
/// [`AttemptOutcome`](super::attempt_outcome::AttemptOutcome)'s complete and failed variants cannot
/// be confused.
///
/// [`Self::sealed`] additionally proves the rungs are the ascending ladder `0..LEN` with no gap,
/// duplicate, or reordering, so a "complete" artifact cannot be assembled from six copies of rung
/// zero.
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

    /// The complete ladder's evidence, always exactly [`GLOBAL_ROW_LADDER_LEN`] rungs in ascending
    /// order.
    pub(crate) fn rungs(&self) -> &[RungEvidence; GLOBAL_ROW_LADDER_LEN] {
        &self.rungs
    }

    /// The highest rung walked — always the ladder's last, by construction.
    pub(crate) fn highest_rung(&self) -> GlobalRowRung {
        self.rungs[GLOBAL_ROW_LADDER_LEN - 1].rung()
    }
}
