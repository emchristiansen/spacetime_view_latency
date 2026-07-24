//! The convergence record retains *every* grid-local basin the search located, not only the winning basin
//! (spec: "Do not report only the winning basin, because convergence status covers the complete global
//! search"). A zero-x-spread ladder makes the constrained RSS flat across the whole domain, so the
//! `≤`-basin rule marks a run of grid nodes as basins; the search refines and records all of them while
//! selecting exactly one.

use crate::analysis::beta::basin_selection::BasinSelection;
use crate::analysis::beta::fit_block::fit_block;
use crate::analysis::beta::fit_block::tests::constant_n_ladder;

#[test]
fn records_every_located_basin_not_only_the_winner() {
    // Every dose shares N = 100, so x = N^β is a single value at every β and the a = 0 edge fit gives an
    // RSS that is flat across the domain; the `≤` local-basin rule then marks a run of grid nodes as
    // basins rather than a single one.
    let ladder = constant_n_ladder(100.0, |dose_index| 100.0 * (dose_index as f64 + 1.0));
    let outcome = fit_block(&ladder);
    let convergence = outcome.convergence();

    let basin_nodes = convergence.grid().basin_node_indices();
    let refined = convergence.refined_basins();

    // One refinement per located basin, and genuinely more than one basin — so the record cannot be "only
    // the winner."
    assert_eq!(
        refined.len(),
        basin_nodes.len(),
        "the convergence records exactly one refinement per located basin"
    );
    assert!(
        refined.len() >= 2,
        "the flat objective locates a run of basins, so more than the winner is recorded, got {}",
        refined.len()
    );

    // Each recorded refinement carries its own grid node, positionally aligned with the grid's basin index
    // list — every located basin is retained with its own telemetry, not collapsed to the winner.
    for (position, &node) in basin_nodes.iter().enumerate() {
        assert_eq!(
            refined[position].termination().grid_node_index(),
            node,
            "refined basin at position {position} retains its own grid node {node}"
        );
    }

    // Exactly one basin is selected as the winner, and at least one *non-selected* basin remains recorded.
    let basin_index = match convergence.selection() {
        BasinSelection::Selected { basin_index } => basin_index,
        BasinSelection::NoFeasiblePositiveScale => {
            panic!("the flat positive-scale ladder selects a basin")
        }
    };
    assert!(
        basin_index < refined.len(),
        "the selected basin index {basin_index} names a recorded basin"
    );
    assert!(
        (0..refined.len()).any(|i| i != basin_index),
        "at least one located basin is retained without being the winner"
    );
}
