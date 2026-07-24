//! A measured-slice target under `UnrelatedGrowth` stays pinned at M's fixed slice across every dose —
//! the flat-under-unrelated-growth positive control — pinned for both row families at dose 1 and dose 10.

use spacetimedb_sdk::Identity;

use crate::dataset::campaign_dataset::CampaignDataset;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::subscribed_rows::SubscribedRows;
use crate::dataset::subscribed_table::SubscribedTable;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::params::{MEASURED_KEY_BASE, M_SLICE_ROWS, NUM_DOSES_USIZE};
use crate::plan::cell::Cell;
use crate::plan::key_scoped_arm::KeyScopedArm;
use crate::plan::table_scoped_arm::TableScopedArm;
use crate::roles::role_identities::RoleIdentities;

/// Under unrelated growth a measured-slice target returns exactly M's fixed `M_SLICE_ROWS` rows,
/// unchanged as G grows. Table-driven over both row families — arm F's `message` point view and arm
/// C's `chronicle_message` semijoin view — and pinned at dose 1 and dose 10 by exact primary-key set.
#[test]
fn a_measured_slice_target_under_unrelated_growth_stays_pinned() {
    const SEED: u64 = 102;
    // Arm F's `message` point view and arm C's `chronicle_message` semijoin view (both arm runs):
    // measured-slice targets under unrelated growth, one per row family.
    let message_target = Cell::KeyScopedUnrelated(KeyScopedArm::PointFilter).matched_runs()[0];
    let chronicle_target =
        Cell::TableScopedUnrelated(TableScopedArm::QuerySemijoin).matched_runs()[0];

    for run in [message_target, chronicle_target] {
        let measured = Identity::from_claims("test-measured", "slice-unrelated");
        let identities = RoleIdentities::resolve(measured, ScheduleSeed::new(SEED), run.cell())
            .expect("measured and growth identities must be distinct");
        let dataset = CampaignDataset::resolve(run, &identities);
        let target = SubscribedTable::from_run(run);

        for dose in [DoseIndex::ALL[0], DoseIndex::ALL[NUM_DOSES_USIZE - 1]] {
            let n = dose.get();
            // M's slice only, pinned at exactly M_SLICE_ROWS rows regardless of the dose.
            let mut expected: Vec<u64> =
                (MEASURED_KEY_BASE..MEASURED_KEY_BASE + M_SLICE_ROWS).collect();
            expected.sort_unstable();

            let actual = subscribed_keys(&target.expected_through_dose(&dataset, dose));
            assert_eq!(
                actual, expected,
                "measured-slice {run:?} at dose {n}: expected exactly M's pinned slice, independent of G"
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
