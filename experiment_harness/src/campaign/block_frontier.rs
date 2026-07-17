//! The structural record of where and how a repetition block stopped short of completion.

use crate::plan::schedule::BlockCoordinate;

use super::run_frontier::RunFrontier;

/// Where a block stopped: the block coordinate, and the frontier of the run that stopped, if a run was
/// outstanding when the block failed.
///
/// `run` is `Option` because a block can fail with no run frontier of its own to name — this milestone
/// only stops *inside* a run (so `run` is `Some`), but the structure admits a future block-scoped stop
/// (e.g. a between-runs reset) that carries no run frontier. A block holds at most one outstanding run,
/// so at most one [`RunFrontier`] is possible; there is no collection of run frontiers to reconcile.
#[derive(Debug)]
pub(crate) struct BlockFrontier {
    block: BlockCoordinate,
    run: Option<RunFrontier>,
}

impl BlockFrontier {
    /// Record a block frontier from the block coordinate and the outstanding run's frontier, if any.
    /// `pub(in crate::campaign)` so only a block cursor's transition builds one.
    pub(in crate::campaign) fn new(block: BlockCoordinate, run: Option<RunFrontier>) -> Self {
        Self { block, run }
    }

    /// The block that stopped short.
    pub(crate) fn block(&self) -> BlockCoordinate {
        self.block
    }

    /// The frontier of the run that was outstanding when the block stopped, if a run was outstanding.
    pub(crate) fn run(&self) -> Option<&RunFrontier> {
        self.run.as_ref()
    }
}
