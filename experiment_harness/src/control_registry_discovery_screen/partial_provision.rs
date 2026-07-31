//! How far acquisition got before an attempt failed to provision.

use serde::Serialize;

use crate::control_registry_discovery_screen::provision_depth::ProvisionDepth;
use crate::entity_owner_pilot::attempt_provenance::{DistributionFacts, ServerFacts};
use crate::manifest::wasm_sha256::WasmSha256;

/// The greatest verified prefix of provisioning facts an attempt established before failing.
///
/// **Monotone by construction.** The four variants are a chain, each adding exactly one fact to the
/// one before it and never dropping one: nothing, then the verified distribution, then the
/// pinned-hash-verified staged module, then the proven server. A record therefore states how far
/// acquisition got without a reader having to reconstruct it from a diagnostic string.
///
/// **Retryability never authorizes discarding these.** Every provisioning failure is a prospective
/// infrastructure failure and so retryable, but a superseded failure stays visible in the ledger and
/// the spec requires failed attempts to retain provision/runtime/module facts. Recording only the
/// pinned identity would throw away the facts that distinguish "the pinned distribution is missing"
/// from "the server started and publication was rejected".
///
/// **Why the database identity never appears here.** Publication is the first point at which a
/// database exists, and publication succeeding is exactly the boundary past which the complete
/// [`AttemptProvenance`](crate::entity_owner_pilot::attempt_provenance::AttemptProvenance) is
/// available instead. The staged WASM hash *does* appear from [`Self::ModuleStaged`] onward, because
/// it is verified against the committed pinned constant before any server exists.
///
/// [`DistributionFacts`] and [`ServerFacts`] are the Pilot's own representations, reused rather than
/// re-minted so a partial record and a complete one cannot disagree about what a provisioning fact
/// is.
#[derive(Debug, Clone, Serialize)]
pub(crate) enum PartialProvision {
    /// The pinned distribution could not be resolved and version-verified. Nothing was established.
    NothingResolved,
    /// The distribution resolved; the module WASM was not staged.
    DistributionResolved { distribution: DistributionFacts },
    /// The module WASM was read, hash-verified against the committed constant, and staged; no
    /// server started.
    ModuleStaged {
        distribution: DistributionFacts,
        staged_wasm_sha256: WasmSha256,
    },
    /// The server started and was proven through `/proc`; publication did not complete.
    ServerStarted {
        distribution: DistributionFacts,
        staged_wasm_sha256: WasmSha256,
        server: ServerFacts,
    },
}

impl PartialProvision {
    /// How deep acquisition got, as the fieldless discriminant the depth rules live on.
    ///
    /// Total, and the only route to those rules, so the facts a prefix carries and the resource
    /// disposition it permits are decided by one value rather than by two matches that could drift.
    pub(crate) fn depth(&self) -> ProvisionDepth {
        match self {
            Self::NothingResolved => ProvisionDepth::NothingResolved,
            Self::DistributionResolved { .. } => ProvisionDepth::DistributionResolved,
            Self::ModuleStaged { .. } => ProvisionDepth::ModuleStaged,
            Self::ServerStarted { .. } => ProvisionDepth::ServerStarted,
        }
    }
}
