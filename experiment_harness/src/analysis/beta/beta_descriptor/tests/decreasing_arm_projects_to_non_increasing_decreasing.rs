//! Dataset-driven gate proof: a control-valid cell whose arm total change lies wholly *below* `-δ`
//! projects to [`ClassifiedCell::NonIncreasing`] with a [`NonIncreasingResponse::Decreasing`] response and
//! **no** secondary β descriptor. Decreasing is a non-Increasing response, so the β fit is never invoked —
//! the exponent descriptor cannot exist on this branch (spec: the gate is structural, not a runtime flag).
//!
//! The control is flat at [`CONTROL_FLAT_NANOS`], so δ = [`FLAT_CONTROL_DELTA`] = 200 000 ns and the
//! control-validity gate passes. The arm *falls* 100 000 ns per dose step, so each of the 30 identical
//! blocks has an exact total change of `9 · (−100 000) = −900 000` ns; the degenerate `[−900 000,
//! −900 000]` interval is entirely below `−δ`, so the arm response is Decreasing. The arm median stays
//! strictly positive (its minimum, at the top dose, is 100 000 ns).

use super::{arm_median, CONTROL_FLAT_NANOS, FLAT_CONTROL_DELTA};
use crate::analysis::beta::beta_descriptor::BetaDescriptor;
use crate::analysis::classify::classified_cell::ClassifiedCell;
use crate::analysis::classify::non_increasing_response::NonIncreasingResponse;
use crate::analysis::validate::validated_campaign::tests::cell_dataset_fixture::CellDatasetFixture;
use crate::plan::run_role::RunRole;

/// The per-dose arm fall whose exact block total change `9 · (−100 000) = −900 000` ns is strictly below
/// `−200 000` ns, so the arm is Decreasing.
const PER_DOSE: i128 = -100_000;

#[test]
fn decreasing_arm_projects_to_non_increasing_decreasing() {
    // The shaped total change is strictly below the lower band bound this proof targets.
    assert!(
        (9 * PER_DOSE) < -(FLAT_CONTROL_DELTA as i128),
        "the fixture arm total 9·({PER_DOSE}) must land below −{FLAT_CONTROL_DELTA}"
    );

    let dataset = CellDatasetFixture::build(CellDatasetFixture::cell(), |_block, role, dose| match role {
        RunRole::Control => CONTROL_FLAT_NANOS,
        RunRole::Arm => arm_median(PER_DOSE, dose),
    });

    match BetaDescriptor::project(&dataset) {
        ClassifiedCell::NonIncreasing { response, .. } => assert_eq!(
            response,
            NonIncreasingResponse::Decreasing,
            "an arm total below −δ is Decreasing, and no β descriptor is minted"
        ),
        other => panic!("a below-band arm must project to NonIncreasing/Decreasing, got {other:?}"),
    }
}
