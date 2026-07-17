//! The campaign state after every scheduled block has run: the mandatory finalize before completion.

use crate::campaign::campaign_incomplete::CampaignIncomplete;
use crate::campaign::campaign_outcome::CampaignOutcome;
use crate::campaign::latest_progress::LatestProgress;
use crate::observation::observation_sink::ObservationSink;

use super::CampaignComplete;

/// A campaign whose block iterator is drained and no block is outstanding: every scheduled block ran to
/// completion. It owns the recovered sink, the schedule seed, and the typed latest progress accumulated
/// from every completed block. Execution being complete is *not* completion: the single required output
/// stream must still be finalized, and only a clean finalize mints [`CampaignComplete`].
///
/// [`Self::finalize`] is the sole transition: it consumes the sink's `finalize`, minting affine
/// [`CampaignComplete`] on a clean sync and otherwise a [`CampaignIncomplete::Finalization`] that retains
/// the accumulated [`LatestProgress`] and the sink's typed [`FinalizeError`](crate::observation::finalize_error::FinalizeError).
/// Because completion is gated on this finalize, a campaign cannot report success until its output
/// stream's final file/directory sync calls have returned success — a process-level contract, not a
/// claim of physical crash persistence.
pub(crate) struct CampaignExhausted {
    sink: ObservationSink,
    seed: u64,
    progress: LatestProgress,
}

impl CampaignExhausted {
    /// Carry a fully-executed campaign's recovered sink, seed, and accumulated progress into the finalize
    /// state. `pub(super)` so only [`CampaignReady::next_block`](super::CampaignReady), at the block
    /// iterator's exhaustion edge, mints one — no other caller can assemble this state from arbitrary
    /// sink/seed/progress parts.
    pub(super) fn new(sink: ObservationSink, seed: u64, progress: LatestProgress) -> Self {
        Self {
            sink,
            seed,
            progress,
        }
    }

    /// Finalize the campaign's owned sink — the mandatory step gating completion. On a clean finalize,
    /// mint affine [`CampaignComplete`] from the seed and accumulated progress. On a finalize failure,
    /// return a finalization-only [`CampaignIncomplete::Finalization`] retaining the typed progress and
    /// the sink's [`FinalizeError`](crate::observation::finalize_error::FinalizeError) — execution
    /// succeeded, only the final sync did not — so no failure is discarded and completion is never
    /// claimed on an unfinalized sink.
    pub(in crate::campaign) fn finalize(self) -> CampaignOutcome {
        match self.sink.finalize() {
            Ok(()) => CampaignOutcome::Complete(CampaignComplete::new(self.seed, self.progress)),
            Err(error) => CampaignOutcome::Incomplete(CampaignIncomplete::Finalization {
                progress: self.progress,
                error,
            }),
        }
    }
}
