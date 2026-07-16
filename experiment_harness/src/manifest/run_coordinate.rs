//! The per-run identity coordinate recorded in a manifest.

use serde::Serialize;

use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;
use crate::plan::schedule::BlockRun;

/// Which run a manifest describes, sourced entirely from schedule-owned typed values so
/// no invalid coordinate is representable.
///
/// Built from a [`BlockRun`] — the schedule's sole valid `(cell, block-index)` value,
/// whose block index is guaranteed in range and whose [`Cell`] rules out impossible
/// arm/regime pairings — plus the [`RunRole`] selecting the arm or its matched control.
/// Only the non-redundant identity is stored: the growth regime, predicted response, and
/// control table are all pure functions of the [`Cell`], so serializing them would add
/// drift surface without information.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct RunCoordinate {
    /// The valid `(arm, growth-regime)` pairing under test.
    cell: Cell,
    /// Whether this run exercises the arm or its matched direct-table control.
    role: RunRole,
    /// 0-based repetition-block index within the cell's fixed block sample (in range by
    /// construction of [`BlockRun`]).
    repetition_block: u32,
}

impl RunCoordinate {
    /// Snapshot the identity of a single run within a scheduled block.
    pub(crate) fn new(block_run: &BlockRun, role: RunRole) -> Self {
        Self {
            cell: block_run.cell(),
            role,
            repetition_block: block_run.block_index(),
        }
    }
}
