//! When `n·p` is already an integer the ceiling is a no-op, so the rank is exactly `n·p`: with
//! `[1, 2, 3, 4, 5, 6]`, `n·0.5 = 3` gives med=x_(3)=3 and `n·0.25 = 1.5` still rounds up to
//! Q1=x_(2)=2 while `n·0.75 = 4.5` rounds up to Q3=x_(5)=5.

use super::quantiles;

#[test]
fn rank_landing_on_an_integer() {
    // n=6: Q1=x_(⌈1.5⌉=2)=2, med=x_(⌈3⌉=3)=3, Q3=x_(⌈4.5⌉=5)=5, IQR=3.
    assert_eq!(quantiles(&[1, 2, 3, 4, 5, 6]), (2, 3, 5, 3));
}
