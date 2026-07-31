//! A paced append channel with no senders left is a harness fault, not a bound exceeded.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::client::measured_step_failure::MeasuredStepFailure;

use super::super::{await_visible_append, PacedAppendMessage};
use super::paced_append_fixture::checked_after;

/// Coverage: the branch that separates "nothing arrived in time" from "nothing can ever arrive".
///
/// The measuring thread holds a sender for the whole batch and clones one per append, so during a
/// real sample the channel cannot run out of senders. A disconnect therefore means the harness
/// dropped one — a defect in this code, not an observation about the system under test. Classifying
/// it as a timeout would file that defect as a latency result: a sample that "exceeded its bound"
/// when in fact its bound was never running.
///
/// Deterministic and wall-clock-free: every sender is dropped before the barrier is entered, so the
/// very first receive returns `Disconnected` immediately whatever the deadline says. The deadline is
/// deliberately generous, so a `Timeout` here would mean the two ends were conflated rather than
/// that the budget was tight.
#[test]
fn a_dropped_paced_append_channel_is_an_infrastructure_failure() {
    let start = Instant::now();
    let (tx, rx) = mpsc::channel::<PacedAppendMessage>();
    drop(tx);

    let deadline = checked_after(start, Duration::from_secs(30));
    let failure = match await_visible_append(&rx, 1_000_000_010, start, deadline) {
        Ok(_) => panic!("a channel with no senders can deliver no visibility"),
        Err(failure) => failure,
    };
    assert!(
        matches!(failure, MeasuredStepFailure::Infrastructure(_)),
        "a dropped sender is the harness losing its own channel, never an elapsed bound"
    );
}
