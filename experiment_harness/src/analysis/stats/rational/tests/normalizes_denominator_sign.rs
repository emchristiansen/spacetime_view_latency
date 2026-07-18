//! The sign always lives in the numerator; the denominator is always strictly positive.

use crate::analysis::stats::rational::Rational;

#[test]
fn normalizes_denominator_sign() {
    let from_negative_den = Rational::new(1, -2);
    assert_eq!(from_negative_den.numerator(), -1);
    assert_eq!(from_negative_den.denominator(), 2);

    // Both-negative reduces to a positive value with a positive denominator.
    let both_negative = Rational::new(-3, -4);
    assert_eq!(both_negative.numerator(), 3);
    assert_eq!(both_negative.denominator(), 4);

    assert_eq!(Rational::new(1, -2), Rational::new(-1, 2));
}
