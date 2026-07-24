//! Different seeds produce different block orders.

use crate::manifest::schedule_seed::ScheduleSeed;
use crate::plan::schedule::Schedule;

/// Seed sensitivity: two distinct selected seeds produce different orders, so the seed
/// genuinely drives the randomization rather than being ignored. This asserts inequality
/// for these chosen seeds — not a universal claim that every seed pair differs.
#[test]
fn selected_seeds_produce_different_orders() {
    const SEED_A: u64 = 1;
    const SEED_B: u64 = 2;

    let order_a = Schedule::preregistered().randomized_block_order(ScheduleSeed::new(SEED_A));
    let order_b = Schedule::preregistered().randomized_block_order(ScheduleSeed::new(SEED_B));

    assert_ne!(
        order_a, order_b,
        "these selected seeds must produce distinct orders"
    );
}
