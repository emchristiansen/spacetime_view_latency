//! A paced sample stops when *its own* row becomes visible, and never on another owned key's update.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::client::measured_step_failure::MeasuredStepFailure;

use super::super::{await_visible_update, PacedMessage};

/// The owned key this sample's write targets.
const TARGET_KEY: u64 = 7;

/// Coverage: one observer serves the whole batch while the frozen schedule cycles the ten owned
/// keys, so a sample's own key is not the only one that arrives; stopping on any would time the
/// wrong write.
///
/// Both halves, because either alone passes a wrong implementation — a loop stopping on anything
/// still returns `Ok` in the first, one never stopping still times out in the second. Together they
/// say the target key is sufficient and a foreign key is not. Which instant the stop reports is a
/// separate question, proved by
/// [`a_paced_sample_reports_its_own_observer_instant`](super::a_paced_sample_reports_its_own_observer_instant).
#[test]
fn a_paced_sample_stops_only_on_its_own_target_key() {
    // Foreign keys ahead of this sample's own: skipped, then stopped on.
    let start = Instant::now();
    let (tx, rx) = mpsc::channel::<PacedMessage>();
    for entity_uuid in [0, 3, 9] {
        tx.send(PacedMessage::Visible {
            entity_uuid,
            observed_at: start + Duration::from_millis(1),
        })
        .expect("the receiver is alive");
    }
    tx.send(PacedMessage::Visible {
        entity_uuid: TARGET_KEY,
        observed_at: start + Duration::from_millis(2),
    })
    .expect("the receiver is alive");

    let deadline = start + Duration::from_secs(30);
    assert!(
        await_visible_update(&rx, TARGET_KEY, start, deadline).is_ok(),
        "an update naming this sample's own key is what makes its write observable"
    );

    // Foreign keys alone, against a deadline set to *now* — already expired by the time any receive
    // runs, so the half is deterministic and wall-clock-free: `recv_timeout` still drains the queued
    // foreign updates through its initial non-blocking receive, then times out on the empty channel
    // with a `Duration::ZERO` budget. The sender is deliberately kept alive so that empty channel
    // yields a `Timeout` rather than a `Disconnected`. The established shape of
    // `a_missing_measured_confirmation_times_out_at_the_supplied_deadline`.
    let (other_tx, other_rx) = mpsc::channel::<PacedMessage>();
    let other_start = Instant::now();
    for entity_uuid in [0, 3, 9] {
        other_tx
            .send(PacedMessage::Visible {
                entity_uuid,
                observed_at: other_start,
            })
            .expect("the receiver is alive");
    }

    let expired = Instant::now();
    let failure = match await_visible_update(&other_rx, TARGET_KEY, other_start, expired) {
        Ok(_) => panic!("another owned key's update is not this sample's write becoming visible"),
        Err(failure) => failure,
    };
    assert!(
        matches!(failure, MeasuredStepFailure::Timeout(_)),
        "a sample whose own row never appeared exceeded its bound; it did not fail at the reducer"
    );
    drop(other_tx);
}
