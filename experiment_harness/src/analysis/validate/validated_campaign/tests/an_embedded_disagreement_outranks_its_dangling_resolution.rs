//! First-error-order proof: within stage-3 binding, a manifest's embedded self-reference is proven to
//! agree with its own manifest identity before any reference→manifest resolution runs, so a manifest that
//! both disagrees internally *and* would leave its observations dangling fails with the embedded
//! disagreement, not the dangling reference.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::analysis::validate::manifest_reference_fault::ManifestReferenceFault;
use crate::analysis::validate::manifest_reference_identity::ManifestReferenceIdentity;
use crate::manifest::database_identity::DatabaseIdentity;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// Retagging one manifest's own module database identity to a different canonical identity makes its own
/// identity disagree with its (unchanged) embedded self-reference *and* orphans its ten observations,
/// whose references still name the original identity that no manifest now provides. The binding proves the
/// embedded self-agreement in its first per-manifest pass, before it resolves observation references
/// against manifests, so the fold returns the typed `EmbeddedDisagreement` — both identities located at
/// the mutated run and disagreeing — rather than the dangling-reference resolution that would otherwise
/// follow.
#[test]
fn an_embedded_disagreement_outranks_its_dangling_resolution() {
    // A canonical identity distinct from the fixture's shared all-zero identity, so the retagged manifest's
    // own identity no longer matches its embedded reference (or its observations' references).
    let divergent_identity = "1".repeat(DatabaseIdentity::CANONICAL_HEX_LEN);

    let mut fixture = CampaignFixture::valid();
    let records = fixture.records_mut();
    match &mut records[0] {
        WireRecordDto::Manifest { body, .. } => {
            body.manifest.module.database_identity = divergent_identity.clone();
        }
        WireRecordDto::Dose { .. } => panic!("the first record must be a manifest"),
    }

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("a manifest disagreeing with its embedded self-reference must fail total validation");
    };

    match error {
        IntegrityError::ManifestReferenceBinding {
            fault: ManifestReferenceFault::EmbeddedDisagreement { reference, manifest },
            ..
        } => {
            let run = super::super::canonical_coordinate(Cell::all()[0], RunRole::Arm, 0);
            let divergent = DatabaseIdentity::parse_canonical_hex(&divergent_identity)
                .expect("the divergent identity is canonical lowercase hex");
            // The embedded self-reference is unmutated: the fixture's canonical identity at this run and
            // the shared schedule seed.
            let expected_reference = ManifestReferenceIdentity::new(
                run.clone(),
                CampaignFixture::database_identity(),
                CampaignFixture::schedule_seed(),
            );
            // The manifest's own identity carries the divergent database component at the same run and
            // schedule seed — the disagreement is exactly and only the database component.
            let expected_manifest =
                ManifestReferenceIdentity::new(run, divergent, CampaignFixture::schedule_seed());
            assert_eq!(
                reference, expected_reference,
                "the embedded reference retains the fixture's canonical identity at the mutated run and \
                 the shared schedule seed"
            );
            assert_eq!(
                manifest, expected_manifest,
                "the manifest's own identity is the divergent database component at the same run and \
                 schedule seed"
            );
        }
        other => panic!(
            "expected ManifestReferenceBinding EmbeddedDisagreement ahead of a dangling resolution, got \
             {other:?}"
        ),
    }
}
