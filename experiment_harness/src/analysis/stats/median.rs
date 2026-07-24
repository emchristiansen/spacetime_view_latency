//! The exact median of a set of rationals.

use crate::analysis::stats::rational::Rational;

/// The exact median of a nonempty rational sample. For an odd count this is the single middle order
/// statistic (exact, no averaging); for an even count it is the exact mean of the two central order
/// statistics. The input is copied and sorted, so the caller's order is preserved.
///
/// The primary Theil–Sen path always feeds this an **odd** count (the 45 pairwise slopes of a
/// ten-dose ladder), so its median is a single exact slope with no averaging; the even branch exists
/// for the δ margin's median over the 300 control per-dose values.
pub(crate) fn median(values: &[Rational]) -> Rational {
    assert!(
        !values.is_empty(),
        "the median of an empty sample is undefined"
    );
    let mut sorted = values.to_vec();
    sorted.sort();
    let n = sorted.len();
    if n % 2 == 1 {
        sorted[n / 2]
    } else {
        sorted[n / 2 - 1].add(sorted[n / 2]).div_int(2)
    }
}

#[cfg(test)]
mod tests;
