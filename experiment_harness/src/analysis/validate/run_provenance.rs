//! The run-varying manifest provenance retained on each trusted run.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use crate::manifest::database_identity::DatabaseIdentity;
use crate::manifest::server_pid::ServerPid;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;

/// The per-run server/module provenance that varies across the campaign's runs (spec: "Each `TrustedRun`
/// owns its run-varying database identity and server facts (PID, listen/client addresses, data directory,
/// and keys directory)"). Each run spawns its own pinned standalone process and publishes its own
/// database, so these facts are proven only for internal consistency/shape per run — never homogeneity —
/// and remain attached to the arm/control run they describe, unlike the campaign-homogeneous
/// [`CampaignProvenance`](super::campaign_provenance::CampaignProvenance) retained once at the root.
///
/// Every value is the *already-parsed typed* domain value: the database identity and server PID are their
/// domain newtypes ([`DatabaseIdentity`]/[`ServerPid`]), the listen address is a [`SocketAddr`], and the
/// data/keys directories are [`PathBuf`]s. Projected from the typed trusted [`ValidatedRunManifest`] bound
/// to the run after manifest validation, never from the untrusted pre-validation DTO's strings. The
/// client URL is retained as a `String` because it *is* the run's self-reported URL text, validated to be
/// the `http://{listen_addr}` derivation of the parsed listen address.
///
/// Fields are private with no defaults; [`Self::mint`] is `pub(super)`, so a `RunProvenance` is assembled
/// only from within the `validate` subtree — in production only by the single validation pass. This is
/// trusted-graph content, not a report DTO: it is not `Serialize`; the report projects it into its own
/// [`RunProvenanceReport`](crate::analysis::report::run_provenance_report::RunProvenanceReport).
#[derive(Debug, Clone)]
pub(crate) struct RunProvenance {
    /// The run's published database identity, needed to connect a client to this run's database.
    database_identity: DatabaseIdentity,
    /// The run's server process id.
    pid: ServerPid,
    /// The run's parsed listen socket address (host:port).
    listen_addr: SocketAddr,
    /// The run's client URL, validated to be the `http://{listen_addr}` derivation.
    client_url: String,
    /// The run's fresh data directory.
    data_dir: PathBuf,
    /// The run's ephemeral JWT keys directory (sibling of [`Self::data_dir`]).
    keys_dir: PathBuf,
}

impl RunProvenance {
    /// Mint the run-varying provenance from the typed trusted [`ValidatedRunManifest`] bound to this run,
    /// reusing the domain values it already carries rather than reparsing pre-validation DTO strings.
    /// `pub(super)` so only the `validate` subtree's pass can construct one.
    pub(super) fn mint(manifest: &ValidatedRunManifest) -> Self {
        let _ = manifest;
        todo!("Phase 2: project the per-run server/module facts into typed domain values")
    }

    /// The run's published database identity.
    pub(crate) fn database_identity(&self) -> DatabaseIdentity {
        self.database_identity
    }

    /// The run's server process id.
    pub(crate) fn pid(&self) -> ServerPid {
        self.pid
    }

    /// The run's parsed listen socket address.
    pub(crate) fn listen_addr(&self) -> SocketAddr {
        self.listen_addr
    }

    /// The run's client URL, the `http://{listen_addr}` derivation.
    pub(crate) fn client_url(&self) -> &str {
        &self.client_url
    }

    /// The run's fresh data directory.
    pub(crate) fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// The run's ephemeral JWT keys directory.
    pub(crate) fn keys_dir(&self) -> &Path {
        &self.keys_dir
    }
}
