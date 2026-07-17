//! The terminal carrier for a completed run: the recovered sink plus the run's completion evidence.

use crate::campaign::run_cleanup::CleanupMinted;
use crate::observation::observation_sink::ObservationSink;

use super::RunComplete;

/// What a completed run hands back to its block: the [`ObservationSink`] recovered by move (so the
/// campaign always regains its single output stream for exactly one finalize) and the run's affine
/// [`RunComplete`]. [`Self::into_parts`] is the sole accessor, so the block takes both together and
/// neither the sink nor the completion can be duplicated out of it.
pub(crate) struct RunDone {
    sink: ObservationSink,
    complete: RunComplete,
}

impl RunDone {
    /// Carry a completed run's recovered sink and completion evidence. Takes a [`CleanupMinted`] witness,
    /// so only the run's linear cleanup owner mints one after a clean cleanup; `pub(in crate::campaign)`
    /// so that sibling-module owner can call it while the witness keeps it unmintable elsewhere.
    pub(in crate::campaign) fn new(
        sink: ObservationSink,
        complete: RunComplete,
        _mint: CleanupMinted,
    ) -> Self {
        Self { sink, complete }
    }

    /// Take the recovered sink and the run's completion evidence together.
    pub(in crate::campaign) fn into_parts(self) -> (ObservationSink, RunComplete) {
        (self.sink, self.complete)
    }
}
