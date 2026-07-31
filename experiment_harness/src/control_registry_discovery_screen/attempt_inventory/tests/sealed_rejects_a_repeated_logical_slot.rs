//! The seal rejects a right-sized inventory that repeats a slot.

use crate::control_registry_discovery_screen::attempt_inventory::{
    AttemptInventory, SCREEN_ATTEMPT_COUNT,
};
use crate::manifest::schedule_seed::ScheduleSeed;

/// Coverage: the duplicate-slot half of the seal actually fires, independently of the length check.
///
/// Built by taking a real frozen inventory and replacing its last attempt with a copy of its first,
/// which keeps the count at exactly sixteen. A seal that only checked length would accept this, and
/// the screen would run one slot twice while never running another — measuring a design nobody
/// froze, with a ledger that looks complete.
#[test]
fn sealed_rejects_a_repeated_logical_slot() {
    let frozen = AttemptInventory::frozen(ScheduleSeed::new(7)).expect("the seed freezes");
    let mut order = frozen.attempts().to_vec();
    assert_eq!(order.len(), SCREEN_ATTEMPT_COUNT);

    let duplicate = order[0];
    *order.last_mut().expect("the inventory is non-empty") = duplicate;

    // `sealed` is private to the parent module; this test is a child of it, so it exercises the
    // seal directly rather than through a test-only constructor widening the real API.
    let error = AttemptInventory::sealed(order)
        .expect_err("an inventory repeating a logical slot must not seal");
    let rendered = format!("{error:#}");
    assert!(
        rendered.contains("repeats a logical slot"),
        "the seal must name the repeated slot, got {rendered:?}"
    );
}
