//! No attempt can name a scale point off the existing frozen ladder.

use crate::control_registry_discovery_screen::attempt_inventory::AttemptInventory;
use crate::control_registry_discovery_screen::screen_params::{
    HIGH_RUNG_INDEX, LOW_RUNG_INDEX, REGISTRY_CONTROLS,
};
use crate::control_registry_discovery_screen::screen_rung::ScreenRung;
use crate::entity_owner_pilot::pilot_params::GLOBAL_ROW_LADDER;
use crate::manifest::schedule_seed::ScheduleSeed;

/// Coverage: every predeclared rung resolves to a position on the pre-existing unrelated/global
/// ladder, and to the two endpoints the freeze names.
///
/// The structural guarantee is stronger than this test — a `GlobalRowRung` can only be obtained from
/// its own frozen `ALL` table, so an off-ladder rung is unconstructable rather than merely unwritten
/// — but the test pins the *correspondence*: that `ScreenRung::Low`/`High` still mean ladder
/// positions 0 and 5, and 1,000 and 32,000 rows. A future edit that repointed them at other ladder
/// positions would remain type-correct and would be caught only here.
///
/// Also asserts exact divisibility at both endpoints, so the per-control seeding depth the driver
/// derives never truncates.
#[test]
fn every_rung_is_a_frozen_ladder_endpoint() {
    let frozen = AttemptInventory::frozen(ScheduleSeed::new(7)).expect("the seed freezes");

    for attempt in frozen.attempts() {
        let rung = attempt.rung();
        let ladder_index = rung.global_rung().get();

        assert!(
            ladder_index < GLOBAL_ROW_LADDER.len(),
            "rung {rung:?} resolved to ladder position {ladder_index}, off the frozen ladder"
        );
        assert_eq!(
            rung.history_rows(),
            GLOBAL_ROW_LADDER[ladder_index],
            "rung {rung:?} must report the ladder's own row count at position {ladder_index}"
        );
        assert!(
            ladder_index == LOW_RUNG_INDEX || ladder_index == HIGH_RUNG_INDEX,
            "rung {rung:?} is at ladder position {ladder_index}, not one of the two frozen \
             endpoints"
        );
        assert_eq!(
            rung.rows_per_control() * REGISTRY_CONTROLS,
            rung.history_rows(),
            "rung {rung:?} must divide exactly into {REGISTRY_CONTROLS} controls"
        );
    }

    assert_eq!(ScreenRung::Low.history_rows(), 1_000);
    assert_eq!(ScreenRung::High.history_rows(), 32_000);
    assert_eq!(ScreenRung::Low.rows_per_control(), 100);
    assert_eq!(ScreenRung::High.rows_per_control(), 3_200);
}
