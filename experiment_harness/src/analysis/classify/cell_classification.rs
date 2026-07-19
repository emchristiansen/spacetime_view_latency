//! The per-cell classification sum type that structurally gates the arm claim on control validity.

use crate::analysis::classify::cell_evidence::CellEvidence;
use crate::analysis::classify::prediction_comparison::PredictionComparison;
use crate::analysis::classify::response_class::ResponseClass;

/// The final classification of one arm/regime cell. The whole-cell control-validity gate is structural:
/// only the [`Valid`](Self::Valid) branch contains the arm's response and prediction comparison, so a
/// cell whose direct control fails the flat-equivalence test cannot expose a primary arm claim (spec:
/// "Make complete-campaign validation and statistical control validity distinct typed outcomes").
/// Both branches retain the same complete [`CellEvidence`].
#[derive(Debug)]
pub(crate) enum CellClassification {
    /// The direct control's total-change interval lies within `[-δ, +δ]`, so the cell is
    /// environmentally valid and carries the arm's four-way response and its prediction comparison.
    Valid {
        evidence: CellEvidence,
        response: ResponseClass,
        comparison: PredictionComparison,
    },
    /// The direct control's interval is not wholly within `[-δ, +δ]`; the whole cell is invalidated and
    /// carries no primary arm claim, but retains its complete evidence and diagnostics.
    InvalidControl { evidence: CellEvidence },
}

impl CellClassification {
    /// Gate the cell on control validity: when the control interval lies within the frozen band, mint
    /// [`Valid`](Self::Valid) with the arm's [`ResponseClass`] and [`PredictionComparison`]; otherwise
    /// mint [`InvalidControl`](Self::InvalidControl) with the same evidence and no arm claim.
    pub(crate) fn classify(evidence: CellEvidence) -> Self {
        // Whole-cell control-validity gate: the direct-table control's total-change interval must lie
        // wholly within the frozen band `[-δ, +δ]` (both bounds inclusive), using exact rational
        // comparison. Only then does the arm response and its prediction comparison become expressible.
        let margin = evidence.margin();
        let control = evidence.control_interval();
        let control_valid = control.lo() >= margin.lower() && control.hi() <= margin.upper();
        if control_valid {
            let response = ResponseClass::classify(evidence.arm_interval(), margin);
            let comparison =
                PredictionComparison::compare(evidence.cell().predicted_response(), response);
            Self::Valid {
                evidence,
                response,
                comparison,
            }
        } else {
            Self::InvalidControl { evidence }
        }
    }

    /// The complete per-cell evidence, present on both branches.
    pub(crate) fn evidence(&self) -> &CellEvidence {
        match self {
            Self::Valid { evidence, .. } | Self::InvalidControl { evidence } => evidence,
        }
    }
}
