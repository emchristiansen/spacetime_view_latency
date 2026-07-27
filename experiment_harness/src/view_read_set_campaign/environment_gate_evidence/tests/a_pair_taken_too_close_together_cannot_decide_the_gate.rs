//! `paired` derives the separation from the samples' own offsets and refuses a pair too short to
//! show a falling trend.

use super::fixture::{
    passing_first, passing_second, AVAILABLE_RAM_BYTES, FIRST_LOAD_CENTI, FIRST_OFFSET_NANOS,
    LOGICAL_CPUS, PSI_CENTI, SECOND_LOAD_CENTI, SECOND_OFFSET_NANOS, SWAP_OUT_BYTES,
};
use crate::view_read_set_campaign::environment_gate_evidence::EnvironmentGateEvidence;
use crate::view_read_set_campaign::environment_sample::EnvironmentSample;

/// Coverage: the separation is a precondition, not a clause. Two readings moments apart would satisfy
/// "strictly lower" on noise alone, so a short pair cannot decide the gate either way — and an
/// inverted pair is a mismatched origin, not a fast one.
#[test]
fn a_pair_taken_too_close_together_cannot_decide_the_gate() {
    EnvironmentGateEvidence::paired(passing_first(), passing_second(), LOGICAL_CPUS)
        .expect("exactly the frozen separation is the shortest interval the gate accepts");

    let one_nanosecond_short = EnvironmentSample::observed(
        SECOND_OFFSET_NANOS - 1,
        SECOND_LOAD_CENTI,
        AVAILABLE_RAM_BYTES,
        SWAP_OUT_BYTES,
        PSI_CENTI,
    );
    assert!(
        EnvironmentGateEvidence::paired(passing_first(), one_nanosecond_short, LOGICAL_CPUS)
            .is_err(),
        "a pair one nanosecond short of the frozen separation cannot have come from the frozen \
         procedure"
    );

    let inverted = EnvironmentSample::observed(
        FIRST_OFFSET_NANOS,
        SECOND_LOAD_CENTI,
        AVAILABLE_RAM_BYTES,
        SWAP_OUT_BYTES,
        PSI_CENTI,
    );
    let later_first = EnvironmentSample::observed(
        SECOND_OFFSET_NANOS,
        FIRST_LOAD_CENTI,
        AVAILABLE_RAM_BYTES,
        SWAP_OUT_BYTES,
        PSI_CENTI,
    );
    assert!(
        EnvironmentGateEvidence::paired(later_first, inverted, LOGICAL_CPUS).is_err(),
        "a second sample taken before the first is a broken origin, not a short interval"
    );
}
