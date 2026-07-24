//! One executable run within a repetition block.
//!
//! A [`Run`] is obtainable only through [`crate::plan::cell::Cell::matched_runs`];
//! its fields are private and it has no public constructor. Because the control
//! table is derived from the cell at construction, a run always carries the control
//! the spec pairs with its arm — a caller cannot assemble a `Run` with a mismatched
//! control, which a public `Cell × RunRole` product would allow.

use crate::plan::cell::Cell;
use crate::plan::control_table::ControlTable;
use crate::plan::run_role::RunRole;

/// A single arm-or-control run of the full dose ladder inside one block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Run {
    cell: Cell,
    role: RunRole,
    control_table: ControlTable,
}

impl Run {
    /// Build the matched arm/control pair for a cell: the arm run first, then its
    /// control-only run. Private to the plan module; the sole entry point is
    /// [`Cell::matched_runs`].
    pub(super) fn matched_pair(cell: Cell) -> [Run; 2] {
        let control_table = cell.control_table();
        [
            Run {
                cell,
                role: RunRole::Arm,
                control_table,
            },
            Run {
                cell,
                role: RunRole::Control,
                control_table,
            },
        ]
    }

    /// The cell this run belongs to.
    pub fn cell(self) -> Cell {
        self.cell
    }

    /// Whether this is the arm run or the control run.
    pub fn role(self) -> RunRole {
        self.role
    }

    /// The base table the matched control subscribes to.
    pub fn control_table(self) -> ControlTable {
        self.control_table
    }
}
