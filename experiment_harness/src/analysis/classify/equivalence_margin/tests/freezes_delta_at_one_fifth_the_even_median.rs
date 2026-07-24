//! `EquivalenceMargin::freeze` takes the exact even-count median of the 300 control dose medians and
//! scales it by one fifth, staying an exact rational, and renders `δ` losslessly-enough in milliseconds.

use crate::analysis::classify::equivalence_margin::{CONTROL_DOSE_MEDIAN_COUNT, EquivalenceMargin};
use crate::analysis::stats::rational::Rational;

/// A 300-sample control-median census of 150 copies of `low` then 150 of `high` (`low ≤ high`), so the
/// even-count median is the exact mean of the two central order statistics `(low + high) / 2`.
fn census(low: i128, high: i128) -> [Rational; CONTROL_DOSE_MEDIAN_COUNT] {
    std::array::from_fn(|i| Rational::from_int(if i < CONTROL_DOSE_MEDIAN_COUNT / 2 { low } else { high }))
}

#[test]
fn freezes_delta_at_one_fifth_the_even_median() {
    // Integer even-median: median((400,600)) = 500, δ = 500 / 5 = 100, with a symmetric band.
    let margin = EquivalenceMargin::freeze(&census(400, 600));
    assert_eq!(
        margin.delta(),
        Rational::from_int(100),
        "δ is one fifth of the even-count median 500"
    );
    assert_eq!(margin.lower(), Rational::from_int(-100), "the band's lower bound is -δ");
    assert_eq!(margin.upper(), Rational::from_int(100), "the band's upper bound is +δ");
    // 100 ns rendered in milliseconds is 1e-4 ms; the rendering is lossy display only.
    assert!(
        (margin.to_millis_f64() - 0.0001).abs() < 1e-15,
        "δ = 100 ns renders as 1e-4 ms"
    );

    // Fractional even-median stays exact: median((401,600)) = 1001/2, δ = 1001/10 — never rounded.
    let fractional = EquivalenceMargin::freeze(&census(401, 600));
    assert_eq!(
        fractional.delta(),
        Rational::new(1001, 10),
        "δ stays an exact rational when the even-count median is fractional"
    );
}
