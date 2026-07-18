//! Category proof: a dose observation whose recorded summary median disagrees with the R-1 recomputation
//! fails the per-record summary proof with typed recomputed/recorded evidence.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::dataset::dose_index::DoseIndex;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// Bumping the first dose observation's recorded median off its true R-1 recomputation fails the summary
/// proof, so the fold fails with a typed `SummaryRecomputationMismatch` whose recomputed value matches the
/// fixture's true summary and whose recorded value is the corrupted wire summary, at the mutated run and
/// dose.
#[test]
fn a_bumped_summary_median_is_rejected() {
    let mut fixture = CampaignFixture::valid();
    // The fixture's recorded summary equals the true R-1 recomputation, so capture it before corrupting.
    let true_summary = match &fixture.records()[1] {
        WireRecordDto::Dose { body, .. } => body.observation.summary,
        WireRecordDto::Manifest { .. } => panic!("the second record must be a dose observation"),
    };

    let records = fixture.records_mut();
    match &mut records[1] {
        WireRecordDto::Dose { body, .. } => {
            body.observation.summary.median_nanos += 1;
        }
        WireRecordDto::Manifest { .. } => panic!("the second record must be a dose observation"),
    }
    // Capture the corrupted wire summary the fold will report as recorded.
    let recorded_dto = match &fixture.records()[1] {
        WireRecordDto::Dose { body, .. } => body.observation.summary,
        WireRecordDto::Manifest { .. } => panic!("the second record must be a dose observation"),
    };

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("a bumped summary median must fail total validation");
    };

    match error {
        IntegrityError::SummaryRecomputationMismatch {
            run,
            dose,
            recomputed,
            recorded,
            ..
        } => {
            assert_eq!(
                run,
                super::super::canonical_coordinate(Cell::all()[0], RunRole::Arm, 0),
                "the fault locates the mutated run"
            );
            assert_eq!(dose, DoseIndex::ALL[0], "the fault names the mutated dose");
            assert_eq!(
                recomputed.median_nanos(),
                true_summary.median_nanos,
                "the recomputation matches the fixture's true median"
            );
            assert_eq!(
                recomputed.iqr_nanos(),
                true_summary.iqr_nanos,
                "the recomputation matches the fixture's true IQR"
            );
            assert_eq!(
                recorded, recorded_dto,
                "the recorded summary is the corrupted wire value"
            );
        }
        other => panic!("expected SummaryRecomputationMismatch, got {other:?}"),
    }
}
