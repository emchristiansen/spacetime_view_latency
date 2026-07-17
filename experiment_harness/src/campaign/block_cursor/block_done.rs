//! The terminal carrier for a completed block: the recovered sink plus the block's completion evidence.

use crate::observation::observation_sink::ObservationSink;

use super::BlockComplete;

/// What a completed block hands back to its campaign: the [`ObservationSink`] recovered by move (so the
/// campaign always regains its single output stream for exactly one finalize) and the block's affine
/// [`BlockComplete`]. [`Self::into_parts`] is the sole accessor, so the campaign takes both together and
/// neither the sink nor the completion can be duplicated out of it.
pub(crate) struct BlockDone {
    sink: ObservationSink,
    complete: BlockComplete,
}

impl BlockDone {
    /// Carry a completed block's recovered sink and completion evidence. `pub(in crate::campaign)` so
    /// only the block-exhaustion transition builds one.
    pub(in crate::campaign) fn new(sink: ObservationSink, complete: BlockComplete) -> Self {
        Self { sink, complete }
    }

    /// Take the recovered sink and the block's completion evidence together.
    pub(in crate::campaign) fn into_parts(self) -> (ObservationSink, BlockComplete) {
        (self.sink, self.complete)
    }
}
