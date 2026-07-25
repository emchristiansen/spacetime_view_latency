//! Why a Pilot's harness build is not authoritative.

use serde::Serialize;

use crate::manifest::development_build_reason::DevelopmentBuildReason;

/// The ledger's projection of [`DevelopmentBuildReason`].
///
/// A separate enum only because the historical reason is not serializable and this module does not
/// modify historical types. [`Self::of`] is a total match, so a reason added there fails to compile
/// here rather than being silently dropped from recorded provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum DevelopmentBuild {
    /// The git checkout that built this binary had uncommitted changes.
    DirtyTree,
}

impl DevelopmentBuild {
    /// Project the harness's typed reason.
    pub(crate) fn of(reason: DevelopmentBuildReason) -> Self {
        match reason {
            DevelopmentBuildReason::DirtyTree => Self::DirtyTree,
        }
    }
}
