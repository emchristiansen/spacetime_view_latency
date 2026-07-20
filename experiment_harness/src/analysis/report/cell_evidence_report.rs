//! The report projection of one cell's complete fixed-cardinality primary evidence.

use serde::Serialize;

use crate::analysis::classify::cell_evidence::CellEvidence;
use crate::analysis::finite_f64::FiniteF64;
use crate::analysis::report::cell_identity_report::CellIdentityReport;
use crate::analysis::report::median_interval_report::MedianIntervalReport;
use crate::observation::record_seq::RecordSeq;
use crate::params::REPETITION_BLOCKS;

/// The fixed 30-block per-cell sample size as an array length, sized from the single frozen
/// [`REPETITION_BLOCKS`] source rather than a re-typed literal, so the "retain every fixed block estimate"
/// obligation is a property of the report type.
const N_BLOCKS: usize = REPETITION_BLOCKS as usize;

/// The report projection of one cell's [`CellEvidence`] — the complete primary evidence retained
/// identically whether the control is valid or not (spec: "retain every fixed block estimate"). The exact
/// [`Rational`](crate::analysis::stats::rational::Rational) totals and [`MedianCi`](crate::analysis::stats::median_ci::MedianCi)
/// intervals cross the lossy floating-point boundary here into finite millisecond [`FiniteF64`] values;
/// the schedule-proven collection-order keys stay exact [`RecordSeq`] integers. Every per-block array is a
/// boxed fixed `N_BLOCKS` array, so the 30-block cardinality is a property of the type.
#[derive(Debug, Serialize)]
pub(crate) struct CellEvidenceReport {
    /// The cell this evidence classifies.
    identity: CellIdentityReport,
    /// The frozen practical-equivalence margin `δ`, in milliseconds.
    margin_millis: FiniteF64,
    /// The exact 95% median interval of the arm's paired-difference total change, in milliseconds.
    arm_interval: MedianIntervalReport,
    /// The exact 95% median interval of the direct control's total change, in milliseconds.
    control_interval: MedianIntervalReport,
    /// The 30 per-block arm-minus-control `T_block` total changes, in schedule-proven collection order, in
    /// milliseconds.
    arm_minus_control_totals_millis: Box<[FiniteF64; N_BLOCKS]>,
    /// The 30 per-block direct-control `T_block` total changes, in the same order, in milliseconds.
    control_totals_millis: Box<[FiniteF64; N_BLOCKS]>,
    /// The schedule-proven collection-order key of each block, aligned with the two total arrays.
    collection_order_keys: [RecordSeq; N_BLOCKS],
}

impl CellEvidenceReport {
    /// Project one cell's complete primary evidence. One input — the exact evidence — projected in one
    /// place across the lossy millisecond boundary.
    pub(crate) fn of(evidence: &CellEvidence) -> Self {
        let _ = evidence;
        todo!("Phase 2: project identity, margin, both intervals, and the two 30-block total arrays to millis")
    }

    /// The typed identity of the cell this evidence classifies. A narrow read accessor so the escalation
    /// gate reads the canonical [`Cell`](crate::plan::cell::Cell) key via
    /// [`CellIdentityReport::cell`](crate::analysis::report::cell_identity_report::CellIdentityReport::cell)
    /// without parsing a tag string.
    pub(crate) fn identity(&self) -> &CellIdentityReport {
        &self.identity
    }
}
