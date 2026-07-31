//! A paced sample's endpoint is the instant *its own* observer callback ran — not a foreign key's
//! instant, and not the moment the measuring thread woke.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::observation::latency_sample::LatencySample;

use super::super::{await_visible_update, PacedMessage};

/// The owned key this sample's write targets.
const TARGET_KEY: u64 = 7;

/// How long after this sample's start its own row became visible.
const OBSERVED_AFTER: Duration = Duration::from_millis(7);

/// Coverage: the estimand is issue-to-visible, so the endpoint must be read where visibility
/// happened. Reading a clock after the barrier returns would fold this thread's wake into every
/// sample — a fixed additive residue, which biases the channel's endpoint factor `T = S_last/S_first`
/// toward one, the direction that makes a growing candidate look flat.
///
/// The two foreign keys are given instants *later* than the target's, and the barrier is entered
/// well after all three were queued. So a barrier returning the last instant seen, the largest one,
/// or its own wall-clock reading each yields a different number, and only "the target message's own
/// instant" yields this one.
#[test]
fn a_paced_sample_reports_its_own_observer_instant() {
    let start = Instant::now();
    let (tx, rx) = mpsc::channel::<PacedMessage>();

    tx.send(PacedMessage::Visible {
        entity_uuid: 3,
        observed_at: start + Duration::from_millis(50),
    })
    .expect("the receiver is alive");
    tx.send(PacedMessage::Visible {
        entity_uuid: TARGET_KEY,
        observed_at: start + OBSERVED_AFTER,
    })
    .expect("the receiver is alive");
    tx.send(PacedMessage::Visible {
        entity_uuid: 9,
        observed_at: start + Duration::from_millis(90),
    })
    .expect("the receiver is alive");

    let deadline = start + Duration::from_secs(30);
    let sample = match await_visible_update(&rx, TARGET_KEY, start, deadline) {
        Ok(sample) => sample,
        Err(failure) => panic!(
            "the target key's update is queued, so the sample completes; got {:#}",
            failure.into_error()
        ),
    };

    assert_eq!(
        sample.nanos(),
        LatencySample::from_elapsed(OBSERVED_AFTER).nanos(),
        "the sample is exactly its own observer callback's interval from this write's issue"
    );
    drop(tx);
}
