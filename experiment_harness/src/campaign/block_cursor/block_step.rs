//! The outcome of drawing the next run from a block's `[Run; 2]`.

use super::BlockDone;
use super::BlockRunPending;

/// What drawing the next run yields: either a run remains, handed out as a [`BlockRunPending`] that owns
/// the run's entry state, single-source coordinate, and the block's private continuation and drives the
/// concrete effect path itself; or the two-run iterator is drained and the block is done ([`BlockDone`]).
/// Exhaustion of the `[Run; 2]` iterator — not a count — is what completes the block.
///
/// The remaining-run arm no longer emits a parallel `run_plan`/`coordinate` pair for an external
/// orchestrator to provision against: the [`BlockRunPending`] carries the single-source coordinate and
/// provisions from it directly, so there is no separately emitted plan value that could drift from the run
/// the cursor opened.
pub(crate) enum BlockStep {
    /// A run remains, carried as the paired [`BlockRunPending`] that acquires-and-drives it.
    Running(BlockRunPending),
    /// The `[Run; 2]` iterator is drained; the block completed.
    Exhausted(BlockDone),
}
