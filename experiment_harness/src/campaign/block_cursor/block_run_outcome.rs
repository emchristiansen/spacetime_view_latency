//! What driving one of a block's runs yields for the block: resume the block, or abort it at the frontier.

use super::BlockIncomplete;
use super::BlockReady;

/// The result of [`BlockRunPending::drive`](super::block_run_pending::BlockRunPending::drive) folding one
/// run's settled terminal back into its block: either the run completed and the block resumes on its own
/// iterator ([`BlockReady`]), or the run stopped short and the block aborted at its frontier
/// ([`BlockIncomplete`]).
///
/// This is only the *successfully acquired* outcome; a pre-cleanup acquisition failure is not a block
/// outcome at all — it short-circuits on the driving method's Err channel as a
/// [`CampaignAborted`](crate::campaign::campaign_aborted::CampaignAborted). `pub(crate)` to match the block
/// cursor's other step types, though only the campaign-block carrier consumes it.
pub(crate) enum BlockRunOutcome {
    /// The run completed; the block resumes for its next run.
    Resumed(BlockReady),
    /// The run stopped short; the block aborted at its frontier.
    Aborted(BlockIncomplete),
}
