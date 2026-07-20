//! Dataset-driven gate proof: a control-valid cell whose 30-block arm interval *straddles* a band bound
//! projects to [`ClassifiedCell::NonIncreasing`] with a [`NonIncreasingResponse::Inconclusive`] response
//! and **no** secondary β descriptor. Inconclusive is retained as a distinct non-Increasing response — it
//! is not folded into Increasing — so the β fit is never invoked.
//!
//! The control is flat at [`CONTROL_FLAT_NANOS`], so δ = [`FLAT_CONTROL_DELTA`] = 200 000 ns and the
//! control-validity gate passes. Unlike the other gate proofs, the 30 blocks are *not* identical: the
//! first 15 blocks have a flat arm (total change 0) and the last 15 have an arm rising 100 000 ns per dose
//! step (total change `9 · 100 000 = 900 000` ns). Sorting the 30 block totals gives fifteen `0`s then
//! fifteen `900 000`s, so the frozen `[X_(10), X_(21)]` order-statistic interval is `[0, 900 000]`: its
//! lower bound `0` is not above `+δ` (so not Increasing) and its upper bound `900 000` is above `+δ` (so
//! not Flat-equivalent), while neither bound is below `−δ` (so not Decreasing) — the interval straddles
//! `+δ` and the response is Inconclusive.

use super::{arm_median, CONTROL_FLAT_NANOS, FLAT_CONTROL_DELTA};
use crate::analysis::beta::beta_descriptor::BetaDescriptor;
use crate::analysis::classify::classified_cell::ClassifiedCell;
use crate::analysis::classify::non_increasing_response::NonIncreasingResponse;
use crate::analysis::validate::validated_campaign::tests::cell_dataset_fixture::CellDatasetFixture;
use crate::plan::run_role::RunRole;

/// The number of leading flat-arm blocks (total change 0). With 30 blocks total and the frozen lower/upper
/// order statistics at 0-based indices 9 and 20, this split places index 9 in the flat run and index 20 in
/// the rising run, so the selected interval is exactly `[0, RISING_TOTAL]`.
const FLAT_BLOCKS: usize = 15;
/// The per-dose arm rise of the trailing blocks, whose exact total change `9 · 100 000 = 900 000` ns sits
/// strictly above `+δ`.
const RISING_PER_DOSE: i128 = 100_000;

#[test]
fn inconclusive_arm_projects_to_non_increasing_inconclusive() {
    // The straddle this proof relies on: the rising blocks' total is above +δ while the flat blocks' total
    // (0) is neither above +δ nor below −δ, so the [X_(10), X_(21)] = [0, rising] interval crosses +δ.
    let rising_total = 9 * RISING_PER_DOSE;
    assert!(
        rising_total > FLAT_CONTROL_DELTA as i128 && 0 < FLAT_CONTROL_DELTA as i128,
        "the interval [0, {rising_total}] must straddle the +{FLAT_CONTROL_DELTA} bound"
    );

    let dataset = CellDatasetFixture::build(CellDatasetFixture::cell(), |block, role, dose| match role {
        RunRole::Control => CONTROL_FLAT_NANOS,
        // Leading blocks flat (per-dose 0 → total 0), trailing blocks rising.
        RunRole::Arm if block < FLAT_BLOCKS => arm_median(0, dose),
        RunRole::Arm => arm_median(RISING_PER_DOSE, dose),
    });

    match BetaDescriptor::project(&dataset) {
        ClassifiedCell::NonIncreasing { response, .. } => assert_eq!(
            response,
            NonIncreasingResponse::Inconclusive,
            "an arm interval straddling +δ is Inconclusive, and no β descriptor is minted"
        ),
        other => panic!("a straddling arm must project to NonIncreasing/Inconclusive, got {other:?}"),
    }
}
