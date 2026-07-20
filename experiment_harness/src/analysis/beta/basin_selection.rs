//! Which refined basin the global search selected, or an explicit no-feasible outcome.

/// The global outcome of the frozen search's basin selection (spec: "the selected basin when a feasible
/// candidate exists; otherwise retain an explicit no-feasible-positive-scale outcome"). Selection does
/// not change the candidate ordering — this only *records* which refined basin the existing
/// least-RSS-ties-to-smaller-β rule chose, so a report can name the winner without re-deriving it.
/// Analysis-domain telemetry, not a report DTO: not `Serialize`.
///
/// [`Selected`](Self::Selected) carries a direct *vector index* into the block's
/// [`refined_basins`](super::block_convergence::BlockConvergence::refined_basins), not a grid-node index,
/// so the winning candidate is reached by direct indexing rather than a search over grid nodes — that
/// directness is what lets the aggregate
/// [`BlockSearchOutcome`](super::block_search_outcome::BlockSearchOutcome) assert the selected basin's
/// candidate equals the authoritative [`BlockFit`](super::block_fit::BlockFit) candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BasinSelection {
    /// The globally least-RSS refined candidate is the one produced by the basin at this 0-based vector
    /// index into [`refined_basins`](super::block_convergence::BlockConvergence::refined_basins).
    Selected {
        /// The 0-based vector index of the winning basin within the block's `refined_basins`.
        basin_index: usize,
    },
    /// No grid node admitted a constrained fit, so the grid located no basin and the block is
    /// [`NonPositiveScale`](super::block_fit::BlockFit::NonPositiveScale) — there is no selected basin.
    NoFeasiblePositiveScale,
}
