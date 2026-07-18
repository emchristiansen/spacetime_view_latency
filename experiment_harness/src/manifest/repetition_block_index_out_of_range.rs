//! Why a candidate repetition-block index is not a valid position in a cell's fixed block sample.

use std::fmt;

use crate::params::REPETITION_BLOCKS;

/// A candidate repetition-block index fell outside the preregistered `0..REPETITION_BLOCKS` (0–29)
/// range of a cell's fixed block sample. It names the exact invariant it violates: a
/// [`RepetitionBlockIndex`](super::repetition_block_index::RepetitionBlockIndex) that exists is a
/// 0-based position strictly less than [`REPETITION_BLOCKS`], so an out-of-range index is rejected
/// here rather than allowed to reach the trusted campaign graph. The rejected value is retained as
/// typed structure, not only as diagnostic text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RepetitionBlockIndexOutOfRange {
    index: u32,
}

impl RepetitionBlockIndexOutOfRange {
    /// Record the out-of-range candidate index that was rejected.
    pub(crate) fn new(index: u32) -> Self {
        Self { index }
    }

    /// The rejected 0-based index.
    pub(crate) fn index(&self) -> u32 {
        self.index
    }
}

impl fmt::Display for RepetitionBlockIndexOutOfRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "repetition-block index {} is out of range: a valid index is 0-based and strictly less \
             than REPETITION_BLOCKS ({})",
            self.index, REPETITION_BLOCKS
        )
    }
}
