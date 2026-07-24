//! A set expecting zero confirmations — the single-table message family, which registers no
//! prerequisites — is complete the moment it is created, so its presence never blocks the barrier.

use crate::observation::confirmation_set::ConfirmationSet;

/// `ConfirmationSet::new(0)` is vacuously complete: there is nothing to wait for.
#[test]
fn an_empty_set_is_immediately_complete() {
    let set = ConfirmationSet::new(0);
    assert!(
        set.is_complete(),
        "a set expecting no confirmations is complete on construction"
    );
}
