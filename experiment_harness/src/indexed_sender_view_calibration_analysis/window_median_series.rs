//! Every contiguous-window median of one replicate at one candidate `W`, with its position.

use crate::analysis::stats::median::median;
use crate::analysis::stats::rational::Rational;
use crate::indexed_sender_view_calibration_analysis::candidate_window::CandidateWindow;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::exact_samples::exact_samples;

/// The exact median of every contiguous window of width `W`, indexed by the window's 0-based start
/// position in the series.
///
/// **An analysis-domain type, deliberately not `Serialize`** — a domain/report boundary, not a
/// withholding. These medians are [`Rational`]s, and a `Rational` has no wire form of its own; it
/// reaches an artifact only through
/// [`ExactRationalReport`](super::exact_rational_report::ExactRationalReport), which renders it
/// losslessly as canonical components. §615 requires this full position-aware series in the emitted
/// report, and [`WindowStabilityReport`](super::window_stability_report::WindowStabilityReport)
/// carries every element of it alongside the projections derived from it. So nothing here is
/// summarised away — it is projected, exactly as the campaign projects every other analysis value.
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
        let samples = exact_samples(replicate);
        let width = window.get();
        assert!(
            width <= samples.len(),
            "a candidate window cannot be wider than the admitted series; admission fixes the \
             series at the frozen count and the widest candidate is that count"
        );
        // `windows` yields every contiguous slice in ascending start position, so the index of a
        // median in this vector *is* its window's start position.
        let medians = samples.windows(width).map(median).collect();
        Self { window, medians }
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
