//! The terminal carrier for a run that stopped short: the recovered sink plus its typed incompletion.

use crate::campaign::run_cleanup::CleanupMinted;
use crate::campaign::run_incompletion::RunIncompletion;
use crate::observation::observation_sink::ObservationSink;

/// What a run that did not complete hands back to its block: the [`ObservationSink`] recovered by move
/// (so the campaign still regains its single output stream to finalize) and the [`RunIncompletion`]
/// recording exactly where and how the run stopped — its execution frontier plus the always-attempted
/// cleanup outcome, or a cleanup-stage frontier plus the cleanup failure. [`Self::into_parts`] is the sole
/// accessor, so the block takes both together.
pub(crate) struct RunIncomplete {
    sink: ObservationSink,
    incompletion: RunIncompletion,
}

impl RunIncomplete {
    /// Carry a stopped run's recovered sink and its typed incompletion. Takes a [`CleanupMinted`] witness,
    /// so — exactly like [`RunDone`](super::RunDone) — only the run's linear cleanup owner mints one, after
    /// the obligatory cleanup has been attempted. `pub(in crate::campaign)` so that sibling-module owner
    /// can call it while the witness keeps it unmintable elsewhere.
    pub(in crate::campaign) fn new(
        sink: ObservationSink,
        incompletion: RunIncompletion,
        _mint: CleanupMinted,
    ) -> Self {
        Self { sink, incompletion }
    }

    /// Take the recovered sink and the run's incompletion together.
    pub(in crate::campaign) fn into_parts(self) -> (ObservationSink, RunIncompletion) {
        (self.sink, self.incompletion)
    }
}
