//! Saturated confirmations delivered out of order still seal in issue order: the barrier sends each
//! timing pair to its issue-indexed slot, so the sealed batch follows the slot index, not delivery
//! order.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::view_read_set_campaign::campaign_params::CHANNEL_SAMPLE_COUNT_USIZE;

use super::super::{collect_saturated_batch, SaturatedMessage};

/// Coverage: this channel issues every write before awaiting any, so completion order and issue
/// order genuinely differ — and its statistic is the slope against *issue index*, so a batch sealed
/// in completion order would reduce to a different number while looking identical.
///
/// Each write's offsets encode its own index, so a sealed position is checked against the identity
/// of the write occupying it rather than against a plausible value. Delivery is scrambled.
#[test]
fn saturated_confirmations_seal_in_issue_order() {
    let (tx, rx) = mpsc::channel::<SaturatedMessage>();

    let deliver = |index: usize| {
        let offset = index as u128;
        tx.send(SaturatedMessage::Confirmed {
            index,
            issue_offset_nanos: offset,
            confirmation_offset_nanos: offset + 1,
        })
        .expect("the receiver is alive");
    };
    for index in (1..CHANNEL_SAMPLE_COUNT_USIZE).step_by(2) {
        deliver(index);
    }
    for index in (0..CHANNEL_SAMPLE_COUNT_USIZE).step_by(2) {
        deliver(index);
    }
    drop(tx);

    let deadline = Instant::now() + Duration::from_secs(30);
    // Matched rather than `expect`ed: the error is a `MeasuredStepFailure`, which carries no `Debug`
    // because its payload is an `anyhow::Error`. The match reports the cause instead of discarding it.
    let batch = match collect_saturated_batch(rx, deadline) {
        Ok(batch) => batch,
        Err(failure) => panic!(
            "a complete saturated batch seals; got {:#}",
            failure.into_error()
        ),
    };
    for (index, timing) in batch.writes().iter().enumerate() {
        assert_eq!(
            timing.issue_offset_nanos(),
            index as u128,
            "the timing at position {index} is the write issued at index {index}"
        );
        assert_eq!(
            timing.latency_nanos(),
            1,
            "position {index} keeps its own pair, so its latency is its own difference"
        );
    }
}
