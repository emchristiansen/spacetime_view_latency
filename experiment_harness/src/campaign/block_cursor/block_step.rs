//! The outcome of drawing the next run from a block's `[Run; 2]`.

use crate::campaign::run_cursor::RunWritingManifest;

use super::BlockDone;
use super::BlockRunningRun;

/// What drawing the next run yields: either a run remains and starts writing its manifest
/// ([`RunWritingManifest`]) with the block's continuation held in a [`BlockRunningRun`], or the two-run
/// iterator is drained and the block is done ([`BlockDone`]). Exhaustion of the `[Run; 2]` iterator —
/// not a count — is what completes the block.
pub(crate) enum BlockStep {
    /// A run remains; it is starting its manifest write, and `resume` holds the block's continuation.
    Running {
        run: RunWritingManifest,
        resume: BlockRunningRun,
    },
    /// The `[Run; 2]` iterator is drained; the block completed.
    Exhausted(BlockDone),
}
