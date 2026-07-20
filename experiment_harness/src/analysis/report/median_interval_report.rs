//! The report projection of an exact order-statistic median confidence interval.

use serde::Serialize;

use crate::analysis::finite_f64::FiniteF64;
use crate::analysis::stats::median_ci::MedianCi;

/// The report projection of an exact [`MedianCi`]: its two `[X_(10), X_(21)]` order-statistic endpoints
/// rendered in milliseconds. The endpoints are exact [`Rational`](crate::analysis::stats::rational::Rational)
/// nanoseconds in the analysis domain; they cross the lossy floating-point boundary here, so both are
/// finite [`FiniteF64`] milliseconds — never a NaN/±∞ sentinel. The exact interval stays non-`Serialize`,
/// so no classification input can be reconstructed from this display projection.
#[derive(Debug, Serialize)]
pub(crate) struct MedianIntervalReport {
    /// The interval's lower bound (`X_(10)`), in milliseconds.
    lo_millis: FiniteF64,
    /// The interval's upper bound (`X_(21)`), in milliseconds.
    hi_millis: FiniteF64,
}

impl MedianIntervalReport {
    /// Project an exact median interval. Takes the `Copy` [`MedianCi`] by value — one input.
    pub(crate) fn of(interval: MedianCi) -> Self {
        let _ = interval;
        todo!("Phase 2: render both order-statistic endpoints to finite millisecond FiniteF64")
    }
}
