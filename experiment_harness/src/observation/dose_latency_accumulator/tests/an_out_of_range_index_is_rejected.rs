//! Recording a confirmation at an index outside the fixed batch is a loud error, not a silent
//! grow-the-vec — an out-of-range measured write is unrepresentable in a well-formed dose.

use std::time::Duration;

use crate::observation::dose_latency_accumulator::DoseLatencyAccumulator;
use crate::observation::latency_sample::LatencySample;
use crate::params::BATCH_SIZE_USIZE;

/// The first index past the batch (`BATCH_SIZE_USIZE`) has no slot, so recording it fails rather than
/// extending the accumulator or being dropped.
#[test]
fn an_out_of_range_index_is_rejected() {
    let mut accumulator = DoseLatencyAccumulator::new();
    let sample = LatencySample::from_elapsed(Duration::from_nanos(1));

    let error = accumulator
        .record(BATCH_SIZE_USIZE, sample)
        .expect_err("an index past the batch must be rejected");
    assert!(
        error.to_string().contains("out of range"),
        "the error names the out-of-range condition: {error}"
    );
}
