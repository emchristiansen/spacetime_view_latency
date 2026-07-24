//! The success proof: the complete, spec-correct synthetic campaign folds into the full trusted graph.
//!
//! It asserts the entire `ValidatedCampaign → CellDataset → MatchedBlock → TrustedRun<R> → TrustedDose`
//! shape: nine cells in canonical order, each with thirty matched blocks, each block's arm and control
//! at their canonical `(cell, role, block)` coordinates, and each run carrying all ten doses in
//! canonical ladder order.

use super::campaign_builder::CampaignFixture;
use super::super::ValidatedCampaign;
use crate::analysis::validate::trusted_dose::TrustedDose;
use crate::analysis::validate::trusted_run::TrustedRun;
use crate::dataset::dose_index::DoseIndex;
use crate::params::REPETITION_BLOCKS;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// The complete, spec-correct campaign folds into the full trusted graph. The trusted graph's fixed arrays
/// are heap-owned at every level (`ValidatedCampaign` → `CellDataset` → `TrustedRun`), so the fold's
/// by-value footprint is bounded by a small constant and this folds directly on the ordinary test thread —
/// no ample-stack accommodation is needed.
#[test]
fn a_complete_valid_campaign_folds() {
    let campaign = ValidatedCampaign::from_records(CampaignFixture::valid().into_records())
        .expect("the complete, spec-correct campaign must fold without an integrity error");

    let all_cells = Cell::all();
    assert_eq!(
        campaign.cells.len(),
        all_cells.len(),
        "the campaign carries exactly the nine preregistered cells"
    );

    for (cell_index, dataset) in campaign.cells.iter().enumerate() {
        let cell = all_cells[cell_index];
        assert_eq!(
            dataset.cell(),
            cell,
            "cells are minted in canonical Cell::all() order"
        );

        let blocks = dataset.blocks();
        assert_eq!(
            blocks.len(),
            REPETITION_BLOCKS as usize,
            "each cell carries its full thirty-block matched sample"
        );

        for (block_index, block) in blocks.iter().enumerate() {
            assert_run(block.arm(), cell, RunRole::Arm, block_index);
            assert_run(block.control(), cell, RunRole::Control, block_index);
        }
    }
}

/// Assert a run sits at its canonical `(cell, role, block)` coordinate and carries all ten doses in
/// canonical ladder order. Generic over the run's role marker so it proves both the arm and the control.
fn assert_run<R>(run: &TrustedRun<R>, cell: Cell, role: RunRole, block_index: usize) {
    assert_eq!(
        *run.coordinate(),
        super::super::canonical_coordinate(cell, role, block_index),
        "the run sits at its canonical (cell, role, block) coordinate"
    );

    let doses: &[TrustedDose] = run.doses();
    assert_eq!(doses.len(), DoseIndex::ALL.len(), "the run carries all ten doses");
    for (dose_index, dose) in doses.iter().enumerate() {
        assert_eq!(
            dose.dose(),
            DoseIndex::ALL[dose_index],
            "doses appear in canonical 1..=NUM_DOSES ladder order"
        );
    }
}
