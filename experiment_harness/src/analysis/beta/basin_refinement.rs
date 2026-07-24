//! One located grid-local basin's refined outcome: its telemetry and the feasible candidate it produced.

use crate::analysis::beta::basin_termination::BasinTermination;
use crate::analysis::beta::beta_candidate::BetaCandidate;

/// The refined outcome of one located grid-local basin: its termination telemetry and the feasible
/// candidate its golden-section refinement produced.
///
/// A basin never fails to yield a candidate, which the frozen search proves rather than this type merely
/// assuming: basins are enumerated only at nodes passing `grid_rss[k].is_finite()` in
/// [`fit_block`](super::fit_block), so a located basin's grid node already admits a constrained fit;
/// [`golden_section`](super::fit_block) is seeded with that feasible candidate and only ever keeps
/// something at least as good, so it always returns a feasible [`BetaCandidate`]. Hence a basin is a
/// *struct* (telemetry + candidate), not an outcome enum with a per-basin no-scale failure — that failure
/// mode does not exist in the search. The only no-feasible case is a block whose grid located *no* basin
/// at all, recorded once at the [`BlockConvergence`](super::block_convergence::BlockConvergence) level as
/// [`NoFeasiblePositiveScale`](super::basin_selection::BasinSelection::NoFeasiblePositiveScale).
///
/// Fields are private with no defaults; [`Self::new`] is `pub(super)`, so a `BasinRefinement` is
/// assembled only from within the `beta` module. The candidate is the non-`Serialize` exact-boundary
/// [`BetaCandidate`], so this whole type is analysis-domain telemetry, not a report DTO.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BasinRefinement {
    /// This basin's golden-section termination telemetry.
    termination: BasinTermination,
    /// The feasible candidate the refinement produced (always present — see the type doc's code-path
    /// proof).
    candidate: BetaCandidate,
}

impl BasinRefinement {
    /// Bind one basin's refinement telemetry and its produced feasible candidate, asserting the candidate
    /// belongs to *this* basin's refinement rather than another's: its exponent must lie within the
    /// recorded initial bracket. [`golden_section`](super::fit_block) evaluates only points inside its
    /// `[lo0, hi0]` bracket (the seed grid node, both endpoints, and every interior golden-section
    /// probe), so the returned candidate's β is proven to lie in the initial bracket — but *not*
    /// necessarily in the narrowed final bracket, since the winner may be an endpoint the refinement
    /// stepped away from, so no final-bracket assertion is made. `pub(super)` so only the `beta` module's
    /// search can mint one; this assert stops telemetry and a candidate from different basins being paired
    /// despite the private fields.
    pub(super) fn new(termination: BasinTermination, candidate: BetaCandidate) -> Self {
        let beta = candidate.beta();
        assert!(
            beta >= termination.initial_bracket_lo() && beta <= termination.initial_bracket_hi(),
            "candidate β {beta} is outside this basin's initial bracket [{}, {}] — telemetry and \
             candidate are from different basins",
            termination.initial_bracket_lo(),
            termination.initial_bracket_hi()
        );
        Self {
            termination,
            candidate,
        }
    }

    /// This basin's termination telemetry.
    pub(crate) fn termination(&self) -> &BasinTermination {
        &self.termination
    }

    /// The feasible candidate this basin's refinement produced.
    pub(crate) fn candidate(&self) -> BetaCandidate {
        self.candidate
    }
}
