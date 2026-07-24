//! Exactly `BATCH_SIZE` samples seal into a `RawLatencies` holding exactly that many.

use crate::observation::raw_latencies::RawLatencies;
use crate::params::BATCH_SIZE_USIZE;

use super::samples::samples;

#[test]
fn exactly_batch_size_seals() {
    let raw = RawLatencies::sealed(samples(BATCH_SIZE_USIZE))
        .expect("a full batch of latency samples seals");
    assert_eq!(
        raw.samples().len(),
        BATCH_SIZE_USIZE,
        "a sealed vector holds exactly BATCH_SIZE latency samples"
    );
}
