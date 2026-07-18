//! Ordering compares by exact cross-multiplication, so unequal denominators order correctly.

use crate::analysis::stats::rational::Rational;

#[test]
fn ordering_via_cross_multiplication() {
    // 1/3 < 1/2 despite the larger denominator.
    assert!(Rational::new(1, 3) < Rational::new(1, 2));
    // -1/2 < 1/3 across the sign boundary.
    assert!(Rational::new(-1, 2) < Rational::new(1, 3));
    // Equal values compare Equal.
    assert!(Rational::new(2, 4) == Rational::new(1, 2));

    let mut values = [
        Rational::new(3, 1),
        Rational::new(-1, 2),
        Rational::new(1, 3),
        Rational::new(1, 2),
    ];
    values.sort();
    assert_eq!(
        values,
        [
            Rational::new(-1, 2),
            Rational::new(1, 3),
            Rational::new(1, 2),
            Rational::new(3, 1),
        ]
    );
}
