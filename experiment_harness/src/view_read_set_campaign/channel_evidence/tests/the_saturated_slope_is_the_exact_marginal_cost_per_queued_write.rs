//! The saturated reduction is the slope against *issue index*, not against issue offset.

use crate::analysis::stats::rational::Rational;
use crate::view_read_set_campaign::campaign_params::{
    CHANNEL_SAMPLE_COUNT, CHANNEL_SAMPLE_COUNT_USIZE,
};
use crate::view_read_set_campaign::channel_evidence::ChannelEvidence;
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;
use crate::view_read_set_campaign::saturated_timing_batch::SaturatedTimingBatch;
use crate::view_read_set_campaign::saturated_write_timing::SaturatedWriteTiming;

/// The per-write growth this batch is constructed to exhibit, in nanoseconds per queued write.
const SLOPE_NANOS_PER_WRITE: u128 = 3;
/// The spacing between consecutive *issue* offsets. Deliberately not equal to
/// [`SLOPE_NANOS_PER_WRITE`] and coprime with it, so a reduction taken against issue offsets instead
/// of issue indices yields `3/7` rather than `3` and this test fails.
const ISSUE_SPACING_NANOS: u128 = 7;
/// The batch's fixed overhead — the intercept, which a slope must be insensitive to.
const BASE_LATENCY_NANOS: u128 = 100;

/// Coverage: the spec's E1 estimand is the marginal cost *per issued write*, so the abscissa is
/// position in the batch. Issue offsets are retained alongside — precisely so the FIFO and
/// stable-spacing reading stays falsifiable — which makes them an easy wrong abscissa to reach for.
///
/// The batch is built exactly linear in the index, so every pairwise slope is identical and the
/// all-pairs Theil–Sen median is exactly [`SLOPE_NANOS_PER_WRITE`]. Issue offsets advance by a
/// different, coprime step, so the two candidate abscissae give different answers and the assertion
/// discriminates between them rather than merely confirming linearity.
///
/// The lossless timings are asserted to survive the reduction, since the spec requires the reported
/// `S` to stay recomputable from the record.
#[test]
fn the_saturated_slope_is_the_exact_marginal_cost_per_queued_write() {
    let mut writes = Vec::with_capacity(CHANNEL_SAMPLE_COUNT_USIZE);
    for index in 0..u128::from(CHANNEL_SAMPLE_COUNT) {
        let issue = index * ISSUE_SPACING_NANOS;
        let latency = BASE_LATENCY_NANOS + index * SLOPE_NANOS_PER_WRITE;
        writes.push(
            SaturatedWriteTiming::observed(issue, issue + latency)
                .expect("a confirmation at or after its issue is well formed"),
        );
    }
    let timings = SaturatedTimingBatch::sealed(writes).expect("a full batch of timings seals");

    let evidence = ChannelEvidence::saturated(timings).expect("a positively-sloped batch reduces");

    assert_eq!(
        evidence.channel(),
        MeasurementChannel::SaturatedQueueGrowthPerWrite,
        "the channel is read off the variant the reduction produced"
    );

    let expected = Rational::new(
        i128::try_from(SLOPE_NANOS_PER_WRITE).expect("the frozen test slope fits i128"),
        1,
    );
    assert_eq!(
        evidence.statistic().get(),
        expected,
        "the slope must be taken against issue index; against issue offset this batch reads 3/7"
    );

    match evidence {
        ChannelEvidence::SaturatedQueueGrowthPerWrite { timings, .. } => assert_eq!(
            timings.writes().len(),
            CHANNEL_SAMPLE_COUNT_USIZE,
            "the lossless per-write record must survive the reduction"
        ),
        other => panic!("the saturated reduction produced {other:?}"),
    }
}
