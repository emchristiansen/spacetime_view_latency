//! Even-`n`: R-1 takes the lower of the two central order statistics — no averaging — so the
//! median of `[10, 30]` is `10` (not `20`), the crisp even-`n` signature distinguishing R-1 from
//! the interpolating R-7 convention.

use super::quantiles;

#[test]
fn even_samples_take_the_lower_middle() {
    // n=2: Q1=x_(⌈0.5⌉=1)=10, med=x_(⌈1⌉=1)=10, Q3=x_(⌈1.5⌉=2)=30, IQR=20.
    assert_eq!(quantiles(&[10, 30]), (10, 10, 30, 20));

    // n=4: Q1=x_(⌈1⌉=1)=10, med=x_(⌈2⌉=2)=20, Q3=x_(⌈3⌉=3)=30, IQR=20.
    assert_eq!(quantiles(&[10, 20, 30, 40]), (10, 20, 30, 20));
}
