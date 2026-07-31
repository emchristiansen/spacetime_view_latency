//! The complete frozen artifact identity every attempt runs against.

use anyhow::{Context, Result};
use semver::Version;
use serde::Serialize;

use crate::control_registry_discovery_screen::candidate_version::{
    CandidateVersion, CONTROL_REGISTRY_DISCOVERY_VERSION,
};
use crate::control_registry_discovery_screen::generated_tree_digest::GeneratedTreeDigest;
use crate::control_registry_discovery_screen::screen_params::DISTRIBUTION_RELEASE_TAG;
use crate::manifest::release_commit::ReleaseCommit;
use crate::manifest::wasm_sha256::WasmSha256;
use crate::module_artifact::module_wasm_sha256::MODULE_WASM_SHA256;
use crate::params::{EXPECTED_RELEASE_COMMIT, EXPECTED_VERSION};

/// Everything the freeze pins an attempt's artifacts to, resolved once and written on every record.
///
/// Complete by construction: there is no constructor that omits a field, so a record cannot carry a
/// module hash while silently dropping the release commit or the generated-tree digest. That
/// totality is the point — a partially-identified artifact is the shape in which two runs of
/// different code get compared as though they were one.
///
/// These are the *expected* pinned values, known before anything is provisioned. The
/// correspondingly *observed* runtime facts are a separate record, carried only by an attempt that
/// actually ran.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct PinnedArtifactIdentity {
    distribution_release_tag: &'static str,
    expected_version: Version,
    expected_release_commit: ReleaseCommit,
    expected_module_wasm_sha256: WasmSha256,
    generated_tree: GeneratedTreeDigest,
    candidate_version: CandidateVersion,
}

impl PinnedArtifactIdentity {
    /// Resolve the frozen identity, failing loud if any pinned constant is malformed.
    pub(crate) fn frozen() -> Result<Self> {
        let expected_version = Version::parse(EXPECTED_VERSION)
            .with_context(|| format!("EXPECTED_VERSION is not semver: {EXPECTED_VERSION:?}"))?;
        let expected_release_commit = ReleaseCommit::parse(EXPECTED_RELEASE_COMMIT)
            .context("EXPECTED_RELEASE_COMMIT is not a canonical hex commit")?;
        Ok(Self {
            distribution_release_tag: DISTRIBUTION_RELEASE_TAG,
            expected_version,
            expected_release_commit,
            expected_module_wasm_sha256: WasmSha256::new(MODULE_WASM_SHA256),
            generated_tree: GeneratedTreeDigest::accepted()?,
            candidate_version: CONTROL_REGISTRY_DISCOVERY_VERSION,
        })
    }
}
