//! One validated cell: its `Cell` and the complete 30-block matched sample.

use crate::analysis::validate::matched_block::MatchedBlock;
use crate::params::REPETITION_BLOCKS;
use crate::plan::cell::Cell;

/// The fixed preregistered per-cell block-sample size as an array length. Guarded by a compile-time
/// round-trip assertion rather than a bare `as` cast — mirroring
/// [`NUM_DOSES_USIZE`](crate::params::NUM_DOSES_USIZE) and
/// [`BATCH_SIZE_USIZE`](crate::params::BATCH_SIZE_USIZE) — so a platform on which
/// [`REPETITION_BLOCKS`] does not fit a `usize` fails to compile instead of silently truncating the
/// array length.
const BLOCKS_PER_CELL: usize = {
    let as_usize = REPETITION_BLOCKS as usize;
    assert!(
        as_usize as u32 == REPETITION_BLOCKS,
        "REPETITION_BLOCKS does not fit in usize on this platform"
    );
    as_usize
};

/// One preregistered cell's complete evidence, proven to contain exactly `REPETITION_BLOCKS` (30)
/// matched blocks — no early stop, no extension (spec: "Classification"). The distribution-free
/// order-statistic interval assumes this fixed sample, so the fixed array makes the block cardinality a
/// property of the type rather than of caller discipline.
///
/// The [`Cell`] is the production trusted `(arm, growth-regime)` pairing whose closed variant set
/// already rules out impossible pairings. Fields are private with no defaults; the module-owned
/// [`Self::new`] is the only assembler.
pub(crate) struct CellDataset {
    /// The preregistered `(arm, growth-regime)` pairing this dataset covers.
    cell: Cell,
    /// The complete fixed sample of matched arm/control blocks for this cell. Heap-owned as a boxed fixed
    /// array so the thirty-block cardinality remains a property of the type while the dataset's by-value
    /// footprint is one pointer — keeping the fold's stack frame bounded regardless of
    /// [`REPETITION_BLOCKS`].
    blocks: Box<[MatchedBlock; BLOCKS_PER_CELL]>,
}

impl CellDataset {
    /// Assemble a cell's dataset from its cell and its already-validated complete block sample. The
    /// caller (the validation pass) owns proving block-index completeness and arm/control presence;
    /// this constructor only binds the proven parts so the private fields cannot be populated from
    /// outside the validation boundary.
    pub(super) fn new(cell: Cell, blocks: Box<[MatchedBlock; BLOCKS_PER_CELL]>) -> Self {
        Self { cell, blocks }
    }

    /// The preregistered `(arm, growth-regime)` pairing this dataset covers.
    pub(crate) fn cell(&self) -> Cell {
        self.cell
    }

    /// The complete fixed sample of matched arm/control blocks for this cell — the classifier's
    /// per-cell evidence source.
    pub(crate) fn blocks(&self) -> &[MatchedBlock; BLOCKS_PER_CELL] {
        &self.blocks
    }
}
