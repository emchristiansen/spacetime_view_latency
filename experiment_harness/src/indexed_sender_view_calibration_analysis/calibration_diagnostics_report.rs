//! The one artifact this analyzer emits.

use serde::Serialize;

use crate::analysis::stats::rational::Rational;
use crate::indexed_sender_view_calibration_analysis::candidate_window::CandidateWindow;
use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::exact_rational_report::ExactRationalReport;
use crate::indexed_sender_view_calibration_analysis::replicate_diagnostics_report::ReplicateDiagnosticsReport;
use crate::indexed_sender_view_calibration_analysis::replicate_pair::ReplicatePair;
use crate::indexed_sender_view_calibration_analysis::window_diagnostics_report::WindowDiagnosticsReport;
use crate::indexed_sender_view_calibration_analysis::window_median_series::WindowMedianSeries;

/// The complete §569 diagnostics: per-replicate whole-series facts, and one row per candidate `W`.
///
/// **The ceiling is enforced by what this type cannot say.** There is no field anywhere in it — or in
/// anything it contains — for a verdict, a recommendation, a chosen `W`, a pass/fail, a threshold, a
/// score, or a cross-candidate comparison. Not "must not be written": there is no shape to write one
/// through. Choosing `W` is Control's act in SSOT, informed by these numbers.
///
/// **Named `…DiagnosticsReport`, never `…Result` or `…Verdict`**, for the same reason the pilot's
/// success outcome is `CalibrationRecorded` rather than `Complete`: a name that sounds like an answer
/// invites being read as one.
///
/// The seven candidate rows are emitted in ascending `W`, which is the order §569 discusses them in.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct CalibrationDiagnosticsReport {
    /// The lower-ordinal replicate's whole-series diagnostics.
    first_replicate: ReplicateDiagnosticsReport,
    /// The higher-ordinal replicate's whole-series diagnostics.
    second_replicate: ReplicateDiagnosticsReport,
    /// The exact `second − first` difference of the two replicates' full-series medians.
    ///
    /// **Stated once, here, because it does not vary with `W`.** It belongs to the pair rather than
    /// to any candidate row, and repeating it in all seven rows would invite a reader to look for a
    /// width effect in a constant. It is the baseline the per-position disagreements are read
    /// against: how far apart the two runs were overall.
    full_series_median_difference: ExactRationalReport,
    /// One row per candidate `W`, ascending — exactly the seven §569 names.
    candidates: Vec<WindowDiagnosticsReport>,
}

impl CalibrationDiagnosticsReport {
    /// Compute the whole report from the admitted pair.
    ///
    /// Each replicate's full-series median is computed **once** here and threaded into both its own
    /// whole-series row and every candidate row, so "deviation from the full-series median" means
    /// one fixed number throughout the artifact.
    pub(crate) fn of(pair: &ReplicatePair) -> Self {
        let first_median = full_series_median(pair.first());
        let second_median = full_series_median(pair.second());
        Self {
            first_replicate: ReplicateDiagnosticsReport::of(pair.first(), first_median),
            second_replicate: ReplicateDiagnosticsReport::of(pair.second(), second_median),
            full_series_median_difference: ExactRationalReport::of(second_median.sub(first_median)),
            candidates: CandidateWindow::ALL
                .into_iter()
                .map(|window| {
                    WindowDiagnosticsReport::of(pair, window, first_median, second_median)
                })
                .collect(),
        }
    }
}

/// One replicate's exact full-series median.
///
/// Taken as the sole window median at the **widest candidate** rather than computed separately. A
/// complete series admits exactly one window of that width, so the two definitions coincide — and
/// routing through the same code guarantees it, instead of leaving the artifact's stated centre and
/// the centre its deviations are measured against as two computations that agree by luck.
fn full_series_median(replicate: &CompleteReplicate) -> Rational {
    let widest = WindowMedianSeries::of(replicate, CandidateWindow::W1000);
    let medians = widest.medians();
    assert_eq!(
        medians.len(),
        1,
        "a complete series admits exactly one window of the widest candidate"
    );
    medians[0]
}
