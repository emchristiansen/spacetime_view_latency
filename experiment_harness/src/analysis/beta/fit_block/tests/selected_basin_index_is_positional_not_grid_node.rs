//! The selected `basin_index` is the *positional* index into `refined_basins`, not the grid-node index of
//! the winning basin — while each refinement separately retains its own grid-node index (proof
//! requirement: prove `basin_index` indexes `refined_basins` positionally). A clean linear ladder `y = 2N`
//! has a single interior basin at grid node 18 (β = 1.0), so the winner's positional index (0) and its
//! grid-node index (18) are *different numbers*: an implementation that stored the grid node as the
//! selection would name slot 18 of a length-1 vector, which is out of range.

use crate::analysis::beta::basin_selection::BasinSelection;
use crate::analysis::beta::block_fit::BlockFit;
use crate::analysis::beta::fit_block::fit_block;
use crate::analysis::beta::fit_block::tests::ladder_from;

/// The grid node whose local basin wins for the clean linear ladder: β = 1.0 = (2 + 18)/20, so the node
/// index is 18 — deliberately far from the positional index 0 the winner occupies in `refined_basins`.
const WINNING_GRID_NODE: usize = 18;

#[test]
fn selected_basin_index_is_positional_not_grid_node() {
    let outcome = fit_block(&ladder_from(|n| 2.0 * n));
    let convergence = outcome.convergence();
    let refined = convergence.refined_basins();

    // The clean linear objective is unimodal: exactly one located basin.
    assert_eq!(refined.len(), 1, "a clean linear ladder locates a single interior basin");

    let basin_index = match convergence.selection() {
        BasinSelection::Selected { basin_index } => basin_index,
        BasinSelection::NoFeasiblePositiveScale => panic!("the clean linear ladder selects a basin"),
    };

    // The selection names the positional slot 0 of the single-element refined_basins vector.
    assert_eq!(basin_index, 0, "the winner is the 0th (and only) refined basin by position");

    // That same basin separately retains its grid-node index 18 — a *different* number from the positional
    // index 0. The two are distinct roles: a positional slot into refined_basins vs. a node into the
    // 79-node grid.
    let grid_node_index = refined[basin_index].termination().grid_node_index();
    assert_eq!(
        grid_node_index, WINNING_GRID_NODE,
        "the winning basin retains its own grid node 18 (β = 1.0)"
    );
    assert_ne!(
        basin_index, grid_node_index,
        "the selected index is the positional refined_basins slot, not the grid node"
    );

    // The candidate reached by *positional* indexing is exactly the authoritative fit candidate — the
    // positional index resolves to the winning fit componentwise.
    let selected = refined[basin_index].candidate();
    let fit_candidate = match outcome.fit() {
        BlockFit::Identifiable(candidate) => candidate,
        other => panic!("a clean linear ladder is identifiable, got {other:?}"),
    };
    assert_eq!(selected.beta(), fit_candidate.beta(), "positional selection resolves to the fit's β");
    assert_eq!(selected.a(), fit_candidate.a(), "positional selection resolves to the fit's a");
    assert_eq!(selected.b(), fit_candidate.b(), "positional selection resolves to the fit's b");
    assert_eq!(selected.rss(), fit_candidate.rss(), "positional selection resolves to the fit's RSS");
}
