//! The report projection of one located basin's refined outcome.

use serde::Serialize;

use crate::analysis::beta::basin_refinement::BasinRefinement;
use crate::analysis::report::basin_termination_report::BasinTerminationReport;
use crate::analysis::report::beta_candidate_report::BetaCandidateReport;

/// The report projection of one [`BasinRefinement`]: its termination telemetry and the feasible candidate
/// its golden-section refinement produced. A basin always yields a candidate (the frozen search proves it),
/// so this is a struct, not an outcome enum with a per-basin failure — matching the analysis-domain type.
#[derive(Debug, Serialize)]
pub(crate) struct BasinRefinementReport {
    /// This basin's golden-section termination telemetry.
    termination: BasinTerminationReport,
    /// The feasible candidate the refinement produced.
    candidate: BetaCandidateReport,
}

impl BasinRefinementReport {
    /// Project one located basin's refined outcome.
    pub(crate) fn of(refinement: &BasinRefinement) -> Self {
        Self {
            termination: BasinTerminationReport::of(refinement.termination()),
            candidate: BetaCandidateReport::of(refinement.candidate()),
        }
    }
}
