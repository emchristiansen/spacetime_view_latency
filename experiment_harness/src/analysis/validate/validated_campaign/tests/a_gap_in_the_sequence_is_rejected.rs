//! Category proof: a gap in the record sequence fails stage-1 contiguity with typed position evidence.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::integrity_error::IntegrityError;

/// Pushing the highest-sequence record one past the end leaves the set non-contiguous: the record at the
/// final sorted position no longer equals its index, so the fold fails with a typed
/// `SequenceNotContiguous` locating that position and the offending sequence.
#[test]
fn a_gap_in_the_sequence_is_rejected() {
    let mut fixture = CampaignFixture::valid();
    let total = fixture.records().len();
    let gap_seq = total as u64;

    let records = fixture.records_mut();
    let last = records.len() - 1;
    match &mut records[last] {
        WireRecordDto::Manifest { record, .. } => record.seq = gap_seq,
        WireRecordDto::Dose { record, .. } => record.seq = gap_seq,
    }

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("a non-contiguous record sequence must fail total validation");
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
                "the gap surfaces at the final sorted position"
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
        other => panic!("expected SequenceNotContiguous, got {other:?}"),
    }
}
