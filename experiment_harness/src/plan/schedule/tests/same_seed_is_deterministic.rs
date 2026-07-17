//! The same seed produces the identical block order.

use crate::plan::schedule::Schedule;

/// Determinism: two independent constructions with the same seed yield identical orders,
/// so a recorded seed reproduces the schedule exactly.
#[test]
fn same_seed_is_deterministic() {
    const SEED: u64 = 0xDEAD_BEEF;

    let first = Schedule::preregistered().randomized_block_order(SEED);
    let second = Schedule::preregistered().randomized_block_order(SEED);

    assert_eq!(first, second, "the same seed must produce the same order");
}
