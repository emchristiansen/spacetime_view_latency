//! How run-resource provisioning failed: the failed acquisition stage plus every partial teardown.

use anyhow::Error;

use crate::provision::teardown::into_error;

/// Which stage of [`provision_run_resources`](super::provision::provision_run_resources) failed, and —
/// for the stages that had a loud capability live when they failed — the retained result of every
/// partial teardown that stage attempted. Cleanup is always attempted for whatever was live, and each
/// attempted teardown's result is kept verbatim (its `Ok(())` or its `Err`), never flattened into the
/// primary cause.
///
/// The variants make the stage structural and rule out impossible teardown combinations (a
/// server-shutdown result at a stage where no server ever existed, say) that a flat
/// `{ error, server_shutdown: Option<_>, staged_cleanup: Option<_> }` shape would admit:
///
/// - [`Self::ResolveDistribution`] / [`Self::StageWasm`]: failed before any loud capability was live,
///   so there is nothing to tear down — the cause stands alone.
/// - [`Self::StartServer`]: the staged WASM was already live; its cleanup was attempted and retained.
/// - [`Self::Publish`]: both the running server and the staged WASM were live; both teardowns were
///   attempted and both retained.
///
/// Holds `anyhow::Error` and so is `Debug`-only (no `Clone`/`Eq`), matching the campaign-level typed
/// failures ([`RunCleanupFailure`](crate::campaign::run_cleanup_failure::RunCleanupFailure) and its
/// siblings) whose dual-evidence structure it mirrors on the acquisition path.
#[derive(Debug)]
pub(crate) enum ProvisionFailure {
    /// Resolving/verifying the pinned distribution failed; nothing was live to tear down.
    ResolveDistribution(Error),
    /// Staging the hash-verified module WASM failed; nothing was live to tear down.
    StageWasm(Error),
    /// Starting the isolated server failed; the staged WASM cleanup was still attempted and retained.
    StartServer {
        error: Error,
        staged_cleanup: Result<(), Error>,
    },
    /// Publishing/snapshotting failed; both the server shutdown and the staged WASM cleanup were
    /// attempted and both retained.
    Publish {
        error: Error,
        server_shutdown: Result<(), Error>,
        staged_cleanup: Result<(), Error>,
    },
}

impl ProvisionFailure {
    /// Render this typed failure into one aggregate [`anyhow::Error`] for the borrowed-capability
    /// provisioning path ([`provision_and_run`](super::provision::provision_and_run)), which does not
    /// retain typed dual evidence. Reproduces the previous `into_error` aggregation exactly: the primary
    /// cause first, then every attempted partial-teardown error, so no cleanup failure is hidden behind
    /// the primary one. The campaign acquisition path keeps the typed value instead of calling this.
    pub(crate) fn into_anyhow(self) -> Error {
        let mut errors = Vec::new();
        match self {
            ProvisionFailure::ResolveDistribution(error) | ProvisionFailure::StageWasm(error) => {
                errors.push(error);
            }
            ProvisionFailure::StartServer {
                error,
                staged_cleanup,
            } => {
                errors.push(error);
                if let Err(e) = staged_cleanup {
                    errors.push(e);
                }
            }
            ProvisionFailure::Publish {
                error,
                server_shutdown,
                staged_cleanup,
            } => {
                errors.push(error);
                if let Err(e) = server_shutdown {
                    errors.push(e);
                }
                if let Err(e) = staged_cleanup {
                    errors.push(e);
                }
            }
        }
        into_error(errors)
    }
}
