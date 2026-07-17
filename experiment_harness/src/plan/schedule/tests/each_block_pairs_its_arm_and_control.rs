//! Each block's ordered runs are exactly its matched arm and control, adjacent.

use crate::plan::run_role::RunRole;
use crate::plan::schedule::Schedule;

/// Adjacency and pairing: `ordered_runs` returns exactly the block's `{Arm, Control}`
/// pair for the block's own cell, as a single two-element array — so the matched pair's
/// adjacency is structural, not caller-maintained. Verified for every block in a seeded
/// order.
#[test]
fn each_block_pairs_its_arm_and_control() {
    const SEED: u64 = 7;

    for block in &Schedule::preregistered().randomized_block_order(SEED) {
        let runs = block.ordered_runs(SEED);

        for run in runs {
            assert_eq!(
                run.cell(),
                block.cell(),
                "each run must belong to the block's cell"
            );
        }

        assert_ne!(
            runs[0].role(),
            runs[1].role(),
            "a block's two runs must be distinct roles"
        );
        assert!(
            runs.iter().any(|run| run.role() == RunRole::Arm)
                && runs.iter().any(|run| run.role() == RunRole::Control),
            "a block's runs must be exactly its arm and its matched control"
        );
    }
}
