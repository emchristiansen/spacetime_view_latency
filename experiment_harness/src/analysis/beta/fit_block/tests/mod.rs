//! Deterministic proofs of the frozen exponent search: power recovery, the `a = 0` acceptances, the
//! rejection outcomes, bound pinning, and the lower-β tie rule with repeatability.
//!
//! Every proof drives [`fit_block`](super::fit_block) over one ten-dose ladder built from synthetic
//! `(N, y)` coordinates via [`BlockPoint::from_synthetic`], so the fixtures are exact power laws and
//! degeneracies chosen to exercise a single frozen behaviour each. The shared ladder spans roughly two
//! decades of `N` (spec: "A labeled exploratory ladder spanning at least two decades may improve this
//! secondary fit"), which conditions the fit well enough that the interior recoveries are distinguishable.

use crate::analysis::beta::block_point::BlockPoint;
use crate::params::NUM_DOSES_USIZE;

/// The shared ten-dose `N` ladder, a geometric span from `10` to `5120` (≈ 2.7 decades). A wide,
/// strictly increasing span gives the constrained fit a well-conditioned `x = N^β` design so an interior
/// exponent is genuinely identifiable rather than washed out by a narrow lever arm.
pub(super) const LADDER_N: [f64; NUM_DOSES_USIZE] =
    [10.0, 20.0, 40.0, 80.0, 160.0, 320.0, 640.0, 1280.0, 2560.0, 5120.0];

/// Build a ten-dose ladder whose latency at each dose is `y_of_n(N)` over the shared [`LADDER_N`]. The
/// closure is the exact synthetic law under test; every coordinate funnels through the checked synthetic
/// mint, so an accidentally invalid fixture is a loud failure rather than a silent bad point.
pub(super) fn ladder_from(y_of_n: impl Fn(f64) -> f64) -> [BlockPoint; NUM_DOSES_USIZE] {
    std::array::from_fn(|dose_index| {
        let n = LADDER_N[dose_index];
        BlockPoint::from_synthetic(n, y_of_n(n))
    })
}

/// Build a ten-dose ladder with a single constant `N` at every dose and latency `y_of_index(i)`. A
/// zero-spread `x = N^β` column collapses the fit's design matrix identically at every β, the degeneracy
/// the flat-objective tie proof needs.
pub(super) fn constant_n_ladder(n: f64, y_of_index: impl Fn(usize) -> f64) -> [BlockPoint; NUM_DOSES_USIZE] {
    std::array::from_fn(|dose_index| BlockPoint::from_synthetic(n, y_of_index(dose_index)))
}

mod accepts_exact_zero_intercept_fit;
mod accepts_projected_to_zero_intercept;
mod flat_interior_objective_is_non_identifiable;
mod golden_refinement_stops_at_bracket_width_with_honest_iteration_count;
mod lower_edge_omits_outward_probe_and_keeps_inward;
mod pins_above_bracket_power_at_upper_bound;
mod pins_constant_ladder_at_lower_bound;
mod records_every_located_basin_not_only_the_winner;
mod recovers_interior_linear_exponent;
mod recovers_interior_sublinear_exponent;
mod rejects_zero_scale_ladder;
mod resolves_flat_objective_tie_to_smaller_beta;
mod selected_basin_index_is_positional_not_grid_node;
mod upper_edge_omits_outward_probe_and_keeps_inward;
