//! A deviation larger than the derived rounding ceiling fails loudly rather than being clamped.

use crate::indexed_sender_view_calibration_analysis::correlation_coefficient::CorrelationCoefficient;

/// Coverage: the property that separates a bounded canonicalization from an unconditional clamp.
///
/// **The exact guarantee, and its limit.** `clamp(-1.0, 1.0)` on its own would absorb any
/// out-of-range value, reporting a plausible boundary number for a computation that had gone wrong.
/// Gating the clamp on the derived ceiling guarantees precisely one thing: a deviation *larger than
/// the ceiling* cannot be silently canonicalized. It does **not** guarantee that every possible
/// implementation bug exceeds the ceiling — a defect that happened to land within a few ULPs of the
/// boundary would be indistinguishable from rounding, and this gate would repair it. That residue is
/// why the exact numerator and energy terms remain the retained authority.
///
/// `1.5` is chosen because it is far outside any rounding budget — roughly `10^15` times the
/// ceiling — so this asserts that the gate exists at all, and deliberately nothing about where its
/// edge falls. The edge itself is pinned by a pair of neighbouring assertions elsewhere:
/// `rounding_overshoot_is_canonicalized_to_the_boundary` accepts `1 + 4·EPSILON` and
/// `a_ratio_one_ulp_beyond_the_ceiling_is_rejected` refuses the next representable value above it.
/// Neither bounds the constant alone; this test bounds it least of all, and would pass against any
/// ceiling below `0.5`.
#[test]
#[should_panic(expected = "indicates a defect in the normalization or evaluation path")]
fn a_gross_out_of_range_ratio_panics() {
    let _ = CorrelationCoefficient::from_rounded_ratio(1.5);
}
