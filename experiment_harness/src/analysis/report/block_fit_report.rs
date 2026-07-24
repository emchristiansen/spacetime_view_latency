//! The report projection of one arm block's secondary-fit outcome.

use serde::Serialize;

use crate::analysis::beta::block_fit::BlockFit;
use crate::analysis::report::beta_candidate_report::BetaCandidateReport;

/// The report projection of one block's [`BlockFit`]. The candidate-retention rule is encoded in the
/// variant shapes exactly as in the analysis-domain type: the two non-identifiable-but-candidate-bearing
/// outcomes carry their candidate, [`NonPositiveScale`](Self::NonPositiveScale) inherently has none, so a
/// "no-scale with a candidate" or a "flat-objective without a candidate" state stays unrepresentable.
#[derive(Debug, Serialize)]
pub(crate) enum BlockFitReport {
    /// An interior, distinguishable, identifiable estimate.
    Identifiable {
        /// The identified candidate.
        candidate: BetaCandidateReport,
    },
    /// The selected exponent is within the bound-pinning threshold of a search-domain bound; the candidate
    /// is retained for diagnostics but is not an identified estimate.
    PinnedAtBound {
        /// The bound-pinned candidate.
        candidate: BetaCandidateReport,
    },
    /// A constrained fit exists at the selected interior exponent but the objective is locally flat there;
    /// the rejected interior candidate is retained for diagnostics.
    FlatObjective {
        /// The rejected interior candidate.
        candidate: BetaCandidateReport,
    },
    /// No exponent admits a constrained fit with strictly positive finite scale — no candidate exists.
    NonPositiveScale,
}

impl BlockFitReport {
    /// Project one block's authoritative fit outcome. Takes the `Copy` [`BlockFit`] by value — one input.
    pub(crate) fn of(fit: BlockFit) -> Self {
        match fit {
            BlockFit::Identifiable(candidate) => Self::Identifiable {
                candidate: BetaCandidateReport::of(candidate),
            },
            BlockFit::PinnedAtBound(candidate) => Self::PinnedAtBound {
                candidate: BetaCandidateReport::of(candidate),
            },
            BlockFit::FlatObjective(candidate) => Self::FlatObjective {
                candidate: BetaCandidateReport::of(candidate),
            },
            BlockFit::NonPositiveScale => Self::NonPositiveScale,
        }
    }
}
