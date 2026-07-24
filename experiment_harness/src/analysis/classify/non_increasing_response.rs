//! The arm response taxonomy for a control-valid cell that is *not* Increasing.

/// The observed primary arm response of a control-valid cell whose response is not Increasing (spec:
/// "Make the primary-to-secondary gate unrepresentable in the stored campaign result"). It is
/// deliberately the four-way [`ResponseClass`](super::response_class::ResponseClass) *minus* its
/// `Increasing` variant: an Increasing cell is a distinct
/// [`ClassifiedCell`](super::classified_cell::ClassifiedCell) branch carrying a mandatory secondary
/// descriptor and no independent response field, so "Increasing without a descriptor" and "non-Increasing
/// with a descriptor" are both unrepresentable.
///
/// `Inconclusive` is retained here rather than folded away because it is epistemically unsettled — the
/// spec names the branch `NonIncreasing`, not `Settled`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NonIncreasingResponse {
    /// The arm's paired-difference interval lies wholly within `[-δ, +δ]`.
    FlatEquivalent,
    /// The arm's paired-difference interval lies wholly below `-δ`.
    Decreasing,
    /// The arm's paired-difference interval straddles a band bound.
    Inconclusive,
}
