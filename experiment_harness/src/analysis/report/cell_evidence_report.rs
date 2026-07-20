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
        // The exact rational totals, margin, and intervals cross the lossy boundary to finite millisecond
        // values in this one place; the schedule-proven collection-order keys stay exact integers. Each
        // 30-block array is projected heap-first, mirroring the trusted graph's boxed fixed arrays.
        let arm_totals: Vec<FiniteF64> = evidence
            .arm_minus_control_totals()
            .iter()
            .map(|total| FiniteF64::new(total.to_f64() / 1_000_000.0))
            .collect();
        let arm_minus_control_totals_millis = arm_totals
            .into_boxed_slice()
            .try_into()
            .ok()
            .expect("exactly N_BLOCKS arm-minus-control totals project into exactly N_BLOCKS millis values");
        let control_totals: Vec<FiniteF64> = evidence
            .control_totals()
            .iter()
            .map(|total| FiniteF64::new(total.to_f64() / 1_000_000.0))
            .collect();
        let control_totals_millis = control_totals
            .into_boxed_slice()
            .try_into()
            .ok()
            .expect("exactly N_BLOCKS control totals project into exactly N_BLOCKS millis values");
        Self {
            identity: CellIdentityReport::of(evidence.cell()),
            margin_millis: FiniteF64::new(evidence.margin().to_millis_f64()),
            arm_interval: MedianIntervalReport::of(evidence.arm_interval()),
            control_interval: MedianIntervalReport::of(evidence.control_interval()),
            arm_minus_control_totals_millis,
            control_totals_millis,
            collection_order_keys: *evidence.collection_order_keys(),
        }
    }

    /// The typed identity of the cell this evidence classifies. A narrow read accessor so the escalation
    /// gate reads the canonical [`Cell`](crate::plan::cell::Cell) key via
    /// [`CellIdentityReport::cell`](crate::analysis::report::cell_identity_report::CellIdentityReport::cell)
    /// without parsing a tag string.
    pub(crate) fn identity(&self) -> &CellIdentityReport {
        &self.identity
    }
}
