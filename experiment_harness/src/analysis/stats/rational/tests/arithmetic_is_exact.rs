//! Addition, subtraction, multiplication, and integer division are exact and reduced.

use crate::analysis::stats::rational::Rational;

#[test]
fn arithmetic_is_exact() {
    // 1/2 + 1/3 = 5/6.
    assert_eq!(
        Rational::new(1, 2).add(Rational::new(1, 3)),
        Rational::new(5, 6)
    );
    // 1/2 - 1/3 = 1/6.
    assert_eq!(
        Rational::new(1, 2).sub(Rational::new(1, 3)),
        Rational::new(1, 6)
    );
    // 2/3 * 3/4 = 1/2 (reduced).
    assert_eq!(
        Rational::new(2, 3).mul(Rational::new(3, 4)),
        Rational::new(1, 2)
    );
    // (3/4) / 2 = 3/8.
    assert_eq!(Rational::new(3, 4).div_int(2), Rational::new(3, 8));
    // Negation moves the sign onto the numerator.
    assert_eq!(Rational::new(1, 2).neg(), Rational::new(-1, 2));
}
