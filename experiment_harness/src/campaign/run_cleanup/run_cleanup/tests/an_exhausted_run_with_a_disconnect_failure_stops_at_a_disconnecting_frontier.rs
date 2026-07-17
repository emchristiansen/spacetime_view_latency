//! An exhausted run whose client disconnect fails settles to a `Disconnecting`-stage
//! [`RunIncomplete`], never a [`RunDone`].
//!
//! Drives the run cursor to a [`RunExecuted`](crate::campaign::run_cursor::RunExecuted) (ten-dose
//! exhaustion) through the shared no-I/O fixture, then calls the private terminal-minter
//! [`settle_executed_with`](super::super::settle_executed_with) with a
//! [`RunCleanupOutcome::Failed`](crate::campaign::run_cleanup_outcome::RunCleanupOutcome::Failed) whose
//! failure is a
//! [`RunCleanupFailure::Disconnect`](crate::campaign::run_cleanup_failure::RunCleanupFailure::Disconnect),
//! and asserts the result is [`RunSettled::Incomplete`](crate::campaign::run_cursor::RunSettled::Incomplete)
//! whose [`RunIncompletion::Cleanup`](crate::campaign::run_incompletion::RunIncompletion::Cleanup) carries
//! a frontier at [`RunStage::Disconnecting`](crate::campaign::run_stage::RunStage::Disconnecting), with the
//! exhausted run's record tail and last dose preserved — proving a disconnect failure after full exhaustion
//! stops at the disconnect stage rather than minting the run's sole `RunComplete`.

use anyhow::anyhow;

use crate::campaign::run_cleanup_failure::RunCleanupFailure;
use crate::campaign::run_cleanup_outcome::RunCleanupOutcome;
use crate::campaign::run_cursor::RunSettled;
use crate::campaign::run_incompletion::RunIncompletion;
use crate::campaign::run_stage::RunStage;
use crate::campaign::tests::drive;
use crate::dataset::dose_index::DoseIndex;
use crate::params::NUM_DOSES_USIZE;

/// Pins that a disconnect failure after dose exhaustion settles to `RunIncompletion::Cleanup` at
/// `RunStage::Disconnecting`.
#[test]
fn an_exhausted_run_with_a_disconnect_failure_stops_at_a_disconnecting_frontier() {
    let executed = drive::drive_first_run_to_executed();
    // Disconnect fails; the still-attempted teardown returned success (retained verbatim).
    let outcome = RunCleanupOutcome::Failed(RunCleanupFailure::Disconnect {
        error: anyhow!("disconnect boom"),
        teardown: Ok(()),
    });

    let incomplete = match super::super::settle_executed_with(executed, outcome) {
        RunSettled::Incomplete(incomplete) => incomplete,
        RunSettled::Done(_) => panic!("a failed cleanup after exhaustion cannot complete the run"),
    };
    let (_sink, incompletion) = incomplete.into_parts();

    match incompletion {
        RunIncompletion::Cleanup { frontier, failure } => {
            assert_eq!(
                frontier.stage(),
                RunStage::Disconnecting,
                "a disconnect failure stops at the disconnecting stage"
            );
            assert!(
                frontier.last_successful_record().is_some(),
                "the exhausted run's record tail is preserved in the cleanup frontier"
            );
            assert_eq!(
                frontier.last_durable_dose(),
                Some(DoseIndex::ALL[NUM_DOSES_USIZE - 1]),
                "the last exhausted dose is preserved in the cleanup frontier"
            );
            assert!(
                matches!(
                    failure,
                    RunCleanupFailure::Disconnect {
                        teardown: Ok(()),
                        ..
                    }
                ),
                "the disconnect failure retains its clean teardown"
            );
        }
        RunIncompletion::Execution { .. } => {
            panic!("dose exhaustion means a Cleanup incompletion, not Execution")
        }
    }
}
