//! The campaign continuation while one of its blocks is outstanding.

use std::vec::IntoIter;

use crate::campaign::block_cursor::BlockDone;
use crate::campaign::block_cursor::BlockIncomplete;
use crate::campaign::campaign_frontier::CampaignFrontier;
use crate::campaign::campaign_incomplete::CampaignIncomplete;
use crate::campaign::campaign_outcome::CampaignOutcome;
use crate::campaign::finalization_outcome::FinalizationOutcome;
use crate::campaign::latest_progress::LatestProgress;
use crate::plan::schedule::BlockCoordinate;
use crate::plan::schedule::BlockRun;

use super::CampaignReady;

/// The campaign's continuation held while exactly one of its blocks is outstanding: the schedule seed,
/// the schedule's own remaining block iterator, the outstanding block's coordinate, and the typed
/// progress accumulated from every earlier completed block. Holding the iterator here (never handing it
/// to a caller) is what lets [`Self::finish`] resume the campaign on its *own* partly-drained iterator.
///
/// Exactly one block is outstanding at a time — a campaign never starts a second block before the first
/// resolves — so the campaign's at-most-one-outstanding-child invariant is structural, not counted.
pub(crate) struct CampaignRunningBlock {
    seed: u64,
    blocks: IntoIter<BlockRun>,
    block_coord: BlockCoordinate,
    progress: LatestProgress,
}

impl CampaignRunningBlock {
    /// Hold a campaign's continuation while its drawn block is outstanding. `pub(super)` so only
    /// [`CampaignReady::next_block`] — not a campaign sibling — supplies the private block iterator.
    pub(super) fn new(
        seed: u64,
        blocks: IntoIter<BlockRun>,
        block_coord: BlockCoordinate,
        progress: LatestProgress,
    ) -> Self {
        Self {
            seed,
            blocks,
            block_coord,
            progress,
        }
    }

    /// Fold a completed block back into the campaign: advance the typed progress from the block's
    /// completion evidence, then resume the campaign on its own preserved iterator. The completed block's
    /// coordinate must match the outstanding block's — exactly as the block cursor binds a run
    /// completion — so a foreign block's completion cannot be lifted into this campaign; a mismatch is a
    /// wiring bug and fails loud. Consuming the affine [`BlockComplete`](crate::campaign::block_cursor::BlockComplete)
    /// (borrowed to record progress, then dropped) means one finished block advances progress exactly
    /// once.
    pub(in crate::campaign) fn finish(self, done: BlockDone) -> CampaignReady {
        let (sink, complete) = done.into_parts();
        assert_eq!(
            complete.block(),
            self.block_coord,
            "the completed block's coordinate must match the outstanding block's coordinate"
        );
        let mut progress = self.progress;
        progress.record_block(&complete);
        CampaignReady::resume(sink, self.seed, self.blocks, progress)
    }

    /// Abort the campaign because its outstanding block stopped short. Lift the block's frontier into a
    /// [`CampaignFrontier`] at this campaign's seed — binding the stopped block's coordinate to the
    /// outstanding block's, exactly as [`Self::finish`] binds a completion, so a foreign block's failure
    /// cannot be lifted in — then **always** finalize the recovered sink in this same terminal
    /// transition, so the required finalize can never be forgotten on the execution-failure path. The
    /// resulting [`CampaignIncomplete::Execution`] retains both the execution frontier and the typed
    /// [`FinalizationOutcome`]; neither error is discarded. The [`CampaignFrontier`] records the seed (so
    /// the exact block order is named) and the failing block's frontier down to its run/stage/record
    /// tail; it does not additionally carry earlier completed blocks' progress, which the execution
    /// frontier is not required to restate.
    pub(in crate::campaign) fn abort(self, incomplete: BlockIncomplete) -> CampaignOutcome {
        let (sink, block_frontier) = incomplete.into_parts();
        assert_eq!(
            block_frontier.block(),
            self.block_coord,
            "the stopped block's coordinate must match the outstanding block's coordinate"
        );
        let frontier = CampaignFrontier::new(self.seed, Some(block_frontier));
        let finalization = match sink.finalize() {
            Ok(()) => FinalizationOutcome::Sealed,
            Err(error) => FinalizationOutcome::Failed(error),
        };
        CampaignOutcome::Incomplete(CampaignIncomplete::Execution {
            frontier,
            finalization,
        })
    }
}
