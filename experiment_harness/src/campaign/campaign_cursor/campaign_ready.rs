//! The campaign state between blocks, and the opaque block draw it alone mints from its own fields.

use std::vec::IntoIter;

use crate::campaign::block_cursor::BlockDone;
use crate::campaign::block_cursor::BlockIncomplete;
use crate::campaign::campaign_frontier::CampaignFrontier;
use crate::campaign::campaign_incomplete::CampaignIncomplete;
use crate::campaign::campaign_outcome::CampaignOutcome;
use crate::campaign::finalization_outcome::FinalizationOutcome;
use crate::campaign::latest_progress::LatestProgress;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::observation::observation_sink::ObservationSink;
use crate::plan::schedule::BlockRun;
use crate::plan::schedule::Schedule;

use super::CampaignBlockPending;
use super::CampaignExhausted;
use super::CampaignStep;

/// A campaign part-way through its schedule: it owns the sink, the schedule seed, the schedule's *own*
/// remaining block iterator, and the typed high-water progress recorded from every block that has
/// completed so far. The block iterator is never accepted from a caller — [`Self::preregistered`]
/// derives it internally from the preregistered [`Schedule`] and the seed, and the same-module
/// [`BlockContinuation::finish`] rebuilds this state directly from the campaign's own partly-drained
/// iterator — so a caller can neither supply a schedule nor reorder, drop, or duplicate its blocks.
///
/// [`Self::next_block`] draws the next block from the iterator — minting a [`CampaignBlockDraw`] that binds
/// it to this campaign — or, when the iterator is drained, hands the recovered sink and the accumulated
/// progress to a [`CampaignExhausted`] for the mandatory finalize. Execution completeness is gated on the
/// iterator's exhaustion, never a count.
pub(crate) struct CampaignReady {
    sink: ObservationSink,
    seed: ScheduleSeed,
    blocks: IntoIter<BlockRun>,
    progress: LatestProgress,
}

impl CampaignReady {
    /// Begin the full preregistered campaign under a schedule seed. Derives the complete seed-randomized
    /// block order internally from [`Schedule::preregistered`] — the caller supplies neither the schedule
    /// nor the block collection — and starts progress at the explicit pre-execution state.
    /// `pub(crate)` because this is the campaign's sole external entry point: a driver hands in only the
    /// seed and the single owned sink.
    pub(crate) fn preregistered(seed: ScheduleSeed, sink: ObservationSink) -> Self {
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

    /// Draw the next block. If the block iterator yields one, mint a [`CampaignBlockDraw`] *by struct literal
    /// from this campaign's own fields* — the sink threaded in by move, the drawn block, and the campaign's
    /// private [`BlockContinuation`] remainder (the canonical seed, the still-owned block iterator, and the
    /// accumulated progress). Because the draw is built only here, from `self`, no code can assemble one from
    /// mismatched pieces; the block the campaign hands out is bound to the campaign's own identity by
    /// construction. When the iterator is drained, execution is complete: carry the recovered sink and
    /// progress out in a [`CampaignExhausted`], which alone performs the mandatory finalize.
    pub(crate) fn next_block(self) -> CampaignStep {
        let mut blocks = self.blocks;
        match blocks.next() {
            Some(block_run) => {
                let draw = CampaignBlockDraw {
                    sink: self.sink,
                    block_run,
                    continuation: BlockContinuation {
                        seed: self.seed,
                        blocks,
                        progress: self.progress,
                    },
                };
                CampaignStep::Running(CampaignBlockPending::from_draw(draw))
            }
            None => {
                CampaignStep::Exhausted(CampaignExhausted::new(self.sink, self.seed, self.progress))
            }
        }
    }
}

/// The opaque draw a campaign mints for its next block: the single value binding that block's owned campaign
/// sink, the drawn [`BlockRun`], and the campaign's private [`BlockContinuation`] remainder (the canonical
/// [`ScheduleSeed`], the campaign's own remaining block iterator, and the accumulated progress).
///
/// It has **no constructor**: its fields are private and it is built only by the struct literal inside
/// [`CampaignReady::next_block`], in this same module, from that campaign's own fields. So there is no API —
/// not even a module-private one — that assembles a draw from loose pieces; a block cannot be paired with a
/// foreign sink, seed, remainder, or progress because a draw can only come into being bound to the campaign
/// that drew it. The carrier [`CampaignBlockPending`] consumes exactly one draw via [`Self::into_parts`];
/// that consuming decomposition is the only capability exposed beyond this module.
pub(in crate::campaign::campaign_cursor) struct CampaignBlockDraw {
    sink: ObservationSink,
    block_run: BlockRun,
    continuation: BlockContinuation,
}

impl CampaignBlockDraw {
    /// Decompose the draw into the pieces the carrier drives with: the owned sink, the drawn block, and the
    /// campaign's remainder. `pub(in crate::campaign::campaign_cursor)` so the carrier can consume it; the
    /// pieces cannot be re-assembled into a draw (there is no draw constructor at all), so exposing them here
    /// opens no cross-pairing seam. The seed the initial block cursor is started under is read from the
    /// returned continuation ([`BlockContinuation::seed`]), so it is single-sourced with the seed the
    /// continuation later resumes the campaign under — there is no separate seed copy to diverge.
    pub(in crate::campaign::campaign_cursor) fn into_parts(
        self,
    ) -> (ObservationSink, BlockRun, BlockContinuation) {
        (self.sink, self.block_run, self.continuation)
    }
}

/// The campaign's continuation held privately while its one drawn block is outstanding: the schedule seed,
/// the campaign's *own* remaining block iterator, and the typed progress accumulated from every earlier
/// completed block. Holding the iterator here (never handing it to a caller) is what lets [`Self::finish`]
/// resume the campaign on its own partly-drained iterator.
///
/// Like [`CampaignBlockDraw`], it has **no constructor** — it is built only by the struct literal inside
/// [`CampaignReady::next_block`], from that campaign's own seed, remaining iterator, and progress, so its
/// seed and blocks are single-sourced and cannot be mismatched. Its `seed`/`finish`/`abort` are the only
/// capabilities exposed beyond this module, and the sole implemented caller is
/// [`CampaignBlockPending::drive`], which reads the seed to start the block cursor and later hands each of
/// `finish`/`abort` the terminal of the exact block the carrier just drove — retiring the old
/// block/continuation coordinate `assert_eq!`.
///
/// Exactly one block is outstanding at a time — a campaign never starts a second block before the first
/// resolves — so the campaign's at-most-one-outstanding-child invariant is structural, not counted.
pub(in crate::campaign::campaign_cursor) struct BlockContinuation {
    seed: ScheduleSeed,
    blocks: IntoIter<BlockRun>,
    progress: LatestProgress,
}

impl BlockContinuation {
    /// The canonical schedule seed, read to start the outstanding block cursor under the *same* seed this
    /// continuation resumes the campaign under. Copy out, so the borrow ends immediately and the continuation
    /// is still owned to fold the block terminal back in.
    pub(in crate::campaign::campaign_cursor) fn seed(&self) -> ScheduleSeed {
        self.seed
    }

