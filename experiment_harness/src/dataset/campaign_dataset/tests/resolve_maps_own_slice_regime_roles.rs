//! `resolve` maps `OwnSliceGrowth` to an M2-driven ladder over a pinned G background.

use spacetimedb_sdk::Identity;

use crate::dataset::campaign_dataset::CampaignDataset;
use crate::dataset::seed_op::SeedOp;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::params::{GROWTH_KEY_BASE, OWN_SLICE_BASELINE};
use crate::plan::cell::Cell;
use crate::plan::control_table::ControlTable;
use crate::plan::key_scoped_arm::KeyScopedArm;
use crate::roles::role_identities::RoleIdentities;

/// Under `OwnSliceGrowth` the measured role M2 advances the ladder while the growth driver G is
/// the pinned, unmeasured background of exactly `OWN_SLICE_BASELINE` rows keyed from
/// `GROWTH_KEY_BASE`. This is the mirror of the unrelated regime: pinned and driving roles swap.
#[test]
fn resolve_maps_own_slice_regime_roles() {
    const SEED: u64 = 22;
    let cell = Cell::KeyScopedOwnSlice(KeyScopedArm::PointFilter);
    assert_eq!(
        cell.control_table(),
        ControlTable::Message,
        "fixture must be message-family so ops carry a comparable (id, sender)"
    );

    let measured = Identity::from_claims("test-measured", "own-slice");
    let identities = RoleIdentities::resolve(measured, ScheduleSeed::new(SEED), cell)
        .expect("measured and growth identities must be distinct");
    let dataset = CampaignDataset::resolve(cell.matched_runs()[0], &identities);

    assert_eq!(dataset.family(), ControlTable::Message);
    assert_eq!(dataset.driving_role_tag(), "own-slice-measured");

    let background = dataset.background_operations();
    assert_eq!(
        background.len(),
        OWN_SLICE_BASELINE as usize,
        "G's pinned background slice is exactly OWN_SLICE_BASELINE rows"
    );
    for op in &background {
        match *op {
            SeedOp::Message { id, sender } => {
                assert_eq!(sender, identities.growth(), "the pinned background is G");
                assert!(
                    (GROWTH_KEY_BASE..GROWTH_KEY_BASE + OWN_SLICE_BASELINE).contains(&id),
                    "background keys stay in G's slice"
                );
            }
            other => panic!("expected a message-family op, got {other:?}"),
        }
    }

    // The first dose is driven by M2 (the measured identity), writing in the measured key space.
    let mut first_dose = Vec::new();
    dataset
        .for_each_dose(|batch| {
            if first_dose.is_empty() {
                first_dose = batch.operations();
            }
            Ok(())
        })
        .expect("ladder completes");
    for op in first_dose {
        match op {
            SeedOp::Message { id, sender } => {
                assert_eq!(
                    sender,
                    identities.measured(),
                    "the driving role is M2 (the measured identity)"
                );
                assert!(id < GROWTH_KEY_BASE, "M2 writes in the measured key space");
            }
            other => panic!("expected a message-family op, got {other:?}"),
        }
    }
}
