//! The exact distribution-free order-statistic confidence interval for a population median.

use crate::analysis::stats::rational::Rational;
use crate::params::REPETITION_BLOCKS;

/// The exact distribution-free two-sided 95% confidence interval for a population median, obtained
/// from binomial order statistics over the fixed 30-block sample. Its endpoints are individual order
/// statistics (never averages), so the interval is exact with no rational division introduced by the
/// selection itself.
///
/// The order indices and achieved coverage are frozen by the spec: at `n = 30` the narrowest
/// symmetric order-statistic interval whose exact binomial coverage is at least 95% is the 1-based
/// `[X_(10), X_(21)]`, with coverage `Σ_{k=10}^{20} C(30,k) / 2^30 = 95.7226%`. Those indices and the
/// exact coverage are recorded here so the report can cite them.
#[derive(Debug, Clone, Copy)]
pub(crate) struct MedianCi {
    lo: Rational,
    hi: Rational,
}

impl MedianCi {
    /// The frozen 1-based lower order-statistic index (`X_(10)` at `n = 30`).
    pub(crate) const LOWER_ORDER_INDEX_ONE_BASED: usize = 10;
    /// The frozen 1-based upper order-statistic index (`X_(21)` at `n = 30`).
    pub(crate) const UPPER_ORDER_INDEX_ONE_BASED: usize = 21;

    /// The interval's lower bound (the `X_(10)` order statistic).
    pub(crate) fn lo(&self) -> Rational {
        self.lo
    }

    /// The interval's upper bound (the `X_(21)` order statistic).
    pub(crate) fn hi(&self) -> Rational {
        self.hi
    }

    /// The exact binomial coverage of the frozen interval, `Σ_{k=10}^{20} C(30,k) / 2^30`. Computed
    /// exactly rather than transcribed, so the reported coverage cannot silently drift from the
    /// interval's actual order indices.
    pub(crate) fn coverage() -> Rational {
        let n = u128::from(REPETITION_BLOCKS);
        let mut numerator: u128 = 0;
        let mut k = Self::LOWER_ORDER_INDEX_ONE_BASED;
        while k <= Self::UPPER_ORDER_INDEX_ONE_BASED - 1 {
            numerator += binomial(n, k as u128);
            k += 1;
        }
        let denominator = 1u128 << n;
        Rational::new(
            i128::try_from(numerator).expect("the coverage numerator fits i128"),
            i128::try_from(denominator).expect("the coverage denominator fits i128"),
        )
    }
}

/// The exact 95% median confidence interval over the fixed 30-block sample: sort the block values by
/// exact rational order and select the frozen `[X_(10), X_(21)]` order statistics. The sample size is
/// pinned to [`REPETITION_BLOCKS`] by the array length, so an off-count sample is a compile error, not
/// a runtime check.
pub(crate) fn exact_median_ci_30(values: &[Rational; REPETITION_BLOCKS as usize]) -> MedianCi {
    let mut sorted = *values;
    sorted.sort();
    MedianCi {
        lo: sorted[MedianCi::LOWER_ORDER_INDEX_ONE_BASED - 1],
        hi: sorted[MedianCi::UPPER_ORDER_INDEX_ONE_BASED - 1],
    }
}

/// The exact binomial coefficient `C(n, k)` by the multiplicative recurrence, which keeps every
/// intermediate an exact integer. For `n = 30` no intermediate overflows `u128`.
fn binomial(n: u128, k: u128) -> u128 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut result: u128 = 1;
    let mut i: u128 = 0;
    while i < k {
        result = result * (n - i) / (i + 1);
        i += 1;
    }
    result
}

#[cfg(test)]
mod tests;
