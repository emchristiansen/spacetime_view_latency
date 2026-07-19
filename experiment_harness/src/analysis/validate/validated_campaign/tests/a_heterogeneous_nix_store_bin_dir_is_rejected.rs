//! Category proof: a second manifest whose pinned nix-store bin dir diverges from the reference run's
//! fails the stage-4 homogeneity check with a typed nix-store-bin-dir contradiction.

use std::path::PathBuf;

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::analysis::validate::stable_fact_contradiction::StableFactContradiction;

/// Diverging the second manifest's pinned nix-store bin dir from the reference (first) manifest's leaves
/// a well-formed but non-homogeneous campaign, so the fold fails with a typed `CampaignFactHeterogeneity`
/// whose nix-store-bin-dir contradiction pairs the reference run's path against the diverging run's, and
/// whose run/reference_run locate the diverging and reference runs.
#[test]
fn a_heterogeneous_nix_store_bin_dir_is_rejected() {
    let divergent_bin_dir = "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-stdb/bin".to_string();

    let mut fixture = CampaignFixture::valid();
    // The fold's homogeneity reference is the first manifest in sequence order; the diverging run is the
    // second. Locate both by manifest ordinal and derive their expected run coordinates from the records
    // themselves — under the seed record order neither is a fixed canonical (cell, role, block).
    let reference_index = fixture.nth_manifest_index(0);
    let diverging_index = fixture.nth_manifest_index(1);
    let (expected_reference_run, ..) =
        CampaignFixture::record_identity(&fixture.records()[reference_index]);
    let (expected_diverging_run, ..) =
        CampaignFixture::record_identity(&fixture.records()[diverging_index]);
    // Capture the reference run's pinned bin dir before diverging the second manifest's.
    let reference_bin_dir = match &fixture.records()[reference_index] {
        WireRecordDto::Manifest { body, .. } => body.manifest.distribution.nix_store_bin_dir.clone(),
        WireRecordDto::Dose { .. } => panic!("the located record must be a manifest"),
    };

    match &mut fixture.records_mut()[diverging_index] {
        WireRecordDto::Manifest { body, .. } => {
            body.manifest.distribution.nix_store_bin_dir = divergent_bin_dir.clone();
        }
        WireRecordDto::Dose { .. } => panic!("the located record must be a manifest"),
    }

    let Err(error) = ValidatedCampaign::from_records(fixture.into_records()) else {
        panic!("a heterogeneous pinned bin dir must fail total validation");
    };

    match error {
        IntegrityError::CampaignFactHeterogeneity {
            run,
            reference_run,
            contradiction: StableFactContradiction::NixStoreBinDir { expected, observed },
            ..
        } => {
            assert_eq!(
                run, expected_diverging_run,
                "the diverging run is the second manifest's run"
            );
            assert_eq!(
                reference_run, expected_reference_run,
                "the reference run is the first manifest's run"
            );
            assert_eq!(
                expected,
                PathBuf::from(reference_bin_dir),
                "the expected path is the reference run's pinned bin dir"
            );
            assert_eq!(
                observed,
                PathBuf::from(divergent_bin_dir),
                "the observed path is the diverging run's value"
            );
        }
        other => panic!("expected CampaignFactHeterogeneity NixStoreBinDir, got {other:?}"),
    }
}
