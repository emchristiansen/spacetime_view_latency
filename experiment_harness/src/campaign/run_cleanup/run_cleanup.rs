//! The obligatory linear cleanup owner: the sole minter of a run's terminal, after real cleanup.

use anyhow::Result;

use crate::campaign::run_cleanup_failure::RunCleanupFailure;
use crate::campaign::run_cleanup_outcome::RunCleanupOutcome;
use crate::campaign::run_cursor::RunComplete;
use crate::campaign::run_cursor::RunDone;
use crate::campaign::run_cursor::RunExecuted;
use crate::campaign::run_cursor::RunExecutionStopped;
use crate::campaign::run_cursor::RunIncomplete;
use crate::campaign::run_cursor::RunSettled;
use crate::campaign::run_frontier::RunFrontier;
use crate::campaign::run_incompletion::RunIncompletion;
use crate::client::connected_client::ConnectedClient;
use crate::provision::run_resources::RunResources;

use super::cleanup_minted::CleanupMinted;
use super::must_disconnect::MustDisconnect;

/// The driver-layer capability that linearly owns one run's measured client and provisioned resources and
/// is the *sole* path that can mint the run's terminal. A run's execution first yields an inert
/// pre-cleanup carrier — [`RunExecuted`] (dose ladder exhausted) or [`RunExecutionStopped`] (a failing
/// edge) — neither of which can settle itself. Consuming this owner via [`Self::settle_executed`] /
/// [`Self::settle_stopped`] always attempts the ordered client disconnect **then** server/staged teardown,
/// and only then mints [`RunDone`]/[`RunIncomplete`], preserving a simultaneous execution-and-cleanup
/// failure as typed structure.
///
/// The client is held behind a [`MustDisconnect`] guard (loud take-on-consume, matching the server's own
/// `Drop` teeth) and the resources behind [`RunResources`]; `RunCleanup` itself has **no** `Drop`, so a
/// consuming settle can destructure `self` and move both guards out without `unsafe` or E0509. Its
/// settlement is the only code that can construct a [`CleanupMinted`] witness, so no other campaign code
/// can reach a run terminal constructor.
pub(crate) struct RunCleanup {
    client: MustDisconnect,
    resources: RunResources,
}

impl RunCleanup {
    /// Arm the run's linear cleanup owner from an **already-connected** measured client and the run's
    /// provisioned resources, taking linear ownership of both. Infallible by construction: connecting the
    /// client and resolving its authenticated role identities are the fallible acquisition steps, and both
    /// run *before* this arming so a resolution failure can disconnect the client and tear the resources
    /// down without a cleanup owner to strand (see
    /// [`RunAcquisitionFailure`](crate::campaign::run_acquisition_failure::RunAcquisitionFailure)). Once
    /// armed, this owner is the "bomb": no path may `?`-early-return or panic while it is owned, and its
    /// consuming [`Self::settle_executed`]/[`Self::settle_stopped`] are the sole minters of the run's
    /// terminal. The owner holds **no** run coordinate: a settled run's terminal coordinate comes solely
    /// from the [`RunExecuted`]/[`RunExecutionStopped`] carrier being settled, so there is no duplicate here
    /// to cross-check against it. `pub(in crate::campaign)` confines arming to the campaign module subtree —
    /// not the acquisition path alone, which the visibility does not single out; the sole implemented caller
    /// is that path (`acquire_and_drive`).
    pub(in crate::campaign) fn arm(client: ConnectedClient, resources: RunResources) -> Self {
        Self {
            client: MustDisconnect::new(client),
            resources,
        }
    }

    /// The live measured client, borrowed for the run's measured writes and correctness checks.
    pub(in crate::campaign) fn client(&self) -> &ConnectedClient {
        self.client.client()
    }

    /// Settle an exhausted run through real cleanup: attempt the ordered disconnect-then-teardown and mint
    /// the terminal — [`RunDone`] on a clean cleanup, or a cleanup-stage [`RunIncomplete`] retaining the
    /// typed failure. The terminal coordinate is the one the exhausted carrier already holds; the owner
    /// keeps no separate coordinate to reconcile.
    pub(in crate::campaign) fn settle_executed(self, executed: RunExecuted) -> RunSettled {
        let Self { client, resources } = self;
        let outcome = run_cleanup(client, resources);
        settle_executed_with(executed, outcome)
    }

    /// Settle a run that stopped short during execution through real cleanup: attempt the ordered
    /// disconnect-then-teardown and mint a [`RunIncomplete`] whose [`RunIncompletion::Execution`] retains
    /// *both* the execution frontier and the cleanup outcome. The frontier's coordinate is the one the
    /// stopped carrier already holds; the owner keeps no separate coordinate to reconcile.
    pub(in crate::campaign) fn settle_stopped(self, stopped: RunExecutionStopped) -> RunIncomplete {
        let Self { client, resources } = self;
        let outcome = run_cleanup(client, resources);
        settle_stopped_with(stopped, outcome)
    }
}

