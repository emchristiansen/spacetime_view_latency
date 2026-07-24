//! The report projection of one block's complete search-convergence record.

use serde::Serialize;

use crate::analysis::beta::block_convergence::BlockConvergence;
use crate::analysis::report::basin_refinement_report::BasinRefinementReport;
use crate::analysis::report::basin_selection_report::BasinSelectionReport;
use crate::analysis::report::grid_outcome_report::GridOutcomeReport;

/// The report projection of one block's [`BlockConvergence`] (spec: "Do not report only the winning basin,
/// because convergence status covers the complete global search"): the 79-node grid outcome, the refined
/// telemetry of *every* located basin, and which basin the global selection chose. The refined-basin list
/// is data-dependent (a block locates a variable number of basins), so it is a `Vec`.
#[derive(Debug, Serialize)]
pub(crate) struct BlockConvergenceReport {
    /// The coarse 79-node grid scan outcome that seeded the refinement.
    grid: GridOutcomeReport,
    /// The refined outcome of every located basin, positionally aligned with the grid's basin indices.
    refined_basins: Vec<BasinRefinementReport>,
    /// Which located basin the global selection chose, or the no-feasible outcome.
    selection: BasinSelectionReport,
}

impl BlockConvergenceReport {
    /// Project one block's complete convergence record.
    pub(crate) fn of(convergence: &BlockConvergence) -> Self {
        // The refined-basin list is positionally aligned with the grid's located-basin indices and is a
        // data-dependent count, so it stays a `Vec`.
        Self {
            grid: GridOutcomeReport::of(convergence.grid()),
            refined_basins: convergence
                .refined_basins()
                .iter()
                .map(BasinRefinementReport::of)
                .collect(),
            selection: BasinSelectionReport::of(convergence.selection()),
        }
    }
}
