//! Deterministic tests for the private [`ordered_cleanup`](super::ordered_cleanup) sequencer and the
//! two private terminal-minters [`settle_executed_with`](super::settle_executed_with) /
//! [`settle_stopped_with`](super::settle_stopped_with).
//!
//! Cleanup ordering and dual-failure retention are load-bearing invariants of the linear cleanup
//! owner, but exercising them against a real client/server would be nondeterministic and slow. The
//! sequencer is abstracted over its two `FnOnce() -> Result<()>` effects precisely so these tests can
//! drive it with deterministic in-process results and a shared order log — no sockets, no sleeps, no
//! threads. The sequencer-level tests pin: disconnect runs before teardown; teardown still runs (and
//! after) when the disconnect fails; and each `(disconnect, teardown)` result pair classifies to the
//! right [`RunCleanupOutcome`](crate::campaign::run_cleanup_outcome::RunCleanupOutcome) with both
//! errors retained and distinguishable.
//!
//! The settle-level tests then pin that the terminal minters carry a failed cleanup *outcome* into the
//! run terminal correctly: an exhausted run whose disconnect fails stops at a
//! [`RunStage::Disconnecting`](crate::campaign::run_stage::RunStage::Disconnecting)
//! [`RunIncompletion::Cleanup`](crate::campaign::run_incompletion::RunIncompletion::Cleanup); an
//! exhausted run whose teardown fails stops at [`RunStage::Teardown`](crate::campaign::run_stage::RunStage::Teardown);
//! a run that stopped short retains its execution frontier alongside the failed cleanup in a
//! [`RunIncompletion::Execution`](crate::campaign::run_incompletion::RunIncompletion::Execution); and a
//! dual disconnect+teardown failure keeps both errors distinct through settlement, not just through the
//! sequencer. One test entity per file.

mod a_disconnect_failure_retains_the_ok_teardown;
mod a_settled_dual_cleanup_failure_retains_both_errors_distinctly;
mod a_stopped_run_retains_its_execution_frontier_through_a_failed_cleanup;
mod a_teardown_only_failure_is_classified_teardown;
mod an_exhausted_run_with_a_disconnect_failure_stops_at_a_disconnecting_frontier;
mod an_exhausted_run_with_a_teardown_failure_stops_at_a_teardown_frontier;
mod disconnect_precedes_teardown;
mod dual_failure_retains_both_distinguishable_errors;
mod teardown_runs_after_a_disconnect_failure;
