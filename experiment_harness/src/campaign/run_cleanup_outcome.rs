//! The outcome of a run's obligatory post-execution cleanup: clean, or a typed cleanup failure.

use super::run_cleanup_failure::RunCleanupFailure;

/// What happened when a run's linear cleanup owner attempted its ordered disconnect-then-teardown after
/// the run left execution. Cleanup is *always* attempted after every run exit — dose-ladder exhaustion or
/// an earlier manifest/dose-write failure alike — so every settled run records this outcome.
///
/// This is the run-level analogue of [`FinalizationOutcome`](super::finalization_outcome::FinalizationOutcome):
/// [`Self::Clean`] mirrors `Sealed` (the cleanup calls returned success) and [`Self::Failed`] mirrors
/// `Failed`, retaining the typed [`RunCleanupFailure`]. A `Clean` cleanup does not by itself make a run
/// complete — a run that stopped short still carries its execution frontier; this only says the disconnect
/// and teardown attempts returned success. These are process-level socket/process/directory contracts
/// returning success, not a claim of physical crash persistence.
#[derive(Debug)]
pub(crate) enum RunCleanupOutcome {
    /// The client disconnect and the server/staged teardown both returned success.
    Clean,
    /// Some part of the ordered cleanup failed; the typed, ordered failure is retained.
    Failed(RunCleanupFailure),
}
