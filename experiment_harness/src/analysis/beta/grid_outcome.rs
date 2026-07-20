//! The coarse 79-node grid scan outcome that seeds the golden-section refinement.

use crate::analysis::beta::fit_block::GRID_NODES;
use crate::analysis::beta::grid_node_outcome::GridNodeOutcome;

/// The outcome of the frozen coarse grid scan over `[0.1, 4.0]` (spec: "the 79-node grid outcome"):
/// every node's typed constrained-fit outcome and the 0-based indices of the grid-local basins the scan
/// located. The grid scan is the deterministic seed of the golden-section refinement, so retaining it
/// lets the report show *why* each basin was refined. The node count is sized from the single frozen
/// [`GRID_NODES`] source in [`fit_block`](super::fit_block), never a re-typed literal, so the search and
/// its telemetry cannot drift. Analysis-domain telemetry, not a report DTO: not `Serialize`.
///
/// Fields are private with no defaults; [`Self::new`] is `pub(super)`, so a `GridOutcome` is assembled
/// only from within the `beta` module — in production only by the [`fit_block`](super::fit_block) scan.
#[derive(Debug, Clone)]
pub(crate) struct GridOutcome {
    /// Each grid node's typed constrained-fit outcome (finite RSS or no-positive-scale) — no `+∞`/NaN
    /// sentinel. Heap-owned as a boxed fixed array so the node cardinality is a property of the type.
    nodes: Box<[GridNodeOutcome; GRID_NODES]>,
    /// The 0-based indices of the grid-local RSS basins the scan located (each an
    /// [`is_local_basin`](super::fit_block) node), strictly ascending — the exact set of nodes the
    /// refinement started from.
    basin_node_indices: Vec<usize>,
}

impl GridOutcome {
    /// Bind the coarse-grid scan outcome, asserting the located-basin index set is well-formed at the
    /// mint boundary: every index is in range, the indices are strictly ascending (hence unique), and
    /// each points to a [`Feasible`](GridNodeOutcome::Feasible) node — a basin is a local RSS minimum, so
    /// it necessarily admitted a constrained fit. `pub(super)` so only the `beta` module's search can
    /// mint one — the caller owns evaluating the frozen grid and identifying its basins; these asserts
    /// make a malformed basin set a loud panic rather than representable telemetry.
    pub(super) fn new(
        nodes: Box<[GridNodeOutcome; GRID_NODES]>,
        basin_node_indices: Vec<usize>,
    ) -> Self {
        for window in basin_node_indices.windows(2) {
            assert!(
                window[0] < window[1],
                "located basin indices must be strictly ascending (hence unique), got {} then {}",
                window[0],
                window[1]
            );
        }
        for &index in &basin_node_indices {
            assert!(
                index < GRID_NODES,
                "located basin index {index} is outside the 0..{GRID_NODES} grid"
            );
            assert!(
                nodes[index].is_feasible(),
                "located basin index {index} points to a node with no constrained fit"
            );
        }
        Self {
            nodes,
            basin_node_indices,
        }
    }

    /// Every grid node's typed constrained-fit outcome, in a fixed [`GRID_NODES`]-length array — the
    /// complete coarse-grid scan the report projects across the lossy boundary. A narrow read accessor
    /// over the boxed array (the report renders every node's outcome, not only the located basins), so the
    /// 79-node cardinality stays a property of the type.
    pub(crate) fn nodes(&self) -> &[GridNodeOutcome; GRID_NODES] {
        &self.nodes
    }

    /// The 0-based indices of the located grid-local basins, strictly ascending.
    pub(crate) fn basin_node_indices(&self) -> &[usize] {
        &self.basin_node_indices
    }
}
