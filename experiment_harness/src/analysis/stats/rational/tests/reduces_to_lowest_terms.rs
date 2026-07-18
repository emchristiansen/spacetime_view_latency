//! Construction reduces `num/den` by their gcd, so equal values share one canonical form.

use crate::analysis::stats::rational::Rational;

#[test]
fn reduces_to_lowest_terms() {
    let half = Rational::new(6, 12);
    assert_eq!(half.numerator(), 1);
    assert_eq!(half.denominator(), 2);

    // Equal values are structurally equal after reduction, so `PartialEq` is numeric equality.
    assert_eq!(Rational::new(6, 12), Rational::new(50, 100));

    // Zero canonicalizes to 0/1 regardless of the denominator it was built with.
    let zero = Rational::new(0, 97);
    assert_eq!(zero.numerator(), 0);
    assert_eq!(zero.denominator(), 1);
    assert_eq!(zero, Rational::zero());
}
