//! A set completes only once every expected slot is recorded — not before — and recording the slots
//! in any order still completes it, since completion is presence, not order.

use crate::observation::confirmation_set::ConfirmationSet;

/// A three-slot set is incomplete after two of three confirmations and complete after the third,
/// even when the slots are recorded out of order.
#[test]
fn a_full_set_completes_only_when_every_slot_is_recorded() {
    let mut set = ConfirmationSet::new(3);

    set.record(2).expect("a fresh in-range slot is accepted");
    set.record(0).expect("a fresh in-range slot is accepted");
    assert!(
        !set.is_complete(),
        "two of three confirmations is not yet complete"
    );

    set.record(1).expect("a fresh in-range slot is accepted");
    assert!(
        set.is_complete(),
        "every expected slot is recorded, so the set is complete"
    );
}
