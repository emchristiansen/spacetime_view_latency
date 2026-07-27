//! A preflight whose readings could not be taken leaves its slot retryable.

use crate::view_read_set_campaign::retry_eligibility::RetryEligibility;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;

use super::fixture;

/// Coverage: the spec's retryable branches are prospective gate invalidation *and* infrastructure
/// failure before the first measured sample, and this outcome is the second one arriving before the
/// gate could produce a verdict at all. Nothing launched, so no measured outcome is referenced.
///
/// Asserted separately from the refusal because the two carry different evidence — a refusal has
/// readings, this has none — and an implementation that classified on the presence of a gate would
/// pass the refusal's test and leave this outcome unclassified.
#[test]
fn an_unreadable_preflight_is_eligible() {
    let unreadable = fixture::record(RetryOrdinal::ORIGINAL, fixture::preflight_unreadable());

    assert_eq!(
        unreadable.retry_eligibility(),
        RetryEligibility::Eligible,
        "a preflight that could not be read measured nothing, so its slot may still be attempted"
    );
}
