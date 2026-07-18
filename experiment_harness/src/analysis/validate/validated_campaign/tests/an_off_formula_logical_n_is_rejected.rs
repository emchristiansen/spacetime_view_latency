//! Category proof: a dose observation whose cumulative logical count is off the preregistered
//! `dose * BATCH_SIZE` formula fails with a typed record-coordinate contradiction.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::analysis::validate::record_coordinate_fault::RecordCoordinateFault;
use crate::dataset::dose_index::DoseIndex;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// Corrupting the first dose observation's cumulative logical count to one past its preregistered
/// `dose * BATCH_SIZE` value fails the per-record own-coordinate proof, so the fold fails with a typed
/// `RecordCoordinateMismatch` whose off-formula evidence pairs the expected and observed counts at the
/// mutated run and dose.
#[test]
fn an_off_formula_logical_n_is_rejected() {
    let expected_logical_n = DoseIndex::ALL[0].cumulative_driving_rows();
    let corrupted_logical_n = expected_logical_n + 1;

    let mut fixture = CampaignFixture::valid();
    let records = fixture.records_mut();
    // The second record is the cell0/arm/block0 dose-1 observation; corrupt its cumulative logical count.
    match &mut records[1] {
        WireRecordDto::Dose { body, .. } => {
            body.observation.coordinate.logical_n = corrupted_logical_n;
        }
        WireRecordDto::Manifest { .. } => panic!("the second record must be a dose observation"),
    }

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("an off-formula logical_n must fail total validation");
    };

    match error {
        IntegrityError::RecordCoordinateMismatch {
            fault:
                RecordCoordinateFault::OffFormulaLogicalN {
                    run,
                    dose,
                    expected,
                    observed,
                },
            ..
        } => {
            assert_eq!(
                run,
                super::super::canonical_coordinate(Cell::all()[0], RunRole::Arm, 0),
                "the fault locates the mutated run"
            );
            assert_eq!(dose, DoseIndex::ALL[0], "the fault names the mutated dose");
            assert_eq!(
                expected, expected_logical_n,
                "the expected count is the preregistered dose*BATCH_SIZE"
            );
            assert_eq!(
                observed, corrupted_logical_n,
                "the observed count is the corrupted value"
            );
        }
        other => panic!("expected RecordCoordinateMismatch OffFormulaLogicalN, got {other:?}"),
    }
}
