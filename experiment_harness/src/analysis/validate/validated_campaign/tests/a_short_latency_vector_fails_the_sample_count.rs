//! Category proof: a dose observation whose raw latency vector is short of BATCH_SIZE fails the
//! per-record sample-count proof with typed count evidence.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::dataset::dose_index::DoseIndex;
use crate::params::BATCH_SIZE_USIZE;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// Dropping one sample from the first dose observation's latency vector fails the exact-sample-count
/// proof, so the fold fails with a typed `SampleCount` pairing the required BATCH_SIZE against the short
/// observed count at the mutated run and dose.
#[test]
fn a_short_latency_vector_fails_the_sample_count() {
    let mut fixture = CampaignFixture::valid();
    let records = fixture.records_mut();
    match &mut records[1] {
        WireRecordDto::Dose { body, .. } => {
            body.observation
                .latencies
                .pop()
                .expect("the dose carries BATCH_SIZE samples, so one can be dropped");
        }
        WireRecordDto::Manifest { .. } => panic!("the second record must be a dose observation"),
    }

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("a short latency vector must fail total validation");
    };

    match error {
        IntegrityError::SampleCount {
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
            assert_eq!(expected, BATCH_SIZE_USIZE, "the required count is BATCH_SIZE");
            assert_eq!(
                observed,
                BATCH_SIZE_USIZE - 1,
                "the observed count is one short of BATCH_SIZE"
            );
        }
        other => panic!("expected SampleCount, got {other:?}"),
    }
}
