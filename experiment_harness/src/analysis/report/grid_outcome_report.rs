//! The report projection of one block's coarse 79-node grid scan outcome.

use serde::Serialize;

use crate::analysis::beta::grid_outcome::GridOutcome;
use crate::analysis::report::grid_nodes_report::GridNodesReport;

/// The report projection of one block's [`GridOutcome`] (spec: "the 79-node grid outcome"): every node's
/// typed constrained-fit outcome and the 0-based indices of the located grid-local basins. The node array
/// is a fixed-cardinality [`GridNodesReport`]; the basin index list is genuinely data-dependent (a block
/// locates a variable number of basins), so it remains a `Vec`.
#[derive(Debug, Serialize)]
pub(crate) struct GridOutcomeReport {
    /// Every grid node's typed outcome, in a fixed 79-node array.
    nodes: GridNodesReport,
    /// The 0-based indices of the located grid-local basins, strictly ascending — a data-dependent count.
    basin_node_indices: Vec<usize>,
}

impl GridOutcomeReport {
    /// Project one block's coarse-grid scan outcome.
    pub(crate) fn of(grid: &GridOutcome) -> Self {
        let _ = grid;
        todo!("Phase 2: project each node outcome into the fixed node array and the located basin indices")
    }
}
