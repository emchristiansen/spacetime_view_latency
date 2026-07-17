//! An exhausted run that disconnects cleanly but whose server/staged teardown fails settles to a
//! `Teardown`-stage [`RunIncomplete`], never a [`RunDone`].
//!
//! Skeleton only — the body lands with the effectful-driver milestone. This test will drive the run
//! cursor to a [`RunExecuted`](crate::campaign::run_cursor::RunExecuted) (ten-dose exhaustion), then
//! call the private terminal-minter [`settle_executed_with`](super::super::settle_executed_with) with a
//! [`RunCleanupOutcome::Failed`](crate::campaign::run_cleanup_outcome::RunCleanupOutcome::Failed) whose
//! failure is a
//! [`RunCleanupFailure::Teardown`](crate::campaign::run_cleanup_failure::RunCleanupFailure::Teardown), and
//! assert the result is [`RunSettled::Incomplete`](crate::campaign::run_cursor::RunSettled::Incomplete)
//! whose [`RunIncompletion::Cleanup`](crate::campaign::run_incompletion::RunIncompletion::Cleanup) carries
//! a frontier at [`RunStage::Teardown`](crate::campaign::run_stage::RunStage::Teardown), with the exhausted
//! run's record tail and last dose preserved — distinguishing the teardown-only stage from the
//! `Disconnecting` stage of a disconnect failure.

/// Pins that a teardown-only failure after dose exhaustion settles to `RunIncompletion::Cleanup` at
/// `RunStage::Teardown`.
#[test]
fn an_exhausted_run_with_a_teardown_failure_stops_at_a_teardown_frontier() {
    todo!(
        "drive to RunExecuted, settle_executed_with a Failed(Teardown(..)) outcome, assert \
         RunSettled::Incomplete / RunIncompletion::Cleanup at RunStage::Teardown"
    )
}
