//! The block state between runs, and the opaque run draw it alone mints from its own fields.

use std::array::IntoIter;

use crate::campaign::block_cursor::BlockComplete;
use crate::campaign::block_cursor::BlockDone;
use crate::campaign::block_cursor::BlockIncomplete;
use crate::campaign::block_cursor::BlockRunPending;
use crate::campaign::block_cursor::BlockStep;
use crate::campaign::block_frontier::BlockFrontier;
use crate::campaign::run_cursor::RunComplete;
use crate::campaign::run_cursor::RunDone;
use crate::campaign::run_cursor::RunIncomplete;
use crate::campaign::run_cursor::RunWritingManifest;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::observation::observation_sink::ObservationSink;
use crate::plan::run::Run;
use crate::plan::schedule::BlockCoordinate;
use crate::plan::schedule::BlockRun;

/// Whether a between-runs block has completed a run yet. Private, so only [`BlockReady`]'s own literal
/// constructions set it: a fresh block is [`Self::NoneYet`], and a resumed block carries exactly the
/// [`RunComplete`] of its most recent run. This makes the completion state unforgeable — no caller can pair a
/// fresh block with a fabricated completion, nor resume a block with none — and each completed run supersedes
/// the previous one, so only the last is kept.
enum LastRun {
    /// A fresh block, before its first run completes.
    NoneYet,
    /// The block's most recently completed run.
    Completed(RunComplete),
}

/// A block part-way through its two runs: it owns the sink, the block it is running, the block's *own*
/// remaining run iterator, and whether a run has completed yet. The run iterator is never accepted from
/// a caller — [`Self::begin`] derives it from the block and seed, and the same-module
/// [`RunContinuation::finish`] rebuilds this state directly from the continuation's own preserved iterator
/// and seed — so a reordered or foreign run set is unrepresentable.
///
/// [`Self::next_run`] draws the next run from the iterator — minting a [`BlockRunDraw`] that binds it to
/// this block — or, when the iterator is drained, mints the block's completion from the last completed run.
/// Completion is gated on the iterator's exhaustion, never a count.
pub(crate) struct BlockReady {
    sink: ObservationSink,
    block_run: BlockRun,
    runs: IntoIter<Run, 2>,
    seed: ScheduleSeed,
    last_run: LastRun,
}

impl BlockReady {
    /// Begin a fresh block. Derives the block's seed-ordered `[Run; 2]` internally from its own
    /// `block_run` and the schedule seed — the caller supplies neither the run collection nor its order —
    /// and encodes no completed run. `pub(in crate::campaign)` so only a campaign cursor starts a block;
    /// the sole implemented caller is the campaign carrier consuming its own opaque draw
    /// ([`CampaignBlockPending`](crate::campaign::campaign_cursor::CampaignBlockPending)). `pub(in crate::campaign)`
    /// is the tightest cross-module floor Rust can express here; a stray `begin` outside that path yields a
    /// block cursor bound to no campaign continuation (continuations live only inside campaign draws, which
    /// only `next_block` mints), so it cannot be folded into a campaign. There is no separate `resume`
    /// constructor: the only other way this state is reached is the same-module [`RunContinuation::finish`],
    /// which rebuilds it directly by struct literal from the continuation's own fields.
    pub(in crate::campaign) fn begin(
        sink: ObservationSink,
        block_run: BlockRun,
        seed: ScheduleSeed,
    ) -> Self {
        let runs = block_run.ordered_runs(seed).into_iter();
        Self {
            sink,
            block_run,
            runs,
            seed,
            last_run: LastRun::NoneYet,
        }
    }

