//! A paced append stops when *its own* row is inserted, reports that insert's own instant, and never
//! stops on another append's.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::client::measured_step_failure::MeasuredStepFailure;
use crate::observation::latency_sample::LatencySample;

use super::super::{await_visible_append, PacedAppendMessage};
use super::paced_append_fixture::checked_after;

/// The activity id this sample's append writes.
const TARGET_ID: u64 = 1_000_000_010;

/// How long after this sample's start its own row was inserted.
const OBSERVED_AFTER: Duration = Duration::from_millis(7);

/// A budget so generous that a timeout could only mean the target's insert was never read.
const GENEROUS: Duration = Duration::from_secs(30);

/// Coverage: the two halves of the stop condition, plus the endpoint it reports.
///
/// One observer serves the whole batch, so an insert naming another append's id can reach the
/// channel — a stale delivery from a sample already sealed, or a row this batch did not write.
/// Stopping on one would time the wrong append.
///
/// Both directions are asserted, because either alone passes a wrong implementation: a loop stopping
/// on anything still returns `Ok` in the first half, and one never stopping still times out in the
/// second. Together they say the target id is sufficient and a foreign id is not.
///
/// The endpoint is asserted in the same scenario rather than in a file of its own, because the
/// scenario is already exactly the one that discriminates: the foreign inserts carry instants *later*
/// than the target's and the barrier is entered well after all three were queued, so a barrier
/// returning the last instant seen, the largest, or its own wall-clock reading each yields a
/// different number. Only "the target message's own instant" yields this one. That matters because
/// the estimand is issue-to-visible: reading a clock after the barrier woke would fold this thread's
/// scheduling into every sample as a fixed additive residue.
#[test]
fn a_paced_append_stops_only_on_its_own_target_id() {
    // Foreign ids around this sample's own: skipped, then stopped on.
    let start = Instant::now();
    let (tx, rx) = mpsc::channel::<PacedAppendMessage>();
    tx.send(PacedAppendMessage::Visible {
        id: 1_000_000_003,
        observed_at: checked_after(start, Duration::from_millis(50)),
    })
    .expect("the receiver is alive");
    tx.send(PacedAppendMessage::Visible {
        id: TARGET_ID,
        observed_at: checked_after(start, OBSERVED_AFTER),
    })
    .expect("the receiver is alive");
    tx.send(PacedAppendMessage::Visible {
        id: 1_000_000_090,
        observed_at: checked_after(start, Duration::from_millis(90)),
    })
    .expect("the receiver is alive");

    let sample = match await_visible_append(&rx, TARGET_ID, start, checked_after(start, GENEROUS)) {
        Ok(sample) => sample,
        Err(failure) => panic!(
            "the target id's insert is queued, so the sample completes; got {:#}",
            failure.into_error()
        ),
    };
    assert_eq!(
        sample.nanos(),
        LatencySample::from_elapsed(OBSERVED_AFTER).nanos(),
        "the sample is exactly its own observer callback's interval from this append's issue"
    );
    drop(tx);

    // Foreign ids alone, against a deadline set to *now* — already expired by the time any receive
    // runs, so the half is deterministic and wall-clock-free: `recv_timeout` still drains the queued
    // foreign inserts through its initial non-blocking receive, then times out on the empty channel
    // with a `Duration::ZERO` budget. The sender is deliberately kept alive so that empty channel
    // yields a `Timeout` rather than a `Disconnected`.
    let (other_tx, other_rx) = mpsc::channel::<PacedAppendMessage>();
    let other_start = Instant::now();
    for id in [1_000_000_003, 1_000_000_090] {
        other_tx
            .send(PacedAppendMessage::Visible {
                id,
                observed_at: other_start,
            })
            .expect("the receiver is alive");
    }

    let expired = Instant::now();
    let failure = match await_visible_append(&other_rx, TARGET_ID, other_start, expired) {
        Ok(_) => panic!("another append's insert is not this sample's row becoming visible"),
        Err(failure) => failure,
    };
    assert!(
        matches!(failure, MeasuredStepFailure::Timeout(_)),
        "a sample whose own row never appeared exceeded its bound; it did not fail at the reducer"
    );
    drop(other_tx);
}
