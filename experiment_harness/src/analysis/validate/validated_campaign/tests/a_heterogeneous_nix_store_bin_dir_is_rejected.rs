//! Category proof: a second manifest whose pinned nix-store bin dir diverges from the reference run's
//! fails the stage-4 homogeneity check with a typed nix-store-bin-dir contradiction.

use std::path::PathBuf;

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::analysis::validate::integrity_error::IntegrityError;
use crate::analysis::validate::stable_fact_contradiction::StableFactContradiction;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// Diverging the second manifest's pinned nix-store bin dir from the reference (first) manifest's leaves
/// a well-formed but non-homogeneous campaign, so the fold fails with a typed `CampaignFactHeterogeneity`
/// whose nix-store-bin-dir contradiction pairs the reference run's path against the diverging run's, and
/// whose run/reference_run locate the diverging and reference runs.
#[test]
fn a_heterogeneous_nix_store_bin_dir_is_rejected() {
    let divergent_bin_dir = "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-stdb/bin".to_string();

    let mut fixture = CampaignFixture::valid();
    // Capture the reference run's pinned bin dir (the first manifest) before diverging a later one.
    let reference_bin_dir = match &fixture.records()[0] {
        WireRecordDto::Manifest { body, .. } => body.manifest.distribution.nix_store_bin_dir.clone(),
        WireRecordDto::Dose { .. } => panic!("the first record must be a manifest"),
    };

    // The second manifest is the cell0/block0/control run: one manifest plus its ten doses precede it.
    let records = fixture.records_mut();
    match &mut records[11] {
        WireRecordDto::Manifest { body, .. } => {
            body.manifest.distribution.nix_store_bin_dir = divergent_bin_dir.clone();
        }
        WireRecordDto::Dose { .. } => panic!("record 11 must be the second manifest"),
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
                run,
                super::super::canonical_coordinate(Cell::all()[0], RunRole::Control, 0),
                "the diverging run is the second manifest's control run"
            );
            assert_eq!(
                reference_run,
                super::super::canonical_coordinate(Cell::all()[0], RunRole::Arm, 0),
                "the reference run is the first manifest's arm run"
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
