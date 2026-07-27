//! A saturated write whose reducer returned an error fails its batch, classified as the application
//! failure it is.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::client::measured_step_failure::MeasuredStepFailure;

use super::super::{collect_saturated_batch, SaturatedMessage};

/// Coverage: the classification is the point, not the rejection. Only this barrier holds the callback
/// outcome, so only here can a reducer error be told from a transport fault — and folding the two
/// together would report the module's own refusal as infrastructure.
///
/// Taken as soon as it arrives: a batch with a failed write cannot seal, so running to the deadline
/// would replace a truthful cause with a timeout.
#[test]
fn a_failed_saturated_write_is_rejected_as_an_application_failure() {
    let (tx, rx) = mpsc::channel::<SaturatedMessage>();

    tx.send(SaturatedMessage::Confirmed {
        index: 0,
        issue_offset_nanos: 0,
        confirmation_offset_nanos: 1,
    })
    .expect("the receiver is alive");
    tx.send(SaturatedMessage::Failed {
        index: 1,
        error: "entity_owner row absent".to_string(),
    })
    .expect("the receiver is alive");
    drop(tx);

    let deadline = Instant::now() + Duration::from_secs(30);
    let failure = match collect_saturated_batch(rx, deadline) {
        Ok(_) => panic!("a batch containing a failed write must not seal"),
        Err(failure) => failure,
    };
    assert!(
        matches!(failure, MeasuredStepFailure::Application(_)),
        "a reducer-returned error is the module's answer, never an infrastructure fault"
    );
    let message = format!("{:#}", failure.into_error());
    assert!(
        message.contains("saturated write index 1")
            && message.contains("entity_owner row absent"),
        "the rejection names the failing write and preserves the reducer's own message; got {message}"
    );
}
