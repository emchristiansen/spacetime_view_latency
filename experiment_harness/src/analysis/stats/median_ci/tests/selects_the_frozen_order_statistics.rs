//! The CI selects the 1-based `[X_(10), X_(21)]` order statistics over the 30-block sample.

use crate::analysis::stats::median_ci::{exact_median_ci_30, MedianCi};
use crate::analysis::stats::rational::Rational;
use crate::params::REPETITION_BLOCKS;

#[test]
fn selects_the_frozen_order_statistics() {
    // Values 1..=30 in a scrambled order; sorted, X_(10) = 10 and X_(21) = 21 (1-based).
    let mut values = [Rational::zero(); REPETITION_BLOCKS as usize];
    for (offset, slot) in values.iter_mut().enumerate() {
        // Descending input, so the selection must sort rather than trust position.
        *slot = Rational::from_int((REPETITION_BLOCKS as i128) - offset as i128);
    }
    let ci = exact_median_ci_30(&values);
    assert_eq!(ci.lo(), Rational::from_int(10));
    assert_eq!(ci.hi(), Rational::from_int(21));

    assert_eq!(MedianCi::LOWER_ORDER_INDEX_ONE_BASED, 10);
    assert_eq!(MedianCi::UPPER_ORDER_INDEX_ONE_BASED, 21);
}
