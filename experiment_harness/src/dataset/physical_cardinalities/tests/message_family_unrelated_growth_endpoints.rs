//! Message family under `UnrelatedGrowth`: exact single-table rows at the dose-1 and dose-10
//! endpoints, pinned to preregistered constants (pinned M baseline `M_SLICE_ROWS`).

use crate::dataset::dose_index::DoseIndex;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::params::{BATCH_SIZE, M_SLICE_ROWS, NUM_DOSES, NUM_DOSES_USIZE};
use crate::plan::cell::Cell;
use crate::plan::table_scoped_arm::TableScopedArm;

#[test]
fn message_family_unrelated_growth_endpoints() {
    // Arm A: a `message`-family, unrelated-growth cell.
    let cell = Cell::TableScopedUnrelated(TableScopedArm::ProceduralRange);

    // Independently re-derived from the raw constants: pinned baseline + cumulative driving rows.
    assert_message_rows(cell, DoseIndex::ALL[0], M_SLICE_ROWS + BATCH_SIZE);
    assert_message_rows(
        cell,
        DoseIndex::ALL[NUM_DOSES_USIZE - 1],
        M_SLICE_ROWS + NUM_DOSES * BATCH_SIZE,
    );
}

fn assert_message_rows(cell: Cell, dose: DoseIndex, expected_rows: u64) {
    let cardinalities = PhysicalCardinalities::expected(cell, dose);
    match cardinalities {
        PhysicalCardinalities::Message(rows) => assert_eq!(
            rows.message_rows(),
            expected_rows,
            "the single message table holds the pinned baseline plus cumulative driving rows"
        ),
        other => panic!("message family must yield Message cardinalities, got {other:?}"),
    }
    assert_eq!(
        cardinalities.total_physical_rows(),
        u128::from(expected_rows),
        "a single-table family's footprint is exactly its message-row count (1×)"
    );
}
