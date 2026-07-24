//! The exact 95% median confidence-interval selector over the fixed 30-block sample.

use super::MedianCi;
use crate::analysis::stats::rational::Rational;
use crate::params::REPETITION_BLOCKS;

/// The exact 95% median confidence interval over the fixed 30-block sample: sort the block values by
/// exact rational order and select the frozen `[X_(10), X_(21)]` order statistics. The sample size is
/// pinned to [`REPETITION_BLOCKS`] by the array length, so an off-count sample is a compile error, not
/// a runtime check.
pub(crate) fn exact_median_ci_30(values: &[Rational; REPETITION_BLOCKS as usize]) -> MedianCi {
    let mut sorted = *values;
    sorted.sort();
    MedianCi::new(
        sorted[MedianCi::LOWER_ORDER_INDEX_ONE_BASED - 1],
        sorted[MedianCi::UPPER_ORDER_INDEX_ONE_BASED - 1],
    )
}
