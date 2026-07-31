//! The paced reduction is the exact rational median, not a rounded or floating-point one.

use crate::analysis::stats::rational::Rational;
use crate::observation::latency_sample::LatencySample;
use crate::observation::raw_latencies::RawLatencies;
use crate::view_read_set_campaign::campaign_params::CHANNEL_SAMPLE_COUNT;
use crate::view_read_set_campaign::channel_evidence::ChannelEvidence;
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;

/// Coverage: the batch size is even, so the median is the mean of the two central order statistics
/// and is generally *not* an integer number of nanoseconds. That is the case that would be silently
/// lost to integer division or to an `f64` round trip, and `T = S_last / S_first` is defined on
/// exact rationals — so the half is the property under test.
///
/// The samples are built as `1..=BATCH_SIZE` nanoseconds in shuffled-looking order: the two central
/// values are then 500 and 501, whose mean is exactly `1001/2`. Issue order is deliberately not
/// ascending, because the median must come from the sorted sample and not from the storage order.
#[test]
fn the_paced_median_is_the_exact_mean_of_the_two_central_samples() {
    let count = u128::from(CHANNEL_SAMPLE_COUNT);
    // Odd nanosecond values first, then even — a permutation of 1..=CHANNEL_SAMPLE_COUNT whose
    // storage order is not its sorted order.
    let mut nanos: Vec<u128> = (1..=count).filter(|n| n % 2 == 1).collect();
    nanos.extend((1..=count).filter(|n| n % 2 == 0));

    let samples: Vec<LatencySample> = nanos
        .iter()
        .map(|n| LatencySample::from_nanos(*n))
        .collect();
    let latencies = RawLatencies::sealed(samples).expect("a full batch of samples seals");

    let evidence = ChannelEvidence::paced(latencies).expect("a positive batch reduces");

    assert_eq!(
        evidence.channel(),
        MeasurementChannel::PacedVisibleApplyLatency,
        "the channel is read off the variant the reduction produced"
    );
    assert_eq!(
        evidence.statistic().get(),
        Rational::new(1_001, 2),
        "the median of 1..=1000 is exactly 1001/2, which no integer statistic can represent"
    );
}
