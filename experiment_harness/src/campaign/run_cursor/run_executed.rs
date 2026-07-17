//! The run state after the dose ladder is drained, before its obligatory cleanup: execution exhausted.

use crate::dataset::dose_index::DoseIndex;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::observation::observation_sink::ObservationSink;
use crate::observation::record_id::RecordId;

/// A run whose dose ladder is fully drained, positioned *before* its obligatory cleanup. Reachable only
/// from [`RunDosing::next_dose`](super::RunDosing) exhaustion, so every dose write's contract returned
/// success by the time this state exists. The manifest receipt has been dropped — no further observations
/// will be written — leaving the recovered sink and the last-success markers to carry into the terminal.
///
/// This is an **inert carrier**: it exposes no transition of its own. It can only be consumed by the run's
/// linear cleanup owner ([`RunCleanup::settle_executed`](crate::campaign::run_cleanup::RunCleanup)), which
/// performs the real ordered disconnect-then-teardown and is the *sole* minter of the run terminals. That
/// is why the accompanying [`RunCleanupOutcome`](crate::campaign::run_cleanup_outcome::RunCleanupOutcome)
/// is never an argument here — an exhausted run cannot settle itself with a caller-supplied "clean"
/// outcome; only executed cleanup can.
///
/// Both last-success markers are required — `last_record: RecordId` (a manifest write's contract returned
/// success) and `last_dose: DoseIndex` (all ten dose writes' contracts returned success) — because this
/// state is past the ladder-exhaustion edge where the accumulating `last_dose` `Option` is converted once
/// with a loud invariant check. `last_dose: None` is unrepresentable from here.
pub(crate) struct RunExecuted {
    sink: ObservationSink,
    coord: RunCoordinate,
    last_record: RecordId,
    last_dose: DoseIndex,
}

impl RunExecuted {
    /// Position a run at its exhausted-execution state after ladder exhaustion. `pub(in
    /// crate::campaign::run_cursor)` so only the dose cursor's exhaustion transition builds one.
    pub(in crate::campaign::run_cursor) fn new(
        sink: ObservationSink,
        coord: RunCoordinate,
        last_record: RecordId,
        last_dose: DoseIndex,
    ) -> Self {
        Self {
            sink,
            coord,
            last_record,
            last_dose,
        }
    }

    /// The coordinate of the exhausted run, borrowed so the linear cleanup owner can assert its own bound
    /// coordinate matches this run's before settling it.
    pub(in crate::campaign) fn coord(&self) -> &RunCoordinate {
        &self.coord
    }

    /// Surrender the exhausted run's recovered sink and last-success markers to its cleanup owner. Yields
    /// only inert data — it cannot mint a terminal, which requires the cleanup owner's unforgeable
    /// `CleanupMinted` witness — so exposing it to the campaign module is harmless. `pub(in crate::campaign)`
    /// because the consuming `run_cleanup` module is not a `run_cursor` descendant.
    pub(in crate::campaign) fn into_parts(
        self,
    ) -> (ObservationSink, RunCoordinate, RecordId, DoseIndex) {
        (self.sink, self.coord, self.last_record, self.last_dose)
    }
}
