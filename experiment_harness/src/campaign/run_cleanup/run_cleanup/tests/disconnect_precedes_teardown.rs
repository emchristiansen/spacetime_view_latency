//! Both effects return success: the sequencer disconnects the measured client *before* tearing down
//! the server/staged resources, and classifies the run's cleanup as clean.

use std::cell::RefCell;

use crate::campaign::run_cleanup_outcome::RunCleanupOutcome;

use super::super::ordered_cleanup;

/// Each effect records its own name into a shared order log as it runs. Because the sequencer runs
/// them synchronously in sequence, the recorded order *is* the executed order — proving disconnect
/// precedes teardown without any timing or threading. Both returning success classifies to `Clean`.
#[test]
fn disconnect_precedes_teardown() {
    let order = RefCell::new(Vec::new());
    let outcome = ordered_cleanup(
        || {
            order.borrow_mut().push("disconnect");
            Ok(())
        },
        || {
            order.borrow_mut().push("teardown");
            Ok(())
        },
    );

    assert_eq!(
        *order.borrow(),
        vec!["disconnect", "teardown"],
        "the client disconnect must be attempted before the server/staged teardown"
    );
    assert!(
        matches!(outcome, RunCleanupOutcome::Clean),
        "both cleanup effects returned success, so the cleanup is clean"
    );
}
