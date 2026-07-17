//! The campaign state between blocks: holding the whole seeded block iterator, ready to draw the next block.

use std::vec::IntoIter;

use crate::campaign::block_cursor::BlockReady;
use crate::campaign::latest_progress::LatestProgress;
use crate::observation::observation_sink::ObservationSink;
use crate::plan::schedule::BlockCoordinate;
use crate::plan::schedule::BlockRun;
use crate::plan::schedule::Schedule;

use super::CampaignExhausted;
use super::CampaignRunningBlock;
use super::CampaignStep;

/// A campaign part-way through its schedule: it owns the sink, the schedule seed, the schedule's *own*
/// remaining block iterator, and the typed high-water progress recorded from every block that has
/// completed so far. The block iterator is never accepted from a caller — [`Self::preregistered`]
/// derives it internally from the preregistered [`Schedule`] and the seed, and [`Self::resume`] only
/// preserves the campaign's own partly-drained iterator — so a caller can neither supply a schedule nor
/// reorder, drop, or duplicate its blocks.
///
/// [`Self::next_block`] draws the next block from the iterator — starting it at [`BlockReady`] with the
/// sink threaded in by move — or, when the iterator is drained, hands the recovered sink and the
/// accumulated progress to a [`CampaignExhausted`] for the mandatory finalize. Execution completeness is
/// gated on the iterator's exhaustion, never a count.
pub(crate) struct CampaignReady {
    sink: ObservationSink,
    seed: u64,
    blocks: IntoIter<BlockRun>,
    progress: LatestProgress,
}

impl CampaignReady {
    /// Begin the full preregistered campaign under a schedule seed. Derives the complete seed-randomized
    /// block order internally from [`Schedule::preregistered`] — the caller supplies neither the schedule
    /// nor the block collection — and starts progress at the explicit pre-execution state.
    /// `pub(crate)` because this is the campaign's sole external entry point: a driver hands in only the
    /// seed and the single owned sink.
    pub(crate) fn preregistered(seed: u64, sink: ObservationSink) -> Self {
        let blocks = Schedule::preregistered()
            .randomized_block_order(seed)
            .into_iter();
        Self {
            sink,
            seed,
            blocks,
            progress: LatestProgress::before_execution(),
        }
    }

    /// Resume a campaign after one of its blocks completed, preserving the campaign's own partly-drained
    /// block iterator and the progress already advanced past that block. `pub(super)` so only a
    /// campaign-cursor continuation ([`CampaignRunningBlock`]) resumes a campaign — no external caller can
    /// supply the iterator or the seed.
    pub(super) fn resume(
        sink: ObservationSink,
        seed: u64,
        blocks: IntoIter<BlockRun>,
        progress: LatestProgress,
    ) -> Self {
        Self {
            sink,
            seed,
            blocks,
            progress,
        }
    }

    /// Draw the next block. If the block iterator yields one, start it at [`BlockReady`] with the sink
    /// threaded in by move, handing the campaign's continuation (the remaining iterator, the outstanding
    /// block's coordinate, and the accumulated progress) to a [`CampaignRunningBlock`]. When the iterator
    /// is drained, execution is complete: carry the recovered sink and progress out in a
    /// [`CampaignExhausted`], which alone performs the mandatory finalize.
    pub(crate) fn next_block(self) -> CampaignStep {
        let mut blocks = self.blocks;
        match blocks.next() {
            Some(block_run) => {
                let block_coord = BlockCoordinate::of(&block_run);
                let block = BlockReady::begin(self.sink, block_run, self.seed);
                let resume =
                    CampaignRunningBlock::new(self.seed, blocks, block_coord, self.progress);
                CampaignStep::Running { block, resume }
            }
            None => CampaignStep::Exhausted(CampaignExhausted::new(
                self.sink,
                self.seed,
                self.progress,
            )),
        }
    }
}
