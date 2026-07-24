//! `from_raw` sorts before ranking: the same 1,000-sample ramp delivered in **descending** issue
//! order (as the raw vector preserves issue order, never sorted) still yields median 500 / IQR 500,
//! proving the quantiles are read from the order statistics, not the arrival order.

use std::time::Duration;

use crate::observation::latency_sample::LatencySample;
use crate::observation::latency_summary::LatencySummary;
use crate::observation::raw_latencies::RawLatencies;
use crate::params::BATCH_SIZE;

#[test]
fn from_raw_sorts_before_ranking() {
    let samples: Vec<LatencySample> = (1..=BATCH_SIZE)
        .rev()
        .map(|i| LatencySample::from_elapsed(Duration::from_nanos(i)))
        .collect();
    let raw = RawLatencies::sealed(samples).expect("a full BATCH_SIZE ramp seals");

    let summary = LatencySummary::from_raw(&raw);
    assert_eq!(
        summary.median_nanos(),
        500,
        "the median is an order statistic, independent of issue order"
    );
    assert_eq!(
        summary.iqr_nanos(),
        500,
        "the IQR is independent of issue order"
    );
}
