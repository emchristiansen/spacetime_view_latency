//! The first out-of-range index: `REPETITION_BLOCKS` is one past the last valid position.

use crate::manifest::repetition_block_index::RepetitionBlockIndex;
use crate::params::REPETITION_BLOCKS;

#[test]
fn the_count_itself_is_rejected() {
    let error = RepetitionBlockIndex::try_new(REPETITION_BLOCKS)
        .expect_err("REPETITION_BLOCKS is one past the last valid index and must be rejected");
    assert_eq!(
        error.index(),
        REPETITION_BLOCKS,
        "the typed error carries the exact rejected index"
    );
}
