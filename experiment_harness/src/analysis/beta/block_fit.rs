//! The per-block secondary-fit outcome: identifiable estimate, bound-pinned, or non-identifiable.

use crate::analysis::beta::beta_candidate::BetaCandidate;
use crate::analysis::beta::non_identifiable_reason::NonIdentifiableReason;

/// The outcome of the frozen exponent search over one arm block's ten-dose ladder. Every block reports
/// exactly one of these (spec: "report all 30 block outcomes and preserve non-identifiable failures");
/// only [`Identifiable`](Self::Identifiable) contributes to the population β interval.
///
/// The candidate-retention rule is encoded in the variant shapes rather than a `reason + Option`
/// product: [`NonPositiveScale`](Self::NonPositiveScale) inherently has no numeric candidate, while
/// [`FlatObjective`](Self::FlatObjective) always retains the flat interior candidate it rejected — so a
/// "no-scale with a candidate" or a "flat-objective without a candidate" state is unrepresentable.
#[derive(Debug, Clone, Copy)]
pub(crate) enum BlockFit {
    /// The search selected an interior exponent whose constrained fit is finite with `b > 0` and whose
    /// RSS is strictly below every *feasible* `β ± 0.001` probe (those whose exponent stays within the
    /// `[0.1, 4.0]` domain; an out-of-domain probe is omitted) by more than the relative tolerance — a
    /// distinguishable, identifiable estimate.
    Identifiable(BetaCandidate),
    /// The selected exponent lies within `1e-4` of a search-domain bound (`0.1` or `4.0`). The numeric
    /// candidate is retained for diagnostics, but a bound-pinned exponent is not an identified estimate
    /// (spec: "reported with `PinnedAtBound` status and remains non-identifiable").
    PinnedAtBound(BetaCandidate),
    /// No exponent in the search domain admits a constrained fit with strictly positive finite scale
    /// `b`, so no numeric candidate exists at all (spec: "no fit exists when no candidate has strictly
    /// positive finite `b`").
    NonPositiveScale,
    /// A constrained fit exists at the selected interior exponent, but the residual-sum objective is
    /// locally flat there — its RSS is not strictly below every *feasible* `β ± 0.001` probe by more than
    /// the relative tolerance. The rejected interior candidate is retained for diagnostics.
    FlatObjective(BetaCandidate),
}

impl BlockFit {
    /// Whether this block yielded an identifiable exponent estimate — the sole outcome that contributes
    /// to the population β interval.
    pub(crate) fn is_identifiable(self) -> bool {
        matches!(self, Self::Identifiable(_))
    }

    /// Whether this block's selected exponent was pinned within `1e-4` of a search-domain bound.
    pub(crate) fn is_pinned_at_bound(self) -> bool {
        matches!(self, Self::PinnedAtBound(_))
    }

    /// The identified exponent, present only on the [`Identifiable`](Self::Identifiable) outcome.
    pub(crate) fn identified_beta(self) -> Option<f64> {
        match self {
            Self::Identifiable(candidate) => Some(candidate.beta()),
            Self::PinnedAtBound(_) | Self::NonPositiveScale | Self::FlatObjective(_) => None,
        }
    }

    /// The numeric candidate retained by this outcome, if any: the fit on
    /// [`Identifiable`](Self::Identifiable)/[`PinnedAtBound`](Self::PinnedAtBound), and the rejected
    /// interior candidate on [`FlatObjective`](Self::FlatObjective). [`NonPositiveScale`](Self::NonPositiveScale)
    /// structurally has none.
    pub(crate) fn candidate(self) -> Option<BetaCandidate> {
        match self {
            Self::Identifiable(candidate)
            | Self::PinnedAtBound(candidate)
            | Self::FlatObjective(candidate) => Some(candidate),
            Self::NonPositiveScale => None,
        }
    }

    /// The coarse non-identifiability taxonomy, derived from the variant. Present on both
    /// non-identifiable outcomes and absent on [`Identifiable`](Self::Identifiable)/[`PinnedAtBound`](Self::PinnedAtBound);
    /// `PinnedAtBound` is a distinct bound-diagnostic outcome, not a `NonIdentifiableReason`.
    pub(crate) fn non_identifiable_reason(self) -> Option<NonIdentifiableReason> {
        match self {
            Self::NonPositiveScale => Some(NonIdentifiableReason::NonPositiveScale),
            Self::FlatObjective(_) => Some(NonIdentifiableReason::FlatObjective),
            Self::Identifiable(_) | Self::PinnedAtBound(_) => None,
        }
    }
}
