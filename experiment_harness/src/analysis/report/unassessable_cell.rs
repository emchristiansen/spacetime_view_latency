//! One cell the escalation gate cannot assess, paired with its typed reason.

use serde::Serialize;

use crate::analysis::report::unassessable_reason::UnassessableReason;
use crate::plan::cell::Cell;

/// One cell the stock-to-patch escalation gate retains without a prediction comparison (spec:
/// "`EscalateToPatch { ..., unassessable_cells: Box<[UnassessableCell]> }`"; "retain every
/// co-occurring invalid-control or inconclusive cell instead of discarding it"). Pairs the canonical
/// cell identity with the closed reason it could not be assessed, so the gate never discards this
/// evidence by folding it into a bare count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct UnassessableCell {
    cell: Cell,
    reason: UnassessableReason,
}

impl UnassessableCell {
    /// Pair a cell with the reason it is unassessable. Two inputs, both required: a cell can never be
    /// retained here without its typed reason.
    pub(crate) fn new(cell: Cell, reason: UnassessableReason) -> Self {
        Self { cell, reason }
    }

    /// The unassessable cell's canonical identity.
    pub(crate) fn cell(&self) -> Cell {
        self.cell
    }

    /// Why this cell is unassessable.
    pub(crate) fn reason(&self) -> UnassessableReason {
        self.reason
    }
}
