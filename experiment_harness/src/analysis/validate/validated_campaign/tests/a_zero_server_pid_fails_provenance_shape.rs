//! Category proof: a manifest whose server pid is zero fails the stage-4 per-run provenance-shape check
//! with a typed zero-pid contradiction.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::analysis::validate::server_provenance_fault::ServerProvenanceFault;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// Zeroing the first manifest's server pid violates the nonzero-pid provenance shape, so the fold fails
/// with a typed `ServerProvenanceShape` whose zero-pid evidence carries the recorded zero at the
/// mutated manifest's run.
#[test]
fn a_zero_server_pid_fails_provenance_shape() {
    let mut fixture = CampaignFixture::valid();
    let records = fixture.records_mut();
    match &mut records[0] {
        WireRecordDto::Manifest { body, .. } => {
            body.manifest.server.pid = 0;
        }
        WireRecordDto::Dose { .. } => panic!("the first record must be a manifest"),
    }

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("a zero server pid must fail total validation");
    };

    match error {
        IntegrityError::ServerProvenanceShape {
            run,
            fault: ServerProvenanceFault::ZeroPid { observed },
            ..
        } => {
            assert_eq!(
                run,
                super::super::canonical_coordinate(Cell::all()[0], RunRole::Arm, 0),
                "the fault locates the mutated manifest's run"
            );
            assert_eq!(observed, 0, "the recorded pid is zero");
        }
        other => panic!("expected ServerProvenanceShape ZeroPid, got {other:?}"),
    }
}
