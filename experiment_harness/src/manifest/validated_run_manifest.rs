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

    /// This run's schedule coordinate (cell, role, repetition block). Copied out so an observation
    /// can carry an immutable reference back to this manifest.
    pub(crate) fn run_coordinate(&self) -> RunCoordinate {
        self.run.clone()
    }

    /// The explicit schedule seed recorded for this run, used to derive deterministic role
    /// identities.
    pub(crate) fn schedule_seed(&self) -> ScheduleSeed {
        self.schedule_seed
    }

    /// A deterministic test fixture manifest for the fixture run coordinate — the fixed own-slice
    /// point-filter arm (see [`RunCoordinate::fixture`]) — under schedule seed `0`. Delegates to
    /// [`Self::fixture_for`]. Test-only: it is **not** a production construction path and mints no
    /// liveness claim.
    #[cfg(test)]
    pub(crate) fn fixture() -> Self {
        use crate::manifest::run_coordinate::RunCoordinate;

        Self::fixture_for(RunCoordinate::fixture(), ScheduleSeed::new(0))
    }

    /// A deterministic test fixture manifest for an arbitrary `run` under an explicit campaign
    /// `schedule_seed`, value-constructing every other immutable fact directly rather than reading them
    /// out of a live [`VerifiedDistribution`]/[`RunningPinnedServer`] — which would require spawning the
    /// pinned 2.6.1 standalone. Every other fact is a fixed placeholder; the run coordinate and the
    /// schedule seed are the exact scheduled values, so the manifest's provenance (and the
    /// [`ManifestReference`](crate::observation::manifest_reference::ManifestReference) derived from run +
    /// database identity + seed) matches the run the campaign actually drew at that seed. Two manifests
    /// built for the same run and seed share one reference. Test-only: it is **not** a production
    /// construction path (production manifests come solely from [`Self::assemble`] over real live
    /// capabilities) and mints no liveness claim. It exists so a run cursor's real
    /// [`write_manifest`](crate::observation::observation_sink::ObservationSink::write_manifest) binding
    /// can be exercised for any scheduled run, not only the fixture coordinate.
    #[cfg(test)]
    pub(crate) fn fixture_for(run: RunCoordinate, schedule_seed: ScheduleSeed) -> Self {
        use std::num::NonZeroU32;

        use spacetimedb_sdk::Identity;

        use crate::manifest::database_identity::DatabaseIdentity;
        use crate::manifest::server_pid::ServerPid;
        use crate::manifest::wasm_sha256::WasmSha256;

        Self {
            run,
            schedule_seed,
            parameters: PreregisteredParameters::preregistered(),
            distribution: DistributionFacts {
                nix_store_bin_dir: PathBuf::from("/fixture/nix/store/bin"),
                cli_exe: PathBuf::from("/fixture/nix/store/bin/spacetimedb-cli"),
                cli_version: Version::new(2, 6, 1),
                cli_release_commit: ReleaseCommit::parse(
                    "052c83fe984a4c4eb7bb4f9afa5c6b1903891d87",
                )
                .expect("the fixture release commit is exact lowercase hex"),
                cli_version_raw: "2.6.1".to_string(),
                standalone_exe: PathBuf::from("/fixture/nix/store/bin/spacetimedb-standalone"),
                standalone_version: Version::new(2, 6, 1),
                standalone_version_raw: "2.6.1".to_string(),
            },
            server: ServerFacts {
                pid: ServerPid::new(NonZeroU32::new(1).expect("1 is nonzero")),
                resolved_exe: PathBuf::from("/fixture/nix/store/bin/spacetimedb-standalone"),
                listen_addr: "127.0.0.1:3000"
                    .parse()
                    .expect("the fixture listen addr parses"),
                client_url: "http://127.0.0.1:3000".to_string(),
                data_dir: PathBuf::from("/fixture/data"),
                keys_dir: PathBuf::from("/fixture/keys"),
            },
            module: ModuleFacts {
                wasm_sha256: WasmSha256::new([0u8; 32]),
                database_identity: DatabaseIdentity::new(Identity::from_claims(
                    "view-read-set-experiment-fixture",
                    "fixture-database",
                )),
            },
        }
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
