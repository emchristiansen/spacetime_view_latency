//! Odd-`n`: the median lands exactly on the central order statistic and the quartile ranks round
//! up, so `[10, 20, 30, 40, 50]` yields Q1=20, med=30, Q3=40, IQR=20.

use super::quantiles;

#[test]
fn odd_sample_quantiles() {
    // n=5: Q1=x_(⌈1.25⌉=2)=20, med=x_(⌈2.5⌉=3)=30, Q3=x_(⌈3.75⌉=4)=40, IQR=20.
    assert_eq!(quantiles(&[10, 20, 30, 40, 50]), (20, 30, 40, 20));
}
