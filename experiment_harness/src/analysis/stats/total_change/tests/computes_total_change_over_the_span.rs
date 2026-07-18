//! Over a strictly increasing span the total change is `slope * (n_max - n_min)`, kept exact.

use crate::analysis::stats::rational::Rational;
use crate::analysis::stats::total_change::total_change;

#[test]
fn computes_total_change_over_the_span() {
    // slope 1/2 over the 1,000..10,000 ladder: 9000 * 1/2 = 4500.
    assert_eq!(
        total_change(Rational::new(1, 2), 1_000, 10_000),
        Rational::from_int(4_500)
    );

    // A slope that does not divide the span stays exact: 9000 * 1/7 = 9000/7.
    assert_eq!(
        total_change(Rational::new(1, 7), 1_000, 10_000),
        Rational::new(9_000, 7)
    );
}
