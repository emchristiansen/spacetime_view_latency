//! The structural record of where and how a campaign's execution stopped short of completion.

use crate::manifest::schedule_seed::ScheduleSeed;

use super::block_frontier::BlockFrontier;

/// Where a campaign's execution stopped: the schedule seed that fixed its block order, and the frontier
/// of the block that stopped, if a block was outstanding when execution failed.
///
/// `block` is `Option` because execution can fail with no block frontier of its own — this milestone
/// only stops *inside* a block (so `block` is `Some`), but the structure admits a future
/// between-blocks stop that carries no block frontier. A campaign holds at most one outstanding block,
/// so at most one [`BlockFrontier`] is possible; there is no collection of block frontiers to
/// reconcile. The seed is retained so an incomplete campaign names the exact block order it was
/// executing.
#[derive(Debug)]
pub(crate) struct CampaignFrontier {
    seed: ScheduleSeed,
    block: Option<BlockFrontier>,
}

impl CampaignFrontier {
    /// Record a campaign frontier from the schedule seed and the outstanding block's frontier, if any.
    /// `pub(in crate::campaign)` so only a campaign cursor's transition builds one.
    pub(in crate::campaign) fn new(seed: ScheduleSeed, block: Option<BlockFrontier>) -> Self {
        Self { seed, block }
    }

    /// The schedule seed that fixed the block order this campaign was executing.
    pub(crate) fn seed(&self) -> ScheduleSeed {
        self.seed
    }

    /// The frontier of the block that was outstanding when execution stopped, if a block was
    /// outstanding.
    pub(crate) fn block(&self) -> Option<&BlockFrontier> {
        self.block.as_ref()
    }
}
