//! A frozen inventory holds exactly sixteen attempts, no two addressing the same logical slot.

use crate::control_registry_discovery_screen::attempt_inventory::{
    AttemptInventory, SCREEN_ATTEMPT_COUNT,
};
use crate::manifest::schedule_seed::ScheduleSeed;

/// Coverage: four measured targets, at each of two ladder endpoints, in each of two counterbalanced
/// blocks, is sixteen predeclared attempts, and every one names a distinct logical slot.
///
/// The distinctness half is what a bare length check would miss — a permutation emitting one
/// `(block, rung)` group twice and another never still produces sixteen attempts, and the screen
/// would silently measure the wrong design. It is also what proves the rung belongs in the identity:
/// were it omitted, each block's low and high attempts would collide as one slot and this test would
/// fail.
///
/// Checked at several seeds because the target permutation is the thing that could break this, and
/// it is a function of the seed.
#[test]
fn frozen_predeclares_sixteen_distinct_logical_slots() {
    const SEEDS: [u64; 4] = [0, 1, 7, 0xDEAD_BEEF];

    assert_eq!(
        SCREEN_ATTEMPT_COUNT, 16,
        "four targets times two rungs times two blocks predeclares sixteen attempts"
    );

    for seed in SEEDS {
        let frozen = AttemptInventory::frozen(ScheduleSeed::new(seed)).expect("the seed freezes");
        let attempts = frozen.attempts();

        assert_eq!(
            attempts.len(),
            SCREEN_ATTEMPT_COUNT,
            "seed {seed} must predeclare exactly {SCREEN_ATTEMPT_COUNT} attempts"
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
