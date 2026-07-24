//! The structural record of where and how a repetition block stopped short of completion.

use crate::plan::schedule::BlockCoordinate;

use super::run_incompletion::RunIncompletion;

/// Where a block stopped: the block coordinate, and the typed incompletion of the run that stopped, if a
/// run was outstanding when the block failed.
///
/// `run` is `Option` because a block can fail with no run incompletion of its own to name — this milestone
/// only stops *inside* a run (so `run` is `Some`), but the structure admits a future block-scoped stop
/// (e.g. a between-runs reset) that carries no run incompletion. A block holds at most one outstanding run,
/// so at most one [`RunIncompletion`] is possible; there is no collection to reconcile. Carrying the full
/// [`RunIncompletion`] — not just an execution frontier — preserves the run's dual execution-and-cleanup
/// failure fidelity all the way up, exactly as [`CampaignIncomplete`](super::campaign_incomplete::CampaignIncomplete)
/// keeps both its siblings.
#[derive(Debug)]
pub(crate) struct BlockFrontier {
    block: BlockCoordinate,
    run: Option<RunIncompletion>,
}

impl BlockFrontier {
    /// Record a block frontier from the block coordinate and the outstanding run's incompletion, if any.
    /// `pub(in crate::campaign)` so only a block cursor's transition builds one.
    pub(in crate::campaign) fn new(block: BlockCoordinate, run: Option<RunIncompletion>) -> Self {
        Self { block, run }
    }

    /// The block that stopped short.
    pub(crate) fn block(&self) -> BlockCoordinate {
        self.block
    }

    /// The incompletion of the run that was outstanding when the block stopped, if a run was outstanding.
    pub(crate) fn run(&self) -> Option<&RunIncompletion> {
        self.run.as_ref()
    }
}
