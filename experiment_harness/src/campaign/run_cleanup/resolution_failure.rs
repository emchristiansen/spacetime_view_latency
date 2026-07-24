//! How resolving a run's authenticated role identities failed after the measured client connected.

use anyhow::Error;

/// The typed failure of resolving a run's [`RoleIdentities`](crate::roles::role_identities::RoleIdentities)
/// after the measured subscriber connected but *before* its linear cleanup owner
/// ([`RunCleanup`](super::run_cleanup::RunCleanup)) was armed.
///
/// Identity resolution necessarily runs after connection — it pins the measured role to the
/// server-issued connection identity — so unlike a connect failure it owns **both** a client-disconnect
/// obligation and the resources-teardown obligation, yet [`RunCleanup`] cannot be armed until the
/// resolution succeeds (arming earlier would strand the cleanup "bomb" across this fallible step). So on a
/// resolution failure the client is disconnected and the resources are torn down here, and this retains
/// all three results verbatim — the resolution error, the client-disconnect outcome, and the
/// resources-teardown outcome — rather than flattening them into one message. A disconnect or teardown that
/// nonetheless succeeded (its `Ok(())`) or that also failed (its `Err`) is never dropped.
///
/// Holds `anyhow::Error` and so is `Debug`-only, matching its typed-failure siblings
/// ([`ConnectFailure`](super::connect_failure::ConnectFailure)). Lifted into
/// [`RunAcquisitionFailure::Resolve`](crate::campaign::run_acquisition_failure::RunAcquisitionFailure::Resolve)
/// by the campaign orchestrator.
#[derive(Debug)]
pub(crate) struct ResolutionFailure {
    /// The identity-resolution error (measured and growth identities collided).
    pub(in crate::campaign) resolve: Error,
    /// The still-attempted client-disconnect result, retained verbatim.
    pub(in crate::campaign) disconnect: Result<(), Error>,
    /// The still-attempted resources-teardown result, retained verbatim.
    pub(in crate::campaign) teardown: Result<(), Error>,
}

impl ResolutionFailure {
    /// Bundle the resolution error with the still-attempted client-disconnect and resources-teardown
    /// results. `pub(in crate::campaign)` confines construction to the campaign module subtree — not the
    /// acquisition path alone, which the visibility does not single out; the sole implemented caller is that
    /// path (`acquire_and_drive`).
    pub(in crate::campaign) fn new(
        resolve: Error,
        disconnect: Result<(), Error>,
        teardown: Result<(), Error>,
    ) -> Self {
        Self {
            resolve,
            disconnect,
            teardown,
        }
    }
}
