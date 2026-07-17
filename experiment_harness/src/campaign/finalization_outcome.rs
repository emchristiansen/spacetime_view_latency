//! The outcome of the always-attempted sink finalization that follows an execution failure.

use crate::observation::finalize_error::FinalizeError;

/// What happened when the campaign finalized its owned sink after execution stopped short. Finalization
/// is *always* attempted after an execution failure — the sink's final file and directory sync attempts
/// run to seal whatever records the writer contract had already returned success for — so an
/// execution-incomplete campaign records both the execution frontier *and* this finalization outcome,
/// never discarding either.
///
/// A `Sealed` finalization does not make the campaign complete: the execution frontier still records
/// that the run set did not finish. This only says the final file/directory sync attempts returned
/// success (`Sealed`) or that a sync attempt itself failed (`Failed`), retaining the sink's typed
/// [`FinalizeError`]. These are process-level file/directory contracts returning success; neither they
/// nor the per-record writes they seal imply physical crash persistence.
#[derive(Debug)]
pub(crate) enum FinalizationOutcome {
    /// The sink's finalize synced cleanly, sealing the records already written.
    Sealed,
    /// The sink's finalize itself failed, on top of the execution failure; the typed error is retained.
    Failed(FinalizeError),
}
