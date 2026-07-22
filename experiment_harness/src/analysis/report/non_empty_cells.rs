//! A non-empty structural product of contradicted cells for the escalation gate.

use serde::Serialize;

use crate::plan::cell::Cell;

/// A non-empty list of contradicted [`Cell`]s (spec: "`NonEmptyCells` ... structural `{ first, rest }`
/// products, never vectors guarded only by runtime assertions"). The mandatory `first` field alone
/// proves non-emptiness at the type level — there is no way to construct this value with zero cells.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct NonEmptyCells {
    first: Cell,
    rest: Box<[Cell]>,
}

impl NonEmptyCells {
    /// Build a non-empty contradicted-cell list from its mandatory first element and the remaining
    /// (possibly empty) cells.
    pub(crate) fn new(first: Cell, rest: Box<[Cell]>) -> Self {
        Self { first, rest }
    }

    /// The mandatory first contradicted cell.
    pub(crate) fn first(&self) -> Cell {
        self.first
    }

    /// The remaining contradicted cells, if any.
    pub(crate) fn rest(&self) -> &[Cell] {
        &self.rest
    }
}
