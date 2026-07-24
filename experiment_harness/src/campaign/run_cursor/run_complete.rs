//! Unforgeable, affine evidence that one run completed its full lifecycle.

use crate::campaign::run_cleanup::CleanupMinted;
use crate::dataset::dose_index::DoseIndex;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::observation::record_id::RecordId;

/// Evidence that one run ran its manifest write, all ten dose observations, its disconnect, and its
/// teardown to completion: the run coordinate, the last record whose writer contract returned success,
/// and the last dose whose observation write's contract returned success. Both markers are required
/// (not `Option`) — a completed run has, by construction, a manifest record and all ten dose records.
///
/// The constructor takes an unforgeable [`CleanupMinted`] witness, so only the run's linear cleanup owner
/// (in [`run_cleanup`](crate::campaign::run_cleanup)) — the sole code that can construct that witness —
/// mints one, and only after the ordered client disconnect and server teardown have returned success
/// (a `Clean` [`RunCleanupOutcome`](crate::campaign::run_cleanup_outcome::RunCleanupOutcome)); never from a
/// count or collection. **Affine, not `Clone`:** a completion is consumed exactly once (by its block, to
/// record progress and mint the block's own completion); duplicating it would let one finished run be
/// counted twice.
///
/// Holding it is evidence the run's record writes' contracts returned success; it is not a claim of
/// physical crash persistence.
#[derive(Debug)]
pub(crate) struct RunComplete {
    run: RunCoordinate,
    last_successful_record: RecordId,
    last_durable_dose: DoseIndex,
}

impl RunComplete {
    /// Mint run-completion evidence. Takes a [`CleanupMinted`] witness, so only the run's linear cleanup
    /// owner constructs one (after a clean cleanup). `pub(in crate::campaign)` so that owner — which lives
    /// in the sibling `run_cleanup` module, not this one — can call it, while the witness keeps it
    /// unmintable by any other campaign code.
    pub(in crate::campaign) fn new(
        run: RunCoordinate,
        last_successful_record: RecordId,
        last_durable_dose: DoseIndex,
        _mint: CleanupMinted,
    ) -> Self {
        Self {
            run,
            last_successful_record,
            last_durable_dose,
        }
    }

    /// The run that completed.
    pub(crate) fn run(&self) -> &RunCoordinate {
        &self.run
    }

    /// The last record whose writer contract returned success in this run.
    pub(crate) fn last_successful_record(&self) -> RecordId {
        self.last_successful_record
    }

    /// The last dose whose observation write's contract returned success in this run.
    pub(crate) fn last_durable_dose(&self) -> DoseIndex {
        self.last_durable_dose
    }
}
