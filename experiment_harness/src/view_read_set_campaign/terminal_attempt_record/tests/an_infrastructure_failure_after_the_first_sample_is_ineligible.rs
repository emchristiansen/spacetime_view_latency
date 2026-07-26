//! Past the first measured sample, even an infrastructure cause is refused a retry.

use crate::view_read_set_campaign::failure_kind::FailureKind;
use crate::view_read_set_campaign::failure_stage::FailureStage;
use crate::view_read_set_campaign::infrastructure_phase::InfrastructurePhase;
use crate::view_read_set_campaign::retry_eligibility::RetryEligibility;
use crate::view_read_set_campaign::retry_ordinal::RetryOrdinal;

use super::fixture;

/// Coverage: infrastructure is the rule's only *conditionally* eligible cause, so this is the test
/// that the condition is actually load-bearing. Everything but the stage is held identical to the
/// eligible case — same cause, same phase, same identity, same empty evidence — leaving the boundary
/// as the single difference producing the opposite verdict.
///
/// The claim behind the refusal is the contract's: "no retry criterion may reference a measured or
/// post-attempt outcome". Once a sample has been taken, retrying the slot would discard a measured
/// observation on the strength of a diagnosis of why measurement stopped — and that diagnosis is the
/// driver's classification, not an independently checkable fact.
#[test]
fn an_infrastructure_failure_after_the_first_sample_is_ineligible() {
    let failed = fixture::record(
        RetryOrdinal::ORIGINAL,
        fixture::failed(
            FailureKind::Infrastructure(InfrastructurePhase::Sample),
            FailureStage::AfterFirstSample,
        ),
    );

    assert_eq!(
        failed.retry_eligibility(),
        RetryEligibility::Ineligible,
        "the attempt had already measured, so retrying its slot would reference a measured outcome"
    );
}
