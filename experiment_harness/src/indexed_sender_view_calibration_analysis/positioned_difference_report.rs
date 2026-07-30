//! One extremal between-replicate difference, with every window position attaining it.

use serde::Serialize;

use crate::analysis::stats::rational::Rational;
use crate::indexed_sender_view_calibration_analysis::exact_rational_report::ExactRationalReport;

/// An extremal per-position difference together with **every** window position that attains it.
///
/// Ties are retained rather than resolved, for the same reason they are in
/// [`WindowStabilityReport`](super::window_stability_report::WindowStabilityReport): a disagreement
/// that peaks at one window early in both runs is a different fact from one that recurs at forty
/// positions throughout, and picking a representative position would report the first story for
/// both.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct PositionedDifferenceReport {
    /// The extremal difference itself.
    difference: ExactRationalReport,
    /// Every 0-based window start position attaining it, ascending.
    positions: Vec<usize>,
}

impl PositionedDifferenceReport {
    /// Pair one extremal difference with all the positions attaining it.
    pub(crate) fn of(difference: Rational, positions: Vec<usize>) -> Self {
        Self {
            difference: ExactRationalReport::of(difference),
            positions,
        }
    }
}
