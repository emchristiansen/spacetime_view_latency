//! Untrusted mirror of the manifest's private `DistributionFacts`
//! (`crate::manifest::validated_run_manifest`).

use serde::Deserialize;

/// The wire form of a run's verified distribution provenance: both binaries' store paths, versions,
/// and the CLI's release commit. Every path, version, and commit is carried verbatim as a string.
/// These are all *campaign-stable* facts — every run resolves the one pinned, verified Nix-store
/// distribution — so `validate` parses the semver versions and lowercase-hex commit, cross-checks them
/// against the expected release, and proves them homogeneous across the campaign. The homogeneity
/// partition is field-specific and derived from provisioning, not blind whole-block equality.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DistributionFactsDto {
    pub(crate) nix_store_bin_dir: String,
    pub(crate) cli_exe: String,
    pub(crate) cli_version: String,
    pub(crate) cli_release_commit: String,
    pub(crate) cli_version_raw: String,
    pub(crate) standalone_exe: String,
    pub(crate) standalone_version: String,
    pub(crate) standalone_version_raw: String,
}
