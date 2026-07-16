//! Immutable per-run provisioning and version manifest.

/// Observed provisioning facts recorded for one measurement run.
///
/// A plain record of what was actually resolved and started; it carries no
/// invariant of its own. The *expected* values it is checked against
/// ([`crate::params::EXPECTED_VERSION`], [`crate::params::EXPECTED_RELEASE_COMMIT`])
/// live in `params`, and provisioning fails before measuring on any mismatch (spec:
/// "Official 2.6.1 server provisioning").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunManifest {
    /// `spacetimedb-cli --version` reported semantic version.
    pub cli_version: String,
    /// `spacetimedb-cli --version` reported release commit.
    pub cli_release_commit: String,
    /// `spacetimedb-standalone --version` reported semantic version.
    pub standalone_version: String,
    /// Absolute path of the standalone executable that was started.
    pub standalone_exe_path: String,
    /// Nix store output the CLI and standalone were resolved from.
    pub nix_store_path: String,
    /// PID of the started standalone server process.
    pub server_pid: u32,
    /// `/proc/<pid>/exe` resolved for the running server; must equal
    /// `standalone_exe_path`.
    pub server_exe_path: String,
    /// Listen address the isolated experiment server was bound to.
    pub server_listen_addr: String,
    /// Data directory the experiment server was started with.
    pub data_dir: String,
    /// SHA-256 of the published WASM module artifact.
    pub wasm_module_sha256: String,
    /// Published database identity the harness bindings correspond to.
    pub database_identity: String,
    /// Explicit seed driving deterministic scheduling for this run.
    pub seed: u64,
}
