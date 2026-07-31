//! Both stages that precede the first measured sample leave an infrastructure failure retryable.

use crate::view_read_set_campaign::failure_kind::FailureKind;
use crate::view_read_set_campaign::failure_stage::FailureStage;
use crate::view_read_set_campaign::infrastructure_phase::InfrastructurePhase;
use crate::view_read_set_campaign::retry_eligibility::RetryEligibility;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;

use super::fixture;

/// Coverage: the rule's timing term is the *first measured sample*, and two of the three lifecycle
/// stages fall before it — one that never published and one that published and had not yet measured.
/// Both are asserted, because they reach the eligible verdict through
/// [`FailureStage::measured_sample_boundary`], and a rule written directly against the stage rather
/// than the derived boundary would plausibly admit only one of them.
///
/// The infrastructure phase is varied along with the stage to make the second claim: the phase is
/// diagnostic and never decisive. [`InfrastructurePhase::Sample`] straddles the boundary — a channel
/// can fail before its first sample confirms or after several have — so a rule that read the phase
/// instead of the stage would be wrong in precisely the case it governs. Here a `Sample`-phase
/// failure at a before-first stage is eligible, which no phase-based reading produces.
#[test]
fn an_infrastructure_failure_before_the_first_sample_is_eligible() {
    for (stage, phase) in [
        (FailureStage::BeforePublish, InfrastructurePhase::Provision),
        (
            FailureStage::AfterPublishBeforeFirstSample,
            InfrastructurePhase::Sample,
        ),
    ] {
        let failed = fixture::record(
            RetryOrdinal::ORIGINAL,
            fixture::failed(FailureKind::Infrastructure(phase), stage),
        );

        assert_eq!(
            failed.retry_eligibility(),
            RetryEligibility::Eligible,
            "an infrastructure failure at {stage:?} preceded the first measured sample, so its \
             slot may be attempted again"
        );
    }
}
