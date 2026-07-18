//! Message family under `OwnSliceGrowth`: exact single-table rows at the dose-1 and dose-10
//! endpoints, pinned to preregistered constants (pinned G baseline `OWN_SLICE_BASELINE`).

use crate::dataset::dose_index::DoseIndex;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::params::{BATCH_SIZE, NUM_DOSES, NUM_DOSES_USIZE, OWN_SLICE_BASELINE};
use crate::plan::cell::Cell;
use crate::plan::key_scoped_arm::KeyScopedArm;

#[test]
fn message_family_own_slice_growth_endpoints() {
    // Arm F under own-slice growth: a `message`-family cell whose pinned baseline is OWN_SLICE_BASELINE.
    let cell = Cell::KeyScopedOwnSlice(KeyScopedArm::PointFilter);

    assert_message_rows(cell, DoseIndex::ALL[0], OWN_SLICE_BASELINE + BATCH_SIZE);
    assert_message_rows(
        cell,
        DoseIndex::ALL[NUM_DOSES_USIZE - 1],
        OWN_SLICE_BASELINE + NUM_DOSES * BATCH_SIZE,
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
