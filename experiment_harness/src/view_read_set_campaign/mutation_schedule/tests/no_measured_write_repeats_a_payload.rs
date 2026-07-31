//! No measured write repeats a payload — within its own batch or across the two channels.

use std::collections::HashSet;

use crate::view_read_set_campaign::campaign_params::SEEDED_ROW_PAYLOAD;
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;
use crate::view_read_set_campaign::mutation_schedule::MutationSchedule;

/// Coverage: the pinned SpacetimeDB source elides a byte-identical update outright, so a repeated
/// payload is a write that measures nothing while still consuming a sample slot. Two distinct
/// repeats are possible and both are checked here in one pass over the combined set.
///
/// *Within* a batch, the write index is what separates payloads — the ten owned keys are revisited
/// a hundred times each, so without the index every revisit after the first would be elided.
///
/// *Across* batches, the channel tag is what separates them. E2 and E1 walk the same write indices
/// over the same keys, so an untagged E1 write `i` would reproduce byte-for-byte what E2 already
/// left on that key — the batch-boundary collision the spec names explicitly.
///
/// The third repeat is against the *seeded* state rather than against another write: every key
/// holds [`SEEDED_ROW_PAYLOAD`] when E2 issues its first write, so a generated payload colliding
/// with that constant would be elided at the batch's very first touch of each key. Cross-channel
/// uniqueness cannot catch it, because the seeded payload is not itself a generated one — it is a
/// separate frozen constant that could drift into the mutation prefix independently.
#[test]
fn no_measured_write_repeats_a_payload() {
    let mut seen: HashSet<String> = HashSet::new();
    let mut issued = 0usize;

    for channel in [
        MeasurementChannel::PacedVisibleApplyLatency,
        MeasurementChannel::SaturatedQueueGrowthPerWrite,
    ] {
        let schedule = MutationSchedule::of(channel)
            .expect("a write-issuing channel has a frozen mutation schedule");
        for write in schedule.writes() {
            let payload = schedule.payload(write);
            assert_ne!(
                payload,
                SEEDED_ROW_PAYLOAD,
                "{channel:?} write {} reproduces the seeded payload, so the very first write to \
                 that key would be elided as byte-identical and measure nothing",
                write.get()
            );
            assert!(
                seen.insert(payload.clone()),
                "{channel:?} write {} repeats the payload {payload:?}, which the pinned runtime \
                 elides as a byte-identical update",
                write.get()
            );
            issued += 1;
        }
    }

    assert_eq!(
        seen.len(),
        issued,
        "every measured write across both channels must carry a distinct payload"
    );
}
