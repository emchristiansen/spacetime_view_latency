//! Official 2.6.1 standalone provisioning.

use anyhow::Result;

use crate::run_manifest::RunManifest;

/// Provision the official Nix-packaged 2.6.1 standalone and produce the immutable run
/// manifest: assert `spacetimedb-cli`/`spacetimedb-standalone` versions and release
/// commit against [`crate::params::EXPECTED_VERSION`] /
/// [`crate::params::EXPECTED_RELEASE_COMMIT`], start the isolated server on
/// `server_listen_addr` with its own data directory, prove `/proc/<pid>/exe` equals
/// the resolved standalone path, publish the module, and record the WASM hash and
/// database identity. Fails before any measurement on any mismatch.
pub fn provision(_server_listen_addr: &str, _seed: u64) -> Result<RunManifest> {
    todo!("resolve Nix spacetimedb-2.6.1, assert versions/commit, prove /proc/<pid>/exe, publish module")
}
