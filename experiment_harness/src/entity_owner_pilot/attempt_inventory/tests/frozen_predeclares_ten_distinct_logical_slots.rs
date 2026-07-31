//! A frozen inventory holds exactly ten attempts, no two addressing the same logical slot.

use crate::entity_owner_pilot::attempt_inventory::{AttemptInventory, PILOT_ATTEMPT_COUNT};
use crate::manifest::schedule_seed::ScheduleSeed;

/// Coverage: five matched blocks times two roles is ten predeclared attempts, and every one names a
/// distinct logical slot. The distinctness half is what a bare length check would miss — a
/// permutation emitting one block twice and another never still produces ten attempts, and the
/// campaign would silently measure the wrong design.
///
/// Checked at several seeds because the permutation is the thing that could break this, and it is a
/// function of the seed.
#[test]
fn frozen_predeclares_ten_distinct_logical_slots() {
    const SEEDS: [u64; 4] = [0, 1, 7, 0xDEAD_BEEF];

    assert_eq!(
        PILOT_ATTEMPT_COUNT, 10,
        "the Pilot's five matched blocks predeclare ten attempts"
    );

    for seed in SEEDS {
        let frozen = AttemptInventory::frozen(ScheduleSeed::new(seed)).expect("the seed freezes");
        let attempts = frozen.attempts();

        assert_eq!(
            attempts.len(),
            PILOT_ATTEMPT_COUNT,
            "seed {seed} must predeclare exactly {PILOT_ATTEMPT_COUNT} attempts"
        );
        for (position, attempt) in attempts.iter().enumerate() {
            for (other_position, other) in attempts.iter().enumerate().skip(position + 1) {
                assert!(
                    !attempt.same_logical_slot(*other),
                    "seed {seed} repeats a logical slot at positions {position} and \
                     {other_position}"
                );
            }
        }
    }
}
