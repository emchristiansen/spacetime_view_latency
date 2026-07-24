//! The upper boundary: `REPETITION_BLOCKS - 1` is the greatest in-range index and round-trips.

use crate::manifest::repetition_block_index::RepetitionBlockIndex;
use crate::params::REPETITION_BLOCKS;

#[test]
fn the_last_in_range_index_is_accepted() {
    let last = REPETITION_BLOCKS - 1;
    let index = RepetitionBlockIndex::try_new(last)
        .expect("REPETITION_BLOCKS - 1 is the last valid repetition-block index");
    assert_eq!(index.get(), last, "the validated index preserves the last in-range value");
}
