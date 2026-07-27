//! A channel that observed its sample and then failed at its reduction is a *measured* attempt, even
//! though its completed prefix is empty.

use crate::observation::latency_sample::LatencySample;
use crate::view_read_set_campaign::channel_evidence::ChannelEvidence;
use crate::view_read_set_campaign::failure_stage::FailureStage;
use crate::view_read_set_campaign::measured_sample_boundary::MeasuredSampleBoundary;

use super::super::failure_stage;

/// Coverage: the cold subscription runs first, so it is the one channel that can observe a sample
/// with an empty prefix — it applies, then its reduction may refuse a nonpositive apply duration.
/// Reading the prefix alone calls that `AfterPublishBeforeFirstSample`, making a measured attempt
/// retry-eligible and omitting the post-attempt reading reconciliation requires for one.
///
/// All four combinations, because the rule is a disjunction and each witness must be independently
/// sufficient: empty/`AfterFirst` is the defect, empty/`BeforeFirst` is what keeps the fix from
/// calling everything measured.
#[test]
fn an_observed_sample_makes_a_failure_measured_with_no_evidence_to_show_for_it() {
    let cold = ChannelEvidence::cold_subscription(LatencySample::from_nanos(1))
        .expect("a one-nanosecond apply duration is strictly positive");

    assert_eq!(
        failure_stage(&[], MeasuredSampleBoundary::AfterFirst),
        FailureStage::AfterFirstSample,
        "an observation the prefix cannot show still makes the attempt a measured one"
    );
    assert_eq!(
        failure_stage(&[], MeasuredSampleBoundary::BeforeFirst),
        FailureStage::AfterPublishBeforeFirstSample,
        "nothing measured and nothing observed is the one retry-eligible stage"
    );
    assert_eq!(
        failure_stage(
            std::slice::from_ref(&cold),
            MeasuredSampleBoundary::BeforeFirst
        ),
        FailureStage::AfterFirstSample,
        "a completed channel proves a sample was taken, whatever the failing step observed"
    );
    assert_eq!(
        failure_stage(
            std::slice::from_ref(&cold),
            MeasuredSampleBoundary::AfterFirst
        ),
        FailureStage::AfterFirstSample,
        "both witnesses agreeing changes nothing"
    );
}
