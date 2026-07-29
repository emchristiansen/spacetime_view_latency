//! A failure's stage and its sampling progress are one decision, pinned to the frozen mapping.

use crate::control_registry_discovery_screen::attempt_stage::AttemptStage;
use crate::control_registry_discovery_screen::failure_kind::FailureKind;
use crate::control_registry_discovery_screen::retry_eligibility::RetryEligibility;
use crate::control_registry_discovery_screen::sampling_progress::SamplingProgress;

use super::fixture::{assert_frozen_tables_are_total, failure_of, FROZEN_STAGES};

/// Coverage: the kind-to-stage mapping itself, then the progress and retry rules that follow from it.
///
/// The expected stage comes from [`FROZEN_STAGES`], transcribed from the spec — never from
/// `kind.stage()`, which is the function under test. Deriving the expectation from it would make
/// this pass for any mapping whatsoever, including one that classified a semantic mismatch as an
/// unprovisioned slot.
///
/// Progress is then derived from the *expected* stage rather than the observed one, and compared
/// against what the failure actually recorded. `AttemptFailure::observed` derives progress from the
/// kind rather than accepting it, so the two cannot disagree at run time; this pins that the
/// derivation is the intended one, and would have to be deleted for a regression that made progress
/// a caller-supplied parameter again to pass.
///
/// Retry eligibility is checked here because it is a function of exactly this pair: only `Provision`
/// and `Connect` are `Retryable`, being the only infrastructure failures whose stage places them
/// before the first measured sample. It is a prospective authorization for a separately frozen retry
/// inventory — this screen schedules nothing on it.
#[test]
fn stage_and_sampling_progress_cannot_disagree() {
    assert_frozen_tables_are_total();

    for (kind, expected_stage) in FROZEN_STAGES {
        let failure = failure_of(kind);

        assert_eq!(
            failure.stage(),
            expected_stage,
            "{kind:?} must be recorded at the stage the frozen mapping names"
        );
        assert_eq!(
            kind.stage(),
            expected_stage,
            "{kind:?} must map to the stage the frozen mapping names"
        );

        // The measurement window is exactly the two bracketing stages; every earlier stage precedes
        // the timed subscription's issue.
        let inside_window = matches!(
            expected_stage,
            AttemptStage::Bracketed | AttemptStage::Unbracketed
        );
        let expected_progress = match inside_window {
            true => SamplingProgress::AtOrAfterFirstSample,
            false => SamplingProgress::BeforeFirstSample,
        };
        assert_eq!(
            failure.progress(),
            expected_progress,
            "{kind:?} is {expected_stage:?}, so its sampling progress must be {expected_progress:?}"
        );
        assert_eq!(
            kind.can_follow_first_sample(),
            inside_window,
            "{kind:?} must agree with the frozen mapping about whether a sample had begun"
        );

        let expected_eligibility = match kind {
            FailureKind::Provision | FailureKind::Connect => RetryEligibility::Retryable,
            FailureKind::Reducer
            | FailureKind::HostObservationBefore
            | FailureKind::TimedSubscription
            | FailureKind::HostObservationAfter
            | FailureKind::ValidationSubscription
            | FailureKind::Sample
            | FailureKind::Semantics => RetryEligibility::NotRetryable,
        };
        assert_eq!(
            failure.retry_eligibility(),
            expected_eligibility,
            "{kind:?} must be {expected_eligibility:?} under the spec's retry rule"
        );
    }
}
