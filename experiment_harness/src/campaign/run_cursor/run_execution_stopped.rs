//! The run state after a failing execution edge, before its obligatory cleanup: execution stopped short.

use crate::campaign::run_frontier::RunFrontier;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::observation::observation_sink::ObservationSink;

/// A run that stopped short during execution — a manifest or dose write whose contract did not return
/// success — positioned *before* its obligatory cleanup. Carries the recovered sink and the
/// [`RunFrontier`] recording exactly where and how execution stopped. Reachable only from a run cursor's
/// failing edge.
///
/// Like [`RunExecuted`](super::RunExecuted) this is an **inert carrier** with no transition of its own: it
/// can only be consumed by the run's linear cleanup owner
/// ([`RunCleanup::settle_stopped`](crate::campaign::run_cleanup::RunCleanup)), which always attempts the
/// ordered disconnect-then-teardown — cleanup is mandatory even after an early execution failure — and is
/// the sole minter of the terminal [`RunIncomplete`](super::RunIncomplete). The resulting incompletion
/// retains *both* this execution frontier and the cleanup outcome, so a simultaneous execution-and-cleanup
/// failure is preserved as typed structure.
pub(crate) struct RunExecutionStopped {
    sink: ObservationSink,
    frontier: RunFrontier,
}

impl RunExecutionStopped {
    /// Position a run at its stopped-execution state from a failing edge. `pub(in
    /// crate::campaign::run_cursor)` so only a run cursor's failing transition builds one.
    pub(in crate::campaign::run_cursor) fn new(
        sink: ObservationSink,
        frontier: RunFrontier,
    ) -> Self {
        Self { sink, frontier }
    }

    /// The coordinate of the stopped run, read from its frontier, so the cleanup owner can assert its bound
    /// coordinate matches this run's before settling it.
    pub(in crate::campaign) fn coord(&self) -> &RunCoordinate {
        self.frontier.run()
    }

    /// Surrender the stopped run's recovered sink and execution frontier to its cleanup owner. Inert data
    /// only — it cannot mint a terminal without the cleanup owner's `CleanupMinted` witness.
    pub(in crate::campaign) fn into_parts(self) -> (ObservationSink, RunFrontier) {
        (self.sink, self.frontier)
    }
}
