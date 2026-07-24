//! The typed contradiction behind a [`BlockCensus`](super::integrity_error::IntegrityError) failure.

/// Why a cell's repetition-block census failed for one `(cell, role)`. Each cell must carry exactly
/// [`REPETITION_BLOCKS`](crate::params::REPETITION_BLOCKS) blocks, indices `expected_min..=expected_max`
/// (`0..=REPETITION_BLOCKS - 1`), each present exactly once for each role. Every mode carries typed
/// range and occurrence-count evidence as fields — never in prose — while the enclosing
/// [`BlockCensus`](super::integrity_error::IntegrityError) variant supplies the `(cell, role)` location.
#[derive(Debug)]
pub(crate) enum BlockCensusFault {
    /// A raw wire block index is not in the valid inclusive range `expected_min..=expected_max`, so
    /// trusted [`RepetitionBlockIndex`](crate::manifest::repetition_block_index::RepetitionBlockIndex)
    /// construction failed on `block`.
    OutOfRange {
        block: u32,
        expected_min: u32,
        expected_max: u32,
    },
    /// A valid block index in range has no manifest for this role: `expected_occurrences` is 1,
    /// `observed_occurrences` is 0.
    Missing {
        block: u32,
        expected_occurrences: usize,
        observed_occurrences: usize,
    },
    /// A valid block index has more than one manifest for this role: `expected_occurrences` is 1,
    /// `observed_occurrences` is the actual count.
    Duplicate {
        block: u32,
        expected_occurrences: usize,
        observed_occurrences: usize,
    },
}
