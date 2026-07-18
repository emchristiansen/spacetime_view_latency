//! Equal-x pairs are excluded from the pairwise-slope set, so a vertical pair never contributes.

use crate::analysis::stats::rational::Rational;
use crate::analysis::stats::theil_sen::theil_sen_slope;

#[test]
fn excludes_equal_x_pairs() {
    // Points (0,0), (0,5), (1,10): the (0,0)-(0,5) pair shares x and is excluded. The surviving
    // slopes are (0,0)->(1,10) = 10 and (0,5)->(1,10) = 5, whose median is (10 + 5)/2 = 15/2.
    let points = [(0, 0), (0, 5), (1, 10)];
    assert_eq!(theil_sen_slope(&points), Rational::new(15, 2));
}
