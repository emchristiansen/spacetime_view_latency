//! The report projection of the stock-to-patch escalation gate assessment.

use serde::Serialize;

use crate::analysis::report::cell_report::CellReport;
use crate::analysis::report::escalation_recommendation::EscalationRecommendation;
use crate::analysis::validate::validated_campaign::CELL_COUNT;

/// The typed stock-to-patch gate assessment (spec: "Emit a typed stock-to-patch gate assessment derived
/// from the nine same-root cell reports. The assessment never patches or authorizes patching"). It carries
/// the closed [`EscalationRecommendation`] and an open rationale naming which cells' outcomes drove it. It
/// is derived from the nine projected cell reports, so it summarizes the same evidence the report already
/// contains rather than a second source.
#[derive(Debug, Serialize)]
pub(crate) struct EscalationAssessmentReport {
    /// The typed recommendation — remain on stock, or warrant a separately-authorized escalation.
    recommendation: EscalationRecommendation,
    /// The rationale naming the cells and predictions that drove the recommendation — open prose.
    rationale: String,
}

impl EscalationAssessmentReport {
    /// Derive the escalation assessment from the nine same-root cell reports. One input — the fixed
    /// nine-cell array — so the gate reads exactly the report's own cells, never an independent source. It
    /// only records a recommendation; it never patches or authorizes patching.
    ///
    /// Phase 2 reads each cell's typed outcome through
    /// [`CellReport::classification`](crate::analysis::report::cell_report::CellReport::classification) and
    /// keys it to the canonical [`Cell`](crate::plan::cell::Cell) via
    /// `classification().evidence().identity().cell()` — a typed enum match, never a tag-string parse — so
    /// the gate compares each cell's classification against its
    /// [`Cell::predicted_response`](crate::plan::cell::Cell::predicted_response) canonically. Those narrow
    /// read accessors exist today, so this projection compiles against the current report API.
    pub(crate) fn of(cells: &[CellReport; CELL_COUNT]) -> Self {
        let _ = cells;
        todo!("Phase 2: apply the escalation gate over the nine cell outcomes and record the typed recommendation")
    }
}
