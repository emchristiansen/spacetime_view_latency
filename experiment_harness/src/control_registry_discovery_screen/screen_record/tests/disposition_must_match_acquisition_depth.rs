//! Acquisition depth and resource disposition cannot contradict each other.

use crate::control_registry_discovery_screen::attempted_outcome::AttemptedOutcome;
use crate::control_registry_discovery_screen::cold_apply_evidence::ColdApplyEvidence;
use crate::control_registry_discovery_screen::failure_kind::FailureKind;
use crate::control_registry_discovery_screen::host_observations::HostObservations;
use crate::control_registry_discovery_screen::provision_depth::ProvisionDepth;
use crate::control_registry_discovery_screen::resource_disposition::ResourceDisposition;
use crate::control_registry_discovery_screen::screen_record::ScreenRecord;
use crate::entity_owner_pilot::attempt_provenance::AttemptProvenance;

use super::fixture::{
    attempt_key, dispositions, environment_sample, failure_of, matched_composition,
    partial_provisions, pinned, seed, RETAINED_NANOS,
};

/// Coverage: every real `PartialProvision` variant maps to its expected depth, and the real
/// `not_provisioned` constructor accepts exactly the frozen valid depth-by-disposition set.
///
/// The frozen rule, transcribed rather than derived: `NothingResolved` and `DistributionResolved`
/// own nothing releasable and so permit only `NotAcquired`; `ModuleStaged` and `ServerStarted` hold
/// the staged tempfile and must report `Released` or `ReleaseFailed`; every post-publication shape
/// held both a staged module and a started server and so likewise rejects `NotAcquired`.
///
/// The depth mapping is checked on the **actual variants** rather than only on `ProvisionDepth`,
/// because `PartialProvision::depth` is the coupling that carries the rule: a regression sending
/// `ServerStarted` to `NothingResolved` would satisfy every property of the depth enum in isolation
/// while admitting `NotAcquired` for an attempt that left a server running.
///
/// Both directions matter. A record claiming a release where nothing was acquired invents a cleanup
/// that never ran; one claiming `NotAcquired` after publication silently drops the fact that a
/// server process and a data directory were this driver's to release — exactly the claim a reader
/// would use to decide no process leaked.
#[test]
fn disposition_must_match_acquisition_depth() {
    let key = attempt_key();
    let pinned = pinned();
    let seed = seed();

    for (partial_provision, expected_depth) in partial_provisions() {
        assert_eq!(
            partial_provision.depth(),
            expected_depth,
            "this provisioning prefix must report {expected_depth:?}"
        );

        let expected_releasable = matches!(
            expected_depth,
            ProvisionDepth::ModuleStaged | ProvisionDepth::ServerStarted
        );
        assert_eq!(
            expected_depth.acquired_releasable(),
            expected_releasable,
            "{expected_depth:?} must agree with the frozen rule about owning a releasable resource"
        );

        for disposition in dispositions() {
            let permitted = match disposition {
                ResourceDisposition::NotAcquired => !expected_releasable,
                ResourceDisposition::Released | ResourceDisposition::ReleaseFailed { .. } => {
                    expected_releasable
                }
            };
            let built = ScreenRecord::not_provisioned(
                key,
                &pinned,
                seed,
                None,
                partial_provision.clone(),
                failure_of(FailureKind::Provision),
                disposition.clone(),
            );
            assert_eq!(
                built.is_ok(),
                permitted,
                "{expected_depth:?} with {disposition:?} must {} the constructor",
                if permitted { "pass" } else { "be refused by" },
            );
        }
    }

    // Every post-publication shape rejects `NotAcquired`: reaching publication means the staged
    // module and the started server were both owned.
    let not_measured = ScreenRecord::not_measured(
        key,
        &pinned,
        seed,
        None,
        AttemptProvenance::fixture(),
        failure_of(FailureKind::Connect),
        ResourceDisposition::NotAcquired,
    );
    assert!(
        not_measured.is_err(),
        "a published instance was acquired, so NotMeasured cannot claim NotAcquired"
    );

    let unbracketed = ScreenRecord::unbracketed(
        key,
        &pinned,
        seed,
        None,
        AttemptProvenance::fixture(),
        environment_sample(0),
        failure_of(FailureKind::HostObservationAfter),
        ResourceDisposition::NotAcquired,
    );
    assert!(
        unbracketed.is_err(),
        "a published instance was acquired, so Unbracketed cannot claim NotAcquired"
    );

    let evidence = ColdApplyEvidence::sealed(RETAINED_NANOS, matched_composition())
        .expect("a positive duration with a matched composition seals");
    let attempted = ScreenRecord::attempted(
        key,
        &pinned,
        seed,
        None,
        AttemptProvenance::fixture(),
        HostObservations::bracketing(environment_sample(0), environment_sample(1)),
        AttemptedOutcome::Complete { evidence },
        ResourceDisposition::NotAcquired,
    );
    assert!(
        attempted.is_err(),
        "a published instance was acquired, so Attempted cannot claim NotAcquired"
    );
}
