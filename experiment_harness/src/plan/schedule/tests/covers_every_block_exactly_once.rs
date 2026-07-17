//! The seeded order contains every `Cell × block` product exactly once.

use std::collections::BTreeSet;

use crate::manifest::schedule_seed::ScheduleSeed;
use crate::plan::cell::Cell;
use crate::plan::schedule::Schedule;

/// Completeness: the globally randomized order is built from the full
/// `Cell × 0..BLOCKS_PER_CELL` product. It has exactly one entry per
/// `(cell canonical tag, block index)` coordinate, matching an independently constructed
/// oracle set — proving no block is missing, extra, or out of range.
#[test]
fn covers_every_block_exactly_once() {
    const SEED: u64 = 0x0BAD_F00D_1234_5678;

    let expected: BTreeSet<(&'static str, u32)> = Cell::all()
        .into_iter()
        .flat_map(|cell| {
            (0..Schedule::BLOCKS_PER_CELL).map(move |block| (cell.canonical_tag(), block))
        })
        .collect();

    let order = Schedule::preregistered().randomized_block_order(ScheduleSeed::new(SEED));
    let produced: BTreeSet<(&'static str, u32)> = order
        .iter()
        .map(|block| (block.cell().canonical_tag(), block.block_index()))
        .collect();

    assert_eq!(
        order.len(),
        Cell::all().len() * Schedule::BLOCKS_PER_CELL as usize,
        "the order length must equal the full block-product size"
    );
    assert_eq!(
        produced, expected,
        "the order must cover exactly the full `Cell × block` product"
    );
}
