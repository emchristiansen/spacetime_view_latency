//! A run that stopped short during execution keeps its execution frontier even when the obligatory
//! cleanup also fails: both failures are retained, never one masking the other.
//!
//! Skeleton only — the body lands with the effectful-driver milestone. This test will drive the run
//! cursor to a [`RunExecutionStopped`](crate::campaign::run_cursor::RunExecutionStopped) (an effect or
//! sink-write failure short of dose exhaustion), then call the private terminal-minter
//! [`settle_stopped_with`](super::super::settle_stopped_with) with a
//! [`RunCleanupOutcome::Failed`](crate::campaign::run_cleanup_outcome::RunCleanupOutcome::Failed), and
//! assert the result is a [`RunIncomplete`](crate::campaign::run_cursor::RunIncomplete) whose
//! [`RunIncompletion::Execution`](crate::campaign::run_incompletion::RunIncompletion::Execution) retains
//! *both* the original execution `frontier` (its exact stage and effect error) *and* the failed `cleanup`
//! outcome — the simultaneous execution-and-cleanup failure the provisional reported-marker states could
//! not represent.

/// Pins that a stopped run's execution frontier survives a failed cleanup inside
/// `RunIncompletion::Execution { cleanup: Failed }`.
#[test]
fn a_stopped_run_retains_its_execution_frontier_through_a_failed_cleanup() {
    todo!(
        "drive to RunExecutionStopped, settle_stopped_with a Failed outcome, assert RunIncomplete / \
         RunIncompletion::Execution retaining both the execution frontier and the Failed cleanup"
    )
}
