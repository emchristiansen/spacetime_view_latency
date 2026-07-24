//! Perfectly collinear points yield the exact line slope with no averaging error.

use crate::analysis::stats::rational::Rational;
use crate::analysis::stats::theil_sen::theil_sen_slope;

#[test]
fn exact_slope_of_a_line() {
    // y = 1 + 2x at four points: every pairwise slope is exactly 2, so the median is 2/1.
    let points = [(0, 1), (1, 3), (2, 5), (3, 7)];
    assert_eq!(theil_sen_slope(&points), Rational::from_int(2));

    // A fractional slope stays exact: y = x/2 at x in {0, 2, 4} gives every pairwise slope 1/2.
    let fractional = [(0, 0), (2, 1), (4, 2)];
    assert_eq!(theil_sen_slope(&fractional), Rational::new(1, 2));
}
