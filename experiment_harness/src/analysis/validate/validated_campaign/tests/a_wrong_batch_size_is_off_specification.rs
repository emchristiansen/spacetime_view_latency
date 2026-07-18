//! Category proof: a manifest whose batch-size parameter is off the frozen constant fails the stage-4
//! off-specification check with a typed batch-size contradiction.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::analysis::validate::malformed_stable_fault::MalformedStableFault;
use crate::analysis::validate::stable_fact_contradiction::StableFactContradiction;
use crate::params::BATCH_SIZE;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// Pushing the first manifest's batch-size parameter one past the preregistered constant fails the
/// off-specification check, so the fold fails with a typed `MalformedStableProvenance` whose
/// off-specification batch-size contradiction pairs the frozen constant against the manifest's value.
#[test]
fn a_wrong_batch_size_is_off_specification() {
    let corrupted_batch_size = BATCH_SIZE + 1;

    let mut fixture = CampaignFixture::valid();
    let records = fixture.records_mut();
    match &mut records[0] {
        WireRecordDto::Manifest { body, .. } => {
            body.manifest.parameters.batch_size = corrupted_batch_size;
        }
        WireRecordDto::Dose { .. } => panic!("the first record must be a manifest"),
    }

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("an off-specification batch size must fail total validation");
    };

    match error {
        IntegrityError::MalformedStableProvenance {
            run,
            fault:
                MalformedStableFault::OffSpecification(StableFactContradiction::BatchSize {
                    expected,
                    observed,
                }),
            ..
        } => {
            assert_eq!(
                run,
                super::super::canonical_coordinate(Cell::all()[0], RunRole::Arm, 0),
                "the fault locates the mutated manifest's run"
            );
            assert_eq!(expected, BATCH_SIZE, "the expected value is the frozen constant");
            assert_eq!(
                observed, corrupted_batch_size,
                "the observed value is the corrupted parameter"
            );
        }
        other => panic!("expected MalformedStableProvenance OffSpecification BatchSize, got {other:?}"),
    }
}
