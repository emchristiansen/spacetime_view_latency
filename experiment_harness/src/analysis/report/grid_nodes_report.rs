//! The fixed-cardinality 79-node grid array with a manual sequence `Serialize`.

use serde::ser::{Serialize, Serializer};

use crate::analysis::beta::fit_block::GRID_NODES;
use crate::analysis::report::grid_node_report::GridNodeReport;

/// The complete coarse-grid scan projection: every one of the [`GRID_NODES`] nodes' outcomes, held in a
/// boxed fixed array so the 79-node cardinality is a property of the type rather than a runtime length.
/// The node count is sized from the single frozen [`GRID_NODES`] source in
/// [`fit_block`](crate::analysis::beta::fit_block), never a re-typed literal.
///
/// `serde` provides no blanket `Serialize` for arrays longer than 32, so this newtype implements
/// [`Serialize`] by hand, emitting the array as a JSON sequence of its elements — preserving the fixed
/// cardinality and valid-by-construction node states without a `Vec` and without a new dependency.
#[derive(Debug)]
pub(crate) struct GridNodesReport(Box<[GridNodeReport; GRID_NODES]>);

impl GridNodesReport {
    /// Bind the complete fixed-length node array. One input; the boxed array's length is the type's
    /// guarantee of the frozen 79-node cardinality.
    pub(crate) fn new(nodes: Box<[GridNodeReport; GRID_NODES]>) -> Self {
        Self(nodes)
    }
}

impl Serialize for GridNodesReport {
    /// Serialize the fixed 79-node array as a plain sequence — the JSON shape is a bare array of node
    /// outcomes, identical to what a supported-length array would emit.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.0.iter())
    }
}
