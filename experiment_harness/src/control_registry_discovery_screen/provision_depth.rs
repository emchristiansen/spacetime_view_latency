//! How deep acquisition got, independent of the facts it gathered.

use serde::Serialize;

/// The four acquisition depths, as a fieldless discriminant of
/// [`PartialProvision`](super::partial_provision::PartialProvision).
///
/// Exists so the two properties that matter about acquisition depth — that the prefix is monotone,
/// and which depths own a releasable resource — are decidable and testable *without* a live
/// distribution or server. `PartialProvision`'s own variants carry
/// [`DistributionFacts`](crate::entity_owner_pilot::attempt_provenance::DistributionFacts) and
/// [`ServerFacts`](crate::entity_owner_pilot::attempt_provenance::ServerFacts), which can only be
/// observed from real provisioning capabilities, so a rule expressed only over those variants could
/// not be proven anywhere but a live run. Splitting the discriminant out keeps the rule in a type
/// that a focused test can enumerate exhaustively.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) enum ProvisionDepth {
    /// The pinned distribution was not resolved.
    NothingResolved,
    /// The distribution resolved and version-verified.
    DistributionResolved,
    /// The module WASM was hash-verified against the committed constant and staged.
    ModuleStaged,
    /// The server started and was proven through `/proc`.
    ServerStarted,
}

impl ProvisionDepth {
    /// Every depth in acquisition order — the monotone chain itself.
    pub(crate) const ALL: [ProvisionDepth; 4] = [
        ProvisionDepth::NothingResolved,
        ProvisionDepth::DistributionResolved,
        ProvisionDepth::ModuleStaged,
        ProvisionDepth::ServerStarted,
    ];

    /// How many provisioning facts this depth carries, `0..=3`.
    pub(crate) fn established_facts(self) -> usize {
        match self {
            Self::NothingResolved => 0,
            Self::DistributionResolved => 1,
            Self::ModuleStaged => 2,
            Self::ServerStarted => 3,
        }
    }

    /// Whether the verified distribution was established.
    pub(crate) fn has_distribution(self) -> bool {
        self >= Self::DistributionResolved
    }

    /// Whether the pinned-hash-verified module WASM was staged.
    pub(crate) fn has_staged_module(self) -> bool {
        self >= Self::ModuleStaged
    }

    /// Whether the server started and was proven.
    pub(crate) fn has_server(self) -> bool {
        self >= Self::ServerStarted
    }

    /// Whether a failure at this depth leaves the driver owning something it must release.
    ///
    /// Exactly [`Self::has_staged_module`], and that equivalence is the whole rule. Resolving the
    /// distribution only reads paths out of the Nix store, so nothing is owned; the staged tempfile
    /// is the first resource the driver holds. A server-start failure adds nothing further to
    /// release, because
    /// [`RunningPinnedServer::start`](crate::provision::running_pinned_server::RunningPinnedServer::start)
    /// reaps its own child and data directory before returning `Err` — the staged module is still
    /// the driver's to clean up, which is why `ModuleStaged` and `ServerStarted` share this answer.
    pub(crate) fn acquired_releasable(self) -> bool {
        self.has_staged_module()
    }
}
