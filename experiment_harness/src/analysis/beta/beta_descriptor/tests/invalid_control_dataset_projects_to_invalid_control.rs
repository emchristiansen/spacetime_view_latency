//! Dataset-driven gate proof: a cell whose *direct control* is itself non-flat projects to
//! [`ClassifiedCell::InvalidControl`], carrying evidence only and **no** arm response, prediction
//! comparison, or secondary β descriptor — even when the arm's paired difference *would* be Increasing
//! under a valid control. The whole cell is invalidated before any arm claim is expressible (spec: "Make
//! complete-campaign validation and statistical control validity distinct typed outcomes"), so the β fit
//! is unreachable.
//!
//! The control rises [`CONTROL_SLOPE_PER_DOSE`] = 100 000 ns per dose step and the arm rises
//! [`ARM_SLOPE_PER_DOSE`] = 200 000 ns per dose step. Two exact, independent facts follow over the
//! 9-step ladder:
//!
//! - The control's *own* total change is `9 · 100 000 = 900 000` ns, so the direct control is non-flat.
//!   The frozen δ is a fifth of the median control dose median, itself at most `CONTROL_FLAT_NANOS +
//!   9 · CONTROL_SLOPE_PER_DOSE`; 900 000 ns exceeds even a fifth of that upper bound, so the control
//!   interval cannot fit `[-δ, +δ]` and the control-validity gate fails.
//! - The arm-minus-control paired difference `D(N) = L_arm − L_control` rises
//!   `ARM_SLOPE_PER_DOSE − CONTROL_SLOPE_PER_DOSE = 100 000` ns per step, so its total change is also
//!   `900 000` ns — above the same δ upper bound. Under a *valid* control this arm would classify
//!   Increasing; the InvalidControl outcome proves the control gate suppresses that otherwise-Increasing
//!   arm claim.

use super::{arm_median, CONTROL_FLAT_NANOS};
use crate::analysis::beta::beta_descriptor::BetaDescriptor;
use crate::analysis::classify::classified_cell::ClassifiedCell;
use crate::analysis::validate::validated_campaign::tests::cell_dataset_fixture::CellDatasetFixture;
use crate::plan::run_role::RunRole;

/// The per-dose rise of every control run — its exact per-block total change `9 · 100 000 = 900 000` ns
/// makes the direct control non-flat and thus invalid.
const CONTROL_SLOPE_PER_DOSE: i128 = 100_000;
/// The per-dose rise of every arm run, steeper than the control's, so the arm-minus-control paired
/// difference is itself rising rather than flat.
const ARM_SLOPE_PER_DOSE: i128 = 200_000;
/// The number of dose steps across the ten-dose ladder (doses `0..=9`) — the span the total changes and
/// the worst-case control median are taken over.
const LADDER_STEPS: i128 = 9;

#[test]
fn invalid_control_dataset_projects_to_invalid_control() {
    // δ ≤ (max control median) / 5, independent of the exact median. Both the control's own total change
    // and the arm-minus-control total change exceed that upper bound, so — under a valid control — the arm
    // would be Increasing, yet the control's own interval cannot fit the band and invalidates the cell.
    let max_control_median = CONTROL_FLAT_NANOS as i128 + LADDER_STEPS * CONTROL_SLOPE_PER_DOSE;
    let control_total = LADDER_STEPS * CONTROL_SLOPE_PER_DOSE;
    let arm_minus_control_total = LADDER_STEPS * (ARM_SLOPE_PER_DOSE - CONTROL_SLOPE_PER_DOSE);
    assert!(
        control_total > max_control_median / 5,
        "the control total {control_total} must exceed the largest possible δ ≤ {max_control_median}/5"
    );
    assert!(
        arm_minus_control_total > max_control_median / 5,
        "the arm-minus-control total {arm_minus_control_total} would be Increasing under a valid control"
    );

    let dataset = CellDatasetFixture::build(CellDatasetFixture::cell(), |_block, role, dose| match role {
        RunRole::Control => arm_median(CONTROL_SLOPE_PER_DOSE, dose),
        RunRole::Arm => arm_median(ARM_SLOPE_PER_DOSE, dose),
    });

    match BetaDescriptor::project(&dataset) {
        ClassifiedCell::InvalidControl { .. } => {}
        other => panic!("a non-flat control must project to InvalidControl with no arm claim, got {other:?}"),
    }
}
