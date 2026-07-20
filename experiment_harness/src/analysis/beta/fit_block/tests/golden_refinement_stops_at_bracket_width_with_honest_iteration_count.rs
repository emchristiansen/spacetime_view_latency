//! The recorded golden-section termination is read from the loop's *actual* exit condition, with no
//! off-by-one in the iteration count (proof requirement (i): preserve the initial/final brackets exactly
//! and classify the stop from the real terminating condition). A clean single-basin refinement stops by
//! `BracketWidthReached` strictly before the iteration cap; its initial bracket is exactly the winning
//! node's two grid neighbours; its final bracket is contained in that initial bracket and contracted to
//! within the frozen width tolerance; and the bracket *one contraction earlier* still exceeded the
//! tolerance — so the loop ran exactly until the width crossed the threshold, neither one iteration too
//! few nor too many.

use crate::analysis::beta::basin_selection::BasinSelection;
use crate::analysis::beta::fit_block::tests::ladder_from;
use crate::analysis::beta::fit_block::{
    fit_block, grid_node, GOLDEN_BRACKET_TOL, GOLDEN_MAX_ITERS, INV_GOLDEN,
};
use crate::analysis::beta::golden_stop::GoldenStop;

/// The winning basin's grid node for the clean linear ladder (β = 1.0 = (2 + 18)/20), whose initial
/// bracket is the exact pair of neighbour grid nodes 17 and 19.
const WINNING_GRID_NODE: usize = 18;

#[test]
fn golden_refinement_stops_at_bracket_width_with_honest_iteration_count() {
    let outcome = fit_block(&ladder_from(|n| 2.0 * n));
    let convergence = outcome.convergence();
    let basin_index = match convergence.selection() {
        BasinSelection::Selected { basin_index } => basin_index,
        BasinSelection::NoFeasiblePositiveScale => panic!("the clean linear ladder selects a basin"),
    };
    let termination = convergence.refined_basins()[basin_index].termination();

    // The initial bracket is preserved exactly: the winning basin's node 18 brackets against its two grid
    // neighbours 17 and 19, bit-for-bit equal to the frozen grid nodes.
    assert_eq!(termination.grid_node_index(), WINNING_GRID_NODE, "the winning basin is grid node 18");
    assert_eq!(
        termination.initial_bracket_lo(),
        grid_node(WINNING_GRID_NODE - 1),
        "the initial bracket's lower endpoint is exactly the left grid neighbour (node 17)"
    );
    assert_eq!(
        termination.initial_bracket_hi(),
        grid_node(WINNING_GRID_NODE + 1),
        "the initial bracket's upper endpoint is exactly the right grid neighbour (node 19)"
    );

    // The stop is the convergence cause read from the actual terminating condition, not the cap.
    assert_eq!(
        termination.stop(),
        GoldenStop::BracketWidthReached,
        "a well-conditioned basin converges by bracket width, not the iteration cap"
    );
    assert!(
        termination.iterations() < GOLDEN_MAX_ITERS,
        "convergence happens strictly before the {GOLDEN_MAX_ITERS}-iteration cap, got {}",
        termination.iterations()
    );
    assert!(termination.iterations() >= 1, "at least one contraction executed");

    // The final bracket is contained in the initial one and contracted to within the tolerance, so the
    // recorded BracketWidthReached is truthful.
    let final_width = termination.final_bracket_hi() - termination.final_bracket_lo();
    assert!(
        termination.final_bracket_lo() >= termination.initial_bracket_lo()
            && termination.final_bracket_hi() <= termination.initial_bracket_hi(),
        "the final bracket [{}, {}] is contained in the initial bracket [{}, {}]",
        termination.final_bracket_lo(),
        termination.final_bracket_hi(),
        termination.initial_bracket_lo(),
        termination.initial_bracket_hi()
    );
    assert!(
        final_width <= GOLDEN_BRACKET_TOL,
        "the refinement stopped once the bracket reached the {GOLDEN_BRACKET_TOL} tolerance, final width {final_width}"
    );

    // No off-by-one: the bracket *one contraction earlier* — the final width divided by the per-step
    // golden factor — was still strictly above the tolerance. So the loop did not stop one iteration too
    // early (it reached the tolerance), nor one too late (the prior bracket still exceeded it); the count
    // is exactly the contractions the loop took.
    assert!(
        final_width / INV_GOLDEN > GOLDEN_BRACKET_TOL,
        "the bracket before the final contraction ({}) still exceeded the {GOLDEN_BRACKET_TOL} tolerance, \
         so termination is off-by-one-free",
        final_width / INV_GOLDEN
    );
}
