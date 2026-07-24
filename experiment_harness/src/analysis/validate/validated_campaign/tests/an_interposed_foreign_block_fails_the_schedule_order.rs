//! Schedule-order stage — negative proof: interposing a foreign block's complete span between a block's
//! two paired runs breaks the pair's adjacency, forcing the foreign block's record into the position the
//! grammar reserves for the pair's second run — the earliest divergence is a `BlockOutOfScheduleOrder`.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::analysis::validate::schedule_order_fault::ScheduleOrderFault;
use crate::dataset::dose_index::DoseIndex;
use crate::observation::record_seq::RecordSeq;

/// Rotating the second block's complete span in front of the first block's second run leaves the first
/// block's two matched runs non-adjacent — its arm and control now straddle a whole foreign block — while
/// keeping every record valid, the census complete, and provenance homogeneous. At the first block's
/// second-run position (sequence 11) the grammar expects the first block and finds the second, so the
/// stage's earliest divergence is a `BlockOutOfScheduleOrder` there: expected the split pair's block,
/// observed the interposed foreign block. Both identities are captured from the pristine seed order.
#[test]
fn an_interposed_foreign_block_fails_the_schedule_order() {
    let mut fixture = CampaignFixture::valid();
    let run_records = 1 + DoseIndex::ALL.len();
    let block_records = 2 * run_records;
    // The split pair's displaced second run (first block) and the foreign block's leading run (second
    // block) that will occupy its position.
    let (paired_run, paired_block, _) =
        CampaignFixture::record_identity(&fixture.records()[run_records]);
    let (foreign_run, foreign_block, _) =
        CampaignFixture::record_identity(&fixture.records()[block_records]);

    // Rotate the second block's whole span ahead of the first block's second run: the range holding the
    // first block's second run followed by the entire second block, rotated left by one run, becomes the
    // second block followed by the first block's second run — the foreign block interposed within the pair.
    let records = fixture.records_mut();
    records[run_records..2 * block_records].rotate_left(run_records);
    fixture.renumber();

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("an interposed foreign block must fail the schedule-order stage");
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
                RecordSeq::new(run_records as u64),
                "the divergence is at the split pair's second-run position",
            );
            assert_eq!(
                expected.cell(),
                paired_run.cell(),
                "the expected block is the split pair's block cell"
            );
            assert_eq!(
                expected.repetition_block(),
                paired_block,
                "the expected block is the split pair's block index"
            );
            assert_eq!(
                observed.cell(),
                foreign_run.cell(),
                "the observed block is the interposed foreign block's cell"
            );
            assert_eq!(
                observed.repetition_block(),
                foreign_block,
                "the observed block is the interposed foreign block's index"
            );
        }
        other => panic!("expected BlockOutOfScheduleOrder at the second-run position, got {other:?}"),
    }
}
