//! The per-replicate diagnostics that do not depend on a candidate width.

use serde::Serialize;

use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::exact_rational_report::ExactRationalReport;
use crate::indexed_sender_view_calibration_analysis::lag_autocorrelation::LagAutocorrelation;
use crate::indexed_sender_view_calibration_analysis::trend_report::TrendReport;

/// One replicate's whole-series diagnostics: its exact full-series median, its early/late drift, and
/// its lag-1 dependence.
///
/// Separate from the per-`W` rows because these are properties of the *run*, not of a candidate
/// width. Reporting the lag coefficient once per candidate would suggest it varies with `W`, which
/// it does not — it is computed over the whole series exactly once.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ReplicateDiagnosticsReport {
    /// Which replicate this is.
    replicate: u32,
    /// How many samples it retained — the frozen count, by admission.
    samples: usize,
    /// The exact median of the whole series, and the centre every window deviation is measured
    /// against.
    full_series_median: ExactRationalReport,
    /// Early-versus-late drift, as two exact segment medians and their difference.
    trend: TrendReport,
    /// Dependence at every lag in the derived domain, ascending.
    ///
    /// A **vector**, not a single lag. §568 asks for "lag dependence/effective information", which
    /// is a statement about dependence across lags; reporting only `k = 1` would answer a narrower
    /// question than the rule asks. The domain itself is
    /// [`lag_domain`](super::lag_domain::lag_domain) — derived, and currently pending a Control
    /// decision recorded there.
    lags: Vec<LagAutocorrelation>,
}

impl ReplicateDiagnosticsReport {
    /// Compute one replicate's whole-series diagnostics.
    ///
    /// Takes the already-computed full-series median so the value in this report and the centre used
    /// by every [`WindowStabilityReport`](super::window_stability_report::WindowStabilityReport) are
    /// one and the same number rather than two independent computations that agree by luck.
    pub(crate) fn of(
        replicate: &CompleteReplicate,
        full_series_median: crate::analysis::stats::rational::Rational,
    ) -> Self {
        todo!("full-series median, trend, and lag for one replicate")
    }
}
