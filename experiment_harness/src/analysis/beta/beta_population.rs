//! The population β confidence interval and its consistency label over all 30 identified blocks.

use crate::analysis::beta::beta_label::BetaLabel;
use crate::analysis::beta::fit_block::{BETA_MAX, BETA_MIN};
use crate::analysis::stats::median_ci::MedianCi;
use crate::params::REPETITION_BLOCKS;

/// The fixed 30-block sample size as an array length: the population β interval is defined only when
/// every one of the cell's blocks is identifiable, so its input is exactly `REPETITION_BLOCKS` exponents.
const N_BLOCKS: usize = REPETITION_BLOCKS as usize;

/// The population β confidence interval over a cell's 30 identified block exponents (spec: the
/// `[X_(10), X_(21)]` order-statistic interval, emitted only when every block is identifiable). It
/// stores only the two order-statistic endpoints — the sole irreducible evidence — and derives its
/// coverage (a frozen binomial constant) and its consistency label (a pure function of the endpoints)
/// on demand, so neither can drift from the interval it describes.
///
/// The endpoints reuse the primary classifier's frozen `[X_(10), X_(21)]` order indices
/// ([`MedianCi::LOWER_ORDER_INDEX_ONE_BASED`]/[`UPPER_ORDER_INDEX_ONE_BASED`]), so the secondary
/// population interval and the primary median interval cannot disagree on the order statistics.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BetaPopulation {
    /// The lower endpoint `X_(10)` (1-based) of the sorted 30 identified exponents.
    interval_lo: f64,
    /// The upper endpoint `X_(21)` (1-based) of the sorted 30 identified exponents.
    interval_hi: f64,
}

impl BetaPopulation {
    /// Select the population interval from the cell's 30 identified block exponents: sort ascending and
    /// take the frozen `[X_(10), X_(21)]` (1-based) order statistics. Asserts the full mint invariant —
    /// every exponent finite and within the frozen `[0.1, 4.0]` search domain, and the resulting
    /// `lo ≤ hi` — so an out-of-domain or mis-ordered interval is a loud failure, not a representable
    /// value. `pub(super)` — the only caller is
    /// [`BetaDescriptor::population`](super::beta_descriptor::BetaDescriptor::population), which supplies
    /// the exponents of an all-identifiable cell.
    pub(super) fn from_identified(betas: &[f64; N_BLOCKS]) -> Self {
        for &beta in betas.iter() {
            assert!(
                beta.is_finite() && (BETA_MIN..=BETA_MAX).contains(&beta),
                "an identified block exponent must be finite and within the frozen [0.1, 4.0] search domain, got {beta}"
            );
        }
        let mut sorted = *betas;
        sorted.sort_by(|a, b| {
            a.partial_cmp(b)
                .expect("identified block exponents are finite and totally ordered")
        });
        let interval_lo = sorted[MedianCi::LOWER_ORDER_INDEX_ONE_BASED - 1];
        let interval_hi = sorted[MedianCi::UPPER_ORDER_INDEX_ONE_BASED - 1];
        assert!(
            interval_lo <= interval_hi,
            "the population interval's lower order statistic must not exceed its upper, got [{interval_lo}, {interval_hi}]"
        );
        Self {
            interval_lo,
            interval_hi,
        }
    }

    /// The lower endpoint `X_(10)` of the population β interval.
    pub(crate) fn interval_lo(self) -> f64 {
        self.interval_lo
    }

    /// The upper endpoint `X_(21)` of the population β interval.
    pub(crate) fn interval_hi(self) -> f64 {
        self.interval_hi
    }

    /// The exact binomial coverage of the frozen `[X_(10), X_(21)]` interval at `n = 30`
    /// (`Σ_{k=10}^{20} C(30,k)/2^30 = 95.7226%`), rendered to `f64` for the report — the same constant the
    /// primary median interval reports, derived here rather than stored so it cannot drift.
    pub(crate) fn coverage(self) -> f64 {
        MedianCi::coverage().to_f64()
    }

    /// The preregistered consistency label, derived from the interval endpoints so it always agrees with
    /// them.
    pub(crate) fn label(self) -> BetaLabel {
        BetaLabel::classify(self.interval_lo, self.interval_hi)
    }
}
