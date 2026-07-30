//! What became of the resources this driver owned when an attempt ended.

use anyhow::{bail, Result};
use serde::Serialize;

use crate::indexed_sender_view_calibration_pilot::diagnostic_artifact::DiagnosticArtifact;

/// The truthful disposition of every resource **this driver owned** at the point an attempt ended.
///
/// Three variants rather than two, because "released" is a false claim where nothing releasable ever
/// existed: resolving the pinned distribution and staging the module WASM both fail before the
/// driver owns anything.
///
/// **Scope, which a reader must not widen.** This describes only resources returned to and owned by
/// the driver. Two provisioning constructors clean up their own partial state before returning
/// `Err`, and their cleanup failures stay inside the provisioning diagnostic rather than appearing
/// here. Publication is the first boundary at which a failure leaves the driver owning both a
/// started server and a staged module.
#[derive(Debug, Clone, Serialize)]
pub(crate) enum ResourceDisposition {
    /// No resource this driver owned existed to release.
    NotAcquired,
    /// Every resource this driver owned was released successfully.
    Released,
    /// Release was attempted and failed. The diagnostic names what may remain outstanding.
    ReleaseFailed { diagnostic: DiagnosticArtifact },
}

impl ResourceDisposition {
    /// The failing release's diagnostic, when this attempt's release did not succeed.
    ///
    /// The driver reads this off the record it just appended to decide whether to mark the remaining
    /// frozen slot `NotRun(PriorAttemptReleaseFailed)`, so the decision and the durable record cannot
    /// disagree about whether a release failed.
    pub(crate) fn failure_diagnostic(&self) -> Option<&DiagnosticArtifact> {
        match self {
            Self::NotAcquired | Self::Released => None,
            Self::ReleaseFailed { diagnostic } => Some(diagnostic),
        }
    }

    /// Fail loud unless this disposition agrees with how deep acquisition actually got.
    ///
    /// Both directions are wrong and both are rejected: a record claiming a release where nothing was
    /// ever acquired invents a cleanup that never ran, and one claiming `NotAcquired` after staging a
    /// module or publishing an instance silently drops the fact that a tempfile, a server process, or
    /// a data directory was this driver's to release.
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
