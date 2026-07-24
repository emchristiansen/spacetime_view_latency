//! The combined outcome of finalizing the durable sink when it did not cleanly complete.

use crate::observation::final_sync_failures::FinalSyncFailures;
use crate::observation::poisoned_tail::PoisonedTail;
use crate::observation::tail_state::TailState;

/// A finalization that did not cleanly complete, retaining *both* aggregated parts rather than
/// discarding either — mirroring the run driver's primary+teardown aggregation (see `execute_run`):
/// the prior persist failure that had poisoned the sink (if any), and the final file+directory sync
/// failures (the independent [`FinalSyncFailures`] slots, if the best-effort sync failed).
/// Finalization is always best-effort: the sink syncs the file and directory even when poisoned, to
/// preserve records and metadata already written, so both a prior failure and fresh sync failures can
/// be present at once. This value only exists on the non-clean path — [`Self::combine`] returns
/// `Ok(())` when there was no prior failure and the final sync succeeded — so at least one part is
/// always present.
///
/// The campaign layer reads the parts to classify incompleteness precisely: a prior pre-write failure
/// with a successful final sync is incomplete because a record is *absent*, yet not
/// durability-ambiguous; a prior ambiguous write stays ambiguous; and a final-sync failure adds
/// finalization ambiguity regardless of the prior part.
#[derive(Debug)]
pub(crate) struct FinalizeError {
    prior: Option<PoisonedTail>,
    sync_failures: Option<FinalSyncFailures>,
}

impl FinalizeError {
    /// Combine the prior poison reason and the final-sync result into a finalization outcome: clean
    /// success (`Ok`) iff there was no prior failure *and* the final sync succeeded, otherwise a typed
    /// error retaining whichever parts are present. Neither error is discarded.
    pub(crate) fn combine(
        prior: Option<PoisonedTail>,
        sync_failures: Option<FinalSyncFailures>,
    ) -> std::result::Result<(), Self> {
        match (prior, sync_failures) {
            (None, None) => Ok(()),
            (prior, sync_failures) => Err(Self {
                prior,
                sync_failures,
            }),
        }
    }

    /// The prior persist failure that had poisoned the sink before finalization, if any.
    pub(crate) fn prior(&self) -> Option<&PoisonedTail> {
        self.prior.as_ref()
    }

    /// The independent file+directory final-sync failures, if the best-effort sync failed.
    pub(crate) fn sync_failures(&self) -> Option<&FinalSyncFailures> {
        self.sync_failures.as_ref()
    }

    /// Whether the sink's durable tail is ambiguous: a prior ambiguous write, or a final-sync
    /// failure, leaves durability unknown. A prior pre-write failure alone leaves the tail clean (the
    /// record is merely absent, not ambiguous).
    pub(crate) fn is_durability_ambiguous(&self) -> bool {
        self.sync_failures.is_some()
            || matches!(
                self.prior.as_ref().map(PoisonedTail::tail_state),
                Some(TailState::DurabilityAmbiguous)
            )
    }
}
