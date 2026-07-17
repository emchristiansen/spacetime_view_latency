//! A full-table target under `OwnSliceGrowth` grows with the measured driving slice while its pinned
//! unrelated (G) background stays fixed — pinned for both row families at dose 1 and dose 10.

use spacetimedb_sdk::Identity;

use crate::dataset::campaign_dataset::CampaignDataset;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::subscribed_rows::SubscribedRows;
use crate::dataset::subscribed_table::SubscribedTable;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::params::{
    BATCH_SIZE, GROWTH_KEY_BASE, MEASURED_KEY_BASE, NUM_DOSES_USIZE, OWN_SLICE_BASELINE,
};
use crate::plan::cell::Cell;
use crate::plan::key_scoped_arm::KeyScopedArm;
use crate::roles::role_identities::RoleIdentities;

/// Under own-slice growth a full-table target returns M2's cumulative `dose * BATCH_SIZE` rows plus
/// G's fixed `OWN_SLICE_BASELINE` background. Table-driven over both row families — the `message` and
/// `chronicle_message` base controls of the two key-scoped own-slice cells — and pinned at dose 1 and
/// dose 10 by exact primary-key set.
#[test]
fn a_full_table_target_under_own_slice_growth_grows_with_the_measured_slice() {
    const SEED: u64 = 103;
    // The `message` and `chronicle_message` base controls (control runs) of the own-slice cells: both
    // full-table targets under own-slice growth, one per row family.
    let message_target = Cell::KeyScopedOwnSlice(KeyScopedArm::PointFilter).matched_runs()[1];
    let chronicle_target = Cell::KeyScopedOwnSlice(KeyScopedArm::PointSemijoin).matched_runs()[1];

    for run in [message_target, chronicle_target] {
        let measured = Identity::from_claims("test-measured", "full-ownslice");
        let identities = RoleIdentities::resolve(measured, ScheduleSeed::new(SEED), run.cell())
            .expect("measured and growth identities must be distinct");
        let dataset = CampaignDataset::resolve(run, &identities);
        let target = SubscribedTable::from_run(run);

        for dose in [DoseIndex::ALL[0], DoseIndex::ALL[NUM_DOSES_USIZE - 1]] {
            let n = dose.get();
            // M2's cumulative slice (one batch per dose) plus G's pinned unrelated background.
            let mut expected: Vec<u64> =
                (MEASURED_KEY_BASE..MEASURED_KEY_BASE + n * BATCH_SIZE).collect();
            expected.extend(GROWTH_KEY_BASE..GROWTH_KEY_BASE + OWN_SLICE_BASELINE);
            expected.sort_unstable();

            let actual = subscribed_keys(&target.expected_through_dose(&dataset, dose));
            assert_eq!(
                actual, expected,
                "full-table {run:?} at dose {n}: expected M2's cumulative slice plus G's pinned background"
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
