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
use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::provision::run_resources::RunResources;
use crate::provision::teardown::into_error;

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
    coord: RunCoordinate,
}

impl RunCleanup {
    /// Connect the measured subscriber to the provisioned run and take linear ownership of both the client
    /// and the resources. The owner's run coordinate is **derived** from the immutable manifest
    /// ([`ValidatedRunManifest::run_coordinate`]), not supplied separately, so the manifest and the
    /// coordinate the owner later asserts against the cursor state cannot disagree — there is no
    /// caller-provided duplicate to drift. Derives the server URL from the live server and the database
    /// identity from the manifest. If the connection fails the resources are torn down (aggregating the
    /// connect error with any teardown error) so nothing live escapes — the client is never left running.
    pub(in crate::campaign) fn connect(
        resources: RunResources,
        manifest: &ValidatedRunManifest,
    ) -> Result<Self> {
        let coord = manifest.run_coordinate();
        let server_url = resources.server().listen().client_url();
        let database_identity = manifest.database_identity().identity().to_hex().to_string();
        match ConnectedClient::connect(&server_url, &database_identity) {
            Ok(client) => Ok(Self {
                client: MustDisconnect::new(client),
                resources,
                coord,
            }),
            Err(connect_error) => {
                let mut errors = vec![connect_error.context("connecting the measured subscriber")];
                if let Err(teardown_error) = resources.teardown() {
                    errors.push(teardown_error);
                }
                Err(into_error(errors))
            }
        }
    }

    /// The live measured client, borrowed for the run's measured writes and correctness checks.
    pub(in crate::campaign) fn client(&self) -> &ConnectedClient {
        self.client.client()
    }

    /// Settle an exhausted run through real cleanup. Asserts the owner's bound coordinate matches the
    /// exhausted run's, attempts the ordered disconnect-then-teardown, and mints the terminal: [`RunDone`]
    /// on a clean cleanup, or a cleanup-stage [`RunIncomplete`] retaining the typed failure.
    pub(in crate::campaign) fn settle_executed(self, executed: RunExecuted) -> RunSettled {
        let Self {
            client,
            resources,
            coord,
        } = self;
        assert_eq!(
            executed.coord(),
            &coord,
            "the cleanup owner's run coordinate must match the exhausted run's coordinate"
        );
        let outcome = run_cleanup(client, resources);
        settle_executed_with(executed, outcome)
    }

    /// Settle a run that stopped short during execution through real cleanup. Asserts the coordinate match,
    /// attempts the ordered disconnect-then-teardown, and mints a [`RunIncomplete`] whose
    /// [`RunIncompletion::Execution`] retains *both* the execution frontier and the cleanup outcome.
    pub(in crate::campaign) fn settle_stopped(self, stopped: RunExecutionStopped) -> RunIncomplete {
        let Self {
            client,
            resources,
            coord,
        } = self;
        assert_eq!(
            stopped.coord(),
            &coord,
            "the cleanup owner's run coordinate must match the stopped run's coordinate"
        );
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

/// Settle an exhausted run with an always-clean **reported** cleanup, for no-I/O cursor tests.
///
/// **Reported, not effected.** This performs the exact terminal typestate settlement a real
/// [`RunCleanup::settle_executed`] would on a clean cleanup, but it does **not** open a socket, disconnect
/// a client, or tear down a server — there is no [`RunCleanup`], [`MustDisconnect`], or [`RunResources`]
/// anywhere in reach. It merely reports the `Clean` outcome a successful real cleanup would have produced,
/// standing in for the retired `DisconnectReported`/`TeardownReported` tokens at the relocated cleanup
/// boundary. Gated `#[cfg(test)]`, so no production path can settle a run without performing real cleanup.
#[cfg(test)]
pub(in crate::campaign) fn settle_executed_reported(executed: RunExecuted) -> RunSettled {
    settle_executed_with(executed, RunCleanupOutcome::Clean)
}

/// Settle a stopped run with an always-clean **reported** cleanup, for no-I/O cursor tests. Reported, not
/// effected — see [`settle_executed_reported`]. Retains the execution frontier with a `Clean` cleanup, so
/// a failure test can assert the run's execution frontier exactly while the cleanup boundary is honest
/// about not having performed real I/O.
#[cfg(test)]
pub(in crate::campaign) fn settle_stopped_reported(stopped: RunExecutionStopped) -> RunIncomplete {
    settle_stopped_with(stopped, RunCleanupOutcome::Clean)
}

#[cfg(test)]
mod tests;
