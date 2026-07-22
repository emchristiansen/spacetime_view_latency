//! Untrusted mirror of the manifest's required role-identity evidence
//! (`crate::manifest::validated_run_manifest`).

use serde::Deserialize;

/// The wire form of a run's two resolved role identities: the server-issued measured identity and the
/// deterministically derived growth identity, both carried verbatim as canonical-hex strings. These are
/// *per-run* facts, never campaign-homogeneous facts — each run resolves its own measured/growth pair —
/// so `validate` parses and binds them per run rather than proving them homogeneous across the campaign.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoleIdentitiesDto {
    pub(crate) measured: String,
    pub(crate) growth: String,
}
