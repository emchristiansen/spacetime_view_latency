//! A fully-recorded batch seals into a `RawLatencies` whose samples are in issue order — even when
//! the confirmations were recorded out of order, proving the seal follows the slot index, not
//! completion order.

use std::time::Duration;

use crate::observation::dose_latency_accumulator::DoseLatencyAccumulator;
use crate::observation::latency_sample::LatencySample;
use crate::params::BATCH_SIZE_USIZE;

/// Record every slot with a sample whose nanos equal its issue index, but record the indices in a
/// scrambled order. The completed batch seals, and the sealed samples read back in issue order
/// `0..BATCH_SIZE` regardless of the record order.
#[test]
fn a_full_batch_seals_in_issue_order() {
    let mut accumulator = DoseLatencyAccumulator::new();

    // A fixed deterministic scramble: fill the odd indices first, then the even ones. Completion
    // order therefore differs from issue order, but the seal must still be issue-ordered.
    for index in (1..BATCH_SIZE_USIZE).step_by(2) {
        accumulator
            .record(
                index,
                LatencySample::from_elapsed(Duration::from_nanos(index as u64)),
            )
            .expect("recording a fresh in-range slot succeeds");
    }
    for index in (0..BATCH_SIZE_USIZE).step_by(2) {
        accumulator
            .record(
                index,
                LatencySample::from_elapsed(Duration::from_nanos(index as u64)),
            )
            .expect("recording a fresh in-range slot succeeds");
    }

    assert!(
        accumulator.is_complete(),
        "every slot is filled, so the batch is complete"
    );
    let latencies = accumulator.seal().expect("a full batch seals");
    for (index, sample) in latencies.samples().iter().enumerate() {
        assert_eq!(
            sample.nanos(),
            index as u128,
            "the sample at position {index} is the write issued at index {index}"
        );
    }
}
