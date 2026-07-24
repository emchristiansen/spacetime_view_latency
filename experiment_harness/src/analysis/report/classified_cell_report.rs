//! The report projection of one cell's stored classification — a sum type mirroring [`ClassifiedCell`].

use serde::Serialize;

use crate::analysis::classify::classified_cell::ClassifiedCell;
use crate::analysis::classify::prediction_comparison::PredictionComparison;
use crate::analysis::report::cell_evidence_report::CellEvidenceReport;
use crate::analysis::report::non_increasing_response_report::NonIncreasingResponseReport;
use crate::analysis::report::secondary_descriptor_report::SecondaryDescriptorReport;

/// The report projection of one cell's stored [`ClassifiedCell`]. It is itself a sum type mirroring the
/// analysis-domain classification variant-for-variant, so the primary-to-secondary gate stays structural
/// in the serialized result exactly as in the trusted one: the secondary descriptor exists *only* in the
/// [`Increasing`](Self::Increasing) branch, the arm response *only* in the
/// [`NonIncreasing`](Self::NonIncreasing) branch, and [`InvalidControl`](Self::InvalidControl) carries
/// evidence alone. "Increasing without a descriptor", "non-Increasing with a descriptor", a response field
/// that disagrees with the branch, and an independently-variable optional β are all unrepresentable —
/// there is no separate primary + optional beta/comparison. The prediction comparison reuses the
/// already-`Serialize` domain [`PredictionComparison`] directly.
#[derive(Debug, Serialize)]
pub(crate) enum ClassifiedCellReport {
    /// The direct control's interval is not wholly within `[-δ, +δ]`; the cell is invalidated and carries
    /// no primary arm claim, but retains its complete evidence.
    InvalidControl {
        /// The complete per-cell primary evidence.
        evidence: CellEvidenceReport,
    },
    /// A control-valid cell whose arm response is not Increasing, with its prediction comparison and no
    /// secondary descriptor. The [`NonIncreasingResponseReport`] excludes Increasing by construction.
    NonIncreasing {
        /// The complete per-cell primary evidence.
        evidence: CellEvidenceReport,
        /// How the observed response compares with the cell's preregistered prediction.
        comparison: PredictionComparison,
        /// The observed non-Increasing arm response.
        response: NonIncreasingResponseReport,
    },
    /// A control-valid Increasing cell, carrying its prediction comparison and the mandatory secondary
    /// descriptor, and no independent response field (the response is definitionally Increasing).
    Increasing {
        /// The complete per-cell primary evidence.
        evidence: CellEvidenceReport,
        /// How the observed response compares with the cell's preregistered prediction.
        comparison: PredictionComparison,
        /// The mandatory secondary `L(N)=a+bN^β` descriptor of this gated Increasing cell.
        beta: SecondaryDescriptorReport,
    },
}

impl ClassifiedCellReport {
    /// Project one cell's stored classification. One input — the exact [`ClassifiedCell`] — so a cell's
    /// classification can never be paired with foreign evidence or a foreign descriptor.
    pub(crate) fn of(classified: &ClassifiedCell) -> Self {
        // Mirror the stored classification variant-for-variant: the secondary descriptor projects only in
        // the Increasing branch and the arm response only in the NonIncreasing branch, so the structural
        // gate is preserved in the serialized result.
        match classified {
            ClassifiedCell::InvalidControl { evidence } => Self::InvalidControl {
                evidence: CellEvidenceReport::of(evidence),
            },
            ClassifiedCell::NonIncreasing {
                evidence,
                response,
                comparison,
            } => Self::NonIncreasing {
                evidence: CellEvidenceReport::of(evidence),
                comparison: *comparison,
                response: NonIncreasingResponseReport::of(*response),
            },
            ClassifiedCell::Increasing {
                evidence,
                comparison,
                beta,
            } => Self::Increasing {
                evidence: CellEvidenceReport::of(evidence),
                comparison: *comparison,
                beta: SecondaryDescriptorReport::of(beta),
            },
        }
    }

    /// The complete per-cell primary evidence, present on *every* branch — mirroring
    /// [`ClassifiedCell::evidence`](crate::analysis::classify::classified_cell::ClassifiedCell::evidence).
    /// A narrow read accessor so a consumer (e.g. the escalation gate) can reach a cell's canonical identity
    /// via [`CellEvidenceReport::identity`](crate::analysis::report::cell_evidence_report::CellEvidenceReport::identity)
    /// without matching the classification branch.
    pub(crate) fn evidence(&self) -> &CellEvidenceReport {
        match self {
            Self::InvalidControl { evidence }
            | Self::NonIncreasing { evidence, .. }
            | Self::Increasing { evidence, .. } => evidence,
        }
    }
}
