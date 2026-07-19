//! Category proof: a manifest whose repetition block is out of range fails block-census coordinate
//! construction with typed range evidence.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::block_census_fault::BlockCensusFault;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::params::REPETITION_BLOCKS;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// Pushing the first manifest's repetition block one past the valid range makes trusted
/// `RepetitionBlockIndex` construction fail during the per-record proofs, so the fold fails with a typed
/// `BlockCensus` located at the mutated `(cell, role)` and carrying the out-of-range block and its
/// expected inclusive range.
#[test]
fn an_out_of_range_block_fails_the_block_census() {
    let mut fixture = CampaignFixture::valid();
    // Locate the cell0/arm/block0 manifest by identity; push its block one past the valid range.
    let index = fixture.manifest_index(Cell::all()[0], RunRole::Arm, 0);
    match &mut fixture.records_mut()[index] {
        WireRecordDto::Manifest { body, .. } => {
            body.manifest.run.repetition_block = REPETITION_BLOCKS;
        }
        WireRecordDto::Dose { .. } => panic!("the located record must be a manifest"),
    }

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("an out-of-range repetition block must fail total validation");
    };

    match error {
        IntegrityError::BlockCensus {
            cell,
            role,
            fault:
                BlockCensusFault::OutOfRange {
                    block,
                    expected_min,
                    expected_max,
                },
            ..
        } => {
            assert_eq!(cell, Cell::all()[0], "the fault locates the mutated cell");
            assert_eq!(role, RunRole::Arm, "the mutated manifest is the arm run");
            assert_eq!(
                block, REPETITION_BLOCKS,
                "the offending block is the out-of-range index"
            );
            assert_eq!(expected_min, 0, "the valid range starts at block zero");
            assert_eq!(
                expected_max,
                REPETITION_BLOCKS - 1,
                "the valid range ends at the last block"
            );
        }
        other => panic!("expected BlockCensus OutOfRange, got {other:?}"),
    }
}
