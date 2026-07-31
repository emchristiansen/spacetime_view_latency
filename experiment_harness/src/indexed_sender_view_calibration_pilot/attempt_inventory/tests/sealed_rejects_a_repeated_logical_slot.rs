//! The seal rejects a right-sized inventory that repeats a slot.

use crate::indexed_sender_view_calibration_pilot::attempt_inventory::{
    AttemptInventory, CALIBRATION_ATTEMPT_COUNT,
};

/// Coverage: the duplicate-slot half of the seal fires independently of the length check.
///
/// Built by replacing the last attempt with a copy of the first, which keeps the count at exactly
/// two. A seal that only checked length would accept it. This is the case the seal's own rustdoc
/// names — "a construction bug emitting one replicate twice and the other never" — and it is
/// unreachable from `frozen()` today precisely because `frozen()` is correct; the seal exists so that
/// a future change to how the order is built cannot quietly stop being correct.
///
/// `sealed` is private to the parent module; this test is a child of it, so it exercises the seal
/// directly rather than through a test-only constructor widening the real API.
#[test]
fn sealed_rejects_a_repeated_logical_slot() {
    let frozen = AttemptInventory::frozen().expect("the frozen inventory seals");
    let mut order = frozen.attempts().to_vec();
    assert_eq!(order.len(), CALIBRATION_ATTEMPT_COUNT);

    let duplicate = order[0];
    *order.last_mut().expect("the inventory is non-empty") = duplicate;

    let error = AttemptInventory::sealed(order)
        .expect_err("an inventory repeating a logical slot must not seal");
    let rendered = format!("{error:#}");
    assert!(
        rendered.contains("repeats a logical slot"),
        "the seal must name the repeated slot, got {rendered:?}"
    );
}
