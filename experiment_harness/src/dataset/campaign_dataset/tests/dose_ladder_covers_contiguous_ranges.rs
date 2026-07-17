//! The dose ladder delivers ten monotonic doses whose keys tile one contiguous range.

use std::collections::BTreeSet;

use spacetimedb_sdk::Identity;

use crate::dataset::campaign_dataset::CampaignDataset;
use crate::dataset::seed_op::SeedOp;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::params::{BATCH_SIZE, GROWTH_KEY_BASE, NUM_DOSES};
use crate::plan::cell::Cell;
use crate::plan::table_scoped_arm::TableScopedArm;
use crate::roles::role_identities::RoleIdentities;

/// The ladder yields exactly `NUM_DOSES` doses, in monotonic `1..=NUM_DOSES` order; each dose
/// writes exactly `BATCH_SIZE` keys and reports `dose * BATCH_SIZE` cumulative logical rows; and
/// the doses' keys tile the contiguous, gap-free range
/// `[driving_key_base, driving_key_base + NUM_DOSES * BATCH_SIZE)` with no overlap. Verified for
/// the `UnrelatedGrowth` regime, where the driving key base is `GROWTH_KEY_BASE`.
#[test]
fn dose_ladder_covers_contiguous_ranges() {
    const SEED: u64 = 33;
    let cell = Cell::TableScopedUnrelated(TableScopedArm::ProceduralRange);
    let measured = Identity::from_claims("test-measured", "ladder");
    let identities = RoleIdentities::resolve(measured, ScheduleSeed::new(SEED), cell)
        .expect("measured and growth identities must be distinct");
    let dataset = CampaignDataset::resolve(cell.matched_runs()[0], &identities);

    let mut all_keys = BTreeSet::new();
    let mut expected_dose = 1u64;
    dataset
        .for_each_dose(|batch| {
            assert_eq!(
                batch.dose().get(),
                expected_dose,
                "doses arrive in monotonic ladder order"
            );
            assert_eq!(
                batch.cumulative_driving_rows(),
                expected_dose * BATCH_SIZE,
                "cumulative logical rows advance by BATCH_SIZE per dose"
            );
            let ops = batch.operations();
            assert_eq!(
                ops.len(),
                BATCH_SIZE as usize,
                "each dose writes exactly BATCH_SIZE rows"
            );
            for op in ops {
                match op {
                    SeedOp::Message { id, .. } => assert!(
                        all_keys.insert(id),
                        "dose keys never overlap across the ladder"
                    ),
                    other => panic!("expected a message-family op, got {other:?}"),
                }
            }
            expected_dose += 1;
            Ok(())
        })
        .expect("ladder completes");

    assert_eq!(
        expected_dose,
        NUM_DOSES + 1,
        "exactly NUM_DOSES doses were delivered"
    );
    let expected: BTreeSet<u64> =
        (GROWTH_KEY_BASE..GROWTH_KEY_BASE + NUM_DOSES * BATCH_SIZE).collect();
    assert_eq!(
        all_keys, expected,
        "the ladder tiles one contiguous, gap-free key range"
    );
}
