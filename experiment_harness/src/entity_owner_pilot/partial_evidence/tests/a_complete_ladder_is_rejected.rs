//! A complete ladder walk does not seal as partial evidence.

use crate::entity_owner_pilot::partial_evidence::PartialEvidence;
use crate::entity_owner_pilot::pilot_params::GLOBAL_ROW_LADDER_LEN;

use super::rungs::rungs;

/// The other half of the two types' separation: only a complete walk seals an `EvidenceArtifact` and
/// only a strict prefix seals this, so neither can be assembled from the other's data. Without this
/// rejection a completed attempt could be recorded as failed-with-partial-evidence and silently drop
/// out of the "one complete attempt per logical slot" selection.
#[test]
fn a_complete_ladder_is_rejected() {
    let complete: Vec<usize> = (0..GLOBAL_ROW_LADDER_LEN).collect();

    let err = PartialEvidence::sealed(rungs(&complete))
        .expect_err("a complete walk must not seal as partial evidence");
    let message = format!("{err:#}");
    assert!(
        message.contains("strict prefix"),
        "the error names the strict-prefix invariant: {message}"
    );
}
