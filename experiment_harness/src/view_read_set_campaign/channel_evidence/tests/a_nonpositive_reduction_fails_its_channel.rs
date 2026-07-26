//! A reduction that is not strictly positive fails its channel instead of being reported as flat.

use crate::observation::latency_sample::LatencySample;
use crate::observation::raw_latencies::RawLatencies;
use crate::view_read_set_campaign::campaign_params::{
    CHANNEL_SAMPLE_COUNT, CHANNEL_SAMPLE_COUNT_USIZE,
};
use crate::view_read_set_campaign::channel_evidence::ChannelEvidence;
use crate::view_read_set_campaign::saturated_timing_batch::SaturatedTimingBatch;
use crate::view_read_set_campaign::saturated_write_timing::SaturatedWriteTiming;

/// Coverage: the spec is explicit that a missing, non-finite, or nonpositive `S` invalidates its
/// block and can *never* count as flat. The danger is specific — a zero statistic satisfies the flat
/// inequality `U <= 5/4` — so the refusal has to happen at the reduction, before any classifier sees
/// the value. Each of the four channels is checked, because each has its own constructor and a
/// missed admission in any one of them reopens the hole.
///
/// The saturated case is the subtle one: a genuinely flat batch has constant latency and therefore
/// slope exactly zero, so the channel that is most likely to *legitimately* produce a nonpositive
/// value is the one where accepting it would be most misleading. The spec's reasoning is recorded on
/// `CellStatistic`: the statistic is a queue service time, structurally above zero, so a zero here is
/// anomalous rather than the flat hypothesis being confirmed.
#[test]
fn a_nonpositive_reduction_fails_its_channel() {
    assert!(
        ChannelEvidence::cold_subscription(LatencySample::from_nanos(0)).is_err(),
        "a zero-nanosecond cold subscription apply is anomalous, not instantaneous"
    );
    assert!(
        ChannelEvidence::reconnect(LatencySample::from_nanos(0)).is_err(),
        "a zero-nanosecond reconnect apply is anomalous, not instantaneous"
    );

    let zero_samples = vec![LatencySample::from_nanos(0); CHANNEL_SAMPLE_COUNT_USIZE];
    let latencies = RawLatencies::sealed(zero_samples).expect("a full batch of samples seals");
    assert!(
        ChannelEvidence::paced(latencies).is_err(),
        "a paced batch whose median is zero must fail its attempt, not classify as flat"
    );

    // Constant latency across the batch — every pairwise slope is zero, so Theil–Sen is exactly zero.
    let mut writes = Vec::with_capacity(CHANNEL_SAMPLE_COUNT_USIZE);
    for index in 0..u128::from(CHANNEL_SAMPLE_COUNT) {
        let issue = index * 7;
        writes.push(
            SaturatedWriteTiming::observed(issue, issue + 100)
                .expect("a confirmation at or after its issue is well formed"),
        );
    }
    let timings = SaturatedTimingBatch::sealed(writes).expect("a full batch of timings seals");
    assert!(
        ChannelEvidence::saturated(timings).is_err(),
        "a zero marginal per-write slope must fail its attempt, not classify as flat"
    );
}
