//! Recording a confirmation at an index outside the expected set is a loud error, not a silent
//! grow-or-drop — an out-of-range prerequisite is unrepresentable in a well-formed batch.

use crate::observation::confirmation_set::ConfirmationSet;

/// The first index past a four-slot set (`4`) has no slot, so recording it fails and names the
/// out-of-range condition.
#[test]
fn an_out_of_range_index_is_rejected() {
    let mut set = ConfirmationSet::new(4);
    let error = set
        .record(4)
        .expect_err("an index past the expected set must be rejected");
    assert!(
        error.to_string().contains("out of range"),
        "the error names the out-of-range condition: {error}"
    );
}
