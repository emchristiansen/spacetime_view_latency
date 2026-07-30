//! Every contiguous-window median of one replicate at one candidate `W`, with its position.

use crate::analysis::stats::rational::Rational;
use crate::indexed_sender_view_calibration_analysis::candidate_window::CandidateWindow;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;

/// The exact median of every contiguous window of width `W`, indexed by the window's 0-based start
/// position in the series.
///
/// **An analysis-domain type, deliberately not `Serialize`.** At `W = 10` a thousand-sample series
/// has 991 windows; emitting all of them for all seven candidates and both replicates would bury the
/// four things §569's rule actually weighs under about ten thousand numbers. This type is the exact
/// input, and [`WindowStabilityReport`](super::window_stability_report::WindowStabilityReport) is the
/// bounded projection that reaches the artifact — the same separation the campaign keeps between
/// `Rational` and its report projections.
///
/// Windows are **contiguous and position-aware**, which is the whole point: §569 asks whether a
/// within-cell median of width `W` is stable *wherever in the series it is taken*, so a window's
/// start position travels with its median rather than being averaged away.
#[derive(Debug, Clone)]
pub(crate) struct WindowMedianSeries {
    window: CandidateWindow,
    medians: Vec<Rational>,
}

impl WindowMedianSeries {
    /// Every contiguous-window median of `replicate` at `window`, in ascending start position.
    ///
    /// There are exactly `n − W + 1` windows, so `W = 1_000` over a complete series yields exactly
    /// one — the full-series median — and the seven candidates therefore range from many overlapping
    /// views of the series down to a single one.
    ///
    /// Exact throughout: each median is the [`median`](crate::analysis::stats::median::median) of
    /// that window's samples as [`Rational`]s, so an even width averages its two central order
    /// statistics without rounding.
    pub(crate) fn of(replicate: &CompleteReplicate, window: CandidateWindow) -> Self {
        todo!("contiguous-window exact medians in ascending start position")
    }

    /// Which candidate `W` these windows have.
    pub(crate) fn window(&self) -> CandidateWindow {
        self.window
    }

    /// The medians, indexed by 0-based window start position.
    pub(crate) fn medians(&self) -> &[Rational] {
        &self.medians
    }
}

#[cfg(test)]
mod tests;
