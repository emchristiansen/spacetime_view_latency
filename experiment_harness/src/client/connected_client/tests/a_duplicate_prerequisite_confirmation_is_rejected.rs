//! A prerequisite write whose confirmation fires twice is rejected: the set is slot-indexed, so a
//! double-fire fails the barrier loud rather than counting twice toward completion.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::params::BATCH_SIZE_USIZE;

use super::super::{collect_prerequisite_batch, PrerequisiteMessage};

/// Deliver two prerequisite confirmations for issue index `0`. The first fills the slot; the second
/// is a duplicate and the barrier returns an error naming the duplicate.
#[test]
fn a_duplicate_prerequisite_confirmation_is_rejected() {
    let (tx, rx) = mpsc::channel::<PrerequisiteMessage>();
    tx.send(PrerequisiteMessage::Confirmed { index: 0 })
        .expect("the receiver is alive");
    tx.send(PrerequisiteMessage::Confirmed { index: 0 })
        .expect("the receiver is alive");
    drop(tx);

    let deadline = Instant::now() + Duration::from_secs(30);
    let error = collect_prerequisite_batch(rx, BATCH_SIZE_USIZE, deadline)
        .expect_err("a duplicate prerequisite confirmation must be rejected");
    assert!(
        error.to_string().contains("duplicate"),
        "the error names the duplicate condition: {error}"
    );
}
