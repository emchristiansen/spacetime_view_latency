//! The frozen order at two fixed seeds equals an independently derived expected permutation.

use crate::entity_owner_pilot::attempt_inventory::AttemptInventory;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::plan::run_role::RunRole::{Arm, Control};

use super::slot::slot;

/// Golden vectors, not a re-derivation: the expected sequences below were computed outside this
/// crate from the two domain-separated SHA-256 subjects the module documents — sorting the five
/// blocks by `("entity-owner-pilot:block-order;seed=…;block=…", block)` and taking each block's role
/// order from the low bit of `"entity-owner-pilot:arm-control-order;seed=…;block=…"`. Asserting
/// against a fixed answer rather than against the code's own derivation is what makes this a test of
/// the derivation instead of a restatement of it.
///
/// This one assertion pins everything positional at once: the block permutation, each block's role
/// order, the pair adjacency, and the full identity minted for every slot. The two seeds produce
/// different permutations and different role orders, so a seed-ignoring implementation cannot pass
/// both.
#[test]
fn frozen_order_matches_independent_derivation() {
    const SEED_A: u64 = 0xDEAD_BEEF;
    const SEED_B: u64 = 7;

    let expected_a = [
        slot(2, Control),
        slot(2, Arm),
        slot(1, Control),
        slot(1, Arm),
        slot(4, Arm),
        slot(4, Control),
        slot(0, Control),
        slot(0, Arm),
        slot(3, Control),
        slot(3, Arm),
    ];
    let expected_b = [
        slot(0, Control),
        slot(0, Arm),
        slot(3, Control),
        slot(3, Arm),
        slot(1, Control),
        slot(1, Arm),
        slot(4, Arm),
        slot(4, Control),
        slot(2, Arm),
        slot(2, Control),
    ];

    let frozen_a = AttemptInventory::frozen(ScheduleSeed::new(SEED_A)).expect("seed A freezes");
    let frozen_b = AttemptInventory::frozen(ScheduleSeed::new(SEED_B)).expect("seed B freezes");

    assert_eq!(
        frozen_a.attempts(),
        expected_a,
        "seed {SEED_A:#x} must freeze the independently derived execution order"
    );
    assert_eq!(
        frozen_b.attempts(),
        expected_b,
        "seed {SEED_B} must freeze the independently derived execution order"
    );
}
