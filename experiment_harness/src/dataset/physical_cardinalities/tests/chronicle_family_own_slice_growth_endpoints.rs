//! Chronicle family under `OwnSliceGrowth`: exact equal visibility/message rows at the dose-1 and
//! dose-10 endpoints, pinned to preregistered constants (pinned G baseline `OWN_SLICE_BASELINE`).

use crate::dataset::dose_index::DoseIndex;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::params::{BATCH_SIZE, NUM_DOSES, NUM_DOSES_USIZE, OWN_SLICE_BASELINE};
use crate::plan::cell::Cell;
use crate::plan::key_scoped_arm::KeyScopedArm;

#[test]
fn chronicle_family_own_slice_growth_endpoints() {
    // Arm F′ under own-slice growth: a `chronicle_message`-family cell with an OWN_SLICE_BASELINE pin.
    let cell = Cell::KeyScopedOwnSlice(KeyScopedArm::PointSemijoin);

    assert_chronicle_rows(cell, DoseIndex::ALL[0], OWN_SLICE_BASELINE + BATCH_SIZE);
    assert_chronicle_rows(
        cell,
        DoseIndex::ALL[NUM_DOSES_USIZE - 1],
        OWN_SLICE_BASELINE + NUM_DOSES * BATCH_SIZE,
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
