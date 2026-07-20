//! Cell-level descriptor proofs: the 30-fit population interval and labels, and one non-identifiable
//! block suppressing the population interval while every block outcome is retained.
//!
//! The descriptor proofs drive [`BetaDescriptor::from_ladders`](super::BetaDescriptor::from_ladders)
//! over 30 synthetic ten-dose ladders, bypassing the trusted graph while exercising the real per-block
//! fit and the derived population interval. The direct order-statistic proof drives
//! [`BetaPopulation::from_identified`](crate::analysis::beta::beta_population::BetaPopulation) so the
//! `[X_(10), X_(21)]` selection is pinned independently of any fit.

use crate::analysis::beta::block_point::BlockPoint;
use crate::params::{NUM_DOSES_USIZE, REPETITION_BLOCKS};

/// The fixed per-cell block count as an array length, mirroring the descriptor's own `N_BLOCKS`.
pub(super) const N_BLOCKS: usize = REPETITION_BLOCKS as usize;

/// The shared ten-dose `N` ladder for the descriptor fixtures, the same ≈ 2.7-decade geometric span the
/// block-fit proofs use, so each replicated block is a well-conditioned identifiable fit.
pub(super) const LADDER_N: [f64; NUM_DOSES_USIZE] =
    [10.0, 20.0, 40.0, 80.0, 160.0, 320.0, 640.0, 1280.0, 2560.0, 5120.0];

/// Build one ten-dose ladder whose latency at each dose is `y_of_n(N)` over the shared [`LADDER_N`].
pub(super) fn ladder_from(y_of_n: impl Fn(f64) -> f64) -> [BlockPoint; NUM_DOSES_USIZE] {
    std::array::from_fn(|dose_index| {
        let n = LADDER_N[dose_index];
        BlockPoint::from_synthetic(n, y_of_n(n))
    })
}

/// The flat direct-control median, in nanoseconds, that the dataset-driven projection gate proofs hold
/// constant across every dose and block. Every control dose median is then this value, so the frozen
/// margin is δ = this / 5 ([`FLAT_CONTROL_DELTA`]) and the control's own total change is zero — the
/// control-validity gate passes, and the arm ladder alone determines the response.
pub(super) const CONTROL_FLAT_NANOS: u128 = 1_000_000;

/// The frozen equivalence half-band δ a flat control at [`CONTROL_FLAT_NANOS`] induces: one fifth of the
/// median of the 300 identical control dose medians. Re-derived here so each gate proof states, in exact
/// nanoseconds, the band its shaped arm total is meant to land in.
pub(super) const FLAT_CONTROL_DELTA: u128 = CONTROL_FLAT_NANOS / 5;

/// One block's arm median at 0-based dose `d`, for an arm whose per-dose latency rises `per_dose`
/// nanoseconds per dose step above [`CONTROL_FLAT_NANOS`]. Because the canonical ladder x-axis is
/// `(d + 1) · BATCH_SIZE` (a uniform 1000-wide step over a 9000-wide span), the arm's paired difference
/// against the flat control is exactly linear, so its block total change is exactly `9 · per_dose`
/// nanoseconds. A negative `per_dose` is a decreasing arm; the fixtures keep every median positive.
pub(super) fn arm_median(per_dose: i128, dose: usize) -> u128 {
    let raw = CONTROL_FLAT_NANOS as i128 + per_dose * dose as i128;
    u128::try_from(raw).expect("every gate-fixture arm median stays strictly positive")
}

mod decreasing_arm_projects_to_non_increasing_decreasing;
mod descriptor_and_evidence_align_by_collection_order_key;
mod flat_equivalent_arm_projects_to_non_increasing_flat_equivalent;
mod inconclusive_arm_projects_to_non_increasing_inconclusive;
mod increasing_arm_projects_to_increasing_with_mandatory_descriptor;
mod invalid_control_dataset_projects_to_invalid_control;
mod one_non_identifiable_block_suppresses_population;
mod population_interval_selects_order_statistics;
mod sole_outcome_storage_derives_fits_and_convergence;
mod the_stored_shape_forbids_optional_or_independent_response;
mod uniform_linear_cell_labels_population_linear;
