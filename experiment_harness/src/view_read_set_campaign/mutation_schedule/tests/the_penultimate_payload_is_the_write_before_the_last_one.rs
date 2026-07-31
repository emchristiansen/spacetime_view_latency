//! The derived pre-image is the payload of the second-to-last write to reach that offset.

use std::collections::HashMap;

use crate::view_read_set_campaign::campaign_params::OWNED_KEY_BASE;
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;
use crate::view_read_set_campaign::mutation_schedule::MutationSchedule;

/// Coverage: `penultimate_payload_at_offset` is the *expected* half of the final-mutation witness —
/// the state a key held immediately before the batch's last write to it. No observation can hold
/// that value, since capturing it would mean stopping the pipeline under measurement, so nothing
/// downstream can contradict a wrong derivation: the composition check would compare the observed
/// final payload against a pre-image from the wrong write and still find them unequal, quietly
/// witnessing a mutation that is not the final one.
///
/// So it is verified here against the schedule's own walk, and independently of the closed form:
/// replay every write in issue order keeping the last *two* payloads seen at each offset, and
/// require the derived pre-image to equal the second-newest. Both channels are checked, since each
/// carries its own tag into the payload and the witness is only meaningful within one batch.
#[test]
fn the_penultimate_payload_is_the_write_before_the_last_one() {
    for channel in [
        MeasurementChannel::PacedVisibleApplyLatency,
        MeasurementChannel::SaturatedQueueGrowthPerWrite,
    ] {
        let schedule = MutationSchedule::of(channel)
            .expect("a write-issuing channel has a frozen mutation schedule");

        // Replay the batch in issue order, keeping `(previous, latest)` per offset — the pair the
        // final write and its own pre-image occupy once the walk finishes.
        let mut seen: HashMap<u64, (Option<String>, String)> = HashMap::new();
        for write in schedule.writes() {
            let key = schedule.target_key(write);
            let offset = key
                .checked_sub(OWNED_KEY_BASE)
                .expect("a measured write targets the owned slice, never below its base");
            let payload = schedule.payload(write);
            match seen.remove(&offset) {
                None => seen.insert(offset, (None, payload)),
                Some((_, latest)) => seen.insert(offset, (Some(latest), payload)),
            };
        }

        for offset in schedule.owned_slice_offsets() {
            let (previous, _) = seen
                .get(&offset.get())
                .expect("the batch's walk reaches every owned slice offset at least once");
            let previous = previous.as_deref().expect(
                "a batch covering the slice at least twice writes every offset more than once",
            );
            assert_eq!(
                schedule.penultimate_payload_at_offset(offset),
                previous,
                "{channel:?}'s derived pre-image at offset {} is not what its own second-to-last \
                 write to that offset left there",
                offset.get()
            );
        }
    }
}
