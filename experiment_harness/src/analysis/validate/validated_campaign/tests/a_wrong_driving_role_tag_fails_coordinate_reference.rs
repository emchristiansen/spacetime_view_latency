//! Category proof: a dose observation whose driving-role tag is not the cell's canonical tag fails the
//! per-record coordinate-reference proof with typed expected/observed evidence.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::coordinate_reference_fault::CoordinateReferenceFault;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::dataset::dose_index::DoseIndex;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// Replacing the first dose observation's driving-role tag with a non-canonical string fails the
/// per-record tag proof, so the fold fails with a typed `CoordinateReferenceMismatch` whose driving-role
/// evidence pairs the cell's expected driving role against the corrupted wire tag at the mutated run.
#[test]
fn a_wrong_driving_role_tag_fails_coordinate_reference() {
    let wrong_tag = "not-a-role".to_string();
    let expected_role = Cell::all()[0].growth_regime().driving_role();

    let mut fixture = CampaignFixture::valid();
    // Locate the cell0/arm/block0 dose-1 observation by identity; corrupt its driving-role tag.
    let index = fixture.dose_index(Cell::all()[0], RunRole::Arm, 0, DoseIndex::ALL[0]);
    match &mut fixture.records_mut()[index] {
        WireRecordDto::Dose { body, .. } => {
            body.observation.coordinate.driving_role_tag = wrong_tag.clone();
        }
        WireRecordDto::Manifest { .. } => panic!("the located record must be a dose observation"),
    }

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("a non-canonical driving-role tag must fail total validation");
    };

    match error {
        IntegrityError::CoordinateReferenceMismatch {
            fault:
                CoordinateReferenceFault::DrivingRoleTag {
                    run,
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
            assert_eq!(
                expected, expected_role,
                "the expected role is the cell's driving role"
            );
            assert_eq!(observed, wrong_tag, "the observed tag is the corrupted wire string");
        }
        other => panic!("expected CoordinateReferenceMismatch DrivingRoleTag, got {other:?}"),
    }
}
