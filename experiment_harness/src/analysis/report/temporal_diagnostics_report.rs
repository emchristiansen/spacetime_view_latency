//! The report projection of one cell's temporal diagnostics: plot data, inline SVG, and autocorrelation.

use serde::Serialize;

use crate::analysis::classify::cell_evidence::CellEvidence;
use crate::analysis::report::autocorrelation_report::AutocorrelationReport;
use crate::analysis::report::collection_order_plot_report::CollectionOrderPlotReport;

/// The complete temporal diagnostics of one cell (spec: "join raw blocks, primary `T_block`, and β
/// outcomes only by the proven `collection_order_key`. Serialize both the exact plot data and a
/// deterministic inline SVG ... Compute lag-1 autocorrelation only as a report-boundary ... diagnostic").
/// The exact plot data and the rendered SVG are two views of the same collection-order series, and the
/// lag-1 autocorrelation is the typed report-boundary diagnostic over that series. The SVG is a
/// deterministic string with a fixed canvas, a polyline keyed on the actual collection-order keys, a
/// data-derived y-axis, and the frozen `±δ` band — rendered with no timestamp, randomness, or new
/// dependency.
#[derive(Debug, Serialize)]
pub(crate) struct TemporalDiagnosticsReport {
    /// The exact collection-order plot data (30 `(key, T_block)` points and the `±δ` band).
    plot: CollectionOrderPlotReport,
    /// The deterministic inline SVG rendering of the plot — a genuinely open text payload.
    svg: String,
    /// The lag-1 autocorrelation of the collection-order-sorted totals, a typed report-boundary diagnostic.
    lag1_autocorrelation: AutocorrelationReport,
}

impl TemporalDiagnosticsReport {
    /// Project one cell's temporal diagnostics from its primary evidence. One input — the exact evidence
    /// carrying the 30 `T_block` totals and their aligned collection-order keys — so the plot, its SVG, and
    /// the autocorrelation all describe the same collection-order series.
    pub(crate) fn of(evidence: &CellEvidence) -> Self {
        let _ = evidence;
        todo!("Phase 2: project the plot, render the deterministic SVG, and compute the lag-1 autocorrelation")
    }
}