    /// Fold a completed block back into the campaign: advance the typed progress from the block's completion
    /// evidence, then resume the campaign on its own preserved iterator and seed. The completed block is this
    /// continuation's own block by construction (the continuation is built only in
    /// [`CampaignReady::next_block`] and reached only through the carrier that drove the block), so no
    /// coordinate cross-check is needed. Consuming the affine
    /// [`BlockComplete`](crate::campaign::block_cursor::BlockComplete) (borrowed to record progress, then
    /// dropped) means one finished block advances progress exactly once. Rebuilds the campaign *by struct
    /// literal* from this continuation's own seed and preserved iterator, so no independently supplied
    /// iterator, seed, or progress can diverge from the campaign that drew the block.
    pub(in crate::campaign::campaign_cursor) fn finish(self, done: BlockDone) -> CampaignReady {
        let (sink, complete) = done.into_parts();
        let mut progress = self.progress;
        progress.record_block(&complete);
        CampaignReady {
            sink,
            seed: self.seed,
            blocks: self.blocks,
            progress,
        }
    }

    /// Conclude the campaign because its outstanding block stopped short. Lift the block's frontier into a
    /// [`CampaignFrontier`] at this campaign's seed, then **always** finalize the recovered sink in this
    /// same terminal transition, so the required finalize can never be forgotten on the execution-failure
    /// path. The resulting [`CampaignIncomplete::Execution`] retains both the execution frontier and the
    /// typed [`FinalizationOutcome`]; neither error is discarded. The stopped block is this continuation's
    /// own block by construction, so no coordinate cross-check is needed. The [`CampaignFrontier`] records
    /// the seed (so the exact block order is named) and the failing block's frontier down to its
    /// run/stage/record tail; it does not additionally carry earlier completed blocks' progress, which the
    /// execution frontier is not required to restate.
    pub(in crate::campaign::campaign_cursor) fn abort(
        self,
        incomplete: BlockIncomplete,
    ) -> CampaignOutcome {
        let (sink, block_frontier) = incomplete.into_parts();
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
