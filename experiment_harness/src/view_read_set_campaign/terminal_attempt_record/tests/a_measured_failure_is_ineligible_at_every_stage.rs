//! The four non-infrastructure causes are refused a retry at every lifecycle stage, early ones
//! included.

use crate::view_read_set_campaign::failure_kind::FailureKind;
use crate::view_read_set_campaign::failure_stage::FailureStage;
use crate::view_read_set_campaign::retry_eligibility::RetryEligibility;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;

use super::fixture;

/// Coverage: the whole product of the four measured causes and the three stages, because the claim
/// is that the cause *alone* settles these — the timing term does not enter. Asserting them only at
/// [`FailureStage::AfterFirstSample`] would pass under a rule that had wrongly conjoined cause with
/// timing, since that stage refuses everything anyway. It is the early stages that discriminate, and
/// they are the ones a mistaken rule would let through.
///
/// [`FailureKind::Timeout`] carries the sharpest version of the claim. A timeout frequently *is* an
/// infrastructure symptom, and the spec names it separately for exactly that reason: folding it into
/// [`FailureKind::Infrastructure`] would let the rule be relaxed by reclassifying a candidate result
/// as an accident of the host. So a `Timeout` at `BeforePublish` — as early as a failure can be — is
/// still ineligible.
///
/// [`FailureKind::SemanticsOrSecurity`] carries the consequence that matters most: for the Arm, that
/// cause is the sender-scoped view leaking the foreign slice. Retrying it would be re-rolling the
/// candidate's answer until the host produced a better one.
#[test]
fn a_measured_failure_is_ineligible_at_every_stage() {
    for kind in [
        FailureKind::Application,
        FailureKind::Timeout,
        FailureKind::NonpositiveStatistic,
        FailureKind::SemanticsOrSecurity,
    ] {
        for stage in [
            FailureStage::BeforePublish,
            FailureStage::AfterPublishBeforeFirstSample,
            FailureStage::AfterFirstSample,
        ] {
            let failed = fixture::record(RetryOrdinal::ORIGINAL, fixture::failed(kind, stage));

            assert_eq!(
                failed.retry_eligibility(),
                RetryEligibility::Ineligible,
                "{kind:?} is a measured answer about the candidate, so it is refused a retry at \
                 {stage:?} as at any other stage"
            );
        }
    }
}
