//! The block continuation while one of its runs is outstanding.

use std::array::IntoIter;

use crate::campaign::block_frontier::BlockFrontier;
use crate::campaign::run_cursor::RunDone;
use crate::campaign::run_cursor::RunIncomplete;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::plan::run::Run;
use crate::plan::schedule::BlockCoordinate;
use crate::plan::schedule::BlockRun;

use super::BlockIncomplete;
use super::BlockReady;

/// The block's continuation held while exactly one of its runs is outstanding: the block it is running,
/// the block's own remaining run iterator, and the outstanding run's coordinate. Holding the iterator
/// here (never handing it to a caller) is what lets [`Self::finish`] resume the block on its *own*
/// partly-drained iterator.
///
/// Exactly one run is outstanding at a time — a block never starts a second run before the first
/// resolves — so the block's at-most-one-outstanding-child invariant is structural, not counted.
pub(crate) struct BlockRunningRun {
    block_run: BlockRun,
    runs: IntoIter<Run, 2>,
    run_coord: RunCoordinate,
}

impl BlockRunningRun {
    /// Hold a block's continuation while its drawn run is outstanding. `pub(super)` so only
    /// [`BlockReady::next_run`] — not a campaign sibling — supplies the private run iterator.
    pub(super) fn new(
        block_run: BlockRun,
        runs: IntoIter<Run, 2>,
        run_coord: RunCoordinate,
    ) -> Self {
        Self {
            block_run,
            runs,
            run_coord,
        }
    }

    /// Fold a completed run back into the block, resuming it on its own preserved iterator with the
    /// run's completion evidence. The completed run's coordinate must match the outstanding run's — a
    /// mismatch is a wiring bug and fails loud.
    pub(in crate::campaign) fn finish(self, done: RunDone) -> BlockReady {
        let (sink, complete) = done.into_parts();
        assert_eq!(
            complete.run(),
            &self.run_coord,
            "the completed run's coordinate must match the outstanding run's coordinate"
        );
        BlockReady::resume(sink, self.block_run, self.runs, complete)
    }

    /// Abort the block because its outstanding run stopped short: recover the sink and lift the run's
    /// frontier into a [`BlockFrontier`] at this block's coordinate. The stopped run's coordinate must
    /// match the outstanding run's — exactly as [`Self::finish`] binds a completion — so a foreign run's
    /// failure cannot be lifted into this block; a mismatch is a wiring bug and fails loud. The block
    /// holds at most one outstanding run, so exactly one run frontier is possible.
    pub(in crate::campaign) fn abort(self, incomplete: RunIncomplete) -> BlockIncomplete {
        let (sink, frontier) = incomplete.into_parts();
        assert_eq!(
            frontier.run(),
            &self.run_coord,
            "the stopped run's coordinate must match the outstanding run's coordinate"
        );
        let block_frontier = BlockFrontier::new(BlockCoordinate::of(&self.block_run), Some(frontier));
        BlockIncomplete::new(sink, block_frontier)
    }
}
