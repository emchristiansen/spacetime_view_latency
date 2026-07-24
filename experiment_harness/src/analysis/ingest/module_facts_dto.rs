//! Untrusted mirror of the manifest's private `ModuleFacts`
//! (`crate::manifest::validated_run_manifest`).

use serde::Deserialize;

/// The wire form of a run's published-module facts: the module WASM SHA-256 and the server-issued
/// database identity, both carried verbatim as canonical-hex strings. These two fields partition by
/// how `validate` treats them: the WASM SHA-256 is a *campaign-stable* artifact hash — every run
/// publishes the one verified module — so it is proven homogeneous across the campaign; the database
/// identity is a *per-run* fact — each run provisions a fresh server and publishes anonymously — so it
/// is never compared across runs, only parsed and proven to bind this run's observations to this
/// manifest by reference equality. `validate` must not compare the whole `ModuleFactsDto` block.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ModuleFactsDto {
    pub(crate) wasm_sha256: String,
    pub(crate) database_identity: String,
}
