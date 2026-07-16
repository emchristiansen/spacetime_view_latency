//! The schedulable plan.

use crate::params::REPETITION_BLOCKS;
use crate::plan::cell::Cell;

use super::BlockRun;

/// The schedulable plan. Stores **only** [`Cell`] values; per-run structure (the
/// matched arm and control runs) is derived at execution time via
/// [`Cell::matched_runs`], and the growth regime / predicted response are derived
/// from each cell — never stored alongside it.
///
/// `Schedule` is the sole constructor of [`BlockRun`]s, and [`Self::all_blocks`]
/// builds exactly every `Cell × 0..BLOCKS_PER_CELL` product once. That guarantees the
/// canonical set *as constructed* is complete with no missing, duplicate, or
/// out-of-range blocks; it does not constrain what a caller does with the owned
/// vector afterward (see [`Self::randomized_block_order`]).
pub struct Schedule {
    cells: Vec<Cell>,
}

impl Schedule {
    /// Number of complete randomized repetition blocks to run per cell.
    pub const BLOCKS_PER_CELL: u32 = REPETITION_BLOCKS;

    /// The full preregistered plan: every valid cell.
    pub fn preregistered() -> Self {
        Self { cells: Cell::all() }
    }

    /// The cells in this plan.
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// The complete, canonical (unrandomized) block set: every `(cell, block-index)`
    /// product exactly once, each cell paired with indices `0..BLOCKS_PER_CELL`.
    fn all_blocks(&self) -> Vec<BlockRun> {
        self.cells
            .iter()
            .flat_map(|&cell| (0..Self::BLOCKS_PER_CELL).map(move |block_index| BlockRun::new(cell, block_index)))
            .collect()
    }

    /// A vector *constructed* from the exhaustive `Cell × 0..BLOCKS_PER_CELL` product
    /// ([`Self::all_blocks`]), globally reordered from an explicit seed so slow-host
    /// drift cannot align with one cell (spec: "Controls and execution").
    ///
    /// Enforced fact: the returned vector is built from the complete internal product,
    /// and external callers cannot mint additional [`BlockRun`] values. It is *not*
    /// enforced that the caller runs every block: the vector is owned and its entries
    /// can be dropped, so executing each block exactly once is a Phase 2 runtime/report
    /// obligation. The Phase 2 permutation must reorder without adding, dropping, or
    /// duplicating a block.
    pub fn randomized_block_order(&self, seed: u64) -> Vec<BlockRun> {
        Self::seeded_permutation(self.all_blocks(), seed)
    }

    /// Deterministic seeded permutation of an exact block set. Stubbed in Phase 1.
    fn seeded_permutation(_blocks: Vec<BlockRun>, _seed: u64) -> Vec<BlockRun> {
        todo!("deterministic seeded permutation of the complete block set")
    }
}
