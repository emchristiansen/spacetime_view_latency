//! A message-family dose occupies one physical table: physical rows equal logical rows.

use spacetimedb_sdk::Identity;

use crate::dataset::campaign_dataset::CampaignDataset;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::params::{NUM_DOSES, OWN_SLICE_BASELINE};
use crate::plan::cell::Cell;
use crate::plan::control_table::ControlTable;
use crate::plan::key_scoped_arm::KeyScopedArm;
use crate::roles::role_identities::RoleIdentities;

/// The message-family counterpart of the Chronicle cardinality test. For a `message`-family cell
/// the production API yields a [`PhysicalCardinalities::Message`] — a single-table count with no
/// visibility component even representable — whose row count is the pinned background baseline plus
/// the cumulative driving logical rows, and whose total footprint is 1× that count. Uses the
/// `OwnSliceGrowth` regime so the pinned baseline is `OWN_SLICE_BASELINE`, complementing the
/// Chronicle test's `UnrelatedGrowth` `M_SLICE_ROWS` baseline.
#[test]
fn message_dose_rows_are_single_table_physical() {
    const SEED: u64 = 66;
    let cell = Cell::KeyScopedOwnSlice(KeyScopedArm::PointFilter);
    assert_eq!(
        cell.control_table(),
        ControlTable::Message,
        "fixture must be message-family"
    );

    let measured = Identity::from_claims("test-measured", "message-cardinality");
    let identities = RoleIdentities::resolve(measured, ScheduleSeed::new(SEED), cell)
        .expect("measured and growth identities must be distinct");
    let dataset = CampaignDataset::resolve(cell.matched_runs()[0], &identities);

    let mut expected_dose = 1u64;
    dataset
        .for_each_dose(|batch| {
            let expected_rows = OWN_SLICE_BASELINE + batch.cumulative_driving_rows();
            let cardinalities = dataset.physical_cardinalities(&batch);
            match cardinalities {
                PhysicalCardinalities::Message(rows) => {
                    assert_eq!(
                        rows.message_rows(),
                        expected_rows,
                        "the single message table holds the pinned baseline plus cumulative rows"
                    );
                }
                other => panic!("message family must yield Message cardinalities, got {other:?}"),
            }
            assert_eq!(
                cardinalities.total_physical_rows(),
                u128::from(expected_rows),
                "a single-table family's footprint is exactly its message-row count (1×)"
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
}
