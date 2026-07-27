//! Retry scheduling yields an identity exactly when the terminal record's eligibility grants one.

use crate::view_read_set_campaign::campaign_driver::schedule_retry;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;

use super::fixture;

/// Coverage: the complete scheduling disposition, exhausted in one place because the four cases are
/// the whole of one rule — what this function returns for each state the eligibility classification
/// can be in, plus the one state that reaches it only through the global ordinal cap.
///
/// **Why the three refusing cases are not one case.** They arrive here for genuinely different
/// reasons, and a scheduler could get one right while getting another wrong. An application fault is
/// a measured answer about the candidate, so the protocol refuses another attempt. An attempt that
/// never ran raises no retry question at all — there is nothing to reopen. And a preflight rejection
/// *at* `RETRY` is categorically the eligible outcome, refused only because the slot has already had
/// its one retry; a scheduler that read the outcome variant instead of the classified eligibility
/// would grant a second one here and produce an identity the protocol forbids and `next_retry` could
/// not even build.
///
/// The granted case asserts the identity rather than merely its presence: the scheduled retry must
/// be the same logical slot at `RETRY`, since a retry filed under any other slot would attach this
/// attempt's second try to a measurement it has nothing to do with.
#[test]
fn a_retry_is_scheduled_exactly_when_eligibility_grants_one() {
    let eligible = fixture::record(RetryOrdinal::ORIGINAL, fixture::preflight_rejected());

    let scheduled = schedule_retry(&eligible)
        .expect("a preflight-rejected original is categorically eligible for its one retry");
    assert_eq!(
        scheduled,
        fixture::key(RetryOrdinal::RETRY),
        "the scheduled retry must be the same logical slot at the retry ordinal"
    );
    assert!(
        eligible.key().same_logical_slot(scheduled),
        "the scheduled retry must address the logical slot that earned it"
    );

    for (description, record) in [
        (
            "an original whose measured failure the protocol refuses to rerun",
            fixture::record(RetryOrdinal::ORIGINAL, fixture::application_failure()),
        ),
        (
            "an original that never executed, so has no measured slot to reopen",
            fixture::record(RetryOrdinal::ORIGINAL, fixture::not_run()),
        ),
        (
            "a retry whose own outcome classifies as eligible, capped because the slot has had its \
             one retry",
            fixture::record(RetryOrdinal::RETRY, fixture::preflight_rejected()),
        ),
    ] {
        assert_eq!(
            schedule_retry(&record),
            None,
            "{description} must schedule nothing"
        );
    }
}
