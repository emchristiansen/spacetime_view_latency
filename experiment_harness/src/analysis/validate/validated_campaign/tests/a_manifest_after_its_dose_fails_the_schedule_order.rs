//! Schedule-order stage — negative proof: placing a run's first dose ahead of its manifest keeps the
//! record in its block and role but violates the manifest-then-doses within-run order, a
//! `RunRecordOutOfScheduleOrder`.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::analysis::validate::record_slot::RecordSlot;
use crate::analysis::validate::schedule_order_fault::ScheduleOrderFault;
use crate::dataset::dose_index::DoseIndex;
use crate::observation::record_seq::RecordSeq;

/// Swapping a run's manifest with its first cumulative dose keeps both records in the same block and run —
/// so the block and role checks pass — but puts a dose observation at the run's first position where the
/// grammar expects the manifest, while the census stays complete and provenance homogeneous. The stage
/// reports a `RunRecordOutOfScheduleOrder` at sequence 0, located at the reordered run, whose expected slot
/// is the manifest and whose observed slot is the first dose. The run identity is captured from the
/// pristine manifest before the swap.
#[test]
fn a_manifest_after_its_dose_fails_the_schedule_order() {
    let mut fixture = CampaignFixture::valid();
    // records[0] is the first run's manifest, records[1] its first cumulative dose.
    let (first_run, ..) = CampaignFixture::record_identity(&fixture.records()[0]);

    fixture.records_mut().swap(0, 1);
    fixture.renumber();

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("a manifest after its first dose must fail the schedule-order stage");
    };

    match error {
        IntegrityError::ScheduleOrderMismatch {
            fault:
                ScheduleOrderFault::RunRecordOutOfScheduleOrder {
                    position,
                    run,
                    expected,
                    observed,
                },
            ..
        } => {
            assert_eq!(
                position,
                RecordSeq::new(0),
                "the misplaced manifest first differs at sequence zero"
            );
            assert_eq!(
                run, first_run,
                "the fault stays within the run whose slots were reordered"
            );
            assert_eq!(
                expected,
                RecordSlot::Manifest,
                "the grammar expects the manifest at the run's first position"
            );
            assert_eq!(
                observed,
                RecordSlot::Dose(DoseIndex::ALL[0]),
                "the observed slot is the first cumulative dose"
            );
        }
        other => panic!("expected RunRecordOutOfScheduleOrder at the run's first position, got {other:?}"),
    }
}
