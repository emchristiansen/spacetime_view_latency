//! A Chronicle-family dose is `BATCH_SIZE` logical pairs whose physical footprint is 2×.

use std::collections::BTreeSet;

use spacetimedb_sdk::Identity;

use crate::dataset::campaign_dataset::CampaignDataset;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::dataset::seed_op::SeedOp;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::params::{BATCH_SIZE, GROWTH_KEY_BASE, M_SLICE_ROWS, NUM_DOSES};
use crate::plan::cell::Cell;
use crate::plan::control_table::ControlTable;
use crate::plan::table_scoped_arm::TableScopedArm;
use crate::roles::role_identities::RoleIdentities;

/// For a Chronicle-family cell, `resolve` derives the `ChronicleMessage` family and each dose is
/// exactly `BATCH_SIZE` `ChroniclePair` ops with unique pair keys, reporting `dose * BATCH_SIZE`
/// cumulative **logical** rows (the x-axis). The post-dose physical footprint is read from the
/// production [`CampaignDataset::physical_cardinalities`] API, which returns a
/// [`PhysicalCardinalities::Chronicle`] whose `message_visibility` and `chronicle_message` counts
/// are equal (the pinned M background plus the cumulative driving rows) and whose total is `2×`.
/// The physical counts come from that typed API — never from treating `operations().len()` (the
/// logical pair count) as a physical row count.
#[test]
fn chronicle_dose_pairs_carry_logical_pair_semantics() {
    const SEED: u64 = 55;
    let cell = Cell::TableScopedUnrelated(TableScopedArm::QuerySemijoin);
    assert_eq!(
        cell.control_table(),
        ControlTable::ChronicleMessage,
        "fixture must be Chronicle-family"
    );

    let measured = Identity::from_claims("test-measured", "chronicle");
    let identities = RoleIdentities::resolve(measured, ScheduleSeed::new(SEED), cell)
        .expect("measured and growth identities must be distinct");
    let dataset = CampaignDataset::resolve(cell.matched_runs()[0], &identities);
    assert_eq!(dataset.family(), ControlTable::ChronicleMessage);

    let mut all_pair_keys = BTreeSet::new();
    let mut expected_dose = 1u64;
    dataset
        .for_each_dose(|batch| {
            let ops = batch.operations();

            // Every op is a Chronicle pair; the op count is the LOGICAL pair count, not physical.
            let logical_pairs = ops.len();
            assert_eq!(
                logical_pairs, BATCH_SIZE as usize,
                "each dose is exactly BATCH_SIZE logical pairs"
            );
            assert_eq!(
                batch.cumulative_driving_rows(),
                expected_dose * BATCH_SIZE,
                "cumulative logical rows advance by BATCH_SIZE per dose"
            );

            for op in ops {
                match op {
                    SeedOp::ChroniclePair { key, viewer } => {
                        assert_eq!(viewer, identities.growth(), "doses are attributed to G");
                        assert!(
                            key >= GROWTH_KEY_BASE,
                            "pairs are keyed in the growth space"
                        );
                        assert!(
                            all_pair_keys.insert(key),
                            "pair keys are unique across the ladder"
                        );
                    }
                    other => panic!("expected a Chronicle-family op, got {other:?}"),
                }
            }

            // The production typed API records the physical table counts. The post-dose logical
            // rows are the pinned M background plus the cumulative driving rows; the Chronicle
            // family stores one visibility row and one message row per logical pair, so the two
            // per-table counts are equal and the total is 2×.
            let expected_logical = M_SLICE_ROWS + batch.cumulative_driving_rows();
            let cardinalities = dataset.physical_cardinalities(&batch);
            match cardinalities {
                PhysicalCardinalities::Chronicle(rows) => {
                    assert_eq!(
                        rows.visibility_rows(),
                        expected_logical,
                        "one message_visibility row per logical pair"
                    );
                    assert_eq!(
                        rows.message_rows(),
                        expected_logical,
                        "one chronicle_message row per logical pair"
                    );
                    assert_eq!(
                        rows.visibility_rows(),
                        rows.message_rows(),
                        "the Chronicle tables carry equal counts"
                    );
                }
                other => {
                    panic!("Chronicle family must yield Chronicle cardinalities, got {other:?}")
                }
            }
            assert_eq!(
                cardinalities.total_physical_rows(),
                2 * u128::from(expected_logical),
                "the physical footprint of a Chronicle dose is twice the logical pair count"
            );

            expected_dose += 1;
            Ok(())
        })
        .expect("ladder completes");

    assert_eq!(
        expected_dose,
        NUM_DOSES + 1,
        "exactly NUM_DOSES doses were delivered"
    );
    assert_eq!(
        all_pair_keys.len(),
        (NUM_DOSES * BATCH_SIZE) as usize,
        "the ladder's logical pairs have NUM_DOSES * BATCH_SIZE distinct keys"
    );
}
