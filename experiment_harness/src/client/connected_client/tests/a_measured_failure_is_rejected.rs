//! A measured write whose callback reports a failure fails the barrier loud rather than sealing a
//! partial batch: the error names the failing write's issue index and carries the underlying text.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::super::{collect_measured_batch, MeasuredMessage};

/// A single [`MeasuredMessage::Failed`] ends the barrier with an error, even though no other
/// confirmations arrived — a measured callback failure is never swallowed.
#[test]
fn a_measured_failure_is_rejected() {
    let (tx, rx) = mpsc::channel::<MeasuredMessage>();
    tx.send(MeasuredMessage::Failed {
        index: 3,
        error: "reducer returned an error: rejected".to_string(),
    })
    .expect("the receiver is alive");
    drop(tx);

    let deadline = Instant::now() + Duration::from_secs(30);
    let error = collect_measured_batch(rx, deadline)
        .expect_err("a measured callback failure must be rejected");
    let rendered = error.to_string();
    assert!(
        rendered.contains("measured write index 3"),
        "the error names the failing measured write: {rendered}"
    );
    assert!(
        rendered.contains("rejected"),
        "the error carries the underlying failure text: {rendered}"
    );
}
