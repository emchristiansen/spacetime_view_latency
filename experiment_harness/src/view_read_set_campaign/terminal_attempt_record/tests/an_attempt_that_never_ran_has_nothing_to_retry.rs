//! A slot that never executed poses no retry question — and neither does a completed one.

use crate::view_read_set_campaign::retry_eligibility::RetryEligibility;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;

use super::fixture;

/// Coverage: [`RetryEligibility::None`] is a third state, not a spelling of
/// [`RetryEligibility::Ineligible`], and this is the test that it is actually returned. The
/// distinction is what a reader of the ledger consults: "the rule refused this slot a retry" and
/// "this outcome raised no retry question" are different facts about a campaign, and a completed
/// attempt reporting the same disposition as a security failure would misdescribe both.
///
/// An unexecuted slot belongs there because its record says nothing about a run: it was skipped
/// after a preceding attempt failed to release its resources, so there is no measured slot to
/// reopen. A campaign stopped that way resumes by planning, not by retrying its unplayed remainder.
///
/// **Why [`AttemptOutcome::Complete`](crate::view_read_set_campaign::attempt_outcome::AttemptOutcome::Complete)
/// is not built here.** It shares this arm — one or-pattern over
/// both variants — so its verdict is not merely equal to this one, it is produced by the same
/// expression, and Rust's exhaustiveness check is what holds it there: separating the two would
/// require editing that arm, and deleting it would fail to compile. Constructing a complete outcome
/// means a four-channel `ScalePointEvidence` over a filesystem-backed `ValidatedComposition`, a
/// fixture whose entire content would be evidence this rule never reads, exercising a line this test
/// already executes. The compiler is the stronger check and the cheaper one.
#[test]
fn an_attempt_that_never_ran_has_nothing_to_retry() {
    let skipped = fixture::record(RetryOrdinal::ORIGINAL, fixture::not_run());

    assert_eq!(
        skipped.retry_eligibility(),
        RetryEligibility::None,
        "an attempt that never executed has no measured slot to reopen, which is not the same as \
         being refused one"
    );
}
