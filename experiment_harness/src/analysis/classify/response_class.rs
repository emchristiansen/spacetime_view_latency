//! The four-way primary response class of an arm's paired-difference interval.

use crate::analysis::classify::equivalence_margin::EquivalenceMargin;
use crate::analysis::stats::median_ci::MedianCi;

/// The preregistered four-way classification of an arm's paired-difference total-change interval `T`
/// against the frozen practical-equivalence band `[-δ, +δ]` (spec: "Classification"). This is the
/// observed primary response, distinct from the preregistered
/// [`PredictedResponse`](crate::plan::predicted_response::PredictedResponse); a contradictory measured
/// class is recorded unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub(crate) enum ResponseClass {
    /// The complete interval for `T` lies within `[-δ, +δ]`.
    FlatEquivalent,
    /// The interval's lower bound is greater than `+δ`.
    Increasing,
    /// The interval's upper bound is less than `-δ`.
    Decreasing,
    /// Every remaining interval — it straddles a band bound.
    Inconclusive,
}

impl ResponseClass {
    /// Classify the arm's paired-difference total-change interval against the frozen margin's band,
    /// using exact rational comparison of the interval endpoints against `±δ` (spec: inclusive
    /// Flat-equivalent band, strict Increasing/Decreasing bounds).
    ///
    /// The band is `[-δ, +δ]`. The four classes are checked in an order that makes them mutually
    /// exclusive: an interval within the band is Flat-equivalent (both bounds inclusive); otherwise a
    /// lower bound strictly above `+δ` is Increasing and an upper bound strictly below `-δ` is
    /// Decreasing; every remaining interval straddles a band bound and is Inconclusive.
    pub(crate) fn classify(arm_interval: MedianCi, margin: EquivalenceMargin) -> Self {
        let lo = arm_interval.lo();
        let hi = arm_interval.hi();
        if lo >= margin.lower() && hi <= margin.upper() {
            Self::FlatEquivalent
        } else if lo > margin.upper() {
            Self::Increasing
        } else if hi < margin.lower() {
            Self::Decreasing
        } else {
            Self::Inconclusive
        }
    }
}
