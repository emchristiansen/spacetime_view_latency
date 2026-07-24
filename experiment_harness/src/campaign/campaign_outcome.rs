//! The single terminal outcome of driving a campaign to its end.

use super::campaign_cursor::CampaignComplete;
use super::campaign_incomplete::CampaignIncomplete;

/// What driving a campaign to its end yields: either it ran every scheduled block and finalized cleanly
/// ([`CampaignComplete`]), or it stopped short in one of the two structurally distinct ways
/// ([`CampaignIncomplete`]). There is no third state and no partial success: a campaign either holds
/// affine completion evidence or a typed incompleteness. Because [`CampaignComplete`] is affine, so is
/// this outcome — a completed campaign's evidence cannot be duplicated out of it.
#[derive(Debug)]
pub(crate) enum CampaignOutcome {
    /// Every block ran and the sink finalized cleanly.
    Complete(CampaignComplete),
    /// The campaign stopped short in execution or in the mandatory finalize.
    Incomplete(CampaignIncomplete),
}
