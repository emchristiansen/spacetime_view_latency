//! The outcome of settling an exhausted run through its obligatory cleanup.

use super::RunDone;
use super::RunIncomplete;

/// What settling an exhausted run ([`RunExecuted`](super::RunExecuted)) through cleanup yields: the run
/// completes ([`RunDone`]) iff its ordered disconnect-then-teardown returned clean, otherwise it stops at
/// a cleanup-stage frontier ([`RunIncomplete`]). Dose exhaustion alone no longer mints completion — only a
/// *clean cleanup after* exhaustion does — so a run cannot report completion while its client or server
/// teardown failed.
pub(crate) enum RunSettled {
    /// Dose exhaustion followed by a clean cleanup: the run completed.
    Done(RunDone),
    /// Dose exhaustion but a failed cleanup: the run stopped at a cleanup-stage frontier.
    Incomplete(RunIncomplete),
}
