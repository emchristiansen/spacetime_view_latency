//! Schedule-order stage — negative proof: reversing a block's two matched run spans keeps both runs in
//! their block but presents them in the opposite role order to the seed-selected one, a
//! `RoleOutOfScheduleOrder`.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::analysis::validate::schedule_order_fault::ScheduleOrderFault;
use crate::dataset::dose_index::DoseIndex;
use crate::observation::record_seq::RecordSeq;

/// Swapping the first block's two adjacent run spans — both belong to that same block, so the block check
/// still passes — presents the run whose role the seed schedule places second at the first run's position,
/// leaving every record valid, the census complete, and provenance homogeneous. The stage reports a
/// `RoleOutOfScheduleOrder` at sequence 0, located in the first block, whose expected role is the
/// seed-selected first role and whose observed role is the second. The block and both roles are captured
/// from the pristine seed order.
#[test]
fn an_opposite_role_order_fails_the_schedule_order() {
    let mut fixture = CampaignFixture::valid();
    let run_records = 1 + DoseIndex::ALL.len();
    let (first_run, first_block, _) = CampaignFixture::record_identity(&fixture.records()[0]);
    let (second_run, ..) = CampaignFixture::record_identity(&fixture.records()[run_records]);

    let records = fixture.records_mut();
    for offset in 0..run_records {
        records.swap(offset, run_records + offset);
    }
    fixture.renumber();

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("an opposite run order must fail the schedule-order stage");
    };

    match error {
        IntegrityError::ScheduleOrderMismatch {
            fault:
                ScheduleOrderFault::RoleOutOfScheduleOrder {
                    position,
                    block,
                    expected,
                    observed,
                },
            ..
        } => {
            assert_eq!(
                position,
                RecordSeq::new(0),
                "the reversed order first differs at sequence zero"
            );
            assert_eq!(
                block.cell(),
                first_run.cell(),
                "the fault stays within the first block's cell"
            );
            assert_eq!(
                block.repetition_block(),
                first_block,
                "the fault stays within the first block's index"
            );
            assert_eq!(
                expected,
                first_run.role(),
                "the expected role is the seed-selected first role"
            );
            assert_eq!(
                observed,
                second_run.role(),
                "the observed role is the seed-selected second role"
            );
        }
        other => panic!("expected RoleOutOfScheduleOrder in the first block, got {other:?}"),
    }
}
