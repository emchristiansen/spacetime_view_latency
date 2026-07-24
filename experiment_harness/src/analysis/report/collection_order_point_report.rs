//! One `(collection-order key, block total)` point of a cell's temporal plot.

use serde::Serialize;

use crate::analysis::finite_f64::FiniteF64;
use crate::observation::record_seq::RecordSeq;

/// One point of the collection-order temporal plot: a block's schedule-proven collection-order key and its
/// primary `T_block` total change in milliseconds. The x-value is the exact [`RecordSeq`] key (preserving
/// global-schedule gaps rather than a rank), and the y-value crosses the lossy boundary to a finite
/// [`FiniteF64`] millisecond value.
#[derive(Debug, Serialize)]
pub(crate) struct CollectionOrderPointReport {
    /// The block's schedule-proven collection-order key — the plot x-coordinate, preserving schedule gaps.
    collection_order_key: RecordSeq,
    /// The block's primary `T_block` total change, in milliseconds — the plot y-coordinate.
    block_total_millis: FiniteF64,
}

impl CollectionOrderPointReport {
    /// Bind one plot point from its already-projected key and finite millisecond total. A trivial leaf
    /// constructor (both parts arrive already projected from the one enclosing plot projection), so it is
    /// not a `todo!()` — it introduces no evidence of its own.
    pub(crate) fn new(collection_order_key: RecordSeq, block_total_millis: FiniteF64) -> Self {
        Self {
            collection_order_key,
            block_total_millis,
        }
    }

    /// The block's schedule-proven collection-order key — the plot x-coordinate. Read by the sibling SVG
    /// renderer so the rendered polyline plots against the same actual keys this point serializes.
    pub(crate) fn collection_order_key(&self) -> RecordSeq {
        self.collection_order_key
    }

    /// The block's primary `T_block` total change in milliseconds — the plot y-coordinate. Read by the
    /// sibling SVG renderer so the rendered polyline and the serialized point share one projected value.
    pub(crate) fn block_total_millis(&self) -> FiniteF64 {
        self.block_total_millis
    }
}
