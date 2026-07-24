//! One arm block's authoritative fit bound to its search-convergence record in a single value.

use crate::analysis::beta::basin_selection::BasinSelection;
use crate::analysis::beta::block_convergence::BlockConvergence;
use crate::analysis::beta::block_fit::BlockFit;

/// One arm block's complete secondary-search result: the authoritative
/// [`BlockFit`](super::block_fit::BlockFit) outcome and the [`BlockConvergence`] record of the search
/// that produced it, bound together so a fit can never be paired with a *different* block's convergence
/// record. The two are minted only through [`Self::mint`], which asserts they agree — there is no way to
/// hold a `BlockSearchOutcome` whose fit and convergence describe different searches.
///
/// The fit remains the authoritative result and its candidate ordering/selection is unchanged (spec:
/// "record termination without changing candidate ordering or selection"); this type only *witnesses*
/// that the recorded convergence's selected basin is exactly the fit's candidate.
///
/// Fields are private with no defaults; [`Self::mint`] is `pub(super)`, so a `BlockSearchOutcome` is
/// assembled only from within the `beta` module. Analysis-domain telemetry, not a report DTO: the
/// convergence holds the non-`Serialize` [`BetaCandidate`](super::beta_candidate::BetaCandidate)
/// boundary, so this is not `Serialize` either.
#[derive(Debug, Clone)]
pub(crate) struct BlockSearchOutcome {
    /// The authoritative per-block fit outcome (identifiable / pinned / flat-objective / no-scale).
    fit: BlockFit,
    /// The complete convergence record of the search that produced [`Self::fit`].
    convergence: BlockConvergence,
}

impl BlockSearchOutcome {
    /// Bind a block's fit and its convergence record, asserting exact agreement between them:
    ///
    /// - A [`Selected`](BasinSelection::Selected) convergence must accompany a fit that retains a
    ///   candidate ([`Identifiable`](BlockFit::Identifiable) / [`PinnedAtBound`](BlockFit::PinnedAtBound)
    ///   / [`FlatObjective`](BlockFit::FlatObjective)), and that candidate must equal the *selected
    ///   basin's* candidate componentwise — the frozen least-RSS-ties-to-smaller-β rule propagates the
    ///   winning basin's candidate verbatim into the fit, so the two are the identical `(β, a, b, RSS)`.
    /// - A [`NoFeasiblePositiveScale`](BasinSelection::NoFeasiblePositiveScale) convergence must
    ///   accompany a [`NonPositiveScale`](BlockFit::NonPositiveScale) fit (no located basin ⇔ no
    ///   candidate).
    ///
    /// `pub(super)` so only the `beta` module's search can mint one; the asserts make a mismatched
    /// fit/convergence pairing unrepresentable rather than merely discouraged.
    pub(super) fn mint(fit: BlockFit, convergence: BlockConvergence) -> Self {
        match convergence.selection() {
            BasinSelection::Selected { basin_index } => {
                let selected = convergence.refined_basins()[basin_index].candidate();
                let fit_candidate = fit
                    .candidate()
                    .expect("a Selected convergence must accompany a candidate-bearing fit");
                assert!(
                    fit_candidate.beta() == selected.beta()
                        && fit_candidate.a() == selected.a()
                        && fit_candidate.b() == selected.b()
                        && fit_candidate.rss() == selected.rss(),
                    "the authoritative fit candidate (β={}, a={}, b={}, rss={}) is not the selected \
                     basin's candidate (β={}, a={}, b={}, rss={})",
                    fit_candidate.beta(),
                    fit_candidate.a(),
                    fit_candidate.b(),
                    fit_candidate.rss(),
                    selected.beta(),
                    selected.a(),
                    selected.b(),
                    selected.rss()
                );
            }
            BasinSelection::NoFeasiblePositiveScale => assert!(
                matches!(fit, BlockFit::NonPositiveScale),
                "a NoFeasiblePositiveScale convergence must accompany a NonPositiveScale fit"
            ),
        }
        Self { fit, convergence }
    }

    /// The authoritative per-block fit outcome.
    pub(crate) fn fit(&self) -> BlockFit {
        self.fit
    }

    /// The convergence record of the search that produced the fit.
    pub(crate) fn convergence(&self) -> &BlockConvergence {
        &self.convergence
    }
}
