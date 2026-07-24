//! The four response classes and their exact band boundaries: Flat-equivalent is inclusive of `±δ`,
//! Increasing/Decreasing are strict, and every straddling interval is Inconclusive.

use crate::analysis::classify::equivalence_margin::{CONTROL_DOSE_MEDIAN_COUNT, EquivalenceMargin};
use crate::analysis::classify::response_class::ResponseClass;
use crate::analysis::stats::median_ci::{exact_median_ci_30, MedianCi};
use crate::analysis::stats::rational::Rational;

/// The frozen margin whose band is `[-δ, +δ]` for the given integer `δ`, built through the production
/// [`EquivalenceMargin::freeze`] so the test exercises the real freeze: a uniform sample of `5·δ` has
/// median `5·δ`, and `δ = median / 5`.
fn margin(delta: i128) -> EquivalenceMargin {
    let medians: [Rational; CONTROL_DOSE_MEDIAN_COUNT] =
        std::array::from_fn(|_| Rational::from_int(delta * 5));
    EquivalenceMargin::freeze(&medians)
}

/// The exact median interval `[lo, hi]` (for `lo ≤ hi`), built through the production
/// [`exact_median_ci_30`] selector: fifteen `lo` samples and fifteen `hi` samples put the sorted
/// `X_(10)` at `lo` and `X_(21)` at `hi`.
fn interval(lo: i128, hi: i128) -> MedianCi {
    let values: [Rational; 30] =
        std::array::from_fn(|i| Rational::from_int(if i < 15 { lo } else { hi }));
    exact_median_ci_30(&values)
}

#[test]
fn classifies_each_band_against_the_frozen_margin() {
    let margin = margin(100);

    // Flat-equivalent: strictly inside the band, and — the boundary case — exactly on both inclusive
    // bounds `[-δ, +δ]`.
    assert_eq!(
        ResponseClass::classify(interval(-50, 50), margin),
        ResponseClass::FlatEquivalent,
        "an interval strictly within [-δ, +δ] is Flat-equivalent"
    );
    assert_eq!(
        ResponseClass::classify(interval(-100, 100), margin),
        ResponseClass::FlatEquivalent,
        "the band bounds are inclusive: [-δ, +δ] is Flat-equivalent"
    );

    // Increasing: the lower bound is strictly above `+δ`.
    assert_eq!(
        ResponseClass::classify(interval(150, 200), margin),
        ResponseClass::Increasing,
        "an interval whose lower bound exceeds +δ is Increasing"
    );

    // Decreasing: the upper bound is strictly below `-δ`.
    assert_eq!(
        ResponseClass::classify(interval(-200, -150), margin),
        ResponseClass::Decreasing,
        "an interval whose upper bound is below -δ is Decreasing"
    );

    // Inconclusive: the interval straddles a band bound — either the upper (lo ≤ +δ < hi) or the lower
    // (lo < -δ ≤ hi).
    assert_eq!(
        ResponseClass::classify(interval(50, 200), margin),
        ResponseClass::Inconclusive,
        "an interval straddling +δ is Inconclusive"
    );
    assert_eq!(
        ResponseClass::classify(interval(-200, -50), margin),
        ResponseClass::Inconclusive,
        "an interval straddling -δ is Inconclusive"
    );
}
