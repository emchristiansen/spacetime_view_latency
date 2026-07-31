//! A short run of rungs that skips a ladder position does not seal.

use crate::entity_owner_pilot::partial_evidence::PartialEvidence;
use crate::entity_owner_pilot::pilot_params::GLOBAL_ROW_LADDER_LEN;

use super::rungs::rungs;

/// Short enough to be a prefix by length, but rungs `0, 2` are not the walk any attempt performs:
/// the ladder is cumulative and walked in order, so a gap means the recorded evidence does not
/// describe a real progression. Length alone would accept it.
#[test]
fn a_gapped_prefix_is_rejected() {
    assert!(
        GLOBAL_ROW_LADDER_LEN >= 3,
        "this fixture needs at least three rungs to express a gap inside a strict prefix"
    );

    let err = PartialEvidence::sealed(rungs(&[0, 2]))
        .expect_err("a gapped prefix must not seal");
    let message = format!("{err:#}");
    assert!(
        message.contains("ascending ladder prefix"),
        "the error names the ordering invariant: {message}"
    );
}
