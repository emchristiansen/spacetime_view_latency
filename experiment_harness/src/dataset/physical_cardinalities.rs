//! The physical table row counts occupied by a run's dataset at a point in the ladder.

use serde::Serialize;

use crate::dataset::chronicle_physical_rows::ChroniclePhysicalRows;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::message_physical_rows::MessagePhysicalRows;
use crate::plan::cell::Cell;
use crate::plan::control_table::ControlTable;

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
    /// The deterministic physical cardinalities a run's dose must occupy, as a pure function of its
    /// [`Cell`] and [`DoseIndex`] — identity-free and role-independent, so a cell's matched arm and
    /// control share the exact same expectation. The logical footprint is the regime's held-constant
    /// pinned baseline ([`GrowthRegime::pinned_baseline_rows`](crate::plan::growth_regime::GrowthRegime::pinned_baseline_rows))
    /// plus this dose's cumulative driving rows ([`DoseIndex::cumulative_driving_rows`]), projected
    /// onto the cell's control-table family. This is the single canonical expectation:
    /// [`CampaignDataset::physical_cardinalities`](crate::dataset::campaign_dataset::CampaignDataset::physical_cardinalities)
    /// delegates here, and the analysis validation pass compares recorded cardinalities against it, so
    /// runtime and analysis can never compute a different expected footprint. The add is checked so a
    /// future constant change fails loud and identically in debug and release rather than wrapping.
    pub(crate) fn expected(cell: Cell, dose: DoseIndex) -> Self {
        let logical_rows = cell
            .growth_regime()
            .pinned_baseline_rows()
            .checked_add(dose.cumulative_driving_rows())
            .expect("dataset logical-row count must not overflow u64");
        match cell.control_table() {
            ControlTable::Message => {
                PhysicalCardinalities::Message(MessagePhysicalRows::from_logical_rows(logical_rows))
            }
            ControlTable::ChronicleMessage => PhysicalCardinalities::Chronicle(
                ChroniclePhysicalRows::from_logical_pairs(logical_rows),
            ),
        }
    }

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

#[cfg(test)]
mod tests;
