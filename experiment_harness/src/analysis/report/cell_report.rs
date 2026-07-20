//! The report projection of one preregistered cell: raw observations, classification, and diagnostics.

use serde::Serialize;

use crate::analysis::report::block_report::BlockReport;
use crate::analysis::report::classified_cell_report::ClassifiedCellReport;
use crate::analysis::report::temporal_diagnostics_report::TemporalDiagnosticsReport;
use crate::analysis::validate::cell_dataset::CellDataset;
use crate::params::REPETITION_BLOCKS;

/// The fixed 30-block per-cell sample size as an array length, sized from the single frozen
/// [`REPETITION_BLOCKS`] source, so the 30 raw-observation blocks are a property of the report type.
const N_BLOCKS: usize = REPETITION_BLOCKS as usize;

/// The complete report for one preregistered cell (spec: each `CellReport` "receives one `&CellDataset`,
/// internally derives" its classification, "and projects raw observations, primary evidence, prediction
/// comparison, diagnostics, and secondary β from that same cell"). It carries the raw per-block arm/control
/// observations, the single stored [`ClassifiedCellReport`] classification value (primary evidence +
/// prediction comparison + gate-structural secondary β, never a separate optional beta), and the temporal
/// diagnostics — all derived from the one dataset. The cell's full classification is derived by a single
/// [`BetaDescriptor::project`](crate::analysis::beta::beta_descriptor::BetaDescriptor::project)`(dataset)` —
/// the crate's sole one-input authoritative classification producer, which internally computes the primary
/// [`classify_cell`](crate::analysis::classify::classify_cell::classify_cell) result and invokes its
/// module-private β fit solely in the Increasing arm, so β is computed only for an Increasing cell. The
/// exhaustive [`ClassifiedCell`](crate::analysis::classify::classified_cell::ClassifiedCell) it returns —
/// whose Increasing branch alone carries a descriptor — is the stored value the report projects.
#[derive(Debug, Serialize)]
pub(crate) struct CellReport {
    /// The stored classification: primary evidence, prediction comparison, and the gate-structural
    /// response/secondary-β, mirroring [`ClassifiedCell`](crate::analysis::classify::classified_cell::ClassifiedCell).
    classification: ClassifiedCellReport,
    /// The 30 matched blocks' raw arm/control observations, in schedule-proven collection order — the
    /// self-contained raw evidence, a boxed fixed array so the 30-block cardinality is a property of the
    /// type.
    observations: Box<[BlockReport; N_BLOCKS]>,
    /// The cell's temporal diagnostics (collection-order plot, inline SVG, lag-1 autocorrelation).
    diagnostics: TemporalDiagnosticsReport,
}

impl CellReport {
    /// Project one cell's complete report from its dataset. The sole input is `&CellDataset`. The cell's
    /// full classification is derived by exactly one call to
    /// [`BetaDescriptor::project`](crate::analysis::beta::beta_descriptor::BetaDescriptor::project)`(dataset)`,
    /// the crate's one-input authoritative classification producer: it internally computes the primary
    /// [`classify_cell`](crate::analysis::classify::classify_cell::classify_cell) result and invokes its
    /// module-private β fit solely in the Increasing arm, so β never runs for a non-Increasing cell and no
    /// outer classification flow is duplicated here. The returned
    /// [`ClassifiedCell`](crate::analysis::classify::classified_cell::ClassifiedCell) projects into a
    /// [`ClassifiedCellReport`]; the raw observations come from the dataset's blocks and the diagnostics
    /// from the derived primary evidence, so a cell's classification, raw evidence, and diagnostics all
    /// share one source.
    pub(crate) fn of(dataset: &CellDataset) -> Self {
        let _ = dataset;
        todo!("Phase 2: BetaDescriptor::project(dataset) once; project the returned ClassifiedCell, the 30 raw blocks, and diagnostics")
    }

    /// This cell's stored classification — the typed outcome the stock-to-patch escalation gate inspects
    /// per cell (spec: the assessment is "derived from the nine same-root cell reports"). A narrow read
    /// accessor so [`EscalationAssessmentReport::of`](crate::analysis::report::escalation_assessment_report::EscalationAssessmentReport::of)
    /// can read the typed [`ClassifiedCellReport`] — and, through it, the cell's canonical identity — without
    /// owning or re-deriving it.
    pub(crate) fn classification(&self) -> &ClassifiedCellReport {
        &self.classification
    }
}
