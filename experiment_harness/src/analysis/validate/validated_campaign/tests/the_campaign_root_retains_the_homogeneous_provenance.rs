//! Provenance-retention proof: the campaign root retains the homogeneous distribution/module facts.
//!
//! The fold proves the pinned distribution paths, resolved executable, versions, release commit, and module
//! WASM digest identical across every run, then mints one `CampaignProvenance` from the sequence-first
//! reference manifest. This proof folds the spec-correct campaign and asserts the root provenance carries
//! exactly the reference manifest's already-parsed typed facts — the projection, not a Phase-1 `todo!()`.

use std::path::Path;

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::manifest::release_commit::ReleaseCommit;
use crate::manifest::wasm_sha256::WasmSha256;

/// The complete campaign folds, and its root `CampaignProvenance` carries the sequence-first reference
/// manifest's homogeneous facts: the pinned nix-store bin dir, CLI and standalone executables, the resolved
/// standalone executable, the parsed release commit, and the committed module WASM digest — each equal to
/// the reference manifest's recorded value, parsed into its domain type.
#[test]
fn the_campaign_root_retains_the_homogeneous_provenance() {
    let fixture = CampaignFixture::valid();

    // The campaign provenance is minted from the first manifest in sequence order; capture that reference
    // manifest's recorded facts before surrendering the records to the fold.
    let reference_index = fixture.nth_manifest_index(0);
    let (nix_store_bin_dir, cli_exe, standalone_exe, resolved_exe, release_commit_hex, wasm_hex) =
        match &fixture.records()[reference_index] {
            WireRecordDto::Manifest { body, .. } => {
                let distribution = &body.manifest.distribution;
                (
                    distribution.nix_store_bin_dir.clone(),
                    distribution.cli_exe.clone(),
                    distribution.standalone_exe.clone(),
                    body.manifest.server.resolved_exe.clone(),
                    distribution.cli_release_commit.clone(),
                    body.manifest.module.wasm_sha256.clone(),
                )
            }
            WireRecordDto::Dose { .. } => panic!("the located record must be a manifest"),
        };

    let campaign = ValidatedCampaign::from_records(fixture.into_records())
        .expect("the complete, spec-correct campaign folds without an integrity error");
    let provenance = campaign.provenance();

    assert_eq!(
        provenance.nix_store_bin_dir(),
        Path::new(&nix_store_bin_dir),
        "the root retains the reference manifest's pinned nix-store bin dir"
    );
    assert_eq!(
        provenance.cli_exe(),
        Path::new(&cli_exe),
        "the root retains the reference manifest's CLI executable path"
    );
    assert_eq!(
        provenance.standalone_exe(),
        Path::new(&standalone_exe),
        "the root retains the reference manifest's standalone executable path"
    );
    assert_eq!(
        provenance.resolved_exe(),
        Path::new(&resolved_exe),
        "the root retains the reference manifest's resolved standalone executable"
    );
    assert_eq!(
        provenance.cli_release_commit(),
        ReleaseCommit::parse_canonical_hex(&release_commit_hex)
            .expect("the fixture release commit is canonical lowercase hex"),
        "the root retains the reference manifest's parsed release commit"
    );
    assert_eq!(
        provenance.wasm_sha256(),
        WasmSha256::parse_canonical_hex(&wasm_hex)
            .expect("the fixture module digest is canonical lowercase hex"),
        "the root retains the reference manifest's parsed module WASM digest"
    );
}
