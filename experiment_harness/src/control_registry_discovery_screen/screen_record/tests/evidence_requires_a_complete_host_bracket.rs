//! Sealed evidence exists only with full provenance and both host observations.

use crate::control_registry_discovery_screen::attempted_outcome::AttemptedOutcome;
use crate::control_registry_discovery_screen::cold_apply_evidence::ColdApplyEvidence;
use crate::control_registry_discovery_screen::failure_kind::FailureKind;
use crate::control_registry_discovery_screen::host_observations::HostObservations;
use crate::control_registry_discovery_screen::resource_disposition::ResourceDisposition;
use crate::control_registry_discovery_screen::screen_record::ScreenRecord;
use crate::entity_owner_pilot::attempt_provenance::AttemptProvenance;

use super::fixture::{
    attempt_key, environment_sample, failure_of, matched_composition, mismatched_composition,
    pinned, seed, RETAINED_NANOS,
};

/// Coverage: the sealing gates, and that a `Complete` outcome can only be recorded in the one shape
/// that carries a complete bracket.
///
/// **The structural half is compile-time and deliberately so.** [`AttemptedOutcome`] — the only type
/// that can hold [`ColdApplyEvidence`] — appears in exactly one `ScreenRecord` variant, and that
/// variant's `provenance` and `host` fields are non-optional. There is therefore no way to write a
/// complete record without both, and no runtime assertion could add anything: the alternative simply
/// does not typecheck. What this test pins is the part that *can* go wrong at run time — the
/// duration and composition gates, and that a failure which never closed its bracket is refused by
/// the shape that would imply one.
#[test]
fn evidence_requires_a_complete_host_bracket() {
    // A cold apply seals only from a strictly positive duration and an exactly matched composition.
    ColdApplyEvidence::sealed(RETAINED_NANOS, matched_composition())
        .expect("a positive duration with a matched composition seals");
    assert!(
        ColdApplyEvidence::sealed(0, matched_composition()).is_err(),
        "a nonpositive statistic invalidates its cell and must not seal"
    );
    assert!(
        ColdApplyEvidence::sealed(RETAINED_NANOS, mismatched_composition()).is_err(),
        "a duration measured against an unestablished composition is not cheap evidence, it is none"
    );

    let key = attempt_key();
    let pinned = pinned();
    let seed = seed();

    // The happy path: full provenance plus both observations records a complete measurement.
    let evidence = ColdApplyEvidence::sealed(RETAINED_NANOS, matched_composition())
        .expect("a positive duration with a matched composition seals");
    ScreenRecord::attempted(
        key,
        &pinned,
        seed,
        None,
        AttemptProvenance::fixture(),
        HostObservations::bracketing(environment_sample(0), environment_sample(1)),
        AttemptedOutcome::Complete { evidence },
        ResourceDisposition::Released,
    )
    .expect("a complete bracket over full provenance records complete evidence");

    // A failure whose bracket never closed cannot be dressed as a measured outcome by handing the
    // constructor observations it never took.
    let dressed_up = ScreenRecord::attempted(
        key,
        &pinned,
        seed,
        None,
        AttemptProvenance::fixture(),
        HostObservations::bracketing(environment_sample(0), environment_sample(1)),
        AttemptedOutcome::Failed {
            failure: failure_of(FailureKind::HostObservationAfter),
        },
        ResourceDisposition::Released,
    );
    assert!(
        dressed_up.is_err(),
        "a HostObservationAfter failure has no closed bracket and cannot inhabit the Attempted shape"
    );

    // It belongs in the shape that retains only the surviving `before` observation.
    ScreenRecord::unbracketed(
        key,
        &pinned,
        seed,
        None,
        AttemptProvenance::fixture(),
        environment_sample(0),
        failure_of(FailureKind::HostObservationAfter),
        ResourceDisposition::Released,
    )
    .expect("an unclosed bracket records the before observation it did take");
}
