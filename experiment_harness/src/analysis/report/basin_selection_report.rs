//! The report projection of which refined basin the global search selected.

use serde::Serialize;

use crate::analysis::beta::basin_selection::BasinSelection;

/// The report projection of a [`BasinSelection`]: the vector index of the winning refined basin, or the
/// explicit no-feasible-positive-scale outcome. Records which basin the frozen least-RSS-ties-to-smaller-β
/// rule chose without changing candidate ordering or selection.
#[derive(Debug, Serialize)]
pub(crate) enum BasinSelectionReport {
    /// The globally least-RSS refined candidate is the one produced by the basin at this 0-based vector
    /// index into the block's refined basins.
    Selected {
        /// The 0-based vector index of the winning basin within the block's refined basins.
        basin_index: usize,
    },
    /// No grid node admitted a constrained fit, so the grid located no basin and the block is
    /// non-positive-scale — there is no selected basin.
    NoFeasiblePositiveScale,
}

impl BasinSelectionReport {
    /// Project the analysis-domain selection. Takes the `Copy` [`BasinSelection`] by value — one input.
    pub(crate) fn of(selection: BasinSelection) -> Self {
        match selection {
            BasinSelection::Selected { basin_index } => Self::Selected { basin_index },
            BasinSelection::NoFeasiblePositiveScale => Self::NoFeasiblePositiveScale,
        }
    }
}
