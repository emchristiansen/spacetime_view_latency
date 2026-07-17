//! A prerequisite batch one confirmation short does not complete: once the supplied absolute deadline
//! has passed, the barrier's receive returns `RecvTimeoutError::Timeout` rather than proceeding.
//!
//! The deadline is set to *now* — already expired by the time any receive runs — so the test is
//! deterministic and wall-clock-free: `recv_timeout` still drains the already-queued messages via its
//! initial non-blocking receive, then returns `Timeout` immediately on the empty channel with a
//! `Duration::ZERO` budget. The sender is deliberately kept alive so the empty channel yields a
//! `Timeout`, not a `Disconnected`.

use std::sync::mpsc;
use std::time::Instant;

use crate::params::BATCH_SIZE_USIZE;

use super::super::{collect_prerequisite_batch, PrerequisiteMessage};

/// Deliver every prerequisite confirmation except the last. With the deadline already expired and a
/// sender still alive, the barrier drains the queued confirmations, finds the set incomplete, and its
/// next receive times out — surfacing a [`mpsc::RecvTimeoutError::Timeout`] source rather than
/// completing.
#[test]
fn a_missing_prerequisite_times_out_at_the_supplied_deadline() {
    let (tx, rx) = mpsc::channel::<PrerequisiteMessage>();

    for index in 0..BATCH_SIZE_USIZE - 1 {
        tx.send(PrerequisiteMessage::Confirmed { index })
            .expect("the receiver is alive");
    }

    let deadline = Instant::now();
    let error = collect_prerequisite_batch(rx, BATCH_SIZE_USIZE, deadline)
        .expect_err("a batch missing a prerequisite must time out, not complete");

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
