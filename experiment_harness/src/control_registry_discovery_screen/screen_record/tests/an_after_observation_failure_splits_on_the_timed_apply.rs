//! A failed `after` observation is two kinds, chosen by whether the timed apply had completed.

use anyhow::anyhow;

use crate::control_registry_discovery_screen::attempt_failure::AttemptFailure;
use crate::control_registry_discovery_screen::attempt_stage::AttemptStage;
use crate::control_registry_discovery_screen::attempted_outcome::AttemptedOutcome;
use crate::control_registry_discovery_screen::diagnostic_artifact::DiagnosticArtifact;
use crate::control_registry_discovery_screen::failure_kind::FailureKind;
use crate::control_registry_discovery_screen::host_observations::HostObservations;
use crate::control_registry_discovery_screen::partial_evidence::PartialEvidence;
use crate::control_registry_discovery_screen::rejected_apply_nanos::RejectedApplyNanos;
use crate::control_registry_discovery_screen::resource_disposition::ResourceDisposition;
use crate::control_registry_discovery_screen::screen_record::ScreenRecord;
use crate::entity_owner_pilot::attempt_provenance::AttemptProvenance;
use crate::provision::teardown::into_error;

use super::fixture::{
    attempt_key, diagnostic, environment_sample, pinned, seed, FROZEN_REQUIRES_SAMPLE,
    RETAINED_NANOS,
};

/// The timed subscription's own failure text, distinctive enough to find in serialized output.
const TIMED_DIAGNOSTIC: &str = "the timed subscription was never applied";

/// The `after` observation's failure text, distinct from the above so neither can stand in for the
/// other.
const AFTER_DIAGNOSTIC: &str = "/proc/pressure/memory could not be read";

/// Coverage: the totality hole the two-kind split exists to close, and the retention rule on each
/// side of it.
///
/// **What was previously unrecordable.** A timed subscription that failed to apply, followed by an
/// `after` observation that also failed, had no valid record shape at all. `TimedSubscription` is
/// `Bracketed`, so its shape demands the complete host pair the failed observation never produced;
/// `HostObservationAfter` demands the raw sample the failed apply never produced. The spec requires
/// the `after` observation be attempted *even when the timed subscription fails*, so this is a
/// reachable state, and "every fallible operation maps to exactly one terminal record" was false
/// until it had a kind of its own.
///
/// The expectations here are written literally rather than read back from `requires_sample`, then
/// cross-checked against the frozen table: a test that asked the predicate which kind carries a
/// sample would pass just as happily with the two kinds swapped, which is the exact regression this
/// pins against.
#[test]
fn an_after_observation_failure_splits_on_the_timed_apply() {
    let key = attempt_key();
    let pinned = pinned();
    let seed = seed();

    // The frozen table and the literal expectations below must name the same split.
    for (kind, expected) in [
        (FailureKind::HostObservationAfterTimedFailure, false),
        (FailureKind::HostObservationAfter, true),
    ] {
        assert!(
            FROZEN_REQUIRES_SAMPLE.contains(&(kind, expected)),
            "{kind:?} must carry a sample: {expected} — the frozen table disagrees"
        );
        assert_eq!(
            kind.stage(),
            AttemptStage::Unbracketed,
            "{kind:?} closed no bracket, so it belongs in the Unbracketed shape"
        );
    }

    // The apply never completed, so there is nothing to retain and nothing may be invented.
    let both_failed = AttemptFailure::observed(
        FailureKind::HostObservationAfterTimedFailure,
        PartialEvidence::NothingObserved,
        chained_diagnostic(),
    )
    .expect("a failed apply followed by a failed after observation is recordable");
    assert!(
        AttemptFailure::observed(
            FailureKind::HostObservationAfterTimedFailure,
            PartialEvidence::RejectedSample {
                apply_nanos: RejectedApplyNanos::of(RETAINED_NANOS),
            },
            diagnostic(),
        )
        .is_err(),
        "the timed apply failed, so this kind cannot carry a sample it never produced"
    );

    // Both failures survive in the one diagnostic, which is the only place either is retained: the
    // record names the kind, not the two errors behind it.
    let record = ScreenRecord::unbracketed(
        key,
        &pinned,
        seed,
        None,
        AttemptProvenance::fixture(),
        environment_sample(0),
        both_failed,
        ResourceDisposition::Released,
    )
    .expect("the surviving before observation is what this shape records");
    let rendered = serde_json::to_string(&record).expect("a record serializes");
    for expected_text in [TIMED_DIAGNOSTIC, AFTER_DIAGNOSTIC] {
        assert!(
            rendered.contains(expected_text),
            "the chained diagnostic must retain {expected_text:?} verbatim, got {rendered}"
        );
    }

    // The other side: the apply *did* complete, so its rejected nanoseconds must be retained.
    assert!(
        AttemptFailure::observed(
            FailureKind::HostObservationAfter,
            PartialEvidence::NothingObserved,
            diagnostic(),
        )
        .is_err(),
        "a completed apply's raw duration cannot be dropped because the bracket later failed"
    );
    let apply_completed = AttemptFailure::observed(
        FailureKind::HostObservationAfter,
        PartialEvidence::RejectedSample {
            apply_nanos: RejectedApplyNanos::of(RETAINED_NANOS),
        },
        diagnostic(),
    )
    .expect("a completed apply with a failed after observation is recordable");
    let record = ScreenRecord::unbracketed(
        key,
        &pinned,
        seed,
        None,
        AttemptProvenance::fixture(),
        environment_sample(0),
        apply_completed,
        ResourceDisposition::Released,
    )
    .expect("the same shape holds both after-observation kinds");
    let rendered = serde_json::to_string(&record).expect("a record serializes");
    assert!(
        rendered.contains(&RETAINED_NANOS.to_string()),
        "the rejected sample must reach the ledger, got {rendered}"
    );

    // Neither may be dressed as a measured outcome: the `Attempted` shape carries a complete host
    // pair, and both of these kinds exist precisely because that pair was never closed.
    for kind in [
        FailureKind::HostObservationAfterTimedFailure,
        FailureKind::HostObservationAfter,
    ] {
        let failure = match kind.requires_sample() {
            false => AttemptFailure::observed(kind, PartialEvidence::NothingObserved, diagnostic()),
            true => AttemptFailure::observed(
                kind,
                PartialEvidence::RejectedSample {
                    apply_nanos: RejectedApplyNanos::of(RETAINED_NANOS),
                },
                diagnostic(),
            ),
        }
        .expect("each kind builds with the partial evidence it requires");
        assert!(
            ScreenRecord::attempted(
                key,
                &pinned,
                seed,
                None,
                AttemptProvenance::fixture(),
                HostObservations::bracketing(environment_sample(0), environment_sample(1)),
                AttemptedOutcome::Failed { failure },
                ResourceDisposition::Released,
            )
            .is_err(),
            "{kind:?} closed no bracket and cannot be handed observations it never took"
        );
    }
}

/// The two failures aggregated exactly as the driver aggregates them, so the test exercises the real
/// shape rather than a hand-written approximation of it.
fn chained_diagnostic() -> DiagnosticArtifact {
    DiagnosticArtifact::of_error(&into_error(vec![
        anyhow!(TIMED_DIAGNOSTIC),
        anyhow!(AFTER_DIAGNOSTIC),
    ]))
}
