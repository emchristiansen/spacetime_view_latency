//! Right count, wrong rows: every same-cardinality substitution in the arm is rejected.

use super::fixture;
use crate::indexed_sender_view_calibration_pilot::calibration_expectation::unrelated_owner;
use crate::indexed_sender_view_calibration_pilot::calibration_params::UNRELATED_CONTROL_UUID;
use crate::indexed_sender_view_calibration_pilot::verified_population::VerifiedPopulation;

/// Coverage: the exact defect the cardinality-only predecessor had.
///
/// Each case keeps the arm's row **count** exactly right and changes one thing about one row, so a
/// check that compared counts would pass all five. They are not interchangeable faults:
///
/// - **wrong owner** at the right count is a sender-scope leak — the arm returning another identity's
///   row is precisely what this candidate must never do;
/// - **wrong control** means the row was attributed to a control the freeze did not name;
/// - **wrong timestamp** means the row was not written by the frozen recipe, so the state was not the
///   one the attempt believes it measured;
/// - **a duplicated id** covers one fewer id while still counting right, which is how a *missing*
///   row hides behind a correct total;
/// - **an id past the end of the own range** belongs to *neither* preregistered population, which is
///   a different fault from the unrelated-row leak covered separately: nothing in the freeze ever
///   writes it, so it is a seeding or key-derivation bug rather than a visibility one.
///
/// Asserted through `verify` returning `Err` rather than by matching fault text, so the test pins the
/// verdict rather than the wording — but each mismatch must name at least one fault, because a
/// mismatch carrying none would put a passing check into the ledger under a failing verdict.
#[test]
fn same_cardinality_substitutions_are_rejected() {
    let expectation = fixture::expectation();
    let witness = fixture::witness_rows();
    let good = fixture::own_rows();

    // Wrong owner: the row the measured identity should own is owned by the other identity.
    let mut wrong_owner = good.clone();
    wrong_owner
        .last_mut()
        .expect("the arm is non-empty")
        .user_identity = unrelated_owner();

    // Wrong control: right owner and key, attributed to the unrelated control.
    let mut wrong_control = good.clone();
    wrong_control
        .last_mut()
        .expect("the arm is non-empty")
        .control_uuid = UNRELATED_CONTROL_UUID;

    // Wrong timestamp: one microsecond off the frozen derivation. Checked arithmetic, because a
    // fixture that overflowed silently would be contradicting the discipline it exists to test.
    let mut wrong_ts = good.clone();
    let offending = wrong_ts.last_mut().expect("the arm is non-empty");
    offending.ts_micros = offending
        .ts_micros
        .checked_add(1)
        .expect("the fixture timestamp is far below i64::MAX");

    // Duplicated id: the count is untouched, but the set covers one fewer id.
    let mut duplicated = good.clone();
    let first = good[0];
    *duplicated.last_mut().expect("the arm is non-empty") = first;

    // An id one past the end of the own range: in no preregistered population at all.
    let mut past_the_end = good.clone();
    *past_the_end.last_mut().expect("the arm is non-empty") = fixture::row(
        fixture::past_own_range_id(),
        fixture::OWN_CONTROL_UUID_FOR_TESTS,
        fixture::measured(),
    );

    for (case, arm) in [
        ("wrong owner", wrong_owner),
        ("wrong control", wrong_control),
        ("wrong timestamp", wrong_ts),
        ("duplicated id", duplicated),
        ("id past the end of the own range", past_the_end),
    ] {
        assert_eq!(
            arm.len(),
            good.len(),
            "{case}: the substitution must keep the cardinality, or it proves nothing about a \
             count-only check"
        );

        let mismatch = VerifiedPopulation::verify(expectation, &arm, &witness)
            .expect_err("a same-cardinality substitution must not mint a verified population");
        assert!(
            !mismatch.faults().is_empty(),
            "{case}: a mismatch must name at least one fault"
        );
    }
}
