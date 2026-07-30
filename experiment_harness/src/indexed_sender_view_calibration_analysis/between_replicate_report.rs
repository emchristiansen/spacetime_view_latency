//! How far the two replicates disagree at one candidate `W`, position by position.

use serde::Serialize;

use crate::indexed_sender_view_calibration_analysis::exact_rational_report::ExactRationalReport;
use crate::indexed_sender_view_calibration_analysis::positioned_difference_report::PositionedDifferenceReport;
use crate::indexed_sender_view_calibration_analysis::window_median_series::WindowMedianSeries;

/// The disagreement between the two replicates at one candidate `W`, defined over the **whole**
/// corresponding window-median series.
///
/// **The fourth thing §569's rule weighs, and the one a single series cannot answer at all** — which
/// is why the freeze runs two replicates and why
/// [`ReplicatePair`](super::replicate_pair::ReplicatePair) makes a one-series analysis
/// unconstructible.
///
/// **The comparison domain is justified from §569's wording, not chosen.** The rule reads: prefer the
/// smallest count "whose position-aware window medians are stable enough for a descriptive screen in
/// **both** attempts, considering … between-attempt disagreement". Two things follow. Disagreement is
/// about the *position-aware* window medians, so it is measured position by position; and it is about
/// **both** attempts across the series, so nothing privileges any single window. An earlier draft
/// compared only each replicate's first window and called it "the median a screen would actually
/// obtain" — an unapproved modelling choice resting on a screen schedule that does not exist yet, and
/// one that discarded 990 of the 991 comparisons available at `W = 10`.
///
/// The correspondence is exact and total: admission fixes both series at the frozen sample count, so
/// both replicates have identically many width-`W` windows and position `i` means the same offset in
/// each.
///
/// **Threshold-free throughout.** Signed differences keep their direction, extrema keep every
/// position that attains them, and the central summary is an exact median rather than a mean whose
/// value a single outlying window could set. There is no bound, no tolerance, and no judgement about
/// whether the disagreement is small.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct BetweenReplicateReport {
    /// How many corresponding window positions were compared — `n − W + 1`, the same for both.
    window_count: usize,
    /// The greatest per-position `second − first`, with every position attaining it.
    ///
    /// Named for its position in the order, not for a sign: if replicate 1 ran slower at *every*
    /// window this maximum is still negative, and a name asserting otherwise would be false of that
    /// perfectly ordinary case.
    maximum_signed_difference: PositionedDifferenceReport,
    /// The least per-position `second − first`, with every position attaining it. Reported alongside
    /// the maximum so a pair that disagrees in both directions cannot be summarised as though it
    /// disagreed in one.
    minimum_signed_difference: PositionedDifferenceReport,
    /// The largest `|second − first|` over all positions, with every position attaining it.
    greatest_absolute_difference: PositionedDifferenceReport,
    /// The exact median of the per-position signed differences — a central summary that is robust to
    /// a single outlying window, unlike a mean.
    median_signed_difference: ExactRationalReport,
}

impl BetweenReplicateReport {
    /// Compare the two replicates' width-`W` window medians at every corresponding position.
    ///
    /// Both arguments are the *same* candidate width — they come from one iteration of the per-`W`
    /// loop — and, by admission, have equal length, so the position correspondence is total.
    pub(crate) fn of(first: &WindowMedianSeries, second: &WindowMedianSeries) -> Self {
        todo!("per-position signed differences, tie-preserving extrema, and the exact median")
    }
}

#[cfg(test)]
mod tests;
