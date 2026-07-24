//! The two structurally distinct ways a single run can stop short of a clean completion.

use crate::manifest::run_coordinate::RunCoordinate;

use super::run_cleanup_failure::RunCleanupFailure;
use super::run_cleanup_outcome::RunCleanupOutcome;
use super::run_frontier::RunFrontier;

/// How a run failed to complete cleanly — exactly one of two structurally distinct cases, so no
/// impossible combination of a present execution frontier with an absent cleanup outcome (or vice versa)
/// is representable. This is the run-level mirror of
/// [`CampaignIncomplete`](super::campaign_incomplete::CampaignIncomplete):
///
/// - [`Self::Execution`]: execution stopped short (a manifest or dose write failed before dose
///   exhaustion). Cleanup is *always* attempted in the settling transition, so this carries the required
///   execution [`RunFrontier`] *and* the required typed [`RunCleanupOutcome`] — both the execution failure
///   and the cleanup result are retained, never one without the other. This is the shape that finally lets
///   a *simultaneous* execution-and-cleanup failure be represented, which the provisional reported-marker
///   states could not.
/// - [`Self::Cleanup`]: all ten doses landed (execution exhausted cleanly), but the obligatory cleanup
///   failed. There is no execution frontier — execution succeeded — so this carries a cleanup-stage
///   [`RunFrontier`] built from the required last-success markers plus the typed [`RunCleanupFailure`].
///
/// Both variants hold a [`RunFrontier`], so [`Self::run`] and [`Self::frontier`] read it uniformly.
/// Holding either variant is evidence about which record writes' contracts returned success; neither is a
/// claim of physical crash persistence.
#[derive(Debug)]
pub(crate) enum RunIncompletion {
    /// Execution stopped short; cleanup was still attempted and its outcome is retained alongside the
    /// execution frontier.
    Execution {
        frontier: RunFrontier,
        cleanup: RunCleanupOutcome,
    },
    /// Execution exhausted cleanly but the obligatory cleanup failed: a cleanup-stage frontier and the
    /// typed cleanup failure.
    Cleanup {
        frontier: RunFrontier,
        failure: RunCleanupFailure,
    },
}

impl RunIncompletion {
    /// The frontier of the run that stopped short — the execution frontier for [`Self::Execution`] or the
    /// cleanup-stage frontier for [`Self::Cleanup`].
    pub(in crate::campaign) fn frontier(&self) -> &RunFrontier {
        match self {
            RunIncompletion::Execution { frontier, .. } => frontier,
            RunIncompletion::Cleanup { frontier, .. } => frontier,
        }
    }

    /// The run that stopped short, read from the frontier. Lets the block fold assert the stopped run's
    /// coordinate matches the outstanding run's exactly as a completion binds its coordinate.
    pub(in crate::campaign) fn run(&self) -> &RunCoordinate {
        self.frontier().run()
    }
}
