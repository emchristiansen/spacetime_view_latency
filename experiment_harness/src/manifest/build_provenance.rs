//! The closed dispatch of one build's harness-commit evidence.

use crate::manifest::development_build_reason::DevelopmentBuildReason;
use crate::manifest::embedded_harness_commit::EmbeddedHarnessCommit;
use crate::manifest::harness_commit::HarnessCommit;

/// The closed dispatch of one build's harness-commit evidence (spec: "typed authoritative/development
/// build provenance"): either the live [`HarnessCommit`] witness from a clean build, or the same commit
/// fact demoted to [`EmbeddedHarnessCommit`] with the typed reason it cannot authorize live minting. Never
/// an optional, default, or placeholder trusted state.
///
/// Constructed only by [`Self::from_build_env`] (`impl` colocated with [`HarnessCommit`] in
/// `harness_commit.rs`, the only module that can reach its private constructor).
///
/// Phase 1 stops at this closed dispatch: live `--harness-checkout-root` comparison, `Campaign`'s
/// rejection of the `Development` variant before staging output, the per-run rechecks before each of the
/// 540 runs, and `EmbeddedHarnessCommit` manifest consumption are all deferred to Phase 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BuildProvenance {
    Authoritative { commit: HarnessCommit },
    Development {
        commit: EmbeddedHarnessCommit,
        reason: DevelopmentBuildReason,
    },
}
