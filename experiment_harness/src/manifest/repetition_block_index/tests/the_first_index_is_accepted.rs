//! The lower boundary: index 0 is in range and round-trips.

use crate::manifest::repetition_block_index::RepetitionBlockIndex;

#[test]
fn the_first_index_is_accepted() {
    let index = RepetitionBlockIndex::try_new(0).expect("0 is the first valid repetition-block index");
    assert_eq!(index.get(), 0, "the validated index preserves its 0 value");
}
