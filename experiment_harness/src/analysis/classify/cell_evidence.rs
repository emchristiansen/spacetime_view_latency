//! The fixed-cardinality per-cell primary evidence.

use crate::analysis::classify::equivalence_margin::EquivalenceMargin;
use crate::analysis::stats::median_ci::{exact_median_ci_30, MedianCi};
use crate::analysis::stats::rational::Rational;
use crate::observation::record_seq::RecordSeq;
use crate::params::REPETITION_BLOCKS;
use crate::plan::cell::Cell;

/// The fixed number of complete randomized repetition blocks per cell (spec: exactly 30). It fixes the
/// length of every per-block estimate array, so the preregistered 30-block census is compiler-visible
/// and an off-count sample is a compile error rather than a runtime check.
const N_BLOCKS: usize = REPETITION_BLOCKS as usize;

/// The complete primary evidence for one arm/regime [`Cell`], retained identically whether the cell's
/// control is valid or not (spec: "Encode the primary gate structurally and retain every fixed block
/// estimate"). It owns, at fixed cardinality: all 30 arm-minus-control per-block total changes, all 30
/// direct-control per-block total changes, their schedule-proven collection-order keys, the frozen
/// equivalence margin, and both exact median confidence intervals.
///
/// The exact [`Rational`] and [`MedianCi`] evidence stays sealed behind private fields; the report
/// renders lossy display values from the accessors, never mutating the exact evidence.
#[derive(Debug)]
pub(crate) struct CellEvidence {
    /// The arm/regime cell this evidence classifies.
    cell: Cell,
    /// The 30 per-block `T_block` values of the arm-minus-control paired difference `D(N)`, in
    /// schedule-proven collection order.
    arm_minus_control_totals: [Rational; N_BLOCKS],
    /// The 30 per-block `T_block` values of the direct-table control alone, in the same order — the
    /// input to the independent control-validity gate.
    control_totals: [Rational; N_BLOCKS],
    /// The schedule-proven collection-order key of each block, aligned with the two total arrays.
    collection_order_keys: [RecordSeq; N_BLOCKS],
    /// The frozen practical-equivalence margin `δ`, shared by the arm response and the control gate.
    margin: EquivalenceMargin,
    /// The exact 95% median interval of the arm's paired-difference total change.
    arm_interval: MedianCi,
    /// The exact 95% median interval of the direct control's total change.
    control_interval: MedianCi,
}

impl CellEvidence {
    /// Bind the per-block estimates and frozen margin, selecting both exact `[X_(10), X_(21)]` median
    /// intervals from the fixed 30-block samples. The caller (the campaign classifier) owns computing
    /// the per-block total changes, collection-order keys, and margin from the trusted graph; this
    /// constructor only selects the two intervals and binds the evidence.
    pub(crate) fn new(
        cell: Cell,
        arm_minus_control_totals: [Rational; N_BLOCKS],
        control_totals: [Rational; N_BLOCKS],
        collection_order_keys: [RecordSeq; N_BLOCKS],
        margin: EquivalenceMargin,
    ) -> Self {
        let arm_interval = exact_median_ci_30(&arm_minus_control_totals);
        let control_interval = exact_median_ci_30(&control_totals);
        Self {
            cell,
            arm_minus_control_totals,
            control_totals,
            collection_order_keys,
            margin,
            arm_interval,
            control_interval,
        }
    }

    /// The arm/regime cell this evidence classifies.
    pub(crate) fn cell(&self) -> Cell {
        self.cell
    }

    /// The frozen practical-equivalence margin.
    pub(crate) fn margin(&self) -> EquivalenceMargin {
        self.margin
    }

    /// The exact median interval of the arm's paired-difference total change.
    pub(crate) fn arm_interval(&self) -> MedianCi {
        self.arm_interval
    }

    /// The exact median interval of the direct control's total change — the control-validity input.
    pub(crate) fn control_interval(&self) -> MedianCi {
        self.control_interval
    }

    /// The 30 arm-minus-control per-block total changes, in schedule-proven collection order.
    pub(crate) fn arm_minus_control_totals(&self) -> &[Rational; N_BLOCKS] {
        &self.arm_minus_control_totals
    }

    /// The 30 direct-control per-block total changes, in the same order.
    pub(crate) fn control_totals(&self) -> &[Rational; N_BLOCKS] {
        &self.control_totals
    }

    /// The schedule-proven collection-order key of each block, aligned with the total arrays.
    pub(crate) fn collection_order_keys(&self) -> &[RecordSeq; N_BLOCKS] {
        &self.collection_order_keys
    }
}
