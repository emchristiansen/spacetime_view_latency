//! A dual disconnect-*and*-teardown cleanup failure survives *terminal settlement* with both errors
//! still distinct — the settle layer preserves what the [`ordered_cleanup`](super::super::ordered_cleanup)
//! sequencer produced, rather than collapsing it.
//!
//! The lower-level
//! [`dual_failure_retains_both_distinguishable_errors`](super::dual_failure_retains_both_distinguishable_errors)
//! proves `ordered_cleanup` *builds* a dual-retaining
//! [`RunCleanupOutcome`](crate::campaign::run_cleanup_outcome::RunCleanupOutcome); it does **not** prove
//! that a terminal minter carries that representation through into a run terminal. This test closes that
//! gap: it drives an exhausted run through the shared no-I/O fixture, settles it via
//! [`settle_executed_with`](super::super::settle_executed_with) with a
//! [`RunCleanupFailure::Disconnect`](crate::campaign::run_cleanup_failure::RunCleanupFailure::Disconnect)
//! whose retained `teardown` is itself an `Err`, and asserts the resulting
//! [`RunIncompletion::Cleanup`](crate::campaign::run_incompletion::RunIncompletion::Cleanup) still holds
//! both the disconnect error and the teardown error, each distinguishable, with the frontier at
//! [`RunStage::Disconnecting`](crate::campaign::run_stage::RunStage::Disconnecting).

use anyhow::anyhow;

use crate::campaign::run_cleanup_failure::RunCleanupFailure;
use crate::campaign::run_cleanup_outcome::RunCleanupOutcome;
use crate::campaign::run_cursor::RunSettled;
use crate::campaign::run_incompletion::RunIncompletion;
use crate::campaign::run_stage::RunStage;
use crate::campaign::tests::drive;

/// Pins that terminal settlement of a dual disconnect+teardown failure retains both errors distinctly
/// inside `RunIncompletion::Cleanup`.
#[test]
fn a_settled_dual_cleanup_failure_retains_both_errors_distinctly() {
    let executed = drive::drive_first_run_to_executed();
    // Both cleanup effects fail: a Disconnect failure whose still-attempted teardown is itself an Err.
    let outcome = RunCleanupOutcome::Failed(RunCleanupFailure::Disconnect {
        error: anyhow!("disconnect boom"),
        teardown: Err(anyhow!("teardown boom")),
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
                "a disconnect failure stops at the disconnecting stage even when the teardown also failed"
            );
            match failure {
                RunCleanupFailure::Disconnect { error, teardown } => {
                    assert!(
                        error.to_string().contains("disconnect boom"),
                        "the disconnect error is retained through settlement: {error}"
                    );
                    let teardown =
                        teardown.expect_err("the teardown also failed, so its error is retained");
                    assert!(
                        teardown.to_string().contains("teardown boom"),
                        "the teardown error is retained through settlement: {teardown}"
                    );
                    assert_ne!(
                        error.to_string(),
                        teardown.to_string(),
                        "the two retained errors stay distinguishable, so neither masks the other"
                    );
                }
                RunCleanupFailure::Teardown(_) => {
                    panic!("a disconnect failure classifies as Disconnect, retaining the teardown result")
                }
            }
        }
        RunIncompletion::Execution { .. } => {
            panic!("dose exhaustion means a Cleanup incompletion, not Execution")
        }
    }
}
