//! The immutable, serialize-only run manifest snapshot.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

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
///
/// It denotes *validated immutable manifest facts*, and has two honestly-distinct origins, never a
/// live-now attestation:
///
/// - **Live capability** — [`Self::assemble`], at record time, value-copies the facts out of the still-live
///   [`VerifiedDistribution`]/[`RunningPinnedServer`] capabilities.
/// - **Fully validated frozen record** — [`Self::from_validated_parts`], at analysis time, rebuilds the exact
///   same facts from a [`ValidatedRunManifestParts`] the analysis validator constructs at the tail of its
///   stage-4 manifest checks, once the relational obligations (raw/parsed equality, campaign homogeneity)
///   have already been established by that control flow. This origin deliberately broadens the type beyond
///   live-capability assembly: a manifest so built claims neither a live-capability origin nor current
///   liveness — it adds the documented validated-frozen-record origin. The raw ingest DTOs still have no path
///   that turns them directly into a `ValidatedRunManifest`.
///
/// Neither origin is a claim the process is still alive; both denote facts already proven immutable.
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

    /// The pinned Nix-store `bin` directory both binaries resolve from.
    pub(crate) fn nix_store_bin_dir(&self) -> &Path {
        &self.distribution.nix_store_bin_dir
    }

    /// The verified CLI executable path.
    pub(crate) fn cli_exe(&self) -> &Path {
        &self.distribution.cli_exe
    }

    /// The parsed, expected-matched CLI semver version.
    pub(crate) fn cli_version(&self) -> &Version {
        &self.distribution.cli_version
    }

    /// The CLI's canonical lowercase-hex release commit.
    pub(crate) fn cli_release_commit(&self) -> ReleaseCommit {
        self.distribution.cli_release_commit
    }

    /// The CLI's raw self-reported version string, retained verbatim.
    pub(crate) fn cli_version_raw(&self) -> &str {
        &self.distribution.cli_version_raw
    }

    /// The verified standalone executable path.
    pub(crate) fn standalone_exe(&self) -> &Path {
        &self.distribution.standalone_exe
    }

    /// The parsed, expected-matched standalone semver version.
    pub(crate) fn standalone_version(&self) -> &Version {
        &self.distribution.standalone_version
    }

    /// The standalone's raw self-reported version string, retained verbatim.
    pub(crate) fn standalone_version_raw(&self) -> &str {
        &self.distribution.standalone_version_raw
    }

    /// The run's server process id.
    pub(crate) fn pid(&self) -> ServerPid {
        self.server.pid
    }

    /// The `/proc/<pid>/exe` resolved standalone executable.
    pub(crate) fn resolved_exe(&self) -> &Path {
        &self.server.resolved_exe
    }

    /// The run's parsed listen socket address (host:port).
    pub(crate) fn listen_addr(&self) -> SocketAddr {
        self.server.listen_addr
    }

    /// The run's client URL, the `http://{listen_addr}` derivation.
    pub(crate) fn client_url(&self) -> &str {
        &self.server.client_url
    }

    /// The run's fresh data directory.
    pub(crate) fn data_dir(&self) -> &Path {
        &self.server.data_dir
    }

    /// The run's ephemeral JWT keys directory (sibling of [`Self::data_dir`]).
    pub(crate) fn keys_dir(&self) -> &Path {
        &self.server.keys_dir
    }

    /// The published module's canonical-hex WASM SHA-256 artifact hash.
    pub(crate) fn wasm_sha256(&self) -> WasmSha256 {
        self.module.wasm_sha256
    }

    /// Reconstruct the immutable manifest from a frozen record's already-parsed typed parts — the
    /// analysis-time origin, dual to the live-capability [`Self::assemble`]. This is the sole non-`assemble`,
    /// non-fixture constructor, and it takes a [`ValidatedRunManifestParts`] rather than raw strings, so the
    /// untrusted ingest DTOs cannot turn themselves directly into a `ValidatedRunManifest`; only the analysis
    /// validator, which builds the parts at the tail of its stage-4 checks, reaches this path. The preregistered
    /// parameters are taken from the frozen [`PreregisteredParameters::preregistered`] source — the exact value
    /// the validator proved each recorded parameter equal to — rather than re-carried through the parts, so this
    /// snapshot cannot disagree with the frozen preregistration.
    pub(crate) fn from_validated_parts(parts: ValidatedRunManifestParts) -> Self {
        let ValidatedRunManifestParts {
            identity: ValidatedManifestIdentity { run, schedule_seed },
            distribution:
                ValidatedDistributionParts {
                    nix_store_bin_dir,
                    cli:
                        ValidatedCliFacts {
                            exe: cli_exe,
                            version: cli_version,
                            version_raw: cli_version_raw,
                            release_commit: cli_release_commit,
                        },
                    standalone:
                        ValidatedStandaloneFacts {
                            exe: standalone_exe,
                            version: standalone_version,
                            version_raw: standalone_version_raw,
                        },
                },
            server:
                ValidatedServerParts {
                    pid,
                    resolved_exe,
                    listen_addr,
                    client_url,
                    data_dir: DataDirectory(data_dir),
                    keys_dir: KeysDirectory(keys_dir),
                },
            module:
                ValidatedModuleParts {
                    wasm_sha256,
                    database_identity,
                },
        } = parts;
        Self {
            run,
            schedule_seed,
            parameters: PreregisteredParameters::preregistered(),
            distribution: DistributionFacts {
                nix_store_bin_dir,
                cli_exe,
                cli_version,
                cli_release_commit,
                cli_version_raw,
                standalone_exe,
                standalone_version,
                standalone_version_raw,
            },
            server: ServerFacts {
                pid,
                resolved_exe,
                listen_addr,
                client_url,
                data_dir,
                keys_dir,
            },
            module: ModuleFacts {
                wasm_sha256,
                database_identity,
            },
        }
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

/// The already-parsed typed parts of one recorded manifest, the sole input to
/// [`ValidatedRunManifest::from_validated_parts`].
///
/// It is colocated with [`ValidatedRunManifest`] deliberately: the conversion has to populate that type's
/// private [`DistributionFacts`]/[`ServerFacts`]/[`ModuleFacts`], so a cross-module constructor would have to
/// widen those private facts. Every part below has private fields and one aptly named `new`, so this whole
/// family is the *only* seam through which typed values become a [`ValidatedRunManifest`]; an untrusted `…Dto`
/// has no direct path.
///
/// The parts are grouped to mirror the manifest's own `distribution`/`server`/`module` structure, and each
/// group's `new` takes only distinct-typed arguments, so the CLI-vs-standalone split and the data-vs-keys path
/// roles are structurally explicit — no two same-typed values are positionally swappable. Field types make a
/// *parse-invalid* value unrepresentable; the only bare strings are the two self-reported `_raw` versions and
/// the client URL, retained verbatim exactly as the manifest stores them.
///
/// The parts do **not** by themselves witness the *relational* obligations — raw/parsed string equality,
/// expected-value matches, or campaign homogeneity — because those are properties of the validator's control
/// flow, not of the field types: trusted crate code could construct the domain values independently. The
/// analysis validator establishes those relational checks, at the tail of its stage-4 manifest pass, *before*
/// it builds these parts. This family is therefore not a proof, a capability, or an attestation of whole-stage
/// or campaign validation; it is only the typed carrier the validator hands to the conversion.
pub(crate) struct ValidatedRunManifestParts {
    identity: ValidatedManifestIdentity,
    distribution: ValidatedDistributionParts,
    server: ValidatedServerParts,
    module: ValidatedModuleParts,
}

impl ValidatedRunManifestParts {
    /// Bind one recorded manifest's four grouped parts. `pub(crate)` is the narrowest visibility that still
    /// lets the analysis validator (in `analysis::validate`) call it; the four arguments are distinct part
    /// types, so no group is positionally swappable and there is no default.
    pub(crate) fn new(
        identity: ValidatedManifestIdentity,
        distribution: ValidatedDistributionParts,
        server: ValidatedServerParts,
        module: ValidatedModuleParts,
    ) -> Self {
        Self {
            identity,
            distribution,
            server,
            module,
        }
    }
}

/// The manifest's run identity and schedule seed — two distinct typed values, so neither can stand in for the
/// other.
pub(crate) struct ValidatedManifestIdentity {
    run: RunCoordinate,
    schedule_seed: ScheduleSeed,
}

impl ValidatedManifestIdentity {
    pub(crate) fn new(run: RunCoordinate, schedule_seed: ScheduleSeed) -> Self {
        Self { run, schedule_seed }
    }
}

/// The campaign-stable distribution facts, split into the manifest's own CLI and standalone groups so a
/// CLI path/version/raw can never be swapped with the standalone's. `nix_store_bin_dir` is the sole bare path
/// at this level, distinct from the two nested groups.
pub(crate) struct ValidatedDistributionParts {
    nix_store_bin_dir: PathBuf,
    cli: ValidatedCliFacts,
    standalone: ValidatedStandaloneFacts,
}

impl ValidatedDistributionParts {
    pub(crate) fn new(
        nix_store_bin_dir: PathBuf,
        cli: ValidatedCliFacts,
        standalone: ValidatedStandaloneFacts,
    ) -> Self {
        Self {
            nix_store_bin_dir,
            cli,
            standalone,
        }
    }
}

/// The verified CLI's executable path, parsed version, raw self-reported version, and release commit — every
/// field a distinct type, so none is positionally swappable.
pub(crate) struct ValidatedCliFacts {
    exe: PathBuf,
    version: Version,
    version_raw: String,
    release_commit: ReleaseCommit,
}

impl ValidatedCliFacts {
    pub(crate) fn new(
        exe: PathBuf,
        version: Version,
        version_raw: String,
        release_commit: ReleaseCommit,
    ) -> Self {
        Self {
            exe,
            version,
            version_raw,
            release_commit,
        }
    }
}

/// The verified standalone's executable path, parsed version, and raw self-reported version — no commit, since
/// the standalone reports none. Every field is a distinct type.
pub(crate) struct ValidatedStandaloneFacts {
    exe: PathBuf,
    version: Version,
    version_raw: String,
}

impl ValidatedStandaloneFacts {
    pub(crate) fn new(exe: PathBuf, version: Version, version_raw: String) -> Self {
        Self {
            exe,
            version,
            version_raw,
        }
    }
}

/// The per-run server facts. The two same-typed directory paths carry field-role newtypes
/// ([`DataDirectory`]/[`KeysDirectory`]), so with `resolved_exe` the sole bare path, every `new` argument is a
/// distinct type and no path role is swappable.
pub(crate) struct ValidatedServerParts {
    pid: ServerPid,
    resolved_exe: PathBuf,
    listen_addr: SocketAddr,
    client_url: String,
    data_dir: DataDirectory,
    keys_dir: KeysDirectory,
}

impl ValidatedServerParts {
    pub(crate) fn new(
        pid: ServerPid,
        resolved_exe: PathBuf,
        listen_addr: SocketAddr,
        client_url: String,
        data_dir: DataDirectory,
        keys_dir: KeysDirectory,
    ) -> Self {
        Self {
            pid,
            resolved_exe,
            listen_addr,
            client_url,
            data_dir,
            keys_dir,
        }
    }
}

/// The run's fresh data directory, as a field-role newtype so it cannot be swapped with the sibling keys
/// directory.
pub(crate) struct DataDirectory(PathBuf);

impl DataDirectory {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

/// The run's ephemeral JWT keys directory, as a field-role newtype so it cannot be swapped with the sibling
/// data directory.
pub(crate) struct KeysDirectory(PathBuf);

impl KeysDirectory {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

/// The published-module facts — the WASM digest and database identity, two distinct types.
pub(crate) struct ValidatedModuleParts {
    wasm_sha256: WasmSha256,
    database_identity: DatabaseIdentity,
}

impl ValidatedModuleParts {
    pub(crate) fn new(wasm_sha256: WasmSha256, database_identity: DatabaseIdentity) -> Self {
        Self {
            wasm_sha256,
            database_identity,
        }
    }
}
