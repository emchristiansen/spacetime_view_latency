//! The frozen constants put the last measured write at index 999, on slice offset 9, after 989.

use crate::view_read_set_campaign::campaign_params::{
    MUTATION_PAYLOAD_PREFIX, SATURATED_MUTATION_TAG,
};
use crate::view_read_set_campaign::measurement_channel::MeasurementChannel;
use crate::view_read_set_campaign::mutation_schedule::MutationSchedule;

/// Coverage: the whole final-mutation witness rests on three derived numbers — which offset the
/// batch's last write lands on, which payload it leaves there, and which payload it replaced. Every
/// other test of this schedule checks those derivations against the schedule's *own* walk, which is
/// the right way to catch a formula error but cannot catch the two premises drifting together: were
/// `CHANNEL_SAMPLE_COUNT` or `SUBSCRIBER_VISIBLE_ROWS_BASELINE` to change, replay and closed form
/// would agree on a new answer and every such test would still pass.
///
/// So this one pins the *frozen* answer, spelled out: offset 9, final payload from write 999,
/// immediate predecessor from write 989. These are the exact numbers the spec froze for the
/// 1,000-write ten-row cycle, so a change to either constant fails here — where the preregistration
/// is named — rather than silently re-deriving the witness a recorded finding was checked against.
///
/// The payloads are rebuilt from the frozen prefix and tag rather than pasted as literals: the
/// claim under test is which *write index* the schedule names, and re-spelling the payload format
/// here would turn an unrelated format change into a failure of this test.
#[test]
fn the_final_write_is_index_999_at_offset_9() {
    let schedule = MutationSchedule::of(MeasurementChannel::SaturatedQueueGrowthPerWrite)
        .expect("the saturated channel issues measured writes, so it has a schedule");

    let offset = schedule.final_write_offset();
    assert_eq!(
        offset.get(),
        9,
        "the frozen 1,000-write batch over a ten-row slice ends on slice offset 9"
    );

    assert_eq!(
        schedule.final_payload_at_offset(offset),
        format!("{MUTATION_PAYLOAD_PREFIX}:{SATURATED_MUTATION_TAG}:999"),
        "the last write of the frozen batch is index 999"
    );

    assert_eq!(
        schedule.penultimate_payload_at_offset(offset),
        format!("{MUTATION_PAYLOAD_PREFIX}:{SATURATED_MUTATION_TAG}:989"),
        "writes reach an offset once per ten-row cycle, so write 999's pre-image is write 989's \
         payload"
    );
}
