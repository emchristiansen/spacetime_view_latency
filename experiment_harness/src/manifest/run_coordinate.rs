//! The per-run identity coordinate recorded in a manifest.

use serde::Serialize;

use crate::plan::cell::Cell;
use crate::plan::run::Run;
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

    /// The cell this run belongs to.
    pub(crate) fn cell(&self) -> Cell {
        self.cell
    }

    /// Whether this run is the arm or its matched control.
    pub(crate) fn role(&self) -> RunRole {
        self.role
    }

    /// The single [`Run`] this coordinate identifies: the cell's matched arm or control, selected by
    /// [`Self::role`]. The `(cell, role)` a coordinate already carries *fully determines* the run — the
    /// control table is a pure function of the cell — so this is the sole derivation, letting the
    /// coordinate be the one carrier of run identity rather than emitting a parallel [`Run`] value that
    /// could drift from it.
    pub(crate) fn run(&self) -> Run {
        let [arm, control] = self.cell.matched_runs();
        match self.role {
            RunRole::Arm => arm,
            RunRole::Control => control,
        }
    }

    /// A deterministic test fixture coordinate: the key-scoped point-filter arm (F) under own-slice
    /// growth, the arm role, and the first repetition block. Builds the private fields directly so
    /// tests needing a manifest/observation coordinate do not have to route through the schedule-owned
    /// [`BlockRun`]. The own-slice regime is chosen deliberately: it is the sole regime in which the
    /// driving role *is* the measured subscriber, so a whole internally coherent observation (the
    /// measured slice's own brand-new inserts) can be assembled from it. Test-only — never a
    /// production construction path.
    #[cfg(test)]
    pub(crate) fn fixture() -> Self {
        use crate::plan::key_scoped_arm::KeyScopedArm;

        Self {
            cell: Cell::KeyScopedOwnSlice(KeyScopedArm::PointFilter),
            role: RunRole::Arm,
            repetition_block: 0,
        }
    }
}
