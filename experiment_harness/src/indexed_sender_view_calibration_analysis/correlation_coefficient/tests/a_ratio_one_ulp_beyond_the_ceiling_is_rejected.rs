//! The derived ceiling is rejected from above at the very next representable value.

use crate::indexed_sender_view_calibration_analysis::correlation_coefficient::CorrelationCoefficient;

/// Coverage: the upper side of the exact rejection edge, one ULP above the ceiling.
///
/// **This is what pins the constant, and it needs a sibling to do it.**
/// `rounding_overshoot_is_canonicalized_to_the_boundary` proves `1 + 4·EPSILON` is *accepted*; on its
/// own that bounds the ceiling only from below, and a mistakenly widened `5·EPSILON` or `8·EPSILON`
/// implementation would satisfy it, the one-ULP and interior cases, the reachable witness, and the
/// gross `1.5` rejection alike. Rejecting the next value up is the assertion those all miss, and the
/// two together admit exactly `4·EPSILON`.
///
/// `1 + 5·EPSILON` really is the immediate neighbour rather than a comfortable margin: on `[1, 2)`
/// the spacing of `f64` is exactly `EPSILON`, so `1 + 4·EPSILON` and `1 + 5·EPSILON` are adjacent
/// representable values and nothing lies between them.
///
/// Only the positive side is asserted here because the gate tests `abs()`, and the sibling's
/// matched `±` boundary cases are what establish that the sign does not decide enforcement.
#[test]
#[should_panic(expected = "indicates a defect in the normalization or evaluation path")]
fn a_ratio_one_ulp_beyond_the_ceiling_is_rejected() {
    let _ = CorrelationCoefficient::from_rounded_ratio(1.0 + 5.0 * f64::EPSILON);
}
