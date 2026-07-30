//! A row from the other population appearing in the arm is a fault, not a row filed elsewhere.

use super::fixture;
use crate::indexed_sender_view_calibration_pilot::calibration_expectation::unrelated_owner;
use crate::indexed_sender_view_calibration_pilot::calibration_params::{
    UNRELATED_ACTIVITY_ID_BASE, UNRELATED_CONTROL_UUID,
};
use crate::indexed_sender_view_calibration_pilot::verified_population::VerifiedPopulation;

/// Coverage: the arm is checked *exclusively* against the own population.
///
/// The substituted row is entirely well-formed — a real unrelated row, correct owner, control, and
/// derived timestamp — and it belongs to a population the witness legitimately holds. A verifier that
/// partitioned the arm the way it partitions the witness would file it under "unrelated" and move on.
///
/// For a sender-scoped view that would be the whole defect: the candidate exists to return the
/// measured identity's rows *and only those*, so a correctly-formed row owned by somebody else
/// showing up in the arm is exactly the leak the screen must detect, not a bookkeeping detail. The
/// fault must say so in the arm's own terms, which is why the wording is checked here and not in the
/// same-cardinality test.
#[test]
fn an_unrelated_row_in_the_arm_is_a_leak() {
    let good = fixture::own_rows();
    let mut arm = good.clone();
    *arm.last_mut().expect("the arm is non-empty") = fixture::row(
        UNRELATED_ACTIVITY_ID_BASE,
        UNRELATED_CONTROL_UUID,
        unrelated_owner(),
    );
    assert_eq!(arm.len(), good.len(), "the cardinality is untouched");

    let mismatch =
        VerifiedPopulation::verify(fixture::expectation(), &arm, &fixture::witness_rows())
            .expect_err("an unrelated row in the arm must not verify");

    let rendered = mismatch.faults().join("; ");
    assert!(
        rendered.contains("does not own"),
        "the fault must state that the arm returned a row the measured identity does not own, got \
         {rendered:?}"
    );
}
