//! Every adjacent pair in the frozen order is one block's matched Arm and Control.

use crate::entity_owner_pilot::attempt_inventory::AttemptInventory;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::plan::run_role::RunRole;

/// Matched-pair adjacency, held across many seeds: the frozen order is five consecutive
/// `{Arm, Control}` pairs, so a matched comparison is never split across the whole campaign's drift.
/// That the two members of each pair belong to the *same block* is pinned by
/// [`super::frozen_order_matches_independent_derivation`]'s golden vectors; what is asserted here is
/// the property that must hold at every seed rather than at two.
///
/// Also asserts that both role orderings occur somewhere, so the per-block role bit is a real
/// seed-dependent choice rather than a constant pairing that happens to be Control-first.
#[test]
fn each_block_runs_its_arm_and_control_adjacently() {
    const SEEDS: [u64; 4] = [0, 1, 7, 0xDEAD_BEEF];

    let mut arm_first = 0usize;
    let mut control_first = 0usize;

    for seed in SEEDS {
        let frozen = AttemptInventory::frozen(ScheduleSeed::new(seed)).expect("the seed freezes");

        for (pair_index, pair) in frozen.attempts().chunks_exact(2).enumerate() {
            let [first, second] = [pair[0], pair[1]];
            assert_ne!(
                first.role(),
                second.role(),
                "seed {seed} pair {pair_index} must hold two distinct roles"
            );
            match first.role() {
                RunRole::Arm => arm_first += 1,
                RunRole::Control => control_first += 1,
            }
        }
    }

    assert!(arm_first > 0, "some block must run its Arm first");
    assert!(
        control_first > 0,
        "some block must run its matched Control first"
    );
}
