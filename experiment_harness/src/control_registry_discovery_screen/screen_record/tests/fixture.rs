//! Shared constructions for the record state-machine tests.

use anyhow::anyhow;

use crate::control_registry_discovery_screen::attempt_failure::AttemptFailure;
use crate::control_registry_discovery_screen::attempt_inventory::AttemptInventory;
use crate::control_registry_discovery_screen::attempt_key::AttemptKey;
use crate::control_registry_discovery_screen::attempt_stage::AttemptStage;
use crate::control_registry_discovery_screen::diagnostic_artifact::DiagnosticArtifact;
use crate::control_registry_discovery_screen::failure_kind::FailureKind;
use crate::control_registry_discovery_screen::four_way_composition::FourWayComposition;
use crate::control_registry_discovery_screen::four_way_expectation::FourWayExpectation;
use crate::control_registry_discovery_screen::four_way_observation::FourWayObservation;
use crate::control_registry_discovery_screen::partial_evidence::PartialEvidence;
use crate::control_registry_discovery_screen::partial_provision::PartialProvision;
use crate::control_registry_discovery_screen::pinned_artifact_identity::PinnedArtifactIdentity;
use crate::control_registry_discovery_screen::provision_depth::ProvisionDepth;
use crate::control_registry_discovery_screen::rejected_apply_nanos::RejectedApplyNanos;
use crate::control_registry_discovery_screen::resource_disposition::ResourceDisposition;
use crate::control_registry_discovery_screen::screen_params::REGISTRY_CONTROLS;
use crate::control_registry_discovery_screen::screen_rung::ScreenRung;
use crate::entity_owner_pilot::attempt_provenance::{DistributionFacts, ServerFacts};
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;
use crate::view_read_set_campaign::environment_sample::EnvironmentSample;

/// The frozen kind-to-stage mapping, **transcribed from the spec** rather than read back from
/// [`FailureKind::stage`].
///
/// An independent oracle is the whole point: a test that computed its expectation from the function
/// under test would pass for any mapping at all, including one that filed a semantic mismatch as an
/// unprovisioned slot. Verified total against [`FailureKind::ALL`] by
/// [`assert_frozen_tables_are_total`], so a kind cannot be silently dropped from the oracle either.
pub(super) const FROZEN_STAGES: [(FailureKind, AttemptStage); 9] = [
    (FailureKind::Provision, AttemptStage::Unprovisioned),
    (FailureKind::Connect, AttemptStage::Unmeasured),
    (FailureKind::Reducer, AttemptStage::Unmeasured),
    (FailureKind::HostObservationBefore, AttemptStage::Unmeasured),
    (FailureKind::HostObservationAfter, AttemptStage::Unbracketed),
    (FailureKind::TimedSubscription, AttemptStage::Bracketed),
    (FailureKind::ValidationSubscription, AttemptStage::Bracketed),
    (FailureKind::Sample, AttemptStage::Bracketed),
    (FailureKind::Semantics, AttemptStage::Bracketed),
];

/// Which kinds are reachable only past a completed timed apply, transcribed from the spec for the
/// same reason as [`FROZEN_STAGES`].
pub(super) const FROZEN_REQUIRES_SAMPLE: [(FailureKind, bool); 9] = [
    (FailureKind::Provision, false),
    (FailureKind::Connect, false),
    (FailureKind::Reducer, false),
    (FailureKind::HostObservationBefore, false),
    (FailureKind::TimedSubscription, false),
    (FailureKind::HostObservationAfter, true),
    (FailureKind::ValidationSubscription, true),
    (FailureKind::Sample, true),
    (FailureKind::Semantics, true),
];

/// Fail loud unless both frozen tables name every kind exactly once.
///
/// Without this a kind added to `FailureKind::ALL` but forgotten in an oracle would simply go
/// unchecked, which is the quiet way an independent table stops being independent.
pub(super) fn assert_frozen_tables_are_total() {
    for kind in FailureKind::ALL {
        assert_eq!(
            FROZEN_STAGES.iter().filter(|(k, _)| *k == kind).count(),
            1,
            "{kind:?} must appear exactly once in the frozen stage table"
        );
        assert_eq!(
            FROZEN_REQUIRES_SAMPLE
                .iter()
                .filter(|(k, _)| *k == kind)
                .count(),
            1,
            "{kind:?} must appear exactly once in the frozen sample-requirement table"
        );
    }
}

/// The seed the inventory tests already use, so a key here is one the screen would really execute.
pub(super) const SEED: u64 = 7;

/// A distinctive nonzero duration, so a retained sample is recognisable in serialized output.
pub(super) const RETAINED_NANOS: u128 = 4_242_424_242;

/// The seed as the driver takes it.
pub(super) fn seed() -> ScheduleSeed {
    ScheduleSeed::new(SEED)
}

