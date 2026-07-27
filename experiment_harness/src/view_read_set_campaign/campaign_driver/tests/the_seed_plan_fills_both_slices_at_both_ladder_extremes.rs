//! The seed plan lays down exactly the two frozen key ranges, each under its own owner, at both ends
//! of the ladder.

use spacetimedb_sdk::Identity;

use crate::params::EXPERIMENT_ISSUER;
use crate::view_read_set_campaign::campaign_driver::seed_plan;
use crate::view_read_set_campaign::campaign_params::{
    GLOBAL_IDENTITY_SUBJECT, GLOBAL_KEY_BASE, OWNED_KEY_BASE, SUBSCRIBER_VISIBLE_ROWS_BASELINE,
};
use crate::view_read_set_campaign::experiment_axis::ExperimentAxis;
use crate::view_read_set_campaign::scale_point::ScalePoint;

/// Coverage: the whole of what the plan decides — how many rows of each slice, which keys they
/// occupy, and which identity owns them. Asserted against the frozen constants rather than against
/// the composition expectation those constants also feed, because agreeing with that expectation is
/// exactly what a wrong plan must be able to fail at.
///
/// Both ladder extremes, because only the swept slice changes between them: a plan that sized the
/// wrong slice could still produce a plausible count at a middle rung.
///
/// Strictly ascending keys across the whole plan is the disjointness claim: every owned key precedes
/// every foreign one, and no key repeats, so no seeded row can collide with another's primary key.
#[test]
fn the_seed_plan_fills_both_slices_at_both_ladder_extremes() {
    let ladder = ScalePoint::ladder(ExperimentAxis::UnrelatedGlobalRows);
    assert!(
        !ladder.is_empty(),
        "the frozen ladder declares at least one rung, and the extremes below index it"
    );
    let owned_owner = Identity::from_claims(EXPERIMENT_ISSUER, "seed-plan-measured-subscriber");
    let foreign_owner = Identity::from_claims(EXPERIMENT_ISSUER, GLOBAL_IDENTITY_SUBJECT);
    assert_ne!(
        owned_owner, foreign_owner,
        "the two seeded identities must differ, or owner assignment is unfalsifiable"
    );

    for scale in [ladder[0], ladder[ladder.len() - 1]] {
        let rung = scale.scale();
        let plan = match seed_plan(scale, owned_owner, foreign_owner) {
            Ok(plan) => plan,
            Err(error) => panic!("rung {rung} must yield a seed plan: {error:#}"),
        };

        let expected_rows = SUBSCRIBER_VISIBLE_ROWS_BASELINE + rung;
        let seeded_rows = match u64::try_from(plan.len()) {
            Ok(seeded_rows) => seeded_rows,
            Err(error) => panic!("a seed plan's length must fit u64: {error}"),
        };
        assert_eq!(
            seeded_rows, expected_rows,
            "rung {rung} seeds its baseline owned slice plus its whole swept foreign slice"
        );

        let mut previous_key = None;
        for (position, row) in plan.iter().enumerate() {
            let position = match u64::try_from(position) {
                Ok(position) => position,
                Err(error) => panic!("a seed plan's position must fit u64: {error}"),
            };
            let (expected_key, expected_owner) = if position < SUBSCRIBER_VISIBLE_ROWS_BASELINE {
                (OWNED_KEY_BASE + position, owned_owner)
            } else {
                (
                    GLOBAL_KEY_BASE + (position - SUBSCRIBER_VISIBLE_ROWS_BASELINE),
                    foreign_owner,
                )
            };

            assert_eq!(
                row.entity_uuid, expected_key,
                "rung {rung}'s row {position} occupies its slice's contiguous key"
            );
            assert_eq!(
                row.owner, expected_owner,
                "rung {rung}'s row {position} is owned by the identity its slice belongs to"
            );
            if let Some(previous_key) = previous_key {
                assert!(
                    previous_key < row.entity_uuid,
                    "rung {rung}'s keys ascend strictly, so the two slices stay disjoint"
                );
            }
            previous_key = Some(row.entity_uuid);
        }
    }
}
