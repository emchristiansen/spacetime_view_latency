//! How a run failed to be acquired before its linear cleanup owner could exist.

use crate::campaign::run_cleanup::ConnectFailure;
use crate::campaign::run_cleanup::ResolutionFailure;
use crate::provision::provision_failure::ProvisionFailure;

/// The three structurally distinct ways acquiring one run can fail *before* its
/// [`RunCleanup`](crate::campaign::run_cleanup::RunCleanup) is armed — i.e. before the cleanup "bomb"
/// exists — each carrying its arm's typed evidence rather than a flattened string. The arms are ordered by
/// the acquisition stage they fail at, so the failed stage is never erased:
///
/// - [`Self::Provision`]: provisioning the isolated server/staged WASM failed. No client had connected, so
///   there is no disconnect obligation. The typed [`ProvisionFailure`] retains the failed acquisition stage
///   plus every attempted partial-teardown result.
/// - [`Self::Connect`]: provisioning succeeded but the measured subscriber never connected. Still no
///   disconnect obligation. The typed [`ConnectFailure`] retains both the connect error and the
///   still-attempted resources-teardown result.
/// - [`Self::Resolve`]: the measured subscriber connected but resolving its authenticated role identities
///   failed (measured and growth identities collided). Because resolution runs *after* connection, this arm
///   owns both a client-disconnect and a resources-teardown obligation; the typed [`ResolutionFailure`]
///   retains the resolution error, the client-disconnect outcome, and the resources-teardown outcome.
///
/// In no arm did the run's cleanup owner ever exist, so there is no honest run terminal to mint — this is
/// deliberately *not* a [`RunIncomplete`](crate::campaign::run_cursor::RunIncomplete), which would require a
/// dishonest second cleanup witness. It is carried out of the campaign on the Err channel inside a
/// [`CampaignAborted`](super::campaign_aborted::CampaignAborted) alongside the finalized sink's
/// [`FinalizationOutcome`](super::finalization_outcome::FinalizationOutcome). `Debug`-only, matching its
/// typed-failure siblings.
#[derive(Debug)]
pub(crate) enum RunAcquisitionFailure {
    /// Provisioning the run's isolated resources failed; the typed stage + partial teardowns are retained.
    Provision(ProvisionFailure),
    /// The run's resources provisioned but the measured subscriber failed to connect; the connect error
    /// and the still-attempted teardown result are retained.
    Connect(ConnectFailure),
    /// The measured subscriber connected but resolving its authenticated role identities failed; the
    /// resolution error, client-disconnect outcome, and resources-teardown outcome are retained.
    Resolve(ResolutionFailure),
}
