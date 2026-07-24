//! The phase-A prerequisite barrier completes once every expected prerequisite has confirmed, in any
//! delivery order: the set tracks presence, not order, so a scrambled arrival still completes.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::params::BATCH_SIZE_USIZE;

use super::super::{collect_prerequisite_batch, PrerequisiteMessage};

/// Deliver every prerequisite confirmation, scrambled (odd indices first, then even). The barrier
/// requires the full expected set and returns `Ok` once all have arrived, independent of order.
#[test]
fn prerequisites_confirm_regardless_of_order() {
    let (tx, rx) = mpsc::channel::<PrerequisiteMessage>();

    for index in (1..BATCH_SIZE_USIZE).step_by(2) {
        tx.send(PrerequisiteMessage::Confirmed { index })
            .expect("the receiver is alive");
    }
    for index in (0..BATCH_SIZE_USIZE).step_by(2) {
        tx.send(PrerequisiteMessage::Confirmed { index })
            .expect("the receiver is alive");
    }
    drop(tx);

    let deadline = Instant::now() + Duration::from_secs(30);
    collect_prerequisite_batch(rx, BATCH_SIZE_USIZE, deadline)
        .expect("every prerequisite confirmed, so the phase completes");
}
