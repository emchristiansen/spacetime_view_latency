//! How connecting the measured subscriber failed before a run's cleanup owner could exist.

use anyhow::Error;

/// The typed failure of connecting the measured subscriber during run acquisition: it never connected, so
/// no [`RunCleanup`](super::run_cleanup::RunCleanup) — and no disconnect obligation — was ever armed, but
/// the provisioned resources *were* still torn down, and that teardown's result is retained verbatim rather
/// than flattened into the connect error.
///
/// This mirrors the shape of [`RunCleanupFailure::Disconnect`](crate::campaign::run_cleanup_failure::RunCleanupFailure::Disconnect)
/// on the settlement side: a primary effect error paired with a still-attempted teardown result, so a
/// teardown that nonetheless succeeded (its `Ok(())`) or that also failed (its `Err`, the dual-failure
/// case) is never dropped. Holds `anyhow::Error` and so is `Debug`-only, matching its siblings
/// ([`ResolutionFailure`](super::resolution_failure::ResolutionFailure)). Produced only by the campaign
/// acquisition path and lifted into
/// [`RunAcquisitionFailure::Connect`](crate::campaign::run_acquisition_failure::RunAcquisitionFailure::Connect).
#[derive(Debug)]
pub(crate) struct ConnectFailure {
    /// The connection handshake error.
    pub(in crate::campaign) connect: Error,
    /// The still-attempted resources teardown result, retained verbatim.
    pub(in crate::campaign) teardown: Result<(), Error>,
}

impl ConnectFailure {
    /// Bundle the connect error with the still-attempted teardown result. `pub(in crate::campaign)`
    /// confines construction to the campaign module subtree — not the acquisition path alone, which the
    /// visibility does not single out; the sole implemented caller is that path (`acquire_and_drive`).
    pub(in crate::campaign) fn new(connect: Error, teardown: Result<(), Error>) -> Self {
        Self { connect, teardown }
    }
}
