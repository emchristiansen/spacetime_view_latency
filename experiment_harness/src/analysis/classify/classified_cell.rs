//! The stored per-cell classification, projected from the exact primary result.

use crate::analysis::beta::beta_descriptor::BetaDescriptor;
use crate::analysis::classify::cell_evidence::CellEvidence;
use crate::analysis::classify::non_increasing_response::NonIncreasingResponse;
use crate::analysis::classify::prediction_comparison::PredictionComparison;

/// The stored classification of one arm/regime cell, minted by the exhaustive projection
/// [`BetaDescriptor::project`] from the exact primary
/// [`CellClassification`](crate::analysis::classify::cell_classification::CellClassification) so the
/// primary-to-secondary gate is structurally unrepresentable (spec: "Make the primary-to-secondary gate
/// unrepresentable in the stored campaign result"). Exactly the three spec-recorded branches:
///
/// - [`InvalidControl`](Self::InvalidControl): the control failed the equivalence gate; evidence only, no
///   arm claim.
/// - [`NonIncreasing`](Self::NonIncreasing): a control-valid cell whose arm response is Flat-equivalent,
///   Decreasing, or Inconclusive — carried in a [`NonIncreasingResponse`] that *has no Increasing
///   variant* — with its prediction comparison, and no secondary descriptor.
/// - [`Increasing`](Self::Increasing): a control-valid Increasing cell, carrying its prediction comparison
///   and a mandatory [`BetaDescriptor`], with *no independently variable response field*.
///
/// So "Increasing without a descriptor", "non-Increasing with a descriptor", and "a response field that
/// disagrees with the branch" are all unrepresentable, and the descriptor exists only where the primary
/// result is Increasing.
#[derive(Debug)]
pub(crate) enum ClassifiedCell {
    /// The direct control's interval is not wholly within `[-δ, +δ]`; the whole cell is invalidated and
    /// carries no primary arm claim, but retains its complete evidence.
    InvalidControl { evidence: CellEvidence },
    /// A control-valid cell whose arm response is not Increasing. The [`NonIncreasingResponse`] excludes
    /// Increasing by construction.
    NonIncreasing {
        evidence: CellEvidence,
        response: NonIncreasingResponse,
        comparison: PredictionComparison,
    },
    /// A control-valid Increasing cell, carrying the mandatory secondary descriptor and no response field
    /// (the response is definitionally Increasing).
    Increasing {
        evidence: CellEvidence,
        comparison: PredictionComparison,
        beta: BetaDescriptor,
    },
}

impl ClassifiedCell {
    /// The complete per-cell evidence, present on every branch.
    pub(crate) fn evidence(&self) -> &CellEvidence {
        match self {
            Self::InvalidControl { evidence }
            | Self::NonIncreasing { evidence, .. }
            | Self::Increasing { evidence, .. } => evidence,
        }
    }
}
