//! Unforgeable, affine evidence that a whole campaign ran every block and finalized cleanly.

use crate::campaign::latest_progress::LatestProgress;

/// Evidence that a campaign executed every scheduled block to completion *and* finalized its single
/// required output stream cleanly: the schedule seed that fixed the block order and the typed
/// [`LatestProgress`] high-water mark from the last completed block.
///
/// The constructor is `pub(super)`: only the enclosing campaign cursor module mints one, and it does so
/// only in [`CampaignExhausted::finalize`](super::CampaignExhausted) after the sink's `finalize` returns
/// success — never from a count, a collection, or an unfinalized sink. **Affine, not `Clone`:** a
/// campaign completes exactly once, so its completion cannot be duplicated or replayed.
///
/// Holding it is evidence the campaign's record writes' contracts returned success and the final
/// file/directory sync returned success; it is not a claim of physical crash persistence.
#[derive(Debug)]
pub(crate) struct CampaignComplete {
    seed: u64,
    progress: LatestProgress,
}

impl CampaignComplete {
    /// Mint campaign-completion evidence. `pub(super)` so only the campaign cursor module (via the
    /// finalize transition, and only on a clean finalize) constructs one.
    pub(super) fn new(seed: u64, progress: LatestProgress) -> Self {
        Self { seed, progress }
    }

    /// The schedule seed that fixed the block order this campaign executed.
    pub(crate) fn seed(&self) -> u64 {
        self.seed
    }

    /// The typed high-water progress of the completed campaign: its last completed block, run, record,
    /// and dose whose writer contracts returned success.
    pub(crate) fn progress(&self) -> &LatestProgress {
        &self.progress
    }
}