    /// Draw the next run. If the `[Run; 2]` iterator yields one, mint a [`BlockRunDraw`] *by struct literal
    /// from this block's own fields* — the run's writing state (the sink threaded in by move), the coordinate
    /// derived here from this block and the drawn run's role, and the block's private [`RunContinuation`]
    /// remainder (the still-owned iterator *and* the canonical seed). Because the draw is built only here,
    /// from `self`, no code can assemble one from mismatched pieces; the run the block hands out is bound to
    /// the block's own identity by construction. When the iterator is drained, the block is complete: mint
    /// [`BlockComplete`] from the last completed run and carry the recovered sink out in a [`BlockDone`].
    pub(in crate::campaign) fn next_run(self) -> BlockStep {
        let mut runs = self.runs;
        match runs.next() {
            Some(run) => {
                let coordinate = RunCoordinate::new(&self.block_run, run.role());
                let writing = RunWritingManifest::begin(self.sink);
                let draw = BlockRunDraw {
                    writing,
                    coordinate,
                    continuation: RunContinuation {
                        block_run: self.block_run,
                        runs,
                        seed: self.seed,
                    },
                };
                BlockStep::Running(BlockRunPending::from_draw(draw))
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

/// The opaque draw a block mints for its next run: the single value binding that run's manifest-writing
/// entry state (which owns the campaign sink), the run's single-source [`RunCoordinate`] (derived in
/// [`BlockReady::next_run`] from the block and the drawn role), and the block's private [`RunContinuation`]
/// remainder — which itself owns the canonical [`ScheduleSeed`], so there is no separate seed copy here to
/// diverge from the one the block resumes under.
///
/// It has **no constructor**: its fields are private and it is built only by the struct literal inside
/// [`BlockReady::next_run`], in this same module, from that block's own fields. So there is no API — not even
/// a module-private one — that assembles a draw from loose pieces; a run cannot be paired with a foreign
/// coordinate or remainder because a draw can only come into being bound to the block that drew it. The
/// carrier [`BlockRunPending`] consumes exactly one draw via [`Self::into_parts`]; that consuming
/// decomposition is the only capability exposed beyond this module.
pub(in crate::campaign::block_cursor) struct BlockRunDraw {
    writing: RunWritingManifest,
    coordinate: RunCoordinate,
    continuation: RunContinuation,
}

impl BlockRunDraw {
    /// Decompose the draw into the pieces the carrier drives with: the writing entry state, the run's
    /// coordinate, and the block's remainder. The seed the run is acquired under is read from the returned
    /// continuation ([`RunContinuation::seed`]), so it is single-sourced with the seed the continuation later
    /// resumes the block under — there is no separate seed to reconcile.
    /// `pub(in crate::campaign::block_cursor)` so the carrier can consume it; the pieces cannot be
    /// re-assembled into a draw (there is no draw constructor at all), so exposing them here opens no
    /// cross-pairing seam.
    pub(in crate::campaign::block_cursor) fn into_parts(
        self,
    ) -> (RunWritingManifest, RunCoordinate, RunContinuation) {
        (self.writing, self.coordinate, self.continuation)
    }
}

/// The block's continuation held privately while its one drawn run is outstanding: the block it is running,
/// the block's *own* remaining run iterator, and the canonical seed. Holding the iterator and seed here
/// (never handing either to a caller as a loose argument) is what lets [`Self::finish`] rebuild the block on
/// its own partly-drained iterator under its own seed.
///
/// Like [`BlockRunDraw`], it has **no constructor** — it is built only by the struct literal inside
/// [`BlockReady::next_run`], from that block's own `block_run`, remaining iterator, and seed, so its block,
/// runs, and seed are single-sourced and cannot be mismatched. Its `seed`/`finish`/`abort` are the only
/// capabilities exposed beyond this module, and the sole implemented caller is [`BlockRunPending::drive`],
/// which reads the seed to acquire the run and later hands `finish`/`abort` the terminal of the exact run the
/// carrier just drove — retiring the old run/continuation coordinate `assert_eq!`.
///
/// Exactly one run is outstanding at a time — a block never starts a second run before the first resolves —
/// so the block's at-most-one-outstanding-child invariant is structural, not counted.
pub(in crate::campaign::block_cursor) struct RunContinuation {
    block_run: BlockRun,
    runs: IntoIter<Run, 2>,
    seed: ScheduleSeed,
}

impl RunContinuation {
    /// The canonical schedule seed, read to acquire the outstanding run under the *same* seed this
    /// continuation resumes the block under. Copy out, so the borrow ends immediately and the continuation is
    /// still owned to fold the run terminal back in.
    pub(in crate::campaign::block_cursor) fn seed(&self) -> ScheduleSeed {
        self.seed
    }

    /// Fold a completed run back into the block, rebuilding it *by struct literal* on its own preserved
    /// iterator and canonical seed with the run's completion evidence. The completed run is this
    /// continuation's own run by construction (the continuation is built only in [`BlockReady::next_run`] and
    /// reached only through the carrier that drove the run), so no coordinate cross-check is needed, and the
    /// resumed block's seed is the continuation's own — no independently supplied seed can diverge from the
    /// one the run was drawn under.
    pub(in crate::campaign::block_cursor) fn finish(self, done: RunDone) -> BlockReady {
        let (sink, complete) = done.into_parts();
        BlockReady {
            sink,
            block_run: self.block_run,
            runs: self.runs,
            seed: self.seed,
            last_run: LastRun::Completed(complete),
        }
    }

    /// Abort the block because its outstanding run stopped short: recover the sink and lift the run's
    /// frontier into a [`BlockFrontier`] at this block's coordinate. The stopped run is this continuation's
    /// own run by construction, so no coordinate cross-check is needed. The block holds at most one
    /// outstanding run, so exactly one run frontier is possible.
    pub(in crate::campaign::block_cursor) fn abort(
        self,
        incomplete: RunIncomplete,
    ) -> BlockIncomplete {
        let (sink, incompletion) = incomplete.into_parts();
        let block_frontier =
            BlockFrontier::new(BlockCoordinate::of(&self.block_run), Some(incompletion));
        BlockIncomplete::new(sink, block_frontier)
    }
}
