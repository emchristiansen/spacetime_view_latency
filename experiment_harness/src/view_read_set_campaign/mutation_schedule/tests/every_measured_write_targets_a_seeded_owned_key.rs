//! Every measured write lands on a key the attempt already seeded, so cardinality never moves.

use crate::view_read_set_campaign::campaign_params::{
    OWNED_KEY_BASE, SUBSCRIBER_VISIBLE_ROWS_BASELINE,
};
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;
use crate::view_read_set_campaign::mutation_schedule::MutationSchedule;

/// Coverage: the spec holds every non-axis dimension at its baseline *while measuring*, so a
/// measured write that reached an unseeded key would raise the owned slice above
/// `SUBSCRIBER_VISIBLE_ROWS_BASELINE` and the attempt would no longer be at the scale point it
/// claims. This checks the two properties that rule that out together — every target is inside the
/// seeded range, and the batch reaches the whole range rather than hammering one key.
///
/// Both write-issuing channels are checked, because the target derivation is shared but the
/// schedules are distinct values.
#[test]
fn every_measured_write_targets_a_seeded_owned_key() {
    for channel in [
        MeasurementChannel::PacedVisibleApplyLatency,
        MeasurementChannel::SaturatedQueueGrowthPerWrite,
    ] {
        let schedule = MutationSchedule::of(channel)
            .expect("a write-issuing channel has a frozen mutation schedule");

        let slice_len = usize::try_from(SUBSCRIBER_VISIBLE_ROWS_BASELINE)
            .expect("the frozen owned slice length fits usize");
        let mut reached = vec![false; slice_len];
        for write in schedule.writes() {
            let key = schedule.target_key(write);
            assert!(
                key >= OWNED_KEY_BASE && key < OWNED_KEY_BASE + SUBSCRIBER_VISIBLE_ROWS_BASELINE,
                "{channel:?} write {} targets {key}, outside the seeded owned slice",
                write.get()
            );
            let offset = usize::try_from(key - OWNED_KEY_BASE)
                .expect("an offset within the frozen owned slice fits usize");
            reached[offset] = true;
        }

        assert!(
            reached.iter().all(|hit| *hit),
            "{channel:?}'s batch must reach every seeded owned key, or the frozen after-state \
             names a payload no write ever wrote"
        );
    }
}
