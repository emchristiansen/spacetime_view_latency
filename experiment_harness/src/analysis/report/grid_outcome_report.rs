//! The report projection of one block's coarse 79-node grid scan outcome.

use serde::Serialize;

use crate::analysis::beta::grid_outcome::GridOutcome;
use crate::analysis::report::grid_node_report::GridNodeReport;
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
        // Project every node outcome heap-first into the fixed 79-node array, mirroring the trusted
        // graph's `Vec -> Box<[T]> -> Box<[T; N]>` construction so no large array materializes on the
        // stack. The located-basin index list is genuinely data-dependent, so it stays a `Vec`.
        let nodes: Vec<GridNodeReport> = grid
            .nodes()
            .iter()
            .map(|&node| GridNodeReport::of(node))
            .collect();
        let nodes = nodes
            .into_boxed_slice()
            .try_into()
            .ok()
            .expect("exactly GRID_NODES node outcomes project into exactly GRID_NODES node reports");
        Self {
            nodes: GridNodesReport::new(nodes),
            basin_node_indices: grid.basin_node_indices().to_vec(),
        }
    }
}
