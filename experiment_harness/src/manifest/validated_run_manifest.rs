//! The immutable, serialize-only run manifest snapshot.

use std::net::SocketAddr;
use std::path::PathBuf;

use semver::Version;
use serde::Serialize;

use crate::manifest::database_identity::DatabaseIdentity;
use crate::manifest::preregistered_parameters::PreregisteredParameters;
use crate::manifest::release_commit::ReleaseCommit;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::manifest::server_pid::ServerPid;
use crate::manifest::verified_module_artifact::VerifiedModuleArtifact;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::provision::running_pinned_server::RunningPinnedServer;
use crate::provision::verified_distribution::VerifiedDistribution;

/// An immutable, `Serialize`-only snapshot of one run's provisioning facts and
/// preregistered parameters.
///
/// This is **not** a liveness proof. It is assembled by value-copying the immutable
/// scalar facts out of the non-serializable owning capabilities ([`VerifiedDistribution`]
/// and [`RunningPinnedServer`], which own the `Child` and fresh data directory) at one
/// instant. The manifest holds no `Child`, no data-dir handle, and no borrow, so it can
/// outlive the server and serialize freely while making no claim that the process is
/// still alive — the capability keeps gating live operations and cleans up on explicit
/// shutdown.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ValidatedRunManifest {
    run: RunCoordinate,
    schedule_seed: ScheduleSeed,
    parameters: PreregisteredParameters,
    distribution: DistributionFacts,
    server: ServerFacts,
    module: ModuleFacts,
}

impl ValidatedRunManifest {
    /// Snapshot the immutable facts of a provisioned run. Value-copies out of the live
    /// capabilities; retains no handle or borrow to them.
    pub(crate) fn assemble(
        run: RunCoordinate,
        distribution: &VerifiedDistribution,
        server: &RunningPinnedServer,
        artifact: &VerifiedModuleArtifact,
        schedule_seed: ScheduleSeed,
    ) -> Self {
        let cli = distribution.cli();
        let standalone = distribution.standalone();
        let process = server.process();
        let listen = server.listen();

        Self {
            run,
            schedule_seed,
            parameters: PreregisteredParameters::preregistered(),
            distribution: DistributionFacts {
                nix_store_bin_dir: distribution.store_bin_dir().to_path_buf(),
                cli_exe: cli.exe().to_path_buf(),
                cli_version: cli.version().clone(),
                cli_release_commit: *cli.commit(),
                cli_version_raw: cli.raw().to_string(),
                standalone_exe: standalone.exe().to_path_buf(),
                standalone_version: standalone.version().clone(),
                standalone_version_raw: standalone.raw().to_string(),
            },
            server: ServerFacts {
                pid: process.pid(),
                resolved_exe: process.resolved_exe().to_path_buf(),
                listen_addr: listen.socket_addr(),
                client_url: listen.client_url(),
                data_dir: server.data_dir().to_path_buf(),
                keys_dir: server.keys_dir().to_path_buf(),
            },
            module: ModuleFacts {
                wasm_sha256: artifact.sha256(),
                database_identity: artifact.database_identity(),
            },
        }
    }

    /// The published database identity, needed to connect a client to this run's database.
    pub(crate) fn database_identity(&self) -> DatabaseIdentity {
        self.module.database_identity
    }

    /// The explicit schedule seed recorded for this run, used to derive deterministic role
    /// identities.
    pub(crate) fn schedule_seed(&self) -> ScheduleSeed {
        self.schedule_seed
    }
}

/// Verified distribution provenance (both binaries + Nix store output).
#[derive(Debug, Clone, Serialize)]
struct DistributionFacts {
    nix_store_bin_dir: PathBuf,
    cli_exe: PathBuf,
    cli_version: Version,
    cli_release_commit: ReleaseCommit,
    cli_version_raw: String,
    standalone_exe: PathBuf,
    standalone_version: Version,
    standalone_version_raw: String,
}

/// Running-server facts proven before publication.
#[derive(Debug, Clone, Serialize)]
struct ServerFacts {
    pid: ServerPid,
    resolved_exe: PathBuf,
    listen_addr: SocketAddr,
    client_url: String,
    data_dir: PathBuf,
    /// Path to the ephemeral JWT key directory (auto-generated `id_ecdsa`/`id_ecdsa.pub`
    /// inside the fresh data dir). Only the path is recorded for provenance — no private
    /// key material is ever read into or serialized by the manifest.
    keys_dir: PathBuf,
}

/// Published-module facts.
#[derive(Debug, Clone, Serialize)]
struct ModuleFacts {
    wasm_sha256: WasmSha256,
    database_identity: DatabaseIdentity,
}
