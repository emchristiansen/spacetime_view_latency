//! Why a build's harness-commit evidence is not authoritative.

/// The exhaustive typed reason a build is not authoritative. `HARNESS_BUILD_TREE_STATE` (`clean` or
/// `dirty`) is the only axis
/// [`BuildProvenance::from_build_env`](super::build_provenance::BuildProvenance::from_build_env)
/// dispatches on, so an uncommitted-changes build is presently the sole possible reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DevelopmentBuildReason {
    /// The git checkout that built this binary had uncommitted changes (`HARNESS_BUILD_TREE_STATE
    /// = "dirty"`).
    DirtyTree,
}
