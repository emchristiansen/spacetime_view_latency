//! Provenance-association proof: a per-run server fact travels to exactly the run it was recorded for.
//!
//! `TrustedRun::mint` derives both a run's coordinate and its `RunProvenance` from the *one* typed manifest
//! structurally bound to that run, so a per-run fact can never be paired with a foreign run's identity. The
//! server pid is a per-run fact the fold checks only for shape (nonzero), never homogeneity, so a distinct
//! pid on one run's manifest survives the fold and must surface on exactly that run's provenance — not its
//! matched sibling's.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::ingest::wire_record_dto::WireRecordDto;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// Stamping a distinct nonzero pid onto only the first cell's arm manifest leaves a well-formed campaign
/// (pid is a per-run shape fact, not a homogeneity fact), so the fold succeeds. The distinct pid then
/// appears on exactly that arm run's provenance, while its matched control — whose manifest was untouched —
/// keeps the fixture's default pid. Because `mint` derives coordinate and provenance from a single manifest,
/// this proves the per-run provenance is bound to the correct run rather than crossed with a neighbour.
#[test]
fn each_run_retains_its_own_server_provenance() {
    // A distinct nonzero pid, different from the fixture's shared default, stamped onto one run only.
    let distinct_pid = 9001u32;

    let mut fixture = CampaignFixture::valid();
    let cell = Cell::all()[0];
    let arm_index = fixture.manifest_index(cell, RunRole::Arm, 0);
    let control_index = fixture.manifest_index(cell, RunRole::Control, 0);

    // Capture the matched control's untouched pid, so the assertion proves the distinct pid did not bleed
    // onto the sibling rather than merely differing from a re-encoded literal.
    let control_pid = match &fixture.records()[control_index] {
        WireRecordDto::Manifest { body, .. } => body.manifest.server.pid,
        WireRecordDto::Dose { .. } => panic!("the located record must be a manifest"),
    };
    assert_ne!(
        distinct_pid, control_pid,
        "the distinct pid must differ from the fixture default to make the association observable"
    );

    match &mut fixture.records_mut()[arm_index] {
        WireRecordDto::Manifest { body, .. } => body.manifest.server.pid = distinct_pid,
        WireRecordDto::Dose { .. } => panic!("the located record must be a manifest"),
    }

    let campaign = ValidatedCampaign::from_records(fixture.into_records())
        .expect("a distinct per-run pid is a shape fact, not a homogeneity fact, so the campaign folds");

    // `cells[0]` is `Cell::all()[0]` (canonical order) and `blocks()[0]` is repetition block 0 (construction
    // order), so this matched block is the one whose arm manifest carried the distinct pid.
    let block = &campaign.cells[0].blocks()[0];
    assert_eq!(
        *block.arm().coordinate(),
        super::super::canonical_coordinate(cell, RunRole::Arm, 0),
        "the arm run sits at its canonical (cell, arm, block 0) coordinate"
    );
    assert_eq!(
        *block.control().coordinate(),
        super::super::canonical_coordinate(cell, RunRole::Control, 0),
        "the control run sits at its canonical (cell, control, block 0) coordinate"
    );
    assert_eq!(
        block.arm().provenance().pid().get(),
        distinct_pid,
        "the distinct per-run pid surfaces on exactly the arm run whose manifest recorded it"
    );
    assert_eq!(
        block.control().provenance().pid().get(),
        control_pid,
        "the matched control run keeps its own untouched pid — the per-run fact did not cross runs"
    );
}
