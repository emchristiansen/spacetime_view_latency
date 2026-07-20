//! The complete search-convergence record of one arm block's exponent fit.

use crate::analysis::beta::basin_refinement::BasinRefinement;
use crate::analysis::beta::basin_selection::BasinSelection;
use crate::analysis::beta::grid_outcome::GridOutcome;

/// The complete convergence record of the frozen search over one arm block's ten-dose ladder (spec:
/// "retain a typed per-block search summary containing the 79-node grid outcome, termination telemetry
/// for every refined local basin, and the selected basin ... Do not report only the winning basin,
/// because convergence status covers the complete global search"). This is the secondary estimator's
/// convergence evidence; the primary Theil–Sen estimator is exact, fixed-30-block, non-iterative, and
/// no-early-stop, and so carries no analogous record.
///
/// It records termination *without changing candidate ordering or selection*: the
/// [`BlockFit`](super::block_fit::BlockFit) outcome and its retained candidate remain the authoritative
/// fit result, and [`Self::selection`] only names which basin the unchanged least-RSS-ties-to-smaller-β
/// rule chose. Analysis-domain telemetry, not a report DTO: not `Serialize`.
///
/// Fields are private with no defaults; [`Self::new`] is `pub(super)`, so a `BlockConvergence` is
/// assembled only from within the `beta` module — in production only by the
/// [`fit_block`](super::fit_block) search that owns the telemetry.
#[derive(Debug, Clone)]
pub(crate) struct BlockConvergence {
    /// The coarse grid scan outcome and the basins it located — the deterministic seed of the refinement.
    grid: GridOutcome,
    /// The refined outcome of *every* located basin, not only the winner — the complete global-search
    /// record, positionally aligned with [`GridOutcome::basin_node_indices`](GridOutcome::basin_node_indices).
    refined_basins: Vec<BasinRefinement>,
    /// Which located basin the global selection chose, or the explicit no-feasible-positive-scale
    /// outcome.
    selection: BasinSelection,
}

impl BlockConvergence {
    /// Bind one block's complete convergence record, asserting it is consistent with the frozen search:
    ///
    /// - Exactly one [`BasinRefinement`] per located basin, in matching order — every located basin's
    ///   grid node is feasible, and [`golden_section`](super::fit_block) always refines it to a feasible
    ///   candidate, so the refinements are positionally aligned with the grid's basin indices, and each
    ///   refinement's [`grid_node_index`](super::basin_termination::BasinTermination::grid_node_index)
    ///   equals the basin index it is paired with (so a refinement cannot be paired to the wrong basin).
    /// - The selection is [`NoFeasiblePositiveScale`](BasinSelection::NoFeasiblePositiveScale) exactly
    ///   when no basin was located (`best` stayed `None`), and otherwise names a
    ///   [`Selected`](BasinSelection::Selected) vector index that is in range for `refined_basins`.
    ///
    /// The stronger cross-check that the selected basin's candidate *equals* the authoritative
    /// [`BlockFit`](super::block_fit::BlockFit) candidate is enforced one level up, where fit and
    /// convergence are bound together by the aggregate
    /// [`BlockSearchOutcome`](super::block_search_outcome::BlockSearchOutcome) — a convergence record
    /// alone cannot see the fit it belongs to.
    ///
    /// `pub(super)` so only the `beta` module's search can mint one; the asserts make an
    /// unreachable-in-the-search record a loud panic rather than representable evidence.
    pub(super) fn new(
        grid: GridOutcome,
        refined_basins: Vec<BasinRefinement>,
        selection: BasinSelection,
    ) -> Self {
        let basin_indices = grid.basin_node_indices();
        assert!(
            refined_basins.len() == basin_indices.len(),
            "expected one refinement per located basin ({} basins), got {}",
            basin_indices.len(),
            refined_basins.len()
        );
        for (refinement, &basin_index) in refined_basins.iter().zip(basin_indices.iter()) {
            assert!(
                refinement.termination().grid_node_index() == basin_index,
                "refinement grid node {} is paired with located basin {}",
                refinement.termination().grid_node_index(),
                basin_index
            );
        }
        match selection {
            BasinSelection::NoFeasiblePositiveScale => assert!(
                refined_basins.is_empty(),
                "NoFeasiblePositiveScale requires no located basin, got {}",
                refined_basins.len()
            ),
            BasinSelection::Selected { basin_index } => assert!(
                basin_index < refined_basins.len(),
                "selected basin vector index {basin_index} is out of range for {} refined basins",
                refined_basins.len()
            ),
        }
        Self {
            grid,
            refined_basins,
            selection,
        }
    }

    /// The coarse grid scan outcome.
    pub(crate) fn grid(&self) -> &GridOutcome {
        &self.grid
    }

    /// The refined outcome of every located basin, in matching order with the grid's basin indices.
    pub(crate) fn refined_basins(&self) -> &[BasinRefinement] {
        &self.refined_basins
    }

    /// Which basin the global selection chose, or the no-feasible outcome.
    pub(crate) fn selection(&self) -> BasinSelection {
        self.selection
    }
}
