//! A measured write whose confirmation fires twice is rejected: the second delivery for the same
//! issue slot fails the barrier loud rather than overwriting the sample or inflating the count.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::observation::latency_sample::LatencySample;

use super::super::{collect_measured_batch, MeasuredMessage};

/// Deliver two measured confirmations for issue index `0`. The first fills the slot; the second is a
/// duplicate and the barrier returns an error naming the duplicate, before the batch could complete.
#[test]
fn a_duplicate_measured_confirmation_is_rejected() {
    let (tx, rx) = mpsc::channel::<MeasuredMessage>();
    tx.send(MeasuredMessage::Confirmed {
        index: 0,
        sample: LatencySample::from_elapsed(Duration::from_nanos(7)),
    })
    .expect("the receiver is alive");
    tx.send(MeasuredMessage::Confirmed {
        index: 0,
        sample: LatencySample::from_elapsed(Duration::from_nanos(9)),
    })
    .expect("the receiver is alive");
    drop(tx);

    let deadline = Instant::now() + Duration::from_secs(30);
    let error = collect_measured_batch(rx, deadline)
        .expect_err("a duplicate measured confirmation must be rejected");
    assert!(
        error.to_string().contains("duplicate"),
        "the error names the duplicate condition: {error}"
    );
}
