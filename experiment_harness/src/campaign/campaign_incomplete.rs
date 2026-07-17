//! The two structurally distinct ways a campaign can stop short of a clean completion.

use crate::campaign::campaign_frontier::CampaignFrontier;
use crate::campaign::finalization_outcome::FinalizationOutcome;
use crate::campaign::latest_progress::LatestProgress;
use crate::observation::finalize_error::FinalizeError;

/// How a campaign failed to complete cleanly — exactly one of two structurally distinct cases, so no
/// impossible combination of a present execution frontier with an absent finalization (or vice versa) is
/// representable:
///
/// - [`Self::Execution`]: a block stopped short. Execution failure carries the required
///   [`CampaignFrontier`] (the exact failed block/run/stage/record tail under the schedule seed) *and*
///   the required typed [`FinalizationOutcome`] — because finalize is always attempted in the aborting
///   transition, both the execution failure and the finalize result are retained, never one without the
///   other.
/// - [`Self::Finalization`]: every block completed, but the mandatory finalize failed. Finalization-only
///   failure carries the typed [`LatestProgress`] high-water mark from the last completed block *and* the
///   sink's required [`FinalizeError`] — execution succeeded, so there is no execution frontier, and the
///   accumulated progress is exactly the typed high-water progress whose writer contracts returned
///   success before finalization failed.
///
/// Holding either variant is evidence about which record writes' contracts returned success; neither is a
/// claim of physical crash persistence.
#[derive(Debug)]
pub(crate) enum CampaignIncomplete {
    /// A block stopped short: the execution frontier and the always-attempted finalization outcome.
    Execution {
        frontier: CampaignFrontier,
        finalization: FinalizationOutcome,
    },
    /// Every block completed but the mandatory finalize failed: the retained progress and the typed error.
    Finalization {
        progress: LatestProgress,
        error: FinalizeError,
    },
}
