//! The exact collection-order plot data of one cell's primary `T_block` temporal series.

use serde::Serialize;

use crate::analysis::classify::cell_evidence::CellEvidence;
use crate::analysis::finite_f64::FiniteF64;
use crate::analysis::report::collection_order_point_report::CollectionOrderPointReport;
use crate::params::REPETITION_BLOCKS;

/// The fixed 30-block per-cell sample size as an array length, sized from the single frozen
/// [`REPETITION_BLOCKS`] source, so the plot's point count is a property of the type.
const N_BLOCKS: usize = REPETITION_BLOCKS as usize;

/// The exact plot data for one cell's collection-order temporal diagnostic (spec: "Serialize both the
/// exact plot data and a deterministic inline SVG"): the 30 primary `T_block` totals plotted against their
/// schedule-proven collection-order keys, plus the frozen `±δ` equivalence band half-width. The points
/// preserve global-schedule gaps by keying on the actual collection-order sequence; the band half-width
/// crosses the lossy boundary to a finite [`FiniteF64`] millisecond value. A boxed fixed array keeps the
/// 30-point cardinality a property of the type.
#[derive(Debug, Serialize)]
pub(crate) struct CollectionOrderPlotReport {
    /// The 30 plotted `(collection-order key, block total)` points, in schedule-proven collection order.
    points: Box<[CollectionOrderPointReport; N_BLOCKS]>,
    /// The frozen `±δ` equivalence band half-width, in milliseconds.
    delta_band_millis: FiniteF64,
}

impl CollectionOrderPlotReport {
    /// Project one cell's plot data from its primary evidence. One input — the exact evidence carrying both
    /// the 30 `T_block` totals and their aligned collection-order keys — so the plotted totals and their
    /// x-coordinates cannot come from different sources.
    pub(crate) fn of(evidence: &CellEvidence) -> Self {
        let _ = evidence;
        todo!("Phase 2: pair each block total with its collection-order key and project the delta band")
    }
}
