//! The actual runtime and published instance one attempt measured against.

use std::net::SocketAddr;
use std::path::PathBuf;

use semver::Version;
use serde::Serialize;

use crate::manifest::database_identity::DatabaseIdentity;
use crate::manifest::release_commit::ReleaseCommit;
use crate::manifest::server_pid::ServerPid;
use crate::manifest::verified_module_artifact::VerifiedModuleArtifact;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::provision::running_pinned_server::RunningPinnedServer;
use crate::provision::verified_distribution::VerifiedDistribution;

/// One attempt's observed provisioning facts: the runtime that ran it, the process that served it,
/// and the fresh database instance it published.
///
/// Assembled the same way [`ValidatedRunManifest`](crate::manifest::validated_run_manifest::ValidatedRunManifest)
/// assembles its own — by value-copying immutable scalars out of the still-live capabilities — but
/// without that type's `RunCoordinate`, which is minted from the historical `Cell` ontology this
/// candidate does not use. It holds no handle and no borrow, so it makes no claim the process is
/// still alive; it says only which instance produced the evidence recorded under the same attempt
/// identity.
///
/// Recorded after publish and before anything connects, so every rung of a partially-completed
/// attempt already has its runtime and instance on disk.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AttemptProvenance {
    distribution: DistributionFacts,
    server: ServerFacts,
    module: ModuleFacts,
}

/// The pinned distribution both binaries were resolved and version-verified from.
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

/// The running server proven before publication.
#[derive(Debug, Clone, Serialize)]
struct ServerFacts {
    pid: ServerPid,
    resolved_exe: PathBuf,
    listen_addr: SocketAddr,
    client_url: String,
    data_dir: PathBuf,
}

/// The published module artifact and the fresh instance it created.
#[derive(Debug, Clone, Serialize)]
struct ModuleFacts {
    wasm_sha256: WasmSha256,
    database_identity: DatabaseIdentity,
}

impl AttemptProvenance {
    /// Snapshot the immutable facts of one provisioned attempt out of its live capabilities.
    pub(crate) fn observed(
        distribution: &VerifiedDistribution,
        server: &RunningPinnedServer,
        artifact: &VerifiedModuleArtifact,
    ) -> Self {
        let cli = distribution.cli();
        let standalone = distribution.standalone();
        let process = server.process();
        let listen = server.listen();

        Self {
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
            },
            module: ModuleFacts {
                wasm_sha256: artifact.sha256(),
                database_identity: artifact.database_identity(),
            },
        }
    }
}
