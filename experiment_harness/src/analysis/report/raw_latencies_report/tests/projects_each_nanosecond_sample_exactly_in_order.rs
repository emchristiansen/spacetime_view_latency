//! `RawLatenciesReport::of` copies every one of the BATCH_SIZE raw nanosecond samples exactly, in order —
//! raw nanoseconds stay exact `u128`, crossing no lossy float boundary.

use crate::analysis::report::raw_latencies_report::RawLatenciesReport;
use crate::observation::latency_sample::LatencySample;
use crate::observation::raw_latencies::RawLatencies;
use crate::params::BATCH_SIZE_USIZE;

#[test]
fn projects_each_nanosecond_sample_exactly_in_order() {
    // Distinct nanosecond values above 2^53, where an `f64` round-trip would lose precision — so a lossy
    // path through a float would corrupt them, and the position-dependent value makes ordering observable.
    let nanos = |i: usize| (1u128 << 60) + i as u128;
    let samples = (0..BATCH_SIZE_USIZE)
        .map(|i| LatencySample::from_nanos(nanos(i)))
        .collect();
    let latencies =
        RawLatencies::sealed(samples).expect("exactly BATCH_SIZE_USIZE samples seal into a raw vector");

    let report = RawLatenciesReport::of(&latencies);

    for i in 0..BATCH_SIZE_USIZE {
        assert_eq!(
            report.0[i],
            nanos(i),
            "sample {i} is preserved exactly as its original u128, in order"
        );
    }
}
