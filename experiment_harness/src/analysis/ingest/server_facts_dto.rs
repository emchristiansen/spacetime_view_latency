//! Untrusted mirror of the manifest's private `ServerFacts`
//! (`crate::manifest::validated_run_manifest`).

use serde::Deserialize;

/// The wire form of a run's running-server facts: the process id, resolved executable, listen address,
/// client URL, and the data/keys directory paths. These do *not* all share one homogeneity class:
///
/// - `resolved_exe` is **campaign-stable**: it is `/proc/<pid>/exe` proven equal to the one verified
///   pinned `spacetimedb-standalone`, so `validate` cross-checks it against
///   [`DistributionFactsDto::standalone_exe`](super::distribution_facts_dto::DistributionFactsDto) and
///   proves it homogeneous across the campaign.
/// - `pid`, `listen_addr`, `client_url`, `data_dir`, and `keys_dir` are **per-run** process
///   instance/location facts (a fresh isolated server, bound port, and temp tree each run), never
///   compared across runs. `validate` checks them for internal consistency and shape only: the pid is
///   nonzero, the address parses, `client_url` equals `http://{listen_addr}` (the derivation used at
///   construction), and `data_dir`/`keys_dir` are the sibling `data`/`keys` children of one temp root.
///
/// No key material is ever read — only the recorded key-directory path is present. `validate` must not
/// compare the whole `ServerFactsDto` block across runs.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ServerFactsDto {
    pub(crate) pid: u32,
    pub(crate) resolved_exe: String,
    pub(crate) listen_addr: String,
    pub(crate) client_url: String,
    pub(crate) data_dir: String,
    pub(crate) keys_dir: String,
}
