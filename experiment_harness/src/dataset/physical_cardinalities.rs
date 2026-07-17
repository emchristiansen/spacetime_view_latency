//! The physical table row counts occupied by a run's dataset at a point in the ladder.

use serde::Serialize;

use crate::dataset::chronicle_physical_rows::ChroniclePhysicalRows;
use crate::dataset::message_physical_rows::MessagePhysicalRows;

/// The physical row counts of a run's dataset, projected onto the family's actual tables.
///
/// The two families occupy different physical shapes, so this enum makes a family's impossible
/// table-count state unrepresentable: the single-table `message` family carries only a
/// [`MessagePhysicalRows`] (no visibility count exists to record), while the Chronicle family
/// carries a [`ChroniclePhysicalRows`] whose two per-table counts are always equal by
/// construction. There is deliberately no zero placeholder for a table a family does not use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum PhysicalCardinalities {
    /// The single-table `message` family.
    Message(MessagePhysicalRows),
    /// The two-table Chronicle family.
    Chronicle(ChroniclePhysicalRows),
}

impl PhysicalCardinalities {
    /// The total physical rows across every table the family uses, widened to `u128` so the
    /// Chronicle family's visibility + message sum cannot overflow — no debug-panic/release-wrap
    /// divergence. The `message` family's total is its single-table count.
    pub(crate) fn total_physical_rows(&self) -> u128 {
        match self {
            PhysicalCardinalities::Message(rows) => u128::from(rows.message_rows()),
            PhysicalCardinalities::Chronicle(rows) => {
                u128::from(rows.visibility_rows()) + u128::from(rows.message_rows())
            }
        }
    }
}
