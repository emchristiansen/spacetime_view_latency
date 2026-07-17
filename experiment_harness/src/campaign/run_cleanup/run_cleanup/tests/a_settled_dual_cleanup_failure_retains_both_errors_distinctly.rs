//! A dual disconnect-*and*-teardown cleanup failure survives *terminal settlement* with both errors
//! still distinct — the settle layer preserves what the [`ordered_cleanup`](super::super::ordered_cleanup)
//! sequencer produced, rather than collapsing it.
//!
//! Skeleton only — the body lands with the effectful-driver milestone. The lower-level
//! [`dual_failure_retains_both_distinguishable_errors`](super::dual_failure_retains_both_distinguishable_errors)
//! proves `ordered_cleanup` *builds* a dual-retaining
//! [`RunCleanupOutcome`](crate::campaign::run_cleanup_outcome::RunCleanupOutcome); it does **not** prove
//! that a terminal minter carries that representation through into a run terminal. This test closes that
//! gap: it settles an exhausted run via [`settle_executed_with`](super::super::settle_executed_with) with a
//! [`RunCleanupFailure::Disconnect`](crate::campaign::run_cleanup_failure::RunCleanupFailure::Disconnect)
//! whose retained `teardown` is itself an `Err`, and asserts the resulting
//! [`RunIncompletion::Cleanup`](crate::campaign::run_incompletion::RunIncompletion::Cleanup) still holds
//! both the disconnect error and the teardown error, each distinguishable, with the frontier at
//! [`RunStage::Disconnecting`](crate::campaign::run_stage::RunStage::Disconnecting).

/// Pins that terminal settlement of a dual disconnect+teardown failure retains both errors distinctly
/// inside `RunIncompletion::Cleanup`.
#[test]
fn a_settled_dual_cleanup_failure_retains_both_errors_distinctly() {
    todo!(
        "settle_executed_with a Failed(Disconnect{{ error, teardown: Err(..) }}) outcome, assert \
         RunIncompletion::Cleanup retains both errors distinctly at RunStage::Disconnecting"
    )
}
