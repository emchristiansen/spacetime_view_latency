//! The stable, serializable identity of one scheduled repetition block.

use serde::Serialize;

use crate::plan::cell::Cell;

use super::BlockRun;

/// Which repetition block a frontier or completion receipt refers to: the cell under test and the
/// 0-based repetition-block index within that cell's fixed block sample.
///
/// Derived **only** from a schedule-owned [`BlockRun`] via [`Self::of`], so every `BlockCoordinate`
/// that exists names a block the schedule actually produced — its index is in range and its [`Cell`]
/// rules out impossible arm/regime pairings by construction of `BlockRun`. Unlike `BlockRun` (which is
/// neither `Clone` nor `Serialize` so it cannot be duplicated or emitted), this is a copyable,
/// serializable *coordinate*: it carries the block's identity for machine-readable evidence without
/// carrying the schedule's construction capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct BlockCoordinate {
    cell: Cell,
    repetition_block: u32,
}

impl BlockCoordinate {
    /// Derive the coordinate from a schedule-owned block. The only constructor: there is no raw
    /// `(cell, index)` path, so a `BlockCoordinate` cannot name a block the schedule did not build.
    pub fn of(block: &BlockRun) -> Self {
        Self {
            cell: block.cell(),
            repetition_block: block.block_index(),
        }
    }

    /// The cell measured in this block.
    pub fn cell(self) -> Cell {
        self.cell
    }

    /// The 0-based repetition-block index within the cell's fixed block sample.
    pub fn repetition_block(self) -> u32 {
        self.repetition_block
    }
}
