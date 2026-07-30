//! How far one replicate's width-`W` window medians spread, and where the extremes sit.

use serde::Serialize;

use crate::indexed_sender_view_calibration_analysis::exact_rational_report::ExactRationalReport;
use crate::indexed_sender_view_calibration_analysis::positioned_median_report::PositionedMedianReport;
use crate::indexed_sender_view_calibration_analysis::window_median_series::WindowMedianSeries;

/// The bounded, **threshold-free** projection of one replicate's contiguous-window medians at one
/// candidate `W`.
///
/// **Every term is defined here rather than left to a reader's assumption**, because "stability" and
/// "worst case" are exactly the words a report can hide a judgement inside:
///
/// - **spread** is the exact `max − min` over the window medians — a range, not a variance, not a
///   percentile band, and not scaled by anything;
/// - **worst** is the single largest absolute deviation of any window median from the *full-series*
///   median, which is the `W = 1_000` window median of the same replicate;
/// - both extremes carry their window positions, and the worst deviation carries **every** position
///   that attains it, since ties are a fact about the series rather than a formatting problem.
///
/// **No pass, no fail, no threshold, no recommendation.** There is no field for "acceptable", no
/// comparison against a bound, and no cross-candidate ranking. Choosing `W` is Control's act in
/// SSOT; this type's whole job is to state, exactly, what the two series did.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct WindowStabilityReport {
    /// How many contiguous windows of this width the series contains — exactly `n − W + 1`.
    window_count: usize,
    /// The smallest window median, with the position of a window attaining it.
    minimum: PositionedMedianReport,
    /// The largest window median, with the position of a window attaining it.
    maximum: PositionedMedianReport,
    /// The exact `maximum − minimum` spread of the window medians.
    spread: ExactRationalReport,
    /// The largest absolute deviation of any window median from the full-series median.
    worst_absolute_deviation: ExactRationalReport,
    /// **Every** window position attaining that worst deviation, ascending. Ties are retained rather
    /// than resolved: two windows equally far from the full-series median in opposite directions is
    /// a different story from one outlying window, and picking either would tell the wrong one.
    worst_deviation_positions: Vec<usize>,
}

impl WindowStabilityReport {
    /// Summarise one width's window medians against the replicate's full-series median.
    ///
    /// The full-series median is passed in rather than recomputed, so every candidate `W` in a
    /// replicate's report is measured against one and the same centre — otherwise "deviation" would
    /// silently mean a different thing per row of the table.
    pub(crate) fn of(
        medians: &WindowMedianSeries,
        full_series_median: crate::analysis::stats::rational::Rational,
    ) -> Self {
        todo!("min/max with positions, exact spread, and tie-retaining worst deviation")
    }
}

#[cfg(test)]
mod tests;
