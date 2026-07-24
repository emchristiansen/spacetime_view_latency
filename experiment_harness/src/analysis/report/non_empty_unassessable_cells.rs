//! A non-empty structural product of unassessable cells for the escalation gate.

use serde::Serialize;

use crate::analysis::report::unassessable_cell::UnassessableCell;

/// A non-empty list of [`UnassessableCell`]s (spec: "`NonEmptyUnassessableCells` ... structural
/// `{ first, rest }` products, never vectors guarded only by runtime assertions"). The mandatory
/// `first` field alone proves non-emptiness at the type level.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct NonEmptyUnassessableCells {
    first: UnassessableCell,
    rest: Box<[UnassessableCell]>,
}

impl NonEmptyUnassessableCells {
    /// Build a non-empty unassessable-cell list from its mandatory first element and the remaining
    /// (possibly empty) cells.
    pub(crate) fn new(first: UnassessableCell, rest: Box<[UnassessableCell]>) -> Self {
        Self { first, rest }
    }

    /// The mandatory first unassessable cell.
    pub(crate) fn first(&self) -> &UnassessableCell {
        &self.first
    }

    /// The remaining unassessable cells, if any.
    pub(crate) fn rest(&self) -> &[UnassessableCell] {
        &self.rest
    }
}
