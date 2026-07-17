//! The outcome of drawing the next block from a campaign's schedule.

use crate::campaign::block_cursor::BlockReady;

use super::CampaignExhausted;
use super::CampaignRunningBlock;

/// What drawing the next block yields: either a block remains and is starting its runs ([`BlockReady`])
/// with the campaign's continuation held in a [`CampaignRunningBlock`], or the schedule's block iterator
/// is drained and execution is complete ([`CampaignExhausted`], which alone performs the mandatory
/// finalize). Exhaustion of the block iterator — not a count — is what ends execution.
pub(crate) enum CampaignStep {
    /// A block remains; it is starting its runs, and `resume` holds the campaign's continuation.
    Running {
        block: BlockReady,
        resume: CampaignRunningBlock,
    },
    /// The schedule's block iterator is drained; execution is complete and awaits finalize.
    Exhausted(CampaignExhausted),
}
