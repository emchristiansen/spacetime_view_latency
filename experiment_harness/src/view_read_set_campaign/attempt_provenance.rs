//! The actual runtime and published instance one attempt measured against.
//!
//! **Module topology is the enforcement mechanism here, not a comment.** The record and its three
//! fact groups live together in the private, *childless* inline module [`sealed`]. Rust makes a
//! private field visible to its declaring module **and every descendant**, so declaring them beside a
//! `#[cfg(test)] mod tests` child — or any child added later — would let that module write the struct
//! literals directly and record a PID, data directory, or module digest that no provisioned server
//! ever produced. `observed` copies those scalars out of live capabilities, so going through it is
//! the only thing tying the record to a real instance at all; downstream code then treats the value
//! semantically, admitting evidence against the campaign pins on its strength.
//!
//! `DistributionFacts`, `ServerFacts`, and `ModuleFacts` are sealed *with* the record rather than
//! beside it, and none is re-exported: they are its private payload, and only it names them.

mod sealed {
    use std::net::SocketAddr;
    use std::path::PathBuf;

    use anyhow::Result;
    use semver::Version;
    use serde::Serialize;

    use crate::manifest::database_identity::DatabaseIdentity;
    use crate::manifest::release_commit::ReleaseCommit;
    use crate::manifest::server_pid::ServerPid;
    use crate::manifest::verified_module_artifact::VerifiedModuleArtifact;
    use crate::manifest::wasm_sha256::WasmSha256;
    use crate::provision::running_pinned_server::RunningPinnedServer;
    use crate::provision::verified_distribution::VerifiedDistribution;
    use crate::view_read_set_campaign::campaign_params::CONFIRMED_READS;
    use crate::view_read_set_campaign::campaign_provenance::CampaignProvenance;

    /// One attempt's observed provisioning facts: the runtime that ran it, the process that served
    /// it, the fresh database instance it published, and the confirmed-read setting its measurement
    /// runs under.
    ///
    /// Copies [`AttemptProvenance`](crate::entity_owner_pilot::attempt_provenance::AttemptProvenance),
    /// the completed Pilot's same-named record, rather than being shared with it: that type is the
    /// shape of evidence already on disk under a different campaign's vocabulary, and adding a field
    /// to it would change how those lines deserialize. The assembly is identical — value-copying
    /// immutable scalars out of the still-live capabilities, exactly as
    /// [`ValidatedRunManifest`](crate::manifest::validated_run_manifest::ValidatedRunManifest) does.
    /// It holds no handle and no borrow, so it makes no claim the process is still alive; it says
    /// only which instance produced the evidence recorded under the same attempt identity.
    ///
    /// **Who can construct it.** Every field is private to this childless module and
    /// [`Self::observed`] is the only constructor. The copying is what ties the record to a live
    /// distribution, server, and published artifact, so confining it here is what stops a value that
    /// merely *looks* like provisioning provenance from being admitted against the campaign pins.
    ///
    /// **`confirmed_reads` is recorded, and its limits are these.** The spec requires every new
    /// provenance record to carry it explicitly so a ledger-only check can verify it without
    /// inferring it from build provenance. It is read from [`CONFIRMED_READS`], which is the same
    /// constant
    /// [`ConnectedClient::connect`](crate::client::connected_client::ConnectedClient::connect)
    /// passes to the SDK's `with_confirmed_reads` — so this is *what the harness applies*, not an
    /// independent read-back of what the connection negotiated, which the SDK builder exposes no way
    /// to obtain. It therefore cannot disagree with
    /// [`CampaignParameters`](crate::view_read_set_campaign::campaign_parameters::CampaignParameters)'s
    /// copy, and a check that compared the two would be checking a constant against itself.
    ///
    /// Recorded after publish and before anything connects, so an attempt that dies mid-measurement
    /// still has its runtime and instance on disk. That ordering is also why the setting here is the
    /// constant and could not be anything else: no client exists yet at the moment this is written.
    #[derive(Debug, Clone, Serialize)]
    pub(crate) struct AttemptProvenance {
        distribution: DistributionFacts,
        server: ServerFacts,
        module: ModuleFacts,
        confirmed_reads: bool,
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
                confirmed_reads: CONFIRMED_READS,
            }
        }

        /// Check this attempt's observed runtime, module, and confirmed-read facts against the
        /// campaign's pins, failing loud on any disagreement.
        ///
        /// Takes the whole [`CampaignProvenance`] rather than four loose expected values, so a caller
        /// cannot check an attempt against one campaign's version and another's module digest.
        ///
        /// **Which clauses are real observations, and which is not.** The CLI and standalone
        /// versions, the CLI release commit, and the module digest were read off the live
        /// distribution and the published artifact, so those four can genuinely disagree with the
        /// pins and are the substance of the gate. `confirmed_reads` cannot disagree *within one
        /// process*, since both records read [`CONFIRMED_READS`]; what it catches is a ledger
        /// assembled from lines written by two different builds, which is exactly the case a
        /// ledger-only reader has no other way to detect.
        ///
        /// **Phase 1 boundary.** This is a campaign-wide admission gate, not a local newtype
        /// constructor, so it is stubbed exactly as the retry and selection rules are. What is fixed
        /// here is the signature — one whole [`CampaignProvenance`], so the comparison cannot be run
        /// against a mixture of pins.
        pub(crate) fn agrees_with(&self, campaign: &CampaignProvenance) -> Result<()> {
            let _ = campaign;
            todo!(
                "Phase 2: require this attempt's observed CLI version and standalone version to \
                 equal CampaignProvenance::expected_version, its CLI release commit to equal \
                 expected_release_commit, its published module digest to equal \
                 expected_module_wasm_sha256, and its confirmed_reads to equal the campaign \
                 parameters' — failing loud, naming both sides, on any disagreement"
            )
        }
    }
}

pub(crate) use sealed::AttemptProvenance;
