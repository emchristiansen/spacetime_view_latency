//! A run that stopped short during execution keeps its execution frontier even when the obligatory
//! cleanup also fails: both failures are retained, never one masking the other.
//!
//! Drives the run cursor to a [`RunExecutionStopped`](crate::campaign::run_cursor::RunExecutionStopped)
//! (a manifest sink-write failure short of dose exhaustion) through the shared no-I/O fixture, then calls
//! the private terminal-minter [`settle_stopped_with`](super::super::settle_stopped_with) with a
//! [`RunCleanupOutcome::Failed`](crate::campaign::run_cleanup_outcome::RunCleanupOutcome::Failed), and
//! asserts the result is a [`RunIncomplete`](crate::campaign::run_cursor::RunIncomplete) whose
//! [`RunIncompletion::Execution`](crate::campaign::run_incompletion::RunIncompletion::Execution) retains
//! *both* the original execution `frontier` (its exact `WritingManifest` stage) *and* the failed `cleanup`
//! outcome — the simultaneous execution-and-cleanup failure the provisional reported-marker states could
//! not represent.

use anyhow::anyhow;

use crate::campaign::run_cleanup_failure::RunCleanupFailure;
use crate::campaign::run_cleanup_outcome::RunCleanupOutcome;
use crate::campaign::run_incompletion::RunIncompletion;
use crate::campaign::run_stage::RunStage;
use crate::campaign::tests::drive;

/// Pins that a stopped run's execution frontier survives a failed cleanup inside
/// `RunIncompletion::Execution { cleanup: Failed }`.
#[test]
fn a_stopped_run_retains_its_execution_frontier_through_a_failed_cleanup() {
    let stopped = drive::drive_first_run_to_stopped();
    // The only frontier identity observable before the carrier is consumed is its run coordinate; capture
    // it so the post-settlement frontier can be proven to be *this* execution frontier, not a fresh one.
    let expected_coord = stopped.coord().clone();
    // The obligatory cleanup also fails (a teardown failure here) — it must not mask the execution stop.
    let outcome = RunCleanupOutcome::Failed(RunCleanupFailure::Teardown(anyhow!("teardown boom")));

    let incomplete = super::super::settle_stopped_with(stopped, outcome);
    let (_sink, incompletion) = incomplete.into_parts();

    match incompletion {
        RunIncompletion::Execution { frontier, cleanup } => {
            // Identity: the retained frontier is the same run's, carried through settlement unchanged.
            assert_eq!(
                frontier.run(),
                &expected_coord,
                "the retained execution frontier names the same run that stopped, not a fresh coordinate"
            );
            // Evidence: it is the genuine manifest-sink-write failure frontier — the `WritingManifest`
            // stage, no record or dose whose contract returned success, and the sink poison the failed
            // write retained — none of which the failed cleanup overwrote.
            assert_eq!(
                frontier.stage(),
                RunStage::WritingManifest,
                "the execution frontier is the original manifest-write failure, untouched by the failed cleanup"
            );
            assert!(
                frontier.last_successful_record().is_none(),
                "no record's writer contract returned success before the manifest write failed"
            );
            assert!(
                frontier.last_durable_dose().is_none(),
                "and no dose either — execution stopped at the manifest write"
            );
            assert!(
                frontier.poison().is_some(),
                "the failed manifest write poisoned the sink, and that poison is retained on the frontier"
            );
            assert!(
                matches!(
                    cleanup,
                    RunCleanupOutcome::Failed(RunCleanupFailure::Teardown(_))
                ),
                "the failed cleanup outcome is retained alongside the execution frontier"
            );
        }
        RunIncompletion::Cleanup { .. } => {
            panic!("a run that stopped during execution is an Execution incompletion, not Cleanup")
        }
    }
}
