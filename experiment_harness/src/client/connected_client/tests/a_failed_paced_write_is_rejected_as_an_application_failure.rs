//! A paced write whose reducer returned an error fails its sample immediately, rather than expiring
//! against a deadline it can never meet.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::client::measured_step_failure::MeasuredStepFailure;

use super::super::{await_visible_update, PacedMessage};

/// Coverage: the stop condition is visibility, not confirmation, so a write that failed at the
/// reducer is never visible. Without the failure reaching the same channel the sample would sit out
/// its budget and report a timeout — the module's refusal recorded as a bound exceeded.
///
/// The deadline is generous, so a `Timeout` result would mean the failure was ignored rather than
/// that the budget was tight.
#[test]
fn a_failed_paced_write_is_rejected_as_an_application_failure() {
    let start = Instant::now();
    let (tx, rx) = mpsc::channel::<PacedMessage>();

    // An unrelated key's update first, so the failure is reached through the skip path rather than
    // as the very first message.
    tx.send(PacedMessage::Visible {
        entity_uuid: 3,
        observed_at: start,
    })
    .expect("the receiver is alive");
    tx.send(PacedMessage::Failed {
        index: 42,
        error: "entity_owner row absent".to_string(),
    })
    .expect("the receiver is alive");

    let deadline = start + Duration::from_secs(30);
    let failure = match await_visible_update(&rx, 7, start, deadline) {
        Ok(_) => panic!("a sample whose write failed can never become visible"),
        Err(failure) => failure,
    };
    assert!(
        matches!(failure, MeasuredStepFailure::Application(_)),
        "a reducer-returned error is the module's answer, never a bound exceeded"
    );
    let message = format!("{:#}", failure.into_error());
    assert!(
        message.contains("paced write index 42") && message.contains("entity_owner row absent"),
        "the rejection names the failing write and preserves the reducer's own message; got {message}"
    );
    drop(tx);
}
