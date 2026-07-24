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
        // The primary `T_block` series is the arm-minus-control paired difference (the same totals the arm
        // interval is computed from), paired position-for-position with its aligned collection-order keys,
        // so the plotted totals and their x-coordinates come from one source. The `±δ` band is the frozen
        // margin's millisecond half-width.
        let points: Vec<CollectionOrderPointReport> = evidence
            .arm_minus_control_totals()
            .iter()
            .zip(evidence.collection_order_keys().iter())
            .map(|(total, key)| {
                CollectionOrderPointReport::new(*key, FiniteF64::new(total.to_f64() / 1_000_000.0))
            })
            .collect();
        let points = points
            .into_boxed_slice()
            .try_into()
            .ok()
            .expect("exactly N_BLOCKS block totals pair with exactly N_BLOCKS collection-order keys");
        Self {
            points,
            delta_band_millis: FiniteF64::new(evidence.margin().to_millis_f64()),
        }
    }

    /// The 30 plotted points, in schedule-proven collection order. Read by the sibling SVG renderer so the
    /// rendered polyline and the serialized plot data are two views of one projected series.
    pub(crate) fn points(&self) -> &[CollectionOrderPointReport; N_BLOCKS] {
        &self.points
    }

    /// The frozen `±δ` equivalence band half-width in milliseconds. Read by the sibling SVG renderer so the
    /// shaded band and the y-domain it must span come from the same projected value the plot serializes.
    pub(crate) fn delta_band_millis(&self) -> FiniteF64 {
        self.delta_band_millis
    }
}

#[cfg(test)]
mod tests;
