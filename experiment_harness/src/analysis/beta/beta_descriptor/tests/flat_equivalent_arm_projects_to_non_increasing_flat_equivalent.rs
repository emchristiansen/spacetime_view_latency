//! Dataset-driven gate proof: a control-valid cell whose arm total change lies *within* the frozen band
//! projects to [`ClassifiedCell::NonIncreasing`] with a [`NonIncreasingResponse::FlatEquivalent`] response
//! and **no** secondary β descriptor (spec: "implement and invoke the β descriptor only for cells whose
//! gated primary result is Increasing"). The whole projection is driven from a `&CellDataset` through the
//! sole crate-visible entry [`BetaDescriptor::project`], which classifies that same dataset internally.
//!
//! The control is flat at [`CONTROL_FLAT_NANOS`], so δ = [`FLAT_CONTROL_DELTA`] = 200 000 ns and the
//! control-validity gate passes. The arm rises 10 000 ns per dose step, so each of the 30 identical blocks
//! has an exact total change of `9 · 10 000 = 90 000` ns; the degenerate `[90 000, 90 000]` interval lies
//! within `[-δ, +δ]`, so the arm response is Flat-equivalent — a positive but practically-flat arm.

use super::{arm_median, CONTROL_FLAT_NANOS, FLAT_CONTROL_DELTA};
use crate::analysis::beta::beta_descriptor::BetaDescriptor;
use crate::analysis::classify::classified_cell::ClassifiedCell;
use crate::analysis::classify::non_increasing_response::NonIncreasingResponse;
use crate::analysis::validate::validated_campaign::tests::cell_dataset_fixture::CellDatasetFixture;
use crate::plan::run_role::RunRole;

/// The per-dose arm rise whose exact block total change `9 · PER_DOSE = 90 000` ns is strictly within the
/// 200 000 ns band, so the arm is Flat-equivalent rather than Increasing.
const PER_DOSE: i128 = 10_000;

#[test]
fn flat_equivalent_arm_projects_to_non_increasing_flat_equivalent() {
    // The shaped total change is within the band this proof targets.
    assert!(
        (9 * PER_DOSE) < FLAT_CONTROL_DELTA as i128,
        "the fixture arm total 9·{PER_DOSE} must land inside the ±{FLAT_CONTROL_DELTA} band"
    );

    let dataset = CellDatasetFixture::build(CellDatasetFixture::cell(), |_block, role, dose| match role {
        RunRole::Control => CONTROL_FLAT_NANOS,
        RunRole::Arm => arm_median(PER_DOSE, dose),
    });

    match BetaDescriptor::project(&dataset) {
        ClassifiedCell::NonIncreasing { response, .. } => assert_eq!(
            response,
            NonIncreasingResponse::FlatEquivalent,
            "an arm total within the band is Flat-equivalent, and no β descriptor is minted"
        ),
        other => panic!("a within-band arm must project to NonIncreasing/FlatEquivalent, got {other:?}"),
    }
}
