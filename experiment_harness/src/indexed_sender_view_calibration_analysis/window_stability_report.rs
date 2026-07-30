//! How far one replicate's width-`W` window medians spread, and where the extremes sit.

use serde::Serialize;

use crate::analysis::stats::rational::Rational;
use crate::indexed_sender_view_calibration_analysis::absolute_value::absolute_value;
use crate::indexed_sender_view_calibration_analysis::exact_rational_report::ExactRationalReport;
use crate::indexed_sender_view_calibration_analysis::positioned_median_report::PositionedMedianReport;
use crate::indexed_sender_view_calibration_analysis::window_median_series::WindowMedianSeries;

/// One replicate's contiguous-window medians at one candidate `W`: the **complete** position-aware
/// series, together with **threshold-free** projections of it.
///
/// **The series is carried, not summarised away.** §615 requires the position-aware contiguous-window
/// medians themselves to reach the artifact, separately from the extrema and the range. The
/// projections below state directly the quantities §569's rule weighs; the series is what lets a
/// reader recompute any of them, or ask a question this type did not anticipate, without rerunning
/// the analyzer against a ledger that by then may not exist. A summary that cannot be audited against
/// its own input is a weaker artifact than one that can.
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
    /// Every window median, ascending by window start position. Its length is exactly
    /// `window_count`, and each element states its own position rather than leaving it implicit in
    /// the array index — so a row quoted out of the artifact still says which window it came from.
    ///
    /// **Last on purpose.** At `W = 10` this is 991 elements for one replicate at one candidate, and
    /// roughly ten thousand across the whole report. Placing it after the projections keeps the
    /// quantities the rule weighs at the top of each row, where they stay readable, instead of behind
    /// a screen of numbers. The size is the cost of the §615 requirement, not an oversight.
    medians: Vec<PositionedMedianReport>,
}

impl WindowStabilityReport {
    /// Summarise one width's window medians against the replicate's full-series median.
    ///
    /// The full-series median is passed in rather than recomputed, so every candidate `W` in a
    /// replicate's report is measured against one and the same centre — otherwise "deviation" would
    /// silently mean a different thing per row of the table.
    pub(crate) fn of(medians: &WindowMedianSeries, full_series_median: Rational) -> Self {
        let values = medians.medians();
        assert!(
            !values.is_empty(),
            "every candidate width admits at least one window over a complete series"
        );

        let minimum = *values.iter().min().expect("the window set is nonempty");
        let maximum = *values.iter().max().expect("the window set is nonempty");
        // The *first* attaining position for each extreme, so the choice is deterministic rather
        // than an artefact of which direction the iterator folded from.
        let minimum_position = values
            .iter()
            .position(|value| *value == minimum)
            .expect("the minimum came from this set");
        let maximum_position = values
            .iter()
            .position(|value| *value == maximum)
            .expect("the maximum came from this set");

        let deviations: Vec<Rational> = values
            .iter()
            .map(|value| absolute_value(value.sub(full_series_median)))
            .collect();
        let worst = *deviations.iter().max().expect("the window set is nonempty");
        let worst_deviation_positions = deviations
            .iter()
            .enumerate()
            .filter(|(_, deviation)| **deviation == worst)
            .map(|(position, _)| position)
            .collect();

        Self {
            window_count: values.len(),
            minimum: PositionedMedianReport::of(minimum_position, minimum),
            maximum: PositionedMedianReport::of(maximum_position, maximum),
            spread: ExactRationalReport::of(maximum.sub(minimum)),
            worst_absolute_deviation: ExactRationalReport::of(worst),
            worst_deviation_positions,
            medians: values
                .iter()
                .enumerate()
                .map(|(position, value)| PositionedMedianReport::of(position, *value))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests;
