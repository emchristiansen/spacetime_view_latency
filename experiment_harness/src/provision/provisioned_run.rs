//! One run's provisioned resources plus its immutable manifest, handed to the run driver as a unit.

use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::provision::run_resources::RunResources;

/// Everything one run's provisioning produced: the live [`RunResources`] (isolated server + staged WASM)
/// and the immutable [`ValidatedRunManifest`] snapshotted while the server was alive. Returned by
/// [`provision_run_resources`](crate::provision::provision::provision_run_resources) so ownership of the
/// live resources leaves the provisioning scope by move instead of being torn down at scope exit — the run
/// driver then owns them for the duration of measurement and hands them to its linear cleanup.
///
/// Holds no `Drop`: [`Self::into_parts`] destructures it by value, and the moved-out `RunResources` still
/// carries its members' loud `Drop` guards, so a dropped-without-teardown resource still panics.
pub(crate) struct ProvisionedRun {
    resources: RunResources,
    manifest: ValidatedRunManifest,
}

impl ProvisionedRun {
    /// Bundle a run's provisioned resources and its immutable manifest. `pub(crate)` so provisioning
    /// assembles one on the success path.
    pub(crate) fn new(resources: RunResources, manifest: ValidatedRunManifest) -> Self {
        Self {
            resources,
            manifest,
        }
    }

    /// The immutable run manifest, borrowed for connection wiring and record assembly.
    pub(crate) fn manifest(&self) -> &ValidatedRunManifest {
        &self.manifest
    }

    /// Take the provisioned resources and the manifest together. The driver moves the resources into its
    /// linear cleanup and keeps the manifest (a value copy holding no live handle) for record assembly.
    pub(crate) fn into_parts(self) -> (RunResources, ValidatedRunManifest) {
        (self.resources, self.manifest)
    }
}
