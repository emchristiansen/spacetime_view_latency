//! The authoritative campaign report: the single, self-contained report projection.

use serde::Serialize;

use crate::analysis::report::cell_report::CellReport;
use crate::analysis::report::environment_report::EnvironmentReport;
use crate::analysis::report::escalation_assessment_report::EscalationAssessmentReport;
use crate::analysis::report::estimator_convergence_report::EstimatorConvergenceReport;
use crate::analysis::report::source_claims_report::SourceClaimsReport;
use crate::analysis::report::stock_observability_report::StockObservabilityReport;
use crate::analysis::validate::validated_campaign::{CELL_COUNT, ValidatedCampaign};

/// The complete authoritative report for one validated campaign (spec: "`CampaignReport::of(&ValidatedCampaign)`
/// is the only report projection"). It is self-contained evidence, not a summary that requires an
/// independently paired file: the campaign-homogeneous environment provenance, the nine per-cell reports
/// (each embedding its raw latency vectors, primary evidence, prediction comparison, temporal diagnostics,
/// and gated secondary β), the six release-qualified mechanism citations plus stock observability limits,
/// and the typed stock-to-patch escalation assessment derived from those nine cells.
///
/// [`Self::of`] is the sole constructor and takes exactly one `&ValidatedCampaign`, so a report can never
/// pair one campaign's cells with another's provenance.
#[derive(Debug, Serialize)]
pub(crate) struct CampaignReport {
    /// The campaign-homogeneous distribution/module provenance, schedule seed, and preregistered
    /// parameters, projected once at the root.
    environment: EnvironmentReport,
    /// The primary estimator's fixed convergence protocol (exact, fixed-30-block, non-iterative,
    /// no-early-stop), stated once at the root because it is identical for every cell.
    primary_estimator: EstimatorConvergenceReport,
    /// The nine per-cell reports, in canonical [`Cell::all`](crate::plan::cell::Cell::all) order. A boxed
    /// fixed array sized from the single frozen [`CELL_COUNT`] source, so the nine-cell cardinality is a
    /// property of the report type (not merely proven by its source) while the campaign report's by-value
    /// footprint stays one pointer — mirroring the trusted graph's `Box<[CellDataset; CELL_COUNT]>`.
    cells: Box<[CellReport; CELL_COUNT]>,
    /// The six release-qualified v2.6.1 mechanism citations.
    source_claims: SourceClaimsReport,
    /// The explicitly reported stock observability limits.
    stock_observability: StockObservabilityReport,
    /// The typed stock-to-patch escalation gate assessment derived from the nine same-root cell reports;
    /// it never patches or authorizes patching.
    escalation: EscalationAssessmentReport,
}

impl CampaignReport {
    /// Project a validated campaign into its authoritative report. The sole report projection and the only
    /// public constructor; every nested projection is derived from this one `&ValidatedCampaign`.
    pub(crate) fn of(campaign: &ValidatedCampaign) -> Self {
        let _ = campaign;
        todo!("Phase 2: project environment, primary-estimator protocol, nine cell reports, citations, observability, escalation")
    }
}
