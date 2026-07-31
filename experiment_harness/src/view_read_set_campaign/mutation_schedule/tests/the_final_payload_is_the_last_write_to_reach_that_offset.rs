//! The derived after-state names exactly the payload the batch's last write to that key left.

use std::collections::HashMap;

use crate::view_read_set_campaign::campaign_params::OWNED_KEY_BASE;
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;
use crate::view_read_set_campaign::mutation_schedule::MutationSchedule;

/// Coverage: `final_payload_at_offset` is what the composition check judges the observed owned
/// slice against after a batch confirms, and it is a *closed-form* claim — the payload of write
/// index `CHANNEL_SAMPLE_COUNT - SUBSCRIBER_VISIBLE_ROWS_BASELINE + k`. If that formula is wrong,
/// the check compares real rows against a state no write produced, and a correct run fails as a
/// semantics violation.
///
/// So the formula is verified against the schedule's own walk rather than restated: replay every
/// write in issue order, keeping the last payload written to each slice offset, and require the
/// closed form to agree at every offset. The two derivations are independent — one is arithmetic on
/// the batch length, the other is a simulation of the walk — so agreeing is evidence rather than a
/// tautology.
///
/// The replay is keyed by *slice offset*, recovered as `target_key(write) - OWNED_KEY_BASE`, so the
/// test also exercises the cycling itself: were `target_key` to stop cycling, offsets would go
/// missing here rather than the two derivations agreeing on a wrong answer.
#[test]
fn the_final_payload_is_the_last_write_to_reach_that_offset() {
    for channel in [
        MeasurementChannel::PacedVisibleApplyLatency,
        MeasurementChannel::SaturatedQueueGrowthPerWrite,
    ] {
        let schedule = MutationSchedule::of(channel)
            .expect("a write-issuing channel has a frozen mutation schedule");

        // Replay the batch in issue order; the last write to an offset wins, as it does on the
        // server.
        let mut last_written: HashMap<u64, String> = HashMap::new();
        for write in schedule.writes() {
            let key = schedule.target_key(write);
            let offset = key
                .checked_sub(OWNED_KEY_BASE)
                .expect("a measured write targets the owned slice, never below its base");
            last_written.insert(offset, schedule.payload(write));
        }

        for offset in schedule.owned_slice_offsets() {
            let replayed = last_written
                .get(&offset.get())
                .expect("the batch's walk reaches every owned slice offset at least once");
            assert_eq!(
                &schedule.final_payload_at_offset(offset),
                replayed,
                "{channel:?}'s closed-form after-state at offset {} disagrees with the batch's \
                 own last write to that offset",
                offset.get()
            );
        }
    }
}
