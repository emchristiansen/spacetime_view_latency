//! Category proof: a dose observation whose ladder index is out of range fails the dose census with
//! typed range evidence.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::record_kind_dto::RecordKindDto;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::dose_census_fault::DoseCensusFault;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::params::NUM_DOSES;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// Dropping the first dose observation's ladder index below the valid `1..=NUM_DOSES` range fails the
/// per-record dose-range proof, so the fold fails with a typed `DoseCensus` located at the mutated run
/// and carrying the out-of-range dose and its expected inclusive range. Both the record kind's carried
/// dose and the body coordinate's dose are set to the same out-of-range zero, so the dose-range obligation
/// is the sole violated invariant — the record kind and body stay mutually consistent, isolating this
/// proof from the kind↔body dose-agreement check regardless of intra-record check order.
#[test]
fn an_out_of_range_dose_fails_the_dose_census() {
    let mut fixture = CampaignFixture::valid();
    let records = fixture.records_mut();
    // The second record is the cell0/arm/block0 dose-1 observation; drop both its record-kind dose and its
    // body coordinate dose to the same out-of-range zero, keeping the record internally consistent.
    match &mut records[1] {
        WireRecordDto::Dose { record, body } => {
            record.kind = RecordKindDto::Dose(0);
            body.observation.coordinate.dose = 0;
        }
        WireRecordDto::Manifest { .. } => panic!("the second record must be a dose observation"),
    }

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("an out-of-range dose must fail total validation");
    };

    match error {
        IntegrityError::DoseCensus {
            run,
            fault:
                DoseCensusFault::OutOfRange {
                    dose,
                    expected_min,
                    expected_max,
                },
            ..
        } => {
            assert_eq!(
                run,
                super::super::canonical_coordinate(Cell::all()[0], RunRole::Arm, 0),
                "the fault locates the mutated run"
            );
            assert_eq!(dose, 0, "the offending dose is the out-of-range index");
            assert_eq!(expected_min, 1, "the valid ladder starts at dose one");
            assert_eq!(expected_max, NUM_DOSES, "the valid ladder ends at NUM_DOSES");
        }
        other => panic!("expected DoseCensus OutOfRange, got {other:?}"),
    }
}
