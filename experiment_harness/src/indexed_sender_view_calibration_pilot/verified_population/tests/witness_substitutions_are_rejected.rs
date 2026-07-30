//! The witness cache is checked on its own path, and every same-cardinality substitution fails it.

use super::fixture;
use crate::indexed_sender_view_calibration_pilot::calibration_expectation::unrelated_owner;
use crate::indexed_sender_view_calibration_pilot::calibration_params::{
    UNRELATED_ACTIVITY_ID_BASE, UNRELATED_CONTROL_UUID,
};
use crate::indexed_sender_view_calibration_pilot::verified_population::VerifiedPopulation;

/// Coverage: the witness path, which is genuinely different code from the arm's.
///
/// The arm goes through the exclusive check — one population, anything outside it a leak. The witness
/// is *partitioned* by key range first and each part checked against its own population, so the arm's
/// cases do not exercise it: a substitution the exclusive check catches by range membership alone may
/// reach the per-column checks here instead, or land in the "neither range" arm.
///
/// Each case leaves the witness total untouched and changes one property of one row.
#[test]
fn witness_substitutions_are_rejected() {
    let expectation = fixture::expectation();
    let arm = fixture::own_rows();
    let good = fixture::witness_rows();

    // Wrong owner on an own-range row the witness legitimately holds.
    let mut wrong_owner = good.clone();
    wrong_owner[0].user_identity = unrelated_owner();

    // Wrong control on an own-range row.
    let mut wrong_control = good.clone();
    wrong_control[0].control_uuid = UNRELATED_CONTROL_UUID;

    // Wrong timestamp, with checked arithmetic for the same reason the arm's case uses it.
    let mut wrong_ts = good.clone();
    wrong_ts[0].ts_micros = wrong_ts[0]
        .ts_micros
        .checked_add(1)
        .expect("the fixture timestamp is far below i64::MAX");

    // A duplicated own-range id: the witness total is untouched, the own population covers one
    // fewer id.
    let mut duplicated = good.clone();
    duplicated[1] = good[0];

    // A row belonging to neither preregistered range, substituted for an own row.
    let mut out_of_both_ranges = good.clone();
    out_of_both_ranges[0] = fixture::row(
        fixture::past_own_range_id(),
        fixture::OWN_CONTROL_UUID_FOR_TESTS,
        fixture::measured(),
    );

    for (case, witness) in [
        ("wrong owner", wrong_owner),
        ("wrong control", wrong_control),
        ("wrong timestamp", wrong_ts),
        ("duplicated own id", duplicated),
        ("row in neither range", out_of_both_ranges),
    ] {
        assert_eq!(
            witness.len(),
            good.len(),
            "{case}: the substitution must keep the witness cardinality"
        );

        let mismatch = VerifiedPopulation::verify(expectation, &arm, &witness)
            .expect_err("a same-cardinality witness substitution must not mint a token");
        assert!(
            !mismatch.faults().is_empty(),
            "{case}: a mismatch must name at least one fault"
        );
    }

    // The case a grand total cannot see, checked explicitly rather than in the loop because both of
    // its faults matter and each names a different population.
    //
    // An own row is replaced by a *valid* unrelated row — correct owner, control, and derived
    // timestamp — that duplicates one already present. Duplicating is not incidental here: the
    // unrelated range is exactly full at the baseline rung, so there is no unused valid unrelated id
    // to add, and a duplicate is the only way an extra well-formed unrelated row can exist at all.
    //
    // The witness total is therefore identical to a correct cache while the own population is short
    // by one, which is the shape of a lost append hiding behind a correct grand total. It is caught
    // only because each population is counted separately.
    let mut compensated = good.clone();
    compensated[0] = fixture::row(
        UNRELATED_ACTIVITY_ID_BASE,
        UNRELATED_CONTROL_UUID,
        unrelated_owner(),
    );
    assert_eq!(compensated.len(), good.len(), "the grand total is unchanged");

    let mismatch = VerifiedPopulation::verify(expectation, &arm, &compensated)
        .expect_err("a lost own row compensated inside the witness total must not verify");
    let rendered = mismatch.faults().join("; ");
    assert!(
        rendered.contains("more than once"),
        "the duplicated unrelated id must be reported, got {rendered:?}"
    );
    assert!(
        rendered.contains("distinct rows rather than the expected"),
        "the own population's shortfall must be reported, got {rendered:?}"
    );
}