/// Attempt the ordered per-run cleanup on the real client and resources, delegating the ordering and
/// classification to [`ordered_cleanup`].
fn run_cleanup(client: MustDisconnect, resources: RunResources) -> RunCleanupOutcome {
    ordered_cleanup(move || client.disconnect(), move || resources.teardown())
}

/// The ordered per-run cleanup sequencer, abstracted over its two effects so it can be exercised with
/// deterministic test results. It calls `disconnect` **first**, then `teardown` — and `teardown` **always
/// runs**, even when `disconnect` returned an error — so a disconnect failure can never hide a teardown
/// failure, and both errors are retained when both fail. The `(disconnect, teardown)` classification:
///
/// - both `Ok` → [`RunCleanupOutcome::Clean`];
/// - disconnect `Ok`, teardown `Err` → [`RunCleanupFailure::Teardown`];
/// - disconnect `Err` → [`RunCleanupFailure::Disconnect`] retaining the still-attempted teardown result
///   verbatim (its `Ok(())` or its `Err`), never dropped.
fn ordered_cleanup(
    disconnect: impl FnOnce() -> Result<()>,
    teardown: impl FnOnce() -> Result<()>,
) -> RunCleanupOutcome {
    let disconnect = disconnect();
    let teardown = teardown();
    match (disconnect, teardown) {
        (Ok(()), Ok(())) => RunCleanupOutcome::Clean,
        (Ok(()), Err(teardown)) => RunCleanupOutcome::Failed(RunCleanupFailure::Teardown(teardown)),
        (Err(error), teardown) => {
            RunCleanupOutcome::Failed(RunCleanupFailure::Disconnect { error, teardown })
        }
    }
}

/// Mint the terminal for an exhausted run given its cleanup `outcome`. A `Clean` outcome mints the run's
/// sole [`RunComplete`] and a completed [`RunDone`]; a `Failed` outcome stops the run at a cleanup-stage
/// frontier (its stage read from the failure before it is moved) retaining the typed failure. The only
/// place — with [`settle_stopped_with`] — that constructs a [`CleanupMinted`], hence the only place that
/// mints a run terminal.
fn settle_executed_with(executed: RunExecuted, outcome: RunCleanupOutcome) -> RunSettled {
    let (sink, coord, last_record, last_dose) = executed.into_parts();
    match outcome {
        RunCleanupOutcome::Clean => {
            let complete = RunComplete::new(coord, last_record, last_dose, CleanupMinted::new());
            RunSettled::Done(RunDone::new(sink, complete, CleanupMinted::new()))
        }
        RunCleanupOutcome::Failed(failure) => {
            let frontier =
                RunFrontier::cleanup_stage(coord, &failure, Some(last_record), Some(last_dose));
            let incompletion = RunIncompletion::Cleanup { frontier, failure };
            RunSettled::Incomplete(RunIncomplete::new(sink, incompletion, CleanupMinted::new()))
        }
    }
}

/// Mint the terminal for a run that stopped short during execution: a [`RunIncomplete`] whose
/// [`RunIncompletion::Execution`] retains both the execution frontier and the cleanup `outcome`.
fn settle_stopped_with(stopped: RunExecutionStopped, outcome: RunCleanupOutcome) -> RunIncomplete {
    let (sink, frontier) = stopped.into_parts();
    let incompletion = RunIncompletion::Execution {
        frontier,
        cleanup: outcome,
    };
    RunIncomplete::new(sink, incompletion, CleanupMinted::new())
}

/// Settle a stopped run with an always-clean **reported** cleanup, for no-I/O cursor tests.
///
/// **Reported, not effected.** This performs the exact terminal typestate settlement a real
/// [`RunCleanup::settle_stopped`] would on a clean cleanup, but it does **not** open a socket, disconnect a
/// client, or tear down a server — there is no [`RunCleanup`], [`MustDisconnect`], or [`RunResources`]
/// anywhere in reach. It retains the execution frontier with a `Clean` cleanup, so a failure test can assert
/// the run's execution frontier exactly while the cleanup boundary is honest about not having performed real
/// I/O. Gated `#[cfg(test)]`, so no production path can settle a run without performing real cleanup.
#[cfg(test)]
pub(in crate::campaign) fn settle_stopped_reported(stopped: RunExecutionStopped) -> RunIncomplete {
    settle_stopped_with(stopped, RunCleanupOutcome::Clean)
}

#[cfg(test)]
mod tests;
