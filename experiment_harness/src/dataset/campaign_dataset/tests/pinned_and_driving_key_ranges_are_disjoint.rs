//! In both regimes the pinned background and the driving ladder occupy disjoint key spaces.

use std::collections::BTreeSet;

use spacetimedb_sdk::Identity;

use crate::dataset::campaign_dataset::CampaignDataset;
use crate::dataset::seed_op::SeedOp;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::plan::cell::Cell;
use crate::plan::key_scoped_arm::KeyScopedArm;
use crate::plan::table_scoped_arm::TableScopedArm;
use crate::roles::role_identities::RoleIdentities;

/// The billion-key gap between `MEASURED_KEY_BASE` and `GROWTH_KEY_BASE` makes a primary-key
/// collision between the pinned background and the cumulative ladder structurally impossible over
/// the whole 10,000-key ladder. Verified in both regimes (which swap which role is pinned) by
/// materializing the full key sets and asserting disjointness.
#[test]
fn pinned_and_driving_key_ranges_are_disjoint() {
    const SEED: u64 = 44;
    let cells = [
        // UnrelatedGrowth: M pinned at the measured base, G drives from the growth base.
        Cell::TableScopedUnrelated(TableScopedArm::ProceduralRange),
        // OwnSliceGrowth: G pinned at the growth base, M2 drives from the measured base.
        Cell::KeyScopedOwnSlice(KeyScopedArm::PointFilter),
    ];

    for cell in cells {
        let measured = Identity::from_claims("test-measured", cell.canonical_tag());
        let identities = RoleIdentities::resolve(measured, ScheduleSeed::new(SEED), cell)
            .expect("measured and growth identities must be distinct");
        let dataset = CampaignDataset::resolve(cell.matched_runs()[0], &identities);

        let pinned: BTreeSet<u64> = dataset
            .background_operations()
            .into_iter()
            .map(message_key)
            .collect();
        let mut driving = BTreeSet::new();
        dataset
            .for_each_dose(|batch| {
                for op in batch.operations() {
                    driving.insert(message_key(op));
                }
                Ok(())
            })
            .expect("ladder completes");

        assert!(!pinned.is_empty(), "the pinned background must be nonempty");
        assert!(!driving.is_empty(), "the driving ladder must be nonempty");
        assert!(
            pinned.is_disjoint(&driving),
            "pinned background and driving ladder keys must not collide in {}",
            cell.canonical_tag()
        );
    }
}

/// Extract the primary key of a message-family seeding op. The test fixtures are message-family,
/// so a Chronicle op here is a fixture defect, not an expected case.
fn message_key(op: SeedOp) -> u64 {
    match op {
        SeedOp::Message { id, .. } => id,
        other => panic!("expected a message-family op, got {other:?}"),
    }
}
