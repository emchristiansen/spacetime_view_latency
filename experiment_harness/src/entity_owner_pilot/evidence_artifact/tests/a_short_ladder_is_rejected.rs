//! A ladder one rung short does not seal a complete artifact, and the error names the required
//! count.

use crate::entity_owner_pilot::evidence_artifact::EvidenceArtifact;
use crate::entity_owner_pilot::pilot_params::GLOBAL_ROW_LADDER_LEN;

use super::rungs::rungs;

/// A truncated walk is exactly what `PartialEvidence` exists to carry, so it must not be able to
/// become the type analysis reads as complete — otherwise "select one *complete* attempt per logical
/// slot" would rest on a length check someone has to remember.
#[test]
fn a_short_ladder_is_rejected() {
    let short: Vec<usize> = (0..GLOBAL_ROW_LADDER_LEN - 1).collect();

    let err = EvidenceArtifact::sealed(rungs(&short)).expect_err("a short ladder must not seal");
    let message = format!("{err:#}");
    assert!(
        message.contains(&GLOBAL_ROW_LADDER_LEN.to_string()),
        "the error names the required rung count: {message}"
    );
}
