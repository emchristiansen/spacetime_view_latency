//! Each failure kind is accepted by exactly one record constructor and rejected by the other three.

use crate::control_registry_discovery_screen::attempt_stage::AttemptStage;
use crate::control_registry_discovery_screen::attempted_outcome::AttemptedOutcome;
use crate::control_registry_discovery_screen::host_observations::HostObservations;
use crate::control_registry_discovery_screen::partial_provision::PartialProvision;
use crate::control_registry_discovery_screen::resource_disposition::ResourceDisposition;
use crate::control_registry_discovery_screen::screen_record::ScreenRecord;
use crate::entity_owner_pilot::attempt_provenance::AttemptProvenance;

use super::fixture::{
    assert_frozen_tables_are_total, attempt_key, environment_sample, failure_of, pinned, seed,
    FROZEN_STAGES,
};

/// Coverage: the kind-to-shape mapping is total *and* injective at the API a caller actually uses.
///
/// Driven through the four failure-bearing constructors themselves rather than through
/// [`AttemptStage::ensure_admits`], because the property that matters is that each constructor
/// enforces its own shape. A constructor that silently stopped validating would still satisfy a test
/// aimed only at the validator, and would then happily record a `Semantics` mismatch as a slot that
/// never provisioned — an attempt that read four live caches, filed as one that never started a
/// server.
///
/// The expected constructor comes from [`FROZEN_STAGES`], transcribed from the spec, so this pins
/// the mapping rather than reading it back out of `kind.stage()`.
///
/// Ten kinds times four constructors, with exactly one acceptance per kind.
#[test]
fn every_failure_kind_maps_to_one_record_shape() {
    assert_frozen_tables_are_total();

    let key = attempt_key();
    let pinned = pinned();
    let seed = seed();

    for (kind, expected_stage) in FROZEN_STAGES {
        // Each call rebuilds its own failure: `AttemptFailure` is not `Copy`, and a constructor that
        // consumes one must not borrow a value another already took.
        let outcomes = [
            (
                "not_provisioned",
                ScreenRecord::not_provisioned(
                    key,
                    &pinned,
                    seed,
                    None,
                    PartialProvision::NothingResolved,
                    failure_of(kind),
                    ResourceDisposition::NotAcquired,
                )
                .is_ok(),
            ),
            (
                "not_measured",
                ScreenRecord::not_measured(
                    key,
                    &pinned,
                    seed,
                    None,
                    AttemptProvenance::fixture(),
                    failure_of(kind),
                    ResourceDisposition::Released,
                )
                .is_ok(),
            ),
            (
                "unbracketed",
                ScreenRecord::unbracketed(
                    key,
                    &pinned,
                    seed,
                    None,
                    AttemptProvenance::fixture(),
                    environment_sample(0),
                    failure_of(kind),
                    ResourceDisposition::Released,
                )
                .is_ok(),
            ),
            (
                "attempted",
                ScreenRecord::attempted(
                    key,
                    &pinned,
                    seed,
                    None,
                    AttemptProvenance::fixture(),
                    HostObservations::bracketing(environment_sample(0), environment_sample(1)),
                    AttemptedOutcome::Failed {
                        failure: failure_of(kind),
                    },
                    ResourceDisposition::Released,
                )
                .is_ok(),
            ),
        ];

        let accepted: Vec<&str> = outcomes
            .iter()
            .filter(|(_, ok)| *ok)
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(
            accepted.len(),
            1,
            "{kind:?} must be accepted by exactly one record constructor, but {accepted:?} accepted \
             it"
        );

        let expected_constructor = match expected_stage {
            AttemptStage::Unprovisioned => "not_provisioned",
            AttemptStage::Unmeasured => "not_measured",
            AttemptStage::Unbracketed => "unbracketed",
            AttemptStage::Bracketed => "attempted",
        };
        assert_eq!(
            accepted[0], expected_constructor,
            "{kind:?} is a {expected_stage:?} failure and must be recorded by {expected_constructor}"
        );
    }
}
