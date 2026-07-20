//! The report projection of one coarse-grid node's constrained-fit outcome.

use serde::Serialize;

use crate::analysis::beta::grid_node_outcome::GridNodeOutcome;
use crate::analysis::finite_f64::FiniteF64;

/// The report projection of one [`GridNodeOutcome`]: a finite, nonnegative residual sum of squares, or the
/// explicit no-positive-scale outcome. The infeasible case is a typed variant, never a `+∞`/NaN sentinel,
/// so a node's serialized "RSS" is always a real residual sum.
#[derive(Debug, Serialize)]
pub(crate) enum GridNodeReport {
    /// A constrained fit exists at this node; its residual sum of squares is finite and nonnegative.
    Feasible {
        /// The finite, nonnegative residual sum of squares of the node's constrained fit.
        rss: FiniteF64,
    },
    /// No constrained fit with strictly positive finite scale exists at this node.
    NoPositiveScale,
}

impl GridNodeReport {
    /// Project one grid node's outcome. Takes the `Copy` [`GridNodeOutcome`] by value — one input.
    pub(crate) fn of(outcome: GridNodeOutcome) -> Self {
        let _ = outcome;
        todo!("Phase 2: project a feasible node's RSS through FiniteF64, else NoPositiveScale")
    }
}
