//! Schedule-order stage — precedence proof: with a within-run slot defect at an early position and a whole
//! block-span swap at a later one, the stage returns the earliest-position fault, so position precedence
//! dominates the block-over-role-over-slot field priority that orders faults competing *within* one
//! position.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::analysis::validate::record_slot::RecordSlot;
use crate::analysis::validate::schedule_order_fault::ScheduleOrderFault;
use crate::dataset::dose_index::DoseIndex;
use crate::observation::record_seq::RecordSeq;

/// Two independent defects that both reach stage 6: the first run's manifest swapped past its first dose (a
/// within-run slot defect at sequence 0) and, further along, two whole block spans swapped (a block
/// defect). The field priority ranks a block defect above a slot defect, but that priority orders only
/// faults competing at one position; across positions the stage scans ascending and returns the first. So
/// it deterministically reports the sequence-0 `RunRecordOutOfScheduleOrder`, not the later, higher-priority
/// block defect.
#[test]
fn the_earliest_schedule_order_defect_outranks_a_later_one() {
    let mut fixture = CampaignFixture::valid();
    let run_records = 1 + DoseIndex::ALL.len();
    let block_records = 2 * run_records;

    let records = fixture.records_mut();
    // Early within-run slot defect at sequence 0: the first run's manifest after its first dose.
    records.swap(0, 1);
    // Later block defect: swap the third and fourth seed blocks' whole spans.
    for offset in 0..block_records {
        records.swap(2 * block_records + offset, 3 * block_records + offset);
    }
    fixture.renumber();

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("a campaign with two schedule-order defects must fail the stage");
    };

    match error {
        IntegrityError::ScheduleOrderMismatch {
            fault:
                ScheduleOrderFault::RunRecordOutOfScheduleOrder {
                    position,
                    expected,
                    observed,
                    ..
                },
            ..
        } => {
            assert_eq!(
                position,
                RecordSeq::new(0),
                "the returned defect is the earliest-position one, not the later block defect"
            );
            assert_eq!(
                expected,
                RecordSlot::Manifest,
                "the earlier defect is the misplaced manifest slot"
            );
            assert_eq!(
                observed,
                RecordSlot::Dose(DoseIndex::ALL[0]),
                "the observed slot is the first dose"
            );
        }
        other => panic!("expected the earliest-position RunRecordOutOfScheduleOrder, got {other:?}"),
    }
}
