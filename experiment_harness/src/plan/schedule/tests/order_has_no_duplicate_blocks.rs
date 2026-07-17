//! The seeded order lists each block coordinate at most once.

use std::collections::HashSet;

use crate::plan::schedule::Schedule;

/// Uniqueness: a permutation neither drops nor duplicates. Every
/// `(cell canonical tag, block index)` coordinate in the seeded order is distinct, so no
/// block is scheduled twice or silently dropped.
#[test]
fn order_has_no_duplicate_blocks() {
    const SEED: u64 = 42;

    let order = Schedule::preregistered().randomized_block_order(SEED);

    let mut seen = HashSet::new();
    for block in &order {
        let coordinate = (block.cell().canonical_tag(), block.block_index());
        assert!(
            seen.insert(coordinate),
            "duplicate block coordinate {coordinate:?} in the seeded order"
        );
    }

    assert_eq!(
        seen.len(),
        order.len(),
        "every block coordinate in the order must be unique"
    );
}
