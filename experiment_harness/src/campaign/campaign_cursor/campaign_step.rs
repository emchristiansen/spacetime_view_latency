//! The outcome of drawing the next block from a campaign's schedule.

use super::CampaignBlockPending;
use super::CampaignExhausted;

/// What drawing the next block yields: either a block remains, handed out as a [`CampaignBlockPending`]
/// that owns the block cursor to drive and the campaign's private continuation and loops the concrete block
/// runs itself; or the schedule's block iterator is drained and execution is complete ([`CampaignExhausted`],
/// which alone performs the mandatory finalize). Exhaustion of the block iterator — not a count — is what
/// ends execution.
pub(crate) enum CampaignStep {
    /// A block remains, carried as the paired [`CampaignBlockPending`] that drives it.
    Running(CampaignBlockPending),
    /// The schedule's block iterator is drained; execution is complete and awaits finalize.
    Exhausted(CampaignExhausted),
}
