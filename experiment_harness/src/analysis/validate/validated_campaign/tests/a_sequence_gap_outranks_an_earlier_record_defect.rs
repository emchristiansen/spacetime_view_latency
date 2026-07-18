//! First-error-order proof: stage-1 sequence contiguity is proven before the stage-2 per-record proofs,
//! so a campaign carrying both a sequence gap and an earlier record's own-coordinate defect fails with the
//! sequence contiguity error, not the record-coordinate one.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::dataset::dose_index::DoseIndex;

/// A campaign with an earlier record's off-formula `logical_n` (a stage-2 per-record defect) *and* a
/// later record's out-of-range sequence (a stage-1 discontinuity) fails with `SequenceNotContiguous`: the
/// fold proves contiguity across every record before it proves any record's own coordinate, so the
/// stage-1 failure is deterministically returned even though the stage-2 defect sits at an earlier
/// sequence position.
#[test]
fn a_sequence_gap_outranks_an_earlier_record_defect() {
    let mut fixture = CampaignFixture::valid();
    let total = fixture.records().len();
    let gap_seq = total as u64;
    let corrupted_logical_n = DoseIndex::ALL[0].cumulative_driving_rows() + 1;

    let records = fixture.records_mut();
    // Earlier per-record defect: corrupt the cell0/arm/block0 dose-1 observation's cumulative logical
    // count off its preregistered `dose * BATCH_SIZE` value — a stage-2 record-coordinate contradiction.
    match &mut records[1] {
        WireRecordDto::Dose { body, .. } => {
            body.observation.coordinate.logical_n = corrupted_logical_n;
        }
        WireRecordDto::Manifest { .. } => panic!("the second record must be a dose observation"),
    }
    // Later sequence discontinuity: push the highest-sequence record one past the end — a stage-1 gap.
    let last = records.len() - 1;
    match &mut records[last] {
        WireRecordDto::Manifest { record, .. } => record.seq = gap_seq,
        WireRecordDto::Dose { record, .. } => record.seq = gap_seq,
    }

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("a campaign with a sequence gap and a record defect must fail total validation");
    };

    match error {
        IntegrityError::SequenceNotContiguous {
            position,
            found_seq,
            total: reported_total,
            ..
        } => {
            assert_eq!(
                position,
                total - 1,
                "the gap surfaces at the final sorted position, ahead of the earlier record defect"
            );
            assert_eq!(
                found_seq, gap_seq,
                "the offending record carried the out-of-range sequence"
            );
            assert_eq!(
                reported_total, total,
                "the reported total is the record count"
            );
        }
        other => panic!("expected SequenceNotContiguous ahead of the record defect, got {other:?}"),
    }
}
