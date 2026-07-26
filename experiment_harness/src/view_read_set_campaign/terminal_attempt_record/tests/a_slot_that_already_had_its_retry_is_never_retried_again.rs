//! The ordinal cap applies to every eligible classification, not to one favoured branch.

use crate::view_read_set_campaign::attempt_outcome::AttemptOutcome;
use crate::view_read_set_campaign::failure_kind::FailureKind;
use crate::view_read_set_campaign::failure_stage::FailureStage;
use crate::view_read_set_campaign::infrastructure_phase::InfrastructurePhase;
use crate::view_read_set_campaign::retry_eligibility::RetryEligibility;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;

use super::fixture;

/// Coverage: the protocol permits at most one retry per logical slot, so an attempt that is *itself*
/// the retry has no successor however it terminated. Both eligible classifications are checked at
/// [`RetryOrdinal::RETRY`] — the preflight rejection and the early infrastructure failure — because
/// the cap is global, and a cap written as a clause inside one arm would pass a test that only
/// exercised that arm.
///
/// Each outcome is judged at both ordinals in turn, from the same value, so the ordinal is provably
/// the only difference between the two verdicts. That is what distinguishes the cap from the
/// classification: without the contrast, an implementation that had simply misclassified these
/// outcomes as ineligible would satisfy the same assertions.
///
/// This is also the test that "categorically eligible" is a statement about the preflight
/// *classification* rather than an exception to the cap. Read the other way — a gate refusal always
/// retryable — a slot could be re-refused and re-attempted without bound on a persistently loaded
/// host, which is the unbounded rescheduling the single-retry cap exists to prevent.
#[test]
fn a_slot_that_already_had_its_retry_is_never_retried_again() {
    let eligible_outcomes: [(&str, fn() -> AttemptOutcome); 2] = [
        ("a refused preflight gate", fixture::preflight_rejected),
        ("an infrastructure failure before the first sample", || {
            fixture::failed(
                FailureKind::Infrastructure(InfrastructurePhase::Connect),
                FailureStage::AfterPublishBeforeFirstSample,
            )
        }),
    ];

    for (description, outcome) in eligible_outcomes {
        let original = fixture::record(RetryOrdinal::ORIGINAL, outcome());
        assert_eq!(
            original.retry_eligibility(),
            RetryEligibility::Eligible,
            "{description} at the original ordinal is the case the cap has something to cap"
        );

        let retry = fixture::record(RetryOrdinal::RETRY, outcome());
        assert_eq!(
            retry.retry_eligibility(),
            RetryEligibility::Ineligible,
            "{description} at the retry ordinal has already used the slot's single permitted retry"
        );
    }
}
