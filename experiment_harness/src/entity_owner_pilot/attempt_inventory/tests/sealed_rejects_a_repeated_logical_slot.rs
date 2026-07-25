//! Sealing rejects a ten-attempt order that repeats a logical slot.

use crate::entity_owner_pilot::attempt_inventory::AttemptInventory;
use crate::plan::run_role::RunRole::{Arm, Control};

use super::slot::slot;

/// The guard [`AttemptInventory::frozen`] relies on, exercised directly against the input a
/// permutation bug would actually produce: the right *count* with one block emitted twice and
/// another never. A bare length check passes this; sealing must not.
#[test]
fn sealed_rejects_a_repeated_logical_slot() {
    // Ten attempts — but block 0 twice and block 3 never.
    let duplicated = vec![
        slot(0, Arm),
        slot(0, Control),
        slot(0, Arm),
        slot(0, Control),
        slot(1, Arm),
        slot(1, Control),
        slot(2, Arm),
        slot(2, Control),
        slot(4, Arm),
        slot(4, Control),
    ];

    let err = AttemptInventory::sealed(duplicated).expect_err("a repeated logical slot must not seal");
    let message = format!("{err:#}");
    assert!(
        message.contains("repeats a logical slot"),
        "the error names the repeated slot: {message}"
    );
}
