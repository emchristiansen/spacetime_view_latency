//! A measured batch one confirmation short does not seal: once the supplied absolute deadline has
//! passed, the barrier's receive returns `RecvTimeoutError::Timeout` rather than sealing a short
//! batch.
//!
//! The deadline is set to *now* — already expired by the time any receive runs — so the test is
//! deterministic and wall-clock-free: `recv_timeout` still drains the already-queued messages via its
//! initial non-blocking receive, then returns `Timeout` immediately on the empty channel with a
//! `Duration::ZERO` budget. The sender is deliberately kept alive so the empty channel yields a
//! `Timeout`, not a `Disconnected`.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::observation::latency_sample::LatencySample;
use crate::params::BATCH_SIZE_USIZE;

use super::super::{collect_measured_batch, MeasuredMessage};

/// Deliver every measured confirmation except the last. With the deadline already expired and a
/// sender still alive, the barrier drains the queued confirmations, finds the batch incomplete, and
/// its next receive times out — surfacing a [`mpsc::RecvTimeoutError::Timeout`] source rather than
/// sealing.
#[test]
fn a_missing_measured_confirmation_times_out_at_the_supplied_deadline() {
    let (tx, rx) = mpsc::channel::<MeasuredMessage>();

    for index in 0..BATCH_SIZE_USIZE - 1 {
        tx.send(MeasuredMessage::Confirmed {
            index,
            sample: LatencySample::from_elapsed(Duration::from_nanos(index as u64)),
        })
        .expect("the receiver is alive");
    }

    let deadline = Instant::now();
    let error = collect_measured_batch(rx, deadline)
        .expect_err("a batch missing a measured confirmation must time out, not seal");

    // `.context` preserves the underlying source, so anyhow can downcast to the exact timeout kind —
    // a stronger assertion than matching the context wording.
    assert!(
        matches!(
            error.downcast_ref::<mpsc::RecvTimeoutError>(),
            Some(mpsc::RecvTimeoutError::Timeout)
        ),
        "the receive failed with a Timeout source, not a disconnect or other error: {error:?}"
    );

    // The sender outlived the receive, so the failure was a genuine timeout rather than a disconnect.
    drop(tx);
}
