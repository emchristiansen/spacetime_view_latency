//! Measured confirmations delivered out of order still seal in issue order: the barrier sends each
//! sample to its issue-indexed slot, so the sealed vector follows the slot index, not delivery order.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::observation::latency_sample::LatencySample;
use crate::params::BATCH_SIZE_USIZE;

use super::super::{collect_measured_batch, MeasuredMessage};

/// Deliver every measured confirmation with a sample whose nanos equal its issue index, but
/// scrambled — the odd indices first, then the even — so delivery order differs from issue order. The
/// batch seals, and the samples read back in issue order regardless.
#[test]
fn measured_confirmations_seal_in_issue_order() {
    let (tx, rx) = mpsc::channel::<MeasuredMessage>();

    for index in (1..BATCH_SIZE_USIZE).step_by(2) {
        tx.send(MeasuredMessage::Confirmed {
            index,
            sample: LatencySample::from_elapsed(Duration::from_nanos(index as u64)),
        })
        .expect("the receiver is alive");
    }
    for index in (0..BATCH_SIZE_USIZE).step_by(2) {
        tx.send(MeasuredMessage::Confirmed {
            index,
            sample: LatencySample::from_elapsed(Duration::from_nanos(index as u64)),
        })
        .expect("the receiver is alive");
    }
    drop(tx);

    let deadline = Instant::now() + Duration::from_secs(30);
    let latencies = collect_measured_batch(rx, deadline).expect("a complete measured batch seals");
    for (index, sample) in latencies.samples().iter().enumerate() {
        assert_eq!(
            sample.nanos(),
            index as u128,
            "the sample at position {index} is the write issued at index {index}"
        );
    }
}
