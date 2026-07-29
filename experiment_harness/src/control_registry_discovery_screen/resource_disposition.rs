//! What became of the resources this driver owned when an attempt ended.

use anyhow::{bail, Result};
use serde::Serialize;

use crate::control_registry_discovery_screen::diagnostic_artifact::DiagnosticArtifact;

/// The truthful disposition of every resource **this driver owned** at the point an attempt ended.
///
/// Three variants rather than two, because "released" is a false claim where nothing releasable ever
/// existed: resolving the pinned distribution and staging the module WASM both fail before the
/// driver owns anything, and a record asserting a successful release there would describe a cleanup
/// that never ran.
///
/// **Scope, which a reader must not widen.** This describes only resources returned to and owned by
/// the driver. Two provisioning constructors clean up their own partial state before returning
/// `Err`, and their cleanup failures stay inside the provisioning diagnostic rather than appearing
/// here:
///
/// - [`RunningPinnedServer::start`](crate::provision::running_pinned_server::RunningPinnedServer::start)
///   builds its asserting `Self` only after the `/proc` proof succeeds; an earlier failure reaps the
///   child and removes the fresh data directory itself, folding every cleanup error into the
///   returned error.
/// - [`StagedModuleWasm::load`](crate::provision::staged_module_wasm::StagedModuleWasm::load)
///   creates its `NamedTempFile` locally, so a failure after creation drops it before any
///   `StagedModuleWasm` exists and `tempfile`'s own `Drop` removes it.
///
/// So a server-start failure can honestly record [`Self::Released`] for the staged module it *did*
/// own without claiming the server process is gone — that fact lives in the provisioning
/// diagnostic, and both fields must be read together. Publication is the first boundary at which a
/// failure leaves the driver owning both a started server and a staged module.
#[derive(Debug, Clone, Serialize)]
pub(crate) enum ResourceDisposition {
    /// No resource this driver owned existed to release.
    NotAcquired,
    /// Every resource this driver owned was released successfully.
    Released,
    /// Release was attempted and failed. The diagnostic names what may remain outstanding — the
    /// aggregating teardown never claims a process is gone when its reap could not prove it.
    ReleaseFailed { diagnostic: DiagnosticArtifact },
}

impl ResourceDisposition {
    /// The failing release's diagnostic, when this attempt's release did not succeed.
    ///
    /// The driver reads this off the record it just appended to decide whether to mark every
    /// remaining frozen slot `NotRun(PriorAttemptReleaseFailed)`, so the decision and the durable
    /// record cannot disagree about whether a release failed.
    pub(crate) fn failure_diagnostic(&self) -> Option<&DiagnosticArtifact> {
        match self {
            Self::NotAcquired | Self::Released => None,
            Self::ReleaseFailed { diagnostic } => Some(diagnostic),
        }
    }

    /// Fail loud unless this disposition agrees with how deep acquisition actually got.
    ///
    /// Acquisition depth and disposition are two statements about the same run and can contradict
    /// each other, so the record model checks them together rather than storing both and trusting
    /// the caller. Both directions are wrong and both are rejected: a record claiming a release
    /// where nothing was ever acquired invents a cleanup that never ran, and one claiming
    /// `NotAcquired` after staging a module or publishing an instance silently drops the fact that a
    /// tempfile, a server process, or a data directory was this driver's to release.
    ///
    /// Every post-publication shape passes `true`, because reaching publication means the staged
    /// module and the started server were both owned.
    pub(crate) fn ensure_matches_acquisition(&self, acquired_releasable: bool) -> Result<()> {
        match (acquired_releasable, self) {
            (false, Self::NotAcquired) | (true, Self::Released | Self::ReleaseFailed { .. }) => {
                Ok(())
            }
            (false, Self::Released | Self::ReleaseFailed { .. }) => bail!(
                "this attempt owned no releasable resource, so its disposition cannot claim a \
                 release was attempted"
            ),
            (true, Self::NotAcquired) => bail!(
                "this attempt owned a releasable resource, so its disposition cannot claim nothing \
                 was acquired"
            ),
        }
    }
}
