//! Ten distinct-x points give all `C(10,2) = 45` pairwise slopes, whose exact middle order statistic
//! is a single hand-derived slope with no averaging.

use crate::analysis::stats::rational::Rational;
use crate::analysis::stats::theil_sen::theil_sen_slope;

#[test]
fn median_of_the_forty_five_pairwise_slopes() {
    // Ten points on y = x^2 at x in 0..=9 (all x distinct, so no equal-x pair is excluded and every
    // one of the C(10,2) = 45 pairs contributes a slope). The slope of any pair (a, a^2)-(b, b^2) is
    // (b^2 - a^2) / (b - a) = a + b exactly, so the 45 slopes are precisely the pairwise sums a + b of
    // distinct a, b in {0, .., 9}.
    let points = [
        (0, 0),
        (1, 1),
        (2, 4),
        (3, 9),
        (4, 16),
        (5, 25),
        (6, 36),
        (7, 49),
        (8, 64),
        (9, 81),
    ];

    // Distribution of the 45 pairwise sums a + b (a < b) over {0, .., 9}, by value:
    //   sum:   1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17
    //   count: 1 1 2 2 3 3 4 4 5  4  4  3  3  2  2  1  1   (total 45)
    // Cumulative count through sum 8 is 1+1+2+2+3+3+4+4 = 20, so the five slopes equal to 9 occupy the
    // 1-based sorted positions 21..=25. The median of 45 values is the single middle order statistic at
    // zero-based index 45 / 2 = 22 (1-based position 23), which lies in that run — so it is exactly 9,
    // selected without any averaging.
    assert_eq!(theil_sen_slope(&points), Rational::from_int(9));
}
