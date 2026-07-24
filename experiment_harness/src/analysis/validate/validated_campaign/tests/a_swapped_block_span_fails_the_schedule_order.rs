//! Schedule-order stage — negative proof: swapping two whole block spans puts a foreign block's record at
//! a sequence position the seed schedule reserves for another block, failing stage 6 with a
//! `BlockOutOfScheduleOrder` at the earliest displaced position.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::analysis::validate::schedule_order_fault::ScheduleOrderFault;
use crate::dataset::dose_index::DoseIndex;
use crate::observation::record_seq::RecordSeq;

/// Swapping the first two seed-ordered block spans — each block is its two matched runs, so 22 contiguous
/// records — leaves every record individually valid, the census complete, and provenance homogeneous, but
/// position 0 now carries the second block's opening record where the grammar expects the first block's.
/// The stage reports a `BlockOutOfScheduleOrder` at sequence position 0 whose expected block is the first
/// seed block and whose observed block is the second. Both blocks' identities are captured from the
/// pristine seed order before the swap, so the expected/observed evidence is derived independently of the
/// stage's own decode.
#[test]
fn a_swapped_block_span_fails_the_schedule_order() {
    let mut fixture = CampaignFixture::valid();
    // Each run emits one manifest plus its ten doses; each block is its two matched runs.
    let run_records = 1 + DoseIndex::ALL.len();
    let block_records = 2 * run_records;
    // Capture the two blocks' identities from the pristine seed order before swapping their spans.
    let (first_run, first_block, _) = CampaignFixture::record_identity(&fixture.records()[0]);
    let (second_run, second_block, _) =
        CampaignFixture::record_identity(&fixture.records()[block_records]);

    let records = fixture.records_mut();
    for offset in 0..block_records {
        records.swap(offset, block_records + offset);
    }
    fixture.renumber();

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("a swapped block span must fail the schedule-order stage");
    };

    match error {
        IntegrityError::ScheduleOrderMismatch {
            fault:
                ScheduleOrderFault::BlockOutOfScheduleOrder {
                    position,
                    expected,
                    observed,
                },
            ..
        } => {
            assert_eq!(
                position,
                RecordSeq::new(0),
                "the first displaced position is sequence zero"
            );
            assert_eq!(
                expected.cell(),
                first_run.cell(),
                "the expected block is the first seed block's cell"
            );
            assert_eq!(
                expected.repetition_block(),
                first_block,
                "the expected block is the first seed block's index"
            );
            assert_eq!(
                observed.cell(),
                second_run.cell(),
                "the observed block is the second seed block's cell"
            );
            assert_eq!(
                observed.repetition_block(),
                second_block,
                "the observed block is the second seed block's index"
            );
        }
        other => panic!("expected BlockOutOfScheduleOrder at position zero, got {other:?}"),
    }
}
