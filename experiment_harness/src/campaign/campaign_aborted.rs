//! The typed pre-cleanup campaign abort: an acquisition failure plus the finalized sink's outcome.

use super::finalization_outcome::FinalizationOutcome;
use super::run_acquisition_failure::RunAcquisitionFailure;

/// A campaign that aborted while acquiring one run *before* that run's linear cleanup owner
/// ([`RunCleanup`](crate::campaign::run_cleanup::RunCleanup)) could exist — see [`RunAcquisitionFailure`].
/// Because no measured client ever connected, there is no disconnect obligation and no honest run terminal
/// to mint. The campaign instead recovers its owned sink from the run-entry
/// [`RunWritingManifest`](crate::campaign::run_cursor::RunWritingManifest) typestate via
/// [`abandon`](crate::campaign::run_cursor::RunWritingManifest::abandon), finalizes it, and carries *both*
/// the typed acquisition failure and that finalization's [`FinalizationOutcome`] out on the campaign's Err
/// channel — neither the acquisition failure nor the sink-sealing outcome is discarded. `Debug`-only,
/// matching its typed-failure siblings.
#[derive(Debug)]
pub(crate) struct CampaignAborted {
    /// Why acquiring the run failed before any cleanup owner existed, with its arm's typed dual evidence.
    pub(in crate::campaign) failure: RunAcquisitionFailure,
    /// What happened when the campaign finalized its recovered sink after the acquisition failure.
    pub(in crate::campaign) finalization: FinalizationOutcome,
}

impl CampaignAborted {
    /// Bundle the acquisition failure with the recovered sink's finalization outcome. `pub(in crate::campaign)`
    /// so only the campaign orchestrator mints one, after it has recovered and finalized the sink.
    pub(in crate::campaign) fn new(
        failure: RunAcquisitionFailure,
        finalization: FinalizationOutcome,
    ) -> Self {
        Self {
            failure,
            finalization,
        }
    }
}
