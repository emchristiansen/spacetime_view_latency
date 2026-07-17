//! An exhausted run whose client disconnect fails settles to a `Disconnecting`-stage
//! [`RunIncomplete`], never a [`RunDone`].
//!
//! Skeleton only — the body lands with the effectful-driver milestone. This test will drive the run
//! cursor to a [`RunExecuted`](crate::campaign::run_cursor::RunExecuted) (ten-dose exhaustion), then
//! call the private terminal-minter [`settle_executed_with`](super::super::settle_executed_with) with a
//! [`RunCleanupOutcome::Failed`](crate::campaign::run_cleanup_outcome::RunCleanupOutcome::Failed) whose
//! failure is a
//! [`RunCleanupFailure::Disconnect`](crate::campaign::run_cleanup_failure::RunCleanupFailure::Disconnect),
//! and assert the result is [`RunSettled::Incomplete`](crate::campaign::run_cursor::RunSettled::Incomplete)
//! whose [`RunIncompletion::Cleanup`](crate::campaign::run_incompletion::RunIncompletion::Cleanup) carries
//! a frontier at [`RunStage::Disconnecting`](crate::campaign::run_stage::RunStage::Disconnecting), with the
//! exhausted run's record tail and last dose preserved — proving a disconnect failure after full exhaustion
//! stops at the disconnect stage rather than minting the run's sole `RunComplete`.

/// Pins that a disconnect failure after dose exhaustion settles to `RunIncompletion::Cleanup` at
/// `RunStage::Disconnecting`.
#[test]
fn an_exhausted_run_with_a_disconnect_failure_stops_at_a_disconnecting_frontier() {
    todo!(
        "drive to RunExecuted, settle_executed_with a Failed(Disconnect{{..}}) outcome, assert \
         RunSettled::Incomplete / RunIncompletion::Cleanup at RunStage::Disconnecting"
    )
}
