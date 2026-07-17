//! The arm/control run order is a deterministic, seed-dependent, non-constant function.

use crate::plan::run_role::RunRole;
use crate::plan::schedule::Schedule;

/// Order behavior, proven three ways over one fixed block set:
/// - **Determinism:** repeating the derivation at a seed yields the identical order.
/// - **Both-orderings coverage:** across all blocks at one seed, both "arm first" and
///   "control first" occur, so the order is not a constant pairing.
/// - **Seed sensitivity:** on the same block coordinates, at least one block's arm/control
///   order changes between two fixed seeds, so the order genuinely depends on the seed
///   (not merely on the block coordinate). This asserts a change for these selected seeds,
///   not for every seed pair.
#[test]
fn arm_control_order_is_deterministic_and_varies() {
    const SEED_A: u64 = 0xA5A5_5A5A;
    const SEED_B: u64 = 0x5A5A_A5A5;

    // Same 270-coordinate block set for both seeds (permutation order is irrelevant here;
    // `ordered_runs` is a pure function of the block coordinate and seed).
    let blocks = Schedule::preregistered().randomized_block_order(SEED_A);

    let mut arm_first = 0usize;
    let mut control_first = 0usize;
    let mut seed_sensitive = false;
    for block in &blocks {
        let first = block.ordered_runs(SEED_A);
        let again = block.ordered_runs(SEED_A);
        assert_eq!(
            first, again,
            "the arm/control order must be deterministic for a given seed"
        );
        match first[0].role() {
            RunRole::Arm => arm_first += 1,
            RunRole::Control => control_first += 1,
        }
        if block.ordered_runs(SEED_B)[0].role() != first[0].role() {
            seed_sensitive = true;
        }
    }

    assert!(arm_first > 0, "some block must run its arm first");
    assert!(
        control_first > 0,
        "some block must run its matched control first"
    );
    assert!(
        seed_sensitive,
        "these selected seeds must change at least one block's arm/control order"
    );
}
