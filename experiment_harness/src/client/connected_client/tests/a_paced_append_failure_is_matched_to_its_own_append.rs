//! A failure report ends only the append it names: a stale one is skipped, and the current one is
//! the module's answer rather than a bound exceeded.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::client::measured_step_failure::MeasuredStepFailure;

use super::super::{await_visible_append, PacedAppendMessage};
use super::paced_append_fixture::checked_after;

/// The activity id this sample's append writes.
const TARGET_ID: u64 = 1_000_000_010;

/// The id of an earlier append whose failure report arrives late.
const STALE_ID: u64 = 1_000_000_009;

/// A budget so generous that a timeout could only mean a message was mishandled.
const GENEROUS: Duration = Duration::from_secs(30);

/// Coverage: the race that makes a failure's key load-bearing, and the classification once it
/// matches.
///
/// **The stale half.** A completion callback and the observer are two producers on one channel. An
/// append's visibility can be delivered *before* a contradictory completion report about the same
/// call — a cache apply followed by an SDK internal error — and that report then sits unread while
/// the next sample waits. A barrier reading failures positionally would end this innocent sample on
/// its predecessor's message, recording an application failure for an append that had not even been
/// issued when the failure was produced, and abandoning a batch that was proceeding correctly. The
/// deadline here is generous, so a `Timeout` would mean the stale message was consumed as a stop and
/// the target's own insert never reached the loop.
///
/// **The matching half.** The stop condition is visibility, not confirmation, so an append that
/// failed at the reducer is never visible. Without its failure reaching the same channel the sample
/// would sit out its whole budget and report a timeout — the module's refusal recorded as a bound
/// exceeded. The deadline is generous there too, so `Timeout` would mean the failure was ignored
/// rather than that the budget was tight.
///
/// The classification is matched through a reference so the diagnostic assertion can still consume
/// the failure afterwards; [`MeasuredStepFailure`] carries an `anyhow::Error` and is not `Copy`.
#[test]
fn a_paced_append_failure_is_matched_to_its_own_append() {
    // A stale failure ahead of this sample's own insert: skipped, and the sample still completes.
    let start = Instant::now();
    let (tx, rx) = mpsc::channel::<PacedAppendMessage>();
    tx.send(PacedAppendMessage::Failed {
        index: 41,
        id: STALE_ID,
        error: "internal error awaiting reducer: late contradictory report".to_string(),
    })
    .expect("the receiver is alive");
    tx.send(PacedAppendMessage::Visible {
        id: TARGET_ID,
        observed_at: checked_after(start, Duration::from_millis(3)),
    })
    .expect("the receiver is alive");

    if let Err(failure) = await_visible_append(&rx, TARGET_ID, start, checked_after(start, GENEROUS))
    {
        panic!(
            "a failure naming another append must not end this one; got {:#}",
            failure.into_error()
        );
    }
    drop(tx);

    // A failure naming *this* append: the module's answer, delivered through the skip path so it is
    // not merely the first message read.
    let (own_tx, own_rx) = mpsc::channel::<PacedAppendMessage>();
    let own_start = Instant::now();
    own_tx
        .send(PacedAppendMessage::Visible {
            id: STALE_ID,
            observed_at: own_start,
        })
        .expect("the receiver is alive");
    own_tx
        .send(PacedAppendMessage::Failed {
            index: 42,
            id: TARGET_ID,
            error: "duplicate unique column id".to_string(),
        })
        .expect("the receiver is alive");

    let own_deadline = checked_after(own_start, GENEROUS);
    let failure = match await_visible_append(&own_rx, TARGET_ID, own_start, own_deadline) {
        Ok(_) => panic!("a sample whose append failed can never become visible"),
        Err(failure) => failure,
    };
    assert!(
        matches!(&failure, MeasuredStepFailure::Application(_)),
        "a reducer-returned or internal error is the module's answer, never a bound exceeded"
    );
    let message = format!("{:#}", failure.into_error());
    assert!(
        message.contains("paced append index 42")
            && message.contains(&TARGET_ID.to_string())
            && message.contains("duplicate unique column id"),
        "the rejection names the failing append by index and id and preserves the reducer's own \
         message; got {message}"
    );
    drop(own_tx);
}
