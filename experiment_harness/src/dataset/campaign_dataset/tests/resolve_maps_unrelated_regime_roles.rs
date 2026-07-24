//! `resolve` maps `UnrelatedGrowth` to a G-driven ladder over a pinned M background.

use spacetimedb_sdk::Identity;

use crate::dataset::campaign_dataset::CampaignDataset;
use crate::dataset::seed_op::SeedOp;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::params::{GROWTH_KEY_BASE, MEASURED_KEY_BASE, M_SLICE_ROWS};
use crate::plan::cell::Cell;
use crate::plan::control_table::ControlTable;
use crate::plan::table_scoped_arm::TableScopedArm;
use crate::roles::role_identities::RoleIdentities;

/// Under `UnrelatedGrowth` the growth driver G advances the ladder while the measured role M is
/// the pinned, unmeasured background of exactly `M_SLICE_ROWS` rows keyed from `MEASURED_KEY_BASE`.
/// Verified structurally: the derived family and driving-role tag, the background
/// cardinality/attribution/key range, and the identity a dose carries.
#[test]
fn resolve_maps_unrelated_regime_roles() {
    const SEED: u64 = 11;
    let cell = Cell::TableScopedUnrelated(TableScopedArm::ProceduralRange);
    assert_eq!(
        cell.control_table(),
        ControlTable::Message,
        "fixture must be message-family so ops carry a comparable (id, sender)"
    );

    let measured = Identity::from_claims("test-measured", "unrelated");
    let identities = RoleIdentities::resolve(measured, ScheduleSeed::new(SEED), cell)
        .expect("measured and growth identities must be distinct");
    let dataset = CampaignDataset::resolve(cell.matched_runs()[0], &identities);

    assert_eq!(dataset.family(), ControlTable::Message);
    assert_eq!(dataset.driving_role_tag(), "growth-driver");

    let background = dataset.background_operations();
    assert_eq!(
        background.len(),
        M_SLICE_ROWS as usize,
        "M's pinned background slice is exactly M_SLICE_ROWS rows"
    );
    for op in &background {
        match *op {
            SeedOp::Message { id, sender } => {
                assert_eq!(
                    sender,
                    identities.measured(),
                    "background is attributed to M"
                );
                assert!(
                    (MEASURED_KEY_BASE..MEASURED_KEY_BASE + M_SLICE_ROWS).contains(&id),
                    "background keys stay in M's slice"
                );
            }
            other => panic!("expected a message-family op, got {other:?}"),
        }
    }

    // The first dose is driven by G, writing in the growth key space.
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
                assert_eq!(sender, identities.growth(), "doses are attributed to G");
                assert!(id >= GROWTH_KEY_BASE, "doses write in the growth key space");
            }
            other => panic!("expected a message-family op, got {other:?}"),
        }
    }
}
