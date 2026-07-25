//! Which harness build produced a Pilot's evidence.

use serde::Serialize;

use crate::entity_owner_pilot::development_build::DevelopmentBuild;
use crate::manifest::build_provenance::BuildProvenance;
use crate::manifest::embedded_harness_commit::EmbeddedHarnessCommit;

/// The ledger's projection of the harness's [`BuildProvenance`].
///
/// Built by a total match over the harness's own closed dispatch rather than from re-read strings,
/// so the recorded provenance cannot claim a build state the harness did not prove. The commit is
/// carried in both variants — a dirty build's evidence is still attributable — but the variant is
/// what says whether it may authorize a conclusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum HarnessBuildProvenance {
    /// Built from a clean checkout at this commit.
    Authoritative { commit: EmbeddedHarnessCommit },
    /// Built from a checkout that cannot authorize live minting, with the typed reason.
    Development {
        commit: EmbeddedHarnessCommit,
        reason: DevelopmentBuild,
    },
}

impl HarnessBuildProvenance {
    /// Project the harness's compile-time build evidence.
    pub(crate) fn of(provenance: BuildProvenance) -> Self {
        match provenance {
            BuildProvenance::Authoritative { commit } => Self::Authoritative {
                commit: commit.embedded(),
            },
            BuildProvenance::Development { commit, reason } => Self::Development {
                commit,
                reason: DevelopmentBuild::of(reason),
            },
        }
    }
}
