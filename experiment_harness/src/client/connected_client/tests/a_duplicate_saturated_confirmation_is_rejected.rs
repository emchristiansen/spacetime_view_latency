//! A saturated write whose confirmation fires twice is rejected rather than counted twice.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::client::measured_step_failure::MeasuredStepFailure;

use super::super::{collect_saturated_batch, SaturatedMessage};

/// Coverage: slot addressing exists so a double-fire cannot inflate one slot while another stays
/// empty — a batch reaching the frozen length that way would seal, over a vector that never happened.
///
/// Classified [`Infrastructure`](MeasuredStepFailure::Infrastructure), not
/// [`Application`](MeasuredStepFailure::Application): the write confirmed; the harness's accounting
/// of it is what failed.
#[test]
fn a_duplicate_saturated_confirmation_is_rejected() {
    let (tx, rx) = mpsc::channel::<SaturatedMessage>();

    for _ in 0..2 {
        tx.send(SaturatedMessage::Confirmed {
            index: 0,
            issue_offset_nanos: 0,
            confirmation_offset_nanos: 1,
        })
        .expect("the receiver is alive");
    }
    drop(tx);

    let deadline = Instant::now() + Duration::from_secs(30);
    let failure = match collect_saturated_batch(rx, deadline) {
        Ok(_) => panic!("a batch with a duplicated confirmation must not seal"),
        Err(failure) => failure,
    };
    assert!(
        matches!(failure, MeasuredStepFailure::Infrastructure(_)),
        "a duplicated confirmation is the harness mis-accounting a write that did confirm"
    );
    let message = format!("{:#}", failure.into_error());
    assert!(
        message.contains("duplicate confirmation for saturated write index 0"),
        "the rejection names the slot that fired twice; got {message}"
    );
}
