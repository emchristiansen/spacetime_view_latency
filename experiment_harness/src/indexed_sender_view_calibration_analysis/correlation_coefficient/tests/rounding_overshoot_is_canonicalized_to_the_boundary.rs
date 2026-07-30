//! One-ULP rounding overshoot is pulled to the boundary; ordinary values pass through untouched.

use crate::indexed_sender_view_calibration_analysis::correlation_coefficient::CorrelationCoefficient;

/// Coverage: the exact value an admissible series really produced.
///
/// A thousand-sample series with samples `1_000_004`, then `1_000_000` repeated 998 times, then
/// `4_100_003`, evaluated at lag 999, has `pairs = 1`. The sums then degenerate to
/// `numerator = a·b`, `left = a²`, `right = b²`, so `numerator² = left · right` **exactly** and the
/// mathematical coefficient is exactly `-1`. Cauchy–Schwarz equality is therefore unconditional at
/// the top of the lag domain rather than a coincidence, which is what makes the floating overshoot
/// reachable rather than hypothetical: the `f64` path returns `-1.0000000000000002`.
///
/// Asserted against the serialized form as well as the value, because the artifact is what Control
/// reads, and `#[serde(transparent)]` is the only thing keeping the JSON a bare number.
#[test]
fn rounding_overshoot_is_canonicalized_to_the_boundary() {
    let overshot = CorrelationCoefficient::from_rounded_ratio(-1.0000000000000002);
    assert_eq!(
        serde_json::to_value(overshot).expect("the coefficient serializes"),
        serde_json::json!(-1.0),
        "a one-ULP overshoot is canonicalized to the boundary the exact components prove, and \
         renders as a bare number rather than a wrapper object"
    );

    assert_eq!(
        serde_json::to_value(CorrelationCoefficient::from_rounded_ratio(1.0000000000000002))
            .expect("the coefficient serializes"),
        serde_json::json!(1.0),
        "the positive end is canonicalized identically, so a sign does not decide whether the \
         boundary is enforced"
    );

    // An interior value is not touched: the boundary repair must not become a general rescaling.
    assert_eq!(
        serde_json::to_value(CorrelationCoefficient::from_rounded_ratio(-0.25))
            .expect("the coefficient serializes"),
        serde_json::json!(-0.25),
        "a value already inside the range passes through unchanged"
    );

    // Exactly at the derived ceiling the value is still admitted, so the bound is inclusive rather
    // than one ULP too strict.
    let at_ceiling = 1.0 + 4.0 * f64::EPSILON;
    assert_eq!(
        serde_json::to_value(CorrelationCoefficient::from_rounded_ratio(at_ceiling))
            .expect("the coefficient serializes"),
        serde_json::json!(1.0),
        "the derived rounding ceiling is inclusive"
    );
}
