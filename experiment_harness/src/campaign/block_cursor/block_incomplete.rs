//! The terminal carrier for a block that stopped short: the recovered sink plus its frontier.

use crate::campaign::block_frontier::BlockFrontier;
use crate::observation::observation_sink::ObservationSink;

/// What a block that did not complete hands back to its campaign: the [`ObservationSink`] recovered by
/// move (so the campaign still regains its single output stream to finalize) and the [`BlockFrontier`]
/// recording exactly where and how the block stopped. [`Self::into_parts`] is the sole accessor, so the
/// campaign takes both together.
pub(crate) struct BlockIncomplete {
    sink: ObservationSink,
    frontier: BlockFrontier,
}

impl BlockIncomplete {
    /// Carry a stopped block's recovered sink and its frontier. `pub(in crate::campaign)` so only a
    /// block cursor's aborting transition builds one.
    pub(in crate::campaign) fn new(sink: ObservationSink, frontier: BlockFrontier) -> Self {
        Self { sink, frontier }
    }

    /// Take the recovered sink and the block's frontier together.
    pub(in crate::campaign) fn into_parts(self) -> (ObservationSink, BlockFrontier) {
        (self.sink, self.frontier)
    }
}
