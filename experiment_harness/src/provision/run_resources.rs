//! The two loud provisioning capabilities of one run, bundled for a single ordered teardown.

use anyhow::Result;

use crate::provision::running_pinned_server::RunningPinnedServer;
use crate::provision::staged_module_wasm::StagedModuleWasm;
use crate::provision::teardown::into_error;

/// The isolated-server-plus-staged-WASM resources owned across one run, bundled so their teardown is one
/// aggregating call. Both members carry their own loud [`Drop`] guards (each asserts it was handed off by
/// an explicit consuming shutdown/cleanup), so a leaked `RunResources` still panics through those.
///
/// `RunResources` itself has **no** `Drop`: [`Self::teardown`] moves both members out of `self` by value,
/// which is legal precisely because the bundle is not a `Drop` type (E0509 would fire otherwise). That is
/// what lets the run's linear cleanup owner destructure it in a consuming settle without `unsafe`.
pub(crate) struct RunResources {
    server: RunningPinnedServer,
    staged: StagedModuleWasm,
}

impl RunResources {
    /// Bundle a run's provisioned server and staged module WASM. `pub(crate)` so provisioning assembles
    /// one after publishing.
    pub(crate) fn new(server: RunningPinnedServer, staged: StagedModuleWasm) -> Self {
        Self { server, staged }
    }

    /// The live server, borrowed for publication/measurement while the resources are owned.
    pub(crate) fn server(&self) -> &RunningPinnedServer {
        &self.server
    }

    /// Tear both resources down in order — server shutdown then staged-file cleanup — attempting **both**
    /// even if the first fails and aggregating every error, mirroring the provisioning driver's
    /// unconditional teardown discipline. Consumes `self`, moving both members out before any fallible
    /// step (legal because `RunResources` is not `Drop`).
    pub(crate) fn teardown(self) -> Result<()> {
        let Self { server, staged } = self;
        let mut errors = Vec::new();
        if let Err(e) = server.shutdown() {
            errors.push(e);
        }
        if let Err(e) = staged.cleanup() {
            errors.push(e);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(into_error(errors))
        }
    }
}
