//! A second confirmation for an already-recorded slot is a loud error, so a double-fire can never
//! count twice toward completion.

use crate::observation::confirmation_set::ConfirmationSet;

/// Recording index `0` twice fails the second time, naming the duplicate.
#[test]
fn a_duplicate_confirmation_is_rejected() {
    let mut set = ConfirmationSet::new(4);
    set.record(0)
        .expect("the first confirmation for a slot is accepted");

    let error = set
        .record(0)
        .expect_err("a duplicate confirmation for the same slot must be rejected");
    assert!(
        error.to_string().contains("duplicate"),
        "the error names the duplicate condition: {error}"
    );
}
