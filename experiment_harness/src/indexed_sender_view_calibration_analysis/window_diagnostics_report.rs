//! Everything §569 asks about one candidate `W`.

use serde::Serialize;

use crate::indexed_sender_view_calibration_analysis::between_replicate_report::BetweenReplicateReport;
use crate::indexed_sender_view_calibration_analysis::candidate_window::CandidateWindow;
use crate::indexed_sender_view_calibration_analysis::replicate_pair::ReplicatePair;
use crate::indexed_sender_view_calibration_analysis::window_stability_report::WindowStabilityReport;

/// One candidate `W`'s complete diagnostic row: within-replicate window stability for **both**
/// replicates, plus their disagreement.
///
/// Both replicates appear side by side rather than merged, because §569 evaluates each candidate
/// against both complete retained series. Nothing here ranks this candidate against another, scores
/// it, or marks it acceptable — the report emits one of these per candidate and stops.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct WindowDiagnosticsReport {
    /// Which candidate `W` this row is about.
    window: CandidateWindow,
    /// The lower-ordinal replicate's window-median stability at this width.
    first_replicate: WindowStabilityReport,
    /// The higher-ordinal replicate's window-median stability at this width.
    second_replicate: WindowStabilityReport,
    /// How far the two replicates disagree at this width.
    between_replicates: BetweenReplicateReport,
}

impl WindowDiagnosticsReport {
    /// Compute one candidate width's diagnostics over both replicates.
    ///
    /// The two full-series medians are passed in rather than recomputed per width: they are
    /// properties of the replicates, not of the candidate, and recomputing them for each of the
    /// seven candidates would repeat a thousand-element median fourteen times while risking two
    /// rows of one table being measured against different centres.
    pub(crate) fn of(
        pair: &ReplicatePair,
        window: CandidateWindow,
        first_full_series_median: crate::analysis::stats::rational::Rational,
        second_full_series_median: crate::analysis::stats::rational::Rational,
    ) -> Self {
        todo!("both replicates' stability at this width, plus their disagreement")
    }
}
