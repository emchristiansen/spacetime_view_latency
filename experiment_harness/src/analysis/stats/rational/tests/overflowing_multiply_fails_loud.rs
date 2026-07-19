//! A checked-arithmetic overflow fails loud (panics) rather than silently wrapping.

use crate::analysis::stats::rational::Rational;

/// Multiplying two rationals whose numerator product exceeds `i128` overflows the checked numerator
/// multiply in [`Rational::mul`], which fails loud rather than wrapping — the fail-fast contract the
/// whole exact-rational estimator path relies on.
#[test]
#[should_panic(expected = "rational product numerator fits i128")]
fn overflowing_multiply_fails_loud() {
    // i128::MAX * 2 cannot fit i128, so Rational::mul's checked numerator multiply panics.
    let _ = Rational::from_int(i128::MAX).mul(Rational::from_int(2));
}
