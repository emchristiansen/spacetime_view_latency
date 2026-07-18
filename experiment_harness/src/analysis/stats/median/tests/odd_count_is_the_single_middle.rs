//! An odd count returns the single middle order statistic exactly, regardless of input order.

use crate::analysis::stats::median::median;
use crate::analysis::stats::rational::Rational;

#[test]
fn odd_count_is_the_single_middle() {
    let values = [
        Rational::from_int(3),
        Rational::from_int(1),
        Rational::from_int(2),
    ];
    assert_eq!(median(&values), Rational::from_int(2));
}
