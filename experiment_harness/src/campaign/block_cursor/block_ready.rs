//! The block state between runs: holding the block's own `[Run; 2]` iterator, ready to draw the next run.

use std::array::IntoIter;

use crate::campaign::run_cursor::RunComplete;
use crate::campaign::run_cursor::RunWritingManifest;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::observation::observation_sink::ObservationSink;
use crate::plan::run::Run;
use crate::plan::schedule::BlockCoordinate;
use crate::plan::schedule::BlockRun;

use super::BlockComplete;
use super::BlockDone;
use super::BlockRunningRun;
use super::BlockStep;

/// Whether a between-runs block has completed a run yet. Private, so only [`BlockReady`]'s
/// state-specific constructors set it: a fresh block is [`Self::NoneYet`], and a resumed block carries
/// exactly the [`RunComplete`] of its most recent run. This makes the completion state unforgeable — no
/// caller can pair a fresh block with a fabricated completion, nor resume a block with none — and each
/// completed run supersedes the previous one, so only the last is kept.
enum LastRun {
    /// A fresh block, before its first run completes.
    NoneYet,
    /// The block's most recently completed run.
    Completed(RunComplete),
}

/// A block part-way through its two runs: it owns the sink, the block it is running, the block's *own*
/// remaining run iterator, and whether a run has completed yet. The run iterator is never accepted from
/// a caller — [`Self::begin`] derives it from the block and seed, and [`Self::resume`] only preserves
/// the block's own partly-drained iterator — so a reordered or foreign run set is unrepresentable.
///
/// [`Self::next_run`] draws the next run from the iterator — starting it at [`RunWritingManifest`] — or,
/// when the iterator is drained, mints the block's completion from the last completed run. Completion is
/// gated on the iterator's exhaustion, never a count.
pub(crate) struct BlockReady {
    sink: ObservationSink,
    block_run: BlockRun,
    runs: IntoIter<Run, 2>,
    last_run: LastRun,
}

impl BlockReady {
    /// Begin a fresh block. Derives the block's seed-ordered `[Run; 2]` internally from its own
    /// `block_run` and the schedule seed — the caller supplies neither the run collection nor its order —
    /// and encodes no completed run. `pub(in crate::campaign)` so only a campaign cursor starts a block.
    pub(in crate::campaign) fn begin(sink: ObservationSink, block_run: BlockRun, seed: u64) -> Self {
        let runs = block_run.ordered_runs(seed).into_iter();
        Self {
            sink,
            block_run,
            runs,
            last_run: LastRun::NoneYet,
        }
    }

    /// Resume a block after one of its runs completed, preserving the block's own partly-drained run
    /// iterator and *requiring* the completed run's evidence. `pub(super)` so only a block-cursor
    /// continuation ([`BlockRunningRun`]) resumes a block — no campaign-level caller can supply the
    /// iterator or the completion.
    pub(super) fn resume(
        sink: ObservationSink,
        block_run: BlockRun,
        runs: IntoIter<Run, 2>,
        complete: RunComplete,
    ) -> Self {
        Self {
            sink,
            block_run,
            runs,
            last_run: LastRun::Completed(complete),
        }
    }

    /// Draw the next run. If the `[Run; 2]` iterator yields one, start it at [`RunWritingManifest`] with
    /// the sink threaded in by move, handing the block's continuation (and the remaining iterator) to a
    /// [`BlockRunningRun`]. When the iterator is drained, the block is complete: mint [`BlockComplete`]
    /// from the last completed run and carry the recovered sink out in a [`BlockDone`].
    pub(in crate::campaign) fn next_run(self) -> BlockStep {
        let mut runs = self.runs;
        match runs.next() {
            Some(run) => {
                let coord = RunCoordinate::new(&self.block_run, run.role());
                let writing = RunWritingManifest::begin(self.sink, coord.clone());
                let resume = BlockRunningRun::new(self.block_run, runs, coord);
                BlockStep::Running {
                    run: writing,
                    resume,
                }
            }
            None => {
                let complete = match self.last_run {
                    LastRun::Completed(complete) => complete,
                    LastRun::NoneYet => unreachable!(
                        "a block runs exactly two runs, so its run iterator drains only after at least one run completed"
                    ),
                };
                let block_complete =
                    BlockComplete::new(BlockCoordinate::of(&self.block_run), complete);
                BlockStep::Exhausted(BlockDone::new(self.sink, block_complete))
            }
        }
    }
}
