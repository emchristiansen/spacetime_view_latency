//! One scheduled repetition block for a single cell.

use sha2::{Digest, Sha256};

use crate::params::ARM_CONTROL_ORDER_DOMAIN;
use crate::plan::cell::Cell;
use crate::plan::run::Run;

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

    /// This block's two runs — its arm and its matched direct-table control — in the
    /// seed-randomized order they execute in, returned together as one array so the
    /// matched pair's adjacency is structural rather than caller-maintained (spec: "The
    /// arm/control order within each block is randomized, and the two runs are adjacent
    /// except for the fresh-server reset"). The pair is always exactly `{Arm, Control}`
    /// for this block's cell; only their order varies with the seed.
    pub fn ordered_runs(&self, seed: u64) -> [Run; 2] {
        let [arm, control] = self.cell.matched_runs();
        if self.control_first(seed) {
            [control, arm]
        } else {
            [arm, control]
        }
    }

    /// Whether this block's matched control run executes before its arm run, derived
    /// deterministically from `seed` and the block's coordinate under a domain distinct
    /// from the global block-order permutation so the two decisions do not correlate.
    fn control_first(&self, seed: u64) -> bool {
        let tag = self.cell.canonical_tag();
        let block_index = self.block_index;
        let subject = format!(
            "{};seed={};cell={};block={}",
            ARM_CONTROL_ORDER_DOMAIN, seed, tag, block_index
        );
        let mut hasher = Sha256::new();
        hasher.update(subject.as_bytes());
        let digest: [u8; 32] = hasher.finalize().into();
        digest[0] & 1 == 1
    }
}
