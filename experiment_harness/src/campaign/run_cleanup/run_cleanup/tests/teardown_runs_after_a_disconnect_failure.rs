//! The disconnect fails: the teardown is still attempted, and still *after* the disconnect — a
//! disconnect failure can neither skip nor reorder the server/staged teardown.

use std::cell::RefCell;

use anyhow::anyhow;

use crate::campaign::run_cleanup_failure::RunCleanupFailure;
use crate::campaign::run_cleanup_outcome::RunCleanupOutcome;

use super::super::ordered_cleanup;

/// The disconnect effect records itself and then fails; the teardown effect records itself and
/// succeeds. The order log must still be `[disconnect, teardown]`, proving the teardown ran and ran
/// after the failed disconnect (not skipped by the early error), and the outcome is a `Disconnect`
/// cleanup failure.
#[test]
fn teardown_runs_after_a_disconnect_failure() {
    let order = RefCell::new(Vec::new());
    let outcome = ordered_cleanup(
        || {
            order.borrow_mut().push("disconnect");
            Err(anyhow!("disconnect failed"))
        },
        || {
            order.borrow_mut().push("teardown");
            Ok(())
        },
    );

    assert_eq!(
        *order.borrow(),
        vec!["disconnect", "teardown"],
        "the teardown must still be attempted, and after the disconnect, even though the disconnect failed"
    );
    assert!(
        matches!(
            outcome,
            RunCleanupOutcome::Failed(RunCleanupFailure::Disconnect { .. })
        ),
        "a failed disconnect classifies as a Disconnect cleanup failure"
    );
}
