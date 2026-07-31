//! A full-length ladder that is not the ascending sequence does not seal.

use crate::entity_owner_pilot::evidence_artifact::EvidenceArtifact;
use crate::entity_owner_pilot::pilot_params::GLOBAL_ROW_LADDER_LEN;

use super::rungs::rungs;

/// Cardinality alone would accept this: the right number of rungs, but a gap at position one and
/// rung two recorded twice. A "complete" artifact assembled from repeated rungs would report a
/// growth curve the run never walked, so ordering is proven at seal time rather than assumed.
#[test]
fn a_gapped_ladder_is_rejected() {
    assert!(
        GLOBAL_ROW_LADDER_LEN >= 3,
        "this fixture needs at least three rungs to express a gap"
    );
    let mut gapped: Vec<usize> = (0..GLOBAL_ROW_LADDER_LEN).collect();
    gapped[1] = 2;

    let err = EvidenceArtifact::sealed(rungs(&gapped)).expect_err("a gapped ladder must not seal");
    let message = format!("{err:#}");
    assert!(
        message.contains("ascending ladder"),
        "the error names the ordering invariant: {message}"
    );
}
