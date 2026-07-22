//! A live authorization witness that a harness-commit fact is authoritative.

use crate::manifest::build_provenance::BuildProvenance;
use crate::manifest::development_build_reason::DevelopmentBuildReason;
use crate::manifest::embedded_harness_commit::EmbeddedHarnessCommit;

/// A live authorization witness that this binary's harness-commit fact is authoritative: observed from a
/// clean git checkout at build time, not merely an embedded string. [`Self::new`] is private to this
/// module and called only from [`BuildProvenance::from_build_env`] below — the sole place a
/// `HarnessCommit` is ever constructed — so nothing can forge one from a deserialized or frozen fact
/// (spec: "a distinct live authorization witness whose private constructor is reachable only through
/// `BuildProvenance::Authoritative`"). Consumed exactly once as authorization by the eventual live
/// `UnvalidatedRunManifest::seal`, which persists only its canonical [`EmbeddedHarnessCommit`] projection;
/// frozen ingest parses that same serialized fact directly into `EmbeddedHarnessCommit` and never mints a
/// `HarnessCommit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HarnessCommit(EmbeddedHarnessCommit);

impl HarnessCommit {
    fn new(commit: EmbeddedHarnessCommit) -> Self {
        Self(commit)
    }

    /// The canonical commit value, for persistence as [`EmbeddedHarnessCommit`] — the live witness itself
    /// never escapes as trusted evidence, only this frozen projection does.
    pub(crate) fn embedded(&self) -> EmbeddedHarnessCommit {
        self.0
    }
}

impl BuildProvenance {
    /// Parse `experiment_harness/build.rs`'s mandatory `cargo:rustc-env` values, embedded at compile time
    /// via [`env!`], into the closed dispatch. Fails loudly on a malformed embedded commit or a tree state
    /// other than the two values the build script ever emits — never a silent default.
    ///
    /// Colocated with [`HarnessCommit`] (rather than with the `BuildProvenance` enum declaration) so this
    /// is the only code in the crate with module access to [`HarnessCommit::new`].
    pub(crate) fn from_build_env() -> Self {
        let commit = EmbeddedHarnessCommit::parse(env!("HARNESS_BUILD_COMMIT"))
            .expect("HARNESS_BUILD_COMMIT is embedded by build.rs as a canonical 40-hex commit");
        match env!("HARNESS_BUILD_TREE_STATE") {
            "clean" => Self::Authoritative {
                commit: HarnessCommit::new(commit),
            },
            "dirty" => Self::Development {
                commit,
                reason: DevelopmentBuildReason::DirtyTree,
            },
            other => panic!(
                "HARNESS_BUILD_TREE_STATE embedded by build.rs must be \"clean\" or \"dirty\", found \
                 {other:?}"
            ),
        }
    }

    /// The embedded commit fact carried by either variant, for the Phase-2 live checkout comparison both
    /// provenance variants require (spec: "compares that Git worktree's current commit with its
    /// `EmbeddedHarnessCommit`, which both provenance variants carry").
    pub(crate) fn embedded_commit(&self) -> EmbeddedHarnessCommit {
        match self {
            Self::Authoritative { commit } => commit.embedded(),
            Self::Development { commit, .. } => *commit,
        }
    }
}
