//! The one artifact this analyzer emits.

use serde::Serialize;

use crate::indexed_sender_view_calibration_analysis::exact_rational_report::ExactRationalReport;
use crate::indexed_sender_view_calibration_analysis::replicate_diagnostics_report::ReplicateDiagnosticsReport;
use crate::indexed_sender_view_calibration_analysis::replicate_pair::ReplicatePair;
use crate::indexed_sender_view_calibration_analysis::window_diagnostics_report::WindowDiagnosticsReport;

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
        todo!("per-replicate diagnostics and one row per candidate window")
    }
}
