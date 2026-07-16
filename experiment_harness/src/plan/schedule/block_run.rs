//! One scheduled repetition block for a single cell.

use crate::plan::cell::Cell;

/// A single scheduled repetition block: run this cell's matched arm and control runs
/// once, as block number `block_index` of the cell's fixed block sample.
///
/// Not `Clone`/`Copy`, with private fields and a constructor confined to the
/// `schedule` subtree. The enforced invariant is narrow: no external caller can
/// *construct* a `BlockRun` or clone an existing value — [`Schedule`](super::Schedule)
/// is the sole constructor. This alone does **not** guarantee execution completeness:
/// a caller holding the returned `Vec<BlockRun>` can still drop entries, and can read
/// `cell()`/`block_index()` to repeat work. Guaranteeing every block runs exactly once
/// is a Phase 2 runtime/report obligation. Carries only the [`Cell`]; the arm/control
/// runs are derived at execution time via [`Cell::matched_runs`].
#[derive(Debug, PartialEq, Eq)]
pub struct BlockRun {
    cell: Cell,
    block_index: u32,
}

impl BlockRun {
    /// Construct a block run. `pub(super)`, so only the `schedule` subtree — i.e.
    /// [`Schedule`](super::Schedule) — builds these, as part of the exhaustive
    /// `Cell × 0..REPETITION_BLOCKS` set.
    pub(super) fn new(cell: Cell, block_index: u32) -> Self {
        Self { cell, block_index }
    }

    /// The cell measured in this block.
    pub fn cell(&self) -> Cell {
        self.cell
    }

    /// 0-based index of this block within the cell's fixed block sample.
    pub fn block_index(&self) -> u32 {
        self.block_index
    }
}
