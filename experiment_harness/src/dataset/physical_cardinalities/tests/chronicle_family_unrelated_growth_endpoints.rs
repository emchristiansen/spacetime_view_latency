//! Chronicle family under `UnrelatedGrowth`: exact equal visibility/message rows at the dose-1 and
//! dose-10 endpoints, pinned to preregistered constants (pinned M baseline `M_SLICE_ROWS`).

use crate::dataset::dose_index::DoseIndex;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::params::{BATCH_SIZE, M_SLICE_ROWS, NUM_DOSES, NUM_DOSES_USIZE};
use crate::plan::cell::Cell;
use crate::plan::table_scoped_arm::TableScopedArm;

#[test]
fn chronicle_family_unrelated_growth_endpoints() {
    // Arm C: a `chronicle_message`-family, unrelated-growth cell.
    let cell = Cell::TableScopedUnrelated(TableScopedArm::QuerySemijoin);

    assert_chronicle_rows(cell, DoseIndex::ALL[0], M_SLICE_ROWS + BATCH_SIZE);
    assert_chronicle_rows(
        cell,
        DoseIndex::ALL[NUM_DOSES_USIZE - 1],
        M_SLICE_ROWS + NUM_DOSES * BATCH_SIZE,
    );
}

fn assert_chronicle_rows(cell: Cell, dose: DoseIndex, expected_pairs: u64) {
    let cardinalities = PhysicalCardinalities::expected(cell, dose);
    match cardinalities {
        PhysicalCardinalities::Chronicle(rows) => {
            assert_eq!(
                rows.visibility_rows(),
                expected_pairs,
                "one message_visibility row per logical pair"
            );
            assert_eq!(
                rows.message_rows(),
                expected_pairs,
                "one chronicle_message row per logical pair, equal to the visibility count"
            );
        }
        other => panic!("Chronicle family must yield Chronicle cardinalities, got {other:?}"),
    }
    assert_eq!(
        cardinalities.total_physical_rows(),
        u128::from(expected_pairs) * 2,
        "a two-table Chronicle family's footprint is exactly 2× the logical-pair count"
    );
}
