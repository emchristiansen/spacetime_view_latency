//! Category proof: a dose observation whose event counts break the insert/delete/net-delta identity
//! fails the per-record event proof with typed net-delta evidence.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::dataset::dose_index::DoseIndex;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// Bumping the first dose observation's insert count breaks the `inserts − deletes = net delta` identity
/// against the unchanged delivered delta, so the fold fails with a typed `EventIdentityViolation` whose
/// evidence carries the corrupted counts, the implied net delta, and the recorded net delta at the
/// mutated run and dose.
#[test]
fn a_broken_event_identity_is_rejected() {
    let mut fixture = CampaignFixture::valid();
    let records = fixture.records_mut();
    match &mut records[1] {
        WireRecordDto::Dose { body, .. } => {
            body.observation.events.inserts += 1;
        }
        WireRecordDto::Manifest { .. } => panic!("the second record must be a dose observation"),
    }
    // Capture the corrupted wire evidence the fold will report as observed.
    let observed_events = match &fixture.records()[1] {
        WireRecordDto::Dose { body, .. } => body.observation.events,
        WireRecordDto::Manifest { .. } => panic!("the second record must be a dose observation"),
    };

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("a broken event identity must fail total validation");
    };

    match error {
        IntegrityError::EventIdentityViolation {
            run,
            dose,
            observed,
            expected_net_delta,
            observed_net_delta,
            ..
        } => {
            assert_eq!(
                run,
                super::super::canonical_coordinate(Cell::all()[0], RunRole::Arm, 0),
                "the fault locates the mutated run"
            );
            assert_eq!(dose, DoseIndex::ALL[0], "the fault names the mutated dose");
            assert_eq!(
                observed, observed_events,
                "the reported evidence is the corrupted wire counts"
            );
            assert_eq!(
                expected_net_delta,
                i128::from(observed_events.inserts) - i128::from(observed_events.deletes),
                "the implied net delta is inserts − deletes"
            );
            assert_eq!(
                observed_net_delta, observed_events.delivered_net_row_delta,
                "the recorded net delta is the wire delivered delta"
            );
        }
        other => panic!("expected EventIdentityViolation, got {other:?}"),
    }
}
