//! A refused preflight gate leaves its slot retryable.

use crate::view_read_set_campaign::retry_eligibility::RetryEligibility;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;

use super::fixture;

/// Coverage: this is the rule's one branch that is not a failure of a run at all. The gate is
/// prospective and ends immediately before launch, so nothing was measured and a retry cannot
/// reference a measured outcome — which is exactly why the spec lets the slot be attempted again on
/// a quieter host rather than recording a candidate result the environment produced.
///
/// It is asserted separately from the infrastructure branch because the two qualify for unrelated
/// reasons: this one because no measurement existed, that one because the measurement had not yet
/// begun. A rule that only handled failures would leave this outcome unclassified.
#[test]
fn the_preflight_rejection_is_eligible() {
    let rejected = fixture::record(RetryOrdinal::ORIGINAL, fixture::preflight_rejected());

    assert_eq!(
        rejected.retry_eligibility(),
        RetryEligibility::Eligible,
        "a prospective gate refusal measured nothing, so its slot may still be attempted"
    );
}
