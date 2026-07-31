//! Every strict ascending prefix of the ladder, including the empty one, seals.

use crate::entity_owner_pilot::partial_evidence::PartialEvidence;
use crate::entity_owner_pilot::pilot_params::GLOBAL_ROW_LADDER_LEN;

use super::rungs::rungs;

/// The complete set of shapes a failing attempt can produce: it dies somewhere in the walk, so it
/// has completed rungs `0..k` for some `k` strictly below the ladder length. All of them must seal —
/// including `k = 0`, which is a real state (the attempt reached a measurable configuration and died
/// before the first rung finished), not an error.
#[test]
fn every_strict_ascending_prefix_seals() {
    for length in 0..GLOBAL_ROW_LADDER_LEN {
        let prefix: Vec<usize> = (0..length).collect();

        PartialEvidence::sealed(rungs(&prefix))
            .unwrap_or_else(|e| panic!("the {length}-rung ascending prefix must seal: {e:#}"));
    }
}
