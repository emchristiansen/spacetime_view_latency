//! Exactly the ascending ladder `0..LEN` seals into a complete evidence artifact.

use crate::entity_owner_pilot::evidence_artifact::EvidenceArtifact;
use crate::entity_owner_pilot::pilot_params::GLOBAL_ROW_LADDER_LEN;

use super::rungs::rungs;

/// The one input a completed attempt produces: every rung of the frozen ladder, once, ascending.
/// This is the only shape that may seal, because an artifact of this type is what analysis accepts
/// as a complete attempt.
#[test]
fn the_complete_ascending_ladder_seals() {
    let complete: Vec<usize> = (0..GLOBAL_ROW_LADDER_LEN).collect();

    EvidenceArtifact::sealed(rungs(&complete)).expect("the complete ascending ladder seals");
}
