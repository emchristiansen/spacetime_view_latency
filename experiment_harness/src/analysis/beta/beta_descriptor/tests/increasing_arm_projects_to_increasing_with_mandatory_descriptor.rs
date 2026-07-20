//! Dataset-driven gate proof: a control-valid cell whose arm total change lies wholly *above* `+δ`
//! projects to [`ClassifiedCell::Increasing`], and that branch carries a **mandatory** secondary
//! [`BetaDescriptor`] — the β fit is invoked exactly here and nowhere else (spec: "implement and invoke
//! the β descriptor only for cells whose gated primary result is Increasing"). This is the sole gate arm
//! that mints a descriptor; the descriptor reports one fit outcome for each of the 30 arm blocks.
//!
//! The control is flat at [`CONTROL_FLAT_NANOS`], so δ = [`FLAT_CONTROL_DELTA`] = 200 000 ns and the
//! control-validity gate passes. The arm rises 100 000 ns per dose step, so each of the 30 identical
//! blocks has an exact total change of `9 · 100 000 = 900 000` ns; the degenerate `[900 000, 900 000]`
//! interval is entirely above `+δ`, so the arm response is Increasing. Because the arm latency is exactly
//! affine in the cumulative-row x-axis (`L(N) = 900 000 + 100 · N`), every block is a well-conditioned
//! β ≈ 1 fit, so the derived population interval is present as well.

use super::{arm_median, CONTROL_FLAT_NANOS, FLAT_CONTROL_DELTA};
use crate::analysis::beta::beta_descriptor::BetaDescriptor;
use crate::analysis::classify::classified_cell::ClassifiedCell;
use crate::analysis::validate::validated_campaign::tests::cell_dataset_fixture::CellDatasetFixture;
use crate::params::REPETITION_BLOCKS;
use crate::plan::run_role::RunRole;

/// The per-dose arm rise whose exact block total change `9 · 100 000 = 900 000` ns is strictly above
/// `+200 000` ns, so the arm is Increasing and the β descriptor is fit.
const PER_DOSE: i128 = 100_000;

#[test]
fn increasing_arm_projects_to_increasing_with_mandatory_descriptor() {
    // The shaped total change is strictly above the upper band bound this proof targets.
    assert!(
        (9 * PER_DOSE) > FLAT_CONTROL_DELTA as i128,
        "the fixture arm total 9·{PER_DOSE} must land above +{FLAT_CONTROL_DELTA}"
    );

    let dataset = CellDatasetFixture::build(CellDatasetFixture::cell(), |_block, role, dose| match role {
        RunRole::Control => CONTROL_FLAT_NANOS,
        RunRole::Arm => arm_median(PER_DOSE, dose),
    });

    match BetaDescriptor::project(&dataset) {
        // Binding `beta` is only possible because the Increasing branch *has* a mandatory descriptor field:
        // there is no descriptor-less Increasing to match.
        ClassifiedCell::Increasing { beta, .. } => {
            assert_eq!(
                beta.block_search_outcomes().len(),
                REPETITION_BLOCKS as usize,
                "the mandatory descriptor reports one fit outcome for every one of the 30 arm blocks"
            );
            assert!(
                beta.population().is_some(),
                "the affine arm ladder is identifiable in every block, so the population interval is present"
            );
        }
        other => panic!("an above-band arm must project to Increasing with a β descriptor, got {other:?}"),
    }
}
