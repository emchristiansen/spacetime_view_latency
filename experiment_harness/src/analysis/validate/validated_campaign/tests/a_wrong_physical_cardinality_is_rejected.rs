//! Category proof: a dose observation whose recorded physical cardinality disagrees with the
//! deterministic dataset expectation fails the per-record cardinality proof with typed evidence.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::physical_cardinalities_dto::PhysicalCardinalitiesDto;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// Bumping the first dose observation's recorded message-row count off the deterministic
/// `PhysicalCardinalities::expected` footprint fails the per-record cardinality proof, so the fold fails
/// with a typed `PhysicalCardinalityMismatch` pairing the deterministic expectation against the corrupted
/// wire value at the mutated run and dose.
#[test]
fn a_wrong_physical_cardinality_is_rejected() {
    let mut fixture = CampaignFixture::valid();
    // Locate the cell0/arm/block0 dose-1 observation by identity; bump its message-row count.
    let index = fixture.dose_index(Cell::all()[0], RunRole::Arm, 0, DoseIndex::ALL[0]);
    match &mut fixture.records_mut()[index] {
        WireRecordDto::Dose { body, .. } => match &mut body.observation.cardinalities {
            PhysicalCardinalitiesDto::Message(rows) => rows.message_rows += 1,
            PhysicalCardinalitiesDto::Chronicle(rows) => rows.message_rows += 1,
        },
        WireRecordDto::Manifest { .. } => panic!("the located record must be a dose observation"),
    }
    // Capture the corrupted wire value the fold will report as observed.
    let observed_dto = match &fixture.records()[index] {
        WireRecordDto::Dose { body, .. } => body.observation.cardinalities,
        WireRecordDto::Manifest { .. } => panic!("the located record must be a dose observation"),
    };

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("a wrong physical cardinality must fail total validation");
    };

    match error {
        IntegrityError::PhysicalCardinalityMismatch {
            run,
            dose,
            expected,
            observed,
            ..
        } => {
            assert_eq!(
                run,
                super::super::canonical_coordinate(Cell::all()[0], RunRole::Arm, 0),
                "the fault locates the mutated run"
            );
            assert_eq!(dose, DoseIndex::ALL[0], "the fault names the mutated dose");
            assert_eq!(
                expected,
                PhysicalCardinalities::expected(Cell::all()[0], DoseIndex::ALL[0]),
                "the expected footprint is the deterministic dataset cardinality"
            );
            assert_eq!(
                observed, observed_dto,
                "the observed footprint is the corrupted wire value"
            );
        }
        other => panic!("expected PhysicalCardinalityMismatch, got {other:?}"),
    }
}
