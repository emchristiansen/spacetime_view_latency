//! One attempt's durable ledger line.

use serde::Serialize;

use crate::control_registry_discovery_screen::attempt_key::AttemptKey;
use crate::control_registry_discovery_screen::attempted_outcome::AttemptedOutcome;
use crate::control_registry_discovery_screen::host_observations::HostObservations;
use crate::control_registry_discovery_screen::method_facts::MethodFacts;
use crate::control_registry_discovery_screen::not_run_reason::NotRunReason;
use crate::control_registry_discovery_screen::pinned_artifact_identity::PinnedArtifactIdentity;
use crate::control_registry_discovery_screen::screen_composition::ScreenComposition;
use crate::control_registry_discovery_screen::supersession::Supersession;
use crate::entity_owner_pilot::attempt_provenance::AttemptProvenance;

/// Exactly one terminal record per predeclared attempt, in one of the two shapes an attempt can
/// actually have.
///
/// The variants exist to make the missing-observation shape unrepresentable rather than merely
/// discouraged. A slot refused at the gate provisioned nothing and observed nothing, so it carries
/// neither provenance nor host observations — not `None` for them, but no field at all. An attempt
/// that ran carries both, non-optionally, so a `Complete` or `Failed` record with absent host
/// observations or incomplete provision provenance cannot be constructed.
///
/// Identity, composition, frozen method facts, pinned artifact identity, and seed appear on *both*
/// variants: a reader holding only the ledger must be able to see what a slot was going to measure,
/// under what method, against which pinned artifacts, even when it never ran.
///
/// [`AttemptProvenance`] is reused verbatim from the Pilot. It is a record of *observed* runtime
/// facts — distribution paths and versions, server pid, listen address and data directory, module
/// hash and database identity — with no candidate vocabulary in it, so reusing it reinterprets no
/// prior evidence and minting a parallel copy would only create a second thing to drift.
#[derive(Debug, Clone, Serialize)]
pub(crate) enum ScreenRecord {
    /// The gate never admitted this slot. It measured nothing and consumed nothing.
    NotRun {
        key: AttemptKey,
        composition: ScreenComposition,
        method: MethodFacts,
        pinned: PinnedArtifactIdentity,
        supersession: Supersession,
        schedule_seed: u64,
        reason: NotRunReason,
    },
    /// The slot provisioned and ran, and ended either complete or failed.
    Attempted {
        key: AttemptKey,
        composition: ScreenComposition,
        method: MethodFacts,
        pinned: PinnedArtifactIdentity,
        supersession: Supersession,
        schedule_seed: u64,
        provenance: AttemptProvenance,
        host: HostObservations,
        outcome: AttemptedOutcome,
    },
}