/// A real frozen attempt identity — the first the screen would execute.
pub(super) fn attempt_key() -> AttemptKey {
    let frozen = AttemptInventory::frozen(seed()).expect("the seed freezes");
    *frozen
        .attempts()
        .first()
        .expect("the inventory is non-empty")
}

/// The frozen pinned artifact identity.
pub(super) fn pinned() -> PinnedArtifactIdentity {
    PinnedArtifactIdentity::frozen().expect("the pinned constants are well formed")
}

/// A diagnostic built from a real error, as the driver's would be.
pub(super) fn diagnostic() -> DiagnosticArtifact {
    DiagnosticArtifact::of_error(&anyhow!("a representative failure"))
}

/// One host reading, distinguished only by its monotonic offset so a `before`/`after` pair is
/// ordered. The four quantities are plausible idle-host values; nothing here reads them.
pub(super) fn environment_sample(offset_nanos: u64) -> EnvironmentSample {
    EnvironmentSample::observed(offset_nanos, 100, 8 << 30, 0, 0)
}

/// All four real [`PartialProvision`] variants, paired with the depth each must map to.
///
/// The expected depths are transcribed, not read back from `PartialProvision::depth`, so a
/// regression mapping `ServerStarted` onto `NothingResolved` — which would then wrongly admit
/// `NotAcquired` and hide an unreleased server — fails here.
pub(super) fn partial_provisions() -> [(PartialProvision, ProvisionDepth); 4] {
    let staged_wasm_sha256 = WasmSha256::new(MODULE_WASM_SHA256);
    [
        (
            PartialProvision::NothingResolved,
            ProvisionDepth::NothingResolved,
        ),
        (
            PartialProvision::DistributionResolved {
                distribution: DistributionFacts::fixture(),
            },
            ProvisionDepth::DistributionResolved,
        ),
        (
            PartialProvision::ModuleStaged {
                distribution: DistributionFacts::fixture(),
                staged_wasm_sha256,
            },
            ProvisionDepth::ModuleStaged,
        ),
        (
            PartialProvision::ServerStarted {
                distribution: DistributionFacts::fixture(),
                staged_wasm_sha256,
                server: ServerFacts::fixture(),
            },
            ProvisionDepth::ServerStarted,
        ),
    ]
}

/// The three dispositions, for exhaustive cross-checks against acquisition depth.
pub(super) fn dispositions() -> [ResourceDisposition; 3] {
    [
        ResourceDisposition::NotAcquired,
        ResourceDisposition::Released,
        ResourceDisposition::ReleaseFailed {
            diagnostic: diagnostic(),
        },
    ]
}

/// A four-way composition in which every cache matched its frozen expectation.
pub(super) fn matched_composition() -> FourWayComposition {
    let rung = ScreenRung::Low;
    FourWayComposition::checked(
        FourWayExpectation::at(rung),
        FourWayObservation::read(
            REGISTRY_CONTROLS,
            REGISTRY_CONTROLS,
            REGISTRY_CONTROLS,
            rung.history_rows(),
        ),
    )
}

/// A four-way composition in which the comparator's Control came up one row short.
pub(super) fn mismatched_composition() -> FourWayComposition {
    let rung = ScreenRung::Low;
    FourWayComposition::checked(
        FourWayExpectation::at(rung),
        FourWayObservation::read(
            REGISTRY_CONTROLS,
            REGISTRY_CONTROLS,
            REGISTRY_CONTROLS,
            rung.history_rows() - 1,
        ),
    )
}

/// The partial evidence a failure of `kind` must carry, derived from the kind's own rules.
///
/// Built from the predicates rather than a hand-written table, so the fixture cannot silently
/// disagree with the invariants under test about which shape a kind requires.
pub(super) fn partial_evidence_for(kind: FailureKind) -> PartialEvidence {
    match (kind.requires_sample(), kind.can_observe_composition()) {
        (false, _) => PartialEvidence::NothingObserved,
        (true, false) => PartialEvidence::RejectedSample {
            apply_nanos: RejectedApplyNanos::of(RETAINED_NANOS),
        },
        (true, true) => PartialEvidence::RejectedSampleAndComposition {
            apply_nanos: RejectedApplyNanos::of(RETAINED_NANOS),
            // A semantic failure *is* a mismatch and is rejected without one; every other
            // composition-observing kind saw the caches agree.
            composition: match kind.requires_observed_composition() {
                true => mismatched_composition(),
                false => matched_composition(),
            },
        },
    }
}

/// A valid failure of `kind`, with the partial evidence that kind requires.
pub(super) fn failure_of(kind: FailureKind) -> AttemptFailure {
    AttemptFailure::observed(kind, partial_evidence_for(kind), diagnostic())
        .expect("the fixture builds the partial evidence each kind requires")
}
