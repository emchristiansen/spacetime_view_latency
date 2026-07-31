//! The append count is bounded at the boundary, so the freeze's overflow proofs cover every value.

use spacetimedb_sdk::Identity;

use crate::indexed_sender_view_calibration_pilot::calibration_expectation::CalibrationExpectation;
use crate::indexed_sender_view_calibration_pilot::calibration_params::{
    MAX_PACED_SAMPLES, SUBSCRIBER_OWN_ROWS,
};
use crate::indexed_sender_view_calibration_pilot::calibration_rung::CalibrationRung;

/// Coverage: the one input the frozen constants do not bound.
///
/// The compile-time freeze proves the timestamp derivation and `MAX_ACTIVITY_ID` only up to
/// `SUBSCRIBER_OWN_ROWS + MAX_PACED_SAMPLES`. An unbounded `recorded_appends` would therefore reach
/// `checked_add`, and later `expected_ts_micros`, carrying `expect` messages that cite a proof not
/// covering the value — a panic whose message claims impossibility. Rejecting the count here is what
/// makes every one of those citations true.
///
/// The ceiling itself must be accepted, not merely values below it: a bound written with the wrong
/// comparison would refuse exactly the complete attempt the pilot exists to record.
#[test]
fn an_over_ceiling_append_count_is_refused() {
    let measured = Identity::from_byte_array([0u8; 32]);
    let ceiling = u64::from(MAX_PACED_SAMPLES);

    let complete = CalibrationExpectation::after(CalibrationRung::Baseline, ceiling, measured)
        .expect("the frozen ceiling is exactly the complete attempt and must be accepted");
    assert_eq!(
        complete.own_population().rows(),
        SUBSCRIBER_OWN_ROWS
            .checked_add(ceiling)
            .expect("the seeded slice plus the ceiling is proven to fit u64 by the freeze"),
        "a complete attempt ends holding the seeded slice plus every append"
    );
    assert_eq!(complete.recorded_appends(), ceiling);

    let just_over = ceiling
        .checked_add(1)
        .expect("the frozen ceiling plus one must not overflow u64");
    for over in [just_over, u64::MAX] {
        let error = CalibrationExpectation::after(CalibrationRung::Baseline, over, measured)
            .expect_err("an append count above the frozen ceiling must be refused");
        let rendered = format!("{error:#}");
        assert!(
            rendered.contains("frozen ceiling"),
            "the refusal must name the ceiling it enforces, got {rendered:?}"
        );
    }
}
