//! A validated 0-based repetition-block index within a cell's fixed block sample.

use serde::Serialize;

use crate::manifest::repetition_block_index_out_of_range::RepetitionBlockIndexOutOfRange;
use crate::params::REPETITION_BLOCKS;

/// The 0-based position of a repetition block within a cell's fixed [`REPETITION_BLOCKS`] (30) block
/// sample.
///
/// This type proves **range validity only**: every `RepetitionBlockIndex` that exists is a member of
/// `0..REPETITION_BLOCKS`. The sole fallible constructor is [`Self::try_new`], which rejects any
/// out-of-range candidate with a typed [`RepetitionBlockIndexOutOfRange`]; the private field means no
/// caller can fabricate an out-of-range index, so range membership is a property of the type rather
/// than of caller discipline. It is serialized transparently as its bare `u32`, so a coordinate
/// carrying one has the exact wire shape of the pre-existing raw index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub(crate) struct RepetitionBlockIndex(u32);

impl RepetitionBlockIndex {
    /// Validate a candidate 0-based index, accepting it only if it is strictly less than
    /// [`REPETITION_BLOCKS`] and failing loud with a typed [`RepetitionBlockIndexOutOfRange`]
    /// otherwise. This is the analysis validation pass's authorized path to a trusted repetition
    /// index from an untrusted wire value.
    pub(crate) fn try_new(index: u32) -> Result<Self, RepetitionBlockIndexOutOfRange> {
        if index < REPETITION_BLOCKS {
            Ok(Self(index))
        } else {
            Err(RepetitionBlockIndexOutOfRange::new(index))
        }
    }

    /// The validated 0-based repetition-block index.
    pub(crate) fn get(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests;
