//! A full-table target under `UnrelatedGrowth` grows by exactly one driving batch per dose while its
//! pinned measured slice stays fixed — pinned for both row families at dose 1 and dose 10.

use spacetimedb_sdk::Identity;

use crate::dataset::campaign_dataset::CampaignDataset;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::subscribed_rows::SubscribedRows;
use crate::dataset::subscribed_table::SubscribedTable;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::params::{BATCH_SIZE, GROWTH_KEY_BASE, MEASURED_KEY_BASE, M_SLICE_ROWS, NUM_DOSES_USIZE};
use crate::plan::cell::Cell;
use crate::plan::table_scoped_arm::TableScopedArm;
use crate::roles::role_identities::RoleIdentities;

/// Under unrelated growth a full-table target returns M's fixed `M_SLICE_ROWS` rows plus G's
/// cumulative `dose * BATCH_SIZE` rows. Table-driven over both row families — arm B's `message` full
/// pass-through view and arm C's matched `chronicle_message` base control — and pinned at dose 1 and
/// dose 10 by exact primary-key set.
#[test]
fn a_full_table_target_under_unrelated_growth_grows_by_one_batch_per_dose() {
    const SEED: u64 = 101;
    // Arm B's `message` full view (arm run) and arm C's matched `chronicle_message` base (control run):
    // both full-table targets under unrelated growth, one per row family.
    let message_target = Cell::TableScopedUnrelated(TableScopedArm::QueryFull).matched_runs()[0];
    let chronicle_target =
        Cell::TableScopedUnrelated(TableScopedArm::QuerySemijoin).matched_runs()[1];

    for run in [message_target, chronicle_target] {
        let measured = Identity::from_claims("test-measured", "full-unrelated");
        let identities = RoleIdentities::resolve(measured, ScheduleSeed::new(SEED), run.cell())
            .expect("measured and growth identities must be distinct");
        let dataset = CampaignDataset::resolve(run, &identities);
        let target = SubscribedTable::from_run(run);

        for dose in [DoseIndex::ALL[0], DoseIndex::ALL[NUM_DOSES_USIZE - 1]] {
            let n = dose.get();
            // M's pinned slice (constant) plus G's cumulative slice (one batch per dose).
            let mut expected: Vec<u64> =
                (MEASURED_KEY_BASE..MEASURED_KEY_BASE + M_SLICE_ROWS).collect();
            expected.extend(GROWTH_KEY_BASE..GROWTH_KEY_BASE + n * BATCH_SIZE);
            expected.sort_unstable();

            let actual = subscribed_keys(&target.expected_through_dose(&dataset, dose));
            assert_eq!(
                actual, expected,
                "full-table {run:?} at dose {n}: expected M's pinned slice plus G's cumulative slice"
            );
        }
    }
}

/// The sorted primary keys of a subscribed result set, regardless of row family — `id` for `message`
/// rows, `uuid` for `chronicle_message` rows. Primary keys are unique and the two roles' key spaces
/// are a billion apart, so the key set alone identifies exactly which rows are present.
fn subscribed_keys(rows: &SubscribedRows) -> Vec<u64> {
    let mut keys: Vec<u64> = match rows {
        SubscribedRows::Message(rows) => rows.iter().map(|m| m.id).collect(),
        SubscribedRows::Chronicle(rows) => rows.iter().map(|m| m.uuid).collect(),
    };
    keys.sort_unstable();
    keys
}
