//! Category proof: a manifest whose module database identity is not canonical hex fails the stage-3
//! manifest-reference binding with a typed malformed-identity contradiction retaining the raw hex.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::analysis::validate::manifest_reference_fault::ManifestReferenceFault;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// Corrupting the first manifest's module database identity to an unparseable string leaves no trusted
/// identity to key the binding on, so the fold fails with a typed `ManifestReferenceBinding` whose
/// malformed-identity evidence locates the manifest's run and retains the exact wire hex.
#[test]
fn a_malformed_module_identity_fails_the_binding() {
    let malformed_hex = "not-canonical-hex".to_string();

    let mut fixture = CampaignFixture::valid();
    // Locate the cell0/arm/block0 manifest by identity; corrupt its module database identity.
    let index = fixture.manifest_index(Cell::all()[0], RunRole::Arm, 0);
    match &mut fixture.records_mut()[index] {
        WireRecordDto::Manifest { body, .. } => {
            body.manifest.module.database_identity = malformed_hex.clone();
        }
        WireRecordDto::Dose { .. } => panic!("the located record must be a manifest"),
    }

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("a malformed module database identity must fail total validation");
    };

    match error {
        IntegrityError::ManifestReferenceBinding {
            fault: ManifestReferenceFault::MalformedIdentity { run, hex, .. },
            ..
        } => {
            assert_eq!(
                run,
                super::super::canonical_coordinate(Cell::all()[0], RunRole::Arm, 0),
                "the fault locates the manifest whose identity failed to parse"
            );
            assert_eq!(
                hex, malformed_hex,
                "the fault retains the exact unparseable wire hex"
            );
        }
        other => panic!("expected ManifestReferenceBinding MalformedIdentity, got {other:?}"),
    }
}
