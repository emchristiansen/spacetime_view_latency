//! An even count returns the exact mean of the two central order statistics.

use crate::analysis::stats::median::median;
use crate::analysis::stats::rational::Rational;

#[test]
fn even_count_averages_two_middles() {
    // (20 + 30) / 2 = 25.
    let values = [
        Rational::from_int(40),
        Rational::from_int(10),
        Rational::from_int(20),
        Rational::from_int(30),
    ];
    assert_eq!(median(&values), Rational::from_int(25));

    // A fractional mean stays exact: (1 + 2)/2 = 3/2.
    let pair = [Rational::from_int(2), Rational::from_int(1)];
    assert_eq!(median(&pair), Rational::new(3, 2));
}
