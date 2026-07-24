//! The 1,000-sample fixture: an ascending ramp `x_(i) = i` ns over the full `BATCH_SIZE` batch.
//! R-1 reads Q1=x_(250)=250, med=x_(500)=500, Q3=x_(750)=750, so the reported summary is
//! median 500 ns / IQR 500 ns — exercised end-to-end through `RawLatencies` and `from_raw`.

use std::time::Duration;

use crate::observation::latency_sample::LatencySample;
use crate::observation::latency_summary::LatencySummary;
use crate::observation::raw_latencies::RawLatencies;
use crate::params::BATCH_SIZE;

#[test]
fn full_batch_ramp_summary() {
    let samples: Vec<LatencySample> = (1..=BATCH_SIZE)
        .map(|i| LatencySample::from_elapsed(Duration::from_nanos(i)))
        .collect();
    let raw = RawLatencies::sealed(samples).expect("a full BATCH_SIZE ramp seals");

    let summary = LatencySummary::from_raw(&raw);
    assert_eq!(
        summary.median_nanos(),
        500,
        "R-1 median rank ⌈1000·0.5⌉ = 500"
    );
    assert_eq!(
        summary.iqr_nanos(),
        500,
        "R-1 IQR = x_(750) − x_(250) = 750 − 250"
    );
}
