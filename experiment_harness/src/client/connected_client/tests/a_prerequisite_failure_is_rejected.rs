//! A prerequisite write whose callback reports a failure fails phase A loud rather than proceeding to
//! the measured batch: the error names the failing write's issue index and carries the underlying
//! text, so a failed prerequisite can never precede a measured write on a missing chronicle row.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::params::BATCH_SIZE_USIZE;

use super::super::{collect_prerequisite_batch, PrerequisiteMessage};

/// A single [`PrerequisiteMessage::Failed`] ends phase A with an error, even though no other
/// confirmations arrived — a prerequisite failure is never swallowed.
#[test]
fn a_prerequisite_failure_is_rejected() {
    let (tx, rx) = mpsc::channel::<PrerequisiteMessage>();
    tx.send(PrerequisiteMessage::Failed {
        index: 5,
        error: "reducer returned an error: chronicle rejected".to_string(),
    })
    .expect("the receiver is alive");
    drop(tx);

    let deadline = Instant::now() + Duration::from_secs(30);
    let error = collect_prerequisite_batch(rx, BATCH_SIZE_USIZE, deadline)
        .expect_err("a prerequisite callback failure must be rejected");
    let rendered = error.to_string();
    assert!(
        rendered.contains("chronicle_message write index 5"),
        "the error names the failing prerequisite write: {rendered}"
    );
    assert!(
        rendered.contains("chronicle rejected"),
        "the error carries the underlying failure text: {rendered}"
    );
}
