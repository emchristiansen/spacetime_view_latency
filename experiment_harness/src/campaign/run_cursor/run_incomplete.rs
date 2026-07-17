//! The terminal carrier for a run that stopped short: the recovered sink plus its frontier.

use crate::campaign::run_frontier::RunFrontier;
use crate::observation::observation_sink::ObservationSink;

/// What a run that did not complete hands back to its block: the [`ObservationSink`] recovered by move
/// (so the campaign still regains its single output stream to finalize) and the [`RunFrontier`] recording
/// exactly where and how the run stopped. [`Self::into_parts`] is the sole accessor, so the block takes
/// both together.
pub(crate) struct RunIncomplete {
    sink: ObservationSink,
    frontier: RunFrontier,
}

impl RunIncomplete {
    /// Carry a stopped run's recovered sink and its frontier. `pub(in crate::campaign)` so only a run
    /// cursor's failing transition builds one.
    pub(in crate::campaign) fn new(sink: ObservationSink, frontier: RunFrontier) -> Self {
        Self { sink, frontier }
    }

    /// Take the recovered sink and the run's frontier together.
    pub(in crate::campaign) fn into_parts(self) -> (ObservationSink, RunFrontier) {
        (self.sink, self.frontier)
    }
}
