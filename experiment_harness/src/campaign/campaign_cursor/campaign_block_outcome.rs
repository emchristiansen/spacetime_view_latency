//! What driving one of a campaign's blocks yields: resume the campaign, or conclude it at a terminal.

use crate::campaign::campaign_outcome::CampaignOutcome;

use super::CampaignReady;

/// The result of
/// [`CampaignBlockPending::drive`](super::campaign_block_pending::CampaignBlockPending::drive) folding one
/// block's terminal back into its campaign: either the block completed and the campaign resumes on its own
/// block iterator ([`CampaignReady`]), or one of the block's runs stopped short and the campaign concluded
/// at a terminal [`CampaignOutcome`] (an execution-stage
/// [`CampaignIncomplete`](crate::campaign::campaign_incomplete::CampaignIncomplete) whose sink is already
/// finalized).
///
/// This is only the *successfully acquired* outcome; a pre-cleanup acquisition failure is not a block
/// outcome at all — it short-circuits on the driving method's Err channel as a
/// [`CampaignAborted`](crate::campaign::campaign_aborted::CampaignAborted). `pub(crate)` to match the
/// campaign cursor's other step types, though only the campaign orchestrator consumes it.
pub(crate) enum CampaignBlockOutcome {
    /// The block completed; the campaign resumes for its next block.
    Resumed(CampaignReady),
    /// A run stopped short; the campaign concluded at its execution-stage terminal.
    Concluded(CampaignOutcome),
}
