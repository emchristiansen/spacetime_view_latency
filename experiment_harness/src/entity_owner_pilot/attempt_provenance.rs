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
///
/// `pub(crate)` so a driver that fails *before* publication can retain the prefix of provisioning
/// facts it did establish — see
/// [`PartialProvision`](crate::control_registry_discovery_screen::partial_provision::PartialProvision).
/// Exposure only; the fields, their order, and therefore this struct's serialization are unchanged,
/// so no immutable Pilot record is reinterpreted.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct DistributionFacts {
    nix_store_bin_dir: PathBuf,
    cli_exe: PathBuf,
    cli_version: Version,
    cli_release_commit: ReleaseCommit,
    cli_version_raw: String,
    standalone_exe: PathBuf,
    standalone_version: Version,
    standalone_version_raw: String,
}

impl DistributionFacts {
    /// Snapshot the resolved, version-verified distribution.
    ///
    /// Extracted from [`AttemptProvenance::observed`], which now calls it, so the partial and
    /// complete provenance shapes are built by one code path and cannot drift into disagreeing
    /// about what a distribution fact is.
    pub(crate) fn observed(distribution: &VerifiedDistribution) -> Self {
        let cli = distribution.cli();
        let standalone = distribution.standalone();
        Self {
            nix_store_bin_dir: distribution.store_bin_dir().to_path_buf(),
            cli_exe: cli.exe().to_path_buf(),
            cli_version: cli.version().clone(),
            cli_release_commit: *cli.commit(),
            cli_version_raw: cli.raw().to_string(),
            standalone_exe: standalone.exe().to_path_buf(),
            standalone_version: standalone.version().clone(),
            standalone_version_raw: standalone.raw().to_string(),
        }
    }

    /// A representative distribution for focused tests. `#[cfg(test)]`, so production opacity is
    /// unchanged and [`Self::observed`] remains the only route from a live run.
    #[cfg(test)]
    pub(crate) fn fixture() -> Self {
        use crate::params::EXPECTED_RELEASE_COMMIT;

        Self {
            nix_store_bin_dir: PathBuf::from(FIXTURE_STORE_BIN_DIR),
            cli_exe: PathBuf::from("/nix/store/fixture-spacetimedb/bin/spacetimedb-cli"),
            cli_version: Version::new(2, 7, 0),
            cli_release_commit: ReleaseCommit::parse(EXPECTED_RELEASE_COMMIT)
                .expect("the pinned release-commit constant is canonical hex"),
            cli_version_raw: "spacetimedb-cli 2.7.0 (fixture)".to_string(),
            standalone_exe: PathBuf::from(FIXTURE_STANDALONE),
            standalone_version: Version::new(2, 7, 0),
            standalone_version_raw: "spacetimedb-standalone 2.7.0 (fixture)".to_string(),
        }
    }
}

/// Recognisably synthetic paths, so a fixture can never be mistaken for a live observation.
#[cfg(test)]
const FIXTURE_STORE_BIN_DIR: &str = "/nix/store/fixture-spacetimedb/bin";
#[cfg(test)]
const FIXTURE_STANDALONE: &str = "/nix/store/fixture-spacetimedb/bin/spacetimedb-standalone";

/// The running server proven before publication.
///
/// `pub(crate)` for the same reason as [`DistributionFacts`], and likewise unchanged in shape.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ServerFacts {
    pid: ServerPid,
    resolved_exe: PathBuf,
    listen_addr: SocketAddr,
    client_url: String,
    data_dir: PathBuf,
}

impl ServerFacts {
    /// Snapshot the `/proc`-proven server process and the instance it serves.
    pub(crate) fn observed(server: &RunningPinnedServer) -> Self {
        let process = server.process();
        let listen = server.listen();
        Self {
            pid: process.pid(),
            resolved_exe: process.resolved_exe().to_path_buf(),
            listen_addr: listen.socket_addr(),
            client_url: listen.client_url(),
            data_dir: server.data_dir().to_path_buf(),
        }
    }

    /// A representative server for focused tests, on the same `#[cfg(test)]` terms as
    /// [`DistributionFacts::fixture`].
    #[cfg(test)]
    pub(crate) fn fixture() -> Self {
        use std::num::NonZeroU32;

        Self {
            pid: ServerPid::new(NonZeroU32::new(4_242).expect("4242 is nonzero")),
            resolved_exe: PathBuf::from(FIXTURE_STANDALONE),
            listen_addr: SocketAddr::from(([127, 0, 0, 1], 3_000)),
            client_url: "http://127.0.0.1:3000".to_string(),
            data_dir: PathBuf::from("/tmp/fixture-data-dir"),
        }
    }
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
        Self {
            distribution: DistributionFacts::observed(distribution),
            server: ServerFacts::observed(server),
            module: ModuleFacts {
                wasm_sha256: artifact.sha256(),
                database_identity: artifact.database_identity(),
            },
        }
    }

    /// A representative provenance for focused tests.
    ///
    /// **`#[cfg(test)]`, so it does not exist in a production build and widens no real API.**
    /// [`Self::observed`] remains the only way to mint provenance outside tests, and it still
    /// demands live provisioning capabilities — the opacity this type has in production is
    /// unchanged.
    ///
    /// It exists because the discovery screen's record constructors take provenance *by value*.
    /// Without it, the three post-publication shapes could only be tested by asserting that each
    /// constructor calls the shared stage and disposition validators — an untested coupling that
    /// could drift the moment a constructor forgot one. With it, every failure-bearing
    /// `ScreenRecord` constructor is driven end to end.
    ///
    /// The values are recognisably synthetic and the pinned release commit and module hash are the
    /// real committed constants, so a fixture can never be mistaken for a live observation while
    /// still exercising the same parsing.
    #[cfg(test)]
    pub(crate) fn fixture() -> Self {
        use spacetimedb_sdk::Identity;

        use crate::manifest::database_identity::DatabaseIdentity;
        use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;

        Self {
            distribution: DistributionFacts::fixture(),
            server: ServerFacts::fixture(),
            module: ModuleFacts {
                wasm_sha256: WasmSha256::new(MODULE_WASM_SHA256),
                database_identity: DatabaseIdentity::new(Identity::from_byte_array([0x11; 32])),
            },
        }
    }
}
