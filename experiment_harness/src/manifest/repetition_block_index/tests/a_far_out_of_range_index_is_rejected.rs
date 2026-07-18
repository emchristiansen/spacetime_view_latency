//! A far out-of-range index (`u32::MAX`) is rejected and its value preserved in the typed error.

use crate::manifest::repetition_block_index::RepetitionBlockIndex;

#[test]
fn a_far_out_of_range_index_is_rejected() {
    let error = RepetitionBlockIndex::try_new(u32::MAX)
        .expect_err("u32::MAX is far out of range and must be rejected");
    assert_eq!(
        error.index(),
        u32::MAX,
        "the typed error carries the exact rejected index, however far out of range"
    );
}
